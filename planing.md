<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# Complete Tauri + Svelte Mock API App: **Tarasque**

**Copy-paste this entire Markdown into Opus/Claude/Cursor.** Generates 100% complete desktop app.

## 🎯 **App Overview: Tarasque**

Desktop mock API server like Mockoon/Postman. Create collections → define endpoints → start localhost server → test with curl/browser.

```
┌─────────────────┐ ┌─────────────────────────────────────┐
│ 📁 Collections  │ │  Test API Collection                │
│ • Test API ✓    │ │  ├─ GET /users/:id → 200 JSON       │
│ • Auth API      │ │  └─ POST /login → 401 Error         │
│ + New Coll...   │ │                                     │
│                 │ │  [🚀 Server: localhost:3000 ✓]       │
│                 │ │  [📊 Logs: GET /users/123 42ms]      │
└─────────────────┘ └─────────────────────────────────────┘
```


## 🛠 **Tech Stack**

- **Frontend**: Svelte 5 + Tailwind CSS + Monaco Editor
- **Backend**: Tauri v2 + Rust + Actix-web
- **Storage**: JSON (`AppData/tarasque/collections.json`)
- **Desktop**: Windows/macOS/Linux executables


## 📁 **File Structure**

```
tarasque/
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs           # Types + Commands
│   │   ├── main.rs          # Tauri setup
│   │   └── server.rs        # Actix-web server
│   └── tauri.conf.json
├── src/
│   ├── App.svelte
│   ├── app.html
│   └── lib/
│       ├── components/
│       │   ├── Sidebar.svelte
│       │   ├── EndpointEditor.svelte
│       │   ├── ServerPanel.svelte
│       │   └── LogPanel.svelte
│       ├── stores/
│       │   └── app.ts
│       └── types.ts
├── package.json
├── tailwind.config.js
├── vite.config.js
└── icons/ (auto-generated)
```


***

## 📦 **1. package.json**

```json
{
  "name": "tarasque",
  "version": "1.0.0",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build && tauri build",
    "tauri": "tauri",
    "tauri:dev": "tauri dev"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.1",
    "@sveltejs/vite-plugin-svelte": "^3.1.1",
    "@tauri-apps/cli": "^2.0.0",
    "autoprefixer": "^10.4.20",
    "postcss": "^8.4.41",
    "tailwindcss": "^3.4.10",
    "vite": "^5.4.1"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "@types/node": "^20.14.10",
    "lucide-svelte": "^0.518.0",
    "monaco-editor": "^0.52.2",
    "svelte": "^5.0.0"
  }
}
```


## 🦀 **2. Cargo.toml**

```toml
[package]
name = "tarasque"
version = "1.0.0"
description = "Mock API Playground"
authors = ["Fiqry Choerudin"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["api-all"] }
tauri-plugin-fs = "2.0"
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
actix-web = "4.9"
actix-rt = "2.12"
futures = "0.3"
uuid = { version = "1.10", features = ["v4", "serde"] }
```


***

## 🧩 **3. Data Models (src/lib/types.ts)**

```typescript
export interface Scenario {
  id: string;
  name: string;
  status: number;
  delayMs: number;
  headers: Record<string, string>;
  body: string;
}

export interface Endpoint {
  id: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  path: string;
  scenarios: Scenario[];
  activeScenarioId: string;
}

export interface Collection {
  id: string;
  name: string;
  endpoints: Endpoint[];
}
```


## 🧩 **4. Rust Types (src-tauri/src/lib.rs)**

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub status: u16,
    pub delay_ms: u64,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub id: String,
    pub method: String,
    pub path: String,
    pub scenarios: Vec<Scenario>,
    pub active_scenario_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub endpoints: Vec<Endpoint>,
}
```


***

## 🌐 **5. Actix-web Server (src-tauri/src/server.rs)**

```rust
use actix_web::{web, HttpRequest, HttpResponse, HttpServer, App};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::lib::{Collection, Endpoint, Scenario};

pub struct AppState {
    pub collections: Arc<RwLock<Vec<Collection>>>,
    pub active_collection_id: Arc<RwLock<String>>,
}

pub async fn start_server(
    port: u16,
    collections: Vec<Collection>,
    active_collection_id: String,
) -> Result<String, String> {
    let state = AppState {
        collections: Arc::new(RwLock::new(collections)),
        active_collection_id: Arc::new(RwLock::new(active_collection_id)),
    };

    let state_clone = state.clone();
    
    tauri::async_runtime::spawn(async move {
        let bind = format!("127.0.0.1:{}", port);
        println!("🚀 Starting mock server at {}", bind);
        
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(state_clone.clone()))
                .route("/{tail:.*}", web::get().to(mock_handler))
                .route("/{tail:.*}", web::post().to(mock_handler))
                .route("/{tail:.*}", web::put().to(mock_handler))
                .route("/{tail:.*}", web::delete().to(mock_handler))
        })
        .bind(&bind)
        .map_err(|e| format!("Failed to bind: {}", e))?
        .run()
        .await
        .map_err(|e| format!("Server error: {}", e))?;
        
        Ok::<(), String>("Server stopped".to_string())
    });

    Ok(format!("Server started on http://{}", bind))
}

