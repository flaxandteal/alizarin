# Alizarin Python Bindings

Python bindings for Alizarin using PyO3, providing JSON serialization/deserialization for Arches tiled resources.

## Status

**In Progress** - Core types implemented, JSON conversion functions stubbed out.

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

## To Implement

### 1. Complete StaticGraphMeta

Currently only has basic fields. Needs:
- `nodes: Vec<StaticNode>` - all nodes in the graph
- `nodegroups: Vec<StaticNodegroup>` - all nodegroups
- `edges: Vec<StaticEdge>` - parent-child relationships

These are needed to understand the graph structure for JSON conversion.

### 2. Implement `resource_to_json()`

Currently returns placeholder. Should:

1. Load all tiles from resource
2. Group tiles by nodegroup_id
3. For each nodegroup:
   - Find corresponding node(s) via edges
   - If cardinality='n', create array
   - If cardinality='1', create single object
4. Build nested structure following node aliases
5. Convert tile data values to appropriate Python types (StaticTranslatableString, etc.)

**Reference implementation**: TypeScript `ResourceInstanceViewModel.__forJsonCache()` in `js/viewModels.ts:396-420`

### 3. Implement `json_to_resource()`

Inverse of `resource_to_json()`. Should:

1. Walk nested JSON structure
2. For each field, find corresponding node by alias
3. Determine nodegroup from node
4. Create tile(s) for that nodegroup
5. Populate tile.data with field values
6. Return StaticResource with all tiles

**Reference implementation**: TypeScript `ResourceInstanceViewModel.update()` (inverse operation)

### 4. Add Graph Loading Helper

```python
def load_graph_model(json_file_or_dict):
    """Load a complete graph model from JSON"""
    # Parse graph JSON
    # Return StaticGraphMeta with full node/nodegroup/edge data
```

## Usage Example

```python
import json
import alizarin

# Load graph model
with open('models/Group.json') as f:
    graph_data = json.load(f)
    graph = alizarin.StaticGraphMeta(json.dumps(graph_data['graph'][0]))

# Load tiled resource
with open('resources/my_group.json') as f:
    resource_data = json.load(f)
    resource = alizarin.StaticResource(json.dumps(resource_data))

# Convert to nested JSON
nested = alizarin.resource_to_json(resource, graph)

# Access fields naturally
print(nested['basic_info'][0]['name'])  # "Global Group"
print(nested['basic_info'][0]['name'].lang('ga'))  # "Grúpa Domhanda"

# Modify and convert back
nested['basic_info'][0]['name'] = {
    'en': 'Updated Group',
    'ga': 'Grúpa Nuashonraithe'
}

updated_resource = alizarin.json_to_resource(nested, graph)
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

Once implemented, tests should mirror the TypeScript test suite in `tests/resources.test.ts`.

```python
import pytest
import alizarin

def test_resource_to_json():
    """Test converting tiled resource to JSON"""
    # Load graph and resource
    # Convert to JSON
    # Assert structure matches expected
    assert json_output['basic_info'][0]['name'] == "Global Group"

def test_json_to_resource():
    """Test converting JSON to tiled resource"""
    # Create JSON structure
    # Convert to resource
    # Assert tiles are created correctly

def test_round_trip():
    """Test JSON -> Resource -> JSON preserves data"""
    # Load resource
    # Convert to JSON
    # Convert back to resource
    # Convert to JSON again
    # Assert both JSON outputs are identical
```

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

## Next Steps

1. Expand StaticGraphMeta with nodes/nodegroups/edges
2. Implement `resource_to_json()` using Rust logic from `instance_wrapper.rs`
3. Implement `json_to_resource()` (inverse operation)
4. Add pytest test suite mirroring JS tests
5. Document edge cases (missing fields, validation, etc.)
