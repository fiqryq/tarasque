# Tarasque Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build Tarasque — a desktop mock API server where users create collections of endpoints with configurable response scenarios, then start a localhost server to serve them.

**Architecture:** Domain-driven Rust modules (models, storage, server/router, server/state, commands) with thin Tauri command wrappers. Svelte 5 frontend with 3 domain stores (collections, server, logs) and feature-grouped components.

**Tech Stack:** Tauri v2, Rust (Actix-web 4, tokio, serde_json, uuid), Svelte 5, Tailwind CSS 3, Monaco Editor, lucide-svelte

**Design doc:** `docs/plans/2026-03-09-tarasque-architecture-design.md`

---

## Task 1: Project Scaffold

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `svelte.config.js`
- Create: `tailwind.config.js`
- Create: `postcss.config.js`
- Create: `src/app.html`
- Create: `src/app.css`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs` (placeholder)

**Step 1: Scaffold via tauri CLI**

```bash
npm create tauri-app@latest tarasque -- --template svelte-ts
cd tarasque
```

**Step 2: Replace `package.json`**

```json
{
  "name": "tarasque",
  "version": "1.0.0",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^3.1.1",
    "@tauri-apps/cli": "^2.0.0",
    "autoprefixer": "^10.4.20",
    "postcss": "^8.4.41",
    "tailwindcss": "^3.4.10",
    "vite": "^5.4.1",
    "svelte": "^5.0.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "lucide-svelte": "^0.518.0",
    "monaco-editor": "^0.52.2"
  }
}
```

**Step 3: Replace `vite.config.ts`**

```typescript
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  resolve: {
    alias: { '$lib': path.resolve('./src/lib') }
  },
  server: {
    port: 1420,
    strictPort: true
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG
  }
});
```

**Step 4: Create `svelte.config.js`**

```javascript
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess()
};
```

**Step 5: Create `tailwind.config.js`**

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: { extend: {} },
  plugins: []
};
```

**Step 6: Create `postcss.config.js`**

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {}
  }
};
```

**Step 7: Replace `src/app.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="/src/app.css" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Tarasque</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

**Step 8: Create `src/app.css`**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

**Step 9: Create `src/main.ts`**

```typescript
import App from './App.svelte';
import { mount } from 'svelte';

const app = mount(App, { target: document.getElementById('app')! });

export default app;
```

**Step 10: Replace `src-tauri/Cargo.toml`**

```toml
[package]
name = "tarasque"
version = "1.0.0"
description = "Mock API Playground"
authors = ["Fiqry Choerudin"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-fs = "2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
actix-web = "4"
actix-cors = "0.7"
futures = "0.3"
uuid = { version = "1", features = ["v4", "serde"] }
```

**Step 11: Replace `src-tauri/tauri.conf.json`**

```json
{
  "productName": "Tarasque",
  "version": "1.0.0",
  "identifier": "com.tarasque.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Tarasque",
        "width": 1400,
        "height": 900,
        "resizable": true
      }
    ],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**Step 12: Install dependencies**

```bash
npm install
cd src-tauri && cargo check
```

Expected: No errors (warnings OK)

**Step 13: Commit**

```bash
git add -A
git commit -m "feat: initial project scaffold"
```

---

## Task 2: Rust Models

**Files:**
- Create: `src-tauri/src/models.rs`

**Step 1: Create `src-tauri/src/models.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub status: u16,
    pub delay_ms: u64,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub id: String,
    pub method: String,
    pub path: String,
    pub scenarios: Vec<Scenario>,
    pub active_scenario_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppData {
    pub collections: Vec<Collection>,
    pub active_collection_id: Option<String>,
}
```

**Step 2: Update `src-tauri/src/main.rs` to include module**

```rust
mod models;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

**Step 4: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/main.rs
git commit -m "feat: add rust data models"
```

---

## Task 3: Storage Module

**Files:**
- Create: `src-tauri/src/storage.rs`

**Step 1: Create `src-tauri/src/storage.rs`**

```rust
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
```

**Step 2: Add module to `main.rs`**

```rust
mod models;
mod storage;
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

**Step 4: Commit**

```bash
git add src-tauri/src/storage.rs src-tauri/src/main.rs
git commit -m "feat: add storage module with tokio async I/O"
```

---

## Task 4: Server State & Handle

**Files:**
- Create: `src-tauri/src/server/mod.rs`
- Create: `src-tauri/src/server/state.rs`

**Step 1: Create `src-tauri/src/server/state.rs`**

```rust
use crate::models::Collection;
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub collection: Arc<RwLock<Collection>>,
    pub app_handle: tauri::AppHandle,
}

#[derive(Default)]
pub struct ServerHandle(pub Mutex<Option<oneshot::Sender<()>>>);
```

**Step 2: Create `src-tauri/src/server/mod.rs` (placeholder)**

```rust
pub mod state;
```

**Step 3: Add server module to `main.rs`**

```rust
mod models;
mod storage;
mod server;
```

**Step 4: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 5: Commit**

```bash
git add src-tauri/src/server/
git commit -m "feat: add server state and handle types"
```

---

## Task 5: Path Router (with tests)

**Files:**
- Create: `src-tauri/src/server/router.rs`

**Step 1: Write the failing tests first — add to `router.rs`**

