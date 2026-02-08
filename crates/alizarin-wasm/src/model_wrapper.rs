use crate::graph::WKRM;
use crate::pseudo_value::{PseudoValue, PseudoValueInner};
use crate::pseudos::PseudoNode;
use wasm_bindgen::prelude::*;
// WASM wrapper types for external interfaces
use crate::graph::StaticGraph as WasmStaticGraph;
use crate::graph::StaticNode as WasmStaticNode;
use crate::graph::StaticNodegroup as WasmStaticNodegroup;
use crate::graph::StaticTile as WasmStaticTile;
// Use core types for internal storage
use alizarin_core::{StaticEdge, StaticGraph, StaticNode, StaticNodegroup, StaticTile};
// Permission rules from core
pub use alizarin_core::PermissionRule;
// Graph pruning from core
use alizarin_core::prune_graph as core_prune_graph;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

// Thread-local registry for canonical model instances
// WASM is single-threaded, so thread_local is safe and appropriate
// Key: graph_id (String), Value: Rc<RefCell<ResourceModelWrapperCore>>
// RefCell allows interior mutability for lazy initialization (build_nodes)
// This is the single source of truth for model state
thread_local! {
    pub(crate) static MODEL_REGISTRY: RefCell<HashMap<String, Rc<RefCell<ResourceModelWrapperCore>>>> =
        RefCell::new(HashMap::new());
}

// Core struct without WASM bindings - contains all the logic
// Uses core types internally for platform independence
#[derive(Clone)]
pub struct ResourceModelWrapperCore {
    wkrm: WKRM,
    graph: Arc<StaticGraph>, // Core StaticGraph

    // Caches - these are built lazily (all core types)
    // Wrapped in Arc for cheap sharing with instance wrappers (avoids cloning on every access)
    edges: Option<Arc<HashMap<String, Vec<String>>>>,
    nodes: Option<Arc<HashMap<String, Arc<StaticNode>>>>,
    nodegroups: Option<Arc<HashMap<String, Arc<StaticNodegroup>>>>,
    nodes_by_alias: Option<Arc<HashMap<String, Arc<StaticNode>>>>,
    // Reverse edge index: child_node_id -> list of parent_node_ids
    // Built alongside edges in build_nodes() for O(1) parent lookups
    reverse_edges: Option<Arc<HashMap<String, Vec<String>>>>,
    // Nodes grouped by nodegroup_id for efficient per-nodegroup iteration
    // Key: nodegroup_id, Value: list of nodes in that nodegroup
    nodes_by_nodegroup: Option<Arc<HashMap<String, Vec<Arc<StaticNode>>>>>,

    /// Permission rules per nodegroup - supports both boolean and conditional rules
    pub(crate) permitted_nodegroups: Option<HashMap<String, PermissionRule>>,
    pub(crate) default_allow: bool,
}

impl ResourceModelWrapperCore {
    pub fn new(wkrm: WKRM, graph: Arc<StaticGraph>, default_allow: bool) -> Self {
        ResourceModelWrapperCore {
            wkrm,
            graph,
            edges: None,
            nodes: None,
            nodegroups: None,
            nodes_by_alias: None,
            reverse_edges: None,
            nodes_by_nodegroup: None,
            permitted_nodegroups: None,
            default_allow,
        }
    }

    pub fn get_root_node(&mut self) -> Result<Arc<StaticNode>, String> {
        // Ensure nodes cache is built
        if self.nodes.is_none() {
            self.build_nodes()?;
        }

        // Search through nodes cache to find root node
        if let Some(ref nodes) = self.nodes {
            for node in nodes.values() {
                if node.nodegroup_id.is_none()
                    || node
                        .nodegroup_id
                        .as_ref()
                        .map(|s| s.is_empty())
                        .unwrap_or(true)
                {
                    // Return Arc clone - cheap reference count increment
                    return Ok(Arc::clone(node));
                }
            }
        }

        Err(format!(
            "COULD NOT FIND ROOT NODE FOR {}. Does the graph {} still exist?",
            self.wkrm.get_model_class_name(),
            self.graph.graph_id()
        ))
    }

    pub fn build_nodes(&mut self) -> Result<(), String> {
        let graph = Arc::clone(&self.graph);
        self.build_nodes_for_graph(&graph)
    }

