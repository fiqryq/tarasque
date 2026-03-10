# Tarasque v2 — Iteration Plan

**Version:** 2.0  
**Date:** 2026-03-09  
**Status:** Planning Complete — Ready for Implementation

---

## Overview

This iteration adds 4 core features and 6 UI enhancements to Tarasque, transforming it from a simple mock server into a comprehensive API development tool.

---

## Features

### F1 — Response Templating

**Description:** Use dynamic values in response body/headers for realistic mocks.

**Template Syntax:**
```json
{
  "id": "{{uuid}}",
  "timestamp": "{{timestamp}}",
  "user": "{{request.params.id}}",
  "message": "Hello {{request.query.name}}"
}
```

**Available Variables:**

| Variable | Description | Example |
|----------|-------------|---------|
| `{{uuid}}` | Random UUID v4 | `"550e8400-e29b-41d4-a716-446655440000"` |
| `{{timestamp}}` | Unix timestamp | `1709999999` |
| `{{iso_timestamp}}` | ISO 8601 | `"2026-03-09T12:00:00Z"` |
| `{{random.int}}` | Random integer 0-1000 | `42` |
| `{{random.string}}` | Random alphanumeric | `"a7x9k2m"` |
| `{{random.bool}}` | Random boolean | `true` |
| `{{request.params.X}}` | Path param | `{{request.params.id}}` |
| `{{request.query.X}}` | Query param | `{{request.query.page}}` |
| `{{request.body.X}}` | JSON body field | `{{request.body.name}}` |
| `{{request.headers.X}}` | Request header | `{{request.headers.Authorization}}` |

**Implementation:**
- Add `TemplateEngine` module in `router.rs`
- Parse template syntax, replace with values
- Add help tooltip in ScenarioEditor

**Confidence:** 9/10

---

### F2 — Import/Export

**Description:** Import collections from OpenAPI/Swagger or export to JSON/Postman.

**Supported Formats:**

| Format | Import | Export |
|--------|--------|--------|
| OpenAPI 3.0/3.1 (JSON) | ✅ | ❌ |
| OpenAPI 3.0/3.1 (YAML) | ✅ | ❌ |
| Postman Collection v2.1 | ✅ | ✅ |
| Tarasque JSON | ✅ | ✅ |

**Import Flow:**
1. Click Import button
2. File picker → select JSON/YAML
3. Parse and show preview
4. Confirm → merge into data

**Export Flow:**
1. Click Export on collection
2. Choose format
3. Download file

**OpenAPI Mapping:**

| OpenAPI | Taresque |
|---------|----------|
| `paths[].{method}` | Endpoint.method |
| `paths[].{method}.operationId` | Scenario name |
| `paths[].{method}.responses[].value` | Scenario.status, Scenario.body |
| `paths[].{method}.responses[].headers` | Scenario.headers |

**Implementation:**
- Add `openapiv3` crate for parsing
- Backend: `import_collection`, `export_collection` commands
- Frontend: Import button + preview modal, Export dropdown

**Confidence:** 7/10 (OpenAPI specs vary widely)

---

### F3 — Smoke Testing

**Description:** Define automated test cases that hit mock endpoints and assert expected responses.

**Data Model:**

```typescript
interface TestAssertion {
  id: string;
  type: 'status' | 'body';
  path?: string;        // JSON path for body, e.g., "data.user.name"
  operator: 'eq' | 'contains' | 'exists' | 'not_exists';
  expected: string | number | boolean;
}

interface TestCase {
  id: string;
  name: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  headers?: Record<string, string>;
  body?: string;
  assertions: TestAssertion[];
}

interface TestSuite {
  id: string;
  name: string;
  collection_id: string;
  test_cases: TestCase[];
}

interface TestResult {
  test_case_id: string;
  passed: boolean;
  duration: number;
  actual_status?: number;
  actual_body?: string;
  error?: string;
  assertion_results: {
    assertion: TestAssertion;
    passed: boolean;
    actual?: string;
  }[];
}

interface TestRunResult {
  suite_id: string;
  timestamp: string;
  total: number;
  passed: number;
  failed: number;
  results: TestResult[];
}
```

**UI Layout:**

```
┌─────────────────────────────────────────────────────────────┐
│ Test Suites                              [▶ Run All]        │
├─────────────────────────────────────────────────────────────┤
│ ○ Get User                    ✓ PASSED (23ms)             │
│ ○ Create User                 ✗ FAILED (45ms)              │
│   → Expected: 201, Got: 200                                  │
│ ○ Delete User                 ✓ PASSED (12ms)              │
└─────────────────────────────────────────────────────────────┘
```

**Test Runner Logic (Rust):**
- Use `reqwest` to send HTTP requests
- Assert status code
- Assert body JSON paths
- Return detailed results

**Implementation:**
- T1: Add TypeScript types
- T2: Add Rust models + storage
- T3: Frontend: Test Suites sidebar section
- T4: Frontend: Test Case editor modal
- T5: Backend: `run_test_suite` command
- T6: Frontend: Results display

**Confidence:** 7/10

---

### F4 — Proxy Forwarding

**Description:** Forward requests to external API when no local mock exists. Per-collection configuration.

**Strategy:** Mock-first
```
Request → Check local endpoints →
  ✓ Match found → Return mock response
  ✗ No match → Forward to proxy_url → Return real response
```

