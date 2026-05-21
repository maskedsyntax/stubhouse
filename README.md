# StubHouse

> *Stub it. Ship it.*

Local-first API client and mock server. See [`spec.md`](./spec.md) for the full product spec.

## Status

0.2.0-dev: local-first API client, file workspaces, environments, imports/exports, request history, an embedded mock server, scenarios, hot reload, and a Rhai-based test runner.

## Layout

```
crates/
  stubhouse-core/   pure Rust: HTTP engine, workspace files, mock runtime, test runner
  stubhouse-cli/    `stubhouse` automation binary
  stubhouse-app/    Tauri 2 desktop app
src-frontend/       Svelte 5 + TS + Tailwind UI
```

## Run (dev)

```
cd src-frontend
bun install           # first time only
bun run tauri dev     # launches the desktop app
```

## Test

```
cargo test -p stubhouse-core
```

## CLI

```
cargo run -p stubhouse-cli -- init acme-api
cargo run -p stubhouse-cli -- import openapi ./openapi.yaml
cargo run -p stubhouse-cli -- serve --port 4000
cargo run -p stubhouse-cli -- scenario list
cargo run -p stubhouse-cli -- test --junit results.xml
```
