# Backend support matrix

Alizarin ships three bindings over one Rust core (`alizarin-core`). They are
**not** identical surfaces — each exposes the capabilities its deployment target
needs. This is a deliberate, supported contract as of 2.0, not partial work.

| Capability | WASM (browser) | Python (notebooks/ETL) | NAPI (Node ETL/builder) |
|---|:--:|:--:|:--:|
| Instance / viewmodel wrapper | ✓ | ✓ | ✓ |
| Resource registry (`getFull`/`getSummary`) | ✓ | ✓ | ✓ |
| Registry calls behind `StaticStore` (`getAllFullForGraph`/`mergeFromResources`/`insert`) | ✓ | ✓ | ✓ |
| Graph / prebuild export | ✓ | ✓ | ✓ |
| SKOS / RDM parse | ✓ | ✓ | ✓ |
| Extension coerce / render / resolve | ✓ | ✓ | ✓ |
| Layer composition (`composeResourceLayers`) | ✓ | ✓ | ✓ |
| Resource lifecycle state / transitions | ✓ | ✓ | ✓ |
| CSV → business_data (`buildResourcesFromBusinessCsv`) | ✓ | ✓ | ✓ |
| `batch_trees_to_tiles` / `tiles_to_trees` | ✓ | ✓ | — |
| `coerce*` value family (standalone) | ✓ | ✓ | — |
| Card-display serialization (standalone fn) | ✓ | ✓ | —¹ |

¹ NAPI renders displays through the **instance-wrapper** (`forJson` /
`forDisplayJson` on `NapiResourceInstanceWrapper`); it simply lacks the
*standalone* `serialize_card` entry point.

The WASM registry returns class instances and the NAPI registry returns plain
JSON objects, so `tilesLoaded`, `toMetadata()` and `fromSummary()` are not methods
you can call on NAPI results. Use `resourceTilesLoaded`, `summaryToMetadata` and
`createStaticResourceFromSummary` from `js/backend.ts`.

## Why the surfaces differ (by design)

- **WASM — browser.** Conversion, coercion, card display, viewmodels and CSV
  ingestion (from strings; no filesystem) are all present.
- **NAPI — Node ETL / builder.** Its consumer (e.g. starches-builder) ingests
  via **CSV → business_data** and edits through the instance-wrapper (the ORM's
  `$commit()` over core `ResourceEdit`), then exports. It does **not** call standalone
  `batch_trees_to_tiles`, `tiles_to_trees`, or per-value `coerce*` — verified: no
  NAPI consumer references them. So those are **intentionally not exposed**.
- **Python — notebooks / interactive ETL.** The fullest surface, because
  notebook users need every primitive directly.

## The shared-layer guarantee

The backend-agnostic TypeScript layer (`js/`) works on **both** WASM and NAPI. It
depends only on capabilities present in **all** backends it runs on — the
instance-wrapper, registry, export, and extension APIs (top four rows). It does
**not** call the conversion/coerce/card-display group, so NAPI's leaner surface
does not limit it. `tests/dual-backend/staticStoreRegistry.test.ts` runs the
`StaticStore` and `all()` → `getRootViewModel` → `toSnapshot` path on both.

## One output: what every backend writes

Surfaces differ; output does not. Wherever two backends both hand out a value,
they hand out the same JSON, byte for byte, because core's serde is the only
definition of it:

- **Core serialises; bindings don't.** No binding builds a `toJSON`/`to_dict` by
  hand. WASM goes through `js_json::to_value`, NAPI through `json_arg::to_js` (or
  a `serde_json::Value` return), and Python's dataclass `to_dict`s through
  `alizarin.canonical_static`. Each route goes via `serde_json::Value`, so JS objects
  are plain (no `Map`s) and keys come in its sorted order.
- **Empty is `null`, never `undefined`.** An absent optional result is `null`
  on every backend: WASM getters and functions return `JsValue` typed `T | null`
  (`js_json::or_null`), never `Option<T>` (which wasm-bindgen makes `undefined`,
  and `JSON.stringify` drops).
- **Which empty fields are written is core's choice, and follows Arches' shape.**
  A tile always writes `tileid`, `parenttile_id` and `sortorder` (`null` when
  empty), because Arches' business-data importer indexes them directly, and
  leaves out an empty `provisionaledits`, which it doesn't read. A publication
  always writes `notes`. Elsewhere a field core marks `skip_serializing_if` is
  left out when empty, on every backend alike.
- **Serialisation is byte-stable.** A `HashMap` field of a serialisable core
  struct either is a `BTreeMap` (small, hot maps such as a tile's `data`) or is
  written through `crate::sorted_serde::sorted_map`/`sorted_opt_map` (public
  fields whose Rust type stays). Otherwise a struct serialised straight to a
  string would come out in hash order, different on each run.

Checked by: `crates/alizarin-core/tests/output_parity_test.rs` (core writes
`tests/data/output-parity/expected.json`; byte-stability), which
`tests/dual-backend/outputParity.test.ts` (WASM, NAPI: every route, plus getters)
and `crates/alizarin-python/tests/test_output_parity.py` hold each binding to,
and `tests/sourceParity.test.ts`, which lints the sources for the rules above. A
deliberate change to the output regenerates the golden file
(`ALIZARIN_UPDATE_GOLDEN=1`), and the diff is the review.

## Addable on demand

The NAPI gaps are **thin wrappers over existing shared core** —
`alizarin_core::batch_convert_trees`, `alizarin_core::type_coercion::*`,
`alizarin_core::card_traversal::serialize_card`. If a future NAPI consumer needs
them, exposing them is a mechanical binding change (mirroring the WASM/Python
wrappers), not new core work. They are omitted today because shipping unused
public surface in a stable release is a liability, not a feature.

## 2.0 statement

For 2.0, the committed surface is the matrix above. New features land on all
three backends with parity tests (see "Backend parity is mandatory" in
`CLAUDE.md`); the NAPI gaps above are the remaining written-down exceptions.
Consumers should target the backend matching their environment (browser → WASM,
Node ETL → NAPI, notebooks → Python).
