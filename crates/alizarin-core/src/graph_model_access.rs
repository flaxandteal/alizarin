/// Platform-agnostic ModelAccess implementation built from a StaticGraph.
///
/// Supports both eager and lazy initialization, with Arc-wrapped caches
/// for cheap sharing across wrapper instances (e.g. WASM's MODEL_REGISTRY).
///
/// Used by WASM, NAPI, and Python bindings as the single source of truth
/// for graph-index-building logic.
use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::prune_graph as core_prune_graph;
use crate::graph::{StaticEdge, StaticGraph, StaticNode, StaticNodegroup, StaticTile};
use crate::instance_wrapper_core::ModelAccess;
use crate::permissions::PermissionRule;

/// ModelAccess implementation built from a StaticGraph.
///
/// Indices are lazily built on first access via [`ensure_built`] and
/// stored as `Arc`-wrapped maps for cheap cloning.
///
/// # Construction
///
/// - [`GraphModelAccess::new`] — lazy, caches built on first `ensure_built()` call
/// - [`GraphModelAccess::new_eager`] — builds caches immediately
/// - [`GraphModelAccess::from_graph`] — backward-compat eager constructor
#[derive(Clone)]
pub struct GraphModelAccess {
    graph: Arc<StaticGraph>,

    // Lazy caches, Arc-wrapped for cheap sharing
    nodes: Option<Arc<HashMap<String, Arc<StaticNode>>>>,
    nodes_by_alias: Option<Arc<HashMap<String, Arc<StaticNode>>>>,
    edges: Option<Arc<HashMap<String, Vec<String>>>>,
    reverse_edges: Option<Arc<HashMap<String, Vec<String>>>>,
    nodes_by_nodegroup: Option<Arc<HashMap<String, Vec<Arc<StaticNode>>>>>,
    nodegroups: Option<Arc<HashMap<String, Arc<StaticNodegroup>>>>,
    root_node_id: Option<String>,

    permitted_nodegroups: HashMap<String, PermissionRule>,
    default_allow: bool,
}

impl GraphModelAccess {
    /// Create a new lazy `GraphModelAccess`. Caches are not built until
    /// [`ensure_built`] is called (or a mutable accessor triggers it).
    pub fn new(graph: Arc<StaticGraph>, default_allow: bool) -> Self {
        GraphModelAccess {
            graph,
            nodes: None,
            nodes_by_alias: None,
            edges: None,
            reverse_edges: None,
            nodes_by_nodegroup: None,
            nodegroups: None,
            root_node_id: None,
            permitted_nodegroups: HashMap::new(),
            default_allow,
        }
    }

    /// Create a `GraphModelAccess` with caches built immediately.
    pub fn new_eager(graph: Arc<StaticGraph>, default_allow: bool) -> Self {
        let mut access = Self::new(graph, default_allow);
        // build_indices cannot fail for valid graphs, but we unwrap to
        // surface malformed graphs early
        access
            .build_indices()
            .expect("Failed to build graph indices");
        access
    }

    /// Backward-compatible eager constructor from a borrowed `StaticGraph`.
    pub fn from_graph(graph: &StaticGraph) -> Self {
        Self::new_eager(Arc::new(graph.clone()), true)
    }

    // =========================================================================
    // Lazy initialization
    // =========================================================================

    /// Ensure all caches are built. No-op if already built.
    pub fn ensure_built(&mut self) -> Result<(), String> {
        if self.nodes.is_none() {
            self.build_indices()?;
        }
        Ok(())
    }

    /// Returns true if caches have been built.
    pub fn is_built(&self) -> bool {
        self.nodes.is_some()
    }

    /// Clear all cached indices. The next call to [`ensure_built`] will rebuild.
    pub fn invalidate_caches(&mut self) {
        self.nodes = None;
        self.nodes_by_alias = None;
        self.edges = None;
        self.reverse_edges = None;
        self.nodes_by_nodegroup = None;
        self.nodegroups = None;
        self.root_node_id = None;
    }

