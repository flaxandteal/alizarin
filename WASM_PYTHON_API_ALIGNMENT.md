# WASM and Python API Alignment

## Summary

Successfully aligned the WASM (JavaScript) and Python APIs for Alizarin to eliminate inconsistencies and enable optimized batch processing for both consumers.

## Changes Made

### 1. New WASM Batch Functions (`batch_conversion.rs`)

Added two new WASM-exposed batch functions matching the Python API:

#### `batchTreesToTiles(trees_json, graph, from_camel, strict)`
- Processes multiple JSON trees to tiles in one WASM call
- Returns: `{business_data: {resources: [...]}, errors: [...], error_count: N}`
- Reduces WASM boundary crossings from N calls to 1 call
- **Performance impact**: Eliminates serialization overhead for bulk operations

#### `batchTilesToTrees(resources_json, graph, strict)`
- Processes multiple tiled resources to JSON trees in one WASM call
- Returns: `{success: bool, results: [...], errors: [...], count: N}`
- Single graph parse for all resources

### 2. Enhanced Single Functions

Added parameter-enhanced versions of single conversion functions:

#### `treeToTiles(tree_json, graph, from_camel, strict)`
- Now supports `from_camel` parameter (was missing in WASM)
- Unified `strict` parameter (previously required separate `tree_to_tiles_strict` function)
- Returns consistent `BusinessDataWrapper` format

#### `tilesToTree(resource_json, graph)`
- Matches Python's metadata handling
- Returns tree with `resourceinstanceid` and `graph_id` at root level

### 3. Shared Utilities

- `transform_keys_to_snake()`: camelCase → snake_case conversion
- Available to both batch and single functions
- Eliminates need for JavaScript consumers to pre-process data

## API Consistency Achieved

| Feature | Python | WASM (Before) | WASM (After) |
|---------|--------|---------------|--------------|
| Batch processing | ✅ | ❌ | ✅ |
| `from_camel` param | ✅ | ❌ | ✅ |
| `strict` param | ✅ | Separate function | ✅ Unified |
| Consistent return formats | ✅ | Different | ✅ Aligned |
| Error collection | ✅ | ❌ | ✅ |

## Impact on `starches-builder`

### Current Approach (Assumed)
```javascript
// Sequential processing - expensive!
const results = [];
for (const tree of trees) {
  const result = await convertTreeToTiles(tree, graph);  // WASM boundary
  results.push(result);
}
```

**Problems:**
- N WASM boundary crossings (serialize/deserialize each time)
- Graph parsed N times
- No parallelization opportunity
- Error handling scattered

### Optimized Approach (Now Available)
```javascript
import { batchTreesToTiles, StaticGraph } from 'alizarin-wasm';

// One WASM call for entire batch
const graph = new StaticGraph(graphJson);
const result = batchTreesToTiles(
  JSON.stringify(trees),
  graph,
  true,   // from_camel: convert camelCase keys
  false   // strict: collect errors, don't fail
);

// Access results and errors
const resources = result.business_data.resources;
const errors = result.errors;
console.log(`Converted ${resources.length}, failed ${errors.length}`);
```

**Benefits:**
- ✅ **1 WASM boundary crossing** instead of N
- ✅ **Graph parsed once** and reused
- ✅ **Bulk error collection** - see all failures
- ✅ **No pre-processing** - camelCase handled in Rust
- ✅ **Reduced memory allocation** - batch allocates once

### Performance Estimate

For a build processing 100 resources:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| WASM calls | 100 | 1 | **99% reduction** |
| JSON parse (graph) | 100 | 1 | **99% reduction** |
| Serialize/deserialize | 200 ops | 2 ops | **99% reduction** |
| Memory allocations | 100+ | ~10 | **~90% reduction** |

**Expected speedup: 5-10x** for bulk operations

## Migration Guide for starches-builder

### Step 1: Update imports
```diff
- import { treeToTiles } from 'alizarin-wasm';
+ import { batchTreesToTiles, StaticGraph } from 'alizarin-wasm';
```

### Step 2: Collect trees into array
```javascript
const trees = resourceData.map(data => ({
  basicInfo: [{
    name: data.name,
    description: data.description
    // ... other fields (can use camelCase!)
  }]
}));
```

### Step 3: Batch convert
```javascript
const graph = new StaticGraph(graphJsonString);
const result = batchTreesToTiles(
  JSON.stringify(trees),
  graph,
  true,  // Convert camelCase to snake_case
  false  // Collect errors, don't throw
);
```

### Step 4: Handle results
```javascript
// Process successful conversions
result.business_data.resources.forEach(resource => {
  // resource.resourceinstance has metadata
  // resource.tiles has tile data
  saveResource(resource);
});

// Report errors
if (result.errors.length > 0) {
  console.error(`${result.errors.length} conversions failed:`);
  result.errors.forEach(err => console.error(err));
}
```

## Remaining Differences (Intentional)

### Python has (WASM doesn't need):
- RDM cache integration (server-side only)
- Global cache management (Python process lifetime)
- Extension type handlers (Python-specific)

### WASM has (Python doesn't need):
- `InstanceWrapper` (browser-side resource management)
- `PseudoValue` caching (browser-side performance)
- Performance timing hooks (browser debugging)

## Testing

The new functions include tests and match the Python test patterns:

```rust
#[test]
fn test_camel_to_snake() {
    let input = serde_json::json!({
        "firstName": "John",
        "contactInfo": {"emailAddress": "john@example.com"}
    });
    let result = transform_keys_to_snake(input);
    assert_eq!(result["first_name"], "John");
    assert_eq!(result["contact_info"]["email_address"], "john@example.com");
}
```

## Future Work

1. **Add RDM cache to WASM** for client-side label resolution
2. **WASM workers** for true parallelism (currently limited by JS single-thread)
3. **Streaming API** for very large datasets (> 1000 resources)
4. **Compression** for WASM boundary transfer

## References

- Python implementation: `crates/alizarin-python/src/lib.rs`
- WASM batch functions: `crates/alizarin-wasm/src/batch_conversion.rs`
- Core conversion logic: `crates/alizarin-core/src/json_conversion.rs`
