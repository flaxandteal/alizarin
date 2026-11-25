# Extension interface split: no extension module links a second core

Status: **done** (2026-11-25), as below, except:

- `StaticEdge` moved too (it lives in the same file as the node types); core's
  `CallbackGraph` was removed rather than kept, having no callers left.
- Ext core tests run under their default `core` feature, as pg's do, rather
  than through a separate dev-dependency.
- WASM keeps its own extension-mutation registry (JS functions can't live in
  core's thread-safe one); Python and NAPI share core's (`mutation_abi`).
- Bugs here are registered in `PARITY-RESEARCH.md` as G-8 (G1), G-9 (G2),
  G-11 (C1), G-12 (C2), G-13 (D3), G-14 (G3), G-15 (C3, open). Found while
  executing: G-10 (`@alizarin/filelist` never registered on NAPI), D-11 (the
  NAPI tests' worker threads shared native globals; and the row sink's test
  raced the global language), D-12 (every ext Python library was `lib_rust.so`
  in the shared target dir).

## Why

Every shipped extension module (`.so`, `.node`, `.wasm`) of clm, filelist and
geo links its own copy of alizarin-core, through its `ext/*/core` crate. That
copy has its own, empty, globals: graph registry, extension registry, RDM
cache, functions registry, current language. Anything an extension reads from
them is wrong, silently. pg was the first case (G-4, fixed in c3f4950 by
linking no core); geo has a live one now (bug G1 below).

The conventional fix, and the one this plan takes: a **stateless interface
crate** that both sides compile against, holding only data types, traits and
the extension side of the C ABI; state stays in the host and reaches the
extension through the host callbacks that already exist. Moving a type into
the interface crate is compile-time only: it is still compiled twice, so the
ABI keeps passing JSON values and `repr(C)` rows, as now, and the fingerprint
still guards layout.

The interface crate is the existing `crates/alizarin-extension-api`. Core
re-exports everything moved, so no host-side path changes.

## Findings this plan rests on

Mapped by three read-only audits of every `alizarin_core` use in `ext/` and
the closure of each item in core; the claims marked ✔ were re-checked by hand.

**What ships (non-test, non-feature) per extension:**

| Extension | Uses from core on its C-ABI / WASM paths | Everything else |
|---|---|---|
| clm | `IndexSpec`, `IndexClass` (serialised to the host, `c_abi.rs:245`) | `ExtensionTypeHandler` impl, `register()`, `with_global_rdm_cache` (in-process only), `mutations.rs` (feature, off in wheels) |
| filelist | nothing | `ExtensionTypeHandler` impl, `register()` |
| geo | `ExtensionError`, `ValidationResult`; derive glue `derive_with`, `derive_provider_info`, `CallbackGraph`; `DeriveConfig`, `ResolvedNode`, `StaticTile`, `DeriveProvider`, `GraphLookup`; `coerce_value_with_registry` | `GeoTypeHandler`, `register()`, `register_functions`, `register_global` (tests only) |

No extension uses `get_current_language` directly, `bump_handlers_generation`,
SKOS, label resolution or core's uuid helpers.

**Why not move the whole handler surface.** `ExtensionTypeHandler::render_display`
takes `SerializationContext`, which holds `NodeConfig`, whose
`DisplayAsListCache` reads the `HANDLERS_GENERATION` global, and the
registry/handler/context form a cycle. Moving it means moving or reshaping
`NodeConfig` for no shipped benefit: nothing across the boundary reads the
context (`AbiExtensionTypeHandler` and `JsExtensionTypeHandler` both ignore
it). Likewise core's full `GraphLookup` returns cards, edges, `CardIndex` and
descriptors; UTM calls one method of it. So the handler trait, the registry,
`GraphLookup` and the graph structs **stay in core**; the in-process impls in
each ext core go behind a `core` feature, the pattern pg-core already uses.

## Live bugs found (fixed in this programme)

- **G1 — geo writes `string` outputs in the wrong language** ✔.
  `ext/geo/core/src/utm.rs:182` calls `coerce_value_with_registry(.., None, None)`;
  `coerce_string` falls back to `get_current_language()`
  (`type_coercion/strings.rs:23-25`), i.e. geo's own never-set copy: always
  `"en"`. Every binding; no test covers a `string` output across the
  boundary. Fixed by step 4.
- **G2 — `@alizarin/geo-napi` doesn't register `"utm"`** ✔.
  `ext/geo/napi/register.js` registers `geoHandlerPtr()` only; the JS package
  registers both. Fixed in step 6.
- **C1 — stored concept URIs are dropped on the C-ABI path** ✔. The host's
  concept lookups build the JSON without `uri`
  (`crates/alizarin-core/src/extension_abi.rs:807`, `:861`), so clm always
  builds a CLM item URI. Affects Python and NAPI `resolve_markers`. The test
  that checks stored URIs (`reference_uses_stored_concept_uri_verbatim`)
  covers only the in-process path, which ships nowhere. Fixed in step 1.
- **C2 — clm's base URI can't be set on NAPI.** `CLM_BASE_URI` is clm's own
  global (correct: it lives in the same module as the handler), but the napi
  binding exports only `referenceHandlerPtr`. Python exports the setter. Fixed
  in step 6.

Recorded, not fixed here (new PARITY-RESEARCH entries, step 8):

- **C3 — bare-UUID reference display differs.** The in-process clm handler
  resolves bare UUIDs through `ctx.external_resolver`; the C-ABI render has no
  resolver (the host drops `_ctx`). Needs a resolver callback in
  `TypeHandlerInfo` — an ABI change of its own. Unverified by running.
- **G3 — geo JS: `registerDeriveProvider("utm", {derive})` may reject on the
  napi backend** (`js/src/main.ts:24-39` vs `js/backend.ts:319`, which wants a
  bigint). Unverified; check during step 6 and fix there if real.

## Target shape

```
alizarin-extension-api   (serde, serde_json; optional wasm deps; no globals)
  ├─ C ABI types (as now)
  ├─ handler data:  ExtensionError, ValidationResult, IndexSpec, IndexClass
  ├─ graph data:    StaticNode, StaticNodegroup, StaticTranslatableString
  │                 (+ their pure methods, sorted_serde, serde_helpers)
  ├─ derive:        DeriveConfig, DeriveMode, ResolvedNode, StaticTile,
  │                 DeriveLookup (slim trait, typed helpers), ExtDeriveProvider,
  │                 derive_with, free_derive_result, derive_provider_info
  └─ mutation:      MutationHandlerInfo (as now) + mutation_handler_info helper
        ▲                                  ▲
alizarin-core (re-exports all of it;       ext/<x>/core
  host side of every ABI; GraphLookup,       default = ["core"]  → in-process impls
  handler trait, registries, globals)        no-default          → api only
                                                   ▲
                                 ext/<x>/{python,napi,wasm}: default-features = false
```

Invariant, guarded in CI (step 7): `cargo tree -e normal` of every
`ext/*/{python,napi,wasm}` crate contains no `alizarin-core`.

## Steps

### 1. Host fixes that don't depend on the split

- C1: add `"uri": concept.uri` to both lookup JSON objects in
  `extension_abi.rs` (807, 861). Add a host-side test through the C ABI with a
  stored URI (extend `crates/alizarin-core` extension_abi tests or clm's
  python tests, which go through the real `.so`).

### 2. Move the handler data types to the api crate

From `crates/alizarin-core/src/extension_type_registry.rs` to a new
`crates/alizarin-extension-api/src/handler.rs` (re-exported at the api root):
`ExtensionError` (+ `new`, `Display`, `Error`), `ValidationResult` (+ its
constructors), `IndexClass`, `IndexSpec`. Core: `pub use
alizarin_extension_api::{...}` in `extension_type_registry` at the same paths,
so `alizarin_core::extension_type_registry::IndexSpec` etc. still resolve.

Only these four: they are what extension code reaches without the trait.
`HandlerCapabilities`, `RemoveStep`, `CoercionResult`, `ExternalResolver` stay
with the trait (they could move later; nothing needs them to).

Check: serde attributes come across verbatim (IndexClass is a tagged enum —
the host deserialises clm's JSON, so a changed tag would break silently; the
existing clm index-spec test through the host covers it).

### 3. Move the derive interface to the api crate

New `crates/alizarin-extension-api/src/derive.rs`:

- `StaticTile` — struct and `new_empty`, verbatim from `graph/tile.rs:175`.
  `ensure_id` (uuid v4) has no callers in crates/ or ext/ ✔: delete it, rather
  than give the api crate uuid. Free functions `canonical_tile_id`,
  `multi_tile_id`, `allocate_multi_tile_id`, `tile_namespace` stay in core.
- `DeriveMode`, `DeriveConfig` (+ `default_overwrite`, `node()`),
  `ResolvedNode` from `graph/computed_tiles.rs`. `DeriveConfig::resolve_nodes`
  takes `&dyn GraphLookup`, so it becomes a core free function
  `resolve_derive_nodes(&config, graph)`; two callers ✔
  (`computed_tiles.rs:149`, `functions.rs:318`) plus geo's test at
  `utm.rs:498`.
- `StaticNode`, `StaticNodegroup`, `StaticTranslatableString` (`graph/nodes.rs`,
  `graph/translatable.rs`), with their inherent methods, which are pure
  (`is_root`, `display_name`, `display_alias`, `to_json`, …; and
  `node_compare.rs`'s `compare`, pure serde_json, moves with `StaticNode`),
  and the helpers their serde needs (`sorted_serde.rs`,
  `graph/serde_helpers.rs`). Core re-exports at the same paths.
  `GraphLookup`, cards, edges, `CardIndex`, descriptors **stay in core**: they
  describe a fully loaded graph, which only the host has (D4).
- `DeriveLookup`: the graph view a provider across the boundary gets. One
  required method, `lookup(&self, kind: u32, key: &str) -> Option<Value>`
  with the existing `LOOKUP_*` kinds, plus provided typed helpers that
  deserialise it: `root() -> Option<StaticNode>`, `node(id)`,
  `node_by_alias(alias)`, `nodegroup(id) -> Option<StaticNodegroup>`. A future
  extension needing more of the graph gets a new lookup kind and helper:
  additive, no break.
- `ExtDeriveProvider`: `fn derive(&self, resource_id, graph: &dyn DeriveLookup,
  tiles: &[StaticTile], config: &DeriveConfig) -> Result<Vec<StaticTile>, String>`.
  Contract (new, documented): **outputs are raw values; the host coerces them
  to each node's datatype** (step 4).
- `derive_with`, `free_derive_result`, `derive_provider_info` moved from
  `core::derive_abi` (367-430), now generic over `ExtDeriveProvider` and
  building a closure-backed `DeriveLookup` over `HostGraph` instead of
  `CallbackGraph`.

Core keeps: `DeriveProvider` (over `&dyn GraphLookup`, used in-process and by
WASM's `JsDeriveProvider`), `CallbackGraph` (used by tests and any in-process
JSON caller), the host side of `derive_abi`. Core adds:

- `GraphLookupView<'a>(&'a dyn GraphLookup)`: `impl DeriveLookup`, answering
  the `LOOKUP_*` kinds by serialising the node/nodegroup (the same answers
  `host_lookup` gives; share the code).
- `InProcess<P>(pub P)`: `impl<P: ExtDeriveProvider> DeriveProvider`, wrapping
  the graph in `GraphLookupView` and coercing outputs (step 4). This is how a
  provider written against the api registers in-process
  (`RegisteredFunction::Derive(Arc::new(InProcess(UtmDeriveProvider)))`).

### 4. The host coerces derived outputs (fixes G1)

One core function, `coerce_derived_tiles(graph, &mut [StaticTile], registry)`:
each value in a derived tile is coerced for its node's datatype with the
host's language and the host's extension registry; an error names the node.
Called by every cross-boundary adaptor:

- `AbiDeriveProvider::derive` after `read_result` (`derive_abi.rs:111-160`);
- WASM `JsDeriveProvider::derive` (`crates/alizarin-wasm/src/derive_registry.rs:81-140`);
- `InProcess<P>` (step 3).

In-process core `DeriveProvider`s are unchanged (they already produce coerced
data). Risk: JS/ABI providers that already emit coerced data (Gramadán's
strings) are coerced twice. `coerce_string` on an already-wrapped dict
normalises, so it should be idempotent — **assert it** with a test per
coercer the existing on-load/stored derive tests touch, before relying on it.

Geo then emits raw values: `zone` as a number, `hemisphere` as `"N"`/`"S"`,
easting/northing as rounded f64. Its datatype check (reject anything but
`number`, `non-localized-string`, `string`) stays local, so the error message
stays geo's.

**ABI 7**: the derive contract changes (raw outputs). An old geo with a new
host still works (re-coercion is idempotent); a new geo with an old host
would store raw values. Bump `ABI_VERSION` so the fingerprint refuses that
pairing. Update `docs/extension-abi.mdx` (version, history, the derive
section's contract).

### 5. Feature-gate the ext cores

Each `ext/{clm,filelist,geo}/core/Cargo.toml`:
`[features] default = ["core"]; core = ["dep:alizarin-core"]`, core optional,
`alizarin-core` also in `[dev-dependencies]` (tests stay as they are). Behind
`#[cfg(feature = "core")]`:

- clm: `impl ExtensionTypeHandler for ReferenceTypeHandler`,
  `create_reference_handler`, `register`, `resolve_reference_markers`
  (the `with_global_rdm_cache` one), `render_reference_display_value_resolved`
  (takes `ExternalResolver`). `reference_index_spec` now returns the api's
  `IndexSpec`.
- filelist: `impl ExtensionTypeHandler for FileListTypeHandler`,
  `create_filelist_handler`, `register`.
- geo: `GeoTypeHandler` and its impl, `create_geo_handler`, `register`,
  `utm::register_functions` (via `InProcess`). **Delete `utm::register_global`**:
  it writes geo's own copy of the functions registry, no production caller ✔,
  and it is exactly the footgun this plan removes. Its test moves to
  `register_functions` on a local registry. `fold.rs`, `c_abi.rs` and
  `utm.rs`'s production code switch to api imports. `derive_json` (WASM) uses
  the api `derive_with` path's `DeriveLookup` closure.

Every `#[cfg(test)]` that uses core needs `#[cfg(all(test, feature = "core"))]`
only if it can't run without the feature; with core as a dev-dependency the
tests compile either way, so prefer leaving them as `#[cfg(test)]` and
importing from `alizarin_core` directly.

Also fix geo's stale doc pointers (`registry.rs:76`, `:691` name
`alizarin_geo_core::register_functions`; it is `utm::register_functions`) and
the inaccurate proj4rs comment at `ext/geo/core/Cargo.toml:12-13`.

### 6. Bindings take the api-only ext core

For each of clm, filelist, geo × python, napi, wasm: depend on the ext core
with `default-features = false`.

- Python re-exports of core-dependent items go
  (clm `python/src/lib.rs:12-17`: `create_reference_handler`,
  `ReferenceTypeHandler`; filelist `python/src/lib.rs:7-11`:
  `create_filelist_handler`, `FileListTypeHandler`). Rust-only re-exports;
  confirm nothing Python-visible changes.
- filelist python: delete the dead `mutations` feature, optional core dep and
  core dev-dep (`Cargo.toml:14, 21, 25`) — no code uses them.
- clm python `mutations` feature, `mutations.rs`, and the Python
  reimplementation (`alizarin_clm/__init__.py:316-392`) are replaced by step
  6b.
- geo wasm: drop the `uuid`/`getrandom` wasm features that exist only for core
  (`ext/geo/wasm/Cargo.toml`).
- G2: `ext/geo/napi/register.js` also calls the host's derive-provider
  registration with `binding.utmDeriveProviderPtr()`. Test in
  `tests/dual-backend/utm.test.ts` (napi leg) requiring only `@alizarin/geo-napi`.
- C2: clm napi exports `setClmBaseUri`/`getClmBaseUri` (as Python does); test
  in the clm napi tests.
- G3: check; fix if real.

Each `ext/*/*/Cargo.lock` regenerates (they are separate lockfiles).

### 6b. Extension mutations on every binding (D3)

Today `clm.reference_change_collection` exists three times: Rust
(`ext/clm/python/src/mutations.rs`, shipped nowhere), Python (the callable in
`alizarin_clm/__init__.py`, the only one that runs), and not at all on WASM
or NAPI; NAPI has no extension-mutation API (`binding-parity.json`:
`apply_mutations_with_extensions`, `register_extension_mutation`,
`get_registered_extension_mutations` are `gap: … (A-6, G)` for napi).

- **One implementation**, in clm-core on the api crate only: a JSON → JSON
  transform (find node by id or alias, check it is `reference`/
  `reference-list`, set `config[config_key]`), as the Python version already
  is — it needs no `StaticGraph`. Exported as a `MutationHandlerInfo` (the
  existing, unused ABI in extension-api, `lib.rs:1107-1250`), plus a plain
  fn for WASM.
- **Host side, in core** (`extension_abi`): `register_mutation_handler_from_ptr`
  → an `ExtensionMutationHandler` adaptor over the fn pointers, fingerprint
  checked, into a core-owned extension-mutation registry. Python's registry
  (`graph_mutator_py.rs:22`) moves to that core one, so all three bindings
  register into, list and apply from the same place.
- **Bindings, one canonical name each** (no new manifest names):
  - Python `register_extension_mutation` also accepts a capsule (as
    `register_extension_handler` does); the callable form stays for
    user-written mutations.
  - NAPI gains `registerExtensionMutation` (a bigint pointer),
    `applyMutationsWithExtensions`, `getRegisteredExtensionMutations` —
    closing those three A-6 gaps.
  - WASM keeps `registerExtensionMutation(name, fn)`; clm's wasm module
    exports the transform and clm's JS registers it on import.
- Delete `mutations.rs`, the clm python `mutations` feature and dev-dep, and
  the Python reimplementation; `alizarin_clm` registers the capsule on import.
- The mutation's conformance comes from the info struct, so it is the same
  everywhere.

Check before starting: whether ABI 7's fingerprint should cover
`MutationHandlerInfo`'s size (add `mutation_info_size` to `AbiFingerprint`
— it is the one handler struct not yet in it), and whether WASM's
`registerExtensionMutation` already routes into core's registry or keeps its
own.

### 7. Parity checks

The aim: every class of gap found in this programme fails a test, using the
machinery that exists, with no new framework.

1. **API manifest (existing).** The NAPI additions in 6b go into
   `tests/binding-parity.json` by deleting their `gap:` REASONS rules and
   regenerating with `binding_manifest.py`; `row_sink`'s Python-only entry
   stays. Nothing new.
2. **What importing an extension registers (new, one fixture).**
   `tests/data/extension-registrations.json`:
   ```json
   { "clm":      { "handlers": ["reference"], "mutations": ["clm.reference_change_collection"], "derive": [] },
     "filelist": { "handlers": ["file-list"], "mutations": [], "derive": [] },
     "geo":      { "handlers": ["geojson-feature-collection"], "mutations": [], "derive": ["utm"] } }
   ```
   (exact handler lists filled in from the current registrations). One test
   per side — a case in `tests/dual-backend/extensions.test.ts` (WASM and
   NAPI) and in `crates/alizarin-python/tests/test_extensions.py` — imports
   each extension **the way a user does**, then reads the host's
   `getRegisteredExtensionHandlers`, `getRegisteredExtensionMutations` and
   `hasDeriveProvider` and compares with the fixture. This is what would have
   caught G2 and the clm-mutation gap. The structural alternative, a manifest
   of each extension's native exports, isn't worth it: they differ by design
   (pointers on Python/NAPI, functions on WASM), so it would be mostly
   `ecosystem:` nulls.
3. **Register through the real entry points.** The dual-backend tests now
   re-implement each napi package's `register.js` ("as register.js does");
   that is why G2 passed. They `require` the package's `register.js` instead
   — less test code, not more.
4. **Behaviour cases, added to the existing paired tests** (same inputs and
   expected values in the TS and Python files, as those files already do):
   UTM into a `string` node under host language `ga` (G1); a stored concept
   URI through `resolve_markers` (C1); clm base URI set then rendered (C2);
   `clm.reference_change_collection` applied (6b).
5. **No second core (new guard).** `scripts/guards/no_second_core.py`, beside
   `test_integrity.py`: for each `ext/*/{python,napi,wasm}/Cargo.toml`,
   `cargo tree -e normal --target all --manifest-path …` must not list
   `alizarin-core`. Run in CI where the other guards run. Includes pg python,
   locking in G-4. Its expected set is "every binding crate, no exceptions",
   so it has no allow-list.

### 8. Tests, docs, register

New tests (each must fail before its fix), besides step 7's:

- G1 also as a hemisphere output on a `string` node, which nothing covers
  today (`tests/dual-backend/utm.test.ts`, `crates/alizarin-python/tests/test_utm.py`).
- `coerce_derived_tiles` idempotence, per datatype the derive tests use
  (core unit tests) — a precondition for step 4.
- ABI 7 fingerprint refusal (existing pattern in `derive_abi` tests), for
  derive and mutation infos.
- clm mutation unit tests move from `mutations.rs` to clm-core, on the JSON
  transform.

Full runs, all three backends plus every ext suite (commands as in
CLAUDE.md; ext: clm py 81, filelist py 27, pg py 5, geo py/napi/wasm,
`cargo test` in each ext core with and without `--no-default-features`).
Guards: test_integrity, fmt check, clippy `-D warnings` on the workspace and on
each ext core in both feature states.

Docs:

- `docs/extension-abi.mdx`: ABI 7, the interface crate's contents, "an
  extension links no core", the raw-output derive contract, `ExtDeriveProvider`
  replacing the `alizarin_core::derive_abi` imports in the provider example.
- `docs/extension-abi.mdx` also: a "Mutations" section (the info struct, host
  registration on each binding), replacing any Python-callable-only wording.
- `PARITY-RESEARCH.md`: new entries G1, G2, C1, C2 (resolved), C3, G3
  (open / resolved as found); the "second core" item as resolved; A-6's
  extension-mutation family resolved.
- `notes/backend-support-matrix.md`: clm napi base-URI row; geo-napi UTM row.
- `CHANGELOG.md`: fixes (G1, G2, C1, C2), **Breaking** for extension authors
  (ABI 7; `derive_abi::derive_with`/`derive_provider_info` move to the api
  crate as `ExtDeriveProvider`-based; `DeriveConfig::resolve_nodes` →
  `resolve_derive_nodes`; `StaticTile::ensure_id` removed; ext cores' in-process
  API behind `core`).
- Memory: update `base-crates-no-ext-deps` with the invariant.

### 9. Publishing

No workflow change needed: `release.yml` already publishes extension-api,
then core, then the ext cores (matrix, `needs: publish-crates-core`), and
`sync-version.sh` pins every path dep (dev-deps included) exactly. The ext
cores' dev-dependency on core keeps that ordering meaningful; leave it.
Release as the next alpha.

## Execution

One commit per step, each green on everything it touches before the next:

1. host C1 fix
2. api handler data types (+ core re-exports)
3. api derive interface (+ core adaptors, `resolve_derive_nodes`)
4. host coercion of derived outputs, ABI 7 (fixes G1)
5. ext cores feature-gated; geo on the api derive interface
6. bindings `default-features = false`; G2, C2, G3; dead features removed
   — with step 7.3 (tests register through the real entry points) first, so
   G2 is seen failing
6b. extension mutations on every binding; one clm implementation
7. parity checks: registrations fixture, behaviour cases, no-second-core
   guard + CI (each lands with the step whose gap it covers where it can, and
   is seen failing first)
8. docs, register, changelog (may fold into the commits above where it
   documents that commit's change)

Constraints as standing: source edits via the Edit tool only; no `cargo fmt`
writes; one-line conventional commit messages, no trailers; no allow-list
entries; push to `release/2.0` when green.

## Decisions (agreed 2026-11-25)

- **D1 — the host coerces derived outputs** (step 4), with its own language
  and extension registry; providers across a boundary emit raw values. Fixes
  the class, not just geo's language. Precondition: the idempotence test, as
  JS providers' already-coerced output is coerced again. Note: the on-load
  path skips a provider that errors (`instance_wrapper_core.rs:859`), so a
  host-side coercion error there drops the derived tile silently, as any
  provider error does now; unchanged here.
- **D2 — ABI 7.** The derive contract changes, and the fingerprint is what
  refuses a new geo under an old host.
- **D3 — consistency over simplicity:** one clm mutation implementation, on
  every binding through the C ABI and WASM (step 6b), closing NAPI's
  extension-mutation gaps; the Rust and Python duplicates go.
- **D4 — move the plain graph structs, not `GraphLookup`.** Every
  cross-boundary extension point carries JSON (that is forced: the two sides
  share no layout), so "typed access" in an extension means structs to
  deserialise into — those move, with their pure methods. `GraphLookup`
  describes a fully loaded graph, which only the host has; an extension asks
  a few questions over a callback, and a new question is a new lookup kind.
- **C3** stays out of scope (needs its own ABI addition).
