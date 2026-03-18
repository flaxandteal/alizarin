"""
Tests for resource handling and instance wrapper.

Ports tests from tests/client.test.ts and covers ResourceInstanceWrapper functionality.
"""

import pytest
import json
from dataclasses import dataclass
from typing import Any, Dict, Optional

import alizarin as alizarin_rust
from alizarin.static_types import (
    StaticNode,
    StaticEdge,
    StaticNodegroup,
    StaticTile,
    StaticGraph,
    StaticGraphMeta,
    StaticResource,
)
from alizarin.instance_wrapper import (
    ResourceInstanceWrapper,
    ResourceInstanceWrapperCore,
)
from alizarin.model_wrapper import ResourceModelWrapper
from alizarin.pseudos import PseudoValue, PseudoList


# =============================================================================
# Mock WKRM for tests
# =============================================================================


@dataclass
class MockWKRM:
    """Mock WKRM for testing."""
    model_name: str
    model_class_name: str
    graph_id: str
    meta: StaticGraphMeta


# =============================================================================
# Test Fixtures
# =============================================================================


def create_test_graph() -> StaticGraph:
    """Create a minimal test graph."""
    root = StaticNode(
        nodeid="root-node",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="root",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        istopnode=True,
    )
    child_node = StaticNode(
        nodeid="child-node",
        name="Child",
        datatype="string",
        nodegroup_id="ng-1",
        alias="child_alias",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
    )
    edge = StaticEdge(
        edgeid="edge-1",
        domainnode_id="root-node",
        rangenode_id="child-node",
        graph_id="test-graph",
    )
    ng = StaticNodegroup(nodegroupid="ng-1", cardinality="n")

    return StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root, child_node],
        edges=[edge],
        nodegroups=[ng],
        root=root,
    )


@pytest.fixture
def registered_test_graph():
    """Fixture that registers the test graph and returns it."""
    graph = create_test_graph()
    graph_dict = graph.to_dict()
    graph_json = json.dumps(graph_dict)
    alizarin_rust.register_graph(graph_json)
    return graph


@pytest.fixture
def test_model(registered_test_graph):
    """Fixture that creates a ResourceModelWrapper with the test graph."""
    graph = registered_test_graph
    wkrm = MockWKRM(
        model_name="TestModel",
        model_class_name="TestModel",
        graph_id=graph.graphid,
        meta=StaticGraphMeta(
            name={"en": "Test Graph"},
            graphid=graph.graphid,
        ),
    )
    return ResourceModelWrapper(wkrm=wkrm, graph=graph)


def create_test_resource(graph_id: str = "test-graph") -> StaticResource:
    """Create a minimal test resource."""
    from alizarin.static_types import StaticResourceMetadata, StaticResourceDescriptors

    tile = StaticTile(
        tileid="tile-1",
        nodegroup_id="ng-1",
        resourceinstance_id="resource-1",
        data={"child-node": {"en": "Test Value"}},
    )
    metadata = StaticResourceMetadata(
        descriptors=StaticResourceDescriptors(),
        graph_id=graph_id,
        name="Test Resource",
        resourceinstanceid="resource-1",
    )
    return StaticResource(
        resourceinstance=metadata,
        tiles=[tile],
    )


def create_test_node(
    nodeid: str,
    alias: str,
    datatype: str = "string",
    nodegroup_id: str = "ng-1",
    is_collector: bool = False,
    is_nodegroup_root: bool = True,  # By default, make node be nodegroup root
) -> StaticNode:
    """Helper to create a test node.

    If is_nodegroup_root=True (default), nodeid will be set equal to nodegroup_id,
    which makes the node a nodegroup root and satisfies the "value exists" check.
    """
    actual_nodeid = nodegroup_id if is_nodegroup_root else nodeid
    return StaticNode(
        nodeid=actual_nodeid,
        name=alias,
        datatype=datatype,
        nodegroup_id=nodegroup_id,
        alias=alias,
        graph_id="test-graph",
        is_collector=is_collector,
        isrequired=False,
        exportable=True,
    )


def create_test_tile(
    tileid: str,
    nodegroup_id: str,
    data: Dict[str, Any],
    parenttile_id: Optional[str] = None,
) -> StaticTile:
    """Helper to create a test tile."""
    return StaticTile(
        tileid=tileid,
        nodegroup_id=nodegroup_id,
        resourceinstance_id="resource-1",
        data=data,
        parenttile_id=parenttile_id,
    )


# =============================================================================
# StaticResource Tests
# =============================================================================


