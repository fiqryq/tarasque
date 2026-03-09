pub mod router;
pub mod state;

use crate::models::{Collection, LogEntry};
use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use state::{AppState, ServerHandle};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{oneshot, RwLock};

pub async fn start(
    collection: Collection,
    app_handle: tauri::AppHandle,
    server_handle: &ServerHandle,
) -> Result<(), String> {
    let port = collection.port;
    let state_data = web::Data::new(AppState {
        collection: Arc::new(RwLock::new(collection)),
        app_handle: app_handle.clone(),
    });

    let (tx, rx) = oneshot::channel::<()>();

    // Bind BEFORE spawning so errors propagate to the command
    let server = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(state_data.clone())
            .route("/{tail:.*}", web::get().to(mock_handler))
            .route("/{tail:.*}", web::post().to(mock_handler))
            .route("/{tail:.*}", web::put().to(mock_handler))
            .route("/{tail:.*}", web::delete().to(mock_handler))
            .route("/{tail:.*}", web::patch().to(mock_handler))
    })
    .bind(format!("127.0.0.1:{port}"))
    .map_err(|e| format!("Bind error: {e}"))?
    .run();

    // Store shutdown sender only after successful bind
    *server_handle.0.lock().await = Some(tx);

    let handle = server.handle();

    // Spawn shutdown listener
    tauri::async_runtime::spawn(async move {
        let _ = rx.await;
        handle.stop(true).await;
    });

    // Spawn server runner
    tauri::async_runtime::spawn(async move {
        let _ = server.await;
    });

    Ok(())
}

pub async fn stop(server_handle: &ServerHandle) -> Result<(), String> {
    let mut guard = server_handle.0.lock().await;
    if let Some(tx) = guard.take() {
        let _ = tx.send(());
    }
    Ok(())
}

async fn mock_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl actix_web::Responder {
    let path = req.path().to_string();
    let method = req.method().as_str().to_string();
    let start = std::time::Instant::now();

    let collection = state.collection.read().await;

    for endpoint in &collection.endpoints {
        if endpoint.method.to_uppercase() != method.to_uppercase() {
            continue;
        }

        if let Some(params) = router::match_path(&endpoint.path, &path) {
            let scenario = endpoint
                .scenarios
                .iter()
                .find(|s| s.id == endpoint.active_scenario_id)
                .or_else(|| endpoint.scenarios.first());

            if let Some(scenario) = scenario {
                if scenario.delay_ms > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(scenario.delay_ms)).await;
                }

                let duration = start.elapsed().as_millis() as u64;
                let body = router::substitute_params(&scenario.body, &params);

                let log = LogEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    method: method.clone(),
                    path: path.clone(),
                    status: scenario.status,
                    duration,
                };
                let _ = state.app_handle.emit_to("main", "server:log", &log);

                let mut response = HttpResponse::build(
                    actix_web::http::StatusCode::from_u16(scenario.status)
                        .unwrap_or(actix_web::http::StatusCode::OK),
                );

                for (key, value) in &scenario.headers {
                    response.insert_header((key.as_str(), value.as_str()));
                }

                return response.body(body);
            }
        }
    }

    let duration = start.elapsed().as_millis() as u64;
    let log = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        method,
        path,
        status: 404,
        duration,
    };
    let _ = state.app_handle.emit_to("main", "server:log", &log);

    HttpResponse::NotFound().json(serde_json::json!({ "error": "No matching endpoint found" }))
}
