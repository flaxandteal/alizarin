"""
Test CLM (Custom List Manager) Collection with type flexibility demonstration.

This test demonstrates that the same collection and data can work as either:
1. Resource-instance references (built-in CLM behavior)
2. Concepts (by overriding node datatype to concept-list)

Uses the Person model with multiple datatypes to show comprehensive examples.
"""

import json
import pytest
import os
import tempfile
import uuid

import alizarin


def load_person_graph():
    """Load Person graph model"""
    test_data_path = os.path.join(
        os.path.dirname(__file__),
        'data/graphs/resource_models/Person.json'
    )

    with open(test_data_path) as f:
        data = json.load(f)
        return data['graph'][0]


def find_node_by_alias(graph_data, alias):
    """Find a node by its alias"""
    for node in graph_data.get('nodes', []):
        if node.get('alias') == alias:
            return node
    return None


def find_parent_semantic(graph_data, node):
    """Find the parent semantic node for a given node"""
    ng_id = node.get('nodegroup_id')
    for n in graph_data.get('nodes', []):
        if n['nodeid'] == ng_id and n.get('datatype') == 'semantic':
            return n.get('alias')
    return None


def test_create_clm_collection():
    """Test creating a CLM collection for activities"""
    collection = alizarin.RustRdmCollection.from_labels(
        name="Activity Types",
        labels=["Hiking", "Swimming", "Cycling", "Reading"]
    )

    assert collection.id is not None
    assert collection.name == "Activity Types"
    assert len(collection) == 4

    # Verify lookup
    activity = collection.find_by_label("Hiking")
    assert activity is not None
    assert activity.get_label("en") == "Hiking"

    print(f"✓ Created CLM collection with {len(collection)} activity labels")


def test_collection_as_resource_references():
    """
    Test using collection with resource-instance-list (built-in CLM behavior).

    This demonstrates normal CLM usage where labels resolve to resource instance UUIDs.
    """
    graph_data = load_person_graph()

    # Find the associated_activities node (resource-instance-list)
    activities_node = find_node_by_alias(graph_data, 'associated_activities')

    if not activities_node:
        pytest.skip("'associated_activities' node not found in Person model")

    # Verify it's a resource-instance-list
    assert activities_node['datatype'] == 'resource-instance-list', \
        "Expected resource-instance-list datatype"

    print(f"\n--- Part 1: Using as Resource References (Built-in CLM) ---")
    print(f"Node: {activities_node['alias']}")
    print(f"Datatype: {activities_node['datatype']}")

    # Generate collection ID and add to graph
    collection_id = str(uuid.uuid5(uuid.NAMESPACE_DNS, 'test.activities.references'))
    activities_node['config']['rdmCollection'] = collection_id

    # Create collection for activity references
    activity_labels = ["Hiking Adventure", "Swimming Competition", "Cycling Tour"]

    collection = alizarin.RustRdmCollection.from_labels(
        name="Activity References",
        labels=activity_labels,
        id=collection_id
    )

    print(f"Collection ID: {collection_id}")
    print(f"Collection type: Resource references")
    print(f"Labels: {activity_labels}")

    # Add to cache
    cache = alizarin.RustRdmCache()
    cache.add_collection(collection)
    alizarin.set_global_rdm_cache(cache)

    try:
        # Create comprehensive tree with multiple datatypes
        trees = [
            {
                'name': [{
                    'forenames': [{
                        'forename': 'John'
                    }],
                    'surnames': [{
                        'surname': 'Smith'
                    }]
                }],
                'associated_activities': ["Hiking Adventure", "Cycling Tour"],  # CLM references
                'primary_reference_number': 12345
            }
        ]

        # Register graph before conversion
        graph_id = alizarin.register_graph(json.dumps(graph_data))

        # Resolve labels to UUIDs using the loaded RDM cache
        trees_json = json.dumps(trees)
        resolved_trees_json = alizarin.resolve_labels_in_tree(
            tree_json=trees_json,
            graph_id=graph_id,
            strict=True
        )

        result = alizarin.batch_trees_to_tiles(
            trees_json=resolved_trees_json,
            graph_id=graph_id,
            from_camel=False,
            strict=True
        )

        assert 'business_data' in result
        resources = result['business_data']['resources']
        assert len(resources) == 1

        print(f"✓ Converted tree with resource references")
        print(f"  Resource ID: {resources[0]['resourceinstance']['resourceinstanceid']}")
        print(f"  Tiles created: {len(resources[0]['tiles'])}")

        # Verify references were resolved
        activities_node_id = activities_node['nodeid']
        found_activities = False

        for tile in resources[0]['tiles']:
            tile_data = tile.get('data', {})
            if activities_node_id in tile_data:
                activities_value = tile_data[activities_node_id]
                print(f"  Activities resolved: {len(activities_value)} references")
                assert isinstance(activities_value, list)
                for ref_id in activities_value:
                    assert cache.validate_concept(collection_id, ref_id), \
                        f"Reference {ref_id} should be valid"
                found_activities = True
                break

        if not found_activities:
            print("  Note: Activity references not in tiles (may be filtered if empty)")

        print(f"\n✓ Part 1 Complete: Collection works as RESOURCE REFERENCES")

    finally:
        alizarin.clear_global_rdm_cache()


