# StubHouse

> *Stub it. Ship it.*

Local-first API client and mock server. See [`spec.md`](./spec.md) for the full product spec.

## Status

Phase 1 vertical slice: URL bar → Rust HTTP engine → response viewer.

## Layout

```
crates/
  stubhouse-core/   pure Rust: HTTP engine, mock runtime (future), config parser (future)
  stubhouse-cli/    `stubhouse` binary (stub)
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
