# CLM Collection Test

## Overview

`test_clm_collection.py` demonstrates using the CLM (Custom List Manager) extension for resource instance references, showing how the same RDM cache system works for both:
- **RDM**: Concept/concept-list nodes (controlled vocabularies)
- **CLM**: Resource-instance/resource-instance-list nodes (references to other resources)

The tests include comprehensive examples with multiple datatypes to show real-world usage.

## What This Test Demonstrates

### 1. **CLM Collections for References**
```python
# CLM collection works identically to RDM, but for resource references
clm_collection = alizarin.RustRdmCollection.from_labels(
    name="People",
    labels=["Alice Smith", "Bob Jones", "Carol Williams"],
    id=person_collection_id  # Matches node's rdmCollection config
)
```
- Creates UUIDs for resource instance references
- Same API as RDM collections
- Can be serialized to SKOS XML

### 2. **Mixed Datatypes in Complex Trees**
```python
tree = {
    'basic_info': [{
        'name': {'en': 'Research Group', 'ga': 'Grúpa Taighde'},  # Multilingual string
    }],
    'formation': [{
        'date': '2024-01-15',                                      # Date
        'person': ["Alice Smith", "Bob Jones"]                     # CLM references
    }],
    'statement': [{
        'description': 'A collaborative research group'            # Plain string
    }],
    'permissions': [{
        'action': "View"                                           # RDM concept
    }],
    'guideline_approval': True                                     # Boolean
}
```

### 3. **Multiple Nodegroups**
The examples demonstrate proper tree structure with multiple semantic nodegroups:
- `basic_info`: Contains name (string) and source (reference-list)
- `formation`: Contains date and person references
- `statement`: Contains description
- `permissions`: Contains action concepts and object reference
- Root level: Contains boolean fields

### 4. **CLM + RDM Together**
```python
# Both in the same cache
cache = alizarin.RustRdmCache()
cache.add_collection(clm_people)     # CLM for references
cache.add_collection(rdm_actions)    # RDM for concepts
alizarin.set_global_rdm_cache(cache)

# Both resolve in the same tree
tree = {
    'formation': [{
        'person': ["Alice Developer"]  # CLM label → UUID
    }],
    'permissions': [{
        'action': "Write"              # RDM label → UUID
    }]
}
```

### 5. **SKOS Roundtrip for CLM**
```python
# 1. Create CLM collection
clm_collection = alizarin.RustRdmCollection.from_labels(
    name="Information Sources",
    labels=["Primary Source", "Secondary Analysis"],
    id=source_collection_id
)

# 2. Serialize to SKOS XML
skos_xml = clm_collection.to_skos_xml(base_uri)

# 3. Write to file
with open('sources.xml', 'w') as f:
    f.write(skos_xml)

# 4. Load from file
cache = alizarin.RustRdmCache()
cache.add_from_skos_xml(skos_xml, base_uri)

# 5. Use in conversion
result = alizarin.batch_trees_to_tiles(
    trees_json=json.dumps(trees),
    graph_json=json.dumps(graph_data)
)
```

## Test Functions

| Function | Purpose |
|----------|---------|
| `test_create_clm_collection_from_labels()` | Basic CLM collection creation |
| `test_comprehensive_tree_with_clm()` | **Main test** - complex tree with CLM references and mixed datatypes |
| `test_clm_skos_roundtrip()` | **SKOS roundtrip** for CLM collections |
| `test_mixed_clm_and_rdm()` | Using both CLM and RDM in same tree |

## Key Differences: CLM vs RDM

### Conceptual Difference
- **RDM (Reference Data Manager)**: Controlled vocabularies, concepts, taxonomies
  - Example: "Action" concepts like "View", "Edit", "Delete"
  - Datatypes: `concept`, `concept-list`

- **CLM (Custom List Manager)**: References to other resource instances
  - Example: "Person" references like "Alice Smith", "Bob Jones"
  - Datatypes: `resource-instance`, `resource-instance-list`

### Technical Similarity
Both use the **same underlying system**:
- Same `RustRdmCollection` class
- Same `RustRdmCache` class
- Same SKOS serialization
- Same label-to-UUID resolution
- Same global cache mechanism

The only difference is **semantic intent** - one is for controlled vocabularies, the other is for resource references.

## Datatypes Demonstrated

### String Datatypes
```python
# Multilingual
'name': {'en': 'English Name', 'ga': 'Ainm Gaeilge'}

# Plain string
'description': 'Plain text description'
```

### Date Datatype
```python
'date': '2024-01-15'  # ISO 8601 format
```

### Boolean Datatype
```python
'guideline_approval': True
```

