# Alizarin Python Bindings

Python bindings for Alizarin using PyO3, providing JSON serialization/deserialization for Arches tiled resources.

## Status

**Complete** - Tree conversion functions fully implemented using shared Rust core logic.

## Goal

Provide simple Python functions to convert between:
- **Tiled Resource Format** (StaticResource with tiles) ← Arches native format
- **Nested JSON Format** (hierarchical dict) ← Developer-friendly format

This mirrors the TypeScript `ResourceInstanceViewModel.forJson()` and inverse functionality.

## Implemented

✅ Core PyO3 types:
- `StaticTranslatableString` - with `__eq__`, `__str__`, `.lang(language)` method
- `StaticGraphMeta` - graph model metadata
- `StaticNode` - node definitions
- `StaticNodegroup` - nodegroup definitions
- `StaticTile` - tile data storage
- `StaticResource` - resource with tiles

✅ Helper functions:
- `serde_json_to_py()` - Convert Rust JSON to Python objects
- `py_to_serde_json()` - Convert Python objects to Rust JSON

## Implementation Notes

The core tree conversion logic is implemented in shared Rust code (`src/json_conversion.rs`), which is exposed to Python via thin PyO3 bindings. This ensures identical behavior between Python and TypeScript/WASM interfaces.

Key implementation details:
- Uses complete `StaticGraph` with nodes, nodegroups, and edges
- Handles nodes without nodegroups (like root nodes)
- Preserves cardinality ('1' vs 'n') when converting
- Creates new tile IDs during tree → tiles conversion
- Supports nested structures with parent-child relationships

## Usage Example

```python
import json
import alizarin

# Load graph model
with open('tests/data/models/Group.json') as f:
    graph_data = json.load(f)
    graph_json = json.dumps(graph_data['graph'][0])

# Example tiles (tiled resource format)
tiles = [{
    'tileid': 'tile-001',
    'nodegroup_id': '12707705-c05e-11e9-8177-a4d18cec433a',
    'resourceinstance_id': 'resource-123',
    'data': {
        '127095f5-c05e-11e9-bb57-a4d18cec433a': {
            'en': 'My Group Name',
            'ga': 'Ainm Mo Ghrúpa'
        }
    }
}]

# Convert tiles to nested JSON tree
tree = alizarin.tiles_to_json_tree(
    tiles_json=json.dumps(tiles),
    resource_id='resource-123',
    graph_id=graph_data['graph'][0]['graphid'],
    graph_json=graph_json
)

# Access fields in tree structure
print(tree['basic_info'][0]['name'])  # {"en": "My Group Name", "ga": "Ainm Mo Ghrúpa"}

# Modify tree structure
tree['basic_info'][0]['name'] = {
    'en': 'Updated Group',
    'ga': 'Grúpa Nuashonraithe'
}

# Convert back to tiles
result = alizarin.json_tree_to_tiles(
    tree_json=json.dumps(tree),
    graph_json=graph_json
)

# Result has: resourceinstanceid, graph_id, tiles
print(f"Created {len(result['tiles'])} tiles")
```

## Building

Requires Rust workspace setup. From project root:

```bash
# Build Python extension
cd crates/alizarin-python
maturin develop

# Or build wheel
maturin build --release
```

## Testing

Tests are located in `tests/test_conversion.py` and verify the bindings work correctly.

```bash
# Install with test dependencies
cd crates/alizarin-python
maturin develop
pip install pytest

# Run tests
pytest tests/

# Or run directly
python tests/test_conversion.py
```

The test suite includes:
- `test_tiles_to_tree_basic()` - Converts tiled resource to JSON tree
- `test_tree_to_tiles_basic()` - Converts JSON tree to tiled resource
- `test_roundtrip_preserves_data()` - Verifies tiles → tree → tiles preserves data
- `test_invalid_json_handling()` - Verifies error handling for malformed JSON

All tests use real data from `tests/data/models/Group.json`.

## Architecture

The Python bindings use the same Rust core logic as the WASM bindings, just with a different FFI layer:

```
┌─────────────────────────┐
│   Python Application    │
│   (uses nested JSON)    │
└───────────┬─────────────┘
            │
            │ resource_to_json()
            │ json_to_resource()
            ▼
┌─────────────────────────┐
│   PyO3 Bindings         │
│   (this crate)          │
└───────────┬─────────────┘
            │
            │ calls
            ▼
┌─────────────────────────┐
│   Rust Core Logic       │
│   (alizarin-core)       │
│   - Node hierarchy      │
│   - Tile management     │
│   - Validation          │
└─────────────────────────┘
```

This is identical to the WASM architecture:

```
JS/TS ← wasm-bindgen ← Rust Core
Python ← PyO3 ← Rust Core (same!)
```

## Why This Approach?

1. **Single Source of Truth**: Business logic lives in Rust, tested once
2. **Performance**: Conversion happens in compiled Rust code
3. **Type Safety**: PyO3 ensures type correctness at compile time
4. **Consistency**: Python and JavaScript get identical behavior

## Potential Enhancements

1. Add type stubs (`.pyi` files) for better IDE support
2. Expose additional Rust types (StaticNode, StaticNodegroup, etc.) if needed
3. Add validation functions to check tree/tile structure
4. Optimize large dataset handling (streaming, batching)
5. Add examples for common use cases (Django integration, data migration, etc.)
