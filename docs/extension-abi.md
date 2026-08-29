# Extension ABI (frozen for 2.0)

Out-of-tree datatype extensions plug into alizarin through a small C ABI. An
extension builds a `TypeHandlerInfo`, wraps it in a `PyCapsule`, and registers it
via `alizarin.register_type_handler(capsule)`. The host (`alizarin-python`)
adapts it into the shared `ExtensionTypeRegistry` that core, and the other
bindings, use.

This document is the committed contract for 2.0. `ABI_VERSION` is **2**.

> **Scope.** The C ABI is the **Python** out-of-tree mechanism (PyCapsule). WASM
> registers handlers via JS callbacks; NAPI static-links its handler cores. Those
> do not use this ABI. "Freeze the extension ABI" means freeze this Python
> capsule contract.

## The surface

`TypeHandlerInfo` (in `alizarin-extension-api`) carries a datatype name, an
`AbiFingerprint`, and optional function pointers for **six** capabilities — the
full set the `ExtensionTypeHandler` trait declares. Each capability a handler
provides is a `Some(fn)`; the rest are `None`, and the host reports
`HandlerCapabilities` from which pointers are present.

| Capability | Fn pointer (+ free) | Contract |
|---|---|---|
| coerce | `coerce_fn` / `free_fn` | raw value (JSON bytes) → `CoerceResult` (tile + display). `None` ⇒ core keeps coercion. |
| display render | `render_display_fn` / `free_display_fn` | resolved value + language → display string |
| resolve markers | `resolve_markers_fn` / `free_resolve_markers_fn` | resolve `__needs_rdm_lookup` etc. after coercion |
| validate | `validate_fn` / `free_validate_fn` | value **by pointer** + config → `ValidationResult` |
| search render | `render_search_fn` / `free_render_search_fn` | value **by pointer** + language → `Option<Value>` (search-indexable JSON) |
| index spec | `index_spec_fn` / `free_index_spec_fn` | value **by pointer** + config → `Option<IndexSpec>` (semantic `IndexClass` + **raw** keys) |

The four newer capabilities pass the value **by pointer** (`*const
serde_json::Value`) — no per-value serialization on the hot path. The small
result (a serialized struct or an error string) is the only thing marshalled
back, and it is freed via the paired `free_*` pointer.

### Design principle: declare semantics, not structure

`index_spec` returns the **semantic index class** (`ConceptHierarchical`,
`Ordered`, `SpatialBbox`, `Link`, …) plus the **raw** keys (ids/values). A
handler declares *what kind of index* its datatype needs — it does **not** choose
the coarse grouping, the head-table layout, or the partition granularity. All
coarsening (concept-hierarchy DFS-intervals, bbox extraction, quantization) lives
in the index **emitter** (e.g. ros-madair), keyed off the class. This keeps
storage/query structure out of the extension surface: an extension author never
needs to know the consumer's parquet/head layout.

There is deliberately **no `coarse` parameter** on the ABI — it would force
structural knowledge into the handler.

## The handshake

`TypeHandlerInfo` carries an `AbiFingerprint`:

```rust
AbiFingerprint {
    version: ABI_VERSION,                          // 2
    value_size: size_of::<serde_json::Value>(),
    handler_info_size: size_of::<TypeHandlerInfo>(),
}
```

The host checks `info.abi == abi_fingerprint()` **at registration**. A mismatch
is refused with an error **before any pointer is dereferenced** — so a stale
extension fails loudly at load, never corrupting memory.

### Soundness constraint (read before shipping an extension)

Pointer-passing `*const serde_json::Value` across the `.so` boundary is sound
**only when core and the extension share the same `serde_json`** (same memory
layout for `Value`). In practice this means: build extensions against the **same
workspace lockfile and toolchain** as the alizarin they load into. The
fingerprint's `value_size` is the guard — a `serde_json` whose `Value` size
differs is refused. (Same-size-but-reordered layout is only caught by the
`version` field, which is why the versioning policy below is strict.)

## Versioning policy

**Bump `ABI_VERSION`** on any change that alters the binary contract:

- adding, removing, or reordering a field in `TypeHandlerInfo`;
- changing any `*Fn` / `*Result` signature or layout;
- a `serde_json` major bump that changes `Value`'s layout;
- any change to the semantics a fn pointer promises.

Additive changes are **not** silently compatible: adding a fn pointer changes
`size_of::<TypeHandlerInfo>()`, so the fingerprint mismatches and old extensions
are refused (safely) until rebuilt. That is intended — there is no partial-ABI
loading. Bump the version, rebuild the in-tree extensions (CLM, filelist, geo),
and cut a coordinated release.

`ABI_VERSION` history:
- **1** — coerce, display, resolve_markers, validate; fingerprint handshake.
- **2** — added `render_search` and `index_spec` (completes parity with the
  `ExtensionTypeHandler` trait's six capabilities).

## Writing an extension (sketch)

```rust
// Build the info once (a constructor sets abi + the unused pointers to None):
let info = TypeHandlerInfo::new_validating(DATATYPE_NAME, validate_fn, free_fn);
// ...or a struct literal for a multi-capability handler; either way set
// `abi: abi_fingerprint()`.

// Expose it as a PyCapsule; register at import:
//   alizarin.register_type_handler(get_handler_capsule())
```

The in-tree extensions are the reference implementations:
`ext/alizarin-clm/python` (coerce + display + resolve_markers),
`ext/filelist/python` (coerce + display), `ext/alizarin-geo/python` (validate).
