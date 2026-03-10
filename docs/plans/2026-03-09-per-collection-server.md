# Per-Collection Server Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow each collection to run its own independent mock server simultaneously, with auto-detecting free ports and a per-collection run/stop control.

**Architecture:** The global `ServerHandle` becomes a `HashMap<collection_id, handles>` managed behind a `Mutex`. The frontend server store changes from single-status scalars to per-collection maps. The run/stop button stays in the top bar but reflects the *active* collection's status; the sidebar shows a live indicator on every running collection.

**Tech Stack:** Rust / Tauri v2 · actix-web 4 · tokio · Svelte 5 runes · TypeScript

---

## Current State (read before touching anything)

| File | Relevant bit |
|------|-------------|
| `src-tauri/src/server/state.rs` | `ServerHandle` — two `Mutex<Option<Sender>>` fields, one HTTP one HTTPS |
| `src-tauri/src/commands.rs` | `start_server`, `stop_server` — operate on the single global handle |
| `src-tauri/src/server/mod.rs` | `start()` / `stop()` — take `&ServerHandle` |
| `src/lib/stores/server.ts` | `serverStatus` / `serverError` — single writable scalars |
| `src/lib/components/server/ServerPanel.svelte` | Port input + Run/Stop button, reads global status |
| `src/lib/components/sidebar/CollectionItem.svelte` | Shows `:port` badge, no running indicator |

---

## Task 1: Rust — Replace `ServerHandle` with `MultiServerHandle`

**Files:**
- Modify: `src-tauri/src/server/state.rs`

**Step 1: Replace the struct**

```rust
// src-tauri/src/server/state.rs
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
```

**Step 2: Verify it compiles (will fail due to downstream breakage — that's expected)**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd src-tauri && cargo check 2>&1 | grep "^error"
```
Expected: several errors in `commands.rs` and `server/mod.rs` referencing the old `ServerHandle`. Proceed.

---

## Task 2: Rust — Update `server/mod.rs` to use `MultiServerHandle`

**Files:**
- Modify: `src-tauri/src/server/mod.rs`

**Step 1: Update `start()` signature and body**

Replace the existing `start()` and `stop()` functions:

```rust
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
```

**Step 2: Add the import at the top of `server/mod.rs`**

```rust
use state::{AppState, CollectionHandle, MultiServerHandle};
```

(Remove the old `ServerHandle` import.)

**Step 3: Check**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd src-tauri && cargo check 2>&1 | grep "^error"
```
Expected: only errors in `commands.rs` now.

---

## Task 3: Rust — Update `commands.rs` + add `find_free_port`

**Files:**
- Modify: `src-tauri/src/commands.rs`

**Step 1: Update imports and `start_server` / `stop_server`**

```rust
use crate::server::state::MultiServerHandle;

#[tauri::command]
pub async fn start_server(
    app: AppHandle,
    multi: State<'_, MultiServerHandle>,
    collection_id: String,
    data: AppData,
) -> Result<(), String> {
    storage::save(&app, &data).await?;

    let collection = data
        .collections
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| format!("Collection '{collection_id}' not found"))?;

    server::start(collection, app.clone(), &multi).await
}

#[tauri::command]
pub async fn stop_server(
    multi: State<'_, MultiServerHandle>,
    collection_id: String,
) -> Result<(), String> {
    server::stop(&collection_id, &multi).await
}

#[tauri::command]
pub async fn get_running_collections(
    multi: State<'_, MultiServerHandle>,
) -> Result<Vec<String>, String> {
    Ok(server::running_ids(&multi).await)
}

/// Find a free TCP port by binding to port 0 and reading the assigned port.
#[tauri::command]
pub fn find_free_port() -> Result<u16, String> {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to find free port: {e}"))?;
    let port = listener.local_addr()
        .map_err(|e| format!("Failed to read port: {e}"))?
        .port();
    Ok(port)
}
```

**Step 2: Update `main.rs` — swap `ServerHandle` for `MultiServerHandle` and register new commands**

```rust
// src-tauri/src/main.rs
use server::state::MultiServerHandle;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(MultiServerHandle::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_data,
            commands::save_data,
            commands::start_server,
            commands::stop_server,
            commands::get_running_collections,
            commands::find_free_port,
            commands::import_openapi,
            commands::export_collection_json,
            commands::export_collection_postman,
            commands::run_test_suite,
            commands::generate_alias_cert,
            commands::trust_cert_macos,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
```

**Step 3: Verify clean compile**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd src-tauri && cargo check 2>&1
```
Expected: `Finished` with no errors. Fix any remaining type issues before continuing.

---

## Task 4: Frontend — Per-collection server store

**Files:**
- Modify: `src/lib/stores/server.ts`

**Step 1: Rewrite the store**

```typescript
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCollectionId, collections } from './collections';
import type { AppData } from '$lib/types';