    /// Build all indices from the current graph.
    fn build_indices(&mut self) -> Result<(), String> {
        let graph = &self.graph;

        let mut nodes: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut nodes_by_alias: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut edges_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_edges_map: HashMap<String, Vec<String>> = HashMap::new();
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

            // Root detection: prefer istopnode
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

        // Fallback root detection for graphs without istopnode
        if root_node_id.is_empty() {
            for node in nodes.values() {
                if node.nodegroup_id.is_none()
                    || node
                        .nodegroup_id
                        .as_ref()
                        .map(|s| s.is_empty())
                        .unwrap_or(true)
                {
                    root_node_id = node.nodeid.clone();
                    break;
                }
            }
        }

        // Build edge indices
        for edge in &graph.edges {
            let parent_id = edge.domainnode_id.clone();
            let child_id = edge.rangenode_id.clone();

            edges_map
                .entry(parent_id.clone())
                .or_default()
                .push(child_id.clone());

            reverse_edges_map
                .entry(child_id)
                .or_default()
                .push(parent_id);
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

        self.nodes = Some(Arc::new(nodes));
        self.nodes_by_alias = Some(Arc::new(nodes_by_alias));
        self.edges = Some(Arc::new(edges_map));
        self.reverse_edges = Some(Arc::new(reverse_edges_map));
        self.nodes_by_nodegroup = Some(Arc::new(nodes_by_nodegroup));
        self.nodegroups = Some(Arc::new(nodegroups));
        self.root_node_id = Some(root_node_id);

        // Pre-populate default permissions if none were explicitly set,
        // so get_permitted_nodegroups() can return a cheap clone instead
        // of rebuilding a HashMap on every call.
        if self.permitted_nodegroups.is_empty() {
            for key in self.nodegroups.as_ref().unwrap().keys() {
                self.permitted_nodegroups
                    .insert(key.clone(), PermissionRule::Boolean(self.default_allow));
            }
            self.permitted_nodegroups
                .insert(String::new(), PermissionRule::Boolean(true));
        }

        Ok(())
    }

    // =========================================================================
    // Internal reference accessors (deref through Arc)
    // =========================================================================

    /// Get nodes by ID (returns None if caches not built).
    pub fn get_nodes_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes.as_ref().map(|arc| arc.as_ref())
    }

