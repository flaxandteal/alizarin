"""
Test RDM Collection creation and usage with batch tree conversion.

This demonstrates the complete workflow:
1. Create a collection from labels
2. Add it to an RDM cache
3. Use it for automatic label resolution in batch conversion
"""

import json
import pytest
import os

import alizarin


def load_test_graph():
    """Load test graph model from Group.json"""
    test_data_path = os.path.join(
        os.path.dirname(__file__),
        '../../../tests/data/models/Group.json'
    )

    with open(test_data_path) as f:
        data = json.load(f)
        return data['graph'][0]


def find_concept_node(graph_data):
    """
    Find a concept/concept-list node in the graph for testing.
    Returns (node, parent_semantic_alias) tuple.
    """
    for node in graph_data.get('nodes', []):
        datatype = node.get('datatype', '')
        if datatype in ('concept', 'concept-list'):
            # Found a concept node - check it has rdmCollection in config
            config = node.get('config', {})
            if 'rdmCollection' in config or 'rdm_collection' in config:
                # Find parent semantic node (nodegroup wrapper)
                ng_id = node.get('nodegroup_id')
                parent_alias = None

                for n2 in graph_data.get('nodes', []):
                    if n2['nodeid'] == ng_id and n2.get('datatype') == 'semantic':
                        parent_alias = n2.get('alias')
                        break

                return (node, parent_alias)
    return (None, None)


def create_tree_with_concept(node_alias, value, parent_alias=None):
    """
    Create a properly structured tree for a concept node.

    If the node has a parent semantic nodegroup, wraps it:
      {'permissions': [{'action': 'value'}]}

    If the node is at root level (no parent), returns flat:
      {'group_type': 'value'}
    """
    if parent_alias:
        # Wrap in parent semantic nodegroup
        return {parent_alias: [{node_alias: value}]}
    else:
        # Root-level node, no wrapper needed
        return {node_alias: value}


def test_create_collection_from_labels():
    """Test creating a collection from simple label strings"""
    # Create collection with auto-generated IDs
    collection = alizarin.RustRdmCollection.from_labels(
        name="Test Categories",
        labels=["Category A", "Category B", "Category C"]
    )

    # Verify collection properties
    assert collection.id is not None, "Collection should have an ID"
    assert collection.name == "Test Categories"
    assert len(collection) == 3, "Should have 3 concepts"

    # Verify concepts exist
    concept_ids = collection.get_concept_ids()
    assert len(concept_ids) == 3

    # Verify we can look up concepts by label
    concept_a = collection.find_by_label("Category A")
    assert concept_a is not None, "Should find concept by label"
    assert concept_a.get_label("en") == "Category A"

    print(f"Created collection '{collection.name}' with ID {collection.id}")
    print(f"Concepts: {concept_ids}")


def test_collection_with_explicit_id():
    """Test creating a collection with an explicit UUID"""
    explicit_id = "550e8400-e29b-41d4-a716-446655440000"

    collection = alizarin.RustRdmCollection.from_labels(
        name="Explicit ID Collection",
        labels=["Item 1", "Item 2"],
        id=explicit_id
    )

    assert collection.id == explicit_id
    assert len(collection) == 2


def test_collection_in_cache():
    """Test adding a collection to an RDM cache"""
    # Create collection
    collection = alizarin.RustRdmCollection.from_labels(
        name="Cached Categories",
        labels=["Red", "Green", "Blue"]
    )

    # Create cache and add collection
    cache = alizarin.RustRdmCache()
    cache.add_collection(collection)

    # Verify collection is cached
    assert cache.has_collection(collection.id)
    assert len(cache) == 1

    # Retrieve collection from cache
    retrieved = cache.get_collection(collection.id)
    assert retrieved is not None
    assert retrieved.name == "Cached Categories"
    assert len(retrieved) == 3

    # Look up concepts through cache
    concept = cache.lookup_by_label(collection.id, "Red")
    assert concept is not None
    assert concept.get_label("en") == "Red"


