# HTTPS Proxy (Docker + nginx) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the existing pf-based HTTPS alias with a single shared nginx Docker container that routes `{hostname}:443` → `localhost:{collection.port}`, with auto and manual cert modes.

**Architecture:** rcgen generates self-signed certs and writes them to `AppData/proxy/cert/{hostname}/`. A new `proxy` Rust module generates nginx configs and a docker-compose.yml then shells out to `docker compose up -d`. The CollectionSettingsModal gains auto/manual cert modes; the sidebar gains a persistent nginx status bar.

**Tech Stack:** Rust (rcgen already in Cargo.toml), `std::process::Command` for docker/security CLI, Svelte 5 proxy store, nginx:1.25-alpine Docker image.

---

### Task 1: Add nginx template files (embedded as Rust string literals)

**Files:**
- Create: `src-tauri/templates/nginx.conf`
- Create: `src-tauri/templates/server.conf.template`
- Create: `src-tauri/templates/docker-compose.yml.template`

**Step 1: Create `src-tauri/templates/nginx.conf`**

```nginx
worker_processes auto;

events {
    worker_connections 20000;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    keepalive_timeout 300;
    sendfile on;

    map $http_upgrade $connection_upgrade {
        default upgrade;
        '' close;
    }

    include /etc/nginx/conf.d/*.conf;
}
```

**Step 2: Create `src-tauri/templates/server.conf.template`**

```nginx
upstream {UPSTREAM_NAME} {
    server host.docker.internal:{PORT};
}

server {
    listen 443 ssl;
    server_name {HOSTNAME};

    ssl_certificate     /usr/local/ssl/cert/{HOSTNAME}/cert.pem;
    ssl_certificate_key /usr/local/ssl/cert/{HOSTNAME}/private.key;

    location / {
        proxy_pass http://{UPSTREAM_NAME};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
        proxy_buffers 16 4k;
        proxy_buffer_size 4k;
    }
}
```

**Step 3: Create `src-tauri/templates/docker-compose.yml.template`**

```yaml
version: '3'
name: tarasqueproxy
services:
  nginx:
    image: nginx:1.25-alpine
    container_name: tarasque-nginx
    ports:
      - "443:443"
    volumes:
      - {CONF_DIR}/nginx.conf:/etc/nginx/nginx.conf
      - {CONF_DIR}/conf.d:/etc/nginx/conf.d
      - {CERT_DIR}:/usr/local/ssl/cert
    extra_hosts:
      - "host.docker.internal:host-gateway"
```

**Step 4: Verify the 3 files exist**

```bash
ls src-tauri/templates/
```
Expected: `docker-compose.yml.template  nginx.conf  server.conf.template`

---

### Task 2: Create the proxy Rust module

**Files:**
- Create: `src-tauri/src/proxy.rs`

**Step 1: Write `src-tauri/src/proxy.rs`**

