"""
Minimal test for batch_trees_to_tiles demonstrating nested structure conversion.

Equivalent to the cross-model traversal test's group.members[0].name.forenames[0].forename
but using the batch conversion API directly.
"""

import json
import os
import pytest

import alizarin


# Test data paths
TEST_DATA_DIR = os.path.join(os.path.dirname(__file__), '../../../tests/data')
MODELS_DIR = os.path.join(TEST_DATA_DIR, 'models')


def load_graph(name: str) -> dict:
    """Load a graph model from the test data directory."""
    path = os.path.join(MODELS_DIR, f'{name}.json')
    with open(path) as f:
        data = json.load(f)
    # Handle wrapped format
    if 'graph' in data and isinstance(data['graph'], list):
        return data['graph'][0]
    return data


def test_batch_trees_to_tiles_nested_name():
    """
    Test batch_trees_to_tiles with nested structure: name.forenames[0].forename

    This mirrors the semantic path: group.members[0].name.forenames[0].forename
    but tests just the Person model's tree-to-tiles conversion.

    Note: The nested semantic structure (name -> forenames -> forename) is
    flattened during conversion. Only nodes with actual data create tiles.
    String values are wrapped in localized format: {"en": "Alice"}
    """
    # Load and register Person graph
    person_graph = load_graph('Person')
    graph_json = json.dumps(person_graph)
    graph_id = alizarin.register_graph(graph_json)

    # Create a tree with nested name structure
    # Structure: name -> forenames[] -> forename
    trees = [
        {
            "name": [{
                "forenames": [{
                    "forename": "Alice"
                }]
            }]
        }
    ]

    trees_json = json.dumps(trees)

    # Convert tree to tiles
    result = alizarin.batch_trees_to_tiles(
        trees_json=trees_json,
        graph_id=graph_id,
        from_camel=False,
        strict=True
    )

    # Verify result structure
    assert 'business_data' in result, "Should have business_data"
    assert 'resources' in result['business_data'], "Should have resources"

    resources = result['business_data']['resources']
    assert len(resources) == 1, "Should have 1 resource"

    resource = resources[0]
    assert 'tiles' in resource, "Resource should have tiles"
    assert 'resourceinstance' in resource, "Resource should have resourceinstance"

    tiles = resource['tiles']
    assert len(tiles) > 0, "Should have at least one tile"

    # Find the forename tile (the one with actual data)
    # Note: String values are wrapped in localized format {"en": "value"}
    forename_tile = None
    for tile in tiles:
        data = tile.get('data', {})
        for node_id, value in data.items():
            if value == "Alice":
                forename_tile = tile
                break
            # String datatype wraps value in localized format
            if isinstance(value, dict) and value.get('en') == "Alice":
                forename_tile = tile
                break

    assert forename_tile is not None, "Should find tile with 'Alice' value"

    # Verify tile structure
    assert 'tileid' in forename_tile, "Tile should have tileid"
    assert 'nodegroup_id' in forename_tile, "Tile should have nodegroup_id"
    assert 'resourceinstance_id' in forename_tile, "Tile should have resourceinstance_id"

    print(f"Successfully converted tree to {len(tiles)} tiles")
    print(f"Forename tile: {json.dumps(forename_tile, indent=2)}")


def test_batch_trees_to_tiles_group_members():
    """
    Test batch_trees_to_tiles for Group with members (resource-instance-list).

    This creates a Group resource with a members field pointing to a Person resource,
    demonstrating the relationship that enables cross-model traversal.
    """
    # Load and register both graphs
    group_graph = load_graph('Group')
    person_graph = load_graph('Person')

    group_graph_id = alizarin.register_graph(json.dumps(group_graph))
    person_graph_id = alizarin.register_graph(json.dumps(person_graph))

    # First create a Person resource
    person_trees = [
        {
            "name": [{
                "forenames": [{
                    "forename": "Alice"
                }]
            }]
        }
    ]

    person_result = alizarin.batch_trees_to_tiles(
        trees_json=json.dumps(person_trees),
        graph_id=person_graph_id,
        strict=True
    )

    person_resource = person_result['business_data']['resources'][0]
    person_id = person_resource['resourceinstance']['resourceinstanceid']

    # Now create a Group that references the Person
    # The members field is a resource-instance-list
    group_trees = [
        {
            "members": [
                {
                    "resourceId": person_id,
                    "ontologyProperty": "",
                    "inverseOntologyProperty": ""
                }
            ]
        }
    ]

    group_result = alizarin.batch_trees_to_tiles(
        trees_json=json.dumps(group_trees),
        graph_id=group_graph_id,
        strict=True
    )

    # Verify Group result
    assert 'business_data' in group_result
    group_resources = group_result['business_data']['resources']
    assert len(group_resources) == 1

    group_resource = group_resources[0]
    group_tiles = group_resource['tiles']

    # Find the members tile
    members_tile = None
    for tile in group_tiles:
        data = tile.get('data', {})
        for node_id, value in data.items():
            if isinstance(value, list) and len(value) > 0:
                if isinstance(value[0], dict) and value[0].get('resourceId') == person_id:
                    members_tile = tile
                    break

    assert members_tile is not None, "Should find members tile with person reference"

    print(f"Group resource created with {len(group_tiles)} tiles")
    print(f"Members tile references Person: {person_id}")