def test_batch_conversion_with_rdm_cache():
    """
    Test batch trees to tiles conversion with automatic label resolution.

    This demonstrates the key workflow:
    1. Create a collection matching a concept node in the graph
    2. Set it as the global RDM cache
    3. Use label strings in the tree (not UUIDs)
    4. Batch conversion automatically resolves labels to UUIDs
    """
    graph_data = load_test_graph()

    # Find a concept node in the graph
    concept_node, parent_alias = find_concept_node(graph_data)

    if not concept_node:
        pytest.skip("No concept nodes in test graph")

    node_alias = concept_node.get('alias')
    config = concept_node.get('config', {})
    collection_id = config.get('rdmCollection') or config.get('rdm_collection')

    if not collection_id:
        pytest.skip("Concept node has no rdmCollection config")

    print(f"\nTesting with concept node: {node_alias}")
    if parent_alias:
        print(f"Parent nodegroup: {parent_alias}")
    print(f"Collection ID: {collection_id}")

    # Create a test collection with this specific ID
    # Using explicit ID to match the graph's rdmCollection config
    collection = alizarin.RustRdmCollection.from_labels(
        name="Test Concepts",
        labels=["Concept Alpha", "Concept Beta", "Concept Gamma"],
        id=collection_id
    )

    # Create cache and add collection
    cache = alizarin.RustRdmCache()
    cache.add_collection(collection)

    # Set as global cache for automatic resolution
    alizarin.set_global_rdm_cache(cache)

    try:
        # Create trees using LABEL STRINGS (not UUIDs)
        # The batch conversion should auto-resolve these to UUIDs
        trees = [
            create_tree_with_concept(node_alias, "Concept Alpha", parent_alias),
            create_tree_with_concept(node_alias, "Concept Beta", parent_alias)
        ]

        trees_json = json.dumps(trees)
        graph_json = json.dumps(graph_data)

        # Batch convert - this should auto-resolve labels using the global cache
        result = alizarin.batch_trees_to_tiles(
            trees_json=trees_json,
            graph_json=graph_json,
            from_camel=False,
            strict=True  # Strict mode: fail if labels can't be resolved
        )

        # Verify result structure
        assert 'business_data' in result
        assert 'resources' in result['business_data']
        resources = result['business_data']['resources']

        assert len(resources) == 2, "Should have converted 2 trees"

        # Verify each resource has tiles with resolved UUIDs
        for i, resource in enumerate(resources):
            assert 'tiles' in resource
            tiles = resource['tiles']
            assert len(tiles) > 0, f"Resource {i} should have tiles"

            # Find the tile containing our concept node
            for tile in tiles:
                tile_data = tile.get('data', {})
                node_id = concept_node['nodeid']

                if node_id in tile_data:
                    value = tile_data[node_id]
                    print(f"Resource {i} - {node_alias} resolved to: {value}")

                    # Value should be a UUID, not the original label
                    assert value is not None
                    # It should be in the collection
                    assert cache.validate_concept(collection_id, value), \
                        f"Resolved value should be a valid concept ID"

        print("\n✓ Label resolution successful!")
        print(f"  Converted {len(resources)} resources")
        print(f"  Labels automatically resolved to UUIDs using RDM cache")

    finally:
        # Clean up global cache
        alizarin.clear_global_rdm_cache()


def test_batch_conversion_with_explicit_cache():
    """
    Test batch conversion with global cache cleanup between calls.

    Note: The Python API only supports global RDM cache, not explicit cache parameters.
    This test demonstrates proper cache management for isolated tests.
    """
    graph_data = load_test_graph()
    concept_node, parent_alias = find_concept_node(graph_data)

    if not concept_node:
        pytest.skip("No concept nodes in test graph")

    node_alias = concept_node.get('alias')
    config = concept_node.get('config', {})
    collection_id = config.get('rdmCollection') or config.get('rdm_collection')

    if not collection_id:
        pytest.skip("Concept node has no rdmCollection config")

    # Ensure clean state
    alizarin.clear_global_rdm_cache()

    try:
        # Create collection and set as global
        collection = alizarin.RustRdmCollection.from_labels(
            name="Isolated Cache Test",
            labels=["Value X", "Value Y"],
            id=collection_id
        )

        cache = alizarin.RustRdmCache()
        cache.add_collection(collection)
        alizarin.set_global_rdm_cache(cache)

        # Create tree with label
        trees = [create_tree_with_concept(node_alias, "Value X", parent_alias)]

        # Batch convert using global cache
        result = alizarin.batch_trees_to_tiles(
            trees_json=json.dumps(trees),
            graph_json=json.dumps(graph_data),
            from_camel=False,
            strict=True
        )

        assert 'business_data' in result
        resources = result['business_data']['resources']
        assert len(resources) == 1

        print("✓ Isolated cache test works!")

    finally:
        # Clean up global cache
        alizarin.clear_global_rdm_cache()


def test_collection_multilanguage():
    """Test creating concepts with multiple languages"""
    collection = alizarin.RustRdmCollection("test-multilang-collection")

    # Add concept with explicit ID and multilanguage labels
    concept_id = collection.add_from_label(
        label={"en": "Red Color", "ga": "Dath Dearg", "fr": "Couleur Rouge"},
        id="red-concept-uuid"
    )

    assert concept_id == "red-concept-uuid"

    # Retrieve and verify
    concept = collection.get_concept(concept_id)
    assert concept is not None
    assert concept.get_label("en") == "Red Color"
    assert concept.get_label("ga") == "Dath Dearg"
    assert concept.get_label("fr") == "Couleur Rouge"