export type ServerStatus = 'stopped' | 'starting' | 'running' | 'error';

// Map of collectionId → status / error
export const serverStatuses = writable<Record<string, ServerStatus>>({});
export const serverErrors   = writable<Record<string, string | null>>({});

/** Reactive status for the currently active collection */
export const activeServerStatus = derived(
  [serverStatuses, activeCollectionId],
  ([$s, $id]) => ($id ? ($s[$id] ?? 'stopped') : 'stopped')
);

export const activeServerError = derived(
  [serverErrors, activeCollectionId],
  ([$e, $id]) => ($id ? ($e[$id] ?? null) : null)
);

function setStatus(id: string, status: ServerStatus) {
  serverStatuses.update((s) => ({ ...s, [id]: status }));
}

function setError(id: string, err: string | null) {
  serverErrors.update((e) => ({ ...e, [id]: err }));
}

export async function startServer(collectionId: string) {
  const cols = get(collections);
  const data: AppData = {
    collections: cols,
    active_collection_id: collectionId,
  };

  setStatus(collectionId, 'starting');
  setError(collectionId, null);

  const timeout = new Promise<never>((_, reject) =>
    setTimeout(() => reject('Server start timed out after 15s'), 15_000)
  );

  try {
    await Promise.race([
      invoke('start_server', { collectionId, data }),
      timeout,
    ]);
    setStatus(collectionId, 'running');
  } catch (e) {
    setStatus(collectionId, 'error');
    setError(collectionId, String(e));
  }
}

export async function stopServer(collectionId: string) {
  try {
    await invoke('stop_server', { collectionId });
    setStatus(collectionId, 'stopped');
    setError(collectionId, null);
  } catch (e) {
    setError(collectionId, String(e));
  }
}

/** Sync running state from Rust on app load */
export async function syncRunningServers() {
  try {
    const ids = await invoke<string[]>('get_running_collections');
    serverStatuses.update((s) => {
      const next = { ...s };
      for (const id of ids) next[id] = 'running';
      return next;
    });
  } catch {
    // non-fatal
  }
}

export async function findFreePort(): Promise<number> {
  return invoke<number>('find_free_port');
}
```

---

## Task 5: UI — Update `ServerPanel.svelte` for per-collection control

**Files:**
- Modify: `src/lib/components/server/ServerPanel.svelte`

**Step 1: Replace the store imports and toggle logic**

```svelte
<script lang="ts">
  import { Play, Square, Loader2, Shuffle } from 'lucide-svelte';
  import { activeServerStatus, activeServerError, startServer, stopServer, findFreePort } from '$lib/stores/server';
  import { activeCollection, activeCollectionId, updateCollectionPort } from '$lib/stores/collections';

  async function toggle() {
    const id = $activeCollectionId;
    if (!id) return;
    if ($activeServerStatus === 'running') {
      await stopServer(id);
    } else {
      await startServer(id);
    }
  }

  async function autoPort() {
    if (!$activeCollectionId || $activeServerStatus === 'running') return;
    try {
      const port = await findFreePort();
      updateCollectionPort($activeCollectionId, port);
    } catch {/* ignore */}
  }
</script>

