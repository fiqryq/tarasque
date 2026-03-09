use crate::models::AppData;
use tauri::Manager;

const DATA_FILE: &str = "collections.json";

pub async fn load(app: &tauri::AppHandle) -> Result<AppData, String> {
    let path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join(DATA_FILE);

    if !path.exists() {
        return Ok(AppData::default());
    }

    let raw = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Read error: {e}"))?;

    serde_json::from_str(&raw).map_err(|e| format!("Parse error: {e}"))
}

pub async fn save(app: &tauri::AppHandle, data: &AppData) -> Result<(), String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?;

    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("Create dir error: {e}"))?;

    let path = dir.join(DATA_FILE);
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Serialize error: {e}"))?;

    tokio::fs::write(&path, json)
        .await
        .map_err(|e| format!("Write error: {e}"))
}
