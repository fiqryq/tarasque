# Tarasque v2 Roadmap Design
**Date:** 2026-03-10
**Status:** Approved

## Overview

Full feature roadmap for Tarasque, prioritized by impact to frontend development workflows. Covers all missing features across 5 phases, ordered from highest daily friction to advanced power features.

**Primary use case:** Frontend development — mocking backend APIs so frontend teams can work independently.

**Prioritization principle:** Impact first. Each phase delivers immediate value before moving to the next.

---

## Phase 1: Daily Friction

Small changes that remove daily annoyances. Low-risk, mostly frontend changes.

### 1.1 Endpoint Search/Filter
Real-time search input at the top of the endpoint list. Filters by HTTP method + path as you type. Pure frontend store filter — no backend changes.

### 1.2 Wildcard Path Matching
Support two wildcard patterns in `router.rs`:
- `/api/*` — matches exactly one additional segment
- `/api/**` — matches any number of additional segments

Handled alongside existing `:param` logic. Useful for catch-all mock routes.

### 1.3 Endpoint Duplication
"Duplicate" button per endpoint. Copies the endpoint with all its scenarios intact. New endpoint gets a `(copy)` suffix on the path. Saves time for similar endpoints (GET /users, GET /users/:id, etc.).

### 1.4 HEAD + OPTIONS Methods
Add HEAD and OPTIONS to the method dropdown.
- HEAD: returns same headers as GET, empty body, status 200
- OPTIONS: returns configurable `Allow` header with status 204

Required for frontend frameworks that preflight OPTIONS before POST/PUT/PATCH.

### 1.5 Endpoint Descriptions
Optional `description?: string` field on `Endpoint`. Shown as a subtitle in the endpoint list and as a tooltip. Helps navigation when collections have 30+ endpoints.

**Data model change:** `Endpoint` gains `description?: string` (optional, backward compatible).

---

## Phase 2: Smarter Mocking

Makes mocks behave like a real API without manual scenario switching.

### 2.1 Conditional Scenario Activation
Each scenario gets optional match rules that auto-select it when a request arrives.

**Rule structure:**
```typescript
interface MatchRule {
  source: 'header' | 'query' | 'body' | 'path_param';
  key: string;           // e.g. "Authorization", "status", "role"
  operator: 'eq' | 'contains' | 'exists' | 'not_exists';
  value?: string;
}
```

**Behavior:**
- Rules within a scenario are AND'd together
- Scenarios evaluated in order — first matching scenario wins
- If no rules match, falls back to the manually active scenario
- Scenarios with no rules are always eligible (existing behavior)

**UI:** A "Rules" tab in ScenarioEditor — simple key/operator/value row form, no code.

**Rust:** Rule evaluation happens in `server/mod.rs` request handler before scenario selection.

### 2.2 Stateful In-Memory Store
A per-collection key-value store that persists while the server is running.

**Template access:**
- Read: `{{store.users}}` → returns stored value
- Write (in response body): `{{store.set.lastId = request.body.id}}`
- Delete: `{{store.delete.lastId}}`

**Rust:** `AppState` gains `store: Arc<RwLock<HashMap<String, serde_json::Value>>>`. Reset button in ServerPanel clears it. Wiped on server stop.

**Use case:** POST /login stores token → GET /profile reads `{{store.token}}`.

### 2.3 Form Data Parsing
Parse `application/x-www-form-urlencoded` request bodies on the Rust side. Values accessible via `{{request.body.field}}` — same interface as JSON. No UI changes needed.

**Data model change:** `Scenario` gains `match_rules: Vec<MatchRule>` (defaults to empty `[]`).

---

## Phase 3: Speed Up Setup

Reduces time to get a full mock API running from scratch.

### 3.1 Enhanced OpenAPI / Swagger Import
Improve the existing OpenAPI importer:
- Create one scenario per status code (200, 400, 404, etc.) from `responses` examples
- Pre-fill response body from `content.application/json.example` or `schema` shape
- Preserve path parameters, query parameters as hints in scenario
- Support OpenAPI 3.0 and Swagger 2.0

### 3.2 cURL Import
Paste a cURL command → auto-creates an endpoint.
- Parses: method (`-X`), headers (`-H`), body (`-d`/`--data`), URL
- Pre-fills method, path, and headers on the new endpoint
- UI: "Import cURL" button in endpoint list header → modal with textarea

### 3.3 Collection Export / Import
- **Export:** Save collection as `.tarasque.json` via native save dialog
- **Import:** Load `.tarasque.json` via file picker, merges or replaces collection
- Format is the existing `Collection` JSON shape — no transformation needed

### 3.4 Postman Collection Import
Parse Postman v2.1 collection JSON:
- Maps `request.method`, `request.url.path`, `request.header`, `request.body.raw`
- Creates one endpoint per request item with a single default scenario
- Less complete than OpenAPI (no response examples) but covers the common case

**No data model changes.** All import/export works with the existing `Collection` shape.

---

## Phase 4: Verification

Confirm your frontend sends correct requests and mocks behave as expected.

### 4.1 Testing Framework (Complete Existing Models)
`TestSuite`, `TestCase`, `TestAssertion` models already exist. This phase completes the UI:

**TestCase:** method + path + optional headers + optional body to send
**Assertions:** status equals, body field equals/contains/exists/not_exists
**Run flow:** fires real HTTP requests against the running mock server → collects results
**Results panel:** pass/fail per assertion, actual vs expected values, total pass rate