    /// Get nodes by alias (returns None if caches not built).
    pub fn get_nodes_by_alias_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes_by_alias.as_ref().map(|arc| arc.as_ref())
    }

    /// Get edges (returns None if caches not built).
    pub fn get_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.edges.as_ref().map(|arc| arc.as_ref())
    }

    /// Get reverse edges (returns None if caches not built).
    pub fn get_reverse_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.reverse_edges.as_ref().map(|arc| arc.as_ref())
    }

    /// Get nodes grouped by nodegroup (returns None if caches not built).
    pub fn get_nodes_by_nodegroup_internal(
        &self,
    ) -> Option<&HashMap<String, Vec<Arc<StaticNode>>>> {
        self.nodes_by_nodegroup.as_ref().map(|arc| arc.as_ref())
    }

    /// Get nodegroups by ID (returns None if caches not built).
    pub fn get_nodegroups_internal(&self) -> Option<&HashMap<String, Arc<StaticNodegroup>>> {
        self.nodegroups.as_ref().map(|arc| arc.as_ref())
    }

    // =========================================================================
    // Arc accessors (for cheap sharing via Arc clone)
    // =========================================================================

    pub fn get_nodes_arc(&self) -> Option<Arc<HashMap<String, Arc<StaticNode>>>> {
        self.nodes.as_ref().map(Arc::clone)
    }

    pub fn get_nodes_by_alias_arc(&self) -> Option<Arc<HashMap<String, Arc<StaticNode>>>> {
        self.nodes_by_alias.as_ref().map(Arc::clone)
    }

    pub fn get_edges_arc(&self) -> Option<Arc<HashMap<String, Vec<String>>>> {
        self.edges.as_ref().map(Arc::clone)
    }

    pub fn get_reverse_edges_arc(&self) -> Option<Arc<HashMap<String, Vec<String>>>> {
        self.reverse_edges.as_ref().map(Arc::clone)
    }

    pub fn get_nodes_by_nodegroup_arc(&self) -> Option<Arc<HashMap<String, Vec<Arc<StaticNode>>>>> {
        self.nodes_by_nodegroup.as_ref().map(Arc::clone)
    }

    pub fn get_nodegroups_arc(&self) -> Option<Arc<HashMap<String, Arc<StaticNodegroup>>>> {
        self.nodegroups.as_ref().map(Arc::clone)
    }

    // =========================================================================
    // Mutable accessors (trigger lazy build)
    // =========================================================================

    /// Get nodes, building caches if needed.
    pub fn get_node_objects(&mut self) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        self.ensure_built()?;
        self.nodes
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build nodes".to_string())
    }

    /// Get nodes by alias, building caches if needed.
    pub fn get_node_objects_by_alias(
        &mut self,
    ) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        self.ensure_built()?;
        self.nodes_by_alias
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build nodes".to_string())
    }

    /// Get edges, building caches if needed.
    pub fn get_edges(&mut self) -> Result<&HashMap<String, Vec<String>>, String> {
        self.ensure_built()?;
        self.edges
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build edges".to_string())
    }

    /// Get nodegroups, building caches if needed.
    pub fn get_nodegroup_objects(
        &mut self,
    ) -> Result<&HashMap<String, Arc<StaticNodegroup>>, String> {
        self.ensure_built()?;
        self.nodegroups
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build nodegroups".to_string())
    }

    /// Get root node, building caches if needed.
    pub fn get_root_node_mut(&mut self) -> Result<Arc<StaticNode>, String> {
        self.ensure_built()?;
        let root_id = self
            .root_node_id
            .as_ref()
            .ok_or_else(|| "Root node ID not set".to_string())?;
        self.nodes
            .as_ref()
            .and_then(|n| n.get(root_id))
            .cloned()
            .ok_or_else(|| "Root node not found in nodes cache".to_string())
    }

    /// Get child nodes for a given node, building caches if needed.
    pub fn get_child_nodes_mut(
        &mut self,
        node_id: &str,
    ) -> Result<HashMap<String, Arc<StaticNode>>, String> {
        self.ensure_built()?;
        // Delegates to the ModelAccess::get_child_nodes default impl
        ModelAccess::get_child_nodes(self, node_id)
    }

    // =========================================================================
    // Non-mutable child node lookup (requires caches already built)
    // =========================================================================

    /// Get child nodes from already-built caches. Returns empty map if caches not built.
    pub fn get_child_nodes(&self, node_id: &str) -> HashMap<String, Arc<StaticNode>> {
        ModelAccess::get_child_nodes(self, node_id).unwrap_or_default()
    }

    // =========================================================================
    // Nodes by alias (non-mutable, backward compat)
    // =========================================================================

    /// Get nodes indexed by alias (requires caches already built).
    pub fn get_nodes_by_alias(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes_by_alias.as_ref().map(|arc| arc.as_ref())
    }

    // =========================================================================
    // Graph access
    // =========================================================================

    /// Get a reference to the underlying graph.
    pub fn get_graph(&self) -> &StaticGraph {
        &self.graph
    }

    /// Get an Arc clone of the underlying graph.
    pub fn get_graph_arc(&self) -> Arc<StaticGraph> {
        Arc::clone(&self.graph)
    }

    /// Get the default_allow setting.
    pub fn get_default_allow(&self) -> bool {
        self.default_allow
    }

    // =========================================================================
    // Graph mutation (with cache invalidation)
    // =========================================================================

    /// Replace the graph and invalidate all caches.
    pub fn set_graph(&mut self, graph: Arc<StaticGraph>) {
        self.graph = graph;
        self.invalidate_caches();
    }

    /// Replace graph nodes and invalidate caches.
    pub fn set_graph_nodes(&mut self, nodes: Vec<StaticNode>) {
        let mut graph = (*self.graph).clone();
        graph.nodes = nodes;
        self.graph = Arc::new(graph);
        self.invalidate_caches();
    }

    /// Replace graph edges and invalidate caches.
    pub fn set_graph_edges(&mut self, edges: Vec<StaticEdge>) {
        let mut graph = (*self.graph).clone();
        graph.edges = edges;
        self.graph = Arc::new(graph);
        self.invalidate_caches();
    }

    /// Replace graph nodegroups and invalidate caches.
    pub fn set_graph_nodegroups(&mut self, nodegroups: Vec<StaticNodegroup>) {
        let mut graph = (*self.graph).clone();
        graph.nodegroups = nodegroups;
        self.graph = Arc::new(graph);
        self.invalidate_caches();
    }

    /// Rebuild indices from a new graph, preserving permissions.
    pub fn rebuild_from_graph(&mut self, graph: &StaticGraph) {
        self.graph = Arc::new(graph.clone());
        self.invalidate_caches();
        // Eagerly rebuild — matches the old behavior where callers expected
        // indices to be available immediately after rebuild.
        let _ = self.ensure_built();
    }

    // =========================================================================
    // Permission management
    // =========================================================================

    /// Set permitted nodegroups with full PermissionRule support.
    pub fn set_permitted_nodegroups_rules(&mut self, permissions: HashMap<String, PermissionRule>) {
        self.permitted_nodegroups = permissions;
    }

    /// Set permitted nodegroups from boolean map (backward compatibility).
    pub fn set_permitted_nodegroups_bool(&mut self, permissions: HashMap<String, bool>) {
        self.permitted_nodegroups = permissions
            .into_iter()
            .map(|(k, v)| (k, PermissionRule::Boolean(v)))
            .collect();
    }

    /// Set the default permission for nodegroups not explicitly listed.
    pub fn set_default_allow(&mut self, default_allow: bool) {
        self.default_allow = default_allow;
    }

    /// Check if a nodegroup is permitted.
    /// Conditional rules return true (nodegroup permitted, tiles filtered separately).
    pub fn is_nodegroup_permitted(&self, nodegroup_id: &str) -> bool {
        self.permitted_nodegroups
            .get(nodegroup_id)
            .map(|rule| rule.permits_nodegroup())
            .unwrap_or(self.default_allow)
    }

    /// Check if a specific tile is permitted by its nodegroup's permission rule.
    pub fn is_tile_permitted(&self, tile: &StaticTile) -> bool {
        self.permitted_nodegroups
            .get(&tile.nodegroup_id)
            .map(|rule| rule.permits_tile(tile))
            .unwrap_or(self.default_allow)
    }

    /// Get the permission rule for a nodegroup.
    pub fn get_permission_rule(&self, nodegroup_id: &str) -> Option<&PermissionRule> {
        self.permitted_nodegroups.get(nodegroup_id)
    }

    /// Get permitted nodegroups as a boolean map (for backward compat).
    pub fn get_permitted_nodegroups_bool(&self) -> HashMap<String, bool> {
        // Defaults are pre-populated in build_indices(), so just convert.
        self.permitted_nodegroups
            .iter()
            .map(|(k, v)| (k.clone(), v.permits_nodegroup()))
            .collect()
    }

    /// Get all permission rules.
    pub fn get_permitted_nodegroups_rules(&self) -> &HashMap<String, PermissionRule> {
        &self.permitted_nodegroups
    }

    // =========================================================================
    // Graph pruning
    // =========================================================================

    /// Prune graph to only include permitted nodegroups and their dependencies.
    /// Updates the graph and rebuilds caches.
    pub fn prune_graph(&mut self, keep_functions: Option<&[String]>) -> Result<(), String> {
        let is_permitted = |ng_id: &str| self.is_nodegroup_permitted(ng_id);
        let pruned = core_prune_graph(&self.graph, is_permitted, keep_functions)
            .map_err(|e| e.to_string())?;
        self.graph = Arc::new(pruned);
        self.invalidate_caches();
        self.ensure_built()?;
        Ok(())
    }
}

