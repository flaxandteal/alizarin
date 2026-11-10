# Scope: Aggregate functions, and spans as one

Status: agreed design, no code yet. Builds on the fold primitive (`notes/fold-scope.md`, phase 1
shipped in alpha.145). Supersedes the "consumer (RM) writes span tiles" plan in
`AGGREGATE-FUNCTIONS-HANDOFF.md` and the Rós Madair a144 handoff: **RM does not compute spans.**

Origin: Road B from the RM-side design review. A Layer carries config naming, in dot notation, the
nodes of each graph whose values make up its spans. A span function, added as many times as needed,
fills a cardinality-n span nodegroup on the Layer resource, with each span typed by a role from an
external taxonomy.

## The shape

Three layers, mirroring Derive:

| Layer | Derive (exists) | Aggregate (new) |
|---|---|---|
| **Function kind**: a contract and hook in core | a resource's own tiles in → tiles for one of its nodegroups | a **slice of other resources** in → tiles for a nodegroup on the calling resource |
| **Provider**: implements the kind; in an extension; registered by UUID | UTM (`ext/geo`), Gramadán | **Span** (`ext/spans`); later a rollup provider |
| **Logic it uses** | its own code | **fold**: the datatype handlers' `fold_identity` / `fold` / `fold_merge` |
| **Per-use config** in `functions_x_graphs` | on the resource's graph | on the **Layer** graph |

Core knows only "Aggregate": read a slice, write tiles on yourself, purely and deterministically.
Core never names spans. Everything span-specific lives in the span provider.

## 1. Core: the Aggregate kind

```rust
pub trait AggregateProvider: Send + Sync {
    fn aggregate(
        &self,
        target_id: &str,                  // the calling resource (e.g. the Layer resource)
        graph: &dyn GraphLookup,          // the calling resource's graph
        slice: &dyn SliceSource,          // the members, visited once
        config: &AggregateConfig,         // this functions_x_graphs entry
        handlers: &ExtensionTypeRegistry, // fold dispatch
        models: &dyn ModelLookup,         // member graphs, to resolve paths per graph
    ) -> Result<Vec<StaticTile>, FunctionError>; // tiles for config.nodegroup
}

pub enum RegisteredFunction { Descriptor(..), Derive(..), Aggregate(Arc<dyn AggregateProvider>) }
```

- **`AggregateConfig`**: a `functions_x_graphs` entry with a new `AGGREGATE_FUNCTION_ID`, shaped like
  `ComputeTilesConfig`: `{ provider, nodegroup, config: <provider-specific JSON> }`.
- **`SliceSource`, not `TileSource`.** An aggregate visits every member once and never needs random
  access. A visitor (`fn visit(&self, f: &mut dyn FnMut(&str /*graph id*/, &str /*resource id*/,
  &[StaticTile]) -> Result<..>) -> Result<..>`) covers both a loaded `StaticResourceRegistry` and a
  **streamed** business-data file, so large layers never need to be held in memory. That was the reason
  RM's emitter folds while streaming.
- **The hook is an explicit call:**
  `apply_aggregate_functions(target_id, graph, slice, functions, handlers, models) -> Vec<StaticTile>`.
  It is not run on load: folding a whole layer whenever its resource opens would cost a full pass, and
  explicit calls also avoid the gap that alizarin's ORM bindings never wire derive-at-load.
- **Re-running replaces the output.** Unlike Derive, where attested data wins, an Aggregate owns its
  target nodegroup: re-running regenerates it, and stable ids keep the result identical when nothing
  changed.
- **Purity:** the same contract as Derive. The fold is order-independent, so the visit order of the
  slice doesn't matter.

## 2. Extension: the span provider (`ext/spans`)

Provider config is **declarative**: it selects and maps, and carries no logic.

```jsonc
{ "provider": "<span provider uuid>",
  "nodegroup": "<Layer spans nodegroup>",
  "config": {
    "items": [
      { "graph": "<Monument graph id>", "path": "geometry",
        "role": "<display-extent concept id>" },
      { "graph": "<Monument graph id>", "path": "construction.date",
        "role": "<construction-time concept id>" },
      { "role": "<valid-time concept id>", "declared": { "...": "a literal summary" } }
    ] } }
```

