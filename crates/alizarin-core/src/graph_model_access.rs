/// Platform-agnostic ModelAccess implementation built from a StaticGraph.
///
/// Used by Python, NAPI, and potentially WASM bindings to avoid duplicating
/// the graph-index-building logic across crates.
use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::{StaticGraph, StaticNode, StaticNodegroup};
use crate::instance_wrapper_core::ModelAccess;
use crate::permissions::PermissionRule;

/// ModelAccess implementation built directly from a StaticGraph.
///
/// Constructs node, edge, and nodegroup indices once via `from_graph()`,
/// then serves them through the `ModelAccess` trait.
pub struct GraphModelAccess {
    nodes: HashMap<String, Arc<StaticNode>>,
    nodes_by_alias: HashMap<String, Arc<StaticNode>>,
    edges: HashMap<String, Vec<String>>,
    reverse_edges: HashMap<String, Vec<String>>,
    nodes_by_nodegroup: HashMap<String, Vec<Arc<StaticNode>>>,
    nodegroups: HashMap<String, Arc<StaticNodegroup>>,
    root_node_id: String,
    permitted_nodegroups: HashMap<String, PermissionRule>,
}

impl GraphModelAccess {
    /// Build indices from a StaticGraph.
    pub fn from_graph(graph: &StaticGraph) -> Self {
        let mut nodes: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut nodes_by_alias: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut nodes_by_nodegroup: HashMap<String, Vec<Arc<StaticNode>>> = HashMap::new();
        let mut nodegroups: HashMap<String, Arc<StaticNodegroup>> = HashMap::new();
        let mut root_node_id = String::new();

        // Build node index
        for node in &graph.nodes {
            let mut node_copy = node.clone();
            // Ensure root node (node without nodegroup_id) has alias set
            if (node_copy.nodegroup_id.is_none()
                || node_copy
                    .nodegroup_id
                    .as_ref()
                    .map(|s| s.is_empty())
                    .unwrap_or(false))
                && node_copy.alias.is_none()
            {
                node_copy.alias = Some(String::new());
            }
            let arc_node = Arc::new(node_copy);
            nodes.insert(arc_node.nodeid.clone(), Arc::clone(&arc_node));

            // Build alias index
            if let Some(ref alias) = arc_node.alias {
                if !alias.is_empty() {
                    nodes_by_alias.insert(alias.clone(), Arc::clone(&arc_node));
                } else {
                    nodes_by_alias.insert(String::new(), Arc::clone(&arc_node));
                }
            } else {
                nodes_by_alias.insert(String::new(), Arc::clone(&arc_node));
            }

            if arc_node.istopnode {
                root_node_id = arc_node.nodeid.clone();
            }

            if let Some(ref ng_id) = arc_node.nodegroup_id {
                nodes_by_nodegroup
                    .entry(ng_id.clone())
                    .or_default()
                    .push(Arc::clone(&arc_node));
            }
        }

        // Build edge indices
        for edge in &graph.edges {
            let parent_id = edge.domainnode_id.clone();
            let child_id = edge.rangenode_id.clone();

            edges
                .entry(parent_id.clone())
                .or_default()
                .push(child_id.clone());

            reverse_edges.entry(child_id).or_default().push(parent_id);
        }

        // Build nodegroup index — first from nodes, then merge actual nodegroups
        for node in &graph.nodes {
            if let Some(ref ng_id) = node.nodegroup_id {
                if !ng_id.is_empty() && !nodegroups.contains_key(ng_id) {
                    nodegroups.insert(
                        ng_id.clone(),
                        Arc::new(StaticNodegroup {
                            cardinality: Some("n".to_string()),
                            legacygroupid: None,
                            nodegroupid: ng_id.clone(),
                            parentnodegroup_id: None,
                            grouping_node_id: None,
                        }),
                    );
                }
            }
        }
        for ng in &graph.nodegroups {
            nodegroups.insert(ng.nodegroupid.clone(), Arc::new(ng.clone()));
        }

        GraphModelAccess {
            nodes,
            nodes_by_alias,
            edges,
            reverse_edges,
            nodes_by_nodegroup,
            nodegroups,
            root_node_id,
            permitted_nodegroups: HashMap::new(),
        }
    }