```rust
use std::collections::HashMap;

pub fn match_path(pattern: &str, actual: &str) -> Option<HashMap<String, String>> {
    todo!()
}

pub fn substitute_params(body: &str, params: &HashMap<String, String>) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_returns_empty_params() {
        let result = match_path("/users", "/users");
        assert_eq!(result, Some(HashMap::new()));
    }

    #[test]
    fn exact_mismatch_returns_none() {
        let result = match_path("/users", "/posts");
        assert_eq!(result, None);
    }

    #[test]
    fn param_match_extracts_value() {
        let result = match_path("/users/:id", "/users/123");
        let mut expected = HashMap::new();
        expected.insert("id".to_string(), "123".to_string());
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn multiple_params_extracted() {
        let result = match_path("/users/:userId/posts/:postId", "/users/42/posts/7");
        let mut expected = HashMap::new();
        expected.insert("userId".to_string(), "42".to_string());
        expected.insert("postId".to_string(), "7".to_string());
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn different_segment_count_returns_none() {
        let result = match_path("/users/:id", "/users/123/extra");
        assert_eq!(result, None);
    }

    #[test]
    fn substitute_replaces_param_in_body() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        let result = substitute_params(r#"{"id": ":id"}"#, &params);
        assert_eq!(result, r#"{"id": "123"}"#);
    }

    #[test]
    fn substitute_no_params_returns_original() {
        let params = HashMap::new();
        let body = r#"{"message": "hello"}"#;
        assert_eq!(substitute_params(body, &params), body);
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test server::router
```

Expected: FAIL — `todo!()` panics

**Step 3: Implement `match_path` and `substitute_params`**

Replace the `todo!()` bodies:

```rust
pub fn match_path(pattern: &str, actual: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let actual_parts: Vec<&str> = actual.split('/').collect();

    if pattern_parts.len() != actual_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (p, a) in pattern_parts.iter().zip(actual_parts.iter()) {
        if let Some(key) = p.strip_prefix(':') {
            params.insert(key.to_string(), a.to_string());
        } else if p != a {
            return None;
        }
    }

    Some(params)
}

pub fn substitute_params(body: &str, params: &HashMap<String, String>) -> String {
    let mut result = body.to_string();
    for (key, value) in params {
        result = result.replace(&format!(":{key}"), value);
    }
    result
}
```

**Step 4: Run tests to verify they pass**

```bash
cd src-tauri && cargo test server::router
```

Expected: All 7 tests PASS

**Step 5: Add router to server/mod.rs**

```rust
pub mod router;
pub mod state;
```

**Step 6: Commit**

```bash
git add src-tauri/src/server/router.rs src-tauri/src/server/mod.rs
git commit -m "feat: add path router with param matching and substitution"
```

---

## Task 6: Actix Server (Start/Stop)

**Files:**
- Modify: `src-tauri/src/server/mod.rs`

**Step 1: Add handler + start function to `server/mod.rs`**

Replace `src-tauri/src/server/mod.rs` fully:

```rust
pub mod router;
pub mod state;

use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use state::{AppState, ServerHandle};
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};
use crate::models::{Collection, LogEntry};
use tauri::Emitter;

pub async fn start(
    collection: Collection,
    app_handle: tauri::AppHandle,
    server_handle: &ServerHandle,
) -> Result<(), String> {
    let port = collection.port;
    let state = AppState {
        collection: Arc::new(RwLock::new(collection)),
        app_handle: app_handle.clone(),
    };

    let (tx, rx) = oneshot::channel::<()>();

    // Store shutdown sender
    *server_handle.0.lock().await = Some(tx);

    tauri::async_runtime::spawn(async move {
        let state_data = web::Data::new(state);

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
        .bind(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Bind error: {e}"))?
        .run();

        let server_handle = server.handle();

        // Spawn task to wait for shutdown signal
        tauri::async_runtime::spawn(async move {
            let _ = rx.await;
            server_handle.stop(true).await;
        });

        server.await.map_err(|e| format!("Server error: {e}"))
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

    // Find matching endpoint
    for endpoint in &collection.endpoints {
        if endpoint.method.to_uppercase() != method.to_uppercase() {
            continue;
        }

        if let Some(params) = router::match_path(&endpoint.path, &path) {
            // Find active scenario
            let scenario = endpoint
                .scenarios
                .iter()
                .find(|s| s.id == endpoint.active_scenario_id)
                .or_else(|| endpoint.scenarios.first());

            if let Some(scenario) = scenario {
                // Apply delay
                if scenario.delay_ms > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(scenario.delay_ms)).await;
                }

                let duration = start.elapsed().as_millis() as u64;

                // Substitute path params in body
                let body = router::substitute_params(&scenario.body, &params);

                // Emit log event
                let log = LogEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    method: method.clone(),
                    path: path.clone(),
                    status: scenario.status,
                    duration,
                };
                let _ = state.app_handle.emit("server:log", &log);

                // Build response
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
    let _ = state.app_handle.emit("server:log", &log);

    HttpResponse::NotFound().json(serde_json::json!({ "error": "No matching endpoint found" }))
}
```

**Step 2: Add `LogEntry` to `models.rs`**

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration: u64,
}
```

**Step 3: Add `chrono` to `Cargo.toml`**

```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Step 4: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

