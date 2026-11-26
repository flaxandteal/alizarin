# ORM traversal: child-node maps and where the rest of the time goes

Status: **step 1 done**; the rest planned.

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
5. **~91% of the traversal is elsewhere**: ~3.3 µs per value. Not profiled:
   `perf` needs `kernel.perf_event_paranoid` ≤ 1 here.

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
2. **Look the child up, don't scan**: `get_semantic_child_value` uses
   `child_nodes.get(alias)`.
3. **Child maps computed once per graph node and shared** (`Arc`), built with
   the model's indices: removes the per-call build and the quadratic
   rebuilding (~90 µs → ~8 µs per resource here). Where: the model's existing
   lazily built indices (`GraphModelAccess`/`StaticGraph`), invalidated with
   them.
4. **Profile the other ~91%** with `perf` (after
   `sudo sysctl kernel.perf_event_paranoid=1`), on the benchmark, and plan
   from what it shows.
5. **Sorted key lists, and a guard**: WASM's `getRegisteredExtensionHandlers`
   sorted (as its mutation listing now is); a `sourceParity` rule flagging
   `.keys()` on a `HashMap`-typed field in a binding crate unless it goes
   through `sorted_keys` (string-level, so a ratchet, not a proof).
6. **Re-run the benchmark** after each of 1–3, and record the numbers here.
