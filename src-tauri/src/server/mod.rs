pub mod router;
pub mod state;
pub mod template;

use crate::models::{Collection, LogEntry};
use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use state::{AppState, CollectionHandle, MultiServerHandle};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{oneshot, RwLock};

/// Headers that must NOT be forwarded to an upstream proxy target.
const HOP_BY_HOP: &[&str] = &[
    "host",
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
    "content-length", // let reqwest set this from the body
];

pub async fn start(
    collection: Collection,
    app_handle: tauri::AppHandle,
    multi: &MultiServerHandle,
) -> Result<(), String> {
    let collection_id = collection.id.clone();
    let http_port = collection.port;

    // Stop existing server for this collection if any
    {
        let mut handles = multi.handles.lock().await;
        if let Some(old) = handles.remove(&collection_id) {
            let _ = old.http.send(());
            if let Some(h) = old.https { let _ = h.send(()); }
        }
    }

    let mut https_shutdown: Option<oneshot::Sender<()>> = None;

    if collection.alias_enabled {
        let (startup_tx, startup_rx) = oneshot::channel::<Result<oneshot::Sender<()>, String>>();
        spawn_https_server(collection.clone(), app_handle.clone(), startup_tx);
        match tokio::time::timeout(std::time::Duration::from_secs(10), startup_rx).await {
            Ok(Ok(Ok(shutdown_tx))) => { https_shutdown = Some(shutdown_tx); }
            Ok(Ok(Err(e))) => return Err(e),
            Ok(Err(_)) => return Err("HTTPS server thread crashed during startup".to_string()),
            Err(_)     => return Err("HTTPS server timed out — port may be in use".to_string()),
        }
    }

    let (startup_tx, startup_rx) = oneshot::channel::<Result<oneshot::Sender<()>, String>>();
    spawn_http_server(http_port, collection, app_handle, startup_tx);

    match tokio::time::timeout(std::time::Duration::from_secs(10), startup_rx).await {
        Ok(Ok(Ok(shutdown_tx))) => {
            let mut handles = multi.handles.lock().await;
            handles.insert(collection_id, CollectionHandle {
                http: shutdown_tx,
                https: https_shutdown,
            });
            Ok(())
        }
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(_)) => Err("HTTP server thread crashed during startup".to_string()),
        Err(_)     => Err("HTTP server timed out — port may already be in use".to_string()),
    }
}

/// Spawns an actix-web HTTP server on its own OS thread with a dedicated actix-rt runtime.
/// Sends `Ok(shutdown_sender)` on successful bind or `Err(message)` on failure.
fn spawn_http_server(
    port: u16,
    collection: Collection,
    app_handle: tauri::AppHandle,
    startup_tx: oneshot::Sender<Result<oneshot::Sender<()>, String>>,
) {
    std::thread::spawn(move || {
        actix_web::rt::System::new().block_on(async move {
            let state_data = web::Data::new(AppState {
                collection: Arc::new(RwLock::new(collection)),
                app_handle,
            });

            let bind_result = HttpServer::new(move || {
                App::new()
                    .wrap(Cors::permissive())
                    .app_data(state_data.clone())
                    .route("/{tail:.*}", web::get().to(mock_handler))
                    .route("/{tail:.*}", web::post().to(mock_handler))
                    .route("/{tail:.*}", web::put().to(mock_handler))
                    .route("/{tail:.*}", web::delete().to(mock_handler))
                    .route("/{tail:.*}", web::patch().to(mock_handler))
                    .route("/{tail:.*}", web::method(actix_web::http::Method::HEAD).to(mock_handler))
                    .route("/{tail:.*}", web::method(actix_web::http::Method::OPTIONS).to(mock_handler))
            })
            .bind(format!("127.0.0.1:{port}"));

            match bind_result {
                Err(e) => {
                    let _ = startup_tx.send(Err(format!("Bind error on port {port}: {e}")));
                }
                Ok(http_server) => {
                    let server = http_server.run();
                    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
                    let handle = server.handle();

                    // Report success and hand back shutdown sender
                    let _ = startup_tx.send(Ok(shutdown_tx));

                    actix_web::rt::spawn(async move {
                        let _ = shutdown_rx.await;
                        handle.stop(true).await;
                    });

                    let _ = server.await;
                }
            }
        });
    });
}