**Step 5: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: add actix server with start/stop and mock handler"
```

---

## Task 7: Tauri Commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create `src-tauri/src/commands.rs`**

```rust
use crate::models::AppData;
use crate::server::state::ServerHandle;
use crate::{server, storage};
use tauri::{AppHandle, Manager, State};

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
) -> Result<(), String> {
    let data = storage::load(&app).await?;

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
```

**Step 2: Replace `src-tauri/src/main.rs`**

```rust
mod commands;
mod models;
mod server;
mod storage;

use server::state::ServerHandle;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(ServerHandle::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_data,
            commands::save_data,
            commands::start_server,
            commands::stop_server,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

**Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "feat: add tauri commands for data and server control"
```

---

## Task 8: TypeScript Types

**Files:**
- Create: `src/lib/types.ts`

**Step 1: Create `src/lib/types.ts`**

```typescript
export interface Scenario {
  id: string;
  name: string;
  status: number;
  delay_ms: number;
  headers: Record<string, string>;
  body: string;
}

export interface Endpoint {
  id: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  scenarios: Scenario[];
  active_scenario_id: string;
}

export interface Collection {
  id: string;
  name: string;
  port: number;
  endpoints: Endpoint[];
}

export interface AppData {
  collections: Collection[];
  active_collection_id: string | null;
}

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  duration: number;
}

// Helpers for creating new entities with defaults
export function newScenario(overrides: Partial<Scenario> = {}): Scenario {
  return {
    id: crypto.randomUUID(),
    name: 'Default',
    status: 200,
    delay_ms: 0,
    headers: { 'Content-Type': 'application/json' },
    body: '{\n  \n}',
    ...overrides,
  };
}

export function newEndpoint(overrides: Partial<Endpoint> = {}): Endpoint {
  const scenario = newScenario();
  return {
    id: crypto.randomUUID(),
    method: 'GET',
    path: '/new-endpoint',
    scenarios: [scenario],
    active_scenario_id: scenario.id,
    ...overrides,
  };
}

export function newCollection(name: string): Collection {
  return {
    id: crypto.randomUUID(),
    name,
    port: 3000,
    endpoints: [],
  };
}
```

**Step 2: Commit**

```bash
git add src/lib/types.ts
git commit -m "feat: add typescript types with factory helpers"
```

---

## Task 9: Collections Store

**Files:**
- Create: `src/lib/stores/collections.ts`

**Step 1: Create `src/lib/stores/collections.ts`**

```typescript
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Collection, Endpoint, AppData } from '$lib/types';
import { newCollection, newEndpoint } from '$lib/types';

export const collections = writable<Collection[]>([]);
export const activeCollectionId = writable<string | null>(null);
export const activeEndpointId = writable<string | null>(null);

export const activeCollection = derived(
  [collections, activeCollectionId],
  ([$cols, $id]) => $cols.find((c) => c.id === $id) ?? null
);

export const activeEndpoint = derived(
  [activeCollection, activeEndpointId],
  ([$col, $eid]) => $col?.endpoints.find((e) => e.id === $eid) ?? null
);

// Auto-save — debounced 500ms
let saveTimer: ReturnType<typeof setTimeout>;
function scheduleAutoSave() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(persistData, 500);
}

async function persistData() {
  const data: AppData = {
    collections: get(collections),
    active_collection_id: get(activeCollectionId),
  };
  try {
    await invoke('save_data', { data });
  } catch (e) {
    console.error('Auto-save failed:', e);
  }
}

export async function loadCollections() {
  try {
    const data = await invoke<AppData>('load_data');
    collections.set(data.collections);
    if (data.active_collection_id) {
      activeCollectionId.set(data.active_collection_id);
    } else if (data.collections.length > 0) {
      activeCollectionId.set(data.collections[0].id);
    }
  } catch (e) {
    console.error('Load failed:', e);
  }
}

export function addCollection(name: string) {
  const col = newCollection(name);
  collections.update((c) => [...c, col]);
  activeCollectionId.set(col.id);
  scheduleAutoSave();
}

export function deleteCollection(id: string) {
  collections.update((c) => c.filter((col) => col.id !== id));
  activeCollectionId.update((current) => (current === id ? null : current));
  scheduleAutoSave();
}

export function renameCollection(id: string, name: string) {
  collections.update((cols) =>
    cols.map((c) => (c.id === id ? { ...c, name } : c))
  );
  scheduleAutoSave();
}

export function updateCollectionPort(id: string, port: number) {
  collections.update((cols) =>
    cols.map((c) => (c.id === id ? { ...c, port } : c))
  );
  scheduleAutoSave();
}

export function addEndpoint(collectionId: string) {
  const ep = newEndpoint();
  collections.update((cols) =>
    cols.map((c) =>
      c.id === collectionId ? { ...c, endpoints: [...c.endpoints, ep] } : c
    )
  );
  activeEndpointId.set(ep.id);
  scheduleAutoSave();
}

export function deleteEndpoint(collectionId: string, endpointId: string) {
  collections.update((cols) =>
    cols.map((c) =>
      c.id === collectionId
        ? { ...c, endpoints: c.endpoints.filter((e) => e.id !== endpointId) }
        : c
    )
  );
  activeEndpointId.update((current) => (current === endpointId ? null : current));
  scheduleAutoSave();
}

