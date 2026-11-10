# Scope: Aggregate functions, and spans as one use of them

Status: core built (Aggregate kind, the built-in fold provider, fold removal); bindings and rollups to
come. Builds on the fold primitive (`notes/fold-scope.md`, phase 1 shipped in alpha.145). Supersedes the
"consumer (RM) writes span tiles" plan in `AGGREGATE-FUNCTIONS-HANDOFF.md` and the Rós Madair a144
handoff: **RM does not compute spans.**

Origin: Road B from the RM-side design review. A Layer carries config naming, in dot notation, the nodes
of each graph whose values make up its spans, and a function on the Layer fills them in.

## The shape

| Layer | Derive (exists) | Aggregate |
|---|---|---|
| **Function kind**: a contract and hook in core | a resource's own tiles in → tiles for one of its nodegroups | a **slice of other resources** in → **node values** on the calling resource |
| **Provider**: implements the kind, registered by UUID | UTM (`ext/geo`), Gramadán | the built-in **fold** provider (core); others as needed |
| **Logic it uses** | its own code | **fold**: the datatype handlers' `fold_identity` / `fold` / `fold_merge` |
| **Per-use config** in `functions_x_graphs` | on the resource's graph | on the calling resource's graph (e.g. the **Layer** graph) |

Core knows only "Aggregate": read a slice, compute node values for named tiles on yourself, purely and
deterministically. **Nothing in alizarin knows what a span is.** A span is a tile on a Layer resource
that the Layer's owner writes (role, labels, anything else), plus node values in it that an Aggregate
computes.

## 1. Core: the Aggregate kind (built)

```rust
pub trait AggregateProvider: Send + Sync {
    fn aggregate(&self, target_id: &str, graph: &dyn GraphLookup, slice: &dyn SliceSource,
                 config: &AggregateConfig, context: &AggregateContext<'_>)
        -> Result<Vec<NodeValueWrite>, String>;
}
pub struct NodeValueWrite { pub tileid: String, pub node_id: String, pub value: Value }
```

- **`AggregateConfig`**: a `functions_x_graphs` entry with `AGGREGATE_FUNCTION_ID`
  (`60000000-…-0003`): `{ provider, config: <provider-specific JSON> }`.
- **`AggregateContext`** (non-exhaustive, built with `new`): the datatype handlers for fold dispatch, and
  a `GraphSource` for the members' graphs.
- **`SliceSource`**: a visitor, not random access. Implemented for a loaded `StaticResourceRegistry`, a
  `Vec<StaticResource>`, and `BusinessDataFileSlice` (streamed one resource at a time; native only), so
  large layers never need to be held in memory.
- **`apply_aggregate_functions(tiles, graph, target_id, slice, functions, context)`**, an explicit call,
  never run on load. It writes node values into the calling resource's tiles:
  - **The tile exists:** its node value is set; everything else in it is untouched. The tile must be in
    the node's nodegroup and belong to the target.
  - **The tile doesn't exist:** it's created with that id, the node's nodegroup, and just that value.
    Only for a top-level nodegroup: a nested one needs a parent tile, which only its owner can supply.
  - The calling resource is left out of its own slice; re-runs overwrite the same node values
    (idempotent); everything is checked before any tile changes (all-or-nothing); two writes to one node
    of one tile is an error.
- **Tile ids are the owner's.** The config supplies them, so they're stable by definition; the owner can
  mint them with `multi_tile_id` and its layer namespace.

## 2. Core: the built-in fold provider (built)

`FoldAggregate`, registered as `FOLD_AGGREGATE_PROVIDER_ID` (`60000000-…-0004`) by
`default_functions_registry()`. It's in core rather than an extension because it depends only on fold
dispatch: any datatype whose handler is registered folds through it (geometry with `alizarin-geo`).

```jsonc
{ "items": [
  { "tileid": "<tile on the calling resource>",
    "graph": "<member graph id>",
    "path": "<dot path to a value node in that graph>",
    "where": { "path": "<dot path>", "allowed": ["<value>", …] },   // optional
    "group_by": "<dot path to a concept, domain or boolean node>",  // optional
    "into": { "<node alias on the calling graph>": "summary" | "summary_text" | "values"
                                                   | "resources" | "skipped" } } ] }
```

- **Outputs:**
  - `summary`: the datatype's fold summary, null when nothing was folded;
  - `summary_text`: the same as a JSON string, for a string node;
  - `values`: values folded (one resource can contribute several on a cardinality-n node);
  - `resources`: members that contributed a value ("how many listed buildings");
  - `skipped`: values the datatype skipped (only unorderable `edtf`).
- **`where`**: set membership with the semantics of `PermissionRule::Conditional`, applied across the
  member's tiles: a member passes if any tile has a value at the path in `allowed` (a string, a boolean
  as `"true"`/`"false"`, or any element of an array). Ranges stay out of config; they'd be a provider.
- **`group_by`**: one summary per value of a fixed-vocabulary node (concept, domain, boolean), and every
  output becomes an object keyed by value, in **one node value**, so the owner needn't know the
  categories in advance. A member with several values counts towards each; one with none is left out.
