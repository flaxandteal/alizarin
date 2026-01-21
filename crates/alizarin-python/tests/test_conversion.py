"""
Python tests for Alizarin tree conversion bindings

These tests verify that the PyO3 bindings correctly expose the Rust
tree conversion functionality to Python.
"""

import json
import pytest
import os
import sys

# Import the compiled Alizarin module
import alizarin


def load_test_data():
    """Load test graph model from Group.json"""
    test_data_path = os.path.join(
        os.path.dirname(__file__),
        '../../../tests/data/models/Group.json'
    )

    with open(test_data_path) as f:
        data = json.load(f)
        return data['graph'][0]


def create_test_tiles(graph_data):
    """Create test tiles matching Rust test data"""
    graph_id = graph_data['graphid']

    # Find nodegroup IDs from the graph
    # Looking for basic_info nodegroup
    basic_info_ng = None
    for nodegroup in graph_data.get('nodegroups', []):
        # Find the nodegroup that contains the name node
        for node in graph_data.get('nodes', []):
            if (node.get('nodegroup_id') == nodegroup['nodegroupid'] and
                node.get('alias') == 'name'):
                basic_info_ng = nodegroup['nodegroupid']
                break
        if basic_info_ng:
            break

    if not basic_info_ng:
        # Fallback: just use first nodegroup if we can't find name
        basic_info_ng = graph_data['nodegroups'][0]['nodegroupid']

    # Find node IDs
    name_node_id = None
    desc_node_id = None
    for node in graph_data.get('nodes', []):
        if node.get('alias') == 'name':
            name_node_id = node['nodeid']
        elif node.get('alias') == 'description':
            desc_node_id = node['nodeid']

    tiles = [{
        'tileid': 'tile-001',
        'nodegroup_id': basic_info_ng,
        'resourceinstance_id': 'test-resource-123',
        'data': {}
    }]

    # Add data if we found the nodes
    if name_node_id:
        tiles[0]['data'][name_node_id] = {
            'en': 'Test Group Name',
            'ga': 'Ainm Grúpa Tástála'
        }

    if desc_node_id:
        tiles[0]['data'][desc_node_id] = 'A test group description'

    return tiles


def test_tiles_to_tree_basic():
    """Test converting tiles to tree structure"""
    graph_data = load_test_data()
    tiles = create_test_tiles(graph_data)
    graph_json = json.dumps(graph_data)

    # Register graph first (required by new API)
    graph_id = alizarin.register_graph(graph_json)

    # Create full resource JSON (new API)
    resource = {
        'resourceinstanceid': 'test-resource-123',
        'graph_id': graph_id,
        'tiles': tiles,
    }
    resource_json = json.dumps(resource)

    # Call the binding with single resource JSON
    result = alizarin.tiles_to_json_tree(resource_json)

    # Verify result structure
    assert isinstance(result, dict), "Result should be a dictionary"
    assert result['resourceinstanceid'] == 'test-resource-123'
    assert result['graph_id'] == graph_data['graphid']

    print(f"Tree structure: {json.dumps(result, indent=2)}")


def test_tree_to_tiles_basic():
    """Test converting tree structure to tiles - returns BusinessDataWrapper format"""
    graph_data = load_test_data()

    # Create a simple tree structure (no metadata, just content)
    # Note: description is in statement nodegroup, not basic_info
    tree = {
        'basic_info': [{
            'name': {
                'en': 'JSON Test Group',
                'ga': 'Grúpa Tástála JSON'
            }
        }],
        'statement': [{
            'description': 'Created from JSON tree'
        }]
    }

    tree_json = json.dumps(tree)
    graph_json = json.dumps(graph_data)

    # Register graph first (required by new API)
    graph_id = alizarin.register_graph(graph_json)

    # Call the binding with resource_id and graph_id as parameters
    result = alizarin.json_tree_to_tiles(
        tree_json=tree_json,
        resource_id='test-resource-456',
        graph_id=graph_id
    )

    # Verify result is BusinessDataWrapper format
    assert isinstance(result, dict), "Result should be a dictionary"
    assert 'business_data' in result, "Result should have business_data"
    assert 'resources' in result['business_data'], "business_data should have resources"

    resources = result['business_data']['resources']
    assert len(resources) == 1, "Should have one resource"

    resource = resources[0]
    assert 'resourceinstance' in resource, "Resource should have resourceinstance"
    assert 'tiles' in resource, "Resource should have tiles"

    ri = resource['resourceinstance']
    assert ri['resourceinstanceid'] == 'test-resource-456'
    assert ri['graph_id'] == graph_data['graphid']
    assert 'descriptors' in ri, "resourceinstance should have descriptors"

    assert isinstance(resource['tiles'], list)
    assert len(resource['tiles']) > 0, "Should have created at least one tile"

    print(f"Created {len(resource['tiles'])} tiles")
    print(f"Descriptors: {ri.get('descriptors', {})}")
    for tile in resource['tiles']:
        print(f"  Tile {tile.get('tileid', '(no id)')} "
              f"has {len(tile.get('data', {}))} data entries")


