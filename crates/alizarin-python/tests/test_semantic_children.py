"""
Tests for semantic child matching logic.

Ports tests from tests/semantic_children_test.rs
Ensures Python logic exactly matches TypeScript semantic.ts and Rust logic.
"""
from typing import Dict, Optional

import pytest
from alizarin.static_types import StaticNode, StaticTile
from alizarin.instance_wrapper import ResourceInstanceWrapperCore


def create_test_node(
    nodeid: str,
    alias: str,
    nodegroup_id: Optional[str],
    is_collector: bool,
    is_nodegroup_root: bool = True,  # By default, make node be nodegroup root for simplicity
) -> StaticNode:
    """Helper to create a test node.

    If is_nodegroup_root=True (default), nodeid will be set equal to nodegroup_id,
    which makes the node a nodegroup root. This satisfies the "value exists" check
    in matches_semantic_child.
    """
    actual_nodeid = nodegroup_id if is_nodegroup_root and nodegroup_id else nodeid
    return StaticNode(
        nodeid=actual_nodeid,
        name=alias,
        datatype="string",
        nodegroup_id=nodegroup_id,
        alias=alias,
        graph_id="test-graph",
        is_collector=is_collector,
        isrequired=False,
        exportable=False,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
    )


def create_test_tile(
    tileid: str,
    nodegroup_id: str,
    parenttile_id: Optional[str],
    data: Optional[Dict] = None,
) -> StaticTile:
    """Helper to create a test tile"""
    return StaticTile(
        tileid=tileid,
        nodegroup_id=nodegroup_id,
        parenttile_id=parenttile_id,
        resourceinstance_id="test-resource",
        data=data if data is not None else {}
    )


def matches_semantic_child(
    parent_tile_id: Optional[str],
    parent_nodegroup_id: Optional[str],
    child_node: StaticNode,
    tile: StaticTile,
) -> bool:
    """Wrapper to call the implementation"""
    return ResourceInstanceWrapperCore.matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        tile
    )


# Test Branch 1: Different nodegroup + null parenttile_id
# PORT: js/semantic.ts line 311 - (!(value.tile.parenttile_id) || ...)
def test_different_nodegroup_null_parent_tile():
    """Branch 1: Different nodegroup + null parenttile_id should match"""
    parent_tile_id = "tile-parent"
    parent_nodegroup_id = "ng-parent"

    child_node = create_test_node("node-child", "child_alias", "ng-child", False)
    child_tile = create_test_tile("tile-child", "ng-child", None)

    assert matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Branch 1: Different nodegroup + null parenttile_id should match"


# Test Branch 1: Different nodegroup + matching parenttile_id
# PORT: js/semantic.ts line 311 - (... || value.tile.parenttile_id == tile.tileid)
def test_different_nodegroup_matching_parent_tile():
    """Branch 1: Different nodegroup + matching parenttile_id should match"""
    parent_tile_id = "tile-parent"
    parent_nodegroup_id = "ng-parent"

    child_node = create_test_node("node-child", "child_alias", "ng-child", False)
    child_tile = create_test_tile("tile-child", "ng-child", "tile-parent")

    assert matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Branch 1: Different nodegroup + matching parenttile_id should match"


# Test Branch 1: Different nodegroup + NON-matching parenttile_id
# Should NOT match
def test_different_nodegroup_non_matching_parent_tile():
    """Branch 1: Different nodegroup + non-matching parenttile_id should NOT match"""
    parent_tile_id = "tile-parent"
    parent_nodegroup_id = "ng-parent"

    child_node = create_test_node("node-child", "child_alias", "ng-child", False)
    child_tile = create_test_tile("tile-child", "ng-child", "tile-different-parent")

    assert not matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Branch 1: Different nodegroup + non-matching parenttile_id should NOT match"


# Test Branch 2: Same nodegroup + shared tile + not collector
# PORT: js/semantic.ts lines 312-315
def test_same_nodegroup_shared_tile_not_collector():
    """Branch 2: Same nodegroup + shared tile + not collector should match"""
    parent_tile_id = "tile-shared"
    parent_nodegroup_id = "ng-shared"

    # Node is nodegroup root (nodeid == nodegroup_id = "ng-shared")
    child_node = create_test_node("node-child", "child_alias", "ng-shared", False)
    # Tile must have data for the node (key = nodeid = "ng-shared")
    child_tile = create_test_tile("tile-shared", "ng-shared", None, data={"ng-shared": "some value"})

    assert matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Branch 2: Same nodegroup + shared tile + not collector should match"