impl ModelAccess for GraphModelAccess {
    fn get_nodes(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes.as_ref().map(|arc| arc.as_ref())
    }

    fn get_edges(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.edges.as_ref().map(|arc| arc.as_ref())
    }

    fn get_reverse_edges(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.reverse_edges.as_ref().map(|arc| arc.as_ref())
    }

    fn get_nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<Arc<StaticNode>>>> {
        self.nodes_by_nodegroup.as_ref().map(|arc| arc.as_ref())
    }

    fn get_nodegroups(&self) -> Option<&HashMap<String, Arc<StaticNodegroup>>> {
        self.nodegroups.as_ref().map(|arc| arc.as_ref())
    }

    fn get_root_node(&self) -> Result<Arc<StaticNode>, String> {
        let root_id = self
            .root_node_id
            .as_ref()
            .ok_or_else(|| "Caches not built".to_string())?;
        self.nodes
            .as_ref()
            .and_then(|n| n.get(root_id))
            .cloned()
            .ok_or_else(|| "Root node not found".to_string())
    }

    fn get_permitted_nodegroups(&self) -> HashMap<String, PermissionRule> {
        // Defaults are pre-populated in build_indices(), so this is always a
        // cheap clone rather than rebuilding on every call.
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
        let nodes = access.nodes.as_ref().unwrap();
        assert_eq!(nodes.len(), 4);
        assert!(nodes.contains_key("root-id"));
        assert!(nodes.contains_key("child-a-id"));
        assert!(nodes.contains_key("child-b-id"));
        assert!(nodes.contains_key("grandchild-id"));
    }

