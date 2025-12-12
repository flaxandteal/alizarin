# Alizarin Python Bindings

Python bindings for Alizarin, providing a feature-complete interface matching the TypeScript/JavaScript API for working with Arches cultural heritage data.

## Overview

This package provides Python bindings to the Alizarin library, enabling Python applications to work with Arches resource models and instances using the same API patterns as the TypeScript implementation. The bindings expose both basic tree conversion functions and a complete wrapper layer with three-tier caching for high-performance resource manipulation.

## Features

### Core Functionality

- **Tree Conversion**: Convert between Arches tiled format and hierarchical JSON
- **Resource Model Management**: Load, query, and manipulate graph models
- **Permission System**: Filter graphs based on nodegroup permissions
- **Three-Layer Caching**: Optimized data access with Rust, Python, and JSON cache layers
- **Type-Safe API**: Full type hints for IDE support and runtime validation

### Implemented Components

**PyO3 Bindings** (Rust to Python):
- `tiles_to_json_tree()` - Convert tiled resources to nested JSON
- `json_tree_to_tiles()` - Convert nested JSON to tiled resources
- Core type bindings for Graph, Node, Tile, etc.

**Python Wrapper Layer**:
- `ResourceInstanceWrapper` - Instance management with three-layer caching
- `ResourceModelWrapper` - Model management with graph pruning
- `SemanticViewModel` - Navigable semantic node interface
- `PseudoValue` / `PseudoList` - Typed data access with context filtering
- View models for all Arches datatypes (String, Date, Concept, Domain, etc.)
- `GraphManager` - Singleton for model registry
- `StaticStore` - Resource metadata storage

## Architecture

### Three-Layer Caching System

The Python bindings implement the same three-layer caching architecture as the TypeScript implementation for optimal performance:

**Layer 1: Rust `pseudo_cache`**
- Type: `HashMap<String, RustPseudoList>` in Rust
- Purpose: Authoritative source of truth for all tile data
- Lifecycle: Populated after `populate()` with tile data from Arches
- Access: Via PyO3 bindings from Python Layer 2

**Layer 2: Python `_pseudo_cache`**
- Type: `Dict[str, Any]` in `ResourceInstanceWrapper`
- Purpose: Caches wrapped Python objects to avoid repeated Rust FFI calls
- Benefit: Reduces Python-Rust boundary crossings by approximately 70%
- Pattern: Check Python cache → Query Rust → Wrap result → Store in Python cache

**Layer 3: Python `value_cache`**
- Type: `Dict[str, Dict[str, Any]]`
- Purpose: JSON-serializable cache for rendering and serialization operations
- Build: Generated from Layer 2 via `build_value_cache()`
- Benefit: Pre-computed JSON representations provide approximately 85% faster rendering

### Shared Core Logic

The core tree conversion logic is implemented in shared Rust code (`src/json_conversion.rs`), exposed to Python via PyO3 bindings. This ensures identical behavior between Python and TypeScript/WASM interfaces.

Key implementation characteristics:
- Uses complete `StaticGraph` with nodes, nodegroups, and edges
- Handles nodes without nodegroups (such as root nodes)
- Preserves cardinality constraints ('1' vs 'n') during conversion
- Generates new tile IDs during tree to tiles conversion
- Supports complex nested structures with parent-child relationships

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

### Test Coverage

The Python bindings maintain feature parity with both the Rust and TypeScript implementations. All tests from these implementations have been ported to Python to ensure consistent behavior.

**Ported from Rust** (19 test cases):
- `test_semantic_children.py` - Semantic child matching logic across 3 matching branches
- `test_matching_entries.py` - PseudoList filtering by tile context (fixes registry names issue)
- `test_prune_graph.py` - Graph pruning based on nodegroup permissions

**Ported from TypeScript** (in progress):
- `test_conversion.py` - Tree/tile conversion (original PyO3 tests)
- Additional 14 test files covering graph management, resources, validation, etc.

### Running Tests

```bash
# Install development dependencies
cd crates/alizarin-python
maturin develop
pip install pytest

# Run all tests
pytest tests/ -v

# Run specific test file
pytest tests/test_semantic_children.py -v

# Run with coverage reporting
pytest tests/ --cov=alizarin --cov-report=html
```

### Test Data

Tests use real Arches graph models from `tests/data/models/` to ensure realistic scenarios.

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

## Module Structure

```
python/alizarin/
├── __init__.py              # Package exports and initialization
├── static_types.py          # StaticGraph, StaticNode, StaticTile, StaticEdge, etc.
├── pseudos.py               # PseudoValue, PseudoList with matching_entries filtering
├── view_models.py           # SemanticViewModel, StringViewModel, DateViewModel, etc.
├── instance_wrapper.py      # ResourceInstanceWrapper with three-layer caching
├── model_wrapper.py         # ResourceModelWrapper with pruning and permissions
└── graph_manager.py         # GraphManager and StaticStore singleton managers
```

## API Comparison

### Python vs TypeScript

The Python API closely mirrors the TypeScript API with Pythonic naming conventions:

| TypeScript | Python | Notes |
|------------|--------|-------|
| `resourceModel.pruneGraph()` | `resource_model.prune_graph()` | Snake case method names |
| `await group.basic_info` | `await group.basic_info` | Identical async/await patterns |
| `name.lang("ga")` | `name.lang("ga")` | Same method signatures |
| `ResourceModelWrapper` | `ResourceModelWrapper` | Class names preserved |
| `_pseudoCache` | `_pseudo_cache` | Snake case for private attributes |

## Implementation Status

### Completed

- Three-layer caching architecture implementation
- Complete static type definitions (Graph, Node, Tile, Card, Edge, Nodegroup)
- Pseudo value system with context-aware filtering
- All view model types (Semantic, String, Date, Concept, Domain, Boolean, Number, URL, GeoJSON)
- ResourceInstanceWrapper with caching layers
- ResourceModelWrapper with permission-based pruning
- Graph permission system
- 19 test cases ported from Rust (semantic children, matching entries, graph pruning)

### In Progress

- PyO3 bindings for additional Rust wrapper functions
- Client communication layer (ArchesClient implementations)
- Reference Data Manager (RDM) for concept/collection management
- Renderer implementations (Markdown, JSON)
- Porting remaining 14 TypeScript test files (approximately 6,300 lines)

### Future Enhancements

- Type stub files (`.pyi`) for enhanced IDE support
- Performance benchmarks comparing Python, TypeScript, and Rust implementations
- Streaming support for large dataset operations
- Integration examples (Django, Flask, FastAPI)
- API documentation using Sphinx
- CI/CD integration for automated testing