def test_get_needed_collections():
    """Test identifying which collections a tree needs"""
    graph_data = load_test_graph()
    concept_node, parent_alias = find_concept_node(graph_data)

    if not concept_node:
        pytest.skip("No concept nodes in test graph")

    node_alias = concept_node.get('alias')
    config = concept_node.get('config', {})
    collection_id = config.get('rdmCollection') or config.get('rdm_collection')

    if not collection_id:
        pytest.skip("Concept node has no rdmCollection config")

    # Create a tree that uses this concept node
    tree = create_tree_with_concept(node_alias, "Some Label", parent_alias)

    # Get needed collections
    needed = alizarin.get_needed_collections(
        tree_json=json.dumps(tree),
        graph_json=json.dumps(graph_data)
    )

    assert collection_id in needed, \
        f"Should identify {collection_id} as needed"

    print(f"Tree needs collections: {needed}")


def test_resolve_labels_function():
    """Test the standalone resolve_labels function"""
    graph_data = load_test_graph()
    concept_node, parent_alias = find_concept_node(graph_data)

    if not concept_node:
        pytest.skip("No concept nodes in test graph")

    node_alias = concept_node.get('alias')
    config = concept_node.get('config', {})
    collection_id = config.get('rdmCollection') or config.get('rdm_collection')

    if not collection_id:
        pytest.skip("Concept node has no rdmCollection config")

    # Create collection
    collection = alizarin.RustRdmCollection.from_labels(
        name="Labels Test",
        labels=["Label One", "Label Two"],
        id=collection_id
    )

    cache = alizarin.RustRdmCache()
    cache.add_collection(collection)

    # Create tree with labels
    tree = create_tree_with_concept(node_alias, "Label One", parent_alias)
    tree_json = json.dumps(tree)
    graph_json = json.dumps(graph_data)

    # Resolve labels
    resolved_json, needed_collections = alizarin.resolve_labels(
        tree_json=tree_json,
        graph_json=graph_json,
        cache=cache,
        strict=True
    )

    # Parse resolved tree
    resolved_tree = json.loads(resolved_json)

    # Extract the resolved value (handle both wrapped and unwrapped structures)
    if parent_alias and parent_alias in resolved_tree:
        # Wrapped: {'permissions': [{'action': UUID}]}
        resolved_value = resolved_tree[parent_alias][0][node_alias]
    else:
        # Unwrapped: {'group_type': UUID}
        resolved_value = resolved_tree[node_alias]

    # Value should now be a UUID, not "Label One"
    assert resolved_value != "Label One", "Should have resolved to UUID"
    assert cache.validate_concept(collection_id, resolved_value), \
        "Resolved value should be valid concept ID"

    assert collection_id in needed_collections

    print(f"✓ resolve_labels: 'Label One' → '{resolved_value}'")