<div class="flex items-center gap-4 h-full w-full font-inter">
  <!-- Port input + auto button -->
  <div class="flex items-center gap-1.5">
    <span class="text-[10px] font-krub text-ink-muted uppercase tracking-widest">Port</span>
    <input
      id="server-port"
      type="number"
      min="1024"
      max="65535"
      value={$activeCollection?.port ?? 3000}
      onchange={(e) => {
        if ($activeCollectionId) updateCollectionPort($activeCollectionId, Number(e.currentTarget.value));
      }}
      disabled={$activeServerStatus === 'running'}
      class="w-16 bg-surface text-ink text-xs font-mono px-2 py-1 border border-line-strong rounded
             focus:outline-none focus:border-accent transition-colors duration-150
             disabled:opacity-30 disabled:cursor-not-allowed"
    />
    <button
      onclick={autoPort}
      disabled={$activeServerStatus === 'running' || !$activeCollection}
      class="w-6 h-6 flex items-center justify-center rounded text-ink-dim hover:text-ink
             hover:bg-hover transition-colors duration-150 disabled:opacity-30 disabled:cursor-not-allowed"
      title="Find a free port"
    >
      <Shuffle size={11} />
    </button>
  </div>

  <!-- Divider -->
  <div class="w-px h-4 bg-line-strong"></div>

  <!-- Run / Stop button -->
  <button
    onclick={toggle}
    disabled={$activeServerStatus === 'starting' || !$activeCollection}
    class="flex items-center gap-1.5 h-7 px-3 rounded text-xs font-krub font-medium
           transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed
      {$activeServerStatus === 'running'
        ? 'bg-err/10 text-err border border-err/20 hover:bg-err/20'
        : 'bg-run/10 text-run border border-run/20 hover:bg-run/20'}"
  >
    {#if $activeServerStatus === 'starting'}
      <Loader2 size={11} class="animate-spin" />
      <span>starting</span>
    {:else if $activeServerStatus === 'running'}
      <Square size={11} />
      <span>stop</span>
    {:else}
      <Play size={11} />
      <span>run</span>
    {/if}
  </button>

  <!-- Running indicator -->
  {#if $activeServerStatus === 'running' && $activeCollection}
    <div class="flex items-center gap-1.5 text-[11px] font-mono text-run">
      <span class="w-1.5 h-1.5 rounded-full bg-run animate-pulse inline-block"></span>
      <span>localhost:{$activeCollection.port}</span>
    </div>
  {/if}

  <!-- Error -->
  {#if $activeServerStatus === 'error' && $activeServerError}
    <span class="text-[11px] text-err font-mono max-w-sm overflow-hidden" title={$activeServerError}>
      {$activeServerError}
    </span>
  {/if}
</div>
```

---

## Task 6: UI — CollectionItem running indicator

**Files:**
- Modify: `src/lib/components/sidebar/CollectionItem.svelte`

**Step 1: Add a reactive running indicator dot**

Import the store and derive the running state for this collection:

```svelte
<script lang="ts">
  import { Trash2, FolderOpen, Folder, Settings } from 'lucide-svelte';
  import { serverStatuses } from '$lib/stores/server';
  import type { Collection } from '$lib/types';

  interface Props {
    collection: Collection;
    active: boolean;
    onSelect: () => void;
    onDelete: () => void;
    onSettings: () => void;
  }

  let { collection, active, onSelect, onDelete, onSettings }: Props = $props();

  let running = $derived($serverStatuses[collection.id] === 'running');
  let starting = $derived($serverStatuses[collection.id] === 'starting');
</script>

<button
  class="w-full text-left px-3 py-2 flex items-center gap-2 group transition-colors duration-150 border-l-[2px]
    {active
      ? 'border-accent bg-hover text-ink'
      : 'border-transparent hover:bg-hover/60 text-ink-muted hover:text-ink'}"
  onclick={onSelect}
>
  {#if active}
    <FolderOpen size={13} class="shrink-0 text-accent" />
  {:else}
    <Folder size={13} class="shrink-0 text-ink-dim group-hover:text-ink-muted transition-colors duration-150" />
  {/if}

  <span class="flex-1 truncate text-xs font-inter">{collection.name}</span>

  <!-- Running / starting indicator -->
  {#if running}
    <span class="w-1.5 h-1.5 rounded-full bg-run animate-pulse shrink-0" title="Server running"></span>
  {:else if starting}
    <span class="w-1.5 h-1.5 rounded-full bg-warn animate-pulse shrink-0" title="Starting…"></span>
  {/if}

  <span class="text-[9px] font-mono text-ink-dim tabular-nums px-1.5 py-0.5 rounded bg-surface shrink-0">
    :{collection.port}
  </span>

  {#if collection.proxy_enabled}
    <span class="text-[9px] font-mono text-amber-400 shrink-0" title="Proxy enabled">↗</span>
  {/if}

  <div
    role="button"
    tabindex="0"
    class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center rounded
           text-ink-dim hover:text-ink hover:bg-hover transition-all duration-150 shrink-0"
    onclick={(e) => { e.stopPropagation(); onSettings(); }}
    onkeydown={(e) => e.key === 'Enter' && onSettings()}
  >
    <Settings size={10} />
  </div>

  <div
    role="button"
    tabindex="0"
    class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center rounded
           text-ink-dim hover:text-err hover:bg-err/10 transition-all duration-150 shrink-0"
    onclick={(e) => { e.stopPropagation(); onDelete(); }}
    onkeydown={(e) => e.key === 'Enter' && onDelete()}
  >
    <Trash2 size={10} />
  </div>
</button>
```

---

## Task 7: Wiring — `App.svelte` init + delete-collection cleanup

**Files:**
- Modify: `src/App.svelte`
- Modify: `src/lib/stores/collections.ts`

**Step 1: Call `syncRunningServers` on mount in `App.svelte`**

```svelte
// In the onMount block, after loadCollections():
import { syncRunningServers } from '$lib/stores/server';

onMount(async () => {
  await loadCollections();
  await initLogListener();
  await syncRunningServers();   // ← add this
  settings.load();
  applyTheme(get(settings).theme);
});
```

**Step 2: Stop server when a collection is deleted**

In `src/lib/stores/collections.ts`, update `deleteCollection`:

```typescript
import { stopServer, serverStatuses, serverErrors } from './server';

export async function deleteCollection(id: string) {
  // Stop the server if it's running for this collection
  const status = get(serverStatuses)[id];
  if (status === 'running' || status === 'starting') {
    try { await stopServer(id); } catch { /* ignore */ }
  }
  // Remove from status maps
  serverStatuses.update((s) => { const n = { ...s }; delete n[id]; return n; });
  serverErrors.update((e)   => { const n = { ...e }; delete n[id]; return n; });

  collections.update((c) => c.filter((col) => col.id !== id));
  activeCollectionId.update((current) => (current === id ? null : current));
  scheduleAutoSave();
}
```

**Step 3: Final compile check**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd src-tauri && cargo check 2>&1
```
Expected: `Finished` with zero errors.

---

## Task 8: Manual smoke test

1. Start dev: `bun run tauri:dev` (with cargo in PATH)
2. Create **collection-a** — click auto-port (🔀), should pick a free port
3. Create **collection-b** — click auto-port, different port
4. With collection-a active → press **run** → green dot appears on collection-a in sidebar, top bar shows `localhost:<port>` for collection-a
5. Switch to collection-b → top bar shows collection-b's status (stopped) — collection-a dot still green
6. Press **run** on collection-b → both dots green simultaneously
7. `curl http://localhost:<portA>/anything` and `curl http://localhost:<portB>/anything` — both return 404 JSON
8. Delete collection-b → its server stops automatically, no orphan processes
9. Stop collection-a → green dot disappears

---

## Confidence Level

**90%**

High confidence because:
- All pieces are well-understood: HashMap state in Rust, derived stores in Svelte 5
- `find_free_port` via `TcpListener::bind("0")` is a one-liner pattern
- No new dependencies required

Main risk (10%):
- Svelte 5 `$derived` on a store-field lookup (`$serverStatuses[collection.id]`) — reactivity fires on the whole-object update, which is correct as long as we always replace the object (`{ ...s, [id]: status }`) rather than mutating it in place
- Circular import between `collections.ts` and `server.ts` (deleteCollection calling stopServer) — resolve by importing `stopServer` lazily inside the function body if needed
