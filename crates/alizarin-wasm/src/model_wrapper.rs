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
use alizarin_core::{StaticGraph, StaticNode, StaticNodegroup, StaticTile};
// Permission rules from core
use alizarin_core::GraphModelAccess;
pub use alizarin_core::PermissionRule;
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

// Core struct without WASM bindings - delegates graph indexing to GraphModelAccess
#[derive(Clone)]
pub struct ResourceModelWrapperCore {
    wkrm: WKRM,
    pub(crate) model_access: GraphModelAccess,
}

impl ResourceModelWrapperCore {
    pub fn new(wkrm: WKRM, graph: Arc<StaticGraph>, default_allow: bool) -> Self {
        ResourceModelWrapperCore {
            wkrm,
            model_access: GraphModelAccess::new(graph, default_allow),
        }
    }

    // =========================================================================
    // Forwarding methods — keep the existing API surface so callers
    // (WASMResourceModelWrapper, instance_wrapper) need minimal changes
    // =========================================================================

    pub fn ensure_built(&mut self) -> Result<(), String> {
        self.model_access.ensure_built()
    }

    pub fn build_nodes(&mut self) -> Result<(), String> {
        self.model_access.ensure_built()
    }

    pub fn build_nodes_for_graph(&mut self, graph: &StaticGraph) -> Result<(), String> {
        self.model_access.rebuild_from_graph(graph);
        Ok(())
    }

    pub fn get_root_node(&mut self) -> Result<Arc<StaticNode>, String> {
        self.model_access.get_root_node_mut()
    }

    pub fn get_child_nodes(
        &mut self,
        node_id: &str,
    ) -> Result<HashMap<String, Arc<StaticNode>>, String> {
        self.model_access.get_child_nodes_mut(node_id)
    }

    pub fn get_node_objects(&mut self) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        self.model_access.get_node_objects()
    }

    pub fn get_node_objects_by_alias(
        &mut self,
    ) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        self.model_access.get_node_objects_by_alias()
    }

    pub fn get_edges(&mut self) -> Result<&HashMap<String, Vec<String>>, String> {
        self.model_access.get_edges()
    }

    pub fn get_nodegroup_objects(
        &mut self,
    ) -> Result<&HashMap<String, Arc<StaticNodegroup>>, String> {
        self.model_access.get_nodegroup_objects()
    }

    // Internal accessors (no lazy build)
    pub fn get_nodes_by_alias_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.model_access.get_nodes_by_alias_internal()
    }

    pub fn get_nodes_internal(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.model_access.get_nodes_internal()
    }

    pub fn get_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.model_access.get_edges_internal()
    }

    pub fn get_reverse_edges_internal(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.model_access.get_reverse_edges_internal()
    }

    pub fn get_nodes_by_nodegroup_internal(
        &self,
    ) -> Option<&HashMap<String, Vec<Arc<StaticNode>>>> {
        self.model_access.get_nodes_by_nodegroup_internal()
    }

    pub fn get_nodegroups_internal(&self) -> Option<&HashMap<String, Arc<StaticNodegroup>>> {
        self.model_access.get_nodegroups_internal()
    }

    // Arc accessors
    pub fn get_nodes_arc(&self) -> Option<Arc<HashMap<String, Arc<StaticNode>>>> {
        self.model_access.get_nodes_arc()
    }

    pub fn get_edges_arc(&self) -> Option<Arc<HashMap<String, Vec<String>>>> {
        self.model_access.get_edges_arc()
    }

    pub fn get_reverse_edges_arc(&self) -> Option<Arc<HashMap<String, Vec<String>>>> {
        self.model_access.get_reverse_edges_arc()
    }

    pub fn get_nodes_by_nodegroup_arc(&self) -> Option<Arc<HashMap<String, Vec<Arc<StaticNode>>>>> {
        self.model_access.get_nodes_by_nodegroup_arc()
    }

    pub fn get_nodegroups_arc(&self) -> Option<Arc<HashMap<String, Arc<StaticNodegroup>>>> {
        self.model_access.get_nodegroups_arc()
    }

    // Permission methods
    pub fn get_permitted_nodegroups(&mut self) -> HashMap<String, bool> {
        self.model_access.get_permitted_nodegroups_bool()
    }

    pub fn is_nodegroup_permitted(&self, nodegroup_id: &str) -> bool {
        self.model_access.is_nodegroup_permitted(nodegroup_id)
    }

    pub fn is_tile_permitted(&self, tile: &StaticTile) -> bool {
        self.model_access.is_tile_permitted(tile)
    }

    pub fn get_permission_rule(&self, nodegroup_id: &str) -> Option<&PermissionRule> {
        self.model_access.get_permission_rule(nodegroup_id)
    }

    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) {
        self.model_access.set_default_allow(default_allow);
    }

    pub fn set_permitted_nodegroups_rules(&mut self, permissions: HashMap<String, PermissionRule>) {
        self.model_access
            .set_permitted_nodegroups_rules(permissions);
    }

    pub fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, bool>) {
        self.model_access.set_permitted_nodegroups_bool(permissions);
    }

    // Graph mutation
    pub fn set_graph_nodes(&mut self, nodes: Vec<StaticNode>) {
        self.model_access.set_graph_nodes(nodes);
    }

    pub fn set_graph_edges(&mut self, edges: Vec<alizarin_core::StaticEdge>) {
        self.model_access.set_graph_edges(edges);
    }

    pub fn set_graph_nodegroups(&mut self, nodegroups: Vec<StaticNodegroup>) {
        self.model_access.set_graph_nodegroups(nodegroups);
    }

    pub fn set_graph(&mut self, graph: StaticGraph) {
        self.model_access.set_graph(Arc::new(graph));
    }

    pub fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> Result<(), String> {
        self.model_access.prune_graph(keep_functions.as_deref())
    }

    // Graph/WKRM access
    pub fn get_wkrm(&self) -> &WKRM {
        &self.wkrm
    }

    pub fn get_graph(&self) -> &StaticGraph {
        self.model_access.get_graph()
    }
}