- **`extracted` items** (`graph` + `path`): resolve the path through that graph's schema **once** to a
  node and datatype, then fold that node's values for the graph's members, in batches, through the
  handler registry.
- **`where` (optional, on extracted items)**: a set-membership filter with the same semantics as
  `PermissionRule::Conditional` (`permissions.rs`): `{ "path": "designation.type", "allowed": [...] }`.
  It gives sub-extents and subcounts, e.g. the bbox and count of listed buildings, as their own span.
  It's declarative, evaluated in Rust with one hash lookup, and needs no expression language. Three
  adaptations from the permissions use:
  - **Resource scope:** the resource passes if **any** of its tiles has a value in the set. The
    condition usually sits in a different nodegroup from the folded value, and the slice visitor
    already gives the provider all of a resource's tiles.
  - **Alias paths**, resolved once at config check to node ids; then reuse `evaluate_tile_path`.
    Factor the evaluator out of `permissions.rs` to share it rather than copying it.
  - **Lists match on any element**, for `concept-list` / `reference` arrays.

  Set membership only: concepts, domain values, booleans and statuses. Ranges ("built after 1900") stay
  out of config; they become a predicate provider if needed.
- **`group_by` (optional, on extracted items)**: a concept, domain or boolean path. Produces one
  summary per value in a single pass (e.g. count and bbox per building type) instead of one `where`
  item per value. Still a monoid: the summary is a map of value → summary, and merging two maps merges
  each value's summary. Restricted to fixed-vocabulary datatypes so its size stays bounded.
- **`declared` items**: no path; the literal is written as given (e.g. a forecast's now-reference and
  offsets, which can't be derived from stored values).
- **`computed`**: not an item type. It is a different provider (Aggregate or Derive) in the same
  registry. Logic always lives in providers, never in config.
- **Each output tile** carries role, datatype, source, scope (graph, path, and any `where` /
  `group_by`), the summary, `values`, `skipped`, `resources`, and provenance.
  - `values` is the number of **values** folded (the summary's `count`); one resource can contribute
    several on a cardinality-n node.
  - `resources` is the number of member resources that contributed: the answer to "how many listed
    buildings". It's free from the slice visitor, and lives on the span tile, not in the datatype
    summary, so the fold summary shape doesn't change.
  - Its id is `multi_tile_id(layer namespace, layer resource id, spans nodegroup, k)`, so re-runs keep
    ids and other layers can address them. `k` must be stable: for a plain item it comes from the
    item's index; a `group_by` item writes one tile per value, so its `k` comes from (item index, group
    value), e.g. by deriving the id from the group value rather than its position, so a new category
    doesn't shift the others' ids.
  - The summary is stored in a **non-geometry datatype**, so content folds and RM's spatial index
    never mistake an extent for content.
- **The provider validates its config before folding:** every path resolves to a value node, every
  datatype has a fold, and every role is present. For example: "item 0: `geojson-feature-collection`
  has no fold; register `alizarin-geo`". So a geo-less setup fails at check time, not mid-run.
- **The Layer model**: `ext/spans` ships a reference spans branch (CSV), covering role, datatype,
  source, scope, value, counts and provenance, for Layer models to include.

## 3. Rollups across layers: a second Aggregate provider

A corpus or catalogue resource whose slice is its member **Layer resources**. It `fold_merge`s their
span tiles grouped by (datatype, role), optionally filtered by graph or path, and writes rollup spans.

- **"How big is this corpus's geography"**: merge every geometry `display-extent` span.
- **"Extent of Building"**: filter by graph (and path).
- **Inclusive semantics:** a merged rollup is a **superset** when upper layers move or delete values.
  That's right for zoom-to-fit, discovery and pruning. `values` counts are not additive across layers.
  An exact (composed-view) span is a later, separate provider.

## Decisions

1. **Declarative config, logic in providers.** No expression language. Filters, transforms and
   computed spans are named providers. Config is parsed and paths are resolved once per run; the
   per-tile work is a map lookup and a batched handler call.
2. **Spans depend on the fold capability, not on geo.** Geo registers its fold on its own handler, as
   indexing does. `ext/spans` offers a default-on `geo` feature, or a bundle, for convenience: no hard
   dependency.
3. **Built at build time on native hosts** (Rust, napi, Python). Function providers have **no C ABI**:
   they must be compiled into the same binary as core. In wasm, an extension module is separate, so a
   provider there can't register, and every fold call would cross into JS with JSON (the cost measured
   for CLM `renderDisplay`). The browser **reads** span tiles and at most merges a handful.