def test_skos_roundtrip():
    """
    Test SKOS RDF/XML roundtrip:
    1. Create RDM collection from labels
    2. Serialize to SKOS XML and write to file
    3. Read SKOS XML from file
    4. Parse back to RDM collection
    5. Use in batch conversion to verify data preservation
    """
    import tempfile

    graph_data = load_test_graph()
    concept_node, parent_alias = find_concept_node(graph_data)

    if not concept_node:
        pytest.skip("No concept nodes in test graph")

    node_alias = concept_node.get('alias')
    config = concept_node.get('config', {})
    collection_id = config.get('rdmCollection') or config.get('rdm_collection')

    if not collection_id:
        pytest.skip("Concept node has no rdmCollection config")

    print(f"\n--- Part 1: Create and Serialize ---")
    print(f"Testing with concept node: {node_alias}")
    if parent_alias:
        print(f"Parent nodegroup: {parent_alias}")
    print(f"Collection ID: {collection_id}")

    # Part 1: Create collection and serialize to SKOS
    original_labels = ["Roundtrip Alpha", "Roundtrip Beta", "Roundtrip Gamma"]

    original_collection = alizarin.RustRdmCollection.from_labels(
        name="SKOS Roundtrip Test",
        labels=original_labels,
        id=collection_id
    )

    # Verify original collection
    assert original_collection.id == collection_id
    assert len(original_collection) == 3
    print(f"Created collection with {len(original_collection)} concepts")

    # Serialize to SKOS XML
    base_uri = "http://example.org/test/"
    skos_xml = original_collection.to_skos_xml(base_uri)

    # Verify XML structure
    assert '<?xml version="1.0"' in skos_xml
    assert 'xmlns:skos="http://www.w3.org/2004/02/skos/core#"' in skos_xml
    assert 'skos:ConceptScheme' in skos_xml
    assert 'SKOS Roundtrip Test' in skos_xml
    for label in original_labels:
        assert label in skos_xml, f"Label '{label}' should be in XML"
    print("✓ Serialized to SKOS XML")

    # Write to temporary file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.xml', delete=False) as f:
        temp_file = f.name
        f.write(skos_xml)
        print(f"✓ Wrote SKOS XML to: {temp_file}")

    try:
        print(f"\n--- Part 2: Load and Use ---")

        # Read from file
        with open(temp_file, 'r') as f:
            loaded_xml = f.read()
        print(f"✓ Read SKOS XML from file ({len(loaded_xml)} bytes)")

        # Parse SKOS XML back to RDM collection
        cache = alizarin.RustRdmCache()
        loaded_collection_ids = cache.add_from_skos_xml(loaded_xml, base_uri)

        assert len(loaded_collection_ids) > 0, "Should load at least one collection"
        print(f"✓ Loaded {len(loaded_collection_ids)} collection(s) from SKOS")

        # Verify the collection was loaded
        assert cache.has_collection(collection_id), \
            f"Cache should contain collection {collection_id}"

        loaded_collection = cache.get_collection(collection_id)
        assert loaded_collection is not None
        assert len(loaded_collection) == 3, \
            f"Loaded collection should have 3 concepts, got {len(loaded_collection)}"
        print(f"✓ Loaded collection has {len(loaded_collection)} concepts")

        # Verify concepts can be looked up by their original labels
        for label in original_labels:
            concept = cache.lookup_by_label(collection_id, label)
            assert concept is not None, \
                f"Should be able to find concept by label '{label}'"
            assert concept.get_label("en") == label, \
                f"Concept label should match original"
        print(f"✓ All {len(original_labels)} labels preserved correctly")

        # Part 3: Use in batch conversion to verify it works end-to-end
        print(f"\n--- Part 3: Batch Conversion Test ---")

        alizarin.set_global_rdm_cache(cache)

        try:
            # Create trees using the labels
            trees = [
                create_tree_with_concept(node_alias, "Roundtrip Alpha", parent_alias),
                create_tree_with_concept(node_alias, "Roundtrip Beta", parent_alias),
            ]

            trees_json = json.dumps(trees)
            graph_json = json.dumps(graph_data)

            # Batch convert - labels should resolve using the loaded cache
            result = alizarin.batch_trees_to_tiles(
                trees_json=trees_json,
                graph_json=graph_json,
                from_camel=False,
                strict=True
            )

            # Verify result
            assert 'business_data' in result
            resources = result['business_data']['resources']
            assert len(resources) == 2, "Should have converted 2 trees"
            print(f"✓ Successfully converted {len(resources)} trees using loaded collection")

            # Verify labels were resolved to UUIDs
            for i, resource in enumerate(resources):
                tiles = resource.get('tiles', [])
                assert len(tiles) > 0, f"Resource {i} should have tiles"

                # Find tile with our concept node
                for tile in tiles:
                    tile_data = tile.get('data', {})
                    node_id = concept_node['nodeid']
                    if node_id in tile_data:
                        resolved_value = tile_data[node_id]
                        assert cache.validate_concept(collection_id, resolved_value), \
                            f"Resolved value should be a valid concept UUID"
                        print(f"  Resource {i}: Label resolved to UUID in tiles")
                        break

            print("\n✓ SKOS Roundtrip Complete!")
            print(f"  1. Created collection with {len(original_labels)} concepts")
            print(f"  2. Serialized to SKOS XML and wrote to file")
            print(f"  3. Loaded from file back into RDM cache")
            print(f"  4. All labels preserved and functional in batch conversion")

        finally:
            alizarin.clear_global_rdm_cache()

    finally:
        # Clean up temp file
        import os
        if os.path.exists(temp_file):
            os.unlink(temp_file)
            print(f"✓ Cleaned up temp file")


if __name__ == '__main__':
    print("=" * 70)
    print("RDM Collection and Batch Conversion Tests")
    print("=" * 70)

    print("\n1. Creating collection from labels...")
    test_create_collection_from_labels()

    print("\n2. Creating collection with explicit ID...")
    test_collection_with_explicit_id()

    print("\n3. Adding collection to cache...")
    test_collection_in_cache()

    print("\n4. Testing multilanguage concepts...")
    test_collection_multilanguage()

    print("\n5. Identifying needed collections...")
    try:
        test_get_needed_collections()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n6. Testing resolve_labels function...")
    try:
        test_resolve_labels_function()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n7. Batch conversion with global RDM cache...")
    try:
        test_batch_conversion_with_rdm_cache()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n8. Batch conversion with explicit cache parameter...")
    try:
        test_batch_conversion_with_explicit_cache()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n9. SKOS roundtrip (serialize → file → parse → convert)...")
    try:
        test_skos_roundtrip()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n" + "=" * 70)
    print("All tests completed!")
    print("=" * 70)