```rust
use std::path::PathBuf;
use tauri::AppHandle;

// Templates embedded at compile time — no runtime file reads needed
const NGINX_CONF: &str = include_str!("../templates/nginx.conf");
const SERVER_CONF_TEMPLATE: &str = include_str!("../templates/server.conf.template");
const DOCKER_COMPOSE_TEMPLATE: &str = include_str!("../templates/docker-compose.yml.template");

/// Root directory for all proxy-related files in AppData
fn proxy_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {e}"))?;
    Ok(base.join("proxy"))
}

/// Write cert.pem and private.key for a hostname to AppData/proxy/cert/{hostname}/
pub fn write_cert_files(
    app: &AppHandle,
    hostname: &str,
    cert_pem: &str,
    key_pem: &str,
) -> Result<(), String> {
    let cert_dir = proxy_dir(app)?.join("cert").join(hostname);
    std::fs::create_dir_all(&cert_dir)
        .map_err(|e| format!("Failed to create cert dir: {e}"))?;
    std::fs::write(cert_dir.join("cert.pem"), cert_pem)
        .map_err(|e| format!("Failed to write cert.pem: {e}"))?;
    std::fs::write(cert_dir.join("private.key"), key_pem)
        .map_err(|e| format!("Failed to write private.key: {e}"))?;
    Ok(())
}

/// Generate nginx conf.d/{hostname}.conf for a single collection alias
fn server_conf(hostname: &str, port: u16) -> String {
    // nginx upstream names cannot contain dots — replace with underscores
    let upstream_name = hostname.replace('.', "_");
    SERVER_CONF_TEMPLATE
        .replace("{HOSTNAME}", hostname)
        .replace("{PORT}", &port.to_string())
        .replace("{UPSTREAM_NAME}", &upstream_name)
}

/// Write all nginx configs and docker-compose.yml, then run `docker compose up -d`.
/// `collections` is a list of (hostname, collection_port) for every alias_enabled collection
/// that has cert files on disk.
pub fn start_proxy(
    app: &AppHandle,
    aliases: &[(String, u16)],
) -> Result<(), String> {
    if aliases.is_empty() {
        return Err("No HTTPS aliases are enabled — enable at least one collection alias first".into());
    }

    let proxy = proxy_dir(app)?;
    let conf_dir = proxy.join("conf");
    let conf_d_dir = conf_dir.join("conf.d");
    let cert_dir = proxy.join("cert");

    std::fs::create_dir_all(&conf_d_dir)
        .map_err(|e| format!("Failed to create conf.d dir: {e}"))?;

    // Write main nginx.conf
    std::fs::write(conf_dir.join("nginx.conf"), NGINX_CONF)
        .map_err(|e| format!("Failed to write nginx.conf: {e}"))?;

    // Write per-hostname server configs
    for (hostname, port) in aliases {
        let conf = server_conf(hostname, *port);
        std::fs::write(conf_d_dir.join(format!("{hostname}.conf")), conf)
            .map_err(|e| format!("Failed to write {hostname}.conf: {e}"))?;
    }

    // Remove .conf files for hostnames no longer active
    if let Ok(entries) = std::fs::read_dir(&conf_d_dir) {
        let active: std::collections::HashSet<String> =
            aliases.iter().map(|(h, _)| format!("{h}.conf")).collect();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".conf") && !active.contains(&name) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    // Write docker-compose.yml
    let compose_content = DOCKER_COMPOSE_TEMPLATE
        .replace("{CONF_DIR}", &conf_dir.to_string_lossy())
        .replace("{CERT_DIR}", &cert_dir.to_string_lossy());
    let compose_path = proxy.join("docker-compose.yml");
    std::fs::write(&compose_path, compose_content)
        .map_err(|e| format!("Failed to write docker-compose.yml: {e}"))?;

    // Remove existing container to pick up config changes
    let _ = std::process::Command::new("docker")
        .args(["compose", "-f", &compose_path.to_string_lossy(), "down"])
        .output();

    // Start container
    let output = std::process::Command::new("docker")
        .args(["compose", "-f", &compose_path.to_string_lossy(), "up", "-d"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("docker compose up failed: {stderr}"))
    }
}

/// Stop the nginx container.
pub fn stop_proxy(app: &AppHandle) -> Result<(), String> {
    let compose_path = proxy_dir(app)?.join("docker-compose.yml");
    if !compose_path.exists() {
        return Ok(()); // Nothing running
    }
    let output = std::process::Command::new("docker")
        .args(["compose", "-f", &compose_path.to_string_lossy(), "down"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("docker compose down failed: {stderr}"))
    }
}

/// Returns "running", "stopped", or "unknown".
pub fn get_proxy_status(app: &AppHandle) -> String {
    let compose_path = match proxy_dir(app) {
        Ok(p) => p.join("docker-compose.yml"),
        Err(_) => return "unknown".into(),
    };
    if !compose_path.exists() {
        return "stopped".into();
    }
    let output = std::process::Command::new("docker")
        .args([
            "compose",
            "-f",
            &compose_path.to_string_lossy(),
            "ps",
            "--format",
            "json",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.contains("tarasque-nginx") && text.contains("running") {
                "running".into()
            } else {
                "stopped".into()
            }
        }
        _ => "unknown".into(),
    }
}
```