export function updateEndpoint(collectionId: string, endpoint: Endpoint) {
  collections.update((cols) =>
    cols.map((c) =>
      c.id === collectionId
        ? { ...c, endpoints: c.endpoints.map((e) => (e.id === endpoint.id ? endpoint : e)) }
        : c
    )
  );
  scheduleAutoSave();
}
```

**Step 2: Commit**

```bash
git add src/lib/stores/collections.ts
git commit -m "feat: add collections store with derived state and auto-save"
```

---

## Task 10: Server & Logs Stores

**Files:**
- Create: `src/lib/stores/server.ts`
- Create: `src/lib/stores/logs.ts`

**Step 1: Create `src/lib/stores/server.ts`**

```typescript
import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCollectionId } from './collections';

export type ServerStatus = 'stopped' | 'starting' | 'running' | 'error';

export const serverStatus = writable<ServerStatus>('stopped');
export const serverError = writable<string | null>(null);

export async function startServer() {
  const collectionId = get(activeCollectionId);
  if (!collectionId) {
    serverError.set('No collection selected');
    serverStatus.set('error');
    return;
  }

  serverStatus.set('starting');
  serverError.set(null);

  try {
    await invoke('start_server', { collectionId });
    serverStatus.set('running');
  } catch (e) {
    serverStatus.set('error');
    serverError.set(e as string);
  }
}

export async function stopServer() {
  try {
    await invoke('stop_server');
    serverStatus.set('stopped');
    serverError.set(null);
  } catch (e) {
    serverError.set(e as string);
  }
}
```

**Step 2: Create `src/lib/stores/logs.ts`**

```typescript
import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { LogEntry } from '$lib/types';

const MAX_LOGS = 200;

export const logs = writable<LogEntry[]>([]);

export function clearLogs() {
  logs.set([]);
}

export async function initLogListener() {
  await listen<LogEntry>('server:log', (event) => {
    logs.update((current) => {
      const next = [...current, event.payload];
      return next.length > MAX_LOGS ? next.slice(-MAX_LOGS) : next;
    });
  });
}
```

**Step 3: Commit**

```bash
git add src/lib/stores/
git commit -m "feat: add server and logs stores"
```

---

## Task 11: Sidebar Components

**Files:**
- Create: `src/lib/components/sidebar/index.svelte`
- Create: `src/lib/components/sidebar/CollectionItem.svelte`

**Step 1: Create `src/lib/components/sidebar/CollectionItem.svelte`**

```svelte
<script lang="ts">
  import { Trash2 } from 'lucide-svelte';
  import type { Collection } from '$lib/types';

  interface Props {
    collection: Collection;
    active: boolean;
    onSelect: () => void;
    onDelete: () => void;
  }

  let { collection, active, onSelect, onDelete }: Props = $props();
</script>

<button
  class="w-full text-left px-3 py-2 rounded-lg flex items-center gap-2 group transition-colors
    {active ? 'bg-blue-600/30 border border-blue-500/50 text-white' : 'hover:bg-slate-700 text-slate-300'}"
  onclick={onSelect}
>
  <span class="flex-1 truncate text-sm">{collection.name}</span>
  <span class="text-xs text-slate-500 font-mono">{collection.port}</span>
  <button
    class="opacity-0 group-hover:opacity-100 p-1 hover:text-red-400 transition-opacity"
    onclick={(e) => { e.stopPropagation(); onDelete(); }}
  >
    <Trash2 size={13} />
  </button>
