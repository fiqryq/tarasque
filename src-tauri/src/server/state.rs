use crate::models::Collection;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct AppState {
    pub collection: Arc<RwLock<Collection>>,
    pub app_handle: tauri::AppHandle,
}

#[derive(Default)]
pub struct ServerHandle(pub Mutex<Option<oneshot::Sender<()>>>);
