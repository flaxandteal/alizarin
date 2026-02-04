//! Graph pruning utilities
//!
//! Provides functions for pruning a graph to only include permitted nodegroups
//! and their dependencies. This is useful for permission-based filtering.

use std::collections::{HashMap, HashSet};
use super::{StaticGraph, StaticNode};

/// Maximum depth for edge traversal to prevent infinite loops
const MAX_GRAPH_DEPTH: usize = 100;

/// Error type for graph pruning operations
#[derive(Debug, Clone)]
pub enum PruneError {
    /// Node has multiple parents (malformed graph)
    MultipleParents { node: String, parent1: String, parent2: String },
    /// Node has no parent but is not root (disconnected)
    NoParent { node: String },
    /// Edge traversal hit depth limit (likely cycle)
    CycleDetected,
    /// Graph has no root node
    NoRootNode,
}

impl std::fmt::Display for PruneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PruneError::MultipleParents { node, parent1, parent2 } => {
                write!(f, "Graph is malformed, node {} has multiple parents: {} and {}",
                       node, parent1, parent2)
            }
            PruneError::NoParent { node } => {
                write!(f, "Graph does not have a parent for {}", node)
            }
            PruneError::CycleDetected => {
                write!(f, "Hit edge traversal limit when pruning, is the graph well-formed without cycles?")
            }
            PruneError::NoRootNode => {
                write!(f, "Could not find root node in graph")
            }
        }
    }
}

impl std::error::Error for PruneError {}

/// Find the root node of a graph
///
/// The root node is the node with no nodegroup_id or an empty nodegroup_id.
pub fn find_root_node(graph: &StaticGraph) -> Option<&StaticNode> {
    graph.nodes.iter().find(|node| {
        node.nodegroup_id.is_none() ||
        node.nodegroup_id.as_ref().map(|s| s.is_empty()).unwrap_or(true)
    })
}

/// Build backedges map (child -> parent) from a graph's edges
///
/// Returns an error if any node has multiple parents.
pub fn build_backedges(graph: &StaticGraph) -> Result<HashMap<String, String>, PruneError> {
    let mut backedges: HashMap<String, String> = HashMap::new();

    for edge in &graph.edges {
        if let Some(existing_parent) = backedges.get(&edge.rangenode_id) {
            return Err(PruneError::MultipleParents {
                node: edge.rangenode_id.clone(),
                parent1: existing_parent.clone(),
                parent2: edge.domainnode_id.clone(),
            });
        }
        backedges.insert(edge.rangenode_id.clone(), edge.domainnode_id.clone());
    }

    Ok(backedges)
}

