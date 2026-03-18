# RDM Collection Test

## Overview

`test_rdm_collection.py` demonstrates the complete workflow for using RDM (Reference Data Manager) collections with the Python Alizarin API, specifically showing how collections enable automatic label-to-UUID resolution in batch tree conversions.

## What This Test Demonstrates

### 1. **Creating Collections from Labels**
```python
collection = alizarin.RustRdmCollection.from_labels(
    name="Test Categories",
    labels=["Category A", "Category B", "Category C"]
)
```
- Auto-generates UUIDs for both collection and concepts
- Supports single-language (string) or multi-language (dict) labels

### 2. **Using RDM Cache**
```python
cache = alizarin.RustRdmCache()
cache.add_collection(collection)
alizarin.set_global_rdm_cache(cache)
```
- Cache stores multiple collections
- Global cache enables automatic label resolution across all conversion functions
- Alternative: pass `rdm_cache=cache` explicitly to functions

### 3. **Batch Conversion with Label Resolution**
```python
# Trees use LABEL STRINGS, not UUIDs
trees = [
    {"my_concept_field": "Category A"},  # Label, not UUID!
    {"my_concept_field": "Category B"}
]

# Batch convert - labels auto-resolve to UUIDs
result = alizarin.batch_trees_to_tiles(
    trees_json=json.dumps(trees),
    graph_json=json.dumps(graph_data),
    strict=True  # Fail if labels can't be resolved
)

# Result contains resources with UUID values in tiles
```

## Key Features Tested

1. **`RustRdmCollection.from_labels()`**
   - Creates collection with auto-generated IDs
   - Accepts list of string labels
   - Returns collection with UUIDs for each concept

2. **`RustRdmCollection.to_skos_xml()`**
   - Serializes collection to SKOS RDF/XML format
   - Returns XML string for file writing or network transfer
   - Preserves hierarchy and multilingual labels
   - Enables interoperability with SKOS-compatible systems

3. **`RustRdmCache`**
   - Stores collections by ID
   - Provides lookup functions
   - Supports global singleton for automatic resolution
   - Can parse SKOS XML via `add_from_skos_xml()`

4. **`batch_trees_to_tiles()` with RDM**
   - Accepts label strings in tree JSON
   - Automatically resolves to UUIDs using cache
   - Strict mode: fails on unresolved labels
   - Returns standard BusinessDataWrapper format

5. **`resolve_labels()` function**
   - Standalone label resolution
   - Returns both resolved JSON and needed collection IDs
   - Useful for pre-processing before conversion

6. **`get_needed_collections()`**
   - Scans tree to find required collection IDs
   - Enables pre-fetching collections
   - No resolution - just identification

7. **SKOS Roundtrip**
   - Create collection → serialize to XML → write file
   - Read SKOS file → parse to collection → use in conversion
   - Full data preservation through roundtrip

## Test Functions

| Function | Purpose |
|----------|---------|
| `test_create_collection_from_labels()` | Basic collection creation |
| `test_collection_with_explicit_id()` | Using explicit UUIDs |
| `test_collection_in_cache()` | Adding to cache and lookup |
| `test_batch_conversion_with_rdm_cache()` | **Main test** - batch conversion with global cache |
| `test_batch_conversion_with_explicit_cache()` | Passing cache explicitly |
| `test_collection_multilanguage()` | Multi-language label support |
| `test_get_needed_collections()` | Collection ID discovery |
| `test_resolve_labels_function()` | Standalone label resolution |
| `test_skos_roundtrip()` | **SKOS roundtrip** - serialize → file → parse → convert |

## Running the Tests

### Prerequisites
The Python module must be built and installed first.

### Option 1: Using pytest (Recommended)
```bash
cd crates/alizarin-python

# Build and install the module
pip install -e .

# Run all RDM collection tests
pytest tests/test_rdm_collection.py -v -p no:hypothesis

# Run specific test
pytest tests/test_rdm_collection.py::test_batch_conversion_with_rdm_cache -v -p no:hypothesis
```

### Option 2: Direct Python execution
```bash
cd crates/alizarin-python

# Build and install
pip install -e .

# Run test script
python tests/test_rdm_collection.py
```

**Note**: The `-p no:hypothesis` flag works around a known compatibility issue with the hypothesis plugin.

## Use Cases

### Static Site Builder
```python
# Define controlled vocabularies
asset_types = alizarin.RustRdmCollection.from_labels(
    name="Asset Types",
    labels=["Building", "Monument", "Archaeological Site"]
)

cache = alizarin.RustRdmCache()
cache.add_collection(asset_types)
alizarin.set_global_rdm_cache(cache)

# Authors use human-readable labels in source files
trees = [
    {"asset_type": "Building", "name": "Old Mill"},
    {"asset_type": "Monument", "name": "Stone Cross"}
]

# Build process converts to UUIDs automatically
result = alizarin.batch_trees_to_tiles(...)
# → asset_type values are now UUIDs referencing the concepts
```

### Data Migration
```python
# Load existing controlled lists from API
collection_ids = alizarin.get_needed_collections(tree_json, graph_json)

cache = alizarin.RustRdmCache()
for coll_id in collection_ids:
    concepts = fetch_from_api(coll_id)  # Your API call
    collection = create_collection_from_concepts(concepts)
    cache.add_collection(collection)

# Migrate legacy data with labels
migrated = alizarin.batch_trees_to_tiles(
    trees_json=legacy_data,
    graph_json=graph_json,
    rdm_cache=cache
)
```

### Form Validation
```python
# Pre-check if all labels are valid
needed = alizarin.get_needed_collections(user_input, graph_json)
ensure_all_loaded(needed)  # Load if missing

resolved, _ = alizarin.resolve_labels(
    tree_json=user_input,
    graph_json=graph_json,
    cache=cache,
    strict=True  # Fail on unknown labels
)
# If this succeeds, all labels were valid
```

## Architecture

```
User Input (Labels)
        ↓
RustRdmCollection.from_labels()
        ↓
RustRdmCache.add_collection()
        ↓
set_global_rdm_cache() or pass explicitly
        ↓
batch_trees_to_tiles(..., rdm_cache)
        ↓
Automatic Label Resolution (Rust)
        ↓
Output (UUIDs in tiles)
```

## Performance Benefits

- **Single pass**: Labels resolved during tree traversal
- **No pre-processing**: Don't need to manually convert labels to UUIDs
- **Batch optimization**: Process multiple trees in one call
- **Rust performance**: Label lookup is O(log n) in Rust HashMap

## Related Documentation

- Python API: `crates/alizarin-python/src/rdm_cache_py.rs`
- Core implementation: `crates/alizarin-core/src/rdm_cache.rs`
- Label resolution: `crates/alizarin-core/src/label_resolution.rs`
- Batch conversion API alignment: `WASM_PYTHON_API_ALIGNMENT.md`

## Common Issues

### Labels Not Resolving
- Ensure collection ID matches the graph node's `rdmCollection` config
- Check collection was added to cache before conversion
- Verify labels match concepts exactly (case-insensitive)

### Multiple Matches
- `find_by_label()` returns `None` if ambiguous
- Use `find_all_by_label()` to see all matches
- Consider using explicit IDs for multi-language scenarios

### UUID Generation
- Collection ID: `uuid5(COLLECTION_NAMESPACE, name)`
- Concept ID: `uuid5(collection_id, label)`
- Deterministic: same label always generates same UUID
