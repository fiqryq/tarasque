# Tarasque — Build Progress

A record of every planning decision and feature that was designed, built, and confirmed working.

---

## Overview

**Tarasque** is a desktop mock API server — like Mockoon or Postman, but built from scratch.
It lets you define HTTP endpoints with configurable responses, start a local server per-collection, and watch live request logs in real time.

**Stack:** Tauri v2 · Rust (Actix-web 4) · Svelte 5 · Tailwind CSS · Monaco Editor

---

## Phase 1 — Architecture Design

**Date:** 2026-03-09
**Doc:** `docs/plans/2026-03-09-tarasque-architecture-design.md`

Decisions made before writing a single line of code:

| Decision | Choice | Reason |
|---|---|---|
| Editor | Monaco (full) | Syntax highlighting, JSON validation, familiar feel |
| Path params | Day 1, critical | `/users/:id` matching needed from the start |
| Save strategy | Auto-save, 500ms debounce | No save button friction |
| Architecture | Domain-driven modules | Clear separation between server, storage, and commands |
| Port scope | Per-collection | Multiple collections can have different ports |
| Shutdown | Oneshot channel | Graceful stop without killing the process |

**Approved data model:**

```
AppData
  └── Collection (id, name, port)
        └── Endpoint (id, method, path, active_scenario_id)
              └── Scenario (id, name, status, delay_ms, headers, body)
```

---

## Phase 2 — Implementation Plan

**Date:** 2026-03-09
**Doc:** `docs/plans/2026-03-09-tarasque-implementation.md`
**Tasks:** 18 total, all completed

### Backend (Rust / Tauri)

| Task | Description | Status |
|---|---|---|
| 1 | Project scaffold — Tauri v2 + Svelte 5 + Tailwind + Monaco | ✅ |
| 2 | Rust models — `Scenario`, `Endpoint`, `Collection`, `AppData`, `LogEntry` | ✅ |
| 3 | Storage module — async tokio::fs, `app_local_data_dir()` | ✅ |
| 4 | Server state — `AppState`, `ServerHandle(Mutex<Option<Sender<()>>>)` | ✅ |
| 5 | Path router — `match_path()` + `substitute_params()` with 7 passing tests | ✅ |
| 6 | Actix server start/stop — `.bind()` before spawn, graceful shutdown | ✅ |
| 7 | Tauri commands — `load_data`, `save_data`, `start_server`, `stop_server` | ✅ |

### Frontend (Svelte 5 / TypeScript)

| Task | Description | Status |
|---|---|---|
| 8 | TypeScript types — interfaces + `newScenario()`, `newEndpoint()`, `newCollection()` factories | ✅ |
| 9 | Collections store — writable + derived stores, debounced auto-save | ✅ |
| 10 | Server + logs stores — `startServer()` passes `AppData` snapshot, ring buffer 200 entries | ✅ |
| 11 | Sidebar components — collection list, create/delete | ✅ |
| 12 | Monaco Editor component — lazy-loaded, `$effect` sync, dispose on destroy | ✅ |
| 13 | Scenario editor — status, delay, headers key/value, Monaco body | ✅ |
| 14 | Endpoint list + detail — method select, path input, scenario tabs | ✅ |
| 15 | Editor index — endpoint list (w-56) + detail panel | ✅ |
| 16 | Server panel + log panel — port input, run/stop, live log rows | ✅ |
| 17 | App root — layout shell, `loadCollections()`, `initLogListener()` on mount | ✅ |
| 18 | Smoke test — full end-to-end verified working | ✅ |

---

## Phase 3 — Critical Bug Fixes

Three architectural bugs caught and fixed before the smoke test:

### C1 — Race condition on server start
**Problem:** `start_server` re-read data from disk. With 500ms debounce, the latest edits weren't saved yet.
**Fix:** `start_server` command now accepts `AppData` directly from the frontend (saves to disk then starts server).

### C2 — Tauri events silently dropped
**Problem:** `emit()` targets all windows — on startup, the window may not exist yet.
**Fix:** Changed to `emit_to("main", ...)` to reliably target the main window.

### C3 — Port-in-use errors swallowed
**Problem:** `.bind()` was inside `tauri::async_runtime::spawn`, so bind errors were lost.
**Fix:** Moved `.bind()` outside the spawn so errors propagate back to the frontend.

### I1 — Stale active scenario ID
**Problem:** `activeScenarioId` was `$state(endpoint.active_scenario_id)` — didn't reset when switching endpoints.
**Fix:** Used `localScenarioId = $state(null)` + `$derived` fallback + `$effect` reset on `endpoint.id` change.

---

## Phase 4 — UI Redesign

Two design passes to reach the final aesthetic.

### Pass 1 — Minimalist dark
- Color scheme: `#181818` base, same topbar color as app
- Fonts: Krub (labels/brand) + Inter (UI/data)
- Borderless inputs, flat surfaces, line separators only

### Pass 2 — Insomnia-style polish (final)

Based on reference image of Insomnia REST client.

**Layout changes:**
- Added icon nav strip (40px) on far-left — "T" brand + Database + Settings icons
- Collection sidebar: `#191919`, folder icons, port badge per collection
- Endpoint list: `#191919`, separated from main content

**Method badges** (colored pill, background + text):

| Method | Style |
|---|---|
| GET | `bg-emerald-500/10 text-emerald-400` |
| POST | `bg-violet-500/10 text-violet-400` |
| PUT | `bg-sky-500/10 text-sky-400` |
| DELETE | `bg-rose-500/10 text-rose-400` |
| PATCH | `bg-amber-500/10 text-amber-400` |

**Run/Stop button:** colored bordered badge — green with `bg-run/10 text-run border-run/20`, flips to red when server is running.

**Status codes:** colored badges in scenario tabs and log rows (green 2xx, amber 3xx, orange 4xx, rose 5xx).

**Log panel:** method colors, status badge pills, tabular layout.

---

## Final State

- **All 18 implementation tasks complete**
- **All 3 critical bugs fixed**
- **UI redesign confirmed working by user**
- App saves data to `app_local_data_dir()` automatically
- Server starts on configurable port per collection
- Path params (`/users/:id`) matched and substituted on day 1
- Live request logs streamed via Tauri events with ring buffer (200 entries)