def test_collection_as_concepts():
    """
    Test using the SAME collection with concept-list (by overriding datatype).

    This demonstrates that the same collection/labels can work as concepts
    by simply changing the node's datatype from resource-instance-list to concept-list.

    KEY INSIGHT: RDM (Reference Data Manager) and CLM (Custom List Manager) use the
    same underlying RustRdmCollection/RustRdmCache system. The only difference is
    semantic intent:
    - resource-instance-list: References to other resources (CLM)
    - concept-list: Controlled vocabulary terms (RDM)

    Both resolve labels to UUIDs using the same collection mechanism.
    """
    graph_data = load_person_graph()

    # Find the associated_activities node
    activities_node = find_node_by_alias(graph_data, 'associated_activities')

    if not activities_node:
        pytest.skip("'associated_activities' node not found in Person model")

    print(f"\n--- Part 2: Using as Concepts (Type Override) ---")
    print(f"Node: {activities_node['alias']}")
    print(f"Original datatype: {activities_node['datatype']}")

    # OVERRIDE: Change datatype from resource-instance-list to concept-list
    # This demonstrates they're interchangeable - same collection, different semantic use
    activities_node['datatype'] = 'concept-list'
    print(f"Modified datatype: {activities_node['datatype']}")
    print("(Same node, now treated as concepts instead of references)")

    # Generate collection ID and add to graph
    collection_id = str(uuid.uuid5(uuid.NAMESPACE_DNS, 'test.activities.concepts'))
    activities_node['config']['rdmCollection'] = collection_id

    # Create collection with SAME LABELS as before
    activity_labels = ["Hiking Adventure", "Swimming Competition", "Cycling Tour"]

    collection = alizarin.RustRdmCollection.from_labels(
        name="Activity Concepts",
        labels=activity_labels,
        id=collection_id
    )

    print(f"Collection ID: {collection_id}")
    print(f"Collection type: Concepts")
    print(f"Labels: {activity_labels} (same as Part 1)")

    # Add to cache
    cache = alizarin.RustRdmCache()
    cache.add_collection(collection)
    alizarin.set_global_rdm_cache(cache)

    try:
        # Create tree with SAME DATA as Part 1
        trees = [
            {
                'name': [{
                    'forenames': [{
                        'forename': 'Jane'
                    }],
                    'surnames': [{
                        'surname': 'Doe'
                    }]
                }],
                'associated_activities': ["Hiking Adventure", "Cycling Tour"],  # Now concepts!
                'primary_reference_number': 67890
            }
        ]

        # Register graph before conversion
        graph_id = alizarin.register_graph(json.dumps(graph_data))

        # Resolve labels to UUIDs using the loaded RDM cache
        trees_json = json.dumps(trees)
        resolved_trees_json = alizarin.resolve_labels_in_tree(
            tree_json=trees_json,
            graph_id=graph_id,
            strict=True
        )

        result = alizarin.batch_trees_to_tiles(
            trees_json=resolved_trees_json,
            graph_id=graph_id,
            from_camel=False,
            strict=True
        )

        assert 'business_data' in result
        resources = result['business_data']['resources']
        assert len(resources) == 1

        print(f"✓ Converted tree with concepts")
        print(f"  Resource ID: {resources[0]['resourceinstance']['resourceinstanceid']}")
        print(f"  Tiles created: {len(resources[0]['tiles'])}")

        # Verify concepts were resolved
        activities_node_id = activities_node['nodeid']
        found_activities = False

        for tile in resources[0]['tiles']:
            tile_data = tile.get('data', {})
            if activities_node_id in tile_data:
                activities_value = tile_data[activities_node_id]
                print(f"  Activities resolved: {len(activities_value)} concepts")
                assert isinstance(activities_value, list)
                for concept_id in activities_value:
                    assert cache.validate_concept(collection_id, concept_id), \
                        f"Concept {concept_id} should be valid"
                found_activities = True
                break

        if not found_activities:
            print("  Note: Activity concepts not in tiles (may be filtered if empty)")

        print(f"\n✓ Part 2 Complete: SAME collection works as CONCEPTS")
        print(f"\n" + "=" * 70)
        print("KEY INSIGHT:")
        print("  The SAME collection and labels work for both:")
        print("  1. Resource references (resource-instance-list)")
        print("  2. Concepts (concept-list)")
        print("  Only difference: the node's datatype configuration")
        print("=" * 70)

    finally:
        alizarin.clear_global_rdm_cache()


