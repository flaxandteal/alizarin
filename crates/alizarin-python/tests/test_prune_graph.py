"""
Tests for graph pruning based on permissions.

Ports tests from tests/prune_graph_test.rs
Tests that unpermitted nodegroups are correctly filtered from the graph.
"""

import pytest
from alizarin.static_types import (
    StaticGraph, StaticNode, StaticEdge, StaticNodegroup,
    StaticCard
)
from alizarin.model_wrapper import ResourceModelWrapper
from alizarin.graph_manager import WKRM


def test_prune_graph_filters_unpermitted_nodegroups():
    """
    Test that pruning correctly removes unpermitted nodes, edges, nodegroups, and cards.

    Creates a simple graph with 3 nodes: root, child1 (permitted), child2 (not permitted)
    After pruning, only root and child1 should remain.
    """
    # Create root node
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test_graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=None,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
        sourcebranchpublication_id=None
    )

    # Create child1 node (permitted)
    child1_node = StaticNode(
        nodeid="child1",
        name="Child 1",
        datatype="string",
        nodegroup_id="child1",
        alias="child1",
        graph_id="test_graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=None,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
        sourcebranchpublication_id=None
    )

    # Create child2 node (not permitted)
    child2_node = StaticNode(
        nodeid="child2",
        name="Child 2",
        datatype="string",
        nodegroup_id="child2",
        alias="child2",
        graph_id="test_graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=None,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
        sourcebranchpublication_id=None
    )

    # Create edges
    edge1 = StaticEdge(
        domainnode_id="root",
        rangenode_id="child1",
        edgeid="edge1",
        graph_id="test_graph",
        name=None,
        ontologyproperty=None,
        description=None
    )

    edge2 = StaticEdge(
        domainnode_id="root",
        rangenode_id="child2",
        edgeid="edge2",
        graph_id="test_graph",
        name=None,
        ontologyproperty=None,
        description=None
    )

    # Create nodegroups
    nodegroup1 = StaticNodegroup(
        nodegroupid="child1",
        cardinality="n",
        parentnodegroup_id=None,
        legacygroupid=None
    )

    nodegroup2 = StaticNodegroup(
        nodegroupid="child2",
        cardinality="n",
        parentnodegroup_id=None,
        legacygroupid=None
    )

    # Create cards
    card1 = StaticCard(
        nodegroup_id="child1",
        cardid="card1",
        active=True,
        component_id="default-card",
        config=None,
        constraints=[],
        cssclass=None,
        description=None,
        graph_id="test_graph",
        helpenabled=False,
        helptext={},
        helptitle={},
        instructions={},
        is_editable=True,
        name={"en": "Card 1"},
        sortorder=None,
        visible=True
    )

    card2 = StaticCard(
        nodegroup_id="child2",
        cardid="card2",
        active=True,
        component_id="default-card",
        config=None,
        constraints=[],
        cssclass=None,
        description=None,
        graph_id="test_graph",
        helpenabled=False,
        helptext={},
        helptitle={},
        instructions={},
        is_editable=True,
        name={"en": "Card 2"},
        sortorder=None,
        visible=True
    )

    # Create graph
    graph = StaticGraph(
        graphid="test_graph",
        name={"en": "Test Graph"},
        nodes=[root_node, child1_node, child2_node],
        edges=[edge1, edge2],
        nodegroups=[nodegroup1, nodegroup2],
        root=root_node,
        cards=[card1, card2],
        author=None,
        config={}
    )

    # Create WKRM
    meta = {
        'graphid': 'test_graph',
        'name': {'en': 'Test Graph'},
        'slug': 'test_graph',
        'relatable_resource_model_ids': []
    }
    wkrm = WKRM(meta)

    # Create wrapper
    wrapper = ResourceModelWrapper(wkrm, graph, False)

    # Set permissions: only child1 is permitted
    permissions = {
        "child1": True,
        "child2": False,
        "": True  # root
    }
    wrapper.set_permitted_nodegroups(permissions)

    # Prune the graph
    wrapper.prune_graph()

    # Verify that only permitted nodes/edges remain
    pruned_graph = wrapper.graph

    # Should have root + child1 (2 nodes)
    assert len(pruned_graph.nodes) == 2, "Should have 2 nodes after pruning"
    assert any(n.nodeid == "root" for n in pruned_graph.nodes)
    assert any(n.nodeid == "child1" for n in pruned_graph.nodes)
    assert not any(n.nodeid == "child2" for n in pruned_graph.nodes)

    # Should have 1 edge (root -> child1)
    assert len(pruned_graph.edges) == 1, "Should have 1 edge after pruning"
    assert pruned_graph.edges[0].rangenode_id == "child1"

    # Should have 1 nodegroup (child1)
    assert len(pruned_graph.nodegroups) == 1, "Should have 1 nodegroup after pruning"
    assert pruned_graph.nodegroups[0].nodegroupid == "child1"

    # Should have 1 card (for child1)
    assert len(pruned_graph.cards) == 1, "Should have 1 card after pruning"
    assert pruned_graph.cards[0].nodegroup_id == "child1"


