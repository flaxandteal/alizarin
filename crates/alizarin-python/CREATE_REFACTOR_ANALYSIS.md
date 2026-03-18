# Analysis: Refactoring `_create` Methods

## Current Status

Most of this refactoring has been completed. Type coercion (all 5 phases) is
implemented in Rust (`type_coercion_py.rs`), the RDM cache with eager-load
collection lookup is in `rdm_cache_py.rs`, and node config types are wrapped
via PyO3 in `node_config_py.rs`. The main outstanding item is lazy/async
collection fetching -- collections must currently be pre-loaded before
conversion.

## Architecture

```
                    Python/TypeScript                          Rust
                    -----------------                          ----
                                                         NodeConfigManager
                                                         (graph domain values,
                                                          boolean labels)
                                                               |
                                                               v
Input Value ---+---> (a) Recognize ViewModel ---> Copy/Reference
               |
               +---> (b) Recognize Tile Data -----------------> json_to_tile_data()
               |       (dict/list passthrough)                      |
               |                                                    v
               +---> (c) Coerce to JSON -------------------------> json_to_tile_data()
                       - resolve promises                            |
                       - convert rich types                          |
                                                                     v
                                                         +-------------------+
                                                         | Domain Lookup     |
                                                         | (UUID -> Value)   |
                                                         +-------------------+
                                                                     |
                                                                     v
                                                         Validated tile data +
                                                         Resolved domain values
```

## What Was Implemented

### Type Coercion in Rust (all phases)

| Phase | Types | Status |
|-------|-------|--------|
| 1 - Simple scalars | number, non_localized_string, edtf, date | Done |
| 2 - Dict types | string, url, geojson | Done |
| 3 - Config-dependent | boolean, domain_value, domain_value_list | Done |
| 4 - RDM-dependent | concept_value, concept_list | Done |
| 5 - Format normalization | resource_instance, resource_instance_list | Done |

Dispatcher: `coerce_value()` with language context, exposed to Python.

### RDM Collection Handling (Eager-Load Pattern)

Collections are loaded into a global `RustRdmCache` before batch operations.
Rust handles all sync lookups; Python/TS handles async I/O (fetching
collections from external services).

```python
cache = alizarin.RustRdmCache()
collection = alizarin.RustRdmCollection.from_labels(namespace, {"en": ["Option 1"]})
cache.add_collection(collection)
alizarin.set_global_rdm_cache(cache)

# Rust uses the pre-loaded cache during conversion
result = alizarin.batch_trees_to_tiles(trees, ...)
```

### Node Config

- Python-side: `NodeConfigManager` with `StaticNodeConfigBoolean`,
  `StaticNodeConfigConcept`, `StaticNodeConfigDomain`,
  `StaticNodeConfigReference` (in `node_config.py`)
- Rust-side: PyO3 wrappers in `node_config_py.rs`
- Config management is driven from Python; Rust provides the types and lookup

### ViewModel Constructors

All ViewModels now have consistent constructors that effectively take tile
data, with Rust doing resolution where needed:

| ViewModel | Constructor | Resolution |
|-----------|-------------|------------|
| Date, EDTF, NonLocalizedString, Number | Tile data directly | None needed |
| String, GeoJSON | Dict directly | None needed |
| Url | Url dataclass | Minor intermediate type |
| Boolean | bool + config | Rust provides labels |
| DomainValue | StaticDomainValue | Rust nodeConfig lookup |
| ConceptValue | StaticValue | Rust RdmCache lookup |
| ResourceInstance | UUID + optional extras | Lazy cross-resource loading |

## Remaining Work

### Lazy RDM collection fetching

The eager-load pattern requires all collections to be loaded before conversion.
A lazy/async loader that fetches collections on-demand during conversion would
allow incremental loading. The `resolve_markers_fn` infrastructure exists for
extension handlers but there is no RDM-specific lazy loader yet.

### NodeConfig centralisation

Node config management is primarily Python-driven. Moving the
`NodeConfigManager.from_graph()` logic fully into Rust would reduce FFI
overhead during graph loading.