    #[test]
    fn from_graph_builds_alias_index() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let aliases = access.nodes_by_alias.as_ref().unwrap();
        assert!(aliases.contains_key("name"));
        assert!(aliases.contains_key("details"));
        assert!(aliases.contains_key("description"));
        assert_eq!(aliases.get("name").unwrap().nodeid, "child-a-id");
    }

    #[test]
    fn from_graph_builds_edges() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let edges = access.edges.as_ref().unwrap();
        let root_children = edges.get("root-id").unwrap();
        assert_eq!(root_children.len(), 2);
        assert!(root_children.contains(&"child-a-id".to_string()));
        assert!(root_children.contains(&"child-b-id".to_string()));
        let b_children = edges.get("child-b-id").unwrap();
        assert_eq!(b_children, &vec!["grandchild-id".to_string()]);
    }

    #[test]
    fn from_graph_builds_reverse_edges() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let rev = access.reverse_edges.as_ref().unwrap();
        let gc_parents = rev.get("grandchild-id").unwrap();
        assert_eq!(gc_parents, &vec!["child-b-id".to_string()]);
        let a_parents = rev.get("child-a-id").unwrap();
        assert_eq!(a_parents, &vec!["root-id".to_string()]);
    }

    #[test]
    fn from_graph_builds_nodegroups() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let ngs = access.nodegroups.as_ref().unwrap();
        assert!(ngs.contains_key("ng1"));
        assert!(ngs.contains_key("ng2"));
        // ng2 should have cardinality from the explicit nodegroup definition
        assert_eq!(ngs.get("ng2").unwrap().cardinality, Some("1".to_string()));
    }

    #[test]
    fn from_graph_identifies_root() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        assert_eq!(access.root_node_id.as_deref(), Some("root-id"));
    }

    #[test]
    fn from_graph_sets_root_alias_when_missing() {
        let mut graph = create_test_graph();
        // Modify root to have no alias and no nodegroup_id (top node pattern)
        graph.nodes[0].alias = None;
        graph.nodes[0].nodegroup_id = None;
        let access = GraphModelAccess::from_graph(&graph);
        let aliases = access.nodes_by_alias.as_ref().unwrap();
        // Should have empty-string alias entry for root
        assert!(aliases.contains_key(""));
        assert_eq!(aliases.get("").unwrap().nodeid, "root-id");
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
        // No permissions set, default_allow=true (from_graph default)
        assert!(access.is_nodegroup_permitted("ng1"));
        // With default_allow=false
        let access2 = GraphModelAccess::new_eager(Arc::new(graph), false);
        assert!(!access2.is_nodegroup_permitted("ng1"));
    }

    #[test]
    fn is_nodegroup_permitted_explicit_deny() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::from_graph(&graph);
        let mut perms = HashMap::new();
        perms.insert("ng1".to_string(), PermissionRule::Boolean(true));
        perms.insert("ng2".to_string(), PermissionRule::Boolean(false));
        access.set_permitted_nodegroups_rules(perms);
        assert!(access.is_nodegroup_permitted("ng1"));
        assert!(!access.is_nodegroup_permitted("ng2"));
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
        access.set_permitted_nodegroups_rules(perms);
        // Conditional permits the nodegroup itself (tile filtering happens separately)
        assert!(access.is_nodegroup_permitted("ng1"));
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
        access.set_permitted_nodegroups_rules(perms);

        // Rebuild from same graph
        access.rebuild_from_graph(&graph);

        // Permissions should be preserved
        assert!(!access.is_nodegroup_permitted("ng1"));
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
        let nbn = access.nodes_by_nodegroup.as_ref().unwrap();
        let ng2_nodes = nbn.get("ng2").unwrap();
        // ng2 has child_b and grandchild
        assert_eq!(ng2_nodes.len(), 2);
        let node_ids: Vec<&str> = ng2_nodes.iter().map(|n| n.nodeid.as_str()).collect();
        assert!(node_ids.contains(&"child-b-id"));
        assert!(node_ids.contains(&"grandchild-id"));
    }

    // --- New tests for lazy mode and cache invalidation ---

    #[test]
    fn lazy_new_does_not_build_caches() {
        let graph = create_test_graph();
        let access = GraphModelAccess::new(Arc::new(graph), true);
        assert!(!access.is_built());
        assert!(access.get_nodes_internal().is_none());
        assert!(access.get_edges_internal().is_none());
    }

    #[test]
    fn ensure_built_populates_caches() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::new(Arc::new(graph), true);
        assert!(!access.is_built());
        access.ensure_built().unwrap();
        assert!(access.is_built());
        assert_eq!(access.get_nodes_internal().unwrap().len(), 4);
    }

    #[test]
    fn ensure_built_is_idempotent() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::new(Arc::new(graph), true);
        access.ensure_built().unwrap();
        let ptr1 = Arc::as_ptr(access.nodes.as_ref().unwrap());
        access.ensure_built().unwrap();
        let ptr2 = Arc::as_ptr(access.nodes.as_ref().unwrap());
        // Same allocation — didn't rebuild
        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn invalidate_caches_clears_indices() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::from_graph(&graph);
        assert!(access.is_built());
        access.invalidate_caches();
        assert!(!access.is_built());
        assert!(access.get_nodes_internal().is_none());
    }

    #[test]
    fn set_graph_nodes_invalidates_caches() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::from_graph(&graph);
        assert_eq!(access.get_nodes_internal().unwrap().len(), 4);

        // Mutate: remove all but root
        let root_only: Vec<StaticNode> = graph
            .nodes
            .iter()
            .filter(|n| n.istopnode)
            .cloned()
            .collect();
        access.set_graph_nodes(root_only);
        // Caches invalidated
        assert!(!access.is_built());
        // Rebuild
        access.ensure_built().unwrap();
        assert_eq!(access.get_nodes_internal().unwrap().len(), 1);
    }

    #[test]
    fn arc_sharing_returns_same_allocation() {
        let graph = create_test_graph();
        let access = GraphModelAccess::from_graph(&graph);
        let arc1 = access.get_nodes_arc().unwrap();
        let arc2 = access.get_nodes_arc().unwrap();
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }

    #[test]
    fn mutable_accessor_triggers_lazy_build() {
        let graph = create_test_graph();
        let mut access = GraphModelAccess::new(Arc::new(graph), true);
        assert!(!access.is_built());
        let nodes = access.get_node_objects().unwrap();
        assert_eq!(nodes.len(), 4);
        assert!(access.is_built());
    }
}
