# StubHouse — Product Specification

> **"Stub it. Ship it."**
> Version: 1.0.0-draft | Status: Implementation-Ready

---

## Table of Contents

1. [Vision & Philosophy](#1-vision--philosophy)
2. [Name & Positioning](#2-name--positioning)
3. [Technology Stack](#3-technology-stack)
4. [Core Architecture](#4-core-architecture)
5. [Feature Set](#5-feature-set)
   - 5.1 [Request Client](#51-request-client)
   - 5.2 [Mock Server Runtime](#52-mock-server-runtime)
   - 5.3 [Collection & Workspace Management](#53-collection--workspace-management)
   - 5.4 [Environment & Variable System](#54-environment--variable-system)
   - 5.5 [Scripting Engine](#55-scripting-engine)
   - 5.6 [Import / Export](#56-import--export)
   - 5.7 [GUI — Tauri + Svelte](#57-gui--tauri--svelte)
6. [Data Model](#6-data-model)
7. [Mock Server Deep Dive](#7-mock-server-deep-dive)
8. [Configuration Format (`.stubhouse/`)](#8-configuration-format-mirage)
9. [CLI Interface](#9-cli-interface)
10. [Plugin System](#10-plugin-system)
11. [Roadmap & Phases](#11-roadmap--phases)
12. [Competitive Differentiation](#12-competitive-differentiation)
13. [Non-Goals](#13-non-goals)
14. [Open Questions](#14-open-questions)

---

## 1. Vision & Philosophy

Most API clients are built around one workflow: **send a request, inspect the response.** Postman, Insomnia, Bruno, and Yaak are all refinements of this loop. They've bolted on mock servers as an afterthought — a secondary tab, a paid feature, or an external companion process.

**StubHouse flips this.** The mock server is a first-class citizen, co-equal with the request client. StubHouse is built for the developer who has to work *before* the API exists, *around* an API that's unreliable, or *offline* when everything else is broken.

The product philosophy in three sentences:

- **Local-first, always.** No cloud account required. No syncing to a remote server unless you want it. Your API definitions live in files you own.
- **Files as the source of truth.** The entire state of a workspace is a `.stubhouse/` directory of YAML/JSON files that are readable, diffable, and committable to git.
- **Dual-mode identity.** StubHouse is simultaneously a request sender and a mock runtime. Both sides are equally powerful. Neither is a plugin of the other.

---

## 2. Name & Positioning

**Product name:** StubHouse  
**Tagline:** *Stub it. Ship it.*  
**Domain target:** `stubhouse.dev`  
**GitHub org:** `github.com/stubhouse` or under CacheVector

### Rationale

"StubHouse" is direct and technical — a stub is exactly what this tool produces. It communicates the core use case immediately, carries no negative connotation in the developer tooling context, and is easy to spell and remember internationally.

### Alternate Names Considered

| Name | Verdict |
|---|---|
| Mirage | Evocative but domain wasn't owned |
| Hollowpoint | Too aggressive for an API tool |
| Decoy | Good but feels deceptive |
| Fauxen | Portmanteau — rejected per preference |
| Sidecar | Already taken by multiple tools |
| **StubHouse** | ✅ Chosen — domain owned |

---

## 3. Technology Stack

### Decision: Tauri 2 + Svelte (not GPUI)

GPUI is compelling in theory — it's GPU-accelerated, pixel-perfect, and Rust-native throughout. In practice it is:

- Actively unstable with breaking API changes between versions
- Essentially undocumented outside of Zed's own source code
- Missing critical primitives (text editing, form inputs, complex layouts) that would require re-implementation from scratch
- A full-time research project before the first feature ships

**Recommendation: Tauri 2 + Svelte.** The Rust core — HTTP parser, mock runtime, request engine, YAML/JSON config engine — is 100% Rust regardless of which UI framework is chosen. Tauri's IPC boundary cleanly separates the UI layer from the runtime, so GPUI remains a viable migration target in 2–3 years once it matures. Nothing is lost except the pure-Rust UI claim.

### Stack

| Layer | Technology | Rationale |
|---|---|---|
| App shell | Tauri 2 | Native binary, system tray, OS integration, IPC |
| Frontend UI | Svelte 5 + TypeScript | Fast, minimal bundle, reactive, no VDOM overhead |
| Styling | Tailwind CSS + custom design tokens | Utility-first, consistent design system |
| HTTP runtime | `hyper` + `tokio` | Industry-standard async HTTP, virtual-thread-equivalent via tokio tasks |
| Mock server | Custom Rust + `hyper` as transport | Full control over matching logic |
| Route matching | Custom trie + regex engine in Rust | Pattern matching, wildcards, path params |
| Config format | YAML (primary) + JSON (secondary) | Human-readable, git-diffable, schema-validated |
| Scripting | `rhai` (embedded Rust scripting lang) | Sandboxed, fast, no Node.js dependency |
| TLS | `rustls` | Memory-safe, no OpenSSL dependency |
| Storage | SQLite via `sqlx` | Request history, logs, analytics — not config |
| Testing | `cargo test` + `vitest` | Rust unit/integration + Svelte component tests |

### Why Rhai over Lua/WASM/JS?

Rhai is a scripting language designed specifically for embedding in Rust applications. It has a Rust-like syntax, compiles to bytecode, is sandboxed by default, and has zero external runtime dependencies. Postman uses a sandboxed JS engine (V8) which adds ~50MB to binary size and carries significant attack surface. Rhai gives StubHouse a scripting story without the weight.

---

## 4. Core Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     StubHouse Desktop App                   │
│                                                          │
│  ┌─────────────────────┐   ┌────────────────────────┐   │
│  │    Svelte 5 UI       │   │   Tauri IPC Layer       │   │
│  │  - Request editor    │◄──►  - Command dispatch    │   │
│  │  - Mock rule editor  │   │  - Event streams        │   │
│  │  - Response viewer   │   │  - File system access   │   │
│  └─────────────────────┘   └──────────┬─────────────┘   │
│                                        │                  │
│  ┌─────────────────────────────────────▼─────────────┐   │
│  │                  Rust Core                          │   │
│  │                                                     │   │
│  │  ┌──────────────┐   ┌───────────────────────────┐  │   │
│  │  │ Request       │   │ Mock Server Runtime        │  │   │
│  │  │ Engine        │   │                            │  │   │
│  │  │ - HTTP/1.1   │   │ - Embedded hyper server    │  │   │
│  │  │ - HTTP/2     │   │ - Route trie matcher        │  │   │
│  │  │ - WebSocket  │   │ - Rule evaluation engine    │  │   │
│  │  │ - TLS/mTLS   │   │ - Response renderer         │  │   │
│  │  └──────────────┘   └───────────────────────────┘  │   │
│  │                                                     │   │
│  │  ┌──────────────┐   ┌───────────────────────────┐  │   │
│  │  │ Config        │   │ Script Engine (Rhai)       │  │   │
│  │  │ Engine        │   │ - Pre-request hooks        │  │   │
│  │  │ - YAML/JSON  │   │ - Post-response hooks       │  │   │
│  │  │ - Validation │   │ - Mock rule conditions      │  │   │
│  │  │ - Migration  │   │ - Variable mutation         │  │   │
│  │  └──────────────┘   └───────────────────────────┘  │   │
│  │                                                     │   │
│  │  ┌──────────────┐   ┌───────────────────────────┐  │   │
│  │  │ History &     │   │ Environment & Variable    │  │   │
│  │  │ Log Store     │   │ System                    │  │   │
│  │  │ (SQLite)      │   │ - Scoped resolution        │  │   │
│  │  └──────────────┘   └───────────────────────────┘  │   │
│  └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
         │                           │
   Real network                Local mock server
   (HTTP/2, TLS)               (127.0.0.1:PORT)
```

**Key principle:** The mock server runs as an embedded Tokio task within the Tauri app process. It binds to a configurable local port and is started/stopped via Tauri commands. It never requires a separate process or CLI invocation — starting StubHouse starts the mock server automatically for any workspace that has mocks defined.

---

## 5. Feature Set

### 5.1 Request Client

#### 5.1.1 Request Composition

- Full HTTP/1.1 and HTTP/2 support
- Methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS, and custom methods
- URL bar with:
  - Inline variable interpolation: `{{base_url}}/users/{{user_id}}`
  - Path parameter extraction and display
  - URL parsing into structured query params table
  - History autocomplete (last 50 unique URLs)
- Headers editor:
  - Key/value table with autocomplete for common headers
  - Bulk edit mode (raw text)
  - Preset header groups (e.g., "JSON API", "Bearer Auth") that apply multiple headers at once
- Request body editor:
  - JSON (syntax-highlighted, prettified, minified toggle)
  - Form-data (multipart with file upload support)
  - URL-encoded form
  - XML
  - Plain text
  - Binary/file upload
  - Raw (pass-through, no processing)
- Auth configuration:
  - None
  - Bearer token
  - Basic auth
  - API Key (header or query param)
  - OAuth 2.0 (client credentials, authorization code via browser redirect)
  - Digest auth
  - AWS Signature v4
  - Custom (Rhai script)

#### 5.1.2 Response Viewer

- Status code + timing + size displayed prominently
- Response body viewer:
  - JSON (collapsible tree + raw toggle)
  - XML (tree view)
  - HTML (rendered preview + source toggle)
  - Image (inline render)
  - Binary (hex view)
  - Plain text
- Response headers table (searchable, copyable)
- Cookie viewer (parsed, tabular)
- Timeline view: DNS → TCP → TLS → Request → Response breakdown
- Save response to file
- Diff view: compare current response to any historical response for the same request

#### 5.1.3 Request History

- Persistent history stored in SQLite (last 10,000 requests)
- Each history entry captures: method, URL, status, duration, timestamp, request + response snapshots
- Filterable by method, status code, URL pattern, date range, collection
- One-click replay of any historical request
- Pin important entries to prevent eviction
- Export history to HAR format

#### 5.1.4 SSL / TLS

- Client certificate support (PEM, PKCS#12)
- Custom CA bundle
- Per-request SSL verification toggle (with warning)
- Certificate inspection UI (view chain, expiry, SAN)

#### 5.1.5 Proxying

- HTTP proxy support (SOCKS5, HTTP CONNECT)
- System proxy auto-detection
- Per-request proxy override
- Proxy authentication

---

### 5.2 Mock Server Runtime

This is StubHouse's crown jewel. The mock server is not an add-on — it is an embedded HTTP server with a full rule evaluation engine.

#### 5.2.1 Mock Rule Definition

A mock rule is a **matcher + response pair**. When the embedded server receives a request, it evaluates all rules for the active workspace in priority order and returns the first match.

**Matcher components:**

| Dimension | Matching Options |
|---|---|
| Method | Exact, wildcard (`*`) |
| Path | Exact, glob (`/users/*`), path param (`:id`), regex |
| Query params | Key presence, key=value, key matches regex |
| Headers | Key presence, key=value, key matches regex |
| Body | JSON path assertion, body contains string, body matches regex, Rhai condition |
| Combined | All of the above, ANDed |

**Response components:**

| Property | Options |
|---|---|
| Status code | Any valid HTTP status |
| Headers | Static key/value pairs, variable interpolation |
| Body | Static JSON/text/HTML/binary, template with variables, Rhai-generated |
| Delay | Fixed ms, random range (min–max ms), per-response Rhai |
| Streaming | Server-sent events (SSE), chunked transfer |
| Trailing headers | HTTP/2 trailers |

#### 5.2.2 Rule Priority & Conflict Resolution

- Rules are ordered within a collection; higher rules take precedence
- Drag-and-drop reordering in UI
- Explicit priority field (integer) in YAML for programmatic control
- Conflict detection: StubHouse warns when two rules have identical matchers
- "First match" semantics (no fallthrough)
- A `default` rule with wildcard matcher serves as the catch-all (returns 404 if not defined)

#### 5.2.3 Response Scenarios

A **scenario** is a named state for a mock endpoint. The same path/method pair can have multiple named scenarios, and the active scenario can be switched at runtime without restarting the server.

Example:

```yaml
# GET /users/:id
scenarios:
  - name: success
    active: true
    response:
      status: 200
      body:
        id: "{{params.id}}"
        name: "Alice"
  - name: not_found
    response:
      status: 404
      body:
        error: "User not found"
  - name: server_error
    response:
      status: 500
      body:
        error: "Internal server error"
```

Switching scenarios: via UI toggle, via CLI `stubhouse scenario set <rule> <scenario>`, or via a special control endpoint (`POST /__mirage/scenario`).

#### 5.2.4 Stateful Mocks

StubHouse can maintain in-memory state across requests within a session. This enables mocks that actually behave like an API:

- **In-memory store:** A key-value store accessible from Rhai scripts
- **CRUD simulation:** Define a "resource" (e.g., `/users`) and StubHouse auto-generates GET list, GET by ID, POST create, PUT/PATCH update, DELETE — all backed by the in-memory store, populated with seed data from a YAML fixture file
- **State reset endpoint:** `POST /__mirage/reset` restores all in-memory state to the initial seed

Example CRUD mock declaration:

```yaml
mock_resources:
  - path: /users
    id_field: id
    seed_file: ./fixtures/users.yaml
    auto_crud: true   # enables GET/POST/PUT/PATCH/DELETE automatically
```

#### 5.2.5 Fault Injection

Mock rules can be configured to simulate real-world failure conditions:

- `fault: connection_reset` — TCP connection reset mid-response
- `fault: timeout` — never respond (triggers client timeout)
- `fault: slow_response` — configurable delay
- `fault: partial_body` — send half the response then close
- `fault: random_5xx` — randomly return 500/502/503 at a configurable probability
- Fault injection can be toggled per-rule or globally from the UI

#### 5.2.6 Request Passthrough

When a rule's `passthrough: true` is set, matching requests are forwarded to a real upstream URL (defined per-rule or globally). This enables:

- **Selective mocking:** Real API for most endpoints, mocked for the ones under development
- **Recording mode:** Forward and record real responses, then replay them offline (see 5.2.7)
- **Proxy mode:** Use StubHouse as a transparent proxy that logs all traffic

#### 5.2.7 Recording Mode

When connected to the internet and pointed at a real API, StubHouse can record all request/response pairs and save them as mock rules. Recording mode:

- Activated per-workspace or per-collection
- All recorded rules are saved to a `recordings/` directory in YAML
- Deduplication: identical path+method combos are grouped; subsequent calls become scenarios
- Sensitive data scrubbing: define regex patterns for values to redact before saving (API keys, tokens, PII)
- One-click "promote" a recording to a permanent mock rule

#### 5.2.8 Mock Server Control API

A special control namespace is served alongside your mocks at `/__mirage/`:

| Endpoint | Purpose |
|---|---|
| `GET /__mirage/status` | Server health, active rules count, active scenarios |
| `POST /__mirage/scenario` | Switch scenario for a rule |
| `POST /__mirage/reset` | Reset all stateful mock data to seed |
| `GET /__mirage/rules` | List all active rules as JSON |
| `GET /__mirage/log` | Last N request logs as JSON |
| `POST /__mirage/fault` | Enable/disable a fault injection globally |

This control API is accessible from test runners (Jest, Vitest, Playwright) so tests can programmatically switch scenarios between test cases.

---

### 5.3 Collection & Workspace Management

#### 5.3.1 Workspace

A **workspace** is the top-level organizational unit. It corresponds to a `.stubhouse/` directory that can be committed to a git repository.

- Multiple workspaces can be open simultaneously (tabs in the UI, similar to VS Code projects)
- Workspaces are self-contained: all config, mock rules, environments, and scripts live inside `.stubhouse/`
- A workspace can be opened from any directory: `stubhouse open /path/to/project`
- Recent workspaces list with quick-switch

#### 5.3.2 Collections

A **collection** is a named group of requests and/or mock rules. Collections can be nested (folders).

```
workspace/
├── collections/
│   ├── users-api/
│   │   ├── auth.yaml          # request definitions
│   │   ├── crud.yaml
│   │   └── mocks/
│   │       └── rules.yaml     # mock rules for this collection
│   └── payments-api/
│       └── ...
```

- Collections are importable/exportable independently
- A collection can reference environments, scripts, and fixtures by relative path
- Collections have a `base_url` override that takes precedence over workspace-level `base_url`

#### 5.3.3 Request Groups & Documentation

Requests within a collection can be annotated:

- `description` field (Markdown, rendered in UI)
- `example_response` field for documentation purposes
- Tags for cross-cutting filtering
- `deprecated: true` flag visually grays out a request
- Group-level README (rendered in collection sidebar)

---

### 5.4 Environment & Variable System

#### 5.4.1 Variable Scopes (resolution order, inner wins)

1. **Request-level** — inline overrides in a single request
2. **Collection-level** — apply to all requests in a collection
3. **Environment-level** — named environments (dev, staging, prod)
4. **Workspace-level** — global defaults

#### 5.4.2 Environment Files

```yaml
# .stubhouse/environments/dev.yaml
name: dev
variables:
  base_url: "http://localhost:3000"
  api_key: "dev-key-abc123"
  timeout_ms: 5000
  user_id: "usr_test001"

secrets:
  # Secrets are never stored in the file directly.
  # They are resolved at runtime from:
  # 1. OS keychain
  # 2. Environment variables
  # 3. .env file (gitignored)
  auth_token:
    source: env           # resolve from $MIRAGE_AUTH_TOKEN
  db_password:
    source: keychain      # resolve from OS keychain entry "mirage/dev/db_password"
```

- Active environment is shown in the toolbar with a color indicator
- Environment variables are shown inline in the request editor (value preview on hover)
- Environment diff view: compare variable values across environments
- Secret values are never written to the `.stubhouse/` directory — only the resolution config is stored

#### 5.4.3 Dynamic Variables

Built-in dynamic variables resolved at request time:

| Variable | Value |
|---|---|
| `{{$timestamp}}` | Unix timestamp (seconds) |
| `{{$isoTimestamp}}` | ISO 8601 datetime |
| `{{$randomUUID}}` | UUID v4 |
| `{{$randomInt(min, max)}}` | Random integer in range |
| `{{$randomEmail}}` | Realistic fake email |
| `{{$randomName}}` | Fake full name |
| `{{$env.VAR_NAME}}` | OS environment variable |
| `{{$response.body.field}}` | Field from previous response (chain requests) |
| `{{$faker.lorem.sentence}}` | Faker.js-equivalent, built in Rust |

---

### 5.5 Scripting Engine

StubHouse uses **Rhai** — a safe, sandboxed scripting language with Rust-like syntax — for all user-defined logic. Scripts have no filesystem access, no network access, and no ability to escape the sandbox.

#### 5.5.1 Pre-Request Scripts

Run before a request is sent. Can mutate request properties:

```rhai
// Pre-request: sign request with HMAC
let body = request.body.to_string();
let signature = hmac_sha256(env["signing_key"], body);
request.headers["X-Signature"] = signature;
request.headers["X-Timestamp"] = timestamp();
```

#### 5.5.2 Post-Response Scripts

Run after a response is received. Can:

- Extract values and store in variables
- Assert on response properties (fail the request with a test result)
- Chain into the next request

```rhai
// Post-response: store auth token
let token = response.json()["access_token"];
env.set("auth_token", token);

// Assert
assert(response.status == 200, "Expected 200 OK");
assert(response.json()["user"]["id"] != null, "User ID must be present");
```

#### 5.5.3 Mock Rule Scripts

Scripts can be used as matchers or response generators in mock rules:

```rhai
// As matcher condition (must return bool)
let body = request.json();
body["amount"] > 1000 && body["currency"] == "USD"

// As response generator (must return a response object)
let user_id = request.params["id"];
let user = mock_store.get("users/" + user_id);
if user == null {
    response(404, #{ "error": "not found" })
} else {
    response(200, user)
}
```

#### 5.5.4 Test Assertions

StubHouse has a built-in test runner for request collections. Each request can have a set of assertions that run post-response:

```rhai
// Assertions block — all run, results collected
test("Status is 200") { response.status == 200 }
test("Body has user field") { response.json()["user"] != null }
test("Response under 500ms") { response.time_ms < 500 }
test("Content-Type is JSON") {
    response.headers["content-type"].contains("application/json")
}
```

Test results are displayed in a test runner panel: pass/fail per assertion, timing, total pass rate. Collections can be run end-to-end as an integration test suite via `stubhouse test`.

---

### 5.6 Import / Export

#### 5.6.1 Import Formats

| Format | Support |
|---|---|
| Postman Collection v2.1 | Full import (requests, folders, variables, scripts translated to Rhai equivalents) |
| Insomnia v4 export | Full import |
| OpenAPI 3.x (YAML/JSON) | Import as collection (requests) + auto-generate mock stubs |
| Swagger 2.0 | Import as collection |
| HAR (HTTP Archive) | Import as request history or convert to collection |
| cURL | Parse cURL command into a request |
| Bruno `.bru` files | Full import |
| StubHouse native YAML | Native round-trip |

#### 5.6.2 Export Formats

| Format | Support |
|---|---|
| StubHouse native YAML | Full fidelity |
| Postman Collection v2.1 | Best-effort (Rhai scripts exported as JS equivalents where possible) |
| OpenAPI 3.x | Export collection as OpenAPI spec (inferred from request/response pairs) |
| HAR | Export history |
| Markdown | Export collection as human-readable documentation |
| Docker Compose | Export mock server as a standalone Docker service (StubHouse headless mode) |

#### 5.6.3 OpenAPI Sync

When an OpenAPI spec file is linked to a workspace:

- StubHouse watches the file for changes and updates mock stubs automatically
- Any defined response schemas are used to validate actual API responses
- Schema drift is highlighted: when a real API response doesn't match the spec, StubHouse flags it

---

### 5.7 GUI — Tauri + Svelte

#### 5.7.1 Layout

```
┌─────────────────────────────────────────────────────────┐
│  [≡ StubHouse]  [workspace: my-project ▾]  [env: dev ▾]   ●○□│
├────────┬────────────────────────────────────────────────┤
│        │  ● POST  https://api.example.com/users         │
│ Sidebar│  ──────────────────────────────────────────── │
│        │  [Params] [Headers] [Auth] [Body] [Scripts]    │
│ ▾ Users│                                                │
│   list │  Body (JSON)         ▸ Prettify  ▸ Minify      │
│   get  │  ╔══════════════════════════════════════╗      │
│   create│ ║ {                                    ║      │
│   ▾ mock│ ║   "name": "{{$randomName}}",         ║      │
│     200 │ ║   "email": "{{$randomEmail}}"        ║      │
│     404 │ ║ }                                    ║      │
│     500 │ ╚══════════════════════════════════════╝      │
│        │                                                │
│ ▾ Auth │  [▶ Send]  [▷ Send + Record]                   │
│   login│  ──────────────────────────────────────────── │
│        │  Response  200 OK  · 142ms · 1.2 KB           │
│ ┄ ┄ ┄  │  [Body] [Headers] [Cookies] [Timeline] [Tests]│
│[+ New] │  ╔══════════════════════════════════════╗      │
│        │  ║ { "id": "usr_abc123", "name": ...    ║      │
│ Mocks  │  ╚══════════════════════════════════════╝      │
│ [●] ON │                                                │
│ Port:  │                                                │
│ 4000   │                                                │
└────────┴────────────────────────────────────────────────┘
│  [Request History ▾]   [Test Results ▾]   [Mock Log ▾]  │
└─────────────────────────────────────────────────────────┘
```

#### 5.7.2 Sidebar

- Tree view: workspaces > collections > requests / mock rules
- Right-click context menus: duplicate, move, export, delete, add mock, run as test
- Drag-and-drop reordering within and across collections
- Search: fuzzy search across all requests by name, URL, method
- Filter: by tag, method, status (mocked/unmocked), recently used
- Mock status indicator per-request: green dot if a mock rule exists for this method+path

#### 5.7.3 Mock Server Panel

A dedicated bottom panel (toggleable) for the mock server:

- On/Off toggle (starts/stops the embedded server)
- Port selection with collision detection
- Live request log: method, path, matched rule, scenario, response status, latency
- Active rules list (filterable)
- Scenario switcher: dropdowns per rule to switch active scenarios
- Fault injection controls: global on/off, per-rule toggles
- Reset state button (for stateful mocks)

#### 5.7.4 Code Generation

From any request, generate a code snippet in:

- `curl`
- `httpie`
- JavaScript (fetch, axios, ky)
- TypeScript (fetch + typed)
- Python (requests, httpx)
- Go (net/http)
- Rust (reqwest)
- Ruby (net/http)
- PHP (curl, Guzzle)
- Java (OkHttp, HttpClient)
- C# (HttpClient)
- Swift (URLSession)
- Kotlin (OkHttp)

#### 5.7.5 Themes & Appearance

- Dark mode and light mode, respecting OS preference
- Code editor themes: Catppuccin Mocha (default dark), Catppuccin Latte (default light), plus Gruvbox, Tokyo Night, Solarized
- Font size control for editors
- Dense / comfortable / spacious layout density toggle
- Customizable accent color

---

## 6. Data Model

All persistent config lives as YAML files in `.stubhouse/`. Runtime state (history, logs) lives in SQLite. Below are the core schema definitions.

### 6.1 Request Definition

```yaml
# .stubhouse/collections/users/get-user.yaml
id: req_get_user
name: "Get User by ID"
description: |
  Retrieves a single user by their unique ID.
  Returns 404 if the user does not exist.
method: GET
url: "{{base_url}}/users/{{user_id}}"
params:
  - key: include_deleted
    value: "false"
    enabled: true
headers:
  - key: Authorization
    value: "Bearer {{auth_token}}"
    enabled: true
  - key: Accept
    value: application/json
    enabled: true
auth:
  type: bearer
  token: "{{auth_token}}"
body: null
pre_request_script: null
post_response_script: |
  test("Status is 200") { response.status == 200 }
  test("User ID matches") { response.json()["id"] == env["user_id"] }
tags:
  - users
  - read
```

### 6.2 Mock Rule Definition

```yaml
# .stubhouse/collections/users/mocks/get-user.yaml
id: mock_get_user
name: "Mock: Get User by ID"
priority: 100
request_ref: req_get_user   # optional: link to a request definition

matcher:
  method: GET
  path: "/users/:id"
  headers: []
  query_params: []
  body_condition: null

scenarios:
  - name: success
    active: true
    response:
      status: 200
      delay_ms: 50
      headers:
        - key: Content-Type
          value: application/json
      body:
        type: json
        content:
          id: "{{params.id}}"
          name: "Alice Nguyen"
          email: "alice@example.com"
          created_at: "{{$isoTimestamp}}"

  - name: not_found
    response:
      status: 404
      delay_ms: 30
      body:
        type: json
        content:
          error: "User not found"
          code: "USER_NOT_FOUND"

  - name: slow_success
    response:
      status: 200
      delay_ms:
        type: random
        min: 800
        max: 3000
      body:
        type: ref
        ref: success    # reuse body from success scenario

passthrough: false
fault: null
tags:
  - users
```

### 6.3 Workspace Manifest

```yaml
# .stubhouse/workspace.yaml
id: ws_my_project
name: "My Project"
version: "1"
created_at: "2025-09-01T00:00:00Z"

default_environment: dev
mock_server:
  port: 4000
  auto_start: true
  cors:
    enabled: true
    origins: ["*"]
  tls: null  # or: { cert: ./certs/server.pem, key: ./certs/server.key }

collections:
  - path: ./collections/users
  - path: ./collections/auth
  - path: ./collections/payments

scripts:
  global_pre_request: ./scripts/global-auth.rhai
  global_post_response: ./scripts/global-log.rhai
```

---

## 7. Mock Server Deep Dive

### 7.1 Route Matching Algorithm

The matcher uses a **priority trie** — a prefix tree indexed by path segments where each node carries priority metadata and match type.

Match order for a given path `/users/123/posts`:

1. Exact match: `/users/123/posts`
2. Param match: `/users/:id/posts`
3. Wildcard match: `/users/*/posts`
4. Glob match: `/users/**`
5. Catch-all: `/**`
6. Default rule (if defined)
7. 404

When multiple rules match at the same specificity level, the higher priority value wins. Ties are broken by insertion order.

### 7.2 Embedded Server Lifecycle

```
App Start
    │
    ▼
Load workspace.yaml
    │
    ▼
Parse + validate all mock rules
    │
    ▼
Build priority trie from rules
    │
    ▼
Spawn Tokio task: bind hyper server to 127.0.0.1:PORT
    │
    ▼
For each incoming request:
    │
    ├─ Extract method, path, headers, body
    │
    ├─ Evaluate trie: find highest-priority matching rule
    │
    ├─ Select active scenario for that rule
    │
    ├─ Evaluate body condition (if Rhai script): run in sandbox
    │
    ├─ Apply fault injection (if configured)
    │
    ├─ Resolve response template (interpolate variables, run Rhai body generator)
    │
    ├─ Apply configured delay
    │
    ├─ Send response
    │
    └─ Emit event to Tauri frontend (for live log panel)
```

### 7.3 Hot Reload

When any file under `.stubhouse/collections/*/mocks/` changes on disk, StubHouse:

1. Validates the changed file
2. If valid: rebuilds affected trie branches without restarting the server
3. If invalid: rejects the change and surfaces a validation error in the UI; the previous rule remains active
4. Emits a "rules reloaded" event to the frontend

This allows external editors (VS Code, Neovim) to modify mock YAML files and see changes applied instantly.

### 7.4 Concurrency

All mock server request handling runs on the Tokio async runtime. Rhai scripts run on a dedicated thread pool to prevent blocking the request loop. The in-memory state store uses `Arc<RwLock<HashMap>>` — reads are concurrent, writes are exclusive. This is sufficient for local development workloads (not designed for production load).

---

## 8. Configuration Format (`.stubhouse/`)

```
.stubhouse/
├── workspace.yaml          # workspace manifest
├── environments/
│   ├── dev.yaml
│   ├── staging.yaml
│   └── prod.yaml
├── collections/
│   ├── users-api/
│   │   ├── collection.yaml     # collection metadata, base_url, etc.
│   │   ├── list-users.yaml
│   │   ├── get-user.yaml
│   │   ├── create-user.yaml
│   │   └── mocks/
│   │       ├── list-users.yaml
│   │       ├── get-user.yaml
│   │       └── create-user.yaml
│   └── auth-api/
│       └── ...
├── fixtures/
│   ├── users.yaml              # seed data for stateful mocks
│   └── products.yaml
├── scripts/
│   ├── global-auth.rhai
│   └── global-log.rhai
├── recordings/                 # auto-generated by recording mode
│   └── 2025-09-01/
│       └── captured-001.yaml
└── .gitignore                  # auto-generated: ignores secrets, SQLite files
```

All YAML files are validated against JSON Schema on load. Schema definitions ship with StubHouse and are published for IDE integration (VS Code YAML extension can use them for autocompletion and validation of `.stubhouse/` files).

---

## 9. CLI Interface

The StubHouse CLI ships as a standalone binary (`stubhouse`) usable without the GUI. This enables CI/CD integration and headless mock server usage.

```
mirage [COMMAND] [OPTIONS]

COMMANDS:
  open <path>               Open a workspace in the GUI
  serve <path>              Start the mock server for a workspace (headless)
  test <path>               Run all request collection tests
  test <path> --collection  Run tests for a specific collection
  record <url>              Start recording mode against a target URL
  import <file>             Import a Postman/Insomnia/OpenAPI collection
  export <format>           Export the workspace to a format
  scenario set <rule> <s>   Switch the active scenario for a mock rule
  reset                     Reset all stateful mock data to seed
  validate <path>           Validate all YAML files in a .stubhouse/ directory
  schema                    Print JSON Schema for all .stubhouse/ file types

OPTIONS:
  --port <port>             Override mock server port (default: from workspace.yaml)
  --env <env>               Set active environment
  --workspace <path>        Path to .stubhouse/ directory (default: CWD)
  --json                    Output as JSON (for CI tooling)
  --verbose                 Enable debug logging
```

### CI/CD Usage Example

```bash
# In a CI pipeline: start mock server, run tests, shut down
stubhouse serve . --port 4000 --env test &
MOCK_PID=$!
sleep 1

# Run your test suite against the mock server
npm test

# Switch scenario and run another test pass
stubhouse scenario set mock_payment checkout_failure
npm test -- --grep "error handling"

kill $MOCK_PID
```

---

## 10. Plugin System

StubHouse exposes a plugin API for extending the app without forking it. Plugins are Rhai scripts or WebAssembly modules placed in `.stubhouse/plugins/`.

### Plugin Types

| Type | Purpose |
|---|---|
| `auth_provider` | Custom auth strategy (e.g., custom signature scheme) |
| `body_transformer` | Transform request/response body (e.g., decrypt a custom encoding) |
| `mock_generator` | Generate mock responses from a custom logic source |
| `importer` | Add support for importing a new collection format |
| `theme` | Custom UI color theme definition (JSON) |

### Plugin Manifest

```yaml
# .stubhouse/plugins/my-auth-plugin/plugin.yaml
id: my_custom_auth
name: "HMAC-SHA256 Auth"
type: auth_provider
entry: ./auth.rhai
version: "1.0.0"
description: "Signs requests using HMAC-SHA256 with a rotating key"
```

Plugin distribution: plugins can be shared as single `.stubhouse-plugin` archive files (a zip with a `plugin.yaml` manifest). The community plugin registry is a future consideration.

---

## 11. Roadmap & Phases

### Phase 1 — Core (Months 1–3)

**Goal:** A working request client with file-based config. Usable as a Postman replacement for basic workflows.

- [ ] Tauri 2 + Svelte project scaffold
- [ ] `.stubhouse/` directory structure and YAML schema definitions
- [ ] Config parser + validator (Rust)
- [ ] Request engine: HTTP/1.1 + HTTP/2 + TLS (hyper + rustls)
- [ ] Basic request editor UI (URL, method, headers, body, auth)
- [ ] Response viewer (JSON tree, raw, headers, status)
- [ ] Environment system (multi-env, variable interpolation)
- [ ] Request history (SQLite)
- [ ] Import: Postman Collection v2.1
- [ ] Export: StubHouse native YAML + cURL snippet

**Milestone:** Can replace Postman/Insomnia for daily request sending, fully offline.

---

### Phase 2 — Mock Server (Months 4–6)

**Goal:** Functional embedded mock server. The feature that differentiates StubHouse.

- [ ] Embedded hyper server in Tokio task
- [ ] Mock rule YAML schema + parser
- [ ] Priority trie route matcher
- [ ] Static response scenarios
- [ ] Scenario switcher (UI + CLI)
- [ ] Mock server panel in UI (live log, on/off, port)
- [ ] Hot reload of mock rules on file change
- [ ] Control API (`/__mirage/*`)
- [ ] Fault injection: timeout, slow_response, connection_reset
- [ ] Request passthrough mode
- [ ] `stubhouse serve` CLI command

**Milestone:** A frontend developer can run `stubhouse serve .` and have a fully functional mock API for their entire project, switchable between scenarios, with no real backend needed.

---

### Phase 3 — Scripting & Testing (Months 7–9)

**Goal:** Automation layer that makes StubHouse useful for integration testing.

- [ ] Rhai scripting engine integration
- [ ] Pre-request + post-response scripts
- [ ] Mock rule Rhai conditions and generators
- [ ] Test assertion syntax + test runner
- [ ] `stubhouse test` CLI command with JUnit XML output (for CI)
- [ ] Collection-level test runner UI panel
- [ ] Dynamic variables (`$randomUUID`, `$faker.*`, etc.)
- [ ] Variable chaining across requests

**Milestone:** A CI pipeline can run `stubhouse test .` and get a pass/fail result with per-assertion detail.

---

### Phase 4 — Advanced Mock Features (Months 10–12)

**Goal:** Stateful mocks and recording make StubHouse a serious offline development platform.

- [ ] In-memory state store + CRUD simulation
- [ ] Fixture files + seed data
- [ ] Recording mode (passthrough + capture)
- [ ] Sensitive data scrubbing config
- [ ] Import: Insomnia, OpenAPI 3.x, HAR, Bruno
- [ ] Export: OpenAPI 3.x, Markdown docs, Docker Compose
- [ ] OpenAPI spec sync + response schema validation
- [ ] Schema drift detection and highlighting

**Milestone:** A developer can record a real API session, go offline, and have a stateful mock that correctly simulates create/update/delete operations.

---

### Phase 5 — Polish & Ecosystem (Months 13–18)

**Goal:** Release-quality product with community-facing features.

- [ ] Plugin system (auth_provider, body_transformer)
- [ ] VS Code YAML schema integration (publish schemas to SchemaStore)
- [ ] Full code generation (all target languages)
- [ ] mTLS + client certificate support
- [ ] WebSocket mock support (upgrade connection, send mock frames)
- [ ] gRPC support (basic: send unary requests, display proto-decoded response)
- [ ] Performance: response timeline, waterfall view
- [ ] Keyboard shortcuts + command palette
- [ ] Dark/light theme polish, accent colors
- [ ] Public documentation site
- [ ] Release: GitHub Releases, Homebrew tap, .deb/.rpm/.msi installers

---

## 12. Competitive Differentiation

| Feature | StubHouse | Postman | Insomnia | Bruno | Yaak |
|---|---|---|---|---|---|
| Local-first (no account) | ✅ | ✗ (cloud-first) | ✅ | ✅ | ✅ |
| Files as source of truth | ✅ YAML | ✗ proprietary DB | ✗ proprietary DB | ✅ .bru | ✅ |
| Embedded mock server | ✅ first-class | ✅ paid add-on | ✗ | ✗ | ✗ |
| Stateful mocks | ✅ | ✗ | ✗ | ✗ | ✗ |
| Scenario switching | ✅ | ✗ | ✗ | ✗ | ✗ |
| Recording mode | ✅ | ✅ paid | ✗ | ✗ | ✗ |
| Fault injection | ✅ | ✗ | ✗ | ✗ | ✗ |
| Mock control API | ✅ | ✗ | ✗ | ✗ | ✗ |
| CI/CD headless mode | ✅ | ✅ Newman | ✗ | ✅ | ✗ |
| OpenAPI schema validation | ✅ | ✅ | ✅ | ✗ | ✗ |
| Native binary (no Electron) | ✅ Tauri | ✗ Electron | ✗ Electron | ✅ Tauri | ✅ Tauri |
| Scripting language | Rhai (sandboxed) | JS (V8) | JS (V8) | JS (V8) | None |
| Free and open source | ✅ | ✗ | ✅ | ✅ | ✅ |

**StubHouse's defensible edge:** No other tool in this category treats the mock server as a first-class, co-equal feature with stateful semantics, scenario switching, fault injection, and a control API designed for integration with test runners. This combination is the moat.

---

## 13. Non-Goals

These are explicitly out of scope for v1.0 and should not creep in:

- **Cloud sync / team collaboration** — StubHouse is a local tool. Teams collaborate via git. Cloud sync is a potential future monetization vector, not a v1 feature.
- **API documentation hosting** — StubHouse can export Markdown, but it does not host docs.
- **API gateway / production proxy** — The embedded server is for local development only. It is explicitly not hardened for public exposure.
- **Database query tool** — Not a DB client.
- **Load testing** — Not a performance testing tool. `stubhouse test` runs functional assertions, not load scenarios.
- **Protocol buffers / gRPC full support** — Phase 5 adds basic gRPC unary; full streaming gRPC is post-v1.
- **Browser extension** — Not a web-based tool.
- **Mobile app** — Desktop only.

---

## 14. Open Questions

These are decisions deferred until implementation reveals the right answer:

1. **Plugin execution model:** Should third-party plugins run as Rhai scripts (limited but safe) or as WASM modules (more powerful but complex sandboxing)? The safe default is Rhai-only for v1.
2. **Mock server port conflict UX:** When the configured port is taken, should StubHouse auto-select the next available port or surface an error? Auto-select is friendlier but may silently break frontend config that hardcodes port 4000.
3. **Binary size target:** Tauri apps with a Svelte frontend and Rust backend can realistically land around 15–25 MB. Is this acceptable, or should we budget-track it explicitly?
4. **YAML vs TOML:** YAML was chosen for familiarity (most developers already know it from Docker Compose / CI configs). TOML is arguably cleaner but less common in this context. Decision: YAML, final, unless there's a strong technical reason to reconsider.
5. **SQLite WAL mode for history:** With WAL enabled, read queries from the UI won't block writes from the mock server log. Should be the default. Confirm during implementation.
6. **Rhai `unsafe` escape hatch for power users:** Some advanced scripting use cases (e.g., binary body manipulation) may require low-level byte access. Should we expose a controlled `unsafe` mode that allows more Rhai capability at the cost of the sandbox guarantee? Defer to v1.1.

---

*Spec version 1.0.0-draft. Written for StubHouse v1.0. Subject to revision during Phase 1 implementation.*