def test_roundtrip_preserves_data():
    """Test that tiles -> tree -> tiles preserves data"""
    graph_data = load_test_data()
    original_tiles = create_test_tiles(graph_data)
    graph_json = json.dumps(graph_data)

    # Register graph first (required by new API)
    graph_id = alizarin.register_graph(graph_json)

    # Create full resource JSON (new API)
    resource = {
        'resourceinstanceid': 'test-resource-123',
        'graph_id': graph_id,
        'tiles': original_tiles,
    }

    # tiles -> tree
    tree = alizarin.tiles_to_json_tree(json.dumps(resource))

    print(f"Intermediate tree: {json.dumps(tree, indent=2)}")

    # tree -> tiles (returns BusinessDataWrapper format)
    tree_json = json.dumps(tree)
    result = alizarin.json_tree_to_tiles(
        tree_json=tree_json,
        resource_id='test-resource-123',
        graph_id=graph_id
    )

    # Verify BusinessDataWrapper structure
    assert 'business_data' in result
    assert 'resources' in result['business_data']
    resources = result['business_data']['resources']
    assert len(resources) == 1

    resource = resources[0]
    assert 'resourceinstance' in resource
    ri = resource['resourceinstance']
    assert ri['resourceinstanceid'] == 'test-resource-123'
    assert ri['graph_id'] == graph_data['graphid']

    # Verify we have tiles
    roundtrip_tiles = resource['tiles']
    assert len(roundtrip_tiles) >= len(original_tiles), \
        f"Roundtrip should have at least as many tiles as original " \
        f"(found {len(roundtrip_tiles)}, original had {len(original_tiles)})"

    # Verify original nodegroups still exist
    original_nodegroups = {tile['nodegroup_id'] for tile in original_tiles}
    roundtrip_nodegroups = {tile['nodegroup_id'] for tile in roundtrip_tiles}

    for ng_id in original_nodegroups:
        assert ng_id in roundtrip_nodegroups, \
            f"Roundtrip should have tile(s) for nodegroup {ng_id}"

    print(f"Roundtrip preserved {len(original_nodegroups)} nodegroups")


def test_invalid_json_handling():
    """Test that invalid JSON raises appropriate errors"""
    graph_data = load_test_data()
    graph_json = json.dumps(graph_data)

    # Register graph first
    graph_id = alizarin.register_graph(graph_json)

    # Invalid resource JSON
    with pytest.raises(Exception):
        alizarin.tiles_to_json_tree("not valid json")

    # Invalid graph JSON during registration
    with pytest.raises(Exception):
        alizarin.register_graph("not valid json")

    # Unregistered graph_id
    with pytest.raises(Exception):
        resource = {
            'resourceinstanceid': 'test-resource',
            'graph_id': 'non-existent-graph-id',
            'tiles': [],
        }
        alizarin.tiles_to_json_tree(json.dumps(resource))