class TestStaticResource:
    """Tests for StaticResource."""

    def test_create_resource(self):
        """Should create a resource with required fields."""
        from alizarin.static_types import StaticResourceMetadata, StaticResourceDescriptors

        metadata = StaticResourceMetadata(
            descriptors=StaticResourceDescriptors(),
            graph_id="graph-1",
            name="Test Resource",
            resourceinstanceid="resource-1",
        )
        resource = StaticResource(
            resourceinstance=metadata,
            tiles=[],
        )
        assert resource.resourceinstance.resourceinstanceid == "resource-1"
        assert resource.resourceinstance.graph_id == "graph-1"
        assert len(resource.get_tiles()) == 0

    def test_resource_with_tiles(self):
        """Should store tiles correctly."""
        from alizarin.static_types import StaticResourceMetadata, StaticResourceDescriptors

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={"node-1": "value"},
        )
        metadata = StaticResourceMetadata(
            descriptors=StaticResourceDescriptors(),
            graph_id="graph-1",
            name="Test Resource",
            resourceinstanceid="resource-1",
        )
        resource = StaticResource(
            resourceinstance=metadata,
            tiles=[tile],
        )
        assert len(resource.tiles) == 1
        assert resource.tiles[0].tileid == "tile-1"

    def test_resource_with_multiple_tiles(self):
        """Should handle multiple tiles."""
        from alizarin.static_types import StaticResourceMetadata, StaticResourceDescriptors

        tiles = [
            StaticTile(
                tileid=f"tile-{i}",
                nodegroup_id="ng-1",
                resourceinstance_id="resource-1",
                data={},
            )
            for i in range(5)
        ]
        metadata = StaticResourceMetadata(
            descriptors=StaticResourceDescriptors(),
            graph_id="graph-1",
            name="Test Resource",
            resourceinstanceid="resource-1",
        )
        resource = StaticResource(
            resourceinstance=metadata,
            tiles=tiles,
        )
        assert len(resource.tiles) == 5

    def test_resource_to_dict(self):
        """Should serialize to dict."""
        from alizarin.static_types import StaticResourceMetadata, StaticResourceDescriptors

        metadata = StaticResourceMetadata(
            descriptors=StaticResourceDescriptors(),
            graph_id="graph-1",
            name="Test Resource",
            resourceinstanceid="resource-1",
        )
        resource = StaticResource(
            resourceinstance=metadata,
            tiles=[],
        )
        d = resource.to_dict()
        assert d["resourceinstance"]["resourceinstanceid"] == "resource-1"
        assert d["resourceinstance"]["graph_id"] == "graph-1"
        assert d["resourceinstance"]["name"] == "Test Resource"

    def test_resource_from_dict(self):
        """Should deserialize from dict."""
        data = {
            "resourceinstance": {
                "descriptors": {},
                "graph_id": "graph-2",
                "name": "Resource Two",
                "resourceinstanceid": "resource-2",
            },
            "tiles": [],
        }
        resource = StaticResource.from_dict(data)
        assert resource.resourceinstance.resourceinstanceid == "resource-2"
        assert resource.resourceinstance.graph_id == "graph-2"


# =============================================================================
# ResourceInstanceWrapperCore Tests - Semantic Child Matching
# =============================================================================


