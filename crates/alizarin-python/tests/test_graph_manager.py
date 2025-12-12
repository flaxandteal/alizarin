"""
Tests for ResourceModelWrapper functionality.

Ports tests from tests/graphManager.test.ts
Tests graph model management, navigation, and caching.
"""

import pytest
from alizarin.static_types import (
    StaticGraph, StaticNode, StaticEdge, StaticNodegroup,
    StaticCard
)
from alizarin.model_wrapper import ResourceModelWrapper
from alizarin.graph_manager import WKRM


def create_test_graph(name="Test Graph", author=None):
    """Helper to create a minimal test graph"""
    root_node = StaticNode(
        nodeid="root-node",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
    )

    graph = StaticGraph(
        graphid="test-graph",
        name={"en": name},
        nodes=[root_node],
        edges=[],
        nodegroups=[],
        root=root_node,
        cards=[],
        author=author,
        config={}
    )

    return graph


def create_test_wkrm(graph):
    """Helper to create a WKRM instance for testing"""
    meta = {
        'graphid': graph.graphid,
        'name': graph.name,
        'slug': 'test_graph',
        'relatable_resource_model_ids': []
    }
    return WKRM(meta)


def test_build_nodes_should_cache_nodes_edges_and_nodegroups():
    """ResourceModelWrapper.build_nodes() should cache nodes, edges, and nodegroups"""
    # Create a simple graph with a child node
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
    )

    child_node = StaticNode(
        nodeid="child1",
        name="Test Field",
        datatype="string",
        nodegroup_id="child1-ng",
        alias="test_field",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
    )

    edge = StaticEdge(
        edgeid="edge1",
        domainnode_id="root",
        rangenode_id="child1",
        graph_id="test-graph"
    )

    nodegroup = StaticNodegroup(
        nodegroupid="child1-ng",
        cardinality="n"
    )

    graph = StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root_node, child_node],
        edges=[edge],
        nodegroups=[nodegroup],
        root=root_node,
        cards=[],
        config={}
    )

    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)

    # Build nodes
    wrapper.build_nodes()

    # Caches should be populated
    assert wrapper.nodes is not None
    assert wrapper.edges is not None
    assert wrapper.nodegroups is not None
    assert len(wrapper.nodes) > 0
    assert len(wrapper.edges) > 0
    assert len(wrapper.nodegroups) > 0


def test_get_node_objects_should_build_nodes_if_not_cached():
    """ResourceModelWrapper.get_node_objects() should build nodes if not cached"""
    graph = create_test_graph()
    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)

    nodes = wrapper.get_node_objects()

    assert wrapper.nodes is not None
    assert len(nodes) > 0
    assert graph.root.nodeid in nodes


def test_get_edges_should_return_edge_map():
    """ResourceModelWrapper.get_edges() should return edge map"""
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
    )

    child_node = StaticNode(
        nodeid="child",
        name="Child Node",
        datatype="string",
        nodegroup_id="child-ng",
        alias="child",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
    )

    edge = StaticEdge(
        edgeid="edge1",
        domainnode_id="root",
        rangenode_id="child",
        graph_id="test-graph"
    )

    graph = StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root_node, child_node],
        edges=[edge],
        nodegroups=[],
        root=root_node,
        cards=[],
        config={}
    )

    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)
    edges = wrapper.get_edges()

    # Should have edge from root to child
    assert root_node.nodeid in edges
    child_edges = edges[root_node.nodeid]
    assert child_edges is not None
    assert len(child_edges) == 1


def test_get_child_nodes_should_return_child_nodes():
    """ResourceModelWrapper.get_child_nodes() should return child nodes"""
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
    )

    child_node = StaticNode(
        nodeid="child",
        name="Child Node",
        datatype="string",
        nodegroup_id="child-ng",
        alias="child",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
    )

    edge = StaticEdge(
        edgeid="edge1",
        domainnode_id="root",
        rangenode_id="child",
        graph_id="test-graph"
    )

    graph = StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root_node, child_node],
        edges=[edge],
        nodegroups=[],
        root=root_node,
        cards=[],
        config={}
    )

    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)
    child_nodes = wrapper.get_child_nodes(root_node.nodeid)

    assert len(child_nodes) == 1
    assert "child" in child_nodes


def test_get_root_node_should_return_root_node():
    """ResourceModelWrapper.get_root_node() should return root node"""
    graph = create_test_graph()
    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)

    root_node = wrapper.get_root_node()

    assert root_node.nodeid == graph.root.nodeid
    assert root_node.nodegroup_id is None


def test_get_collections_should_return_unique_collection_ids():
    """ResourceModelWrapper.get_collections() should return unique collection IDs"""
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
    )

    # Create concept nodes with same collection
    concept1 = StaticNode(
        nodeid="concept1",
        name="Concept Field 1",
        datatype="concept",
        nodegroup_id="concept1-ng",
        alias="concept1",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
        config={"rdmCollection": "collection-123"}
    )

    concept2 = StaticNode(
        nodeid="concept2",
        name="Concept Field 2",
        datatype="concept",
        nodegroup_id="concept2-ng",
        alias="concept2",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
        config={"rdmCollection": "collection-123"}
    )

    graph = StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root_node, concept1, concept2],
        edges=[],
        nodegroups=[],
        root=root_node,
        cards=[],
        config={}
    )

    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)
    collections = wrapper.get_collections()

    assert len(collections) == 1
    assert collections[0] == "collection-123"


def test_get_branch_publication_ids_should_return_unique_ids():
    """ResourceModelWrapper.get_branch_publication_ids() should return unique branch publication IDs"""
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
    )

    field_node = StaticNode(
        nodeid="field1",
        name="Field 1",
        datatype="string",
        nodegroup_id="field1-ng",
        alias="field1",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
        sourcebranchpublication_id="branch-pub-123"
    )

    graph = StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root_node, field_node],
        edges=[],
        nodegroups=[],
        root=root_node,
        cards=[],
        config={}
    )

    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)
    branch_pub_ids = wrapper.get_branch_publication_ids()

    assert len(branch_pub_ids) == 1
    assert branch_pub_ids[0] == "branch-pub-123"


def test_get_permitted_nodegroups_should_default_to_all_permitted():
    """ResourceModelWrapper.get_permitted_nodegroups() should default to all permitted"""
    graph = create_test_graph()
    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)

    permissions = wrapper.get_permitted_nodegroups()

    # Should have permissions for root
    assert len(permissions) >= 1
    assert permissions.get("") == True  # root


def test_set_permitted_nodegroups_should_set_permissions_by_nodegroup_id():
    """ResourceModelWrapper.set_permitted_nodegroups() should set permissions by nodegroup ID"""
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
    )

    field_node = StaticNode(
        nodeid="field",
        name="Field",
        datatype="string",
        nodegroup_id="field-ng",
        alias="field",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=0,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
    )

    graph = StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root_node, field_node],
        edges=[],
        nodegroups=[],
        root=root_node,
        cards=[],
        config={}
    )

    wrapper = ResourceModelWrapper(create_test_wkrm(graph), graph)

    permissions = {
        "": True,
        "field-ng": False
    }
    wrapper.set_permitted_nodegroups(permissions)

    assert wrapper.is_nodegroup_permitted("field-ng", None) == False


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