- **Checked before folding:** unknown fields, unknown outputs, unresolvable paths, datatypes with no
  fold (e.g. geometry without `alizarin-geo` registered), non-groupable `group_by`, empty `allowed`,
  `into` aliases missing from the calling graph, and empty `into`.
- **One pass:** the slice is visited once for all items.
- **No literals, no template:** the role, datatype label and anything else in a span tile are the
  owner's data, written with the same tile id.

## 3. What a span is, then

The Layer model has a spans nodegroup, and the Layer's owner writes a span tile with its role (a concept
from whatever taxonomy the owner uses), labels and so on, and gives the fold provider that tile's id and
the node(s) to fill. **The role taxonomy is entirely the owner's choice; alizarin never reads it.** A
suggested starter set, anchored where standards exist: `display-extent` (GeoSPARQL), `valid-time` and
`transaction-time` (OWL-Time / bitemporal), `forecast-series` (instants relative to a declared "now"),
and domain events such as `construction-time`. A "declared" span (e.g. a forecast frame) is a span tile
the owner writes in full, with no Aggregate involved.

## 4. Rollups across layers (to come)

A corpus or catalogue resource whose slice is its member **Layer resources**: an Aggregate provider that
`fold_merge`s their span values (filtered as the owner configures) into its own node values. Merged
rollups are **inclusive**: a superset when upper layers move or delete values. Right for zoom-to-fit,
discovery and pruning; `values` counts aren't additive across layers. An exact composed-view rollup
would be a separate provider.

## Decisions

1. **Config maps; it doesn't compute.** No expression language: named outputs into named nodes,
   set-membership `where`, fixed-vocabulary `group_by`. Logic lives in providers.
2. **Aggregates write node values, not tiles.** The owner owns tiles, ids and every other node value.
3. **Folds depend on the fold capability, not on geo.** Geo registers its fold on its own handler.
4. **Built at build time on native hosts** (Rust, napi, Python). Function providers have **no C ABI**:
   they must be compiled into the same binary as core. In wasm an extension module is separate, so a
   provider there can't register, and every fold call would cross into JS with JSON. The browser reads
   span values and at most merges a handful.
5. **RM is a consumer.** It indexes the Layer resource like any other. A summary node should not be a
   geometry datatype, so RM's spatial index doesn't treat an extent as content.
6. **Membership by default is the loaded layer** (registry or business-data file) minus the calling
   resource; a caller-supplied slice narrows it.

## Incremental updates

Additions are free: fold the new values and `fold_merge` them in. Removal needs care, because min/max
and bbox have no inverse.

**Built: removal with a loose flag** (`datatype_fold::fold_remove`, the handler's optional `fold_remove`):

- Removing a value that isn't at an extreme changes nothing; the summary stays exact.
- Removing one that touches an extreme keeps the bound (a superset) and marks the summary
  `"loose": true` until a re-fold.
- No counts are kept at the extremes: ties are rare and a miss only costs a loose flag. (Considered and
  rejected as overkill: `(value, n)` per extreme.)
- `count` is invertible; removing every value gives the exact empty summary.
- `geometry_types` is **inclusive**: never shrunk by removal, never makes a summary loose.
- **The flag lives in the summary and core owns it:** `datatype_fold` strips it before calling a handler
  and sets it on the result, so handlers never see it. Folding into, or merging with, a loose summary
  gives a loose summary, so a rollup is loose if any input is. Absent when exact. "Does this extent need
  checking?" is `is_loose(summary)`.
- A handler without removal returns `None` and the caller re-folds. Only object summaries can carry the
  flag; `loose` is a reserved summary key.

**Still parked:** an optional `AggregateProvider::update(previous, delta /* added, changed, removed with
old values */, …) -> Option<…>` defaulting to `None` (the caller runs a full `aggregate`), and any trigger
that calls it. It's a trait default method and providers have no C ABI, so adding it later breaks
nothing.

## Phases

1. ~~Core: the Aggregate kind, slices, `apply_aggregate_functions`.~~ Built.
2. ~~The fold provider (`where`, `group_by`, outputs, config checks).~~ Built, in core.
3. **Bindings:** napi and Python `applyAggregateFunctions` over a loaded registry or a business-data file
   path, with the default functions registry plus any registered handlers; a test that both give
   identical node values.
4. **Rollup provider** over member Layer resources, and `fold_merge_all`.
5. **Later:** an `on-member-change` trigger with the `update` hook (parked above), exact composed-view
   rollups, richer summaries (hull, cell-set, EDTF granularity).

## Open questions

- **Phase 2 of `fold-scope.md`** (`foldResources` on wasm/napi/Python) is no longer on the critical path:
  Aggregate's bindings replace it as the entry point. Keep a thin `foldResources` for ad-hoc use, or drop
  it?
- **A starter role taxonomy:** worth shipping as example reference data in the docs, or left entirely to
  Layer owners?