def test_prune_graph_preserves_parent_nodes():
    """
    Test that pruning preserves parent nodes even if they're not directly permitted,
    as long as they have permitted children.
    """
    # Create root node
    root_node = StaticNode(
        nodeid="root",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="",
        graph_id="test_graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=None,
        hascustomalias=False,
        issearchable=False,
        istopnode=True,
        sourcebranchpublication_id=None
    )

    # Create parent semantic node
    parent_node = StaticNode(
        nodeid="parent",
        name="Parent",
        datatype="semantic",
        nodegroup_id="parent",
        alias="parent",
        graph_id="test_graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=None,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
        sourcebranchpublication_id=None
    )

    # Create child node (permitted)
    child_node = StaticNode(
        nodeid="child",
        name="Child",
        datatype="string",
        nodegroup_id="child",
        alias="child",
        graph_id="test_graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        sortorder=None,
        hascustomalias=False,
        issearchable=False,
        istopnode=False,
        sourcebranchpublication_id=None
    )

    # Create edges
    edge1 = StaticEdge(
        domainnode_id="root",
        rangenode_id="parent",
        edgeid="edge1",
        graph_id="test_graph"
    )

    edge2 = StaticEdge(
        domainnode_id="parent",
        rangenode_id="child",
        edgeid="edge2",
        graph_id="test_graph"
    )

    # Create nodegroups
    nodegroup_parent = StaticNodegroup(nodegroupid="parent", cardinality="n")
    nodegroup_child = StaticNodegroup(nodegroupid="child", cardinality="1")

    # Create graph
    graph = StaticGraph(
        graphid="test_graph",
        name={"en": "Test Graph"},
        nodes=[root_node, parent_node, child_node],
        edges=[edge1, edge2],
        nodegroups=[nodegroup_parent, nodegroup_child],
        root=root_node,
        cards=[],
        config={}
    )

    meta = {
        'graphid': 'test_graph',
        'name': {'en': 'Test Graph'},
        'slug': 'test_graph',
        'relatable_resource_model_ids': []
    }
    wkrm = WKRM(meta)

    wrapper = ResourceModelWrapper(wkrm, graph, False)

    # Only permit the child, not the parent nodegroup
    permissions = {
        "": True,
        "child": True,
        # parent is implicitly not in permissions, defaults to True
    }
    wrapper.set_permitted_nodegroups(permissions)

    wrapper.prune_graph()

    # Parent should still exist because child is permitted
    assert any(n.alias == "parent" for n in wrapper.graph.nodes), \
        "Parent node should be preserved when child is permitted"
    assert any(n.alias == "child" for n in wrapper.graph.nodes), \
        "Child node should exist"


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