**Data Model:**

```typescript
interface Collection {
  id: string;
  name: string;
  port: number;
  endpoints: Endpoint[];
  // New
  proxy_url?: string;       // e.g., "https://api.host.com"
  proxy_enabled: boolean;
}
```

**Examples:**

```
Collection "Dev API" (port 3000)
├── GET /users/:id     → Local mock (200)
├── POST /users        → Local mock (201)
└── (no /products)    → Proxy to https://api.host.com/products

Collection "Staging" (port 3001)
├── proxy_url: https://staging.api.com
└── (all requests)    → Forward to staging
```

**Implementation:**
- T1: Add `proxy_url`, `proxy_enabled` to Collection model
- T2: Backend: Check endpoint match first, then proxy
- T3: Backend: Forward unmatched requests to proxy_url using `reqwest`
- T4: Backend: Preserve method, headers, body when forwarding
- T5: Frontend: Add proxy toggle + URL input in Collection settings
- T6: Frontend: Show "Proxied" indicator in log panel

**Confidence:** 8/10

---

## UI Enhancements

### UI1 — Log Panel (Collapsible + Resizable)

**Features:**
- Toggle show/hide via button
- Resizable via drag handle
- State persisted in localStorage

**Behavior:**
- Click toggle → slide up/down
- Drag handle → resize height
- Minimum height when collapsed: just header with toggle button
- Default: expanded

**Keyboard Shortcut:** `Cmd+L` to toggle

---

### UI2 — Settings Modal

**Trigger:** Gear icon in sidebar nav strip

**Structure:**

```
┌─────────────────────────────────────┐
│  Settings                    [×]   │
├─────────────────────────────────────┤
│  [General] [Appearance] [About]    │
├─────────────────────────────────────┤
│                                     │
│  Tab Content...                     │
│                                     │
└─────────────────────────────────────┘
```

**Tabs:**

| Tab | Settings |
|-----|----------|
| **General** | Default port, auto-save debounce (ms), log buffer size |
| **Appearance** | Theme (dark/light/system), Code font size (12-18px), UI font size (12-16px) |
| **About** | Version, credits |

**Implementation:**
- CSS variables for theme switching
- Settings stored in localStorage
- Apply on load

---

### UI3 — Appearance Settings

**Theme Options:**
- Dark (current default)
- Light
- System (follow OS)

**Font Size:**
- Code: 12px, 13px, 14px, 15px, 16px, 17px, 18px
- UI: 12px, 13px, 14px, 15px, 16px

**Implementation:**
- CSS custom properties for colors and fonts
- Apply via `data-theme` attribute on `<html>`
- Persist in localStorage

---

### UI4 — Search/Filter

**Features:**
- Search endpoints by path
- Filter by HTTP method (GET, POST, etc.)
- Real-time filtering as you type

**UI:**
- Search input in endpoint list header
- Method filter dropdown/chips

---

### UI5 — Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+N` | New endpoint |
| `Cmd+S` | Force save |
| `Cmd+L` | Toggle log panel |
| `Cmd+,` | Open settings |
| `Cmd+Enter` | Run server (if stopped) / Stop server (if running) |

**Implementation:**
- Global keyboard listener
- Check focus state (don't trigger in inputs)

---

### UI6 — Toast Notifications

**Types:**
- Success: "Saved", "Test passed"
- Error: "Save failed", "Server error"
- Info: "Server started"

**Behavior:**
- Show bottom-right
- Auto-dismiss after 3 seconds
- Click to dismiss

---

## Implementation Order

| Phase | Work | Difficulty |
|-------|------|------------|
| **1** | F1: Response Templating | Easy |
| **2** | F4: Proxy + UI2: Settings Modal + UI3: Appearance | Medium |
| **3** | UI1: Log Panel + UI4: Search/Filter | Easy |
| **4** | F2: Import/Export | Medium |
| **5** | F3: Smoke Testing + UI5: Shortcuts + UI6: Toasts | Hard |

---

## Storage Changes

All new data stored in existing `app_data.json`:

```json
{
  "collections": [...],
  "active_collection_id": "...",
  "test_suites": [...],
  "settings": {
    "theme": "dark",
    "codeFontSize": 14,
    "uiFontSize": 14,
    "defaultPort": 3000,
    "autoSaveDebounce": 500,
    "logBufferSize": 200
  }
}
```

---

## Questions Resolved

| Question | Decision |
|----------|----------|
| Template escape | `{{ "{{" }}` for literal |
| OpenAPI version | 3.0/3.1 only, skip 2.0 |
| Smoke assertions | Start with status + body path |
| Proxy SSL | Skip verification (dev mode) |
| Log panel default | Expanded |
| Settings trigger | Gear icon in sidebar |

---

## Confidence Score

**Overall: 8/10**

| Item | Confidence |
|------|------------|
| F1 Templating | 9/10 |
| F2 Import/Export | 7/10 |
| F3 Smoke Testing | 7/10 |
| F4 Proxy | 8/10 |
| UI Enhancements | 9/10 |

---

## Notes

- Environment variables deferred to v3
- Request chaining (response → input) deferred to v3
- All UI changes use existing component patterns
- Backend changes maintain backward compatibility
