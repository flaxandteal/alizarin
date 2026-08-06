# Known Issues

## WASM backend: `getSummary` / `getFull` returns objects incompatible with JS property access

**Date**: 2026-08-02
**Affects**: `StaticResourceRegistry.getSummary()` and `getFull()` in WASM backend
**Workaround**: Use NAPI backend (`ALIZARIN_BACKEND=napi`) instead of WASM

When resources are loaded into the WASM registry via `mergeFromResourcesJson` as
summaries, `getSummary(id)` returns a WASM wrapper object. Accessing properties
like `.resourceinstance`, `.name`, `.graph_id` on this wrapper does not behave
the same as on plain JS objects returned by the NAPI backend.

This causes `__cache` population in starches-builder's ETL to silently fail
under WASM — resource-instance references (e.g. associated Person actors) are
found in the registry but their metadata cannot be read, so `__cache` entries
are never created. The result is that referenced resources display as "(unknown)"
on the public site.

The NAPI backend returns plain JS objects from `getSummary` / `getFull`, so the
same code works correctly.

### Impact

- `__cache` not populated for resource-instance references (People, Periods, etc.)
- `forDisplayJson` cannot resolve references to display strings
- Template rendering shows "(unknown)" for associated actors

### To fix

Ensure WASM `getSummary` / `getFull` return objects with the same JS property
access semantics as NAPI (plain objects or WASM wrappers with working getters
for `resourceinstance`, `name`, `graph_id`, `descriptors`).

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