**Step 2: Confirm it compiles (errors only)**

```bash
rtk cargo check 2>&1 | head -40
```
Expected: errors about `mod proxy` not declared in `main.rs` — that's fine, fixed in Task 3.

---

### Task 3: Wire proxy module into commands and main.rs

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/commands.rs`

**Step 1: Add `mod proxy;` to `main.rs` and register new commands, remove pf commands**

In `src-tauri/src/main.rs`, change:
```rust
mod commands;
mod models;
mod server;
mod storage;
```
to:
```rust
mod commands;
mod models;
mod proxy;
mod server;
mod storage;
```

And in the `invoke_handler`, remove `commands::setup_port_forward` and `commands::teardown_port_forward`, add 4 new commands:
```rust
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
    commands::add_hosts_entry,
    commands::write_cert_files,
    commands::start_proxy,
    commands::stop_proxy,
    commands::get_proxy_status,
])
```

**Step 2: Add 4 new commands + remove 2 old commands in `commands.rs`**

Remove these two functions entirely from `commands.rs`:
- `setup_port_forward` (lines 430–466)
- `teardown_port_forward` (lines 469–482)

Modify `generate_alias_cert` to also write cert files to disk (add `app: AppHandle` param and call `proxy::write_cert_files`):

```rust
#[tauri::command]
pub fn generate_alias_cert(app: AppHandle, hostname: String) -> Result<AliasCredentials, String> {
    let san = vec![hostname.clone()];
    let certified = rcgen::generate_simple_self_signed(san)
        .map_err(|e| format!("Certificate generation failed: {e}"))?;

    let cert_pem = certified.cert.pem();
    let key_pem = certified.key_pair.serialize_pem();

    // Write to disk for Docker volume mount
    crate::proxy::write_cert_files(&app, &hostname, &cert_pem, &key_pem)?;

    Ok(AliasCredentials { cert_pem, key_pem })
}
```

Add 4 new commands at the bottom of `commands.rs` (before `fn extract_json_path`):

```rust
#[tauri::command]
pub fn write_cert_files(
    app: AppHandle,
    hostname: String,
    cert_pem: String,
    key_pem: String,
) -> Result<(), String> {
    crate::proxy::write_cert_files(&app, &hostname, &cert_pem, &key_pem)
}

#[derive(serde::Deserialize)]
pub struct AliasEntry {
    pub hostname: String,
    pub port: u16,
}

#[tauri::command]
pub fn start_proxy(app: AppHandle, aliases: Vec<AliasEntry>) -> Result<(), String> {
    let pairs: Vec<(String, u16)> = aliases.into_iter().map(|a| (a.hostname, a.port)).collect();
    crate::proxy::start_proxy(&app, &pairs)
}

#[tauri::command]
pub fn stop_proxy(app: AppHandle) -> Result<(), String> {
    crate::proxy::stop_proxy(&app)
}

#[tauri::command]
pub fn get_proxy_status(app: AppHandle) -> Result<String, String> {
    Ok(crate::proxy::get_proxy_status(&app))
}
```

**Step 3: Compile check**

```bash
rtk cargo check 2>&1 | head -60
```
Expected: clean compile (0 errors).

**Step 4: Full build to confirm no warnings block**

```bash
rtk cargo build 2>&1 | tail -5
```
Expected: `Finished dev profile`.

---

### Task 4: Add `proxy.ts` Svelte store

**Files:**
- Create: `src/lib/stores/proxy.ts`

**Step 1: Write `src/lib/stores/proxy.ts`**

```typescript
import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { collections } from './collections';
import type { Collection } from '$lib/types';

