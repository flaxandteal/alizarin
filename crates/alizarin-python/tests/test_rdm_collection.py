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


# Test namespace UUID for deterministic ID generation
TEST_RDM_NAMESPACE = "550e8400-e29b-41d4-a716-446655440000"


@pytest.fixture(autouse=True)
def setup_rdm_namespace():
    """Set up RDM namespace before each test, clean up after."""
    alizarin.set_rdm_namespace(TEST_RDM_NAMESPACE)
    yield
    alizarin.clear_rdm_namespace()


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


def create_tree_with_concept(node_alias, value, parent_alias=None, datatype=None):
    """
    Create a properly structured tree for a concept node.

    If the node has a parent semantic nodegroup, wraps it:
      {'permissions': [{'action': ['value']}]}  (concept-list)
      {'permissions': [{'action': 'value'}]}    (concept)

    If the node is at root level (no parent), returns flat:
      {'group_type': ['value']}  (concept-list)
      {'group_type': 'value'}    (concept)
    """
    # concept-list expects an array of values
    if datatype == "concept-list":
        value = [value]

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


def test_namespace_required_for_auto_generation():
    """Test that namespace must be set for auto-generated IDs"""
    # Clear namespace to test error case
    alizarin.clear_rdm_namespace()

    # Should raise error when trying to auto-generate collection ID
    with pytest.raises(RuntimeError, match="RDM namespace not set"):
        alizarin.RustRdmCollection.from_labels(
            name="Should Fail",
            labels=["A", "B"]
        )

    # Explicit ID should still work without namespace
    collection = alizarin.RustRdmCollection.from_labels(
        name="Explicit ID Works",
        labels=[],
        id="explicit-uuid-here"
    )
    assert collection.id == "explicit-uuid-here"

    # Re-set namespace for subsequent tests
    alizarin.set_rdm_namespace(TEST_RDM_NAMESPACE)


def test_deterministic_uuid_generation():
    """Test that UUIDs are deterministic with same namespace and inputs"""
    # Create two collections with same name - should have same ID
    collection1 = alizarin.RustRdmCollection.from_labels(
        name="Deterministic Test",
        labels=["Alpha", "Beta"]
    )

    collection2 = alizarin.RustRdmCollection.from_labels(
        name="Deterministic Test",
        labels=["Alpha", "Beta"]
    )

    assert collection1.id == collection2.id, "Same name should produce same collection ID"

    # Concepts with same labels should have same IDs
    concept1 = collection1.find_by_label("Alpha")
    concept2 = collection2.find_by_label("Alpha")
    assert concept1.id == concept2.id, "Same label should produce same concept ID"