### Concept-List (RDM)
```python
'action': "View"  # Label resolves to concept UUID
```

### Resource-Instance-List (CLM)
```python
'person': ["Alice Smith", "Bob Jones"]  # Labels resolve to resource UUIDs
```

### Resource-Instance (CLM)
```python
'object': "SomeResource"  # Single reference (not a list)
```

## Running the Tests

### Prerequisites
The Python module must be built and installed first.

### Option 1: Using pytest (Recommended)
```bash
cd crates/alizarin-python

# Build and install the module
pip install -e .

# Run all CLM collection tests
pytest tests/test_clm_collection.py -v -p no:hypothesis

# Run specific test
pytest tests/test_clm_collection.py::test_comprehensive_tree_with_clm -v -p no:hypothesis
```

### Option 2: Direct Python execution
```bash
cd crates/alizarin-python

# Build and install
pip install -e .

# Run test script
python tests/test_clm_collection.py
```

## Use Cases

### Static Site with Author References
```python
# Define authors as CLM collection
authors = alizarin.RustRdmCollection.from_labels(
    name="Site Authors",
    labels=["Jane Doe", "John Smith"],
    id=author_collection_id
)

cache = alizarin.RustRdmCache()
cache.add_collection(authors)
alizarin.set_global_rdm_cache(cache)

# Markdown frontmatter can use human-readable names
trees = [{
    'article_info': [{
        'title': 'My Article',
        'author': ["Jane Doe"],      # CLM label
        'category': "Technology"     # RDM label (if category is concept-list)
    }]
}]

# Build converts to UUIDs
result = alizarin.batch_trees_to_tiles(trees_json, graph_json)
```

### Data Migration with References
```python
# Legacy system has names, need to convert to UUIDs
legacy_people = ["Person A", "Person B", "Person C"]

# Create CLM collection
people = alizarin.RustRdmCollection.from_labels(
    name="Legacy People",
    labels=legacy_people,
    id=person_collection_id
)

cache = alizarin.RustRdmCache()
cache.add_collection(people)

# Migrate with automatic resolution
migrated = alizarin.batch_trees_to_tiles(
    trees_json=legacy_data,
    graph_json=graph_json,
    rdm_cache=cache  # Explicit cache
)
```

### Form Validation with Mixed Types
```python
# Pre-validate user input
tree = {
    'basic_info': [{
        'name': user_input['name'],           # String
    }],
    'formation': [{
        'date': user_input['date'],           # Date
        'person': user_input['team_members']  # CLM references
    }],
    'permissions': [{
        'action': user_input['permission']    # RDM concept
    }]
}

# Validate all fields and references
resolved, needed = alizarin.resolve_labels(
    tree_json=json.dumps(tree),
    graph_json=graph_json,
    cache=cache,
    strict=True  # Fail if any reference invalid
)
```

## Architecture

```
User Input (Mixed Datatypes)
        ↓
Strings, Dates, Booleans → Processed directly
CLM Labels → RustRdmCollection → UUID resolution
RDM Labels → RustRdmCollection → UUID resolution
        ↓
batch_trees_to_tiles()
        ↓
Tiles with proper datatypes and resolved UUIDs
```

## Performance Benefits

- **Single cache**: Both CLM and RDM use same cache infrastructure
- **Single pass**: All label types resolved during tree traversal
- **No duplication**: Same code handles concepts and references
- **Batch optimization**: Process multiple resources with multiple reference types in one call

## Related Documentation

- Python RDM API: `crates/alizarin-python/src/rdm_cache_py.rs`
- RDM collection tests: `test_rdm_collection.py`
- Core implementation: `crates/alizarin-core/src/rdm_cache.rs`
- Label resolution: `crates/alizarin-core/src/label_resolution.rs`

## Common Issues

### References Not Resolving
- Ensure CLM collection ID matches the node's `rdmCollection` config
- Check both RDM and CLM collections are added to cache before conversion
- Verify labels match exactly (case-insensitive)

### Mixing Up CLM and RDM
- **RDM** is for `concept` and `concept-list` nodes
- **CLM** is for `resource-instance` and `resource-instance-list` nodes
- Both use the same `RustRdmCollection` API
- Check node datatype in graph to know which you need

### Tree Structure Issues
- Remember to wrap nodes in their parent semantic nodegroup
- Check `test_rdm_collection.py::create_tree_with_concept()` for pattern
- Use same pattern for references as for concepts

### UUID Generation
- Collection ID: `uuid5(COLLECTION_NAMESPACE, name)`
- Reference/Concept ID: `uuid5(collection_id, label)`
- Deterministic: same label always generates same UUID
- Both CLM and RDM use identical UUID generation