def test_cross_model_with_registry():
    """
    Full example: Load Person resources, then create Groups referencing them.

    This demonstrates:
    1. Creating Person resources with tiles via batch_trees_to_tiles
    2. Registering those resources in a ResourceRegistry
    3. Creating Group resources that reference the Persons
    4. Using populate_caches to resolve the relationships
    """
    # Load and register both graphs
    group_graph = load_graph('Group')
    person_graph = load_graph('Person')

    group_graph_id = alizarin.register_graph(json.dumps(group_graph))
    person_graph_id = alizarin.register_graph(json.dumps(person_graph))

    # Step 1: Create multiple Person resources
    person_trees = [
        {
            "name": [{
                "forenames": [{
                    "forename": "Alice"
                }]
            }]
        },
        {
            "name": [{
                "forenames": [{
                    "forename": "Bob"
                }]
            }]
        },
        {
            "name": [{
                "forenames": [{
                    "forename": "Charlie"
                }]
            }]
        }
    ]

    person_result = alizarin.batch_trees_to_tiles(
        trees_json=json.dumps(person_trees),
        graph_id=person_graph_id,
        strict=True
    )

    person_resources = person_result['business_data']['resources']
    print(f"Created {len(person_resources)} Person resources:")
    for p in person_resources:
        print(f"  - {p['resourceinstance']['resourceinstanceid']}")

    # Step 2: Register Person resources in a ResourceRegistry
    # This enables relationship resolution when creating Groups
    registry = alizarin.ResourceRegistry()
    registry.merge_from_resources(
        json.dumps(person_resources),
        store_full=True,  # Store full resources for traversal
        include_caches=True
    )
    print(f"\nRegistry now contains {len(registry)} resources")

    # Get person IDs for referencing
    alice_id = person_resources[0]['resourceinstance']['resourceinstanceid']
    bob_id = person_resources[1]['resourceinstance']['resourceinstanceid']
    charlie_id = person_resources[2]['resourceinstance']['resourceinstanceid']

    # Step 3: Create Group resources that reference Persons
    # Note: Group model uses 'name' for group name, 'members' for member list
    group_trees = [
        {
            # Group 1: Alice and Bob
            "members": [
                {"resourceId": alice_id, "ontologyProperty": "", "inverseOntologyProperty": ""},
                {"resourceId": bob_id, "ontologyProperty": "", "inverseOntologyProperty": ""}
            ]
        },
        {
            # Group 2: Bob and Charlie
            "members": [
                {"resourceId": bob_id, "ontologyProperty": "", "inverseOntologyProperty": ""},
                {"resourceId": charlie_id, "ontologyProperty": "", "inverseOntologyProperty": ""}
            ]
        }
    ]

    group_result = alizarin.batch_trees_to_tiles(
        trees_json=json.dumps(group_trees),
        graph_id=group_graph_id,
        strict=True
    )

    group_resources = group_result['business_data']['resources']
    print(f"\nCreated {len(group_resources)} Group resources:")

    # Step 4: Use populate_caches to resolve member references
    # This adds __cache with related resource summaries
    cache_result = registry.populate_caches(
        json.dumps(group_resources),
        graph_id=group_graph_id,
        enrich_relationships=True
    )

    enriched_groups = cache_result['resources']
    unknown_refs = cache_result['unknown_references']

    print(f"Unknown references: {len(unknown_refs)}")

    def extract_related_ids(related_cache):
        """Extract resource IDs from cache structure {tileId: {nodeId: entry}}.

        Handles both single entries (resource-instance) and list entries (resource-instance-list).
        - Single: {datatype: "resource-instance", id, type, graphId, title}
        - List: {datatype: "resource-instance-list", _: [...], meta}
        """
        ids = set()
        if isinstance(related_cache, dict):
            for tile_id, nodes in related_cache.items():
                if isinstance(nodes, dict):
                    for node_id, entry in nodes.items():
                        if isinstance(entry, dict):
                            if 'id' in entry:
                                # Single entry (resource-instance)
                                ids.add(entry['id'])
                            elif '_' in entry and isinstance(entry['_'], list):
                                # List entry (resource-instance-list)
                                for sub_entry in entry['_']:
                                    if isinstance(sub_entry, dict) and 'id' in sub_entry:
                                        ids.add(sub_entry['id'])
        return ids

    for i, group in enumerate(enriched_groups):
        group_id = group['resourceinstance']['resourceinstanceid']
        # __cache now directly contains {tileId: {nodeId: entry}} (no relatedResources wrapper)
        cache = group.get('__cache', {})
        related_ids = extract_related_ids(cache)
        print(f"\nGroup {i+1}: {group_id}")
        print(f"  Cache keys: {list(cache.keys())}")
        print(f"  Related resources in cache: {len(related_ids)}")
        for tile_id, nodes in cache.items() if isinstance(cache, dict) else []:
            for node_id, entry in nodes.items() if isinstance(nodes, dict) else []:
                if isinstance(entry, dict):
                    print(f"    - {entry.get('id')}: {entry.get('title', '(no name)')}")

    # Verify the relationships were resolved
    assert len(enriched_groups) == 2, "Should have 2 groups"

    # Check that Group 1 has related resources in cache
    # __cache now directly contains {tileId: {nodeId: entry}} (no relatedResources wrapper)
    group1_cache = enriched_groups[0].get('__cache', {})
    print(f"\n  Cache type: {type(group1_cache)}")
    team_alpha_member_ids = extract_related_ids(group1_cache)
    assert len(team_alpha_member_ids) > 0, "Group 1 should have related resources in cache"

    print(f"  Group 1 member IDs in cache: {team_alpha_member_ids}")
    print(f"  Expected: alice={alice_id}, bob={bob_id}")

    print("\n✓ Cross-model relationship resolution successful!")