    /// Set permitted nodegroups (for permission filtering).
    pub fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, PermissionRule>) {
        self.permitted_nodegroups = permissions;
    }

    /// Get nodes indexed by alias.
    pub fn get_nodes_by_alias(&self) -> &HashMap<String, Arc<StaticNode>> {
        &self.nodes_by_alias
    }

    /// Get child nodes for a given node ID, keyed by alias.
    pub fn get_child_nodes(&self, node_id: &str) -> HashMap<String, Arc<StaticNode>> {
        let mut child_nodes = HashMap::new();
        if let Some(child_ids) = self.edges.get(node_id) {
            for child_id in child_ids {
                if let Some(node) = self.nodes.get(child_id) {
                    if let Some(ref alias) = node.alias {
                        if !alias.is_empty() {
                            child_nodes.insert(alias.clone(), Arc::clone(node));
                        }
                    }
                }
            }
        }
        child_nodes
    }

    /// Check if a nodegroup is permitted.
    pub fn is_nodegroup_permitted(&self, nodegroup_id: &str, default_allow: bool) -> bool {
        self.permitted_nodegroups
            .get(nodegroup_id)
            .map(|rule| rule.permits_nodegroup())
            .unwrap_or(default_allow)
    }

    /// Get permitted nodegroups as a boolean map (for backward compat).
    pub fn get_permitted_nodegroups_bool(&self) -> HashMap<String, bool> {
        if self.permitted_nodegroups.is_empty() {
            // Default: all nodegroups permitted
            let mut perms: HashMap<String, bool> =
                self.nodegroups.keys().map(|k| (k.clone(), true)).collect();
            perms.insert(String::new(), true);
            return perms;
        }
        self.permitted_nodegroups
            .iter()
            .map(|(k, v)| (k.clone(), v.permits_nodegroup()))
            .collect()
    }

    /// Rebuild indices from a new graph (e.g., after pruning).
    pub fn rebuild_from_graph(&mut self, graph: &StaticGraph) {
        let fresh = Self::from_graph(graph);
        self.nodes = fresh.nodes;
        self.nodes_by_alias = fresh.nodes_by_alias;
        self.edges = fresh.edges;
        self.reverse_edges = fresh.reverse_edges;
        self.nodes_by_nodegroup = fresh.nodes_by_nodegroup;
        self.nodegroups = fresh.nodegroups;
        self.root_node_id = fresh.root_node_id;
        // Keep existing permissions
    }
}

impl ModelAccess for GraphModelAccess {
    fn get_nodes(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        Some(&self.nodes)
    }

    fn get_edges(&self) -> Option<&HashMap<String, Vec<String>>> {
        Some(&self.edges)
    }

    fn get_reverse_edges(&self) -> Option<&HashMap<String, Vec<String>>> {
        Some(&self.reverse_edges)
    }

    fn get_nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<Arc<StaticNode>>>> {
        Some(&self.nodes_by_nodegroup)
    }

    fn get_nodegroups(&self) -> Option<&HashMap<String, Arc<StaticNodegroup>>> {
        Some(&self.nodegroups)
    }

    fn get_root_node(&self) -> Result<Arc<StaticNode>, String> {
        self.nodes
            .get(&self.root_node_id)
            .cloned()
            .ok_or_else(|| "Root node not found".to_string())
    }