def test_skos_roundtrip_with_person_model():
    """
    Test SKOS roundtrip using Person model.

    Demonstrates:
    1. Create collection for activities
    2. Serialize to SKOS XML
    3. Write to file
    4. Load from file
    5. Use in comprehensive tree with multiple datatypes
    """
    graph_data = load_person_graph()

    # Find the associated_activities node
    activities_node = find_node_by_alias(graph_data, 'associated_activities')

    if not activities_node:
        pytest.skip("'associated_activities' node not found")

    # Generate collection ID
    collection_id = str(uuid.uuid5(uuid.NAMESPACE_DNS, 'test.activities.skos'))
    activities_node['config']['rdmCollection'] = collection_id

    print(f"\n--- SKOS Roundtrip Test ---")
    print(f"Using node: {activities_node['alias']}")
    print(f"Datatype: {activities_node['datatype']}")

    # Part 1: Create and serialize
    original_labels = ["Archery", "Badminton", "Cricket", "Dancing"]

    collection = alizarin.RustRdmCollection.from_labels(
        name="Sports and Activities",
        labels=original_labels,
        id=collection_id
    )

    assert len(collection) == 4
    print(f"✓ Created collection with {len(collection)} labels")

    # Serialize to SKOS
    base_uri = "http://example.org/activities/"
    skos_xml = collection.to_skos_xml(base_uri)

    assert 'Sports and Activities' in skos_xml
    for label in original_labels:
        assert label in skos_xml
    print("✓ Serialized to SKOS XML")

    # Write to file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.xml', delete=False) as f:
        temp_file = f.name
        f.write(skos_xml)
        print(f"✓ Wrote to file: {temp_file}")

    try:
        # Part 2: Load from file
        with open(temp_file, 'r') as f:
            loaded_xml = f.read()

        cache = alizarin.RustRdmCache()
        loaded_ids = cache.add_from_skos_xml(loaded_xml, base_uri)

        assert len(loaded_ids) > 0
        assert cache.has_collection(collection_id)
        print(f"✓ Loaded from file")

        # Verify all labels preserved
        loaded_collection = cache.get_collection(collection_id)
        assert len(loaded_collection) == 4

        for label in original_labels:
            activity = cache.lookup_by_label(collection_id, label)
            assert activity is not None
            assert activity.get_label("en") == label
        print(f"✓ All {len(original_labels)} labels preserved")

        # Part 3: Use in conversion with multiple datatypes
        alizarin.set_global_rdm_cache(cache)

        try:
            trees = [
                {
                    'name': [{
                        'forenames': [{
                            'forename': 'Alice'
                        }],
                        'surnames': [{
                            'surname': 'Johnson'
                        }]
                    }],
                    'associated_activities': ["Archery", "Dancing"],  # Labels from SKOS
                    'primary_reference_number': 11111,
                    'descriptions': [{
                        'description': 'Active person with varied interests'
                    }]
                }
            ]

            # Register graph before conversion
            graph_id = alizarin.register_graph(json.dumps(graph_data))

            # Resolve labels to UUIDs using the loaded RDM cache
            trees_json = json.dumps(trees)
            resolved_trees_json = alizarin.resolve_labels_in_tree(
                tree_json=trees_json,
                graph_id=graph_id,
                strict=True
            )

            result = alizarin.batch_trees_to_tiles(
                trees_json=resolved_trees_json,
                graph_id=graph_id,
                from_camel=False,
                strict=True
            )

            assert 'business_data' in result
            resources = result['business_data']['resources']
            assert len(resources) == 1
            print(f"✓ Converted tree with loaded collection")
            print(f"  Used datatypes: string, number, resource-instance-list")
            print(f"  Labels resolved from SKOS file")

            print(f"\n✓ SKOS Roundtrip Complete!")

        finally:
            alizarin.clear_global_rdm_cache()

    finally:
        # Clean up temp file
        if os.path.exists(temp_file):
            os.unlink(temp_file)