    pub fn build_nodes_for_graph(&mut self, graph: &StaticGraph) -> Result<(), String> {
        if self.nodes.is_some() || self.nodegroups.is_some() {
            return Err("Cache should never try and rebuild nodes when non-empty".to_string());
        }

        let mut edges_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_edges_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut nodes_map: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut nodegroups_map: HashMap<String, Arc<StaticNodegroup>> = HashMap::new();
        let mut nodes_by_nodegroup_map: HashMap<String, Vec<Arc<StaticNode>>> = HashMap::new();

        // Build nodes map
        for node in graph.nodes.iter() {
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
            let node_arc = Arc::new(node_copy);
            nodes_map.insert(node_arc.nodeid.clone(), Arc::clone(&node_arc));

            // Build nodes-by-nodegroup index
            let ng_id = node_arc.nodegroup_id.clone().unwrap_or_default();
            nodes_by_nodegroup_map
                .entry(ng_id)
                .or_default()
                .push(node_arc);
        }

        // Build nodegroups map from nodes with nodegroup_id
        for node in graph.nodes.iter() {
            if let Some(ref nodegroup_id) = node.nodegroup_id {
                if !nodegroup_id.is_empty() && !nodegroups_map.contains_key(nodegroup_id) {
                    // Create a minimal StaticNodegroup
                    nodegroups_map.insert(
                        nodegroup_id.clone(),
                        Arc::new(StaticNodegroup {
                            cardinality: Some("n".to_string()),
                            legacygroupid: None,
                            nodegroupid: nodegroup_id.clone(),
                            parentnodegroup_id: None,
                            grouping_node_id: None,
                        }),
                    );
                }
            }
        }

        // Merge with actual nodegroups from graph
        for nodegroup in graph.nodegroups.iter() {
            nodegroups_map.insert(nodegroup.nodegroupid.clone(), Arc::new(nodegroup.clone()));
        }

        // Build edges map AND reverse edges map in single pass
        for edge in graph.edges.iter() {
            // Forward: parent -> children
            edges_map
                .entry(edge.domainnode_id.clone())
                .or_default()
                .push(edge.rangenode_id.clone());
            // Reverse: child -> parents (for O(1) parent lookups)
            reverse_edges_map
                .entry(edge.rangenode_id.clone())
                .or_default()
                .push(edge.domainnode_id.clone());
        }

        // Build nodes by alias map
        let mut nodes_by_alias_map: HashMap<String, Arc<StaticNode>> = HashMap::new();
        for (_, node) in nodes_map.iter() {
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    nodes_by_alias_map.insert(alias.clone(), Arc::clone(node));
                    continue;
                }
            };
            nodes_by_alias_map.insert(String::new(), Arc::clone(node));
        }

        self.nodes = Some(Arc::new(nodes_map));
        self.nodegroups = Some(Arc::new(nodegroups_map));
        self.edges = Some(Arc::new(edges_map));
        self.reverse_edges = Some(Arc::new(reverse_edges_map));
        self.nodes_by_alias = Some(Arc::new(nodes_by_alias_map));
        self.nodes_by_nodegroup = Some(Arc::new(nodes_by_nodegroup_map));

        Ok(())
    }

    pub fn get_child_nodes(
        &mut self,
        node_id: &str,
    ) -> Result<HashMap<String, Arc<StaticNode>>, String> {
        if self.nodes.is_none() {
            self.build_nodes()?;
        }
        let mut child_nodes: HashMap<String, Arc<StaticNode>> = HashMap::new();

        if let Some(ref edges) = self.edges {
            if let Some(child_ids) = edges.get(node_id) {
                if let Some(ref nodes) = self.nodes {
                    for child_id in child_ids {
                        if let Some(node) = nodes.get(child_id) {
                            if let Some(ref alias) = node.alias {
                                if !alias.is_empty() {
                                    child_nodes.insert(alias.clone(), Arc::clone(node));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(child_nodes)
    }

    /// Get permitted nodegroups as boolean map (for backward compatibility)
    /// Conditional rules are converted to true (nodegroup is permitted, tiles filtered)
    pub fn get_permitted_nodegroups(&mut self) -> HashMap<String, bool> {
        if let Some(ref permitted) = self.permitted_nodegroups {
            // Convert PermissionRule to bool for backward compatibility
            return permitted
                .iter()
                .map(|(k, v)| (k.clone(), v.permits_nodegroup()))
                .collect();
        }

        // Initialize with all nodegroups permitted
        let mut permissions = HashMap::new();

        if let Some(ref nodegroups) = self.nodegroups {
            for key in nodegroups.keys() {
                permissions.insert(key.clone(), PermissionRule::Boolean(true));
            }
        }

        // Root node must be accessible
        permissions.insert(String::new(), PermissionRule::Boolean(true));

        // Store it
        self.permitted_nodegroups = Some(permissions.clone());

        // Return as bool map
        permissions
            .iter()
            .map(|(k, v)| (k.clone(), v.permits_nodegroup()))
            .collect()
    }

    /// Check if a nodegroup is permitted (for graph pruning, etc.)
    /// Conditional rules return true (nodegroup permitted, tiles filtered separately)
    pub fn is_nodegroup_permitted(&self, nodegroup_id: &str) -> bool {
        if let Some(ref permissions) = self.permitted_nodegroups {
            return permissions
                .get(nodegroup_id)
                .map(|rule| rule.permits_nodegroup())
                .unwrap_or(false);
        }
        self.default_allow
    }

    /// Check if a specific tile is permitted by its nodegroup's permission rule
    pub fn is_tile_permitted(&self, tile: &StaticTile) -> bool {
        if let Some(ref permissions) = self.permitted_nodegroups {
            return permissions
                .get(&tile.nodegroup_id)
                .map(|rule| rule.permits_tile(tile))
                .unwrap_or(self.default_allow);
        }
        self.default_allow
    }

    /// Get the permission rule for a nodegroup (for tile filtering)
    pub fn get_permission_rule(&self, nodegroup_id: &str) -> Option<&PermissionRule> {
        self.permitted_nodegroups.as_ref()?.get(nodegroup_id)
    }

    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) {
        self.default_allow = default_allow;
    }

    /// Set permitted nodegroups with full PermissionRule support
    pub fn set_permitted_nodegroups_rules(&mut self, permissions: HashMap<String, PermissionRule>) {
        self.permitted_nodegroups = Some(permissions);
    }

    /// Set permitted nodegroups from boolean map (backward compatibility)
    pub fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, bool>) {
        let rules: HashMap<String, PermissionRule> = permissions
            .into_iter()
            .map(|(k, v)| (k, PermissionRule::Boolean(v)))
            .collect();
        self.permitted_nodegroups = Some(rules);
    }

    // Internal accessors for Rust code
    pub fn get_nodes_by_alias_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes_by_alias.as_ref().map(|arc| arc.as_ref())
    }

    pub fn get_nodes_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes.as_ref().map(|arc| arc.as_ref())
    }

    pub fn get_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.edges.as_ref().map(|arc| arc.as_ref())
    }

    /// Get reverse edges (child_id -> parent_ids) for O(1) parent lookups
    pub fn get_reverse_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.reverse_edges.as_ref().map(|arc| arc.as_ref())
    }

    /// Get nodes grouped by nodegroup_id for efficient per-nodegroup iteration
    pub fn get_nodes_by_nodegroup_internal(
        &self,
    ) -> Option<&HashMap<String, Vec<Arc<StaticNode>>>> {
        self.nodes_by_nodegroup.as_ref().map(|arc| arc.as_ref())
    }

    pub fn get_nodegroups_internal(&self) -> Option<&HashMap<String, Arc<StaticNodegroup>>> {
        self.nodegroups.as_ref().map(|arc| arc.as_ref())
    }

    /// Get Arc-wrapped nodes for cheap sharing (just increments refcount)
    pub fn get_nodes_arc(&self) -> Option<Arc<HashMap<String, Arc<StaticNode>>>> {
        self.nodes.as_ref().map(Arc::clone)
    }

    /// Get Arc-wrapped edges for cheap sharing
    pub fn get_edges_arc(&self) -> Option<Arc<HashMap<String, Vec<String>>>> {
        self.edges.as_ref().map(Arc::clone)
    }

    /// Get Arc-wrapped reverse edges for cheap sharing
    pub fn get_reverse_edges_arc(&self) -> Option<Arc<HashMap<String, Vec<String>>>> {
        self.reverse_edges.as_ref().map(Arc::clone)
    }

    /// Get Arc-wrapped nodes-by-nodegroup for cheap sharing
    pub fn get_nodes_by_nodegroup_arc(&self) -> Option<Arc<HashMap<String, Vec<Arc<StaticNode>>>>> {
        self.nodes_by_nodegroup.as_ref().map(Arc::clone)
    }

    /// Get Arc-wrapped nodegroups for cheap sharing
    pub fn get_nodegroups_arc(&self) -> Option<Arc<HashMap<String, Arc<StaticNodegroup>>>> {
        self.nodegroups.as_ref().map(Arc::clone)
    }

    pub fn get_node_objects(&mut self) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        if self.nodes.is_none() {
            self.build_nodes()?;
        }
        self.nodes
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build nodes".to_string())
    }

    pub fn get_node_objects_by_alias(
        &mut self,
    ) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        if self.nodes_by_alias.is_none() {
            self.build_nodes()?;
        }
        self.nodes_by_alias
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build nodes".to_string())
    }

    pub fn get_edges(&mut self) -> Result<&HashMap<String, Vec<String>>, String> {
        if self.edges.is_none() {
            self.build_nodes()?;
        }
        self.edges
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build edges".to_string())
    }

    pub fn get_nodegroup_objects(
        &mut self,
    ) -> Result<&HashMap<String, Arc<StaticNodegroup>>, String> {
        if self.nodegroups.is_none() {
            self.build_nodes()?;
        }
        self.nodegroups
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| "Could not build nodegroups".to_string())
    }

    pub fn set_graph_nodes(&mut self, nodes: Vec<StaticNode>) {
        let mut graph = (*self.graph).clone();
        graph.nodes = nodes;
        self.graph = Arc::new(graph);
    }

    pub fn set_graph_edges(&mut self, edges: Vec<StaticEdge>) {
        let mut graph = (*self.graph).clone();
        graph.edges = edges;
        self.graph = Arc::new(graph);
    }

    pub fn set_graph_nodegroups(&mut self, nodegroups: Vec<StaticNodegroup>) {
        let mut graph = (*self.graph).clone();
        graph.nodegroups = nodegroups;
        self.graph = Arc::new(graph);
    }

    pub fn get_wkrm(&self) -> &WKRM {
        &self.wkrm
    }

    pub fn get_graph(&self) -> &StaticGraph {
        &self.graph
    }

    pub fn set_graph(&mut self, graph: StaticGraph) {
        self.graph = Arc::new(graph);
    }

    /// Prune graph to only include permitted nodegroups and their dependencies
    /// Filters nodes, edges, cards, nodegroups, and functions based on permissions
    /// Returns error if graph is malformed or has cycles
    ///
    /// Delegates to `alizarin_core::prune_graph` for the actual pruning logic.
    pub fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> Result<(), String> {
        // Create permission check closure that captures self
        let is_permitted = |ng_id: &str| self.is_nodegroup_permitted(ng_id);

        // Delegate to core prune_graph function
        let keep_fns_ref = keep_functions.as_deref();
        let pruned =
            core_prune_graph(&self.graph, is_permitted, keep_fns_ref).map_err(|e| e.to_string())?;

        // Update the graph and clear caches to force rebuild
        self.graph = Arc::new(pruned);
        self.nodes = None;
        self.edges = None;
        self.nodegroups = None;
        self.nodes_by_alias = None;
        self.reverse_edges = None;
        self.nodes_by_nodegroup = None;

        Ok(())
    }
}