    fn get_permitted_nodegroups(&self) -> HashMap<String, PermissionRule> {
        self.permitted_nodegroups.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{StaticEdge, StaticNodegroup};
    use crate::instance_wrapper_core::ModelAccess;
    use serde_json::json;
    use std::collections::HashSet;

    /// Build a test graph:
    /// ```
    /// root (semantic, alias="root", istopnode=true, nodegroup_id=None)
    /// ├── child_a (string, alias="name", nodegroup_id="ng1")
    /// ├── child_b (semantic, alias="details", nodegroup_id="ng2")
    /// │   └── grandchild (string, alias="description", nodegroup_id="ng2")
    /// ```
    fn create_test_graph() -> StaticGraph {
        let root: StaticNode = serde_json::from_value(json!({
            "nodeid": "root-id",
            "name": "Root",
            "alias": "root",
            "datatype": "semantic",
            "graph_id": "test-graph",
            "is_collector": false,
            "isrequired": false,
            "issearchable": false,
            "istopnode": true,
            "sortorder": 0
        }))
        .unwrap();

        let child_a: StaticNode = serde_json::from_value(json!({
            "nodeid": "child-a-id",
            "name": "Name",
            "alias": "name",
            "datatype": "string",
            "nodegroup_id": "ng1",
            "graph_id": "test-graph",
            "is_collector": false,
            "isrequired": false,
            "issearchable": false,
            "istopnode": false,
            "sortorder": 0
        }))
        .unwrap();

        let child_b: StaticNode = serde_json::from_value(json!({
            "nodeid": "child-b-id",
            "name": "Details",
            "alias": "details",
            "datatype": "semantic",
            "nodegroup_id": "ng2",
            "graph_id": "test-graph",
            "is_collector": true,
            "isrequired": false,
            "issearchable": false,
            "istopnode": false,
            "sortorder": 1
        }))
        .unwrap();

        let grandchild: StaticNode = serde_json::from_value(json!({
            "nodeid": "grandchild-id",
            "name": "Description",
            "alias": "description",
            "datatype": "string",
            "nodegroup_id": "ng2",
            "graph_id": "test-graph",
            "is_collector": false,
            "isrequired": false,
            "issearchable": false,
            "istopnode": false,
            "sortorder": 0
        }))
        .unwrap();

        let ng1 = StaticNodegroup {
            cardinality: Some("n".to_string()),
            legacygroupid: None,
            nodegroupid: "ng1".to_string(),
            parentnodegroup_id: None,
            grouping_node_id: None,
        };
        let ng2 = StaticNodegroup {
            cardinality: Some("1".to_string()),
            legacygroupid: None,
            nodegroupid: "ng2".to_string(),
            parentnodegroup_id: None,
            grouping_node_id: None,
        };

        let edge_root_a: StaticEdge = serde_json::from_value(json!({
            "domainnode_id": "root-id",
            "rangenode_id": "child-a-id",
            "edgeid": "edge-1",
            "graph_id": "test-graph"
        }))
        .unwrap();

        let edge_root_b: StaticEdge = serde_json::from_value(json!({
            "domainnode_id": "root-id",
            "rangenode_id": "child-b-id",
            "edgeid": "edge-2",
            "graph_id": "test-graph"
        }))
        .unwrap();

        let edge_b_gc: StaticEdge = serde_json::from_value(json!({
            "domainnode_id": "child-b-id",
            "rangenode_id": "grandchild-id",
            "edgeid": "edge-3",
            "graph_id": "test-graph"
        }))
        .unwrap();

        serde_json::from_value(json!({
            "graphid": "test-graph",
            "name": {"en": "Test Graph"},
            "root": root,
            "nodes": [root.clone(), child_a, child_b, grandchild],
            "edges": [edge_root_a, edge_root_b, edge_b_gc],
            "nodegroups": [ng1, ng2]
        }))
        .unwrap()
    }

    #[test]
    fn from_graph_builds_node_index() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        assert_eq!(access.nodes.len(), 4);
        assert!(access.nodes.contains_key("root-id"));
        assert!(access.nodes.contains_key("child-a-id"));
        assert!(access.nodes.contains_key("child-b-id"));
        assert!(access.nodes.contains_key("grandchild-id"));
    }