4. **RM is a consumer.** It indexes the Layer resource like any other; no span logic. Its uncommitted
   `GeoExtentFold` / `ModelManifest.spatial_extent` work can be dropped or kept as a cross-check.
5. **Membership by default is the loaded layer.** The slice is everything in the layer's registry or
   business data, minus the Layer resource itself. A caller-supplied id set narrows it.

## Bindings

- **Rust:** `apply_aggregate_functions` directly. RM, or a native build tool.
- **napi / Python:** `applyAggregateFunctions(layerResourceId, slice, …)` over a loaded registry, or a
  business-data file path that streams. The span and geo providers are compiled into the binding, as
  function providers must be.
- **wasm:** none for running Aggregates (decision 3). A merge helper over span summaries
  (`fold_merge_all`) is cheap enough to expose if a browser client needs a rollup.

## Phases

1. **Core:** `AggregateProvider`, the `Aggregate` variant, `AGGREGATE_FUNCTION_ID` + config,
   `SliceSource` (registry + streamed-file implementations), `apply_aggregate_functions`, and tests with
   a stub provider.
2. **`ext/spans`:** the span provider (extracted + declared), `where`, `resources` on span tiles,
   config validation, stable ids, the reference spans branch, and tests with geo + date over a fixture
   layer. `group_by` here or just after.
3. **Bindings:** napi and Python `applyAggregateFunctions`, plus a dual-backend test (napi vs Python
   give identical span tiles).
4. **Rollup provider** and `fold_merge_all`.
5. **Later:** an `on-member-change` trigger with the provider `update` hook (parked; see "Incremental
   updates"), exact
   composed-view spans, and richer summaries (hull, cell-set, EDTF granularity).

## Incremental updates

Additions are free: fold the new values and `fold_merge` them in. Removal needs care, because min/max
and bbox have no inverse.

**Built: removal with a loose flag** (`datatype_fold::fold_remove`, the handler's optional
`fold_remove`). It compares removed values against the extremes:

- Removing a value that isn't at an extreme changes nothing, and the summary stays exact.
- Removing one that touches an extreme keeps the bound (still a superset) and marks the summary
  `"loose": true` until a re-fold.
- No counts are kept at the extremes: ties at an extreme are rare, and a miss only costs a loose flag.
  (Considered and rejected as overkill: storing `(value, n)` per extreme to stay exact on ties.)
- `count` is invertible: subtract. Removing every value gives the exact empty summary.
- `geometry_types` is **inclusive**: the types seen among folded values. Removal doesn't shrink it and
  doesn't make the summary loose. (Every removal has a type, so treating it as exact would make every
  removal loose. Counts per type are the option if an exact type set is ever needed.)
- **The flag lives in the summary, and core owns it.** `datatype_fold` strips it before calling a
  handler and sets it on the result, so handlers never see it: folding into a loose summary, or merging
  with one, gives a loose summary, so a rollup across layers is loose if any input is. It's absent when
  exact, so exact summaries are unchanged. "Does this extent need checking?" is `is_loose(summary)`.
  Span tiles therefore need no separate `freshness` field.
- A handler that doesn't implement removal returns `None`, and the caller re-folds. Only object
  summaries can carry the flag; `loose` is a reserved summary key.

**Still parked:** the provider hook, an optional `AggregateProvider::update(previous, delta /* added,
changed, removed with old values */, …) -> Option<…>` defaulting to `None` (the caller runs a full
`aggregate`), and any trigger that calls it. It's a trait default method, and function providers have no
C ABI, so adding it later breaks nothing.

## Open questions

- **Role taxonomy:** which SKOS collection, and whether `ext/spans` ships a starter one.
- **Span provider UUID(s), and `AGGREGATE_FUNCTION_ID`:** fixed constants, as `COMPUTE_TILES_FUNCTION_ID`
  is.
- **Phase 2 of `fold-scope.md`** (`foldResources` on wasm/napi/Python) is no longer on the critical
  path: Aggregate's bindings replace it as the entry point. Keep a thin `foldResources` for ad-hoc use,
  or drop it?