</button>
```

**Step 2: Create `src/lib/components/sidebar/index.svelte`**

```svelte
<script lang="ts">
  import { Plus } from 'lucide-svelte';
  import CollectionItem from './CollectionItem.svelte';
  import {
    collections,
    activeCollectionId,
    addCollection,
    deleteCollection,
  } from '$lib/stores/collections';

  let newName = $state('');

  function handleCreate() {
    if (newName.trim()) {
      addCollection(newName.trim());
      newName = '';
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="px-4 py-3 border-b border-slate-700">
    <h1 class="text-lg font-bold text-white tracking-tight">Tarasque</h1>
    <p class="text-xs text-slate-500 mt-0.5">Mock API Server</p>
  </div>

  <div class="p-3 border-b border-slate-700">
    <div class="flex gap-2">
      <input
        bind:value={newName}
        onkeydown={(e) => e.key === 'Enter' && handleCreate()}
        placeholder="New collection..."
        class="flex-1 bg-slate-700 text-white text-sm px-3 py-1.5 rounded-lg
               placeholder:text-slate-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
      />
      <button
        onclick={handleCreate}
        class="p-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
      >
        <Plus size={16} class="text-white" />
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto p-2 space-y-0.5">
    {#each $collections as collection (collection.id)}
      <CollectionItem
        {collection}
        active={$activeCollectionId === collection.id}
        onSelect={() => activeCollectionId.set(collection.id)}
        onDelete={() => deleteCollection(collection.id)}
      />
    {/each}

    {#if $collections.length === 0}
      <p class="text-slate-500 text-xs text-center mt-6 px-4">
        Create a collection to get started
      </p>
    {/if}
  </div>
</div>
```

**Step 3: Commit**

```bash
git add src/lib/components/sidebar/
git commit -m "feat: add sidebar components"
```

---

## Task 12: Monaco Editor Component

**Files:**
- Create: `src/lib/components/editor/MonacoEditor.svelte`

**Step 1: Create `src/lib/components/editor/MonacoEditor.svelte`**

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type * as Monaco from 'monaco-editor';

  interface Props {
    value: string;
    language?: string;
    onChange: (value: string) => void;
  }

  let { value, language = 'json', onChange }: Props = $props();

  let container: HTMLDivElement;
  let editor: Monaco.editor.IStandaloneCodeEditor | null = null;
  let monaco: typeof Monaco | null = null;

  onMount(async () => {
    monaco = await import('monaco-editor');

    editor = monaco.editor.create(container, {
      value,
      language,
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 13,
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      tabSize: 2,
    });

    editor.onDidChangeModelContent(() => {
      onChange(editor!.getValue());
    });
  });

  // Sync external value changes without moving cursor
  $effect(() => {
    if (editor && editor.getValue() !== value) {
      const model = editor.getModel();
      if (model) {
        model.pushEditOperations(
          [],
          [{ range: model.getFullModelRange(), text: value }],
          () => null
        );
      }
    }
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div bind:this={container} class="h-full w-full"></div>
```

**Step 2: Commit**

```bash
git add src/lib/components/editor/MonacoEditor.svelte
git commit -m "feat: add lazy-loaded monaco editor component"
```

---

## Task 13: Scenario Editor Component

**Files:**
- Create: `src/lib/components/editor/ScenarioEditor.svelte`

**Step 1: Create `src/lib/components/editor/ScenarioEditor.svelte`**

```svelte
<script lang="ts">
  import MonacoEditor from './MonacoEditor.svelte';
  import type { Scenario } from '$lib/types';

  interface Props {
    scenario: Scenario;
    onChange: (scenario: Scenario) => void;
  }

  let { scenario, onChange }: Props = $props();

  function update(partial: Partial<Scenario>) {
    onChange({ ...scenario, ...partial });
  }

  let newHeaderKey = $state('');
  let newHeaderVal = $state('');

  function addHeader() {
    if (newHeaderKey.trim()) {
      update({ headers: { ...scenario.headers, [newHeaderKey.trim()]: newHeaderVal } });
      newHeaderKey = '';
      newHeaderVal = '';
    }
  }

  function removeHeader(key: string) {
    const { [key]: _, ...rest } = scenario.headers;
    update({ headers: rest });
  }
</script>

<div class="flex flex-col gap-4 p-4 h-full overflow-y-auto">
  <!-- Status + Delay -->
  <div class="flex gap-4">
    <div class="flex flex-col gap-1">
      <label class="text-xs text-slate-400 font-medium">Status Code</label>
      <input
        type="number"
        value={scenario.status}
        onchange={(e) => update({ status: Number(e.currentTarget.value) })}
        class="w-24 bg-slate-700 text-white px-2 py-1.5 rounded text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
      />
    </div>
    <div class="flex flex-col gap-1">
      <label class="text-xs text-slate-400 font-medium">Delay (ms)</label>
      <input
        type="number"
        min="0"
        value={scenario.delay_ms}
        onchange={(e) => update({ delay_ms: Number(e.currentTarget.value) })}
        class="w-28 bg-slate-700 text-white px-2 py-1.5 rounded text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
      />
    </div>
  </div>

  <!-- Headers -->
  <div class="flex flex-col gap-2">
    <label class="text-xs text-slate-400 font-medium">Response Headers</label>
    <div class="space-y-1">
      {#each Object.entries(scenario.headers) as [key, val] (key)}
        <div class="flex gap-2 items-center">
          <span class="text-xs font-mono text-slate-300 bg-slate-700 px-2 py-1 rounded flex-1">{key}</span>
          <span class="text-xs font-mono text-slate-400 bg-slate-700 px-2 py-1 rounded flex-1">{val}</span>
          <button
            onclick={() => removeHeader(key)}
            class="text-slate-500 hover:text-red-400 text-xs px-1"
          >✕</button>
        </div>
      {/each}
    </div>
    <div class="flex gap-2">
      <input
        bind:value={newHeaderKey}
        placeholder="Key"
        class="flex-1 bg-slate-700 text-white text-xs px-2 py-1 rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
      />
      <input
        bind:value={newHeaderVal}
        placeholder="Value"
        class="flex-1 bg-slate-700 text-white text-xs px-2 py-1 rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
      />
      <button
        onclick={addHeader}
        class="text-xs px-2 py-1 bg-slate-600 hover:bg-slate-500 rounded"
      >Add</button>
    </div>
  </div>

  <!-- Body -->
  <div class="flex flex-col gap-1 flex-1 min-h-0">
    <label class="text-xs text-slate-400 font-medium">Response Body</label>
    <div class="flex-1 rounded overflow-hidden border border-slate-600">
      <MonacoEditor
        value={scenario.body}
        onChange={(val) => update({ body: val })}
      />
    </div>
  </div>
</div>
```

**Step 2: Commit**

```bash
git add src/lib/components/editor/ScenarioEditor.svelte
git commit -m "feat: add scenario editor component"
```

---

## Task 14: Endpoint Detail & List

**Files:**
- Create: `src/lib/components/editor/EndpointDetail.svelte`
- Create: `src/lib/components/editor/EndpointList.svelte`

**Step 1: Create `src/lib/components/editor/EndpointDetail.svelte`**

```svelte
<script lang="ts">
  import ScenarioEditor from './ScenarioEditor.svelte';
  import type { Endpoint, Scenario } from '$lib/types';
  import { newScenario } from '$lib/types';

  interface Props {
    endpoint: Endpoint;
    collectionId: string;
    onUpdate: (endpoint: Endpoint) => void;
  }

  let { endpoint, collectionId, onUpdate }: Props = $props();

  const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'] as const;
  const METHOD_COLORS: Record<string, string> = {
    GET: 'text-green-400',
    POST: 'text-yellow-400',
    PUT: 'text-blue-400',
    DELETE: 'text-red-400',
    PATCH: 'text-purple-400',
  };

  let activeScenarioId = $state(endpoint.active_scenario_id);
  let activeScenario = $derived(
    endpoint.scenarios.find((s) => s.id === activeScenarioId) ?? endpoint.scenarios[0]
  );

  function updateScenario(updated: Scenario) {
    onUpdate({
      ...endpoint,
      scenarios: endpoint.scenarios.map((s) => (s.id === updated.id ? updated : s)),
    });
  }

  function addScenario() {
    const s = newScenario({ name: `Scenario ${endpoint.scenarios.length + 1}` });
    onUpdate({ ...endpoint, scenarios: [...endpoint.scenarios, s] });
    activeScenarioId = s.id;
  }

  function deleteScenario(id: string) {
    if (endpoint.scenarios.length <= 1) return;
    const remaining = endpoint.scenarios.filter((s) => s.id !== id);
    const newActive = remaining[0].id;
    activeScenarioId = newActive;
    onUpdate({ ...endpoint, scenarios: remaining, active_scenario_id: newActive });
  }

  function setActiveScenario(id: string) {
    activeScenarioId = id;
    onUpdate({ ...endpoint, active_scenario_id: id });
  }
</script>

<div class="flex flex-col h-full">
  <!-- Method + Path -->
  <div class="flex gap-2 p-4 border-b border-slate-700">
    <select
      value={endpoint.method}
      onchange={(e) => onUpdate({ ...endpoint, method: e.currentTarget.value })}
      class="bg-slate-700 text-white text-sm px-2 py-1.5 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 font-mono {METHOD_COLORS[endpoint.method]}"
    >
      {#each METHODS as m}
        <option value={m} class={METHOD_COLORS[m]}>{m}</option>
      {/each}
    </select>
    <input
      value={endpoint.path}
      oninput={(e) => onUpdate({ ...endpoint, path: e.currentTarget.value })}
      placeholder="/your/path/:id"
      class="flex-1 bg-slate-700 text-white text-sm px-3 py-1.5 rounded font-mono
             focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <!-- Scenario Tabs -->
  <div class="flex items-center gap-1 px-4 py-2 border-b border-slate-700 overflow-x-auto">
    {#each endpoint.scenarios as scenario (scenario.id)}
      <div class="flex items-center gap-1">
        <button
          onclick={() => setActiveScenario(scenario.id)}
          class="text-xs px-3 py-1 rounded transition-colors whitespace-nowrap
            {activeScenarioId === scenario.id
              ? 'bg-blue-600 text-white'
              : 'text-slate-400 hover:bg-slate-700'}"
        >
          {scenario.name}
          <span class="ml-1 font-mono opacity-70">{scenario.status}</span>
        </button>
        {#if endpoint.scenarios.length > 1}
          <button
            onclick={() => deleteScenario(scenario.id)}
            class="text-slate-600 hover:text-red-400 text-xs"
          >✕</button>
        {/if}
      </div>
    {/each}
    <button
      onclick={addScenario}
      class="text-xs px-2 py-1 text-slate-500 hover:text-white hover:bg-slate-700 rounded whitespace-nowrap"
    >
      + Add Scenario
    </button>
  </div>

  <!-- Scenario Editor -->
  <div class="flex-1 min-h-0">
    {#if activeScenario}
      <ScenarioEditor scenario={activeScenario} onChange={updateScenario} />
    {/if}
  </div>
</div>
```

**Step 2: Create `src/lib/components/editor/EndpointList.svelte`**

```svelte
<script lang="ts">
  import { Plus, Trash2 } from 'lucide-svelte';
  import type { Endpoint } from '$lib/types';

  interface Props {
    endpoints: Endpoint[];
    activeId: string | null;
    onSelect: (id: string) => void;
    onAdd: () => void;
    onDelete: (id: string) => void;
  }

  let { endpoints, activeId, onSelect, onAdd, onDelete }: Props = $props();

  const METHOD_COLORS: Record<string, string> = {
    GET: 'text-green-400 bg-green-400/10',
    POST: 'text-yellow-400 bg-yellow-400/10',
    PUT: 'text-blue-400 bg-blue-400/10',
    DELETE: 'text-red-400 bg-red-400/10',
    PATCH: 'text-purple-400 bg-purple-400/10',
  };
</script>

<div class="flex flex-col h-full border-r border-slate-700">
  <div class="flex items-center justify-between px-3 py-2 border-b border-slate-700">
    <span class="text-xs font-medium text-slate-400 uppercase tracking-wider">Endpoints</span>
    <button
      onclick={onAdd}
      class="p-1 hover:bg-slate-700 rounded transition-colors"
      title="Add endpoint"
    >
      <Plus size={14} class="text-slate-400" />
    </button>
  </div>

  <div class="flex-1 overflow-y-auto">
    {#each endpoints as endpoint (endpoint.id)}
      <button
        onclick={() => onSelect(endpoint.id)}
        class="w-full text-left px-3 py-2.5 flex items-center gap-2 group transition-colors
          {activeId === endpoint.id ? 'bg-slate-700' : 'hover:bg-slate-700/50'}"
      >
        <span class="text-xs font-mono font-bold px-1.5 py-0.5 rounded {METHOD_COLORS[endpoint.method] ?? 'text-slate-400 bg-slate-700'}">
          {endpoint.method}
        </span>
        <span class="flex-1 text-sm text-slate-300 truncate font-mono">{endpoint.path}</span>
        <button
          onclick={(e) => { e.stopPropagation(); onDelete(endpoint.id); }}
          class="opacity-0 group-hover:opacity-100 p-1 hover:text-red-400 transition-opacity"
        >
          <Trash2 size={12} />
        </button>
      </button>
    {/each}

    {#if endpoints.length === 0}
      <p class="text-slate-500 text-xs text-center mt-8 px-4">No endpoints yet</p>
    {/if}
  </div>
</div>
```

**Step 3: Commit**

```bash
git add src/lib/components/editor/
git commit -m "feat: add endpoint list and detail components"
```

---

## Task 15: Editor Index

**Files:**
- Create: `src/lib/components/editor/index.svelte`

**Step 1: Create `src/lib/components/editor/index.svelte`**

```svelte
<script lang="ts">
  import EndpointList from './EndpointList.svelte';
  import EndpointDetail from './EndpointDetail.svelte';
  import {
    activeCollection,
    activeEndpoint,
    activeEndpointId,
    activeCollectionId,
    addEndpoint,
    deleteEndpoint,
    updateEndpoint,
  } from '$lib/stores/collections';
</script>

{#if $activeCollection}
  <div class="flex h-full">
    <!-- Left: endpoint list (fixed width) -->
    <div class="w-64 shrink-0">
      <EndpointList
        endpoints={$activeCollection.endpoints}
        activeId={$activeEndpointId}
        onSelect={(id) => activeEndpointId.set(id)}
        onAdd={() => addEndpoint($activeCollection!.id)}
        onDelete={(id) => deleteEndpoint($activeCollection!.id, id)}
      />
    </div>

    <!-- Right: endpoint detail -->
    <div class="flex-1 min-w-0">
      {#if $activeEndpoint && $activeCollectionId}
        <EndpointDetail
          endpoint={$activeEndpoint}
          collectionId={$activeCollectionId}
          onUpdate={(ep) => updateEndpoint($activeCollectionId!, ep)}
        />
      {:else}
        <div class="flex items-center justify-center h-full text-slate-500 text-sm">
          Select or create an endpoint
        </div>
      {/if}
    </div>
  </div>
{/if}
```

**Step 2: Commit**

```bash
git add src/lib/components/editor/index.svelte
git commit -m "feat: add editor layout component"
```

---

## Task 16: Server Panel & Log Panel

**Files:**
- Create: `src/lib/components/server/ServerPanel.svelte`
- Create: `src/lib/components/server/LogPanel.svelte`

**Step 1: Create `src/lib/components/server/ServerPanel.svelte`**

```svelte
<script lang="ts">
  import { Play, Square, Loader } from 'lucide-svelte';
  import { serverStatus, serverError, startServer, stopServer } from '$lib/stores/server';
  import { activeCollection, activeCollectionId, updateCollectionPort } from '$lib/stores/collections';

  async function toggle() {
    if ($serverStatus === 'running') {
      await stopServer();
    } else {
      await startServer();
    }
  }
</script>

<div class="flex items-center gap-4">
  <!-- Port -->
  <div class="flex items-center gap-2">
    <label class="text-xs text-slate-400">Port</label>
    <input
      type="number"
      min="1024"
      max="65535"
      value={$activeCollection?.port ?? 3000}
      onchange={(e) => {
        if ($activeCollectionId) updateCollectionPort($activeCollectionId, Number(e.currentTarget.value));
      }}
      disabled={$serverStatus === 'running'}
      class="w-20 bg-slate-700 text-white text-sm px-2 py-1 rounded font-mono
             focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:opacity-50"
    />
  </div>

  <!-- Start/Stop Button -->
  <button
    onclick={toggle}
    disabled={$serverStatus === 'starting' || !$activeCollection}
    class="flex items-center gap-2 px-4 py-1.5 rounded-lg text-sm font-medium transition-colors disabled:opacity-50
      {$serverStatus === 'running'
        ? 'bg-red-600/20 text-red-400 border border-red-500/30 hover:bg-red-600/30'
        : 'bg-green-600/20 text-green-400 border border-green-500/30 hover:bg-green-600/30'}"
  >
    {#if $serverStatus === 'starting'}
      <Loader size={14} class="animate-spin" />
      Starting...
    {:else if $serverStatus === 'running'}
      <Square size={14} />
      Stop Server
    {:else}
      <Play size={14} />
      Start Server
    {/if}
  </button>

  <!-- Status indicator -->
  {#if $serverStatus === 'running' && $activeCollection}
    <span class="text-xs font-mono text-green-400 flex items-center gap-1">
      <span class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"></span>
      localhost:{$activeCollection.port}
    </span>
  {/if}

  {#if $serverStatus === 'error' && $serverError}
    <span class="text-xs text-red-400">{$serverError}</span>
  {/if}
</div>
```

**Step 2: Create `src/lib/components/server/LogPanel.svelte`**

```svelte
<script lang="ts">
  import { logs, clearLogs } from '$lib/stores/logs';
  import { Trash2 } from 'lucide-svelte';

  const STATUS_COLORS: Record<number, string> = {};
  function statusColor(status: number): string {
    if (status < 300) return 'text-green-400';
    if (status < 400) return 'text-yellow-400';
    if (status < 500) return 'text-orange-400';
    return 'text-red-400';
  }

  const METHOD_COLORS: Record<string, string> = {
    GET: 'text-green-300',
    POST: 'text-yellow-300',
    PUT: 'text-blue-300',
    DELETE: 'text-red-300',
    PATCH: 'text-purple-300',
  };

  let logContainer: HTMLDivElement;

  $effect(() => {
    if ($logs.length > 0 && logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between px-4 py-2 border-b border-slate-700">
    <span class="text-xs font-medium text-slate-400 uppercase tracking-wider">Request Logs</span>
    <button
      onclick={clearLogs}
      class="p-1 hover:bg-slate-700 rounded transition-colors"
      title="Clear logs"
    >
      <Trash2 size={13} class="text-slate-500" />
    </button>
  </div>

  <div bind:this={logContainer} class="flex-1 overflow-y-auto font-mono text-xs p-2 space-y-0.5">
    {#each $logs as log (log.timestamp + log.path)}
      <div class="flex gap-3 px-2 py-1 rounded hover:bg-slate-800">
        <span class="text-slate-600 shrink-0">{new Date(log.timestamp).toLocaleTimeString()}</span>
        <span class="font-bold {METHOD_COLORS[log.method] ?? 'text-slate-300'} w-14 shrink-0">{log.method}</span>
        <span class="text-slate-300 flex-1 truncate">{log.path}</span>
        <span class="shrink-0 {statusColor(log.status)}">{log.status}</span>
        <span class="text-slate-600 shrink-0 w-16 text-right">{log.duration}ms</span>
      </div>
    {/each}

    {#if $logs.length === 0}
      <p class="text-slate-600 text-center mt-6">Requests will appear here once the server is running</p>
    {/if}
  </div>
</div>
```

**Step 3: Commit**

```bash
git add src/lib/components/server/
git commit -m "feat: add server panel and log panel components"
```

---

## Task 17: App Root

**Files:**
- Create: `src/App.svelte`

**Step 1: Create `src/App.svelte`**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from '$lib/components/sidebar/index.svelte';
  import Editor from '$lib/components/editor/index.svelte';
  import ServerPanel from '$lib/components/server/ServerPanel.svelte';
  import LogPanel from '$lib/components/server/LogPanel.svelte';
  import { loadCollections, activeCollection } from '$lib/stores/collections';
  import { initLogListener } from '$lib/stores/logs';

  onMount(async () => {
    await loadCollections();
    await initLogListener();
  });
</script>

<div class="h-screen bg-slate-900 text-white flex overflow-hidden">
  <!-- Sidebar -->
  <div class="w-56 shrink-0 bg-slate-800 border-r border-slate-700 flex flex-col">
    <Sidebar />
  </div>

  <!-- Main -->
  <div class="flex-1 flex flex-col min-w-0">
    <!-- Top bar: server controls -->
    <div class="shrink-0 px-4 py-2.5 bg-slate-800 border-b border-slate-700">
      <ServerPanel />
    </div>

    <!-- Editor area -->
    <div class="flex-1 min-h-0">
      {#if $activeCollection}
        <Editor />
      {:else}
        <div class="flex flex-col items-center justify-center h-full text-slate-500">
          <p class="text-lg font-medium">No collection selected</p>
          <p class="text-sm mt-1">Create or select a collection from the sidebar</p>
        </div>
      {/if}
    </div>

    <!-- Log panel -->
    <div class="h-48 shrink-0 bg-slate-800 border-t border-slate-700">
      <LogPanel />
    </div>
  </div>
</div>
```

**Step 2: Commit**

```bash
git add src/App.svelte
git commit -m "feat: add app root with full layout"
```

---

## Task 18: Smoke Test

**Step 1: Start dev server**

```bash
npm run tauri:dev
```

Expected: App window opens

**Step 2: Verify basic flow**

- [ ] Create collection "Test API"
- [ ] Add `GET /users/:id` endpoint
- [ ] Set body to `{"id": ":id", "name": "John"}`
- [ ] Start server on port 3000
- [ ] Run: `curl http://localhost:3000/users/123`
- [ ] Expected response: `{"id": "123", "name": "John"}`
- [ ] Logs show the request in LogPanel
- [ ] Stop server
- [ ] Restart app — collection persists

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete tarasque MVP"
```

---

## Summary

| Task | Component | Est. Files |
|---|---|---|
| 1 | Project scaffold | 11 |
| 2 | Rust models | 1 |
| 3 | Storage module | 1 |
| 4 | Server state/handle | 2 |
| 5 | Path router + tests | 1 |
| 6 | Actix server | 1 |
| 7 | Tauri commands | 2 |
| 8 | TypeScript types | 1 |
| 9 | Collections store | 1 |
| 10 | Server + logs stores | 2 |
| 11 | Sidebar components | 2 |
| 12 | Monaco component | 1 |
| 13 | Scenario editor | 1 |
| 14 | Endpoint list + detail | 2 |
| 15 | Editor index | 1 |
| 16 | Server + log panels | 2 |
| 17 | App root | 1 |
| 18 | Smoke test | — |
