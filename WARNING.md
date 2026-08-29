# Known Issues

## WASM backend: `getSummary` / `getFull` returns objects incompatible with JS property access

**Date**: 2026-08-02
**Affects**: `StaticResourceRegistry.getSummary()` and `getFull()` in WASM backend

> **RESOLVED** (2026-08-29). Root cause: the `descriptors` getters on the WASM
> `StaticResourceMetadata` / `StaticResourceSummary` wrappers returned a nested
> `StaticResourceDescriptors` **wrapper**. Property *reads* worked, but wrappers
> have no enumerable own properties, so when the ETL stored `meta.descriptors`
> into `__cache` and later `JSON.stringify`d it, it serialised to `{}` — hence
> "(unknown)". Fixed in `crates/alizarin-wasm/src/graph.rs` by having both
> getters return a plain JS object (`serde_wasm_bindgen::to_value`), matching the
> NAPI backend. The WASM workaround (`ALIZARIN_BACKEND=napi`) is no longer
> required.

Original symptom: under WASM, resource-instance references (e.g. associated
Person actors) were found in the registry but their metadata serialised to `{}`,
so starches-builder's `__cache` was never populated, `forDisplayJson` could not
resolve the references, and referenced resources displayed as "(unknown)" on the
public site. The NAPI backend was unaffected because it already returned plain
JS objects.

## Graph export missing `descriptor_types`

**Date**: 2026-08-02
**Affects**: Descriptor recomputation in starches-builder ETL

The Heritage Item graph JSON export has `descriptor_types: {}` (empty). This
means `getDescriptors(true)` on the instance wrapper cannot recompute resource
names from tiles — there is no template to evaluate.

The Arches server has the template (`<primary_reference_number> <monument_name>
<version>`) but it is not included in the graph JSON export. As a workaround,
starches-builder post-processes serialized JSON to strip version suffixes
(e.g. " 3.0") from resource names.

### To fix

Include `descriptor_types` in the graph JSON export so that `getDescriptors(true)`
can recompute names from tile data without relying on server-baked values.