    #[test]
    fn from_graph_builds_alias_index() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        assert!(access.nodes_by_alias.contains_key("name"));
        assert!(access.nodes_by_alias.contains_key("details"));
        assert!(access.nodes_by_alias.contains_key("description"));
        assert_eq!(
            access.nodes_by_alias.get("name").unwrap().nodeid,
            "child-a-id"
        );
    }

    #[test]
    fn from_graph_builds_edges() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let root_children = access.edges.get("root-id").unwrap();
        assert_eq!(root_children.len(), 2);
        assert!(root_children.contains(&"child-a-id".to_string()));
        assert!(root_children.contains(&"child-b-id".to_string()));
        let b_children = access.edges.get("child-b-id").unwrap();
        assert_eq!(b_children, &vec!["grandchild-id".to_string()]);
    }

    #[test]
    fn from_graph_builds_reverse_edges() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let gc_parents = access.reverse_edges.get("grandchild-id").unwrap();
        assert_eq!(gc_parents, &vec!["child-b-id".to_string()]);
        let a_parents = access.reverse_edges.get("child-a-id").unwrap();
        assert_eq!(a_parents, &vec!["root-id".to_string()]);
    }

    #[test]
    fn from_graph_builds_nodegroups() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        assert!(access.nodegroups.contains_key("ng1"));
        assert!(access.nodegroups.contains_key("ng2"));
        // ng2 should have cardinality from the explicit nodegroup definition
        assert_eq!(
            access.nodegroups.get("ng2").unwrap().cardinality,
            Some("1".to_string())
        );
    }

    #[test]
    fn from_graph_identifies_root() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        assert_eq!(access.root_node_id, "root-id");
    }

    #[test]
    fn from_graph_sets_root_alias_when_missing() {
        let mut graph = create_test_graph();
        // Modify root to have no alias and no nodegroup_id (top node pattern)
        graph.nodes[0].alias = None;
        graph.nodes[0].nodegroup_id = None;
        let access = GraphModelAccess::from_graph(&graph);
        // Should have empty-string alias entry for root
        assert!(access.nodes_by_alias.contains_key(""));
        assert_eq!(access.nodes_by_alias.get("").unwrap().nodeid, "root-id");
    }

    #[test]
    fn get_child_nodes_returns_children_by_alias() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let children = access.get_child_nodes("root-id");
        assert_eq!(children.len(), 2);
        assert!(children.contains_key("name"));
        assert!(children.contains_key("details"));
        assert_eq!(children.get("name").unwrap().nodeid, "child-a-id");
    }

    #[test]
    fn get_child_nodes_empty_for_leaf() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let children = access.get_child_nodes("grandchild-id");
        assert!(children.is_empty());
    }

    #[test]
    fn is_nodegroup_permitted_default_allow() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        // No permissions set — default_allow=true should return true
        assert!(access.is_nodegroup_permitted("ng1", true));
        // default_allow=false should return false
        assert!(!access.is_nodegroup_permitted("ng1", false));
    }

    #[test]
    fn is_nodegroup_permitted_explicit_deny() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::from_graph(&graph);
        let mut perms = HashMap::new();
        perms.insert("ng1".to_string(), PermissionRule::Boolean(true));
        perms.insert("ng2".to_string(), PermissionRule::Boolean(false));
        access.set_permitted_nodegroups(perms);
        assert!(access.is_nodegroup_permitted("ng1", true));
        assert!(!access.is_nodegroup_permitted("ng2", true));
    }

    #[test]
    fn is_nodegroup_permitted_conditional() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::from_graph(&graph);
        let mut perms = HashMap::new();
        perms.insert(
            "ng1".to_string(),
            PermissionRule::Conditional {
                path: ".data.field".to_string(),
                allowed: HashSet::from(["value1".to_string()]),
            },
        );
        access.set_permitted_nodegroups(perms);
        // Conditional permits the nodegroup itself (tile filtering happens separately)
        assert!(access.is_nodegroup_permitted("ng1", false));
    }

    #[test]
    fn get_permitted_nodegroups_bool_returns_all_when_empty() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let perms = access.get_permitted_nodegroups_bool();
        // Should have all nodegroups + root empty-string
        assert!(perms.contains_key("ng1"));
        assert!(perms.contains_key("ng2"));
        assert!(perms.contains_key(""));
        assert!(perms.values().all(|&v| v));
    }

    #[test]
    fn rebuild_from_graph_preserves_permissions() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::from_graph(&graph);
        let mut perms = HashMap::new();
        perms.insert("ng1".to_string(), PermissionRule::Boolean(false));
        access.set_permitted_nodegroups(perms);

        // Rebuild from same graph
        access.rebuild_from_graph(&graph);

        // Permissions should be preserved
        assert!(!access.is_nodegroup_permitted("ng1", true));
    }

    #[test]
    fn trait_get_root_node_returns_correct_node() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let root = access.get_root_node().unwrap();
        assert_eq!(root.nodeid, "root-id");
        assert!(root.istopnode);
    }

    #[test]
    fn nodes_by_nodegroup_indexed_correctly() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let ng2_nodes = access.nodes_by_nodegroup.get("ng2").unwrap();
        // ng2 has child_b and grandchild
        assert_eq!(ng2_nodes.len(), 2);
        let node_ids: Vec<&str> = ng2_nodes.iter().map(|n| n.nodeid.as_str()).collect();
        assert!(node_ids.contains(&"child-b-id"));
        assert!(node_ids.contains(&"grandchild-id"));
    }
}