def test_tiles_to_trees_then_cross_model():
    """
    Alternative flow: Start with existing tiles, convert to trees, then reference.

    This shows:
    1. Starting with tile data (as you'd get from Arches API)
    2. Converting tiles to trees for inspection/modification
    3. Creating new resources that reference the originals
    """
    # Load graphs
    person_graph = load_graph('Person')
    group_graph = load_graph('Group')

    person_graph_id = alizarin.register_graph(json.dumps(person_graph))
    group_graph_id = alizarin.register_graph(json.dumps(group_graph))

    # Simulate existing Person resources with tiles (as from database)
    # In practice, these would come from Arches API
    existing_person_id = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"

    # Find the forename node ID from the graph
    forename_node = None
    forename_nodegroup = None
    for node in person_graph.get('nodes', []):
        if node.get('alias') == 'forename':
            forename_node = node['nodeid']
            forename_nodegroup = node['nodegroup_id']
            break

    existing_person = {
        "resourceinstance": {
            "resourceinstanceid": existing_person_id,
            "graph_id": person_graph_id,
            "name": "Existing Person",
            "descriptors": {"name": "Diana"}
        },
        "tiles": [
            {
                "tileid": "11111111-2222-3333-4444-555555555555",
                "nodegroup_id": forename_nodegroup,
                "resourceinstance_id": existing_person_id,
                "parenttile_id": None,
                "sortorder": 0,
                "data": {
                    forename_node: {"en": "Diana"}
                }
            }
        ]
    }

    print(f"Existing Person resource: {existing_person_id}")
    print(f"  Forename tile data: {existing_person['tiles'][0]['data']}")

    # Register the existing resource
    registry = alizarin.ResourceRegistry()
    registry.merge_from_resources(
        json.dumps([existing_person]),
        store_full=True,
        include_caches=True
    )

    # Convert tiles to tree to see the semantic structure
    # Note: graph_id is extracted from each resource's resourceinstance.graph_id
    tree_result = alizarin.batch_tiles_to_trees(
        resources_json=json.dumps([existing_person]),
        strict=True
    )

    print(f"\nPerson as tree (type={type(tree_result)}):")
    if isinstance(tree_result, list):
        print(json.dumps(tree_result[0], indent=2)[:500] + "...")
    elif isinstance(tree_result, dict):
        # batch_tiles_to_trees returns {trees: [...], errors: [...]}
        trees = tree_result.get('trees', [])
        if trees:
            print(json.dumps(trees[0], indent=2)[:500] + "...")
        else:
            print(f"  Result: {tree_result}")
    else:
        print(f"  Unexpected type: {tree_result}")

    # Now create a Group that references this Person
    group_tree = [
        {
            "members": [
                {
                    "resourceId": existing_person_id,
                    "ontologyProperty": "",
                    "inverseOntologyProperty": ""
                }
            ]
        }
    ]

    group_result = alizarin.batch_trees_to_tiles(
        trees_json=json.dumps(group_tree),
        graph_id=group_graph_id,
        strict=True
    )

    group_resource = group_result['business_data']['resources'][0]
    group_id = group_resource['resourceinstance']['resourceinstanceid']

    print(f"\nCreated Group: {group_id}")
    print(f"  References Person: {existing_person_id}")

    # Populate cache to resolve the reference
    cache_result = registry.populate_caches(
        json.dumps([group_resource]),
        graph_id=group_graph_id,
        enrich_relationships=True
    )

    enriched_group = cache_result['resources'][0]
    # __cache now directly contains {tileId: {nodeId: entry}} (no relatedResources wrapper)
    cache = enriched_group.get('__cache', {})

    # Helper to extract IDs from cache structure {tileId: {nodeId: entry}}
    # Handles both single entries and list entries
    def extract_related_ids_local(cache_data):
        ids = set()
        if isinstance(cache_data, dict):
            for tile_id, nodes in cache_data.items():
                if isinstance(nodes, dict):
                    for node_id, entry in nodes.items():
                        if isinstance(entry, dict):
                            if 'id' in entry:
                                ids.add(entry['id'])
                            elif '_' in entry and isinstance(entry['_'], list):
                                for sub_entry in entry['_']:
                                    if isinstance(sub_entry, dict) and 'id' in sub_entry:
                                        ids.add(sub_entry['id'])
        return ids

    related_ids = extract_related_ids_local(cache)
    print(f"  Resolved references: {len(related_ids)}")

    for tile_id, nodes in cache.items() if isinstance(cache, dict) else []:
        for node_id, entry in nodes.items() if isinstance(nodes, dict) else []:
            if isinstance(entry, dict):
                print(f"    - {entry.get('id')}: graph={entry.get('graphId')}")

    assert len(related_ids) == 1, "Should resolve 1 related resource"
    assert existing_person_id in related_ids, f"Should have {existing_person_id} in related"

    print("\n✓ Tiles-to-trees-to-cross-model flow successful!")