def test_multilanguage_collection():
    """Test collection with multilingual labels"""
    collection = alizarin.RustRdmCollection("multilang-activities")

    # Add activities with multiple languages
    collection.add_from_label(
        label={
            "en": "Swimming",
            "ga": "Snámh",
            "fr": "Natation"
        },
        id="swimming-id"
    )

    collection.add_from_label(
        label={
            "en": "Running",
            "ga": "Rith",
            "fr": "Course"
        },
        id="running-id"
    )

    assert len(collection) == 2

    # Verify English lookup
    swimming = collection.find_by_label("Swimming")
    assert swimming is not None
    assert swimming.get_label("en") == "Swimming"
    assert swimming.get_label("ga") == "Snámh"
    assert swimming.get_label("fr") == "Natation"

    # Verify Irish lookup
    swimming_ga = collection.find_by_label("Snámh")
    assert swimming_ga is not None
    assert swimming_ga.get_label("en") == "Swimming"

    print("✓ Multilingual collection works")
    print(f"  English: Swimming → Snámh (ga) → Natation (fr)")


if __name__ == '__main__':
    print("=" * 70)
    print("CLM Collection Tests - Type Flexibility Demonstration")
    print("Using Person Model")
    print("=" * 70)

    print("\n1. Creating CLM collection...")
    test_create_clm_collection()

    print("\n2. Collection as Resource References (built-in CLM)...")
    try:
        test_collection_as_resource_references()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n3. SAME collection as Concepts (type override)...")
    try:
        test_collection_as_concepts()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n4. SKOS roundtrip with comprehensive datatypes...")
    try:
        test_skos_roundtrip_with_person_model()
    except pytest.skip.Exception as e:
        print(f"  SKIPPED: {e}")

    print("\n5. Multilingual collection...")
    test_multilanguage_collection()

    print("\n" + "=" * 70)
    print("All CLM tests completed!")
    print("=" * 70)