def test_batch_trees_to_tiles():
    """Test batch conversion of trees to tiles - returns BusinessDataWrapper format"""
    graph_data = load_test_data()

    # Create multiple tree structures
    # Note: description is in statement nodegroup, not basic_info
    trees = [
        {
            'basic_info': [{
                'name': {'en': 'Resource One'}
            }],
            'statement': [{
                'description': 'First resource'
            }]
        },
        {
            'basic_info': [{
                'name': {'en': 'Resource Two'}
            }],
            'statement': [{
                'description': 'Second resource'
            }]
        }
    ]

    trees_json = json.dumps(trees)
    graph_json = json.dumps(graph_data)

    # Register graph first (required by new API)
    graph_id = alizarin.register_graph(graph_json)

    # Call batch conversion
    result = alizarin.batch_trees_to_tiles(
        trees_json=trees_json,
        graph_id=graph_id
    )

    # Verify result structure - BusinessDataWrapper format
    assert isinstance(result, dict), "Result should be a dictionary"
    assert 'business_data' in result, "Result should have 'business_data' key"
    assert 'resources' in result['business_data'], "business_data should have 'resources'"

    resources = result['business_data']['resources']
    assert len(resources) == 2, "Should have 2 resources"

    for i, resource in enumerate(resources):
        # Each resource should be in StaticResource format
        assert 'resourceinstance' in resource, f"Resource {i} should have resourceinstance"
        assert 'tiles' in resource, f"Resource {i} should have tiles"

        ri = resource['resourceinstance']
        assert 'resourceinstanceid' in ri, f"Resource {i} should have resourceinstanceid"
        assert 'graph_id' in ri, f"Resource {i} should have graph_id"
        assert 'descriptors' in ri, f"Resource {i} should have descriptors"
        assert isinstance(resource['tiles'], list), f"Resource {i} tiles should be a list"

        print(f"Resource {i}: {ri['resourceinstanceid']}")
        print(f"  Descriptors: {ri.get('descriptors', {})}")
        print(f"  Tiles: {len(resource['tiles'])}")


def test_batch_tiles_to_trees():
    """Test batch conversion of tiles to trees"""
    graph_data = load_test_data()
    graph_json = json.dumps(graph_data)

    # Register graph first (required by new API)
    graph_id = alizarin.register_graph(graph_json)

    # Create resources in StaticResource format
    resources = [
        {
            'resourceinstance': {
                'resourceinstanceid': 'batch-resource-1',
                'graph_id': graph_id,
                'name': 'Batch Resource 1',
                'descriptors': {}
            },
            'tiles': create_test_tiles(graph_data)
        },
        {
            'resourceinstance': {
                'resourceinstanceid': 'batch-resource-2',
                'graph_id': graph_id,
                'name': 'Batch Resource 2',
                'descriptors': {}
            },
            'tiles': create_test_tiles(graph_data)
        }
    ]

    resources_json = json.dumps(resources)

    # Call batch conversion
    result = alizarin.batch_tiles_to_trees(
        resources_json=resources_json
    )

    # Debug output
    print(f"Graph ID registered: {graph_id}")
    print(f"Result: {result}")

    # Verify result structure
    assert isinstance(result, dict), "Result should be a dictionary"
    assert 'results' in result, "Result should have 'results' key"

    # Check for errors
    if result.get('errors'):
        print(f"Errors: {result['errors']}")

    assert len(result['results']) == 2, f"Should have 2 trees, got {len(result['results'])}, errors: {result.get('errors', [])}"

    for i, tree in enumerate(result['results']):
        # Tree format has resourceinstanceid at root level
        assert 'resourceinstanceid' in tree, f"Tree {i} should have resourceinstanceid"
        assert 'graph_id' in tree, f"Tree {i} should have graph_id"

        print(f"Tree {i}: {tree['resourceinstanceid']}")


if __name__ == '__main__':
    # Run tests when executed directly
    print("Running tiles_to_tree test...")
    test_tiles_to_tree_basic()
    print("\nRunning tree_to_tiles test...")
    test_tree_to_tiles_basic()
    print("\nRunning roundtrip test...")
    test_roundtrip_preserves_data()
    print("\nRunning batch_trees_to_tiles test...")
    test_batch_trees_to_tiles()
    print("\nRunning batch_tiles_to_trees test...")
    test_batch_tiles_to_trees()
    print("\nRunning invalid JSON test...")
    test_invalid_json_handling()
    print("\nAll tests passed!")
