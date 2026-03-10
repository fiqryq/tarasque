use crate::models::Collection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::sync::oneshot;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EndpointStats {
    pub count: u64,
    pub error_count: u64,
    pub total_duration: u64,
    /// Capped at 1000 entries for p95 calculation.
    pub durations: Vec<u64>,
    pub last_hit: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub collection: Arc<RwLock<Collection>>,
    pub app_handle: tauri::AppHandle,
    /// Per-server key-value store. Cleared on server restart.
    pub store: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Per-endpoint request statistics.
    pub stats: Arc<RwLock<HashMap<String, EndpointStats>>>,
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