// WASM wrapper - lightweight reference that borrows from registry
// Only holds the graph_id and looks up the core from MODEL_REGISTRY
#[wasm_bindgen]
#[derive(Clone)]
pub struct WASMResourceModelWrapper {
    graph_id: String,
}

impl WASMResourceModelWrapper {
    /// Internal helper to get the core from the registry
    /// This is how WASMResourceModelWrapper accesses the canonical instance
    pub(crate) fn with_core<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ResourceModelWrapperCore) -> R,
    {
        MODEL_REGISTRY.with(|registry| {
            let registry_borrow = registry.borrow();
            let core_arc = registry_borrow
                .get(&self.graph_id)
                .unwrap_or_else(|| panic!("Model not found in registry: {}", self.graph_id));
            let core_borrow = core_arc.borrow();
            f(&core_borrow)
        })
    }

    /// Internal helper to get mutable core from the registry
    pub(crate) fn with_core_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ResourceModelWrapperCore) -> R,
    {
        MODEL_REGISTRY.with(|registry| {
            let registry_borrow = registry.borrow();
            let core_arc = registry_borrow
                .get(&self.graph_id)
                .unwrap_or_else(|| panic!("Model not found in registry: {}", self.graph_id));
            let mut core_borrow = core_arc.borrow_mut();
            f(&mut core_borrow)
        })
    }
}