def test_node_values_index():
    """
    Test get_node_values_index and get_value_to_resources_index functions.

    These allow efficient lookup of node values across resources without
    JSON serialization overhead.
    """
    # Load and register Person graph
    person_graph = load_graph('Person')
    graph_json = json.dumps(person_graph)
    graph_id = alizarin.register_graph(graph_json)

    # Create multiple Person resources with different forenames
    person_trees = [
        {"name": [{"forenames": [{"forename": "Alice"}]}]},
        {"name": [{"forenames": [{"forename": "Bob"}]}]},
        {"name": [{"forenames": [{"forename": "Alice"}]}]},  # Duplicate forename
    ]

    result = alizarin.batch_trees_to_tiles(
        trees_json=json.dumps(person_trees),
        graph_id=graph_id,
        strict=True
    )

    resources = result['business_data']['resources']
    assert len(resources) == 3

    # Register resources in registry
    registry = alizarin.ResourceRegistry()
    registry.merge_from_resources(
        json.dumps(resources),
        store_full=True,
        include_caches=True
    )

    # Test get_node_values_index - maps resource IDs to values
    node_values_index = registry.get_node_values_index(graph_id, 'forename')
    print(f"Node values index: {node_values_index}")

    assert len(node_values_index) == 3, "Should have 3 resources in index"

    # Each resource should have one forename value
    for resource_id, values in node_values_index.items():
        assert len(values) >= 1, f"Resource {resource_id} should have at least 1 value"
        # Value is localized format {"en": "Alice"}
        first_value = values[0]
        assert isinstance(first_value, dict), "Value should be localized dict"
        assert 'en' in first_value, "Value should have 'en' key"

    # Test get_value_to_resources_index - maps values to resource IDs
    value_to_resources = registry.get_value_to_resources_index(
        graph_id, 'forename', flatten_localized=True
    )
    print(f"Value to resources index: {value_to_resources}")

    assert 'Alice' in value_to_resources, "Should have 'Alice' as key"
    assert 'Bob' in value_to_resources, "Should have 'Bob' as key"

    # Two resources have 'Alice' as forename
    assert len(value_to_resources['Alice']) == 2, "Two resources should have 'Alice'"
    assert len(value_to_resources['Bob']) == 1, "One resource should have 'Bob'"

    print("✓ Node values indexing works correctly!")


if __name__ == '__main__':
    print("=" * 60)
    print("Running test_batch_trees_to_tiles_nested_name...")
    print("=" * 60)
    test_batch_trees_to_tiles_nested_name()

    print("\n" + "=" * 60)
    print("Running test_batch_trees_to_tiles_group_members...")
    print("=" * 60)
    test_batch_trees_to_tiles_group_members()

    print("\n" + "=" * 60)
    print("Running test_cross_model_with_registry...")
    print("=" * 60)
    test_cross_model_with_registry()

    print("\n" + "=" * 60)
    print("Running test_tiles_to_trees_then_cross_model...")
    print("=" * 60)
    test_tiles_to_trees_then_cross_model()

    print("\n" + "=" * 60)
    print("Running test_node_values_index...")
    print("=" * 60)
    test_node_values_index()

    print("\n" + "=" * 60)
    print("All tests passed!")
    print("=" * 60)