/// Prune a graph to only include permitted nodegroups and their dependencies.
///
/// # Arguments
/// * `graph` - The graph to prune
/// * `is_nodegroup_permitted` - Function that returns true if a nodegroup is permitted
/// * `keep_functions` - Optional list of function IDs to keep (if None, all functions are removed)
///
/// # Returns
/// A new pruned graph containing only permitted nodes, edges, cards, etc.
///
/// # Errors
/// Returns an error if the graph is malformed (multiple parents, cycles, etc.)
pub fn prune_graph<F>(
    graph: &StaticGraph,
    is_nodegroup_permitted: F,
    keep_functions: Option<&[String]>,
) -> Result<StaticGraph, PruneError>
where
    F: Fn(&str) -> bool,
{
    // Find root node
    let root_node = find_root_node(graph).ok_or(PruneError::NoRootNode)?;
    let root = root_node.nodeid.clone();

    // Build nodegroup set from nodes
    let all_nodegroups: HashSet<String> = graph.nodes.iter()
        .filter_map(|n| n.nodegroup_id.clone())
        .collect();

    // Build allowed_nodegroups map: nodegroup_id -> is_rooted
    // Filter to only permitted nodegroups
    let mut allowed_nodegroups: HashMap<String, bool> = all_nodegroups.iter()
        .filter(|ng_id| is_nodegroup_permitted(ng_id))
        .map(|ng_id| {
            let is_root = ng_id.is_empty() || *ng_id == root;
            (ng_id.clone(), is_root)
        })
        .collect();

    // Build backedges map (child -> parent)
    let backedges = build_backedges(graph)?;

    // Mark root as rooted
    allowed_nodegroups.insert(root.clone(), true);

    // Iteratively ensure all kept nodegroups have path to root
    let mut loops = 0;
    while loops < MAX_GRAPH_DEPTH {
        let unrooted: Vec<String> = allowed_nodegroups.iter()
            .filter(|(_, &rooted)| !rooted)
            .map(|(ng, _)| ng.clone())
            .collect();

        if unrooted.is_empty() {
            break;
        }

        for ng in unrooted {
            if ng == root {
                continue;
            }

            let next = backedges.get(&ng)
                .ok_or_else(|| PruneError::NoParent { node: ng.clone() })?;

            allowed_nodegroups.insert(ng.clone(), true);
            if !allowed_nodegroups.contains_key(next) {
                allowed_nodegroups.insert(next.clone(), false);
            }
        }

        loops += 1;
    }

    if loops >= MAX_GRAPH_DEPTH {
        return Err(PruneError::CycleDetected);
    }

    // Build set of allowed node IDs
    let allowed_nodes: HashSet<String> = graph.nodes.iter()
        .filter(|node| {
            node.nodegroup_id.as_ref()
                .and_then(|ng_id| allowed_nodegroups.get(ng_id))
                .copied()
                .unwrap_or(false)
                || node.nodeid == root
        })
        .map(|node| node.nodeid.clone())
        .collect();

    // Create pruned graph
    let mut pruned = graph.clone();

    // Filter cards
    pruned.cards = pruned.cards.map(|cards| {
        cards.into_iter()
            .filter(|card| allowed_nodegroups.get(&card.nodegroup_id).copied().unwrap_or(false))
            .collect()
    });

    // Filter cards_x_nodes_x_widgets
    pruned.cards_x_nodes_x_widgets = pruned.cards_x_nodes_x_widgets.map(|cxnxws| {
        cxnxws.into_iter()
            .filter(|cxnxw| allowed_nodes.contains(&cxnxw.node_id))
            .collect()
    });

    // Filter edges
    pruned.edges = pruned.edges.into_iter()
        .filter(|edge| {
            (edge.domainnode_id == root || allowed_nodes.contains(&edge.domainnode_id))
                && allowed_nodes.contains(&edge.rangenode_id)
        })
        .collect();

    // Filter nodegroups
    pruned.nodegroups = pruned.nodegroups.into_iter()
        .filter(|ng| allowed_nodegroups.contains_key(&ng.nodegroupid))
        .collect();

    // Filter nodes
    pruned.nodes = pruned.nodes.into_iter()
        .filter(|node| allowed_nodes.contains(&node.nodeid))
        .collect();

    // Filter functions_x_graphs
    if let Some(keep_fns) = keep_functions {
        pruned.functions_x_graphs = pruned.functions_x_graphs.map(|fxgs| {
            fxgs.into_iter()
                .filter(|fxg| keep_fns.contains(&fxg.function_id))
                .collect()
        });
    } else {
        pruned.functions_x_graphs = Some(Vec::new());
    }

    Ok(pruned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{StaticEdge, StaticNodegroup};
    use serde_json::json;

    fn create_test_node(nodeid: &str, nodegroup_id: Option<&str>) -> StaticNode {
        let node_json = json!({
            "nodeid": nodeid,
            "name": nodeid,
            "datatype": if nodegroup_id.is_none() { "semantic" } else { "string" },
            "nodegroup_id": nodegroup_id,
            "alias": nodeid,
            "graph_id": "test_graph",
            "is_collector": false,
            "isrequired": false,
            "exportable": true,
            "issearchable": false,
            "istopnode": nodegroup_id.is_none(),
        });
        serde_json::from_value(node_json).expect("Failed to create test node")
    }

    fn create_test_edge(domain: &str, range: &str) -> StaticEdge {
        StaticEdge {
            edgeid: format!("{}->{}", domain, range),
            domainnode_id: domain.to_string(),
            rangenode_id: range.to_string(),
            graph_id: "test_graph".to_string(),
            name: None,
            description: None,
            ontologyproperty: None,
            source_identifier_id: None,
        }
    }

    fn create_test_nodegroup(nodegroupid: &str) -> StaticNodegroup {
        StaticNodegroup {
            nodegroupid: nodegroupid.to_string(),
            cardinality: Some("n".to_string()),
            parentnodegroup_id: None,
            legacygroupid: None,
            grouping_node_id: None,
        }
    }

    #[test]
    fn test_prune_graph_filters_unpermitted_nodegroups() {
        // Create nodes: root, child1 (permitted), child2 (not permitted)
        let root = create_test_node("root", None);
        let child1 = create_test_node("child1", Some("child1"));
        let child2 = create_test_node("child2", Some("child2"));

        let edge1 = create_test_edge("root", "child1");
        let edge2 = create_test_edge("root", "child2");

        let ng1 = create_test_nodegroup("child1");
        let ng2 = create_test_nodegroup("child2");

        let graph_json = json!({
            "graphid": "test_graph",
            "name": {"en": "Test Graph"},
            "nodes": [root.clone(), child1, child2],
            "edges": [edge1, edge2],
            "nodegroups": [ng1, ng2],
            "root": root,
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "functions_x_graphs": [],
            "config": {}
        });

        let graph: StaticGraph = serde_json::from_value(graph_json).expect("Failed to create graph");

        // Only permit child1
        let permitted = |ng: &str| ng == "child1";

        let pruned = prune_graph(&graph, permitted, None).expect("Prune failed");

        // Verify child1 is included, child2 is not
        assert!(pruned.nodes.iter().any(|n| n.nodeid == "root"), "Root should be included");
        assert!(pruned.nodes.iter().any(|n| n.nodeid == "child1"), "child1 should be included");
        assert!(!pruned.nodes.iter().any(|n| n.nodeid == "child2"), "child2 should NOT be included");

        // Verify nodegroups
        assert!(pruned.nodegroups.iter().any(|ng| ng.nodegroupid == "child1"));
        assert!(!pruned.nodegroups.iter().any(|ng| ng.nodegroupid == "child2"));

        // Verify edges
        assert!(pruned.edges.iter().any(|e| e.rangenode_id == "child1"));
        assert!(!pruned.edges.iter().any(|e| e.rangenode_id == "child2"));
    }

    #[test]
    fn test_prune_graph_includes_path_to_root() {
        // Create a chain: root -> middle -> leaf
        // If only leaf is permitted, middle should also be included to maintain path
        let root = create_test_node("root", None);
        let middle = create_test_node("middle", Some("middle"));
        let leaf = create_test_node("leaf", Some("leaf"));

        let edge1 = create_test_edge("root", "middle");
        let edge2 = create_test_edge("middle", "leaf");

        let ng_middle = create_test_nodegroup("middle");
        let ng_leaf = create_test_nodegroup("leaf");

        let graph_json = json!({
            "graphid": "test_graph",
            "name": {"en": "Test Graph"},
            "nodes": [root.clone(), middle, leaf],
            "edges": [edge1, edge2],
            "nodegroups": [ng_middle, ng_leaf],
            "root": root,
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "functions_x_graphs": [],
            "config": {}
        });

        let graph: StaticGraph = serde_json::from_value(graph_json).expect("Failed to create graph");

        // Only permit leaf - middle should still be included for path to root
        let permitted = |ng: &str| ng == "leaf";

        let pruned = prune_graph(&graph, permitted, None).expect("Prune failed");

        // All nodes should be included to maintain path
        assert!(pruned.nodes.iter().any(|n| n.nodeid == "root"));
        assert!(pruned.nodes.iter().any(|n| n.nodeid == "middle"), "middle should be included for path");
        assert!(pruned.nodes.iter().any(|n| n.nodeid == "leaf"));
    }

    #[test]
    fn test_prune_graph_detects_multiple_parents() {
        let root = create_test_node("root", None);
        let child = create_test_node("child", Some("child"));

        // Two edges pointing to same child = multiple parents
        let edge1 = create_test_edge("root", "child");
        let mut edge2 = create_test_edge("root", "child");
        edge2.domainnode_id = "other_parent".to_string();

        let ng = create_test_nodegroup("child");

        let graph_json = json!({
            "graphid": "test_graph",
            "name": {"en": "Test Graph"},
            "nodes": [root.clone(), child],
            "edges": [edge1, edge2],
            "nodegroups": [ng],
            "root": root,
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "functions_x_graphs": [],
            "config": {}
        });

        let graph: StaticGraph = serde_json::from_value(graph_json).expect("Failed to create graph");

        let result = prune_graph(&graph, |_| true, None);
        assert!(matches!(result, Err(PruneError::MultipleParents { .. })));
    }
}
