# StubHouse TODO

Status snapshot, derived from `spec.md` (Roadmap §11) against the current codebase.
Top of list is "do next". Mark with `[x]` when done.

---

## Phase 1 — Core

- [x] Tauri 2 + Svelte project scaffold
- [x] `.stubhouse/` directory structure (`workspace.yaml`, `collections/`)
- [x] Config parser + validator (Rust) — `workspace.rs`
- [x] Request engine: HTTP/1.1 + HTTP/2 + TLS via `reqwest`+`rustls`
- [x] Basic request editor UI (URL, method, headers, body, auth)
- [x] Response viewer (JSON tree, raw, headers, status)
- [x] Request history (SQLite via `rusqlite`)
- [x] Compose layer with bearer / basic / api-key auth + json/text/form bodies
- [x] Environment file parser + variable resolution (core) — `environment.rs`
- [x] Variable interpolation engine (core) — `interpolate.rs`
- [x] **Wire environments + interpolation through Tauri commands**
- [x] **Environment switcher UI** (header dropdown + color dot)
- [x] **Inline resolved-URL preview under the URL bar**
- [x] **Export: Copy as cURL** (button in request pane, POSIX-quoted)
- [x] **Import: Postman Collection v2.1** (sidebar Import button, file picker)
- [x] **CLI** (`stubhouse init / validate / list / show / envs / import postman / export curl`)
- [x] Align `environments/` with `collections/` at workspace root (was misnested under `.stubhouse/`)

**Phase 1 exit criteria reached:** can replace Postman/Insomnia for daily request sending, fully offline. ✅

---

## Phase 2 — Mock Server (the differentiator)

- [x] `stubhouse-mock` module: embedded hyper server bound to `127.0.0.1:PORT`
- [x] Mock rule YAML schema (`collections/*/mocks/*.yaml`) + parser
- [x] Priority trie route matcher (exact > `:param` > `*` > `**` > catch-all)
- [x] Scenario model (named response states, `active: true`)
- [x] Scenario switcher — UI dropdown + CLI command
  - [x] CLI: `stubhouse scenario list` / `stubhouse scenario activate <name>`
  - [x] UI dropdown
- [x] Mock server panel in UI: on/off toggle, port picker, live request log
- [x] Hot reload: watch `.stubhouse/collections/**/mocks/*.yaml` for changes
- [x] Control API at `/__mirage/*` (status, scenario, reset, rules, log)
- [x] Fault injection: `timeout`, `slow_response`, `connection_reset`, `partial_body`, `random_5xx`
- [x] Request passthrough (selective proxying to real upstream)
- [x] `stubhouse serve` CLI command (headless)

**Phase 2 exit criteria:** `stubhouse serve .` gives a frontend developer a fully functional mock API with scenario switching, no real backend needed.

---

## Phase 3 — Scripting & Testing

- [x] Embed `rhai` scripting engine
- [x] Pre-request scripts (mutate `request.*`)
- [x] Post-response scripts (assertions, variable extraction)
- [x] Mock rule Rhai conditions + body generators
- [x] `test(...)` assertion DSL + test runner
- [x] `stubhouse test` CLI with JUnit XML output
- [x] Test results panel in UI
- [x] Dynamic variables: `$randomInt`, `$randomEmail`, `$randomName`, `$faker.*`, `$response.*` chaining
  - [x] `$timestamp`, `$isoTimestamp`, `$randomUUID`, `$env.*`, `$randomInt`, `$randomEmail`, `$randomName`, `$faker.*`, `$response.*` already done

---

## Phase 4 — Advanced Mock Features

- [x] In-memory state store + CRUD simulation (`mock_resources`)
- [x] Fixture files + seed data
- [ ] Recording mode (passthrough + capture to YAML)
- [ ] Sensitive data scrubbing config
- [ ] Imports: Insomnia v4, OpenAPI 3.x, HAR, Bruno `.bru`
- [ ] Exports: OpenAPI 3.x, Markdown docs, Docker Compose (headless mock service)
- [ ] OpenAPI spec sync + response schema drift detection

---

## Phase 5 — Polish & Ecosystem

- [ ] Plugin system (`auth_provider`, `body_transformer`, `mock_generator`)
- [ ] VS Code YAML schema integration (publish to SchemaStore)
- [ ] Full code generation (curl, httpie, JS, TS, Python, Go, Rust, Ruby, PHP, Java, C#, Swift, Kotlin)
- [ ] mTLS + client certificate support
- [ ] WebSocket mock support
- [ ] Basic gRPC unary support
- [ ] Timeline / waterfall view
- [ ] Keyboard shortcuts + command palette
- [ ] Theme polish (Catppuccin, Gruvbox, Tokyo Night, Solarized)
- [ ] Public docs site
- [ ] Release: GitHub Releases, Homebrew, .deb/.rpm/.msi

---

## Open questions (track until resolved — spec §14)

- [ ] Plugin model: Rhai-only for v1, or WASM too?
- [ ] Mock port-conflict UX: auto-pick next free port, or surface error?
- [ ] Binary-size budget for Tauri build?
- [ ] SQLite WAL mode for history — enable by default during Phase 1 wrap-up
- [ ] Rhai `unsafe` escape hatch — deferred to v1.1
