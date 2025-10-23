use wasm_bindgen::prelude::*;
use crate::pseudos::PseudoNode;
use crate::graph::{WKRM, StaticGraph, StaticNode, StaticNodegroup, StaticEdge};
use std::collections::HashMap;
use std::sync::Arc;

// Core struct without WASM bindings - contains all the logic
#[derive(Clone)]
pub struct ResourceModelWrapperCore {
    wkrm: WKRM,
    graph: Arc<StaticGraph>,

    // Caches - these are built lazily
    edges: Option<HashMap<String, Vec<String>>>,
    nodes: Option<HashMap<String, StaticNode>>,
    nodegroups: Option<HashMap<String, StaticNodegroup>>,
    nodes_by_alias: Option<HashMap<String, StaticNode>>,

    permitted_nodegroups: Option<HashMap<String, bool>>,
}

impl ResourceModelWrapperCore {
    pub fn new(wkrm: WKRM, graph: Arc<StaticGraph>) -> Self {
        ResourceModelWrapperCore {
            wkrm,
            graph,
            edges: None,
            nodes: None,
            nodegroups: None,
            nodes_by_alias: None,
            permitted_nodegroups: None,
        }
    }

    pub fn get_root_node(&self) -> Result<StaticNode, String> {
        // Access graph directly, no need for nodes cache to be built
        for node in self.graph.nodes.iter() {
            if node.nodegroup_id.is_none() ||
               node.nodegroup_id.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                let mut root_node = node.clone();
                // Ensure alias is set (to empty string if not present)
                if root_node.alias.is_none() {
                    root_node.alias = Some(String::new());
                }
                return Ok(root_node);
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
        let mut nodes_map: HashMap<String, StaticNode> = HashMap::new();
        let mut nodegroups_map: HashMap<String, StaticNodegroup> = HashMap::new();

        // Build nodes map
        for node in graph.nodes.iter() {
            nodes_map.insert(node.nodeid.clone(), node.clone());
        }

        // Build nodegroups map from nodes with nodegroup_id
        for node in graph.nodes.iter() {
            if let Some(ref nodegroup_id) = node.nodegroup_id {
                if !nodegroup_id.is_empty() && !nodegroups_map.contains_key(nodegroup_id) {
                    // Create a minimal StaticNodegroup
                    nodegroups_map.insert(nodegroup_id.clone(), StaticNodegroup {
                        cardinality: Some("n".to_string()),
                        legacygroupid: None,
                        nodegroupid: nodegroup_id.clone(),
                        parentnodegroup_id: None,
                    });
                }
            }
        }

        // Merge with actual nodegroups from graph
        for nodegroup in graph.nodegroups.iter() {
            nodegroups_map.insert(nodegroup.nodegroupid.clone(), nodegroup.clone());
        }

        // Build edges map
        for edge in graph.edges.iter() {
            edges_map
                .entry(edge.domainnode_id.clone())
                .or_insert_with(Vec::new)
                .push(edge.rangenode_id.clone());
        }

        // Build nodes by alias map
        let mut nodes_by_alias_map: HashMap<String, StaticNode> = HashMap::new();
        for node in nodes_map.values() {
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    nodes_by_alias_map.insert(alias.clone(), node.clone());
                }
            }
        }

        self.nodes = Some(nodes_map);
        self.nodegroups = Some(nodegroups_map);
        self.edges = Some(edges_map);
        self.nodes_by_alias = Some(nodes_by_alias_map);