class TestResourceInstanceWrapperCore:
    """Tests for ResourceInstanceWrapperCore.matches_semantic_child()."""

    def test_matches_different_nodegroup_with_parent_tile(self):
        """Branch 1: Different nodegroup + parent_tile_id exists."""
        child_node = create_test_node("child", "child_alias", nodegroup_id="ng-child")
        tile = create_test_tile("tile-1", "ng-child", {}, parenttile_id="parent-tile")

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id="parent-tile",
            parent_nodegroup_id="ng-parent",
            child_node=child_node,
            tile=tile,
        )
        assert result is True

    def test_matches_different_nodegroup_with_null_parenttile(self):
        """Branch 1: Different nodegroup + null parenttile_id on tile."""
        child_node = create_test_node("child", "child_alias", nodegroup_id="ng-child")
        tile = create_test_tile("tile-1", "ng-child", {}, parenttile_id=None)

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id="parent-tile",
            parent_nodegroup_id="ng-parent",
            child_node=child_node,
            tile=tile,
        )
        assert result is True

    def test_no_match_different_nodegroup_without_parent_tile_id(self):
        """Branch 1: Should not match if parent_tile_id is None."""
        child_node = create_test_node("child", "child_alias", nodegroup_id="ng-child")
        tile = create_test_tile("tile-1", "ng-child", {})

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id=None,  # No parent tile
            parent_nodegroup_id="ng-parent",
            child_node=child_node,
            tile=tile,
        )
        # This should only match via Branch 3 (collector), which is not set
        assert result is False

    def test_matches_same_nodegroup_shared_tile(self):
        """Branch 2: Same nodegroup + shared tile + not collector."""
        # Node is nodegroup root (nodeid == nodegroup_id = "ng-shared")
        child_node = create_test_node(
            "child", "child_alias", nodegroup_id="ng-shared", is_collector=False
        )
        # Tile must have data for the node (key = nodeid = "ng-shared")
        tile = create_test_tile("tile-shared", "ng-shared", {"ng-shared": "some value"})

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id="tile-shared",  # Same as tile.tileid
            parent_nodegroup_id="ng-shared",
            child_node=child_node,
            tile=tile,
        )
        assert result is True

    def test_no_match_same_nodegroup_different_tile(self):
        """Branch 2: Same nodegroup but different tile ID."""
        child_node = create_test_node(
            "child", "child_alias", nodegroup_id="ng-shared", is_collector=False
        )
        tile = create_test_tile("tile-1", "ng-shared", {})

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id="tile-different",  # Different from tile.tileid
            parent_nodegroup_id="ng-shared",
            child_node=child_node,
            tile=tile,
        )
        # Different tile ID, so Branch 2 doesn't match
        # Branch 1 also doesn't match (same nodegroup)
        # Branch 3 doesn't match (not collector)
        assert result is False

    def test_no_match_same_nodegroup_collector(self):
        """Branch 2: Same nodegroup but node is collector."""
        child_node = create_test_node(
            "child", "child_alias", nodegroup_id="ng-shared", is_collector=True
        )
        tile = create_test_tile("tile-shared", "ng-shared", {})

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id="tile-shared",
            parent_nodegroup_id="ng-shared",
            child_node=child_node,
            tile=tile,
        )
        # is_collector = True, so Branch 2 condition fails
        # But Branch 3 doesn't apply (same nodegroup)
        assert result is False

    def test_matches_collector_node(self):
        """Branch 3: Different nodegroup + is collector always matches."""
        child_node = create_test_node(
            "child", "child_alias", nodegroup_id="ng-child", is_collector=True
        )
        tile = create_test_tile("tile-1", "ng-child", {})

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id=None,  # Even without parent tile
            parent_nodegroup_id="ng-parent",  # Different nodegroup
            child_node=child_node,
            tile=tile,
        )
        assert result is True

    def test_collector_different_nodegroup_no_parent_tile(self):
        """Branch 3: Collector matches even without parent_tile_id."""
        child_node = create_test_node(
            "child", "child_alias", nodegroup_id="ng-collector", is_collector=True
        )
        tile = create_test_tile("tile-1", "ng-collector", {})

        result = ResourceInstanceWrapperCore.matches_semantic_child(
            parent_tile_id=None,
            parent_nodegroup_id="ng-parent",
            child_node=child_node,
            tile=tile,
        )
        assert result is True


# =============================================================================
# ResourceInstanceWrapper Tests
# =============================================================================


