use crate::models::Collection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct AppState {
    pub collection: Arc<RwLock<Collection>>,
    pub app_handle: tauri::AppHandle,
}

/// Per-collection shutdown handles.
pub struct CollectionHandle {
    pub http:  oneshot::Sender<()>,
    pub https: Option<oneshot::Sender<()>>,
}

/// Global registry of all running collection servers.
#[derive(Default)]
pub struct MultiServerHandle {
    pub handles: Mutex<HashMap<String, CollectionHandle>>,
}
