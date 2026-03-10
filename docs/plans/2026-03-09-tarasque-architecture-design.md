# Tarasque Architecture Design
**Date:** 2026-03-09
**Status:** Approved

## Overview

Desktop mock API server (like Mockoon/Postman). Create collections → define endpoints → start localhost server → test with curl/browser.

**Tech Stack:** Tauri v2 + Rust + Actix-web + Svelte 5 + Tailwind CSS + Monaco Editor

---

## Data Models

### Rust (`src-tauri/src/models.rs`)

```rust
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub status: u16,
    pub delay_ms: u64,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct Endpoint {
    pub id: String,
    pub method: String,           // GET | POST | PUT | DELETE | PATCH
    pub path: String,             // e.g. /users/:id
    pub scenarios: Vec<Scenario>,
    pub active_scenario_id: String,
}

pub struct Collection {
    pub id: String,
    pub name: String,
    pub port: u16,                // per-collection port
    pub endpoints: Vec<Endpoint>,
}

pub struct AppData {
    pub collections: Vec<Collection>,
    pub active_collection_id: Option<String>,
}
```

**Key decision:** `port` is per-collection, not global.

### TypeScript (`src/lib/types.ts`)
Mirror of Rust types — identical shape for zero-friction serde.

---

## Folder Structure

```
tarasque/
├── src-tauri/
│   └── src/
│       ├── main.rs           # Tauri setup only (~20 lines)
│       ├── commands.rs       # All #[tauri::command] — thin wrappers
│       ├── models.rs         # Shared types (Serialize + Deserialize)
│       ├── storage.rs        # File I/O using tokio::fs
│       └── server/
│           ├── mod.rs        # Start/stop + graceful shutdown
│           ├── router.rs     # Path param matching (:id support)
│           └── state.rs      # AppState + ServerHandle
└── src/
    ├── App.svelte
    ├── app.html
    └── lib/
        ├── components/
        │   ├── sidebar/
        │   │   ├── index.svelte
        │   │   └── CollectionItem.svelte
        │   ├── editor/
        │   │   ├── index.svelte
        │   │   ├── EndpointList.svelte
        │   │   ├── EndpointDetail.svelte
        │   │   ├── ScenarioEditor.svelte
        │   │   └── MonacoEditor.svelte
        │   └── server/
        │       ├── ServerPanel.svelte
        │       └── LogPanel.svelte
        ├── stores/
        │   ├── collections.ts
        │   ├── server.ts
        │   └── logs.ts
        └── types.ts
```

---

## Rust Backend

### `main.rs` — Tauri setup only
```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(ServerHandle::default())
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
}
```

### `commands.rs` — thin wrappers, no business logic
```rust
#[tauri::command] async fn load_data(app: AppHandle) -> Result<AppData, String>
#[tauri::command] async fn save_data(app: AppHandle, data: AppData) -> Result<(), String>
#[tauri::command] async fn start_server(app: AppHandle, state: State<ServerHandle>, collection_id: String) -> Result<(), String>
#[tauri::command] async fn stop_server(state: State<ServerHandle>) -> Result<(), String>
```

### `storage.rs` — tokio::fs, maps all errors to String
```rust
pub async fn load(app: &AppHandle) -> Result<AppData, String>
pub async fn save(app: &AppHandle, data: &AppData) -> Result<(), String>
// Uses app.path().app_local_data_dir() (Tauri v2 API)
```

### `server/state.rs`
```rust
pub struct AppState {
    pub collection: Arc<RwLock<Collection>>,
    pub app_handle: AppHandle,
}

#[derive(Default)]
pub struct ServerHandle(pub Mutex<Option<oneshot::Sender<()>>>);
```

### `server/mod.rs` — graceful shutdown via oneshot channel
```rust
pub async fn start(collection: Collection, app_handle: AppHandle)
    -> Result<oneshot::Sender<()>, String>
// Caller stores sender in ServerHandle managed state
// stop_server calls sender.send(()) to shut down
```

### `server/router.rs` — path parameter matching
```rust
pub fn match_path(pattern: &str, actual: &str) -> Option<HashMap<String, String>>
// /users/:id matches /users/123 → { "id": "123" }
// Params substituted into response body templates
```

