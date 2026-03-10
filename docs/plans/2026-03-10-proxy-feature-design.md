# Proxy Feature Design (HTTPS Alias per Collection)
**Date:** 2026-03-10
**Status:** Approved

## Overview

Add a per-collection HTTPS alias feature to Tarasque. Each collection can optionally expose its mock server behind a custom local domain (e.g. `myapi.local:443`) via a single shared nginx Docker container — mirroring the Ophiuchi approach.

**Approach:** `rcgen` (Rust-native) for cert generation + Docker CLI for nginx container management.

---

## Data Model

### TypeScript (`src/lib/types.ts`)
Already present — no changes needed:
```typescript
alias_enabled: boolean        // toggle the HTTPS alias on/off
alias_hostname?: string       // e.g. "myapi.local"
alias_port?: number           // nginx listens on this port (default 443)
alias_cert_pem?: string       // path to cert.pem
alias_key_pem?: string        // path to private.key
```

### Rust (`src-tauri/src/models.rs`)
Mirror the alias fields on `Collection`. Add a global proxy config (not per-collection):
```rust
pub struct ProxyConfig {
    pub nginx_container_name: String,  // "tarasque-nginx"
    pub docker_compose_path: String,   // AppData/proxy/docker-compose.yml
}
```

### File Layout (AppData)
```
AppData/
  proxy/
    docker-compose.yml          ← regenerated each time container starts
    conf/
      nginx.conf                ← main nginx config (from template)
      conf.d/
        {hostname}.conf         ← per-collection server block
    cert/
      {hostname}/
        cert.pem
        private.key
```

---

## Architecture & Flow

### Container Lifecycle (single shared nginx)
```
User enables alias on collection
  → cert generated (auto or manual)
  → write {hostname}.conf to AppData/proxy/conf/conf.d/
  → if container running: restart (picks up new config)
  → if container not running: start it

User disables alias
  → delete {hostname}.conf
  → restart container (or stop if no aliases remain)
```

### Cert Generation

**Auto mode:**
1. `rcgen` generates self-signed cert (10yr validity, SAN: hostname + 127.0.0.1)
2. Write `cert.pem` + `private.key` to `AppData/proxy/cert/{hostname}/`
3. Run: `security add-trusted-cert -k ~/Library/Keychains/login.keychain-db {cert.pem}`
4. Prompt user for sudo password → run: `sudo sh -c 'echo "127.0.0.1 {hostname}" >> /etc/hosts'`

**Manual mode:**
1. `rcgen` generates cert files identically to auto
2. Display two copyable commands in the UI:
   - `security add-trusted-cert -k ~/Library/Keychains/login.keychain-db "{cert.pem path}"`
   - `sudo sh -c 'echo "127.0.0.1 {hostname}" >> /etc/hosts'`
3. User confirms they've run them → alias marked ready

### nginx Templates

**docker-compose.yml** (generated to AppData/proxy/):
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
      - ./conf/nginx.conf:/etc/nginx/nginx.conf
      - ./conf/conf.d:/etc/nginx/conf.d
      - ./cert:/usr/local/ssl/cert
    extra_hosts:
      - "host.docker.internal:host-gateway"
```

**nginx.conf** (static template bundled in app):
- Standard nginx worker config
- Includes `/etc/nginx/conf.d/*.conf`

**{hostname}.conf** (generated per collection):
```nginx
upstream {upstream_suffix} {
    server host.docker.internal:{port};
}
server {
    listen 443 ssl;
    server_name {hostname};
    ssl_certificate /usr/local/ssl/cert/{hostname}/cert.pem;
    ssl_certificate_key /usr/local/ssl/cert/{hostname}/private.key;
    location / {
        proxy_pass http://{upstream_suffix};
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
    }
}
```

### New Tauri Commands (`commands.rs`)
```rust
generate_cert(hostname: String) -> Result<CertPaths, String>
  // rcgen → writes cert.pem + private.key, returns paths

add_to_keychain(cert_path: String, password: String) -> Result<(), String>
  // security add-trusted-cert ...

add_to_hosts(hostname: String, password: String) -> Result<(), String>
  // sudo sh -c 'echo "127.0.0.1 {hostname}" >> /etc/hosts'

start_proxy() -> Result<(), String>
  // regenerates docker-compose.yml + all .conf files → docker compose up -d

stop_proxy() -> Result<(), String>
  // docker compose down

get_proxy_status() -> Result<String, String>
  // docker compose ps → "running" | "stopped" | "unknown"
```

### New Svelte Store (`src/lib/stores/proxy.ts`)
```typescript
export const proxyStatus = writable<'running' | 'stopped' | 'unknown'>('unknown');
export async function startProxy() { ... }
export async function stopProxy() { ... }
export async function refreshProxyStatus() { ... }
```

---

## UI Design

### Collection Settings Modal — new "HTTPS Alias" tab
```
┌─ Collection Settings ──────────────────────┐
│  General  |  HTTPS Alias                   │
├────────────────────────────────────────────┤
│  HTTPS Alias                    [toggle]   │
│                                            │
│  Hostname   [ myapi.local        ]         │
│  Port       [ 443                ]         │
│                                            │
│  Certificate                               │
│  ○ Auto   ● Manual                         │
│                                            │
│  [  Generate Certificate  ]                │
│                                            │
│  — Manual steps ——————————————————————     │
│  Run these commands in your terminal:      │
│                                            │
│  security add-trusted-cert -k ...  [copy]  │
│  sudo sh -c 'echo ...'             [copy]  │
│                                            │
│  [ I've run both commands ]                │
└────────────────────────────────────────────┘
```

### ServerPanel — HTTPS alias status badge
```
[▶ Run]   HTTPS: myapi.local:443 ● active
```
Only shown when `alias_enabled = true` on active collection.

### Sidebar bottom — global nginx status
```
◉ nginx  [Start] [Stop]
```
Persistent at bottom of sidebar. Shows Docker container status. Polls every 5 seconds when app is focused.

---

## Error Handling

| Scenario | Handling |
|---|---|
| Docker not installed | `get_proxy_status` returns error → sidebar shows "Docker required" link |
| Port 443 already in use | `start_proxy` error → toast with message |
| Cert generation fails | Error shown in modal, alias not marked ready |
| sudo password wrong | Error returned → re-prompt |
| Manual mode: user hasn't confirmed | Alias badge shows "pending" state |

---

## Key Decisions

| Decision | Choice | Reason |
|---|---|---|
| Cert generation | `rcgen` Rust crate | No `openssl` binary dependency |
| Container strategy | Single shared nginx | Matches Ophiuchi, efficient |
| Config regeneration | Full regen on each start | Simple, no incremental update bugs |
| Hosts file | sudo via tauri-plugin-shell | Required for /etc/hosts write |
| Cert trust | security CLI (macOS only) | Standard macOS keychain approach |
| Auto/manual | Both supported | Flexibility for users who prefer control |
