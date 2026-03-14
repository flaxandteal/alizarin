"""
Tests for PseudoList.matching_entries

Ports tests from tests/matching_entries_test.rs
Tests the fix for the registry names issue where inner values need to be filtered
by their parent tile context.
"""
from typing import Optional

import pytest
from alizarin.static_types import StaticNode, StaticTile
from alizarin.pseudos import PseudoValue, create_pseudo_value as factory_create_pseudo_value, create_pseudo_list


def create_node(nodeid: str, alias: str, nodegroup_id: str) -> StaticNode:
    """Create a test node"""
    return StaticNode(
        nodeid=nodeid,
        name=alias,
        datatype="string",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        issearchable=False,
        istopnode=False,
        alias=alias,
        nodegroup_id=nodegroup_id,
        exportable=False,
        sortorder=0
    )


def create_tile(tileid: str, nodegroup_id: str, parenttile_id: Optional[str] = None) -> StaticTile:
    """Create a test tile"""
    return StaticTile(
        tileid=tileid,
        nodegroup_id=nodegroup_id,
        resourceinstance_id="test-resource",
        parenttile_id=parenttile_id,
        data={}
    )


def create_pseudo_value(node: StaticNode, tile: StaticTile, value: str = "test-value") -> PseudoValue:
    """Create a PseudoValue with Rust backing via factory"""
    return factory_create_pseudo_value(
        node=node,
        tile=tile,
        value=value,
        parent_pseudo=None
    )


def test_matching_entries_filters_by_parent_tile():
    """
    Test: matching_entries filters by tile context

    Scenario: Two outer tiles (like names[0] and names[1]) each have inner values
    (like name_use_type). When querying inner values for a specific outer tile,
    only values from that tile should be returned.
    """
    nodegroup_id = "ng-names"

    # Create two outer tiles (like names[0] and names[1])
    outer_tile_a = create_tile("tile-outer-a", nodegroup_id)
    outer_tile_b = create_tile("tile-outer-b", nodegroup_id)

    # Create inner values - each belongs to its respective outer tile
    inner_node = create_node("node-name-use-type", "name_use_type", nodegroup_id)

    # Value A belongs to tile A (same tile = inner value on outer tile)
    value_a = create_pseudo_value(inner_node, outer_tile_a, "Primary")
    # Value B belongs to tile B
    value_b = create_pseudo_value(inner_node, outer_tile_b, "Alternative")

    # Create a PseudoList with both values (using factory for Rust backing)
    pseudo_list = create_pseudo_list(
        alias="name_use_type",
        values=[value_a, value_b],
        is_single=True  # is_single - this is a single value per outer tile
    )

    # Query for tile A - should only get value from tile A
    matches_a = pseudo_list.matching_entries(
        parent_tile_id="tile-outer-a",
        nodegroup_id=nodegroup_id
    )
    assert len(matches_a) == 1, "Should only match one value for tile A"
    assert matches_a[0].tile.tileid == "tile-outer-a", "Should match value from tile A"

    # Query for tile B - should only get value from tile B
    matches_b = pseudo_list.matching_entries(
        parent_tile_id="tile-outer-b",
        nodegroup_id=nodegroup_id
    )
    assert len(matches_b) == 1, "Should only match one value for tile B"
    assert matches_b[0].tile.tileid == "tile-outer-b", "Should match value from tile B"


def test_matching_entries_handles_child_tiles():
    """
    Test: matching_entries handles child tiles (different nodegroups)

    When inner values are in child tiles (tiles with parenttile_id pointing to outer tile),
    they should still be matched correctly.
    """
    outer_nodegroup = "ng-outer"
    inner_nodegroup = "ng-inner"

    # Create outer tile
    outer_tile = create_tile("tile-outer", outer_nodegroup)

    # Create child tiles that point to the outer tile
    child_tile_a = create_tile("tile-child-a", inner_nodegroup, "tile-outer")
    child_tile_b = create_tile("tile-child-b", inner_nodegroup, "tile-outer")
    unrelated_tile = create_tile("tile-unrelated", inner_nodegroup, "tile-other-parent")

    inner_node = create_node("node-inner", "inner_value", inner_nodegroup)

    value_a = create_pseudo_value(inner_node, child_tile_a)
    value_b = create_pseudo_value(inner_node, child_tile_b)
    value_unrelated = create_pseudo_value(inner_node, unrelated_tile)

    pseudo_list = create_pseudo_list(
        alias="inner_value",
        values=[value_a, value_b, value_unrelated],
        is_single=False  # Can have multiple
    )

    # Query for outer tile - should get child tiles A and B, not unrelated
    matches = pseudo_list.matching_entries(
        parent_tile_id="tile-outer",
        nodegroup_id=inner_nodegroup
    )

    assert len(matches) == 2, "Should match both child tiles pointing to outer tile"

    # Verify we got the right tiles
    tile_ids = [m.tile.tileid for m in matches]
    assert "tile-child-a" in tile_ids
    assert "tile-child-b" in tile_ids
    assert "tile-unrelated" not in tile_ids


