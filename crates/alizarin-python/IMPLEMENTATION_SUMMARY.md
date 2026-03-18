# Alizarin Python Bindings - Implementation Summary

## Overview

Python bindings for Alizarin that replicate the JavaScript/TypeScript API,
including three-layer caching, type coercion, RDM collection management,
graph mutations, and extension type handlers.

## Python Modules

Location: `python/alizarin/`

| Module | Lines | Description |
|--------|-------|-------------|
| `__init__.py` | ~515 | Package exports, Rust bindings import, chunked merge helper |
| `view_models.py` | ~1,888 | 16 ViewModel classes + caching infrastructure |
| `static_types.py` | ~1,063 | Dataclass definitions for Arches graph/resource structures |
| `model_wrapper.py` | ~679 | ResourceModelWrapper with permissions and pruning |
| `pseudos.py` | ~657 | PseudoValue/PseudoList wrapper types |
| `instance_wrapper.py` | ~535 | ResourceInstanceWrapper and core population logic |
| `graph_manager.py` | ~443 | GraphManager coordinator and protocol definitions |
| `cli.py` | ~364 | Command-line interface |
| `semantic.py` | ~321 | SemanticViewModel for nested structures |
| `node_config.py` | ~207 | Node configuration classes |

Total: ~6,672 lines

## Rust PyO3 Bindings

Location: `src/`

| Module | Lines | Description |
|--------|-------|-------------|
| `lib.rs` | ~2,762 | Extension registry, batch tree conversion, mutation API, SKOS |
| `rdm_cache_py.rs` | ~1,744 | RDM cache with label resolution |
| `instance_wrapper_py.rs` | ~572 | ResourceInstanceWrapper bindings |
| `pseudos.py.rs` | ~457 | Pseudo value wrappers |
| `graph_mutator_py.rs` | ~455 | Graph mutation operations |
| `node_config_py.rs` | ~395 | Node configuration types |
| `type_coercion_py.rs` | ~368 | Type coercion functions |
| `skos_py.rs` | ~114 | SKOS RDF/XML parsing |
| `python_json.rs` | ~96 | JSON-Python conversion helpers |

Total: ~6,963 lines

## Test Suite

Location: `tests/` -- 16 test files, 204 tests passing.

| Test file | Tests | Lines | Covers |
|-----------|-------|-------|--------|
| `test_rdm_collection.py` | 27 | ~1,303 | RDM cache, collections, label resolution, SKOS |
| `test_resources.py` | 17 | ~678 | Resource wrapper, instance creation, pseudo lists |
| `test_batch_trees_minimal.py` | 8 | ~664 | Batch tree-to-tiles, cross-model, slug errors |
| `test_view_models.py` | 23 | ~603 | ViewModel creation and serialization |
| `test_static_types.py` | 27 | ~603 | Type serialization and deserialization |
| `test_graph_manager.py` | 9 | ~469 | GraphManager operations, caching |
| `test_conversion.py` | 6 | ~381 | JSON/tree conversions |
| `test_prune_graph.py` | 2 | ~331 | Permission pruning, ancestor preservation |
| `test_type_coercion.py` | 24 | ~301 | Domain/concept value coercion round-trips |
| `test_semantic_children.py` | 10 | ~290 | Semantic child matching |
| `test_cross_model_traversal.py` | 6 | ~280 | Multi-graph navigation |
| `test_matching_entries.py` | 5 | ~275 | Parent tile filtering, nodegroup matching |
| `test_create_graph_mutation.py` | 8 | ~267 | Graph creation mutations |
| `test_wkrm.py` | 7 | ~157 | Well-known resource model metadata |
| `test_attr_promise.py` | 5 | ~101 | Promise-like lazy attributes |
| `test_regression.py` | 1 | ~91 | Regression checks |

## Feature Status

### Complete

- **Type coercion** -- all 5 phases in Rust (scalars, dicts, config-dependent,
  RDM-dependent, format normalisation)
- **RDM cache** -- global singleton with collection loading, label resolution,
  UUID generation, SKOS XML parsing
- **Graph mutations** -- JSON-based mutation API, extension mutation registry,
  graph building from instructions and CSV
- **Extension type handlers** -- PyCapsule-based registration, coercion, display
  rendering, marker resolution via C ABI callbacks
- **Batch operations** -- `batch_trees_to_tiles`, `batch_tiles_to_trees`,
  `batch_merge_resources`, lazy `TreeToTilesIterator`
- **View models** -- 16 types: String, NonLocalizedString, Number, Boolean,
  Date, EDTF, Url, GeoJSON, DomainValue, DomainValueList, ConceptValue,
  ConceptList, ResourceInstance, ResourceInstanceList, Semantic + base IViewModel
- **Cross-model traversal** -- GraphManager, WKRM protocol, semantic child
  matching, resource lookup
- **Resource model wrapper** -- permissions (boolean + conditional), graph
  pruning, node/edge/nodegroup caching
- **Three-layer caching** -- Rust pseudo_cache -> Python _pseudo_cache ->
  Python value_cache (JSON-serialisable)
- **SKOS** -- RDF/XML parsing and serialisation
- **Label resolution** -- graph-aware, config-driven, integrated with batch ops
- **Node config** -- Python-side manager with Rust PyO3 type wrappers
- **CLI** -- command-line interface for common operations
- **CI/CD** -- GitHub Actions for tests and releases

### Not implemented

- Lazy/async RDM collection fetching (collections must be pre-loaded)
- Type stub files (`.pyi`)
- Sphinx documentation
- General-purpose renderers (only extension handler display rendering exists)

## Architecture

### Three-Layer Caching

```
Arches -> Tiles -> Layer 1 (Rust) -> Layer 2 (Python objects) -> Layer 3 (JSON)
                       |                    |                         |
                   populate()        retrieve_pseudo()      build_value_cache()
```

Layer 1 (Rust `pseudo_cache`) is the authoritative source. Layer 2 caches
wrapped Python objects to reduce FFI calls. Layer 3 provides JSON-serialisable
representations for rendering.

### Rust-Python Boundary

- PyO3 with C ABI callbacks for extension handlers
- Thread-safe global state via `RwLock` + `lazy_static` (RDM cache, type
  handlers, graph registry)
- Parallel batch processing via rayon

## Building and Testing

```bash
cd crates/alizarin-python
maturin develop --release
pytest tests/ -v
```

## Usage

```python
from alizarin import graph_manager

# Load model
Groups = await graph_manager.get("Group")

# Navigate with caching
groups = await Groups.all()
group = groups[0]
basic_info = await group.basic_info
name = await basic_info[0].name
print(name)               # "Global Group"
print(name.lang("ga"))    # "Grupa Domhanda"
```