        Ok(())
    }

    pub fn get_child_nodes(&mut self, node_id: &str) -> Result<HashMap<String, StaticNode>, String> {
        if self.nodes.is_none() {
            self.build_nodes()?;
        }
        let mut child_nodes: HashMap<String, StaticNode> = HashMap::new();

        if let Some(ref edges) = self.edges {
            if let Some(child_ids) = edges.get(node_id) {
                if let Some(ref nodes) = self.nodes {
                    for child_id in child_ids {
                        if let Some(node) = nodes.get(child_id) {
                            if let Some(ref alias) = node.alias {
                                if !alias.is_empty() {
                                    child_nodes.insert(alias.clone(), node.clone());
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
        false
    }

    pub fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, bool>) {
        self.permitted_nodegroups = Some(permissions);
    }

    // Internal accessors for Rust code
    pub fn get_nodes_by_alias_internal(&self) -> Option<&HashMap<String, StaticNode>> {
        self.nodes_by_alias.as_ref()
    }

    pub fn get_nodes_internal(&self) -> Option<&HashMap<String, StaticNode>> {
        self.nodes.as_ref()
    }

    pub fn get_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.edges.as_ref()
    }

    pub fn get_node_objects(&mut self) -> Result<&HashMap<String, StaticNode>, String> {
        if self.nodes.is_none() {
            self.build_nodes()?;
        }
        self.nodes.as_ref().ok_or_else(|| "Could not build nodes".to_string())
    }

    pub fn get_node_objects_by_alias(&mut self) -> Result<&HashMap<String, StaticNode>, String> {
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

    pub fn get_nodegroup_objects(&mut self) -> Result<&HashMap<String, StaticNodegroup>, String> {
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

// WASM wrapper - thin layer that delegates to core
#[wasm_bindgen]
#[derive(Clone)]
pub struct WASMResourceModelWrapper {
    core: ResourceModelWrapperCore,
}

#[wasm_bindgen]
impl WASMResourceModelWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(wkrm: &WKRM, graph: &StaticGraph) -> WASMResourceModelWrapper {
        WASMResourceModelWrapper {
            core: ResourceModelWrapperCore::new(wkrm.clone(), Arc::new(graph.copy()))
        }
    }

    #[wasm_bindgen(js_name = createPseudoNode)]
    pub fn create_pseudo_node(
        &mut self,
        child_node: JsValue,
        _node: JsValue,
    ) -> Result<JsValue, JsValue> {
        // Ensure nodes cache is built
        if self.core.nodes.is_none() {
            self.core.build_nodes().map_err(|e| JsValue::from_str(&e))?;
        }

        // Get the alias from childNode
        let key = js_sys::Reflect::get(&child_node, &JsValue::from_str("alias"))?;
        let key_str = key.as_string().ok_or_else(|| {
            let nodeid = js_sys::Reflect::get(&child_node, &JsValue::from_str("nodeid"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "unknown".to_string());
            JsValue::from_str(&format!("Cannot add a pseudo node with no alias {}", nodeid))
        })?;

        // Get the node by alias using internal Rust HashMap
        let nodes_by_alias = self.get_nodes_by_alias_internal()
            .ok_or_else(|| JsValue::from_str("Could not access nodes by alias"))?;

        let node = nodes_by_alias.get(key_str.as_str())
            .ok_or_else(|| JsValue::from_str(&format!("Could not find node with alias: {}", key_str)))?.clone();

        // Check datatype
        let datatype = &node.datatype;
        let is_semantic = datatype == "semantic";
        let child_nodes = self.get_child_nodes(&node.nodeid)?;

        // Check if child_nodes Map has any entries
        let child_nodes_map = child_nodes.dyn_ref::<js_sys::Map>()
            .ok_or_else(|| JsValue::from_str("child_nodes is not a Map"))?;
        let has_children = child_nodes_map.size() > 0;

        // Determine inner value and final child nodes
        let (inner, final_child_nodes) = if has_children && !is_semantic {
            (JsValue::from(PseudoNode::new_from_static_node(node.copy(), child_nodes, JsValue::TRUE)?), js_sys::Map::new().into())
        } else {
            (JsValue::FALSE, child_nodes)
        };

        // Pass StaticNode directly to the internal constructor
        let pseudo_node = PseudoNode::new_from_static_node(node, final_child_nodes, inner)?;

        Ok(JsValue::from(pseudo_node))
    }


    #[wasm_bindgen(js_name = getRootNode)]
    pub fn get_root_node(&self) -> Result<StaticNode, JsValue> {
        self.core.get_root_node().map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = getNodeObjects)]
    pub fn get_node_objects(&mut self) -> Result<JsValue, JsValue> {
        let nodes = self.core.get_node_objects().map_err(|e| JsValue::from_str(&e))?;
        // Convert HashMap to js_sys::Map
        let map = js_sys::Map::new();
        for (key, node) in nodes.iter() {
            map.set(&JsValue::from_str(key), &JsValue::from(node.clone()));
        }
        Ok(map.into())
    }

    #[wasm_bindgen(js_name = getNodeObjectsByAlias)]
    pub fn get_node_objects_by_alias(&mut self) -> Result<JsValue, JsValue> {
        let nodes_by_alias = self.core.get_node_objects_by_alias().map_err(|e| JsValue::from_str(&e))?;
        // Convert HashMap to js_sys::Map
        let map = js_sys::Map::new();
        for (key, node) in nodes_by_alias.iter() {
            map.set(&JsValue::from_str(key), &JsValue::from(node.clone()));
        }
        Ok(map.into())
    }

    #[wasm_bindgen(js_name = getChildNodes)]
    pub fn get_child_nodes(&mut self, node_id: &str) -> Result<JsValue, JsValue> {
        let child_nodes = self.core.get_child_nodes(node_id).map_err(|e| JsValue::from_str(&e))?;
        let map = js_sys::Map::new();
        for (alias, node) in child_nodes.iter() {
            map.set(&JsValue::from_str(alias), &JsValue::from(node.clone()));
        }
        Ok(map.into())
    }

    #[wasm_bindgen(js_name = getEdges)]
    pub fn get_edges(&mut self) -> Result<JsValue, JsValue> {
        let edges = self.core.get_edges().map_err(|e| JsValue::from_str(&e))?;
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
    }

    #[wasm_bindgen(js_name = getNodegroupObjects)]
    pub fn get_nodegroup_objects(&mut self) -> Result<JsValue, JsValue> {
        let nodegroups = self.core.get_nodegroup_objects().map_err(|e| JsValue::from_str(&e))?;
        // Convert HashMap to js_sys::Map
        let map = js_sys::Map::new();
        for (key, nodegroup) in nodegroups.iter() {
            map.set(&JsValue::from_str(key), &JsValue::from(nodegroup.clone()));
        }
        Ok(map.into())
    }

    #[wasm_bindgen(js_name = getPermittedNodegroups)]
    pub fn get_permitted_nodegroups(&mut self) -> Result<JsValue, JsValue> {
        let permissions = self.core.get_permitted_nodegroups();
        // Convert to js_sys::Map
        let map = js_sys::Map::new();
        for (key, value) in permissions.iter() {
            map.set(&JsValue::from_str(key), &JsValue::from_bool(*value));
        }
        Ok(map.into())
    }

    #[wasm_bindgen(js_name = isNodegroupPermitted)]
    pub fn is_nodegroup_permitted(&self, nodegroup_id: &str, _tile: JsValue) -> Result<bool, JsValue> {
        Ok(self.core.is_nodegroup_permitted(nodegroup_id))
    }

    #[wasm_bindgen(js_name = setPermittedNodegroups)]
    pub fn set_permitted_nodegroups(&mut self, permissions: js_sys::Map) {
        let mut perms = HashMap::new();

        let keys = js_sys::Object::keys(&permissions);
        for i in 0..keys.length() {
            let key = keys.get(i);
            if let Some(key_str) = key.as_string() {
                let value = permissions.get(&key);
                if let Some(bool_val) = value.as_bool() {
                    perms.insert(key_str, bool_val);
                }
            }
        }

        self.core.set_permitted_nodegroups(perms);
    }

    pub fn build_nodes(&mut self) -> Result<(), JsValue> {
        self.core.build_nodes().map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = buildNodesForGraph)]
    pub fn build_nodes_for_graph(&mut self, graph: &StaticGraph) -> Result<(), JsValue> {
        self.core.build_nodes_for_graph(graph).map_err(|e| JsValue::from_str(&e))
    }

    // Internal accessors for Rust code (not exposed to JavaScript)
    pub(crate) fn get_nodes_by_alias_internal(&self) -> Option<&HashMap<String, StaticNode>> {
        self.core.get_nodes_by_alias_internal()
    }

    pub(crate) fn get_nodes_internal(&self) -> Option<&HashMap<String, StaticNode>> {
        self.core.get_nodes_internal()
    }

    pub(crate) fn get_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.core.get_edges_internal()
    }

    // Graph modification methods
    #[wasm_bindgen(js_name = setGraphNodes)]
    pub fn set_graph_nodes(&mut self, nodes: Vec<StaticNode>) {
        self.core.set_graph_nodes(nodes);
    }

    #[wasm_bindgen(js_name = setGraphEdges)]
    pub fn set_graph_edges(&mut self, edges: Vec<StaticEdge>) {
        self.core.set_graph_edges(edges);
    }

    #[wasm_bindgen(js_name = setGraphNodegroups)]
    pub fn set_graph_nodegroups(&mut self, nodegroups: Vec<StaticNodegroup>) {
        self.core.set_graph_nodegroups(nodegroups);
    }

    // Getters
    #[wasm_bindgen(getter = wkrm)]
    pub fn get_wkrm(&self) -> WKRM {
        self.core.get_wkrm().clone()
    }

    #[wasm_bindgen(getter = graph)]
    pub fn get_graph(&self) -> StaticGraph {
        self.core.get_graph().clone()
    }

    #[wasm_bindgen(setter = graph)]
    pub fn set_graph(&mut self, graph: StaticGraph) {
        self.core.set_graph(graph);
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