def test_multilingual_label_deterministic():
    """Test that multilingual labels produce deterministic UUIDs"""
    # Collection ID must be a valid UUID for concept ID generation
    collection_uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
    collection1 = alizarin.RustRdmCollection(collection_uuid)
    collection2 = alizarin.RustRdmCollection(collection_uuid)

    # Same multilingual dict (regardless of iteration order) should produce same ID
    id1 = collection1.add_from_label({"en": "Hello", "de": "Hallo", "fr": "Bonjour"})
    id2 = collection2.add_from_label({"fr": "Bonjour", "en": "Hello", "de": "Hallo"})

    assert id1 == id2, "Multilingual labels with same content should produce same ID"


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
            create_tree_with_concept(node_alias, "Concept Alpha", parent_alias, datatype=concept_node['datatype']),
            create_tree_with_concept(node_alias, "Concept Beta", parent_alias, datatype=concept_node['datatype'])
        ]

        graph_json = json.dumps(graph_data)

        # Register the graph first (required by new API)
        graph_id = alizarin.register_graph(graph_json)

        # Resolve labels to UUIDs using the global cache before batch conversion
        # (batch_trees_to_tiles does not auto-resolve; use resolve_labels_in_tree)
        resolved_trees = [
            json.loads(alizarin.resolve_labels_in_tree(json.dumps(t), graph_id, strict=True))
            for t in trees
        ]
        trees_json = json.dumps(resolved_trees)

        # Batch convert with resolved UUIDs
        result = alizarin.batch_trees_to_tiles(
            trees_json=trees_json,
            graph_id=graph_id,
            from_camel=False,
            strict=True,
            random_ids=True,
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
                    # concept-list stores an array of UUIDs; concept stores a single UUID
                    values_to_check = value if isinstance(value, list) else [value]
                    for v in values_to_check:
                        assert cache.validate_concept(collection_id, v), \
                            f"Resolved value {v} should be a valid concept ID"

        print("\n✓ Label resolution successful!")
        print(f"  Converted {len(resources)} resources")
        print("  Labels automatically resolved to UUIDs using RDM cache")

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
        trees = [create_tree_with_concept(node_alias, "Value X", parent_alias, datatype=concept_node['datatype'])]

        # Register the graph first (required by new API)
        graph_id = alizarin.register_graph(json.dumps(graph_data))

        # Resolve labels to UUIDs before batch conversion
        resolved_trees = [
            json.loads(alizarin.resolve_labels_in_tree(json.dumps(t), graph_id, strict=True))
            for t in trees
        ]

        # Batch convert using resolved UUIDs
        result = alizarin.batch_trees_to_tiles(
            trees_json=json.dumps(resolved_trees),
            graph_id=graph_id,
            from_camel=False,
            strict=True,
            random_ids=True,
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
    tree = create_tree_with_concept(node_alias, "Some Label", parent_alias, datatype=concept_node['datatype'])

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
    tree = create_tree_with_concept(node_alias, "Label One", parent_alias, datatype=concept_node['datatype'])
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
    # concept-list stores an array of UUIDs; concept stores a single UUID
    values_to_check = resolved_value if isinstance(resolved_value, list) else [resolved_value]
    for v in values_to_check:
        assert cache.validate_concept(collection_id, v), \
            "Resolved value should be valid concept ID"

    assert collection_id in needed_collections

    print(f"✓ resolve_labels: 'Label One' → '{resolved_value}'")


def test_hierarchical_collection():
    """Test creating a collection with hierarchical concepts using add_child_from_label"""
    # Create collection
    collection = alizarin.RustRdmCollection.from_labels(
        name="Hierarchical Test",
        labels=["Animals"],
        id="550e8400-e29b-41d4-a716-446655440001"
    )

    # Find the parent concept
    parent = collection.find_by_label("Animals")
    assert parent is not None, "Should find parent concept"
    parent_id = parent.id

    # Add child concepts using add_child_from_label
    child1_id = collection.add_child_from_label(parent_id, "Mammals")
    child2_id = collection.add_child_from_label(parent_id, "Birds")

    # Verify collection has 3 concepts total
    assert len(collection) == 3, "Should have 3 concepts (1 parent + 2 children)"

    # Verify children exist
    child1 = collection.get_concept(child1_id)
    child2 = collection.get_concept(child2_id)
    assert child1 is not None, "Child 1 should exist"
    assert child2 is not None, "Child 2 should exist"

    assert child1.get_label("en") == "Mammals"
    assert child2.get_label("en") == "Birds"

    # Verify only parent is top-level
    top_concepts = collection.get_top_concepts()
    assert len(top_concepts) == 1, "Should have 1 top-level concept"
    assert top_concepts[0].id == parent_id, "Parent should be the top concept"

    print(f"✓ Created hierarchical collection with parent '{parent.get_label('en')}' and 2 children")


def test_hierarchical_collection_multilingual():
    """Test hierarchical collection with multilingual labels"""
    collection = alizarin.RustRdmCollection.from_labels(
        name="Multilingual Hierarchy",
        labels=["Colors"],
        id="550e8400-e29b-41d4-a716-446655440002"
    )

    parent = collection.find_by_label("Colors")
    parent_id = parent.id

    # Add child with multilingual labels
    child_id = collection.add_child_from_label(
        parent_id,
        label={"en": "Red", "ga": "Dearg", "fr": "Rouge"},
        id="red-color-uuid"
    )

    assert child_id == "red-color-uuid"

    child = collection.get_concept(child_id)
    assert child.get_label("en") == "Red"
    assert child.get_label("ga") == "Dearg"
    assert child.get_label("fr") == "Rouge"

    print("✓ Created hierarchical collection with multilingual child")


def test_hierarchical_collection_skos_serialization():
    """Test that hierarchical collections serialize correctly to SKOS XML"""
    # Create hierarchical collection
    collection = alizarin.RustRdmCollection.from_labels(
        name="SKOS Hierarchy Test",
        labels=["Parent Concept"],
        id="550e8400-e29b-41d4-a716-446655440003"
    )

    parent = collection.find_by_label("Parent Concept")
    collection.add_child_from_label(parent.id, "Child One")
    collection.add_child_from_label(parent.id, "Child Two")

    # Serialize to SKOS XML
    base_uri = "http://example.org/test/"
    skos_xml = collection.to_skos_xml(base_uri)

    # Verify hierarchy is represented
    assert "skos:narrower" in skos_xml, "Should have narrower relationships"
    assert "skos:broader" in skos_xml, "Should have broader relationships"
    assert "Parent Concept" in skos_xml
    assert "Child One" in skos_xml
    assert "Child Two" in skos_xml

    print("✓ Hierarchical collection serializes with narrower/broader relationships")


def test_arches_collection_type():
    """Test creating an Arches-compatible Collection (not ConceptScheme)"""
    # Create with node_type="Collection"
    collection = alizarin.RustRdmCollection.from_labels(
        name="Arches Collection",
        labels=["Item A", "Item B"],
        id="550e8400-e29b-41d4-a716-446655440004",
        node_type="Collection"
    )

    assert collection.node_type == "Collection"
    assert len(collection) == 2

    # Serialize - should use skos:Collection not skos:ConceptScheme
    base_uri = "http://example.org/test/"
    skos_xml = collection.to_skos_xml(base_uri)

    assert "skos:Collection" in skos_xml, "Should be skos:Collection type"
    assert "skos:ConceptScheme" not in skos_xml, "Should NOT be skos:ConceptScheme"
    assert "skos:member" in skos_xml, "Should use skos:member relations"

    print("✓ Created Arches-compatible Collection with skos:member relations")


def test_arches_collection_with_hierarchy():
    """Test that Arches Collections support hierarchical concepts"""
    # Create Collection type (not ConceptScheme)
    collection = alizarin.RustRdmCollection.from_labels(
        name="Hierarchical Arches Collection",
        labels=["Root"],
        id="550e8400-e29b-41d4-a716-446655440005",
        node_type="Collection"
    )

    assert collection.node_type == "Collection"

    # Add child concepts
    parent = collection.find_by_label("Root")
    collection.add_child_from_label(parent.id, "Branch A")
    collection.add_child_from_label(parent.id, "Branch B")

    assert len(collection) == 3

    # Serialize
    base_uri = "http://example.org/test/"
    skos_xml = collection.to_skos_xml(base_uri)

    # Should be Collection type
    assert "skos:Collection" in skos_xml
    assert "skos:ConceptScheme" not in skos_xml

    # Should list ALL concepts as members (including children)
    member_count = skos_xml.count("<skos:member>")
    assert member_count == 3, f"Should have 3 members (all concepts), got {member_count}"

    # Should also have narrower/broader on concepts
    assert "skos:narrower" in skos_xml, "Should have narrower relationships"
    assert "skos:broader" in skos_xml, "Should have broader relationships"

    print("✓ Arches Collection supports hierarchy with member + narrower/broader")


def test_hierarchical_collection_roundtrip():
    """Test SKOS roundtrip preserves hierarchy"""

    # Create hierarchical collection
    collection = alizarin.RustRdmCollection.from_labels(
        name="Roundtrip Hierarchy",
        labels=["Animals"],
        id="550e8400-e29b-41d4-a716-446655440006"
    )

    parent = collection.find_by_label("Animals")
    collection.add_child_from_label(parent.id, "Mammals")
    collection.add_child_from_label(parent.id, "Birds")

    original_count = len(collection)
    original_top_count = len(collection.get_top_concepts())

    # Serialize
    base_uri = "http://example.org/test/"
    skos_xml = collection.to_skos_xml(base_uri)

    # Parse back
    cache = alizarin.RustRdmCache()
    loaded_ids = cache.add_from_skos_xml(skos_xml, base_uri)

    assert len(loaded_ids) > 0

    # Get the loaded collection
    loaded = cache.get_collection(collection.id)
    assert loaded is not None

    # Verify counts match
    assert len(loaded) == original_count, \
        f"Should preserve concept count: expected {original_count}, got {len(loaded)}"

    loaded_top = loaded.get_top_concepts()
    assert len(loaded_top) == original_top_count, \
        f"Should preserve top-level count: expected {original_top_count}, got {len(loaded_top)}"

    # Verify labels preserved
    assert loaded.find_by_label("Animals") is not None
    assert loaded.find_by_label("Mammals") is not None
    assert loaded.find_by_label("Birds") is not None

    print("✓ Hierarchical collection roundtrip preserves structure")


def test_from_nested_labels():
    """Test creating a hierarchical collection from nested dictionary structure"""
    # Create hierarchical collection from nested dict
    collection = alizarin.RustRdmCollection.from_nested_labels(
        "Animals",
        {
            "Mammals": {
                "Dogs": None,
                "Cats": None
            },
            "Birds": {
                "Eagles": None,
                "Sparrows": None
            }
        }
    )

    # Verify collection properties
    assert collection.id is not None
    assert collection.name == "Animals"
    assert len(collection) == 6, f"Should have 6 concepts, got {len(collection)}"

    # Verify all concepts exist
    assert collection.find_by_label("Mammals") is not None
    assert collection.find_by_label("Dogs") is not None
    assert collection.find_by_label("Cats") is not None
    assert collection.find_by_label("Birds") is not None
    assert collection.find_by_label("Eagles") is not None
    assert collection.find_by_label("Sparrows") is not None

    # Verify hierarchy - only Mammals and Birds should be top-level
    top_concepts = collection.get_top_concepts()
    assert len(top_concepts) == 2, f"Should have 2 top-level concepts, got {len(top_concepts)}"
    top_labels = {c.get_label("en") for c in top_concepts}
    assert top_labels == {"Mammals", "Birds"}, f"Top concepts should be Mammals and Birds, got {top_labels}"

    # Verify parent-child relationships
    mammals = collection.find_by_label("Mammals")
    assert "Dogs" in [collection.get_concept(n).get_label("en") for n in mammals.narrower]
    assert "Cats" in [collection.get_concept(n).get_label("en") for n in mammals.narrower]

    dogs = collection.find_by_label("Dogs")
    assert mammals.id in dogs.broader, "Dogs should have Mammals as broader"

    print("Created hierarchical collection from nested dict:")
    print("  - 6 concepts total, 2 top-level")
    print("  - Mammals -> Dogs, Cats")
    print("  - Birds -> Eagles, Sparrows")


def test_from_nested_labels_flat():
    """Test from_nested_labels with all None values (flat collection)"""
    collection = alizarin.RustRdmCollection.from_nested_labels(
        "Colors",
        {"Red": None, "Green": None, "Blue": None}
    )

    assert len(collection) == 3
    top_concepts = collection.get_top_concepts()
    assert len(top_concepts) == 3, "All concepts should be top-level"

    print("Created flat collection from nested dict with None values")


def test_from_nested_labels_with_node_type():
    """Test from_nested_labels with Collection node type"""
    collection = alizarin.RustRdmCollection.from_nested_labels(
        "Arches Hierarchy",
        {
            "Category A": {
                "Sub A1": None,
                "Sub A2": None
            }
        },
        node_type="Collection"
    )

    assert collection.node_type == "Collection"
    assert len(collection) == 3

    # Serialize and verify it uses skos:Collection
    skos_xml = collection.to_skos_xml("http://example.org/test/")
    assert "skos:Collection" in skos_xml
    assert "skos:member" in skos_xml
    # Should have 3 members (all concepts)
    member_count = skos_xml.count("<skos:member>")
    assert member_count == 3

    print("Created Arches Collection from nested dict")


def test_to_simplified_json():
    """Test exporting collection to simplified JSON format (Arches prebuild compatible)"""
    # Create a hierarchical collection
    collection = alizarin.RustRdmCollection.from_nested_labels(
        "Animals",
        {
            "Mammals": {
                "Dogs": None,
                "Cats": None
            },
            "Birds": None
        },
        id="550e8400-e29b-41d4-a716-446655440099"
    )

    # Export to simplified JSON
    json_str = collection.to_simplified_json()
    data = json.loads(json_str)

    # Verify structure
    assert data["id"] == "550e8400-e29b-41d4-a716-446655440099"
    assert data["prefLabels"][""]["value"] == "Animals"
    assert "concepts" in data

    # Should have 2 top-level concepts (Mammals, Birds)
    assert len(data["concepts"]) == 2

    # Find Mammals concept
    mammals = None
    for concept_id, concept in data["concepts"].items():
        if concept["prefLabels"]["en"]["value"] == "Mammals":
            mammals = concept
            break

    assert mammals is not None, "Mammals concept should exist"
    assert "children" in mammals, "Mammals should have children"
    assert len(mammals["children"]) == 2, "Mammals should have 2 children"

    # Children should have the expected structure
    child_labels = {c["prefLabels"]["en"]["value"] for c in mammals["children"]}
    assert child_labels == {"Dogs", "Cats"}

    # Each concept should have source and sortOrder (even if null)
    for concept in data["concepts"].values():
        assert "source" in concept
        assert "sortOrder" in concept
        assert "id" in concept["prefLabels"]["en"]  # value ID

    print("Exported simplified JSON with hierarchical structure")


def test_to_simplified_json_flat():
    """Test simplified JSON export with flat collection"""
    collection = alizarin.RustRdmCollection.from_labels(
        "Colors",
        ["Red", "Green", "Blue"],
        id="550e8400-e29b-41d4-a716-446655440098"
    )

    json_str = collection.to_simplified_json()
    data = json.loads(json_str)

    assert len(data["concepts"]) == 3
    # None should have children
    for concept in data["concepts"].values():
        assert "children" not in concept

    print("Exported flat collection to simplified JSON")


def test_from_nested_labels_deep_hierarchy():
    """Test from_nested_labels with 3+ levels of nesting"""
    collection = alizarin.RustRdmCollection.from_nested_labels(
        "Deep Taxonomy",
        {
            "Level 1": {
                "Level 2": {
                    "Level 3": None
                }
            }
        }
    )

    assert len(collection) == 3

    # Verify hierarchy chain
    level1 = collection.find_by_label("Level 1")
    level2 = collection.find_by_label("Level 2")
    level3 = collection.find_by_label("Level 3")

    assert level1 is not None
    assert level2 is not None
    assert level3 is not None

    # Check relationships
    assert level2.id in level1.narrower
    assert level3.id in level2.narrower
    assert level1.id in level2.broader
    assert level2.id in level3.broader

    # Only Level 1 should be top-level
    top_concepts = collection.get_top_concepts()
    assert len(top_concepts) == 1
    assert top_concepts[0].id == level1.id

    print("Created 3-level deep hierarchy from nested dict")


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

    print("\n--- Part 1: Create and Serialize ---")
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
        print("\n--- Part 2: Load and Use ---")

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
                "Concept label should match original"
        print(f"✓ All {len(original_labels)} labels preserved correctly")

        # Part 3: Use in batch conversion to verify it works end-to-end
        print("\n--- Part 3: Batch Conversion Test ---")

        alizarin.set_global_rdm_cache(cache)

        try:
            # Create trees using the labels
            trees = [
                create_tree_with_concept(node_alias, "Roundtrip Alpha", parent_alias, datatype=concept_node['datatype']),
                create_tree_with_concept(node_alias, "Roundtrip Beta", parent_alias, datatype=concept_node['datatype']),
            ]

            graph_json = json.dumps(graph_data)

            # Register the graph first (required by new API)
            graph_id = alizarin.register_graph(graph_json)

            # Resolve labels to UUIDs before batch conversion
            resolved_trees = [
                json.loads(alizarin.resolve_labels_in_tree(json.dumps(t), graph_id, strict=True))
                for t in trees
            ]
            trees_json = json.dumps(resolved_trees)

            # Batch convert with resolved UUIDs
            result = alizarin.batch_trees_to_tiles(
                trees_json=trees_json,
                graph_id=graph_id,
                from_camel=False,
                strict=True,
                random_ids=True,
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
                        # concept-list stores an array of UUIDs; concept stores a single UUID
                        values_to_check = resolved_value if isinstance(resolved_value, list) else [resolved_value]
                        for v in values_to_check:
                            assert cache.validate_concept(collection_id, v), \
                                "Resolved value should be a valid concept UUID"
                        print(f"  Resource {i}: Label resolved to UUID in tiles")
                        break

            print("\n✓ SKOS Roundtrip Complete!")
            print(f"  1. Created collection with {len(original_labels)} concepts")
            print("  2. Serialized to SKOS XML and wrote to file")
            print("  3. Loaded from file back into RDM cache")
            print("  4. All labels preserved and functional in batch conversion")

        finally:
            alizarin.clear_global_rdm_cache()

    finally:
        # Clean up temp file
        import os
        if os.path.exists(temp_file):
            os.unlink(temp_file)
            print("✓ Cleaned up temp file")


def test_deterministic_skos_export():
    """SKOS XML export should be identical across multiple calls.

    This ensures exports can be compared with git diff for changes.
    """
    # Create a collection with multiple concepts and multilingual labels
    # Use dict format for explicit ordering control
    collection = alizarin.RustRdmCollection.from_nested_labels(
        name="Determinism Test",
        structure={
            "Animals": {
                "Mammals": {
                    "Dogs": None,
                    "Cats": None,
                },
                "Birds": {
                    "Eagles": None,
                    "Sparrows": None,
                }
            },
            "Plants": {
                "Trees": None,
                "Flowers": None,
            }
        }
    )

    # Add multilingual labels to make it more interesting
    base_uri = "http://example.org/test/"

    # Export multiple times
    xml1 = collection.to_skos_xml(base_uri)
    xml2 = collection.to_skos_xml(base_uri)
    xml3 = collection.to_skos_xml(base_uri)

    # All exports should be byte-identical
    assert xml1 == xml2, "First two SKOS XML exports should be identical"
    assert xml2 == xml3, "Second and third SKOS XML exports should be identical"

    print(f"✓ SKOS XML is deterministic ({len(xml1)} bytes, 3 identical exports)")


def test_deterministic_simplified_json_export():
    """Simplified JSON export should be identical across multiple calls.

    This ensures exports can be compared with git diff for changes.
    """
    # Create a collection with multiple concepts and hierarchy
    collection = alizarin.RustRdmCollection.from_nested_labels(
        name="JSON Determinism Test",
        structure={
            "Category A": {
                "Sub A1": None,
                "Sub A2": None,
            },
            "Category B": {
                "Sub B1": {
                    "Sub B1a": None,
                    "Sub B1b": None,
                },
                "Sub B2": None,
            },
            "Category C": None,
        }
    )

    # Export multiple times
    json1 = collection.to_simplified_json()
    json2 = collection.to_simplified_json()
    json3 = collection.to_simplified_json()

    # All exports should be string-identical
    assert json1 == json2, "First two simplified JSON exports should be identical"
    assert json2 == json3, "Second and third simplified JSON exports should be identical"

    # Also verify it's valid JSON
    data = json.loads(json1)
    assert "concepts" in data
    assert "id" in data

    print(f"✓ Simplified JSON is deterministic ({len(json1)} bytes, 3 identical exports)")


def test_url_namespace():
    """Test that URLs can be used as namespaces for deterministic ID generation.

    This allows using the same URL as both the RDF namespace and the UUID
    generation namespace.
    """
    # Clear any existing namespace
    alizarin.clear_rdm_namespace()

    # Set namespace using a URL
    url = "http://example.org/vocab/"
    alizarin.set_rdm_namespace(url)

    # The URL should be converted to a deterministic UUID
    derived_uuid = alizarin.get_rdm_namespace()
    assert derived_uuid is not None, "Namespace should be set"
    assert alizarin.is_valid_uuid(derived_uuid), "Derived value should be a valid UUID"

    # Create a collection
    collection1 = alizarin.RustRdmCollection.from_labels(
        name="URL Test",
        labels=["Item A", "Item B"]
    )
    id1 = collection1.id
    concept_ids1 = collection1.get_concept_ids()

    # Clear and set the same URL again
    alizarin.clear_rdm_namespace()
    alizarin.set_rdm_namespace(url)

    # Should get the same derived UUID
    assert alizarin.get_rdm_namespace() == derived_uuid, "Same URL should give same UUID"

    # Create the same collection again
    collection2 = alizarin.RustRdmCollection.from_labels(
        name="URL Test",
        labels=["Item A", "Item B"]
    )

    # Should get identical IDs (compare as sets since order isn't guaranteed)
    assert collection2.id == id1, "Same URL namespace should produce same collection ID"
    assert set(collection2.get_concept_ids()) == set(concept_ids1), "Same URL namespace should produce same concept IDs"

    # Reset to test namespace
    alizarin.clear_rdm_namespace()
    alizarin.set_rdm_namespace(TEST_RDM_NAMESPACE)

    print(f"✓ URL namespace works ('{url}' → '{derived_uuid}')")


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

    print("\n10. Hierarchical collection with add_child_from_label...")
    test_hierarchical_collection()

    print("\n11. Hierarchical collection with multilingual labels...")
    test_hierarchical_collection_multilingual()

    print("\n12. Hierarchical collection SKOS serialization...")
    test_hierarchical_collection_skos_serialization()

    print("\n13. Arches Collection type...")
    test_arches_collection_type()

    print("\n14. Arches Collection with hierarchy...")
    test_arches_collection_with_hierarchy()

    print("\n15. Hierarchical collection roundtrip...")
    test_hierarchical_collection_roundtrip()

    print("\n16. from_nested_labels - hierarchical dict...")
    test_from_nested_labels()

    print("\n17. from_nested_labels - flat dict...")
    test_from_nested_labels_flat()

    print("\n18. from_nested_labels - with Collection node type...")
    test_from_nested_labels_with_node_type()

    print("\n19. from_nested_labels - deep hierarchy (3 levels)...")
    test_from_nested_labels_deep_hierarchy()

    print("\n20. Deterministic SKOS XML export...")
    test_deterministic_skos_export()

    print("\n21. Deterministic simplified JSON export...")
    test_deterministic_simplified_json_export()

    print("\n22. URL namespace support...")
    test_url_namespace()

    print("\n" + "=" * 70)
    print("All tests completed!")
    print("=" * 70)
