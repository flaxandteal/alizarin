# ORM traversal: child-node maps and where the rest of the time goes

Status: **steps 1, 2, 3, 4 and 7 done**: a full traversal ~4× faster
(~1 ms → ~0.25 ms per resource). Planned: 8, then 5, then re-profile.

## What was measured

`crates/alizarin-core/tests/child_nodes_bench.rs` (ignored by default):

    cargo test --release -p alizarin-core --test child_nodes_bench -- --ignored --nocapture

A full ORM traversal, as every binding's ORM does it (from the root, each
node's semantic children through `get_all_semantic_child_values`, recursing
into semantic nodes), over the largest model in `tests/data` (Heritage Item:
439 nodes, 39 nodegroups), one resource with a tile in every nodegroup.
Two runs, 2026-11-25:

| | per resource / per call |
|---|---|
| full traversal | 1.05–1.08 ms (292 values, 381 child-map builds) |
| child map, `HashMap` build (today) | 237–241 ns |
| child map, `BTreeMap` build | 183–185 ns |
| child map, cached `Arc` | 21–22 ns |
| child-map builds' share of a traversal | ~91 µs (8–9%) |

Per-call figures are over the 218 nodes with children, the ones a traversal
builds maps for. At a million resources of this model: a traversal of ~18
minutes, ~90 s of it child maps.

## Findings

1. **`ModelAccess::get_child_nodes` builds a fresh map on every call**
   (`instance_wrapper_core.rs:190`): walks the node's edges, clones each alias
   `String`, inserts. It runs whenever a pseudo node is made.
2. **It's called quadratically.** `get_all_semantic_child_values` builds the
   map, then calls `get_semantic_child_value` per child, each of which builds
   it again, and then finds the child with a linear `.values().find(…)` rather
   than a lookup (`instance_wrapper_core.rs:1803`). 381 builds for 292 values.
3. **A `BTreeMap` costs about the same as the `HashMap` here.** The first
   run's "23% cheaper" was wrong: it timed core's `get_child_nodes` (which
   also cloned the edge list, a `Vec<String>`, on every call) against the
   benchmark's own build into a `BTreeMap`. Like for like, the same build into
   each, three runs: `HashMap` 170 ns, `BTreeMap` 180–183 ns (~6% more,
   ~4 µs of a ~950 µs traversal). The maps are small (one node's children;
   mean 2, max 22 in Heritage Item). It makes every key list sorted by
   construction. The edge-list clone was ~40 ns a call, more than the map
   type; it is gone (step 1).
4. **Key lists reach users unsorted today**, against the output-parity rule:
   `childNodeAliases` on WASM, NAPI and Python, and NAPI's
   `getChildNodeAliases` (all from these `HashMap`s), and WASM's
   `getRegisteredExtensionHandlers` (a `HashMap` in a thread-local). Nothing
   lints for it: `tests/sourceParity.test.ts` rule 5 covers serialised
   `HashMap` fields only.
5. **~91% of the traversal is elsewhere**: ~3.3 µs per value. Profiled (step
   4, below): it is mostly copying.

## What stays a `HashMap`

Large maps: the graph registry, per-graph node indexes (by id, by alias), the
resource registry. At a million UUID keys a `BTreeMap` lookup is ~20
comparisons over 6–8 tree nodes, likely cache misses each, against one hash
and one or two misses: an estimated 2–4× slower on the hottest maps. Sort
those only when listing (`sorted_serde::sorted_keys`), and paginate large
listings.

Tile `data` is already a `BTreeMap` (for deterministic serialisation). A
`BTreeMap` with any entry allocates a full 11-slot leaf (~630 bytes for
`String` → `Value`), against ~250 for a 3-entry `HashMap`. Fine while tiles
load lazily per resource; if resident tile memory ever matters, a sorted
`Vec<(String, Value)>` gives the same order at ~56 bytes an entry. Measure
first.

## Steps

