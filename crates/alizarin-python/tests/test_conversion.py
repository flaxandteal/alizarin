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

    # Convert Python objects to JSON strings
    tiles_json = json.dumps(tiles)
    graph_json = json.dumps(graph_data)

    # Call the binding
    result = alizarin.tiles_to_json_tree(
        tiles_json=tiles_json,
        resource_id='test-resource-123',
        graph_id=graph_data['graphid'],
        graph_json=graph_json
    )

    # Verify result structure
    assert isinstance(result, dict), "Result should be a dictionary"
    assert result['resourceinstanceid'] == 'test-resource-123'
    assert result['graph_id'] == graph_data['graphid']

    print(f"Tree structure: {json.dumps(result, indent=2)}")


def test_tree_to_tiles_basic():
    """Test converting tree structure to tiles"""
    graph_data = load_test_data()

    # Create a simple tree structure
    tree = {
        'resourceinstanceid': 'test-resource-456',
        'graph_id': graph_data['graphid'],
        'basic_info': [{
            'name': {
                'en': 'JSON Test Group',
                'ga': 'Grúpa Tástála JSON'
            },
            'description': 'Created from JSON tree'
        }]
    }

    tree_json = json.dumps(tree)
    graph_json = json.dumps(graph_data)

    # Call the binding
    result = alizarin.json_tree_to_tiles(
        tree_json=tree_json,
        graph_json=graph_json
    )

    # Verify result structure
    assert isinstance(result, dict), "Result should be a dictionary"
    assert result['resourceinstanceid'] == 'test-resource-456'
    assert result['graph_id'] == graph_data['graphid']
    assert 'tiles' in result
    assert isinstance(result['tiles'], list)
    assert len(result['tiles']) > 0, "Should have created at least one tile"

    print(f"Created {len(result['tiles'])} tiles")
    for tile in result['tiles']:
        print(f"  Tile {tile.get('tileid', '(no id)')} "
              f"has {len(tile.get('data', {}))} data entries")


def test_roundtrip_preserves_data():
    """Test that tiles -> tree -> tiles preserves data"""
    graph_data = load_test_data()
    original_tiles = create_test_tiles(graph_data)

    tiles_json = json.dumps(original_tiles)
    graph_json = json.dumps(graph_data)

    # tiles -> tree
    tree = alizarin.tiles_to_json_tree(
        tiles_json=tiles_json,
        resource_id='test-resource-123',
        graph_id=graph_data['graphid'],
        graph_json=graph_json
    )

    print(f"Intermediate tree: {json.dumps(tree, indent=2)}")

    # tree -> tiles
    tree_json = json.dumps(tree)
    result = alizarin.json_tree_to_tiles(
        tree_json=tree_json,
        graph_json=graph_json
    )

    # Verify metadata preserved
    assert result['resourceinstanceid'] == 'test-resource-123'
    assert result['graph_id'] == graph_data['graphid']

    # Verify we have tiles
    roundtrip_tiles = result['tiles']
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

    # Invalid tiles JSON
    with pytest.raises(Exception):
        alizarin.tiles_to_json_tree(
            tiles_json="not valid json",
            resource_id='test-resource',
            graph_id=graph_data['graphid'],
            graph_json=json.dumps(graph_data)
        )

    # Invalid graph JSON
    with pytest.raises(Exception):
        alizarin.tiles_to_json_tree(
            tiles_json="[]",
            resource_id='test-resource',
            graph_id='test-graph',
            graph_json="not valid json"
        )


if __name__ == '__main__':
    # Run tests when executed directly
    print("Running tiles_to_tree test...")
    test_tiles_to_tree_basic()
    print("\nRunning tree_to_tiles test...")
    test_tree_to_tiles_basic()
    print("\nRunning roundtrip test...")
    test_roundtrip_preserves_data()
    print("\nRunning invalid JSON test...")
    test_invalid_json_handling()
    print("\nAll tests passed!")
