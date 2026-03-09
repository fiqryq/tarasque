use crate::models::AppData;
use crate::server::state::ServerHandle;
use crate::{server, storage};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn load_data(app: AppHandle) -> Result<AppData, String> {
    storage::load(&app).await
}

#[tauri::command]
pub async fn save_data(app: AppHandle, data: AppData) -> Result<(), String> {
    storage::save(&app, &data).await
}

#[tauri::command]
pub async fn start_server(
    app: AppHandle,
    handle: State<'_, ServerHandle>,
    collection_id: String,
    data: AppData,
) -> Result<(), String> {
    // Persist current state immediately (bypasses debounce)
    storage::save(&app, &data).await?;

    let collection = data
        .collections
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| format!("Collection '{collection_id}' not found"))?;

    server::start(collection, app.clone(), &handle).await
}

#[tauri::command]
pub async fn stop_server(handle: State<'_, ServerHandle>) -> Result<(), String> {
    server::stop(&handle).await
}