class TestResourceInstanceWrapper:
    """Tests for ResourceInstanceWrapper."""

    def test_create_wrapper(self, test_model):
        """Should create wrapper instance."""
        mock_wkri = object()

        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        assert wrapper.wkri is mock_wkri
        assert wrapper._rust_core is not None
        assert wrapper._pseudo_cache == {}
        assert wrapper.value_cache is None

    def test_set_pseudo(self, test_model):
        """Should set pseudo values in cache."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        node = create_test_node("node-1", "test_alias")
        tile = create_test_tile("tile-1", "ng-1", {})

        pseudo = PseudoValue(node=node, tile=tile, value="test_value")
        wrapper.set_pseudo("test_alias", pseudo)

        assert "test_alias" in wrapper._pseudo_cache
        assert len(wrapper._pseudo_cache["test_alias"]) == 1

    def test_set_pseudo_list(self, test_model):
        """Should set PseudoList in cache."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        node = create_test_node("node-1", "test_alias")
        tile = create_test_tile("tile-1", "ng-1", {})

        pseudo_list = PseudoList(
            alias="test_alias",
            values=[PseudoValue(node=node, tile=tile, value="value1")],
            is_single=False,
        )
        wrapper.set_pseudo("test_alias", pseudo_list)

        assert "test_alias" in wrapper._pseudo_cache
        assert wrapper._pseudo_cache["test_alias"] is pseudo_list

    def test_clear_caches(self, test_model):
        """Should clear all caches."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        # Add some cache entries
        wrapper._pseudo_cache["key1"] = PseudoList("key1", [], False)
        wrapper.value_cache = {"tile-1": {"node-1": "value"}}

        wrapper.clear_caches()

        assert wrapper._pseudo_cache == {}
        assert wrapper.value_cache is None

    @pytest.mark.asyncio
    async def test_has_pseudo_returns_false_when_empty(self, test_model):
        """Should return False when cache is empty."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        result = await wrapper.has_pseudo("nonexistent")
        assert result is False

    @pytest.mark.asyncio
    async def test_retrieve_pseudo_returns_default_when_not_found(self, test_model):
        """Should return default when pseudo not found."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        result = await wrapper.retrieve_pseudo("nonexistent", default=[])
        assert result == []

    @pytest.mark.asyncio
    async def test_retrieve_pseudo_raises_when_not_found_and_raise_error(self, test_model):
        """Should raise KeyError when pseudo not found and raise_error=True."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        with pytest.raises(KeyError, match="Pseudo value not found"):
            await wrapper.retrieve_pseudo("nonexistent", raise_error=True)

    def test_all_entries_iterates_cache(self, test_model):
        """Should iterate over all cached entries."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        # Add cache entries
        wrapper._pseudo_cache["alias1"] = PseudoList("alias1", [], False)
        wrapper._pseudo_cache["alias2"] = PseudoList("alias2", [], False)

        entries = list(wrapper.all_entries())
        assert len(entries) == 2
        aliases = [e[0] for e in entries]
        assert "alias1" in aliases
        assert "alias2" in aliases

    def test_tile_to_dict(self, test_model):
        """Should convert tile to dict format."""
        mock_wkri = object()
        wrapper = ResourceInstanceWrapper(
            wkri=mock_wkri,
            model=test_model,
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={"node-1": "value"},
            parenttile_id="parent-tile",
            sortorder=1,
        )

        result = wrapper._tile_to_dict(tile)

        assert result["tileid"] == "tile-1"
        assert result["nodegroup_id"] == "ng-1"
        assert result["resourceinstance_id"] == "resource-1"
        assert result["data"] == {"node-1": "value"}
        assert result["parenttile_id"] == "parent-tile"
        assert result["sortorder"] == 1


# =============================================================================
# Resource Graph Relationship Tests
# =============================================================================


class TestResourceGraphRelationship:
    """Tests for resource and graph relationships."""

    def test_resource_references_graph(self):
        """Resource should reference its graph."""
        graph = create_test_graph()
        resource = create_test_resource(graph.graphid)

        assert resource.resourceinstance.graph_id == graph.graphid

    def test_tile_references_resource(self):
        """Tiles should reference their parent resource."""
        resource = create_test_resource()

        for tile in resource.tiles:
            assert tile.resourceinstance_id == resource.resourceinstance.resourceinstanceid

    def test_tile_references_nodegroup(self):
        """Tiles should reference valid nodegroups."""
        graph = create_test_graph()
        resource = create_test_resource(graph.graphid)

        nodegroup_ids = {ng.nodegroupid for ng in graph.nodegroups}

        for tile in resource.tiles:
            assert tile.nodegroup_id in nodegroup_ids


# =============================================================================
# PseudoList Tests
# =============================================================================


class TestPseudoList:
    """Tests for PseudoList operations."""

    def test_create_pseudo_list(self):
        """Should create PseudoList with values."""
        node = create_test_node("node-1", "alias")
        tile = create_test_tile("tile-1", "ng-1", {})

        values = [
            PseudoValue(node=node, tile=tile, value="value1"),
            PseudoValue(node=node, tile=tile, value="value2"),
        ]

        pseudo_list = PseudoList(alias="alias", values=values, is_single=False)

        assert pseudo_list.alias == "alias"
        assert len(pseudo_list) == 2
        assert not pseudo_list.is_single

    def test_pseudo_list_iteration(self):
        """Should iterate over values."""
        node = create_test_node("node-1", "alias")
        tile = create_test_tile("tile-1", "ng-1", {})

        values = [
            PseudoValue(node=node, tile=tile, value=f"value{i}")
            for i in range(3)
        ]

        pseudo_list = PseudoList(alias="alias", values=values, is_single=False)

        iterated = list(pseudo_list)
        assert len(iterated) == 3

    def test_pseudo_list_append(self):
        """Should append new values."""
        node = create_test_node("node-1", "alias")
        tile = create_test_tile("tile-1", "ng-1", {})

        pseudo_list = PseudoList(alias="alias", values=[], is_single=False)
        pseudo_list.append(PseudoValue(node=node, tile=tile, value="new_value"))

        assert len(pseudo_list) == 1

    def test_pseudo_list_single(self):
        """Should mark as single when appropriate."""
        node = create_test_node("node-1", "alias")
        tile = create_test_tile("tile-1", "ng-1", {})

        pseudo_list = PseudoList(
            alias="alias",
            values=[PseudoValue(node=node, tile=tile, value="single")],
            is_single=True,
        )

        assert pseudo_list.is_single is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