/// ModelAccess implementation for ResourceModelWrapperCore.
/// Delegates to the inner GraphModelAccess.
impl alizarin_core::ModelAccess for ResourceModelWrapperCore {
    fn get_nodes(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.model_access.get_nodes_internal()
    }

    fn get_edges(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.model_access.get_edges_internal()
    }

    fn get_reverse_edges(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.model_access.get_reverse_edges_internal()
    }

    fn get_nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<Arc<StaticNode>>>> {
        self.model_access.get_nodes_by_nodegroup_internal()
    }

    fn get_nodegroups(&self) -> Option<&HashMap<String, Arc<StaticNodegroup>>> {
        self.model_access.get_nodegroups_internal()
    }

    fn get_permitted_nodegroups(&self) -> HashMap<String, alizarin_core::PermissionRule> {
        alizarin_core::ModelAccess::get_permitted_nodegroups(&self.model_access)
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
        self.with_core_mut(|core| core.ensure_built().map_err(|e| JsValue::from_str(&e)))?;

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
        self.with_core_mut(|core| core.ensure_built().map_err(|e| JsValue::from_str(&e)))?;

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
        // Extract path from JS object
        let path = js_sys::Reflect::get(value, &JsValue::from_str("path"))
            .map_err(|_| "failed to read 'path' property".to_string())?;
        let path_str = path.as_string().ok_or("'path' must be a string")?;

        // Extract allowed array from JS object
        let allowed = js_sys::Reflect::get(value, &JsValue::from_str("allowed"))
            .map_err(|_| "failed to read 'allowed' property".to_string())?;
        if !js_sys::Array::is_array(&allowed) {
            return Err("'allowed' must be an array".to_string());
        }
        let allowed_array = js_sys::Array::from(&allowed);
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

        // Delegate validation and construction to core
        PermissionRule::conditional(path_str, allowed_set)
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