UI: a "Test Suites" section in the sidebar (component already exists as stub).

### 4.2 Request Analytics Panel
New "Stats" tab in the bottom panel alongside the log panel. In-memory counters updated on each request:
- Request count per endpoint
- Error rate (4xx/5xx)
- Average + p95 response time
- Last hit timestamp

Reset on server restart. Answers: "is my frontend actually hitting this endpoint?"

**Rust:** `AppState` gains `stats: Arc<RwLock<HashMap<String, EndpointStats>>>`. Updated in request handler alongside log emit.

### 4.3 Request Schema Validation Hints
When a request body arrives, loosely compare it against the active scenario's body shape. If fields are missing or types differ — add a `schema_warning: bool` flag to the `LogEntry`. Shown as a yellow dot in the log panel. Non-blocking — mock still responds normally.

**Data model change:** `LogEntry` gains `schema_warning: bool`.

---

## Phase 5: Advanced

Power features for complex frontend scenarios.

### 5.1 Environment Variables
Per-collection key-value environments (dev/staging/prod).

**Structure:**
```typescript
interface Environment {
  id: string;
  name: string;           // "development", "staging"
  variables: Record<string, string>;
}
```

**Template access:** `{{env.userId}}`, `{{env.baseUrl}}`
**UI:** "Environments" dropdown in collection header + modal editor
**Active environment** stored on `Collection` as `active_env_id?: string`

**Data model change:** `Collection` gains `environments: Environment[]` and `active_env_id?: string`.

### 5.2 Advanced Templating
Extend the existing Rust template engine:
- **Conditionals:** `{{#if request.headers.Authorization}}...{{else}}...{{/if}}`
- **Repeat:** `{{#repeat 3}}{"id": {{index}}, "name": "item-{{index}}"}{{/repeat}}`
- **Math:** `{{request.params.id | add 1}}`
- **String filters:** `{{request.body.name | uppercase}}`, `{{request.body.name | lowercase}}`

Custom recursive parser in `server/template.rs` — no external dependencies.

### 5.3 WebSocket Support
Per-endpoint WS mode toggle (`ws_enabled: bool` on `Endpoint`).

When enabled:
- Endpoint accepts WebSocket upgrade requests
- Sends a configurable sequence of JSON messages on a timer
- Timer interval configurable per scenario (ms)
- Message body uses the same template system

Powered by `actix-web`'s built-in WebSocket support (`actix-ws` crate).

### 5.4 Response Sequence Mode
Per-scenario option: `sequence_mode: bool`. When enabled, the collection of scenarios cycles automatically:
- 1st request → scenario 0
- 2nd request → scenario 1
- nth request → wraps back to scenario 0

Counter stored in server state per endpoint. Reset on server restart.
Perfect for: paginated endpoints, flaky API simulation, multi-step flows.

**Data model change:** `Endpoint` gains `sequence_mode: bool`.

### 5.5 Request History Export (HAR)
Export the current log panel as a `.har` file (HTTP Archive format).
- Standard format — importable into Postman, Insomnia, browser DevTools
- One "Export HAR" button in the log panel header
- Frontend-only: transforms `LogEntry[]` into HAR JSON, triggers download

**No backend changes.**

---

## Data Model Changes Summary

| Phase | Model | Change |
|---|---|---|
| 1 | `Endpoint` | `+ description?: string` |
| 2 | `Scenario` | `+ match_rules: MatchRule[]` |
| 2 | `AppState` (runtime) | `+ store: RwLock<HashMap>` |
| 4 | `LogEntry` | `+ schema_warning: bool` |
| 4 | `AppState` (runtime) | `+ stats: RwLock<HashMap>` |
| 5 | `Collection` | `+ environments: Environment[]`, `+ active_env_id?: string` |
| 5 | `Endpoint` | `+ ws_enabled: bool`, `+ sequence_mode: bool` |

All changes are additive and backward compatible with existing saved data.

---

## Implementation Order

```
Phase 1 (Daily Friction)
  └─ 1.1 Endpoint search
  └─ 1.2 Wildcard path matching
  └─ 1.3 Endpoint duplication
  └─ 1.4 HEAD + OPTIONS methods
  └─ 1.5 Endpoint descriptions

Phase 2 (Smarter Mocking)
  └─ 2.3 Form data parsing        ← easiest, no data model change
  └─ 2.1 Conditional scenarios    ← biggest impact
  └─ 2.2 Stateful store           ← enables full API simulation

Phase 3 (Speed Up Setup)
  └─ 3.3 Collection export/import ← simplest, no new format
  └─ 3.2 cURL import              ← quick, high value
  └─ 3.1 Enhanced OpenAPI import  ← builds on existing importer
  └─ 3.4 Postman import           ← similar to OpenAPI

Phase 4 (Verification)
  └─ 4.1 Testing framework        ← models already exist
  └─ 4.2 Analytics panel          ← in-memory, no persistence
  └─ 4.3 Schema validation hints  ← non-blocking, low risk

Phase 5 (Advanced)
  └─ 5.1 Environment variables
  └─ 5.2 Advanced templating
  └─ 5.4 Response sequence mode   ← simple counter
  └─ 5.5 HAR export               ← frontend only
  └─ 5.3 WebSocket support        ← most complex, last
```