def test_matching_entries_handles_root_level():
    """
    Test: matching_entries handles root level (no parent tile)

    When querying at the root level (no parent tile), only tiles without
    a parenttile_id should match.
    """
    nodegroup_id = "ng-root"

    # Create root-level tiles (no parenttile_id)
    root_tile_a = create_tile("tile-root-a", nodegroup_id)
    root_tile_b = create_tile("tile-root-b", nodegroup_id)

    # Create a child tile (has parenttile_id)
    child_tile = create_tile("tile-child", nodegroup_id, "tile-some-parent")

    node = create_node("node-root", "root_value", nodegroup_id)

    value_root_a = create_pseudo_value(node, root_tile_a)
    value_root_b = create_pseudo_value(node, root_tile_b)
    value_child = create_pseudo_value(node, child_tile)

    pseudo_list = create_pseudo_list(
        alias="root_value",
        values=[value_root_a, value_root_b, value_child],
        is_single=False
    )

    # Query at root level (no parent tile) - should get only root tiles
    matches = pseudo_list.matching_entries(
        parent_tile_id=None,  # No parent tile = root query
        nodegroup_id=nodegroup_id
    )

    assert len(matches) == 2, "Should match only root-level tiles"

    tile_ids = [m.tile.tileid for m in matches]
    assert "tile-root-a" in tile_ids
    assert "tile-root-b" in tile_ids
    assert "tile-child" not in tile_ids


def test_matching_entries_requires_nodegroup_match():
    """
    Test: matching_entries requires nodegroup match

    Values should only be returned if they're in the requested nodegroup.
    """
    nodegroup_a = "ng-a"
    nodegroup_b = "ng-b"

    # Create tiles in different nodegroups
    tile_a = create_tile("tile-a", nodegroup_a)
    tile_b = create_tile("tile-b", nodegroup_b)

    node_a = create_node("node-a", "value_a", nodegroup_a)
    node_b = create_node("node-b", "value_b", nodegroup_b)

    value_a = create_pseudo_value(node_a, tile_a)
    value_b = create_pseudo_value(node_b, tile_b)

    pseudo_list = create_pseudo_list(
        alias="mixed_values",
        values=[value_a, value_b],
        is_single=False
    )

    # Query for nodegroup A - should only get value from nodegroup A
    matches = pseudo_list.matching_entries(
        parent_tile_id=None,
        nodegroup_id=nodegroup_a
    )

    assert len(matches) == 1, "Should only match values in requested nodegroup"
    assert matches[0].tile.nodegroup_id == nodegroup_a, "Should match value from nodegroup A"


def test_registry_names_regression():
    """
    Regression test: Original registry names bug

    This simulates the exact scenario that was failing:
    - Registry has multiple names (outer loop)
    - Each name has a name_use_type (inner value, is_single=True)
    - Without proper filtering, ALL name_use_type values were returned for each name
    """
    names_nodegroup = "ng-names"

    # Create two name tiles (simulating names[0] and names[1])
    name_tile_0 = create_tile("name-tile-0", names_nodegroup)
    name_tile_1 = create_tile("name-tile-1", names_nodegroup)

    # Each name tile has its own name_use_type value
    name_use_type_node = create_node("node-name-use-type", "name_use_type", names_nodegroup)

    # Value for names[0] - "Primary"
    value_0 = create_pseudo_value(name_use_type_node, name_tile_0, "Primary")

    # Value for names[1] - "Alternative"
    value_1 = create_pseudo_value(name_use_type_node, name_tile_1, "Alternative")

    # Create the PseudoList with is_single=True (cardinality-1)
    pseudo_list = create_pseudo_list(
        alias="name_use_type",
        values=[value_0, value_1],
        is_single=True  # CRITICAL: is_single=True means we expect exactly 0 or 1 value per context
    )

    # Query for names[0]'s tile - should get exactly 1 value
    matches_0 = pseudo_list.matching_entries(
        parent_tile_id="name-tile-0",
        nodegroup_id=names_nodegroup
    )

    assert len(matches_0) == 1, \
        f"REGRESSION: names[0].name_use_type should return exactly 1 value, not {len(matches_0)}"
    assert matches_0[0]._value == "Primary", "names[0].name_use_type should be 'Primary'"

    # Query for names[1]'s tile - should get exactly 1 value
    matches_1 = pseudo_list.matching_entries(
        parent_tile_id="name-tile-1",
        nodegroup_id=names_nodegroup
    )

    assert len(matches_1) == 1, \
        f"REGRESSION: names[1].name_use_type should return exactly 1 value, not {len(matches_1)}"
    assert matches_1[0]._value == "Alternative", "names[1].name_use_type should be 'Alternative'"


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