---

## Frontend

### Stores

**`stores/collections.ts`**
```typescript
export const collections = writable<Collection[]>([]);
export const activeCollectionId = writable<string | null>(null);
export const activeCollection = derived([collections, activeCollectionId], ...);
export const activeEndpointId = writable<string | null>(null);
export const activeEndpoint = derived([activeCollection, activeEndpointId], ...);

// Actions — auto-save debounced 500ms after any mutation
export async function loadCollections() { ... }
export function addCollection(name: string) { ... }
export function deleteCollection(id: string) { ... }
export function addEndpoint(collectionId: string) { ... }
export function updateEndpoint(collectionId: string, endpoint: Endpoint) { ... }
export function deleteEndpoint(collectionId: string, endpointId: string) { ... }
```

**`stores/server.ts`**
```typescript
export const serverState = writable<'stopped'|'starting'|'running'|'error'>('stopped');
export const serverError = writable<string | null>(null);
export async function startServer(collectionId: string) { ... }
export async function stopServer() { ... }
```

**`stores/logs.ts`**
```typescript
export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  duration: number;
}
export const logs = writable<LogEntry[]>([]);
// Ring buffer — max 200 entries
export function clearLogs() { ... }
```

### Component Tree
```
App.svelte (onMount → loadCollections())
├── sidebar/index.svelte
│   └── CollectionItem.svelte
├── editor/index.svelte
│   ├── EndpointList.svelte
│   ├── EndpointDetail.svelte
│   ├── ScenarioEditor.svelte
│   └── MonacoEditor.svelte  (lazy-loaded on mount)
└── server/
    ├── ServerPanel.svelte
    └── LogPanel.svelte
```

### Monaco — lazy loaded
```typescript
onMount(async () => {
  const monaco = await import('monaco-editor');
  editor = monaco.editor.create(container, {
    language: 'json',
    theme: 'vs-dark',
    automaticLayout: true,
    minimap: { enabled: false }
  });
});
```

---

## Data Flow

```
App boots
  └─ App.svelte onMount → loadCollections()
       └─ invoke('load_data') → storage::load() → AppData
            └─ collections.set() + activeCollectionId.set(first id)

User edits endpoint
  └─ updateEndpoint(collectionId, endpoint)
       └─ collections.update() → debounced auto-save (500ms)
            └─ invoke('save_data', data) → storage::save()

User starts server
  └─ startServer(collectionId)
       └─ serverState = 'starting'
       └─ invoke('start_server', { collectionId })
            └─ server::start() → binds port → stores shutdown sender
            └─ serverState = 'running'  OR  serverError = message

HTTP request hits mock server
  └─ actix mock_handler
       └─ router::match_path() → find active scenario
       └─ apply delay → substitute params → return response
       └─ app_handle.emit('server:log', LogEntry)
            └─ logs store updates → LogPanel re-renders
```

---

## Error Handling

- **Rust:** All commands return `Result<T, String>`, all errors mapped via `.map_err(|e| e.to_string())?`
- **Frontend:** Each store action has try/catch, errors stored in writable error stores, surfaced in UI
- **Auto-save failures:** Shown as subtle toast, never block the user
- **Server bind failure:** `serverState = 'error'`, `serverError` shows message in ServerPanel

---

## Security Fixes Over Original Plan

| Issue | Fix |
|---|---|
| `{@html log}` XSS | Structured `LogEntry` type, text-only rendering |
| No input validation | Endpoint path validated before save |
| No CORS headers | `actix-cors` added to Actix app |

---

## Key Decisions Summary

| Decision | Choice | Reason |
|---|---|---|
| Port scope | Per-collection | Matches Mockoon behavior, enables multi-collection testing |
| Save strategy | Auto-save debounced 500ms | No friction, immediate persistence |
| Server shutdown | oneshot channel in managed state | Clean, no process kills |
| Path matching | Custom regex-free parser | Simple, no deps, covers `:param` patterns |
| Monaco loading | Lazy (onMount dynamic import) | Avoids blocking initial render |
| Store structure | 3 domain stores + derived | Single responsibility, no derived logic in components |
