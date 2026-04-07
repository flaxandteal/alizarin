# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Alizarin is a TypeScript/Rust WASM SDK for working with [Arches](https://www.archesproject.org/) graph-based data management systems. It provides an ORM layer over Arches' resource models. Pure JS/TS reimplementation of [AORM](https://github.com/flaxandteal/arches-orm/) for browser and Node.js.

Licensed AGPL-3.0. PRs must be MIT-licensed for potential future relicensing.

## Build & Development Commands

```bash
# Full build (WASM + JS)
npm run build

# WASM only (requires wasm-pack)
npm run build:wasm        # release
npm run build:wasm:dev    # debug

# JS only (requires WASM already built in pkg/)
npm run build:js

# Dev server
npm run dev

# Tests (vitest, bails on first failure)
npm test                  # single run
npm run test:watch        # watch mode
npm run test:ui           # vitest UI

# Lint (typescript-eslint, currently allows 46 warnings)
npm run lint

# Validate graph definitions
npm run validate

# Sync version across Cargo.toml, package.json, VERSION
npm run version:set
```

All JS/test commands need `NODE_OPTIONS='--experimental-wasm-modules'` (already set in package.json scripts).

Rust linting (pre-commit hooks):
```bash
cargo fmt --all -- --check
cargo clippy --workspace --exclude alizarin-python -- -D warnings
```

## Architecture

### Two-layer design: Rust WASM core + TypeScript ORM

**Rust crates** (`crates/`):
- `alizarin-core` — Core data structures, graph traversal, JSON conversion, SKOS parsing, type coercion, label resolution. Shared between WASM and Python targets.
- `alizarin-wasm` — WASM bindings (`wasm-bindgen`) that wrap core functionality. Builds to `pkg/` via `wasm-pack`.
- `alizarin-python` — PyO3 bindings (separate build, excluded from workspace clippy).
- `alizarin-explorer` — TUI explorer tool for Arches graphs.
- `alizarin-extension-api` — Extension API types.
- `wasm-wrapper-derive` — Proc macro for generating WASM wrapper boilerplate.

**TypeScript** (`js/`):
- `main.ts` — Public API surface, re-exports everything.
- `_wasm.ts` — WASM initialization (browser fetch vs Node.js sync). Handles `setWasmURL()` for custom WASM locations.
- `graphManager.ts` — `GraphManager` and `ResourceModelWrapper` — central ORM: loads graph definitions, wraps resource instances.
- `client.ts` — Client abstraction (`ArchesClientRemote`, `ArchesClientRemoteStatic`, `ArchesClientLocal`).
- `viewModels.ts` / `viewModels/` — View model types (String, Number, Date, Concept, GeoJSON, ResourceInstanceList, Semantic).
- `semantic.ts` — Semantic (nested) data resolution with timing instrumentation.
- `rdm.ts` — Reference Data Manager for concept/vocabulary lookups.
- `validation/` — Separate entry point (`alizarin/validation`), JSON schema validation with ajv.

**Build output**:
- `pkg/` — WASM build output (wasm-pack). Contains `alizarin_bg.wasm` and generated JS/TS bindings.
- `dist/` — Final JS library build (Vite). Entry points: `alizarin.js`, `alizarin.inline.js`, `validation/index.js`.

### WASM initialization flow

WASM auto-initializes after a microtask via `Promise.resolve().then(() => initWasm())`. Consumers can call `setWasmURL()` synchronously before this fires. `wasmReady` promise gates WASM-dependent operations.

### Key patterns

- Properties use both `camelCase` and `snake_case` (ESLint allows both, matching Arches API conventions).
- `@` path alias maps to `js/` in vitest config.
- Vite build externalizes `fs`, `path`, `url`, `ajv`, `ajv-formats`. A custom plugin replaces base64-inlined WASM with a file reference (workaround for Vite 5).
- Tests use `vitest` with thread pool and `--experimental-wasm-modules`. Setup in `tests/setupVitest.js`.
- Test data in `tests/data/` and `tests/definitions/` includes Arches resource models and graph definitions (third-party, noted in README).

### Extension system

Browser extensions live in `ext/` (JS and Python subdirs). Built separately via `.github/workflows/extensions.yml`. Extension handlers registered via `registerExtensionHandler()`.