/// Spawns an actix-web HTTPS server on its own OS thread.
fn spawn_https_server(
    collection: Collection,
    app_handle: tauri::AppHandle,
    startup_tx: oneshot::Sender<Result<oneshot::Sender<()>, String>>,
) {
    use rustls::ServerConfig;
    use std::io::BufReader;

    let alias_port = collection.alias_port.unwrap_or(8443);
    let cert_pem = match collection.alias_cert_pem.clone() {
        Some(p) if !p.is_empty() => p,
        _ => {
            let _ = startup_tx.send(Err(
                "No certificate found. Generate one first in Collection Settings.".to_string(),
            ));
            return;
        }
    };
    let key_pem = match collection.alias_key_pem.clone() {
        Some(k) if !k.is_empty() => k,
        _ => {
            let _ = startup_tx.send(Err("No private key found.".to_string()));
            return;
        }
    };

    // Parse TLS material synchronously before spawning the thread (better error messages)
    let certs: Vec<rustls::pki_types::CertificateDer<'static>> =
        match rustls_pemfile::certs(&mut BufReader::new(cert_pem.as_bytes()))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = startup_tx.send(Err(format!("Invalid certificate PEM: {e}")));
                return;
            }
        };

    let key = match rustls_pemfile::private_key(&mut BufReader::new(key_pem.as_bytes())) {
        Ok(Some(k)) => k,
        Ok(None) => {
            let _ = startup_tx.send(Err("No private key found in PEM".to_string()));
            return;
        }
        Err(e) => {
            let _ = startup_tx.send(Err(format!("Invalid key PEM: {e}")));
            return;
        }
    };

    let tls_config = match ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
    {
        Ok(c) => c,
        Err(e) => {
            let _ = startup_tx.send(Err(format!("TLS config error: {e}")));
            return;
        }
    };

    std::thread::spawn(move || {
        actix_web::rt::System::new().block_on(async move {
            let state_data = web::Data::new(AppState {
                collection: Arc::new(RwLock::new(collection)),
                app_handle,
            });

            let bind_result = HttpServer::new(move || {
                App::new()
                    .wrap(Cors::permissive())
                    .app_data(state_data.clone())
                    .route("/{tail:.*}", web::get().to(mock_handler))
                    .route("/{tail:.*}", web::post().to(mock_handler))
                    .route("/{tail:.*}", web::put().to(mock_handler))
                    .route("/{tail:.*}", web::delete().to(mock_handler))
                    .route("/{tail:.*}", web::patch().to(mock_handler))
                    .route("/{tail:.*}", web::method(actix_web::http::Method::HEAD).to(mock_handler))
                    .route("/{tail:.*}", web::method(actix_web::http::Method::OPTIONS).to(mock_handler))
            })
            .bind_rustls_0_23(format!("127.0.0.1:{alias_port}"), tls_config);

            match bind_result {
                Err(e) => {
                    let _ =
                        startup_tx.send(Err(format!("HTTPS bind error on port {alias_port}: {e}")));
                }
                Ok(https_server) => {
                    let server = https_server.run();
                    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
                    let handle = server.handle();

                    let _ = startup_tx.send(Ok(shutdown_tx));

                    actix_web::rt::spawn(async move {
                        let _ = shutdown_rx.await;
                        handle.stop(true).await;
                    });

                    let _ = server.await;
                }
            }
        });
    });
}

pub async fn stop(collection_id: &str, multi: &MultiServerHandle) -> Result<(), String> {
    let mut handles = multi.handles.lock().await;
    if let Some(handle) = handles.remove(collection_id) {
        let _ = handle.http.send(());
        if let Some(h) = handle.https { let _ = h.send(()); }
        Ok(())
    } else {
        Err(format!("No running server for collection '{collection_id}'"))
    }
}

pub async fn is_running(collection_id: &str, multi: &MultiServerHandle) -> bool {
    multi.handles.lock().await.contains_key(collection_id)
}

pub async fn running_ids(multi: &MultiServerHandle) -> Vec<String> {
    multi.handles.lock().await.keys().cloned().collect()
}