# Test Branch 2: Same nodegroup + shared tile + IS collector
# Should NOT match (collector nodes don't share tiles)
def test_same_nodegroup_shared_tile_is_collector():
    """Branch 2: Same nodegroup + shared tile + IS collector should NOT match"""
    parent_tile_id = "tile-shared"
    parent_nodegroup_id = "ng-shared"

    child_node = create_test_node("node-child", "child_alias", "ng-shared", True)
    child_tile = create_test_tile("tile-shared", "ng-shared", None)

    assert not matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Branch 2: Same nodegroup + shared tile + IS collector should NOT match"


# Test Branch 3: Different nodegroup + is_collector
# PORT: js/semantic.ts lines 324-327
def test_different_nodegroup_is_collector():
    """Branch 3: Different nodegroup + is_collector should match when parenttile matches"""
    parent_tile_id = "tile-parent"
    parent_nodegroup_id = "ng-parent"

    # Collector with matching parenttile_id should match
    child_node = create_test_node("node-child", "child_alias", "ng-child", True)
    child_tile = create_test_tile("tile-child", "ng-child", "tile-parent")

    assert matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Branch 3: Different nodegroup + is_collector + matching parent should match"

    # Collector with null parenttile_id should also match
    child_tile_null = create_test_tile("tile-child2", "ng-child", None)
    assert matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile_null
    ), "Branch 3: Different nodegroup + is_collector + null parent should match"

    # Collector with non-matching parenttile_id should NOT match
    child_tile_other = create_test_tile("tile-child3", "ng-child", "tile-some-other-parent")
    assert not matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile_other
    ), "Branch 3: Different nodegroup + is_collector + wrong parent should NOT match"


# Test Edge Case: Same nodegroup + different tile
# Should NOT match (not shared)
def test_same_nodegroup_different_tile():
    """Same nodegroup + different tile should NOT match"""
    parent_tile_id = "tile-parent"
    parent_nodegroup_id = "ng-shared"

    child_node = create_test_node("node-child", "child_alias", "ng-shared", False)
    child_tile = create_test_tile("tile-different", "ng-shared", None)

    assert not matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Same nodegroup + different tile should NOT match"


# Test Edge Case: null parent tile
# Should NOT match any branch (except collector)
def test_null_parent_tile():
    """Null parent tile + not collector should NOT match"""
    parent_tile_id = None
    parent_nodegroup_id = "ng-parent"

    child_node = create_test_node("node-child", "child_alias", "ng-child", False)
    child_tile = create_test_tile("tile-child", "ng-child", None)

    assert not matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Null parent tile + not collector should NOT match"


# Test Edge Case: null parent tile + collector
# Should match Branch 3
def test_null_parent_tile_collector():
    """Null parent tile + different nodegroup + collector should match Branch 3"""
    parent_tile_id = None
    parent_nodegroup_id = "ng-parent"

    child_node = create_test_node("node-child", "child_alias", "ng-child", True)
    child_tile = create_test_tile("tile-child", "ng-child", None)

    assert matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        child_tile
    ), "Null parent tile + different nodegroup + collector should match Branch 3"


# Test comprehensive scenario: Multiple children with different relationships
def test_multiple_children_relationships():
    """Test multiple children with different relationship types"""
    parent_tile_id = "tile-parent"
    parent_nodegroup_id = "ng-parent"

    # Child 1: Different nodegroup, null parent (Branch 1) - MATCH
    child1 = create_test_node("node-child1", "child1", "ng-child1", False)
    tile1 = create_test_tile("tile-child1", "ng-child1", None)
    assert matches_semantic_child(parent_tile_id, parent_nodegroup_id, child1, tile1)

    # Child 2: Same nodegroup, shared tile, not collector (Branch 2) - MATCH
    # Node is nodegroup root (nodeid == nodegroup_id = "ng-parent")
    child2 = create_test_node("node-child2", "child2", "ng-parent", False)
    # Tile must have data for the node (key = nodeid = "ng-parent")
    tile2 = create_test_tile("tile-parent", "ng-parent", None, data={"ng-parent": "some value"})
    assert matches_semantic_child(parent_tile_id, parent_nodegroup_id, child2, tile2)

    # Child 3: Different nodegroup, collector (Branch 3) - MATCH (matching parent)
    child3 = create_test_node("node-child3", "child3", "ng-child3", True)
    tile3 = create_test_tile("tile-child3", "ng-child3", "tile-parent")
    assert matches_semantic_child(parent_tile_id, parent_nodegroup_id, child3, tile3)

    # Child 4: Same nodegroup, collector (no branch matches) - NO MATCH
    child4 = create_test_node("node-child4", "child4", "ng-parent", True)
    tile4 = create_test_tile("tile-parent", "ng-parent", None)
    assert not matches_semantic_child(parent_tile_id, parent_nodegroup_id, child4, tile4)


if __name__ == '__main__':
    # Run all tests
    pytest.main([__file__, '-v'])
