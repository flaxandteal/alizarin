use alizarin::model_wrapper::ResourceModelWrapperCore;
use alizarin::graph::WKRM;
use alizarin_core::{StaticGraph, StaticNode, StaticEdge, StaticNodegroup, StaticCard, StaticGraphMeta, StaticTranslatableString};
use std::sync::Arc;
use std::collections::HashMap;

#[test]
fn test_prune_graph_filters_unpermitted_nodegroups() {
    // Create a simple graph with 3 nodes: root, child1 (permitted), child2 (not permitted)
    let root_node = StaticNode {
        nodeid: "root".to_string(),
        name: "Root".to_string(),
        datatype: "semantic".to_string(),
        nodegroup_id: None,
        alias: Some("".to_string()),
        graph_id: "test_graph".to_string(),
        is_collector: false,
        isrequired: false,
        exportable: true,
        sortorder: None,
        config: HashMap::new(),
        parentproperty: None,
        ontologyclass: None,
        description: None,
        fieldname: None,
        hascustomalias: false,
        issearchable: false,
        istopnode: true,
        sourcebranchpublication_id: None,
    };

    let child1_node = StaticNode {
        nodeid: "child1".to_string(),
        name: "Child 1".to_string(),
        datatype: "string".to_string(),
        nodegroup_id: Some("child1".to_string()),
        alias: Some("child1".to_string()),
        graph_id: "test_graph".to_string(),
        is_collector: false,
        isrequired: false,
        exportable: true,
        sortorder: None,
        config: HashMap::new(),
        parentproperty: None,
        ontologyclass: None,
        description: None,
        fieldname: None,
        hascustomalias: false,
        issearchable: false,
        istopnode: false,
        sourcebranchpublication_id: None,
    };

    let child2_node = StaticNode {
        nodeid: "child2".to_string(),
        name: "Child 2".to_string(),
        datatype: "string".to_string(),
        nodegroup_id: Some("child2".to_string()),
        alias: Some("child2".to_string()),
        graph_id: "test_graph".to_string(),
        is_collector: false,
        isrequired: false,
        exportable: true,
        sortorder: None,
        config: HashMap::new(),
        parentproperty: None,
        ontologyclass: None,
        description: None,
        fieldname: None,
        hascustomalias: false,
        issearchable: false,
        istopnode: false,
        sourcebranchpublication_id: None,
    };

    let edge1 = StaticEdge {
        domainnode_id: "root".to_string(),
        rangenode_id: "child1".to_string(),
        edgeid: "edge1".to_string(),
        graph_id: "test_graph".to_string(),
        name: None,
        ontologyproperty: None,
        description: None,
    };

    let edge2 = StaticEdge {
        domainnode_id: "root".to_string(),
        rangenode_id: "child2".to_string(),
        edgeid: "edge2".to_string(),
        graph_id: "test_graph".to_string(),
        name: None,
        ontologyproperty: None,
        description: None,
    };

    let nodegroup1 = StaticNodegroup {
        nodegroupid: "child1".to_string(),
        cardinality: Some("n".to_string()),
        parentnodegroup_id: None,
        legacygroupid: None,
    };

    let nodegroup2 = StaticNodegroup {
        nodegroupid: "child2".to_string(),
        cardinality: Some("n".to_string()),
        parentnodegroup_id: None,
        legacygroupid: None,
    };

    let card1 = StaticCard {
        nodegroup_id: "child1".to_string(),
        cardid: "card1".to_string(),
        active: true,
        component_id: "default-card".to_string(),
        config: None,
        constraints: vec![],
        cssclass: None,
        description: None,
        graph_id: "test_graph".to_string(),
        helpenabled: false,
        helptext: StaticTranslatableString::from_string("".to_string()),
        helptitle: StaticTranslatableString::from_string("".to_string()),
        instructions: StaticTranslatableString::from_string("".to_string()),
        is_editable: Some(true),
        name: StaticTranslatableString::from_string("Card 1".to_string()),
        sortorder: None,
        visible: true,
    };

    let card2 = StaticCard {
        nodegroup_id: "child2".to_string(),
        cardid: "card2".to_string(),
        active: true,
        component_id: "default-card".to_string(),
        config: None,
        constraints: vec![],
        cssclass: None,
        description: None,
        graph_id: "test_graph".to_string(),
        helpenabled: false,
        helptext: StaticTranslatableString::from_string("".to_string()),
        helptitle: StaticTranslatableString::from_string("".to_string()),
        instructions: StaticTranslatableString::from_string("".to_string()),
        is_editable: Some(true),
        name: StaticTranslatableString::from_string("Card 2".to_string()),
        sortorder: None,
        visible: true,
    };

    // Create graph using JSON to handle private fields
    let graph_json = serde_json::json!({
        "graphid": "test_graph",
        "name": {"en": "Test Graph"},
        "nodes": [root_node.clone(), child1_node, child2_node],
        "edges": [edge1, edge2],
        "nodegroups": [nodegroup1, nodegroup2],
        "root": root_node,
        "cards": [card1, card2],
        "cards_x_nodes_x_widgets": [],
        "functions_x_graphs": [],
        "config": {}
    });

    let mut graph: StaticGraph = serde_json::from_value(graph_json).unwrap();
    graph.build_indices();

    // Create StaticGraphMeta for WKRM
    let meta = StaticGraphMeta {
        graphid: "test_graph".to_string(),
        name: Some(StaticTranslatableString::from_string("Test Graph".to_string())),
        author: None,
        cards: None,
        cards_x_nodes_x_widgets: None,
        color: None,
        description: None,
        edges: None,
        iconclass: None,
        is_editable: None,
        isresource: None,
        jsonldcontext: None,
        nodegroups: None,
        nodes: None,
        ontology_id: None,
        publication: None,
        relatable_resource_model_ids: vec![],
        resource_2_resource_constraints: None,
        root: None,
        slug: None,
        subtitle: None,
        version: None,
        extra_fields: HashMap::new(),
    };

    let wkrm = WKRM::from_meta(meta);

    let mut wrapper = ResourceModelWrapperCore::new(wkrm, Arc::new(graph), false);

    // Set permissions: only child1 is permitted
    let mut permissions = HashMap::new();
    permissions.insert("child1".to_string(), true);
    permissions.insert("child2".to_string(), false);
    permissions.insert("".to_string(), true); // root
    wrapper.set_permitted_nodegroups(permissions);

    // Prune the graph
    wrapper.prune_graph(None).expect("Prune should succeed");

    // Verify that only permitted nodes/edges remain
    let pruned_graph = wrapper.get_graph();

    // Should have root + child1 (2 nodes)
    assert_eq!(pruned_graph.nodes.len(), 2, "Should have 2 nodes after pruning");
    assert!(pruned_graph.nodes.iter().any(|n| n.nodeid == "root"));
    assert!(pruned_graph.nodes.iter().any(|n| n.nodeid == "child1"));
    assert!(!pruned_graph.nodes.iter().any(|n| n.nodeid == "child2"));

    // Should have 1 edge (root -> child1)
    assert_eq!(pruned_graph.edges.len(), 1, "Should have 1 edge after pruning");
    assert_eq!(pruned_graph.edges[0].rangenode_id, "child1");

    // Should have 1 nodegroup (child1)
    assert_eq!(pruned_graph.nodegroups.len(), 1, "Should have 1 nodegroup after pruning");
    assert_eq!(pruned_graph.nodegroups[0].nodegroupid, "child1");

    // Should have 1 card (for child1)
    assert_eq!(pruned_graph.cards.as_ref().unwrap().len(), 1, "Should have 1 card after pruning");
    assert_eq!(pruned_graph.cards.as_ref().unwrap()[0].nodegroup_id, "child1");
}