async fn mock_handler(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> impl actix_web::Responder {
    let path = req.path().to_string();
    let method = req.method().as_str().to_string();
    let query_string = req.query_string().to_string();
    let start = std::time::Instant::now();

    let query_params: HashMap<String, String> = query_string
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Some((
                    key.to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string(),
                ))
            } else {
                None
            }
        })
        .collect();

    let headers: HashMap<String, String> = req
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body_str = String::from_utf8_lossy(&body).to_string();
    let body_json: Option<serde_json::Value> = serde_json::from_str(&body_str).ok();

    let collection = state.collection.read().await;

    // --- Try to match a mock endpoint ---
    // For HEAD requests: try HEAD endpoints first, then fall back to matching GET endpoints.
    let match_methods: Vec<&str> = if method.to_uppercase() == "HEAD" {
        vec!["HEAD", "GET"]
    } else {
        vec![method.as_str()]
    };

    'outer: for match_method in &match_methods {
        for endpoint in &collection.endpoints {
            if endpoint.method.to_uppercase() != match_method.to_uppercase() {
                continue;
            }

            if let Some(route_params) = router::match_path(&endpoint.path, &path) {
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
                    let body_rendered = template::TemplateEngine::render(
                        &router::substitute_params(&scenario.body, &route_params),
                        &route_params,
                        &query_params,
                        &headers,
                        body_json.as_ref(),
                    );

                    let log = LogEntry {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        method: method.clone(),
                        path: path.clone(),
                        status: scenario.status,
                        duration,
                        query: query_params.clone(),
                        headers: headers.clone(),
                        body: body_str.clone(),
                        response_body: Some(body_rendered.clone()),
                        proxied: Some(false),
                    };
                    let _ = state.app_handle.emit_to("main", "server:log", &log);

                    let mut response = HttpResponse::build(
                        actix_web::http::StatusCode::from_u16(scenario.status)
                            .unwrap_or(actix_web::http::StatusCode::OK),
                    );
                    for (key, value) in &scenario.headers {
                        response.insert_header((key.as_str(), value.as_str()));
                    }
                    // HEAD: same status/headers but empty body
                    if method.to_uppercase() == "HEAD" {
                        return response.finish();
                    }
                    return response.body(body_rendered);
                }
                break 'outer;
            }
        }
    }

    // --- No matching endpoint — try proxy forwarding ---
    if collection.proxy_enabled {
        if let Some(proxy_url) = &collection.proxy_url {
            // Build target URL including query string
            let target_url = if query_string.is_empty() {
                format!("{}{}", proxy_url.trim_end_matches('/'), path)
            } else {
                format!("{}{}?{}", proxy_url.trim_end_matches('/'), path, query_string)
            };

            // Build reqwest client — optionally skip TLS verification
            let client = if collection.proxy_insecure {
                reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap_or_default()
            } else {
                reqwest::Client::new()
            };

            let mut request = client.request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                &target_url,
            );

            // Forward headers — skip hop-by-hop
            for (key, value) in &headers {
                if !HOP_BY_HOP.contains(&key.to_lowercase().as_str()) {
                    request = request.header(key.as_str(), value.as_str());
                }
            }

            if !body_str.is_empty() {
                request = request.body(body_str.clone());
            }

            match request.send().await {
                Ok(resp) => {
                    let duration = start.elapsed().as_millis() as u64;
                    let status = resp.status().as_u16();
                    let resp_headers: Vec<(String, String)> = resp
                        .headers()
                        .iter()
                        .filter(|(k, _)| !HOP_BY_HOP.contains(&k.as_str()))
                        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                        .collect();
                    let response_body = resp.text().await.unwrap_or_default();

                    let log = LogEntry {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        method: method.clone(),
                        path: path.clone(),
                        status,
                        duration,
                        query: query_params,
                        headers: headers.clone(),
                        body: body_str.clone(),
                        response_body: Some(response_body.clone()),
                        proxied: Some(true),
                    };
                    let _ = state.app_handle.emit_to("main", "server:log", &log);

                    let mut builder = HttpResponse::build(
                        actix_web::http::StatusCode::from_u16(status)
                            .unwrap_or(actix_web::http::StatusCode::OK),
                    );
                    for (key, value) in resp_headers {
                        builder.insert_header((key.as_str(), value.as_str()));
                    }
                    return builder.body(response_body);
                }
                Err(e) => {
                    let duration = start.elapsed().as_millis() as u64;
                    let log = LogEntry {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        method: method.clone(),
                        path: path.clone(),
                        status: 502,
                        duration,
                        query: query_params,
                        headers,
                        body: body_str,
                        response_body: Some(format!("Proxy error: {e}")),
                        proxied: Some(true),
                    };
                    let _ = state.app_handle.emit_to("main", "server:log", &log);
                    return HttpResponse::BadGateway()
                        .json(serde_json::json!({ "error": format!("Proxy error: {e}") }));
                }
            }
        }
    }

    // --- OPTIONS fallback: no matching endpoint configured, return 204 with Allow header ---
    if method.to_uppercase() == "OPTIONS" {
        let duration = start.elapsed().as_millis() as u64;
        let log = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            method: method.clone(),
            path: path.clone(),
            status: 204,
            duration,
            query: query_params,
            headers,
            body: body_str,
            response_body: None,
            proxied: Some(false),
        };
        let _ = state.app_handle.emit_to("main", "server:log", &log);
        return HttpResponse::NoContent()
            .insert_header(("Allow", "GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS"))
            .finish();
    }

    // --- 404 ---
    let duration = start.elapsed().as_millis() as u64;
    let log = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        method,
        path,
        status: 404,
        duration,
        query: query_params,
        headers,
        body: body_str,
        response_body: None,
        proxied: Some(false),
    };
    let _ = state.app_handle.emit_to("main", "server:log", &log);
    HttpResponse::NotFound().json(serde_json::json!({ "error": "No matching endpoint found" }))
}
