use wasm_bindgen::prelude::*;
use crate::pseudos::PseudoNode;
use crate::graph::{WKRM, StaticTile, StaticGraph, StaticNode, StaticNodegroup, StaticEdge};
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;

// Thread-local registry for canonical model instances
// WASM is single-threaded, so thread_local is safe and appropriate
// Key: graph_id (String), Value: Arc<RefCell<ResourceModelWrapperCore>>
// RefCell allows interior mutability for lazy initialization (build_nodes)
// This is the single source of truth for model state
thread_local! {
    pub(crate) static MODEL_REGISTRY: RefCell<HashMap<String, Arc<RefCell<ResourceModelWrapperCore>>>> =
        RefCell::new(HashMap::new());
}

// Core struct without WASM bindings - contains all the logic
#[derive(Clone)]
pub struct ResourceModelWrapperCore {
    wkrm: WKRM,
    graph: Arc<StaticGraph>,

    // Caches - these are built lazily
    edges: Option<HashMap<String, Vec<String>>>,
    nodes: Option<HashMap<String, Arc<StaticNode>>>,
    nodegroups: Option<HashMap<String, Arc<StaticNodegroup>>>,
    nodes_by_alias: Option<HashMap<String, Arc<StaticNode>>>,

    permitted_nodegroups: Option<HashMap<String, bool>>,
    default_allow: bool
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
            permitted_nodegroups: None,
            default_allow: default_allow
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
                if node.nodegroup_id.is_none() ||
                   node.nodegroup_id.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    // Return Arc clone - cheap reference count increment
                    return Ok(Arc::clone(node));
                }
            }
        }

        Err(format!(
            "COULD NOT FIND ROOT NODE FOR {}. Does the graph {} still exist?",
            self.wkrm.get_model_class_name(),
            self.graph.get_graphid()
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
        let mut nodes_map: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut nodegroups_map: HashMap<String, Arc<StaticNodegroup>> = HashMap::new();

        // Build nodes map
        for node in graph.nodes.iter() {
            let mut node_copy = node.clone();
            // Ensure root node (node without nodegroup_id) has alias set
            if (node_copy.nodegroup_id.is_none() ||
                node_copy.nodegroup_id.as_ref().map(|s| s.is_empty()).unwrap_or(false)) &&
               node_copy.alias.is_none() {
                node_copy.alias = Some(String::new());
            }
            nodes_map.insert(node_copy.nodeid.clone(), Arc::new(node_copy));
        }

        // Build nodegroups map from nodes with nodegroup_id
        for node in graph.nodes.iter() {
            if let Some(ref nodegroup_id) = node.nodegroup_id {
                if !nodegroup_id.is_empty() && !nodegroups_map.contains_key(nodegroup_id) {
                    // Create a minimal StaticNodegroup
                    nodegroups_map.insert(nodegroup_id.clone(), Arc::new(StaticNodegroup {
                        cardinality: Some("n".to_string()),
                        legacygroupid: None,
                        nodegroupid: nodegroup_id.clone(),
                        parentnodegroup_id: None,
                    }));
                }
            }
        }

        // Merge with actual nodegroups from graph
        for nodegroup in graph.nodegroups.iter() {
            nodegroups_map.insert(nodegroup.nodegroupid.clone(), Arc::new(nodegroup.clone()));
        }

        // Build edges map
        for edge in graph.edges.iter() {
            edges_map
                .entry(edge.domainnode_id.clone())
                .or_insert_with(Vec::new)
                .push(edge.rangenode_id.clone());
        }

        // Build nodes by alias map
        let mut nodes_by_alias_map: HashMap<String, Arc<StaticNode>> = HashMap::new();
        for (_, node) in nodes_map.iter() {
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    nodes_by_alias_map.insert(alias.clone(), Arc::clone(node));
                    continue
                }
            };
            nodes_by_alias_map.insert(String::new(), Arc::clone(node));
        }

        self.nodes = Some(nodes_map);
        self.nodegroups = Some(nodegroups_map);
        self.edges = Some(edges_map);
        self.nodes_by_alias = Some(nodes_by_alias_map);

        Ok(())
    }

    pub fn get_child_nodes(&mut self, node_id: &str) -> Result<HashMap<String, Arc<StaticNode>>, String> {
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

    pub fn get_permitted_nodegroups(&mut self) -> HashMap<String, bool> {
        if let Some(ref permitted) = self.permitted_nodegroups {
            return permitted.clone();
        }

        // Initialize with all nodegroups permitted
        let mut permissions = HashMap::new();

        if let Some(ref nodegroups) = self.nodegroups {
            for key in nodegroups.keys() {
                permissions.insert(key.clone(), true);
            }
        }

        // Root node must be accessible
        permissions.insert(String::new(), true);

        // Store it
        self.permitted_nodegroups = Some(permissions.clone());
        permissions
    }

    pub fn is_nodegroup_permitted(&self, nodegroup_id: &str) -> bool {
        if let Some(ref permissions) = self.permitted_nodegroups {
            return *permissions.get(nodegroup_id).unwrap_or(&false);
        }
        self.default_allow
    }

    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) {
        self.default_allow = default_allow;
    }

    pub fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, bool>) {
        self.permitted_nodegroups = Some(permissions);
    }

    // Internal accessors for Rust code
    pub fn get_nodes_by_alias_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes_by_alias.as_ref()
    }

    pub fn get_nodes_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes.as_ref()
    }

    pub fn get_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.edges.as_ref()
    }

    pub fn get_node_objects(&mut self) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        if self.nodes.is_none() {
            self.build_nodes()?;
        }
        self.nodes.as_ref().ok_or_else(|| "Could not build nodes".to_string())
    }

    pub fn get_node_objects_by_alias(&mut self) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        if self.nodes_by_alias.is_none() {
            self.build_nodes()?;
        }
        self.nodes_by_alias.as_ref().ok_or_else(|| "Could not build nodes".to_string())
    }

    pub fn get_edges(&mut self) -> Result<&HashMap<String, Vec<String>>, String> {
        if self.edges.is_none() {
            self.build_nodes()?;
        }
        self.edges.as_ref().ok_or_else(|| "Could not build edges".to_string())
    }

    pub fn get_nodegroup_objects(&mut self) -> Result<&HashMap<String, Arc<StaticNodegroup>>, String> {
        if self.nodegroups.is_none() {
            self.build_nodes()?;
        }
        self.nodegroups.as_ref().ok_or_else(|| "Could not build nodegroups".to_string())
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
            let core_arc = registry_borrow.get(&self.graph_id)
                .expect(&format!("Model not found in registry: {}", self.graph_id));
            let core_borrow = core_arc.borrow();
            f(&*core_borrow)
        })
    }

    /// Internal helper to get mutable core from the registry
    pub(crate) fn with_core_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ResourceModelWrapperCore) -> R,
    {
        MODEL_REGISTRY.with(|registry| {
            let registry_borrow = registry.borrow();
            let core_arc = registry_borrow.get(&self.graph_id)
                .expect(&format!("Model not found in registry: {}", self.graph_id));
            let mut core_borrow = core_arc.borrow_mut();
            f(&mut *core_borrow)
        })
    }
}