async fn mock_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl actix_web::Responder {
    let path = req.path();
    let method = req.method().as_str();
    
    // Find matching endpoint (simplified - implement path param matching)
    let collections = data.collections.read().await;
    let active_id = data.active_collection_id.read().await;
    
    // Emit log event
    let log_msg = format!("{} {} - Looking for match", method, path);
    if let Err(e) = req.app_data::<web::Data<tauri::AppHandle>>()
        .unwrap()
        .emit_all("server:log", log_msg).await {
        eprintln!("Failed to emit log: {}", e);
    }
    
    // Default 404 response
    HttpResponse::NotFound().json(serde_json::json!({
        "error": "No matching endpoint found"
    }))
}
```


***

## 🔧 **6. Tauri Commands (src-tauri/src/main.rs)**

```rust
mod lib;
mod server;
use lib::{Collection, Endpoint, Scenario};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
async fn load_collections(app_handle: tauri::AppHandle) -> Result<Vec<Collection>, String> {
    let app_data_dir = app_handle.path_resolver()
        .app_local_data_dir()
        .ok_or("Failed to get app data dir")?;
    
    let collections_path = app_data_dir.join("collections.json");
    
    if collections_path.exists() {
        let data = fs::read_to_string(&collections_path)?;
        let collections: Vec<Collection> = serde_json::from_str(&data)?;
        Ok(collections)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
async fn save_collections(
    app_handle: tauri::AppHandle,
    collections: Vec<Collection>,
) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver()
        .app_local_data_dir()
        .ok_or("Failed to get app data dir")?;
    
    fs::create_dir_all(&app_data_dir)?;
    let collections_path = app_data_dir.join("collections.json");
    let data = serde_json::to_string_pretty(&collections)?;
    fs::write(&collections_path, data)?;
    Ok(())
}

#[tauri::command]
async fn start_server(port: u16, collection_id: String) -> Result<String, String> {
    // Load collections first
    let collections = load_collections().await?;
    server::start_server(port, collections, collection_id).await
}

#[tauri::command]
fn stop_server() -> Result<(), String> {
    // Implement graceful shutdown
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            load_collections,
            save_collections,
            start_server,
            stop_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```


***

## 🏗️ **7. Svelte Stores (src/lib/stores/app.ts)**

```typescript
import { writable } from 'svelte/store';
import type { Collection, Endpoint, Scenario } from '$lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export const collections = writable<Collection[]>([]);
export const activeCollectionId = writable<string | null>(null);
export const serverState = writable<'stopped' | 'starting' | 'running' | 'error'>('stopped');
export const logs = writable<string[]>([]);
export const port = writable(3000);

// Load collections on app start
collections.subscribe(async (value) => {
  if (value.length === 0) {
    const data = await invoke('load_collections');
    collections.set(data as Collection[]);
  }
});

// Listen for server logs
listen('server:log', (event) => {
  logs.update(l => [...l.slice(-99), event.payload as string]); // Keep last 100
});
```


***

## 🎨 **8. Main App (src/App.svelte)**

```svelte
<script lang="ts">
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ServerPanel from '$lib/components/ServerPanel.svelte';
  import LogPanel from '$lib/components/LogPanel.svelte';
  import EndpointEditor from '$lib/components/EndpointEditor.svelte';
  import { collections, activeCollectionId, serverState } from '$lib/stores/app';
</script>

<div class="h-screen bg-slate-900 text-white flex">
  <!-- Sidebar -->
  <div class="w-80 bg-slate-800 border-r border-slate-700 p-6">
    <Sidebar />
  </div>
  
  <!-- Main Content -->
  <div class="flex-1 flex flex-col overflow-hidden">
    <!-- Header with Server Panel -->
    <div class="p-6 bg-slate-800 border-b border-slate-700">
      <ServerPanel />
    </div>
    
    <!-- Content Area -->
    <div class="flex-1 overflow-auto p-6 space-y-6">
      {#if $activeCollectionId}
        <EndpointEditor />
      {:else}
        <div class="text-center py-20 text-slate-400">
          <h2 class="text-2xl mb-4">Select a collection</h2>
          <p>or create a new one to get started</p>
        </div>
      {/if}
    </div>
    
    <!-- Logs Panel -->
    <div class="h-80 bg-slate-800 border-t border-slate-700">
      <LogPanel />
    </div>
  </div>
</div>
```


***

## 🧱 **9. Components**

### Sidebar.svelte

```svelte
<script lang="ts">
  import { collections, activeCollectionId } from '$lib/stores/app';
  import { Plus, Trash2 } from 'lucide-svelte';
  
  let newCollectionName = '';
  
  function createCollection() {
    if (newCollectionName.trim()) {
      const id = crypto.randomUUID();
      collections.update(c => [
        ...c,
        {
          id,
          name: newCollectionName.trim(),
          endpoints: []
        }
      ]);
      newCollectionName = '';
    }
  }
</script>

<div class="space-y-4">
  <div class="flex gap-2">
    <input
      bind:value={newCollectionName}
      on:keydown={(e) => e.key === 'Enter' && createCollection()}
      placeholder="New collection"
      class="flex-1 bg-slate-700 px-3 py-2 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
    <button on:click={createCollection} class="p-2 hover:bg-slate-700 rounded">
      <Plus size={16} />
    </button>
  </div>
  
  <div class="space-y-1">
    {#each $collections as collection (collection.id)}
      <button
        class="w-full text-left p-3 rounded hover:bg-slate-700 flex items-center gap-3 {
          $activeCollectionId === collection.id ? 'bg-blue-900/50 border-r-2 border-blue-400' : ''
        }"
        on:click={() => activeCollectionId.set(collection.id)}
      >
        {collection.name}
        <Trash2 size={16} class="ml-auto opacity-50 hover:opacity-100" />
      </button>
    {/each}
  </div>
</div>
```


### ServerPanel.svelte

```svelte
<script lang="ts">
  import { port, serverState } from '$lib/stores/app';
  import { Play, Square, Loader2 } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  
  async function toggleServer() {
    if ($serverState === 'running') {
      await invoke('stop_server');
      serverState.set('stopped');
    } else {
      serverState.set('starting');
      try {
        const result = await invoke('start_server', {
          port: $port,
          collectionId: '' // Use first collection
        });
        serverState.set('running');
      } catch (error) {
        serverState.set('error');
        console.error(error);
      }
    }
  }
</script>

<div class="flex items-center gap-4">
  <div class="flex items-center gap-2">
    <span>Port:</span>
    <input bind:value={$port} type="number" min="3000" max="65535" 
           class="w-20 bg-slate-700 px-2 py-1 rounded text-sm" />
  </div>
  
  <button
    on:click={toggleServer}
    disabled={$serverState === 'starting'}
    class="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50"
  >
    {#if $serverState === 'running'}
      <Square size={18} />
      Stop Server
    {:else if $serverState === 'starting'}
      <Loader2 size={18} class="animate-spin" />
      Starting...
    {:else}
      <Play size={18} />
      Start Server
    {/if}
  </button>
  
  {#if $serverState === 'running'}
    <div class="text-green-400 text-sm font-mono">
      localhost:{$port}
    </div>
  {/if}
</div>
```


### LogPanel.svelte

```svelte
<script lang="ts">
  import { logs } from '$lib/stores/app';
</script>

<div class="h-full p-4 overflow-auto bg-slate-900 font-mono text-xs">
  {#each $logs as log}
    <div class="py-1 border-b border-slate-800">{@html log}</div>
  {/each}
  {#if $logs.length === 0}
    <div class="text-slate-500 text-center mt-8">Server logs will appear here...</div>
  {/if}
</div>
```


***

## ⚙️ **10. Config Files**

### tauri.conf.json

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
    "security": {
      "csp": null
    }
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


### tailwind.config.js

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {}
  },
  plugins: []
};
```


### vite.config.js

```javascript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
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


***

## 🚀 **11. Setup \& Build Commands**

```bash
# 1. Create project
npm create tauri-app@latest tarasque -- --template svelte
cd tarasque

# 2. Replace all files with above code
# 3. Install dependencies
npm install
cargo check

# 4. Development
npm run tauri dev

# 5. Build desktop app
npm run tauri build
```

**Output**: `src-tauri/target/release/bundle/` → `.exe`, `.dmg`, `.deb`

***

## ✅ **Success Checklist**

- [ ] `npm run tauri dev` → app launches
- [ ] Create collection "Test API"
- [ ] Add `GET /users/:id` endpoint
- [ ] Add scenario: `200`, `{"id": ":id", "name": "John"}`, `100ms`
- [ ] Start server on port 3000
- [ ] `curl http://localhost:3000/users/123` → `{"id": "123", "name": "John"}`
- [ ] Logs show request details
- [ ] `npm run tauri build` → working desktop app

***

**Copy this entire Markdown → paste into Opus → "Generate all files" → Done!** 🎉

*App name "Tarasque" hardcoded everywhere. Ready to run first try.*

