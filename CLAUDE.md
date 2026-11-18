# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Alizarin is a TypeScript/Rust WASM SDK for working with [Arches](https://www.archesproject.org/) graph-based data management systems. It provides an ORM layer over Arches' resource models. Pure JS/TS reimplementation of [AORM](https://github.com/flaxandteal/arches-orm/) for browser and Node.js.

Licensed AGPL-3.0. PRs must be MIT-licensed for potential future relicensing.

## Backend parity is mandatory

Alizarin has three backends over one Rust core: **WASM** (`crates/alizarin-wasm`, plus the TS layer in `js/`), **NAPI** (`crates/alizarin-napi`) and **Python** (`crates/alizarin-python`). **Every enhancement must keep them in parity unless there is a specific reason one backend cannot or should not have it, and must update the parity tests in the same change.** A feature that lands in one binding only is unfinished.

- **Logic lives in `alizarin-core`.** Bindings are thin wrappers: parse arguments, call core, serialise core's result. Where a binding needs a JSON boundary, add a `*_from_json` function in core (e.g. `compose_resource_layers_from_json`, `lifecycle_state_from_json`) so each wrapper is a one-liner.
- **Output is core's serialisation, byte for byte.** WASM returns through `js_json::to_value`, NAPI through `json_arg::to_js` or a `serde_json::Value`, Python through `pythonize` / `alizarin.canonical_static`. No hand-built `toJSON`/`to_dict`. Absent values are `null`, never `undefined`. Rules are in `notes/backend-support-matrix.md` and linted by `tests/sourceParity.test.ts`.
- **Update the parity tests with the feature:**
  - New output → add a case to `crates/alizarin-core/tests/output_parity_test.rs` (fixtures in `tests/data/output-parity/`), regenerate with `ALIZARIN_UPDATE_GOLDEN=1 cargo test -p alizarin-core --test output_parity_test`, review the `expected.json` diff, and add the same route to `tests/dual-backend/outputParity.test.ts` (WASM and NAPI) and `crates/alizarin-python/tests/test_output_parity.py`.
  - New wrapper / pseudo-value / tile method → list it in `tests/binding-parity.json` for every binding.
  - New Python-side field on a mirrored type (`python/alizarin/static_types.py`) → keep it in step with the Rust struct.
- **A deliberate difference must be written down:** as `null` with an `ecosystem:` (by design) or `gap:` (to fix) reason in `tests/binding-parity.json`, and in the matrix in `notes/backend-support-matrix.md`.
- **Run all three before calling it done:** `npm test` (WASM), `npx vitest run --config vitest.napi.config.js` after `npx napi build --platform --release` in `crates/alizarin-napi` (NAPI), and `maturin develop --release` + `pytest` in `crates/alizarin-python` using the repo's `.venv` (Python).

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
cargo clippy --workspace -- -D warnings
```

## Architecture

### Two-layer design: Rust WASM core + TypeScript ORM

**Rust crates** (`crates/`):
- `alizarin-core` — Core data structures, graph traversal, JSON conversion, SKOS parsing, type coercion, label resolution. Shared between WASM and Python targets.
- `alizarin-wasm` — WASM bindings (`wasm-bindgen`) that wrap core functionality. Builds to `pkg/` via `wasm-pack`.
- `alizarin-python` — PyO3 bindings (separate build via maturin; excluded from workspace `cargo test`, as an extension module can't link a test binary).
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