export type ProxyStatus = 'running' | 'stopped' | 'unknown';

export const proxyStatus = writable<ProxyStatus>('unknown');
export const proxyError = writable<string | null>(null);
export const proxyLoading = writable(false);

function enabledAliases(cols: Collection[]) {
  return cols
    .filter((c) => c.alias_enabled && c.alias_hostname && c.alias_cert_pem)
    .map((c) => ({ hostname: c.alias_hostname!, port: c.port }));
}

export async function refreshProxyStatus() {
  try {
    const status = await invoke<string>('get_proxy_status');
    proxyStatus.set(status as ProxyStatus);
  } catch {
    proxyStatus.set('unknown');
  }
}

export async function startProxy() {
  proxyLoading.set(true);
  proxyError.set(null);
  try {
    const aliases = enabledAliases(get(collections));
    await invoke('start_proxy', { aliases });
    proxyStatus.set('running');
  } catch (err) {
    proxyError.set(String(err));
    proxyStatus.set('stopped');
  } finally {
    proxyLoading.set(false);
  }
}

export async function stopProxy() {
  proxyLoading.set(true);
  proxyError.set(null);
  try {
    await invoke('stop_proxy');
    proxyStatus.set('stopped');
  } catch (err) {
    proxyError.set(String(err));
  } finally {
    proxyLoading.set(false);
  }
}
```

**Step 2: Verify TypeScript compiles**

```bash
rtk tsc --noEmit 2>&1 | head -20
```
Expected: 0 errors.

---

### Task 5: Add nginx status bar to sidebar

**Files:**
- Modify: `src/lib/components/sidebar/index.svelte`

**Step 1: Read current sidebar bottom section (already done above)**

**Step 2: Add nginx status bar at the very bottom of the sidebar**

Add imports at top of `<script>`:
```typescript
import { proxyStatus, proxyLoading, startProxy, stopProxy, refreshProxyStatus } from '$lib/stores/proxy';
import { onMount } from 'svelte';
```

Add `onMount` block:
```typescript
onMount(() => {
  refreshProxyStatus();
  const interval = setInterval(refreshProxyStatus, 5000);
  return () => clearInterval(interval);
});
```

Add this block right before the closing `</div>` of the main flex container (after `<ImportExport />`):

```svelte
<!-- Nginx Proxy Status -->
<div class="shrink-0 border-t border-line px-3 py-2.5 flex items-center gap-2">
  <span class="w-1.5 h-1.5 rounded-full shrink-0 {
    $proxyStatus === 'running' ? 'bg-run' :
    $proxyStatus === 'stopped' ? 'bg-ink-dim' : 'bg-warn'
  }"></span>
  <span class="text-[10px] font-krub text-ink-dim flex-1 uppercase tracking-widest">nginx</span>
  {#if $proxyLoading}
    <span class="text-[10px] text-ink-dim">…</span>
  {:else if $proxyStatus === 'running'}
    <button
      onclick={stopProxy}
      class="text-[10px] font-krub text-ink-dim hover:text-err transition-colors"
    >Stop</button>
  {:else}
    <button
      onclick={startProxy}
      class="text-[10px] font-krub text-ink-dim hover:text-run transition-colors"
    >Start</button>
  {/if}
</div>
```

**Step 3: Check browser dev tools for runtime errors**

Run `bun run dev` and open the app. Expected: nginx status bar visible at bottom of sidebar, showing "stopped" with a Start button.

---

### Task 6: Update CollectionSettingsModal — replace pf section with Docker proxy controls

**Files:**
- Modify: `src/lib/components/CollectionSettingsModal.svelte`

**Step 1: Add `certMode` state and remove pf state**

In the `<script>` block:
- Remove: `let forwardingPort = $state(false);`
- Add:
```typescript
let certMode = $state<'auto' | 'manual'>('auto');
```

**Step 2: Remove `setupPortForward` and `teardownPortForward` functions**

Delete both functions (lines 117–139).

**Step 3: Replace the cert generation button block with auto/manual mode selector**

Find the `<!-- Certificate -->` section inside `{#if aliasEnabled}`. Replace the entire block from `<!-- Certificate -->` down to and including `<!-- /etc/hosts + trust instructions -->` with:

```svelte
<!-- Certificate Mode -->
<div>
  <div class="flex items-center gap-4 mb-3">
    <span class="text-[10px] text-ink-muted">Certificate Mode</span>
    <label class="flex items-center gap-1.5 cursor-pointer">
      <input type="radio" bind:group={certMode} value="auto" class="accent-[var(--color-accent)]" />
      <span class="text-[11px] text-ink-muted">Auto</span>
    </label>
    <label class="flex items-center gap-1.5 cursor-pointer">
      <input type="radio" bind:group={certMode} value="manual" class="accent-[var(--color-accent)]" />
      <span class="text-[11px] text-ink-muted">Manual</span>
    </label>
  </div>

  <div class="flex items-center justify-between mb-2">
    <span class="text-[10px] text-ink-muted">TLS Certificate</span>
    {#if hasCert}
      <span class="text-[10px] text-run flex items-center gap-1">
        <ShieldCheck size={10} /> Generated
      </span>
    {:else}
      <span class="text-[10px] text-warn flex items-center gap-1">
        <ShieldAlert size={10} /> Not generated
      </span>
    {/if}
  </div>

  {#if certMode === 'auto'}
    <!-- Auto mode: one button does generate + trust + hosts -->
    <button
      onclick={generateAndSetupAuto}
      disabled={generatingCert || !aliasHostname.trim()}
      class="w-full px-3 py-2 text-xs font-krub bg-app border border-line-strong rounded
             text-ink-muted hover:text-ink hover:border-accent transition-colors
             disabled:opacity-40 disabled:cursor-not-allowed"
    >
      {generatingCert ? 'Setting up…' : hasCert ? 'Regenerate & Re-trust' : 'Generate, Trust & Add to hosts'}
    </button>
    {#if hasCert}
      <p class="text-[10px] text-ink-dim mt-1.5">
        Certificate trusted in macOS Keychain. Restart your browser once.
      </p>
    {/if}
  {:else}
    <!-- Manual mode: generate cert, show commands -->
    <div class="flex gap-2 mb-3">
      <button
        onclick={generateCert}
        disabled={generatingCert || !aliasHostname.trim()}
        class="flex-1 px-3 py-2 text-xs font-krub bg-app border border-line-strong rounded
               text-ink-muted hover:text-ink hover:border-accent transition-colors
               disabled:opacity-40 disabled:cursor-not-allowed"
      >
        {generatingCert ? 'Generating…' : hasCert ? 'Regenerate Certificate' : 'Generate Certificate'}
      </button>
      {#if hasCert}
        <button
          onclick={downloadCert}
          class="px-3 py-2 text-xs font-krub bg-app border border-line-strong rounded
                 text-ink-muted hover:text-ink hover:border-accent transition-colors"
          title="Download certificate (.crt)"
          aria-label="Download certificate"
        >
          <Download size={12} />
        </button>
      {/if}
    </div>

    {#if hasCert && aliasHostname}
      <div class="bg-app rounded-lg border border-line p-3 space-y-3 text-[11px]">
        <p class="text-ink-muted font-krub mb-1">Run these commands in your terminal:</p>

        <!-- Keychain command -->
        <div>
          <p class="text-ink-dim mb-1">① Trust certificate (macOS Keychain)</p>
          <div class="flex items-center gap-2 bg-surface rounded px-2.5 py-2 font-mono text-[10px] text-ink border border-line">
            <span class="flex-1 select-all break-all">{keychainCmd}</span>
            <button
              onclick={() => copyCmd(keychainCmd)}
              class="text-ink-dim hover:text-accent shrink-0"
              title="Copy"
            ><Copy size={11} /></button>
          </div>
        </div>

        <!-- Hosts command -->
        <div>
          <p class="text-ink-dim mb-1">② Add to /etc/hosts</p>
          <div class="flex items-center gap-2 bg-surface rounded px-2.5 py-2 font-mono text-[10px] text-ink border border-line">
            <span class="flex-1 select-all break-all">{hostsCmd}</span>
            <button
              onclick={() => copyCmd(hostsCmd)}
              class="text-ink-dim hover:text-accent shrink-0"
              title="Copy"
            ><Copy size={11} /></button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>
```

**Step 4: Add derived command strings and new functions to `<script>`**

Add these derived values (after `let hasCert = ...`):

```typescript
const certFilePath = $derived(
  `${aliasHostname ? `~/Library/Application Support/com.tarasque.app/proxy/cert/${aliasHostname}/cert.pem` : ''}`
);
const keychainCmd = $derived(
  `security add-trusted-cert -k ~/Library/Keychains/login.keychain-db "${certFilePath}"`
);
const hostsCmd = $derived(
  `sudo sh -c 'echo "127.0.0.1 ${aliasHostname || '<hostname>'}" >> /etc/hosts'`
);
```

Add `copyCmd` helper and `generateAndSetupAuto` function:

```typescript
function copyCmd(cmd: string) {
  navigator.clipboard.writeText(cmd);
  toasts.show('info', 'Copied to clipboard');
}

async function generateAndSetupAuto() {
  if (!aliasHostname.trim()) {
    toasts.show('error', 'Enter a hostname first');
    return;
  }
  generatingCert = true;
  try {
    // 1. Generate cert (also writes files to disk via Rust)
    const result = await invoke<{ cert_pem: string; key_pem: string }>('generate_alias_cert', {
      hostname: aliasHostname.trim(),
    });
    aliasCertPem = result.cert_pem;
    aliasKeyPem  = result.key_pem;

    // 2. Trust in macOS Keychain
    await invoke('trust_cert_macos', { certPem: aliasCertPem });

    // 3. Add to /etc/hosts (osascript admin dialog)
    await invoke('add_hosts_entry', { hostname: aliasHostname.trim() });

    toasts.show('success', `HTTPS alias set up for ${aliasHostname} — restart your browser`);
  } catch (err) {
    const msg = String(err);
    if (msg.includes('cancelled') || msg.includes('User canceled')) {
      toasts.show('info', 'Setup cancelled');
    } else {
      toasts.show('error', `Setup failed: ${msg}`);
    }
  } finally {
    generatingCert = false;
  }
}
```

**Step 5: Check TypeScript**

```bash
rtk tsc --noEmit 2>&1 | head -30
```
Expected: 0 errors.

---

### Task 7: Run dev and smoke test

**Step 1: Start the dev server**

```bash
bun run tauri dev
```

**Step 2: Manual verification checklist**

- [ ] Sidebar bottom shows nginx status bar (stopped / Start button)
- [ ] Open Collection Settings modal → HTTPS Alias section exists
- [ ] Toggle alias on → hostname/port fields appear
- [ ] Auto mode: "Generate, Trust & Add to hosts" button present
- [ ] Manual mode: after generating cert, keychain and hosts commands shown with copy buttons
- [ ] Port forward section is gone
- [ ] Click Start in sidebar → if Docker running, container starts; proxy status turns green
- [ ] Click Stop → status returns to stopped

**Step 3: Verify Docker container**

```bash
docker ps | grep tarasque-nginx
```
Expected: `tarasque-nginx` container listed as `Up`.

**Step 4: Test HTTPS access (if a collection alias is configured)**

```bash
curl -k https://api.local/your-endpoint
```
Expected: response from the mock server.
