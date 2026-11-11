# Scope: `fold` — reducing a datatype over a set of resources

Status: phase 1 built (core + `ext/geo`, Rust only). Supersedes the alizarin
half of `AGGREGATE-FUNCTIONS-HANDOFF.md` and `LAYER-SPANS-HANDOFF.md`.

## What it is

A generic functional primitive: walk a set of resources, and reduce every value
at a path into one summary using that value's datatype. Spans (RM/Layer) are one
consumer; facets, zoom-to-fit, descriptors and validation are others. alizarin
never names spans, roles or taxonomies.

It follows the indexing pattern (`datatype_index_spec`):

| | Indexing | Fold |
|---|---|---|
| Handler declares | how a value indexes (`IndexClass` + keys) | how a datatype summarises (identity, fold, merge) |
| alizarin owns | registry-first dispatch + core fallbacks | the same, plus a thin backend-agnostic runner |
| Consumer owns | grouping, layout, storage | meaning (role), storage (e.g. a span nodegroup) |

## Components

### 1. Capability on `ExtensionTypeHandler` (core, Rust trait)

Three optional methods, a monoid so results don't depend on visit order:

```rust
fn fold_identity(&self, config: Option<&Value>) -> Option<Value>; // None: can't fold
fn fold(&self, acc: Value, values: &[&Value], config: Option<&Value>) -> Result<Value, ExtensionError>;
fn fold_merge(&self, a: Value, b: Value, config: Option<&Value>) -> Result<Value, ExtensionError>;
```

- `fold` takes a batch so the C ABI (phase 2) isn't crossed once per value.
- The summary is JSON (`serde_json::Value`, object keys sorted), so it can cross
  the ABI and serialise identically on every backend.
- No `can_fold` flag: `fold_identity` returning `Some` is the opt-in (adding a
  `HandlerCapabilities` field would touch ~30 struct literals across crates).

### 2. Dispatch + core fallbacks (`datatype_fold.rs`, beside `datatype_index.rs`)

Registry-first, then built-ins for core-owned datatypes only:

- `number` → `{ "min", "max", "count" }`
- `date` → `{ "min", "max", "count" }` as raw strings, ordered with
  `quantize::quantize_date` (the same truth the index uses).
- `edtf` → the same shape, but **needs an EDTF parser core does not have**:
  `quantize_date` reads only `YYYY[-MM[-DD]]`, and EDTF coercion accepts any
  string (`type_coercion/scalars.rs`). With "unparseable fails the fold", real
  EDTF (`1850/1860`, `1850?`, `185X`, `../1860`) would fail. Each value must
  give a lower and an upper bound, precision-aware (the upper bound of `1860` is
  1860-12-31, not 1860-01-01). Prefer a tested crate to a hand-rolled parser;
  it must be pure Rust (WASM). Open ends (`..`) contribute nothing on that side.
  The index's `edtf` quantizing has the same gap and should share the parser.
  **Until then (phase 1):** EDTF values `quantize_date` can't order are skipped
  quietly, not an error; `FoldResult.skipped` counts them so the gap is visible.
  This is the one exception to "a bad value fails the fold".
- Everything else → no summary unless a handler claims it.

### 3. Geometry in `ext/geo`

`GeoTypeHandler` (today validate-only) gains the fold capability:
`{ "bbox": [min_lng, min_lat, max_lng, max_lat], "geometry_types": [...], "count" }`.
Core stays GeoJSON-free (`datatype_index.rs` keeps it WASM-clean on purpose).
Consumers wanting spatial summaries register `alizarin_geo_core::register`.
Hull / Morton cell-set come later, in the same place.

### 4. Runner (`fold.rs`, core)

```rust
pub struct FoldResult {
    pub path: String,
    pub datatype: String,
    pub summary: Option<Value>, // None when no values were folded
    pub values: usize,          // values folded
    pub skipped: usize,         // EDTF values the phase-1 parser couldn't order
}

pub fn fold_resources(
    tiles: &dyn TileSource,
    resource_ids: &[String],          // injected slice; alizarin doesn't own membership
    model: &dyn ModelAccess,          // one model per call
    paths: &[String],
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<Vec<FoldResult>, FoldError>;
```

- Resolves each `path` with `path_resolution::resolve_path_segments`, so it knows
  the node and nodegroup; asks `TileSource` once per resource per nodegroup,
  however many paths share it.
- Resources of another model have no tiles in its nodegroups and contribute
  nothing, so a multi-model layer gets a per-model breakdown by one call per
  model, and a union by `fold_merge` over those results.
- Returns summaries. It does **not** write tiles: the target nodegroup is the
  consumer's schema. Consumers mint result tiles with `multi_tile_id` + their
  layer namespace.

### 5. `impl TileSource for StaticResourceRegistry`

Nothing implements `TileSource` today. This lets every binding fold over a loaded
registry (BRACE's case) without a native tile source.

### 6. Bindings (phase 2)

- **C ABI (built, `ABI_VERSION` 4):** `fold_identity_fn`, `fold_fn`,
  `fold_merge_fn`, `fold_remove_fn` and one `free_fold_fn` on `TypeHandlerInfo`.
  Values cross by pointer (an array of `*const Value`), never copied. ABI 3
  shipped in alpha.144/145, so this is a new version; the fingerprint check
  refuses mismatched extensions. Geo implements it.
- **WASM (built):** JS handlers register `foldIdentity` / `fold` / `foldMerge` /
  `foldRemove` callbacks (batched JSON); geo's wasm build exports and registers
  them.
- `foldResources(registry, graph(s), resourceIds, specs)` on WASM, NAPI, Python:
  **not built**. Aggregate functions are the entry point
  (`applyAggregateFunctions`, see `notes/spans-scope.md`).
- **WASM and NAPI must be equivalent:** same signature through `backend.ts`,
  byte-identical summaries for the same input. A `tests/dual-backend/` test
  folds the same registry on both and compares; Python is checked against the
  same expected summaries. The runner stays in core so there is one
  implementation behind all three.

## Phases

1. **Core + `ext/geo`, Rust only** (RM links both natively, so no ABI needed yet):
   capability trait, dispatch with number + date fallbacks, geo bbox fold, runner,
   registry `TileSource`, tests including order-independence (fold vs merge of
   partial folds give the same summary).
2. **C ABI + bindings** so JS (BRACE) and Python get it. The fold capability now crosses both the C
   ABI (ABI 4) and the wasm JS bridge. The entry point is Aggregate functions
   (`notes/spans-scope.md`); a thin `foldResources` binding is not built.
3. **More summaries:** EDTF granularity, hull / cell-set, others as consumers ask.
4. (Deferred, per the handoff) incremental recompute when a layer changes. Removal that compares
   against the extremes and marks the summary loose (`fold_remove`) is built; the provider-level
   `update` hook and any trigger are parked. See `notes/spans-scope.md`, "Incremental updates".

## Decisions

1. **Name:** `fold` (capability, runner) and "summary" (result).
2. **EDTF:** a value contributes both ends (`1850/1860` gives 1850 and 1860).
   Unparseable values are not dropped (see 5). Granularity: later.
3. **Empty input (whole fold, no values seen):** `summary: None` with
   `values: 0`, not the identity, so "no data" is distinguishable. A resource
   with no values just contributes nothing.
4. **Paths:** value nodes only.
5. **Errors:** a bad value fails the whole fold, with an error naming the
   resource, path and value. Exception: EDTF values the phase-1 parser can't
   order are skipped quietly and counted in `skipped`.

## Non-goals

Spans, roles, taxonomy, `declared` values, writing the Layer's nodegroup, and
deciding layer membership — all consumer-side.