#[wasm_bindgen]
impl WASMResourceModelWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        wkrm: &WKRM,
        graph: &WasmStaticGraph,
        default_allow: bool,
    ) -> WASMResourceModelWrapper {
        // Extract core graph from WASM wrapper
        let core_graph = graph.0.clone();
        let graph_id = core_graph.graph_id().to_string();

        // Register graph in core registry (for batch_merge_resources descriptor computation)
        alizarin_core::register_graph(&graph_id, Arc::new(core_graph.clone()));

        // Create and register the core in the WASM-specific registry
        let core = ResourceModelWrapperCore::new(wkrm.clone(), Arc::new(core_graph), default_allow);
        MODEL_REGISTRY.with(|registry| {
            registry
                .borrow_mut()
                .insert(graph_id.clone(), Rc::new(RefCell::new(core)));
        });

        // Return a lightweight wrapper that just holds the graph_id
        WASMResourceModelWrapper { graph_id }
    }

    /// Get a registered model by graph_id (returns None if not found or graph_id is null)
    #[wasm_bindgen(js_name = getModelByGraphId)]
    pub fn get_model_by_graph_id(graph_id: Option<String>) -> Option<WASMResourceModelWrapper> {
        let graph_id = graph_id?;
        MODEL_REGISTRY.with(|registry| {
            if registry.borrow().contains_key(&graph_id) {
                Some(WASMResourceModelWrapper {
                    graph_id: graph_id.clone(),
                })
            } else {
                None
            }
        })
    }

    /// Check if a model is registered (returns false if graph_id is null)
    #[wasm_bindgen(js_name = isModelRegistered)]
    pub fn is_model_registered(graph_id: Option<String>) -> bool {
        match graph_id {
            Some(id) => MODEL_REGISTRY.with(|registry| registry.borrow().contains_key(&id)),
            None => false,
        }
    }

    /// Get the graph_id for this model
    #[wasm_bindgen(js_name = getGraphId)]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    /// Create a root PseudoNode (no parent)
    #[wasm_bindgen(js_name = createPseudoNode)]
    pub fn create_pseudo_node(
        &mut self,
        child_node: Option<String>,
    ) -> Result<PseudoNode, JsValue> {
        self._create_pseudo_node(child_node, None)
    }

    /// Create a child PseudoNode by borrowing the parent (doesn't consume it)
    #[wasm_bindgen(js_name = createPseudoNodeChild)]
    pub fn create_pseudo_node_child(
        &mut self,
        child_node: String,
        parent_pseudo: &PseudoNode,
    ) -> Result<PseudoNode, JsValue> {
        self._create_pseudo_node(Some(child_node), Some(parent_pseudo))
    }

    fn _create_pseudo_node(
        &mut self,
        child_node: Option<String>,
        parent_pseudo: Option<&PseudoNode>,
    ) -> Result<PseudoNode, JsValue> {
        // Ensure nodes cache is built
        self.with_core_mut(|core| {
            if core.nodes.is_none() {
                core.build_nodes().map_err(|e| JsValue::from_str(&e))
            } else {
                Ok(())
            }
        })?;

        // Get the node (Arc<StaticNode>) either by alias or as root
        let arc_node: Arc<StaticNode> = if let Some(parent) = parent_pseudo {
            let child_node_id: Option<String> = child_node.clone();
            Arc::clone(
                parent
                    .child_nodes
                    .get(
                        child_node
                            .ok_or_else(|| {
                                JsValue::from_str("Must have a child node name if passing a parent")
                            })?
                            .as_str(),
                    )
                    .ok_or_else(|| {
                        JsValue::from_str(
                            format!(
                                "This parent node does not have this child: {:?} {:?}",
                                parent.get_nodeid(),
                                child_node_id
                            )
                            .as_str(),
                        )
                    })?,
            )
        } else if let Some(key_str) = child_node {
            // Get by alias - clone the Arc (cheap)
            self.with_core(|core| {
                let nodes_by_alias = core
                    .get_nodes_by_alias_internal()
                    .ok_or_else(|| JsValue::from_str("Could not access nodes by alias"))?;
                Ok::<Arc<StaticNode>, JsValue>(Arc::clone(
                    nodes_by_alias.get(key_str.as_str()).ok_or_else(|| {
                        JsValue::from_str(&format!("Could not find node with alias: {}", key_str))
                    })?,
                ))
            })?
        } else {
            // Get root node - returns Arc<StaticNode>
            self.with_core_mut(|core| {
                core.get_root_node()
                    .map_err(|e| JsValue::from_str(&format!("Could not find root node: {}", e)))
            })?
        };

        // Dereference Arc to get StaticNode for JavaScript
        let node = (*arc_node).clone();

        // Check datatype
        let child_nodes = self.with_core_mut(|core| core.get_child_nodes(&node.nodeid))?;

        // Pass StaticNode directly to the internal constructor
        let pseudo_node = PseudoNode::new_from_static_node(arc_node, child_nodes)?;

        Ok(pseudo_node)
    }

    /// Create a PseudoValue - the new unified pseudo type that includes runtime state
    /// This will eventually replace createPseudoNode
    #[wasm_bindgen(js_name = createPseudoValue)]
    pub fn create_pseudo_value(
        &mut self,
        alias: Option<String>,
        tile: JsValue,
        parent: JsValue,
    ) -> Result<PseudoValue, JsValue> {
        // Ensure nodes cache is built
        self.with_core_mut(|core| {
            if core.nodes.is_none() {
                core.build_nodes().map_err(|e| JsValue::from_str(&e))
            } else {
                Ok(())
            }
        })?;

        // Get the node by alias or as root
        let arc_node: Arc<StaticNode> = if let Some(key_str) = alias {
            self.with_core(|core| {
                let nodes_by_alias = core
                    .get_nodes_by_alias_internal()
                    .ok_or_else(|| JsValue::from_str("Could not access nodes by alias"))?;
                Ok::<Arc<StaticNode>, JsValue>(Arc::clone(
                    nodes_by_alias.get(key_str.as_str()).ok_or_else(|| {
                        JsValue::from_str(&format!("Could not find node with alias: {}", key_str))
                    })?,
                ))
            })?
        } else {
            self.with_core_mut(|core| {
                core.get_root_node()
                    .map_err(|e| JsValue::from_str(&format!("Could not find root node: {}", e)))
            })?
        };

        // Get child node IDs
        let child_node_ids: Vec<String> = self.with_core_mut(|core| {
            let child_nodes = core.get_child_nodes(&arc_node.nodeid)?;
            Ok::<Vec<String>, JsValue>(child_nodes.keys().cloned().collect())
        })?;

        // Parse tile if provided
        let tile_arc: Option<Arc<StaticTile>> = if tile.is_null() || tile.is_undefined() {
            None
        } else {
            serde_wasm_bindgen::from_value::<StaticTile>(tile)
                .ok()
                .map(Arc::new)
        };

        // Extract tile data for this node
        let tile_data = tile_arc
            .as_ref()
            .and_then(|t| t.data.get(&arc_node.nodeid))
            .cloned();

        // Parse parent if provided
        let parent_opt = if parent.is_null() || parent.is_undefined() {
            None
        } else {
            Some(parent)
        };

        // Create the PseudoValueInner with proper inner/outer handling
        let rust_value = PseudoValueInner::new(
            arc_node,
            tile_arc,
            tile_data,
            parent_opt,
            child_node_ids,
            false, // not inner
        );

        Ok(PseudoValue::from_rust(rust_value))
    }

    #[wasm_bindgen(js_name = getRootNode)]
    pub fn get_root_node(&mut self) -> Result<WasmStaticNode, JsValue> {
        self.with_core_mut(|core| {
            let arc_node = core.get_root_node().map_err(|e| JsValue::from_str(&e))?;
            // Dereference Arc to clone the StaticNode, wrap for JavaScript
            Ok(WasmStaticNode((*arc_node).clone()))
        })
    }

    #[wasm_bindgen(js_name = getNodeObjects)]
    pub fn get_node_objects(&mut self) -> Result<JsValue, JsValue> {
        self.with_core_mut(|core| {
            let nodes = core.get_node_objects().map_err(|e| JsValue::from_str(&e))?;
            // Convert HashMap to js_sys::Map, wrapping core nodes in WASM wrappers
            let map = js_sys::Map::new();
            for (key, node) in nodes.iter() {
                map.set(
                    &JsValue::from_str(key),
                    &JsValue::from(WasmStaticNode((**node).clone())),
                );
            }
            Ok(map.into())
        })
    }

    #[wasm_bindgen(js_name = getNodeObjectsByAlias)]
    pub fn get_node_objects_by_alias(&mut self) -> Result<JsValue, JsValue> {
        self.with_core_mut(|core| {
            let nodes_by_alias = core
                .get_node_objects_by_alias()
                .map_err(|e| JsValue::from_str(&e))?;
            // Convert HashMap to js_sys::Map, wrapping core nodes in WASM wrappers
            let map = js_sys::Map::new();
            for (key, node) in nodes_by_alias.iter() {
                map.set(
                    &JsValue::from_str(key),
                    &JsValue::from(WasmStaticNode((**node).clone())),
                );
            }
            Ok(map.into())
        })
    }

    #[wasm_bindgen(js_name = getNodeObjectFromAlias)]
    pub fn get_node_object_from_alias(
        &mut self,
        alias: Option<String>,
    ) -> Result<WasmStaticNode, JsValue> {
        let alias = alias.ok_or_else(|| JsValue::from_str("alias is null or undefined"))?;
        self.with_core_mut(|core| {
            let nodes_by_alias = core
                .get_node_objects_by_alias()
                .map_err(|e| JsValue::from_str(&e))?;
            let node = nodes_by_alias
                .get(&alias)
                .ok_or_else(|| JsValue::from_str(&format!("Node not found in model: {}", alias)))?;
            Ok(WasmStaticNode((**node).clone()))
        })
    }

    #[wasm_bindgen(js_name = getNodeObjectFromId)]
    pub fn get_node_object_from_id(
        &mut self,
        id: Option<String>,
    ) -> Result<WasmStaticNode, JsValue> {
        let id = id.ok_or_else(|| JsValue::from_str("id is null or undefined"))?;
        self.with_core_mut(|core| {
            let nodes = core.get_node_objects().map_err(|e| JsValue::from_str(&e))?;
            let node = nodes
                .get(&id)
                .ok_or_else(|| JsValue::from_str(&format!("Node not found in model: {}", id)))?;
            Ok(WasmStaticNode((**node).clone()))
        })
    }

    #[wasm_bindgen(js_name = getChildNodes)]
    pub fn get_child_nodes(&self, node_id: Option<String>) -> Result<JsValue, JsValue> {
        let node_id = node_id.ok_or_else(|| JsValue::from_str("node_id is null or undefined"))?;
        self.with_core_mut(|core| {
            let child_nodes = core
                .get_child_nodes(&node_id)
                .map_err(|e| JsValue::from_str(&e))?;
            let map = js_sys::Map::new();
            for (alias, node) in child_nodes.iter() {
                map.set(
                    &JsValue::from_str(alias),
                    &JsValue::from(WasmStaticNode((**node).clone())),
                );
            }
            Ok(map.into())
        })
    }

    #[wasm_bindgen(js_name = getEdges)]
    pub fn get_edges(&mut self) -> Result<JsValue, JsValue> {
        self.with_core_mut(|core| {
            let edges = core.get_edges().map_err(|e| JsValue::from_str(&e))?;
            // Convert HashMap<String, Vec<String>> to js_sys::Map
            let map = js_sys::Map::new();
            for (key, values) in edges.iter() {
                let array = js_sys::Array::new();
                for value in values {
                    array.push(&JsValue::from_str(value));
                }
                map.set(&JsValue::from_str(key), &array.into());
            }
            Ok(map.into())
        })
    }

    #[wasm_bindgen(js_name = getNodegroupObjects)]
    pub fn get_nodegroup_objects(&mut self) -> Result<JsValue, JsValue> {
        self.with_core_mut(|core| {
            let nodegroups = core
                .get_nodegroup_objects()
                .map_err(|e| JsValue::from_str(&e))?;
            // Convert HashMap to js_sys::Map, wrapping in WASM types
            let map = js_sys::Map::new();
            for (key, nodegroup) in nodegroups.iter() {
                map.set(
                    &JsValue::from_str(key),
                    &JsValue::from(WasmStaticNodegroup((**nodegroup).clone())),
                );
            }
            Ok(map.into())
        })
    }

    /// Get child node aliases for a given node ID
    /// Returns array of alias strings instead of full node objects
    #[wasm_bindgen(js_name = getChildNodeAliases)]
    pub fn get_child_node_aliases(&self, node_id: Option<String>) -> Result<Vec<String>, JsValue> {
        let node_id = node_id.ok_or_else(|| JsValue::from_str("node_id is null or undefined"))?;
        self.with_core_mut(|core| {
            let child_nodes = core
                .get_child_nodes(&node_id)
                .map_err(|e| JsValue::from_str(&e))?;
            Ok(child_nodes.keys().cloned().collect())
        })
    }

    /// Get nodegroup IDs
    /// Returns array of IDs instead of full nodegroup objects
    #[wasm_bindgen(js_name = getNodegroupIds)]
    pub fn get_nodegroup_ids(&mut self) -> Result<Vec<String>, JsValue> {
        self.with_core_mut(|core| {
            let nodegroups = core
                .get_nodegroup_objects()
                .map_err(|e| JsValue::from_str(&e))?;
            Ok(nodegroups.keys().cloned().collect())
        })
    }

    /// Get nodegroup name by ID
    /// Returns just the name string instead of full node object
    /// Note: nodegroup_id is actually a node ID (the root node of the nodegroup)
    #[wasm_bindgen(js_name = getNodegroupName)]
    pub fn get_nodegroup_name(&mut self, nodegroup_id: Option<String>) -> Result<String, JsValue> {
        let nodegroup_id =
            nodegroup_id.ok_or_else(|| JsValue::from_str("nodegroup_id is null or undefined"))?;
        self.with_core_mut(|core| {
            let nodes = core.get_node_objects().map_err(|e| JsValue::from_str(&e))?;
            let node = nodes
                .get(&nodegroup_id)
                .ok_or_else(|| JsValue::from_str(&format!("Node not found: {}", nodegroup_id)))?;
            Ok(node.name.clone())
        })
    }

    /// Get node ID from alias
    /// Returns just the node ID string instead of full node map
    #[wasm_bindgen(js_name = getNodeIdFromAlias)]
    pub fn get_node_id_from_alias(&mut self, alias: Option<String>) -> Result<String, JsValue> {
        let alias = alias.ok_or_else(|| JsValue::from_str("alias is null or undefined"))?;
        self.with_core_mut(|core| {
            let nodes_by_alias = core
                .get_node_objects_by_alias()
                .map_err(|e| JsValue::from_str(&e))?;
            let node = nodes_by_alias.get(&alias).ok_or_else(|| {
                JsValue::from_str(&format!("Node not found for alias: {}", alias))
            })?;
            Ok(node.nodeid.clone())
        })
    }

    #[wasm_bindgen(js_name = getPermittedNodegroups)]
    pub fn get_permitted_nodegroups(&mut self) -> Result<JsValue, JsValue> {
        self.with_core_mut(|core| {
            let permissions = core.get_permitted_nodegroups();
            // Convert to js_sys::Map
            let map = js_sys::Map::new();
            for (key, value) in permissions.iter() {
                map.set(&JsValue::from_str(key), &JsValue::from_bool(*value));
            }
            Ok(map.into())
        })
    }

    #[wasm_bindgen(js_name = isNodegroupPermitted)]
    pub fn is_nodegroup_permitted(
        &self,
        nodegroup_id: &str,
        _tile: Option<WasmStaticTile>,
    ) -> Result<bool, JsValue> {
        Ok(self.with_core(|core| core.is_nodegroup_permitted(nodegroup_id)))
    }

    #[wasm_bindgen(js_name = setDefaultAllowAllNodegroups)]
    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) {
        self.with_core_mut(|core| core.set_default_allow_all_nodegroups(default_allow));
    }

    /// Set permitted nodegroups with support for both boolean and conditional rules.
    ///
    /// Accepts a Map where values can be:
    /// - boolean: true/false for simple allow/deny
    /// - object: { path: ".data.uuid.field", allowed: ["value1", "value2"] }
    ///   for conditional filtering based on tile data
    ///
    /// Returns an error if any permission rule is invalid.
    #[wasm_bindgen(js_name = setPermittedNodegroups)]
    pub fn set_permitted_nodegroups(&mut self, permissions: js_sys::Map) -> Result<(), JsValue> {
        let mut perms: HashMap<String, PermissionRule> = HashMap::new();
        let mut errors: Vec<String> = Vec::new();

        // Use for_each callback to avoid iterator retention
        permissions.for_each(&mut |value, key| {
            let key_str = match key.as_string() {
                Some(k) => k,
                None => {
                    errors.push(format!("Invalid key (not a string): {:?}", key));
                    return;
                }
            };

            // Check if it's a boolean (simple permission)
            if let Some(bool_val) = value.as_bool() {
                perms.insert(key_str, PermissionRule::Boolean(bool_val));
            }
            // Check if it's an object (conditional permission)
            else if value.is_object() && !value.is_null() {
                match Self::parse_conditional_rule(&value) {
                    Ok(rule) => {
                        perms.insert(key_str, rule);
                    }
                    Err(e) => {
                        errors.push(format!("Invalid conditional rule for '{}': {}", key_str, e));
                    }
                }
            } else {
                errors.push(format!(
                    "Invalid permission value for '{}': expected boolean or {{path, allowed}} object",
                    key_str
                ));
            }
        });

        if !errors.is_empty() {
            return Err(JsValue::from_str(&format!(
                "Permission validation errors:\n  - {}",
                errors.join("\n  - ")
            )));
        }

        self.with_core_mut(|core| core.set_permitted_nodegroups_rules(perms));

        Ok(())
    }

    /// Parse a conditional permission rule from a JS object
    /// Expected format: { path: ".data.uuid.field", allowed: ["value1", "value2"] }
    fn parse_conditional_rule(value: &JsValue) -> Result<PermissionRule, String> {
        // Get the "path" property
        let path = js_sys::Reflect::get(value, &JsValue::from_str("path"))
            .map_err(|_| "failed to read 'path' property")?;
        let path_str = path.as_string().ok_or("'path' must be a string")?;

        if path_str.is_empty() {
            return Err("'path' cannot be empty".to_string());
        }

        // Get the "allowed" property (should be an array)
        let allowed = js_sys::Reflect::get(value, &JsValue::from_str("allowed"))
            .map_err(|_| "failed to read 'allowed' property")?;

        if !js_sys::Array::is_array(&allowed) {
            return Err("'allowed' must be an array".to_string());
        }

        let allowed_array = js_sys::Array::from(&allowed);

        if allowed_array.length() == 0 {
            return Err("'allowed' array cannot be empty".to_string());
        }

        // Convert to HashSet<String>
        let mut allowed_set = HashSet::new();
        for i in 0..allowed_array.length() {
            let item = allowed_array.get(i);
            match item.as_string() {
                Some(s) => {
                    allowed_set.insert(s);
                }
                None => {
                    return Err(format!("'allowed[{}]' must be a string", i));
                }
            }
        }

        Ok(PermissionRule::Conditional {
            path: path_str,
            allowed: allowed_set,
        })
    }

    pub fn build_nodes(&mut self) -> Result<(), JsValue> {
        self.with_core_mut(|core| core.build_nodes().map_err(|e| JsValue::from_str(&e)))
    }

    #[wasm_bindgen(js_name = buildNodesForGraph)]
    pub fn build_nodes_for_graph(&mut self, graph: &WasmStaticGraph) -> Result<(), JsValue> {
        let graph = graph.0.clone();
        self.with_core_mut(|core| {
            core.build_nodes_for_graph(&graph)
                .map_err(|e| JsValue::from_str(&e))
        })
    }

    // Graph modification methods - accept WASM wrappers, extract core types
    #[wasm_bindgen(js_name = setGraphNodes)]
    pub fn set_graph_nodes(&mut self, nodes: JsValue) {
        if let Ok(wasm_nodes) =
            serde_wasm_bindgen::from_value::<Vec<alizarin_core::StaticNode>>(nodes)
        {
            self.with_core_mut(|core| core.set_graph_nodes(wasm_nodes));
        }
    }

    #[wasm_bindgen(js_name = setGraphEdges)]
    pub fn set_graph_edges(&mut self, edges: JsValue) {
        if let Ok(wasm_edges) =
            serde_wasm_bindgen::from_value::<Vec<alizarin_core::StaticEdge>>(edges)
        {
            self.with_core_mut(|core| core.set_graph_edges(wasm_edges));
        }
    }

    #[wasm_bindgen(js_name = setGraphNodegroups)]
    pub fn set_graph_nodegroups(&mut self, nodegroups: JsValue) {
        if let Ok(wasm_nodegroups) =
            serde_wasm_bindgen::from_value::<Vec<alizarin_core::StaticNodegroup>>(nodegroups)
        {
            self.with_core_mut(|core| core.set_graph_nodegroups(wasm_nodegroups));
        }
    }

    // Getters
    #[wasm_bindgen(getter = wkrm)]
    pub fn get_wkrm(&self) -> WKRM {
        self.with_core(|core| core.get_wkrm().clone())
    }

    #[wasm_bindgen(getter = graph)]
    pub fn get_graph(&self) -> WasmStaticGraph {
        self.with_core(|core| WasmStaticGraph(core.get_graph().clone()))
    }

    #[wasm_bindgen(setter = graph)]
    pub fn set_graph(&mut self, graph: WasmStaticGraph) {
        self.with_core_mut(|core| core.set_graph(graph.0));
    }

    #[wasm_bindgen(getter = edges)]
    pub fn get_edges_prop(&mut self) -> JsValue {
        self.get_edges().unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(getter = nodes)]
    pub fn get_nodes_prop(&mut self) -> JsValue {
        self.get_node_objects().unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(getter = nodegroups)]
    pub fn get_nodegroups_prop(&mut self) -> JsValue {
        self.get_nodegroup_objects().unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(getter = nodesByAlias)]
    pub fn get_nodes_by_alias_prop(&mut self) -> JsValue {
        self.get_node_objects_by_alias()
            .unwrap_or(JsValue::UNDEFINED)
    }

    /// Prune graph to only include permitted nodegroups and their dependencies
    /// Filters nodes, edges, cards, nodegroups, and functions based on permissions
    #[wasm_bindgen(js_name = pruneGraph)]
    pub fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> Result<(), JsValue> {
        self.with_core_mut(|core| {
            core.prune_graph(keep_functions)
                .map_err(|e| JsValue::from_str(&e))
        })
    }
}