#[wasm_bindgen]
impl WASMResourceModelWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(wkrm: &WKRM, graph: &StaticGraph, default_allow: bool) -> WASMResourceModelWrapper {
        let graph_id = graph.get_graphid();

        // Create and register the core in the registry
        let core = ResourceModelWrapperCore::new(wkrm.clone(), Arc::new(graph.copy()), default_allow);
        MODEL_REGISTRY.with(|registry| {
            registry.borrow_mut().insert(graph_id.clone(), Arc::new(RefCell::new(core)));
        });

        // Return a lightweight wrapper that just holds the graph_id
        WASMResourceModelWrapper {
            graph_id,
        }
    }

    /// Get a registered model by graph_id (returns None if not found)
    #[wasm_bindgen(js_name = getModelByGraphId)]
    pub fn get_model_by_graph_id(graph_id: &str) -> Option<WASMResourceModelWrapper> {
        MODEL_REGISTRY.with(|registry| {
            if registry.borrow().contains_key(graph_id) {
                Some(WASMResourceModelWrapper {
                    graph_id: graph_id.to_string(),
                })
            } else {
                None
            }
        })
    }

    /// Check if a model is registered
    #[wasm_bindgen(js_name = isModelRegistered)]
    pub fn is_model_registered(graph_id: &str) -> bool {
        MODEL_REGISTRY.with(|registry| {
            registry.borrow().contains_key(graph_id)
        })
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
        child_node: Option<String>
    ) -> Result<PseudoNode, JsValue> {
        self._create_pseudo_node(child_node, None)
    }

    /// Create a child PseudoNode by borrowing the parent (doesn't consume it)
    #[wasm_bindgen(js_name = createPseudoNodeChild)]
    pub fn create_pseudo_node_child(
        &mut self,
        child_node: String,
        parent_pseudo: &PseudoNode
    ) -> Result<PseudoNode, JsValue> {
        self._create_pseudo_node(Some(child_node), Some(parent_pseudo))
    }

    fn _create_pseudo_node(
        &mut self,
        child_node: Option<String>,
        parent_pseudo: Option<&PseudoNode>
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
            Arc::clone(parent.child_nodes.get(
                child_node.ok_or_else(|| JsValue::from_str("Must have a child node name if passing a parent"))?.as_str()
            ).ok_or_else(|| JsValue::from_str("This parent node does not have this child"))?)
        } else {
            if let Some(key_str) = child_node {
                // Get by alias - clone the Arc (cheap)
                self.with_core(|core| {
                    let nodes_by_alias = core.get_nodes_by_alias_internal()
                        .ok_or_else(|| JsValue::from_str("Could not access nodes by alias"))?;
                    Ok::<Arc<StaticNode>, JsValue>(Arc::clone(
                        nodes_by_alias.get(key_str.as_str())
                            .ok_or_else(|| JsValue::from_str(&format!("Could not find node with alias: {}", key_str)))?
                    ))
                })?
            } else {
                // Get root node - returns Arc<StaticNode>
                self.with_core_mut(|core| {
                    core.get_root_node()
                        .map_err(|e| JsValue::from_str(&format!("Could not find root node: {}", e)))
                })?
            }
        };

        // Dereference Arc to get StaticNode for JavaScript
        let node = (*arc_node).clone();

        // Check datatype
        let child_nodes = self.with_core_mut(|core| core.get_child_nodes(&node.nodeid))?;

        // Pass StaticNode directly to the internal constructor
        let pseudo_node = PseudoNode::new_from_static_node(arc_node, child_nodes)?;

        Ok(pseudo_node)
    }


    #[wasm_bindgen(js_name = getRootNode)]
    pub fn get_root_node(&mut self) -> Result<StaticNode, JsValue> {
        self.with_core_mut(|core| {
            let arc_node = core.get_root_node().map_err(|e| JsValue::from_str(&e))?;
            // Dereference Arc to clone the StaticNode for JavaScript
            Ok((*arc_node).clone())
        })
    }

    #[wasm_bindgen(js_name = getNodeObjects)]
    pub fn get_node_objects(&mut self) -> Result<JsValue, JsValue> {
        self.with_core_mut(|core| {
            let nodes = core.get_node_objects().map_err(|e| JsValue::from_str(&e))?;
            // Convert HashMap to js_sys::Map
            let map = js_sys::Map::new();
            for (key, node) in nodes.iter() {
                map.set(&JsValue::from_str(key), &JsValue::from((**node).clone()));
            }
            Ok(map.into())
        })
    }

    #[wasm_bindgen(js_name = getNodeObjectsByAlias)]
    pub fn get_node_objects_by_alias(&mut self) -> Result<JsValue, JsValue> {
        self.with_core_mut(|core| {
            let nodes_by_alias = core.get_node_objects_by_alias().map_err(|e| JsValue::from_str(&e))?;
            // Convert HashMap to js_sys::Map
            let map = js_sys::Map::new();
            for (key, node) in nodes_by_alias.iter() {
                map.set(&JsValue::from_str(key), &JsValue::from((**node).clone()));
            }
            Ok(map.into())
        })
    }

    #[wasm_bindgen(js_name = getNodeObjectFromAlias)]
    pub fn get_node_object_from_alias(&mut self, alias: &str) -> Result<StaticNode, JsValue> {
        self.with_core_mut(|core| {
            let nodes_by_alias = core.get_node_objects_by_alias().map_err(|e| JsValue::from_str(&e))?;
            let node = nodes_by_alias.get(alias)
                .ok_or_else(|| JsValue::from_str(&format!("Node not found in model: {}", alias)))?;
            Ok((**node).clone())
        })
    }

    #[wasm_bindgen(js_name = getNodeObjectFromId)]
    pub fn get_node_object_from_id(&mut self, id: &str) -> Result<StaticNode, JsValue> {
        self.with_core_mut(|core| {
            let nodes = core.get_node_objects().map_err(|e| JsValue::from_str(&e))?;
            let node = nodes.get(id)
                .expect(&format!("Node not found in model: {}", id));
            Ok((**node).clone())
        })
    }

    #[wasm_bindgen(js_name = getChildNodes)]
    pub fn get_child_nodes(&self, node_id: &str) -> Result<JsValue, JsValue> {
        let node_id = node_id.to_string();
        self.with_core_mut(|core| {
            let child_nodes = core.get_child_nodes(&node_id).map_err(|e| JsValue::from_str(&e))?;
            let map = js_sys::Map::new();
            for (alias, node) in child_nodes.iter() {
                map.set(&JsValue::from_str(alias), &JsValue::from((**node).clone()));
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
            let nodegroups = core.get_nodegroup_objects().map_err(|e| JsValue::from_str(&e))?;
            // Convert HashMap to js_sys::Map
            let map = js_sys::Map::new();
            for (key, nodegroup) in nodegroups.iter() {
                map.set(&JsValue::from_str(key), &JsValue::from((**nodegroup).clone()));
            }
            Ok(map.into())
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
    pub fn is_nodegroup_permitted(&self, nodegroup_id: &str, _tile: Option<StaticTile>) -> Result<bool, JsValue> {
        Ok(self.with_core(|core| core.is_nodegroup_permitted(nodegroup_id)))
    }

    #[wasm_bindgen(js_name = setDefaultAllowAllNodegroups)]
    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) {
        self.with_core_mut(|core| core.set_default_allow_all_nodegroups(default_allow));
    }

    #[wasm_bindgen(js_name = setPermittedNodegroups)]
    pub fn set_permitted_nodegroups(&mut self, permissions: js_sys::Map) {
        let mut perms = HashMap::new();

        // Iterate over the Map using the keys() iterator
        let keys_iter = permissions.keys();
        loop {
            let next = keys_iter.next().unwrap();
            if next.done() {
                break;
            }
            let key = next.value();
            if let Some(key_str) = key.as_string() {
                let value = permissions.get(&key);
                if let Some(bool_val) = value.as_bool() {
                    perms.insert(key_str, bool_val);
                }
            }
        }

        self.with_core_mut(|core| core.set_permitted_nodegroups(perms));
    }

    pub fn build_nodes(&mut self) -> Result<(), JsValue> {
        self.with_core_mut(|core| core.build_nodes().map_err(|e| JsValue::from_str(&e)))
    }

    #[wasm_bindgen(js_name = buildNodesForGraph)]
    pub fn build_nodes_for_graph(&mut self, graph: &StaticGraph) -> Result<(), JsValue> {
        let graph = graph.clone();
        self.with_core_mut(|core| core.build_nodes_for_graph(&graph).map_err(|e| JsValue::from_str(&e)))
    }

    // Internal accessors for Rust code (not exposed to JavaScript)
    // These can't use with_core because they return references, so we need a different approach
    // We'll make these methods panic if called on an unregistered model (shouldn't happen in practice)
    pub(crate) fn get_nodes_by_alias_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        // Cannot return references from with_core closure - would need different design
        // For now, panic - this should not be called on unregistered models
        panic!("get_nodes_by_alias_internal called - refactor needed to use registry directly")
    }

    pub(crate) fn get_nodes_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        panic!("get_nodes_internal called - refactor needed to use registry directly")
    }

    pub(crate) fn get_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        panic!("get_edges_internal called - refactor needed to use registry directly")
    }

    // Graph modification methods
    #[wasm_bindgen(js_name = setGraphNodes)]
    pub fn set_graph_nodes(&mut self, nodes: Vec<StaticNode>) {
        self.with_core_mut(|core| core.set_graph_nodes(nodes));
    }

    #[wasm_bindgen(js_name = setGraphEdges)]
    pub fn set_graph_edges(&mut self, edges: Vec<StaticEdge>) {
        self.with_core_mut(|core| core.set_graph_edges(edges));
    }

    #[wasm_bindgen(js_name = setGraphNodegroups)]
    pub fn set_graph_nodegroups(&mut self, nodegroups: Vec<StaticNodegroup>) {
        self.with_core_mut(|core| core.set_graph_nodegroups(nodegroups));
    }

    // Getters
    #[wasm_bindgen(getter = wkrm)]
    pub fn get_wkrm(&self) -> WKRM {
        self.with_core(|core| core.get_wkrm().clone())
    }

    #[wasm_bindgen(getter = graph)]
    pub fn get_graph(&self) -> StaticGraph {
        self.with_core(|core| core.get_graph().clone())
    }

    #[wasm_bindgen(setter = graph)]
    pub fn set_graph(&mut self, graph: StaticGraph) {
        self.with_core_mut(|core| core.set_graph(graph));
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
        self.get_node_objects_by_alias().unwrap_or(JsValue::UNDEFINED)
    }
}