1. **Done: `BTreeMap` for child-node maps**, as core's `ChildNodes` alias
   (`PseudoNodeCore.child_nodes`, `ModelAccess::get_child_nodes`,
   `GraphModelAccess::get_child_nodes`, NAPI's `getChildNodes`), and
   `get_child_nodes` borrows the edge list rather than cloning it. The
   bindings' `childNodeAliases` and `getChildNodeAliases` are sorted on all
   three; `resourceModelWrapper.test.ts` (WASM, NAPI) and
   `test_node_view_model.py` check the order. Measured after, three runs:
   `get_child_nodes` 184–202 ns, the same as the plain `BTreeMap` build;
   traversal 0.92–1.04 ms (noise between runs here is ~10%).
2. **Done: look the child up, don't scan**: `get_semantic_child_value` uses
   `child_nodes.get(alias)`.
3. **Done: child maps computed once per graph node and shared.**
   `ModelAccess::get_child_nodes` returns `Arc<ChildNodes>` (the default
   still builds one per call); `GraphModelAccess` builds every node's in
   `build_indices`, from the same nodes and edges indices the default reads,
   and `invalidate_caches` drops them with those. Every way its graph
   changes (`set_graph`, `set_graph_nodes`/`_edges`/`_nodegroups`,
   `prune_graph`) goes through `invalidate_caches`, so the maps are exactly
   as fresh as the indices they were derived from per call before;
   `get_child_nodes_follows_a_changed_graph` checks it. A map already handed
   out keeps the graph it was taken from, as any `Arc` of the old indices
   does. `LayeredGraph` is not involved: it implements `GraphLookup`, not
   `ModelAccess`, has its own `OnceLock` indices, and a layer is added or
   removed by making a new one (`with_overlay`, `without_overlay`), never by
   changing one. WASM's `ResourceModelWrapperCore` delegates to its
   `GraphModelAccess`. `PseudoNodeCore.child_nodes` is the shared `Arc`.
   Measured after, three runs: `get_child_nodes` 24–25 ns (from ~200);
   traversal **0.25–0.26 ms** per resource (from 0.45–0.51).
4. **Done: profile.** `BENCH_RESOURCES=3000`, built with
   `RUSTFLAGS="-C force-frame-pointers=yes"` (DWARF unwinding lost most
   callers), `perf record -e cycles:u --call-graph fp`, 31,640 samples;
   callees aggregated from `perf script` (frame below the function). Shares
   of all samples:

   | | |
   |---|---|
   | `get_all_semantic_child_values` (inclusive) | 71% |
   | → `get_semantic_child_value` (inclusive) | 56% |
   | → → **deep copy of each matching tile** (`Arc::new(tile.clone())`, its `data` `BTreeMap` cloned) | **~23%** |
   | → → `get_child_nodes` | 12% (16% with its other caller) |
   | → → `semantic_child_tile_ids` | 6% |
   | → → dropping the copies | 4% |
   | the benchmark's own walk (outside core) | ~20% |
   | flat: `malloc`/`free` and friends | ~40% |

   `get_semantic_child_value` (`instance_wrapper_core.rs:1834-1839`) hands
   each pseudo value `Arc::new(tile.clone())`: the store holds tiles by value
   (`TileStore = IndexMap<String, StaticTile>`), so every read of a child
   copies every matching tile, all its nodes' values with it, and a tile in
   a nodegroup of k nodes is copied k times per traversal. Beside it, the
   child's edge list is cloned per call (`:1849`) and again per value
   (`:1864`).
7. **Done: tiles shared, not copied.** `TileStore = IndexMap<String,
   Arc<StaticTile>>`: a pseudo value takes `Arc::clone`, as do
   `values_from_resource_nodegroup` and `make_pseudo_value`, and a
   `ResourceEdit`'s all-or-nothing snapshot is now a copy of `Arc`s. An edit
   can't write through an `Arc`, so the one in-place write (`ResourceEdit::set_value`)
   copies the tile, changes it, and stores the new one, as before; the
   compiler found every site (about 20, in `instance_wrapper_core.rs`,
   `resource_edit.rs` and WASM's `getTile`). APIs returning owned tiles
   (`resolve_and_filter_tiles`, descriptors, `derive_on_load`'s provider
   input) copy out, as they did. Measured after, three runs: traversal
   **0.45–0.51 ms** per resource, from 0.92–1.08 ms: about 2×, more than the
   copy's 23% share, since the allocation churn of making and freeing the
   copies went with it. All suites unchanged.
8. **No per-value edge-list copies**: `PseudoValueCore`'s child node ids as
   a shared slice (`Arc<[String]>`, or the model's own, borrowed).

   Order by measured size: 7, then 3 (with 2), then 8. Re-run the benchmark,
   and the profile, after each.
5. **Sorted key lists, and a guard**: WASM's `getRegisteredExtensionHandlers`
   sorted (as its mutation listing now is); a `sourceParity` rule flagging
   `.keys()` on a `HashMap`-typed field in a binding crate unless it goes
   through `sorted_keys` (string-level, so a ratchet, not a proof).
6. **Re-run the benchmark** after each of 1–3, and record the numbers here.
