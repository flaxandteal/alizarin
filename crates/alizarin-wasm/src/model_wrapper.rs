use crate::graph::WKRM;
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
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

// The WKRM each model was registered with, by graph id: all WASM keeps of a
// model. The model itself (graph indices, permissions, pruning) is core's shared
// one (`alizarin_core::get_model`), as on NAPI and Python. WASM is
// single-threaded, so thread_local is safe.
thread_local! {
    pub(crate) static MODEL_REGISTRY: RefCell<HashMap<String, Rc<WKRM>>> =
        RefCell::new(HashMap::new());
}

/// A registered model as one call sees it: its WKRM and core's shared model as
/// it is now. Assembled per call (an `Rc` and an `Arc`), so it never goes
/// stale; changes go through core, never through a view.
#[derive(Clone)]
pub struct ResourceModelWrapperCore {
    wkrm: Rc<WKRM>,
    pub(crate) model_access: Arc<GraphModelAccess>,
}

impl ResourceModelWrapperCore {
    /// The view for the model `id` names (a model id, or a graph id for its
    /// latest); an error if no model wrapper registered it.
    pub(crate) fn for_graph(id: &str) -> Result<Self, String> {
        let model_access = alizarin_core::get_model(id)
            .ok_or_else(|| format!("No shared model registered for graph {id}"))?;
        let graph_id = model_access.get_graph_arc().graph_id().to_string();
        let wkrm = MODEL_REGISTRY
            .with(|registry| registry.borrow().get(&graph_id).cloned())
            .ok_or_else(|| format!("Model not found in registry: {graph_id}"))?;
        Ok(ResourceModelWrapperCore { wkrm, model_access })
    }

    // =========================================================================
    // Forwarding methods — keep the existing API surface so callers
    // (WASMResourceModelWrapper, instance_wrapper) need minimal changes
    // =========================================================================

    pub fn get_root_node(&self) -> Result<Arc<StaticNode>, String> {
        alizarin_core::ModelAccess::get_root_node(&*self.model_access)
    }

    pub fn get_child_nodes(&self, node_id: &str) -> Result<Arc<alizarin_core::ChildNodes>, String> {
        Ok(self.model_access.get_child_nodes(node_id))
    }

    pub fn get_node_objects(&self) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        self.model_access
            .get_nodes_internal()
            .ok_or_else(|| "graph nodes not built".to_string())
    }

    pub fn get_node_objects_by_alias(&self) -> Result<&HashMap<String, Arc<StaticNode>>, String> {
        self.model_access
            .get_nodes_by_alias_internal()
            .ok_or_else(|| "graph nodes by alias not built".to_string())
    }

    pub fn get_edges(&self) -> Result<&HashMap<String, Vec<String>>, String> {
        self.model_access
            .get_edges_internal()
            .ok_or_else(|| "graph edges not built".to_string())
    }

    pub fn get_nodegroup_objects(&self) -> Result<&HashMap<String, Arc<StaticNodegroup>>, String> {
        self.model_access
            .get_nodegroups_internal()
            .ok_or_else(|| "graph nodegroups not built".to_string())
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
    pub fn get_permitted_nodegroups(&self) -> HashMap<String, bool> {
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

    // Changes (permissions, graph, pruning) go through core's registry; see
    // WASMResourceModelWrapper's setters.

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

    fn get_child_nodes(&self, node_id: &str) -> Result<Arc<alizarin_core::ChildNodes>, String> {
        alizarin_core::ModelAccess::get_child_nodes(&*self.model_access, node_id)
    }

    fn get_node_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>> {
        alizarin_core::ModelAccess::get_node_by_alias(&*self.model_access, alias)
    }

    fn get_permitted_nodegroups(&self) -> HashMap<String, alizarin_core::PermissionRule> {
        alizarin_core::ModelAccess::get_permitted_nodegroups(&*self.model_access)
    }

    fn on_load_functions(&self) -> Arc<Vec<alizarin_core::DeriveConfig>> {
        alizarin_core::ModelAccess::on_load_functions(&*self.model_access)
    }
}

// WASM wrapper - lightweight reference that borrows from registry. Holds its
// core model's id (a graph id names that graph's latest model), so two
// wrappers over one graph keep their own permissions.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WASMResourceModelWrapper {
    graph_id: String,
}

impl WASMResourceModelWrapper {
    /// Run `f` on this model's view: its WKRM and core's shared model as it is now.
    pub(crate) fn with_core<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ResourceModelWrapperCore) -> R,
    {
        let core =
            ResourceModelWrapperCore::for_graph(&self.graph_id).unwrap_or_else(|e| panic!("{e}"));
        f(&core)
    }

    /// Throw a core error to JS, for setters that return nothing.
    fn or_throw(result: Result<(), String>) {
        if let Err(e) = result {
            wasm_bindgen::throw_str(&e);
        }
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
        // Register the graph and its shared model with core (one copy of the
        // graph, which core's graph registry shares); WASM keeps only the WKRM.
        let core_graph = graph.0.clone();
        let graph_id = core_graph.graph_id().to_string();
        let (model_id, _) =
            alizarin_core::register_model_with_id(Arc::new(core_graph), default_allow)
                .unwrap_or_else(|e| wasm_bindgen::throw_str(&e));

        MODEL_REGISTRY.with(|registry| {
            registry
                .borrow_mut()
                .insert(graph_id, Rc::new(wkrm.clone()));
        });

        // A lightweight wrapper holding its own model's id.
        WASMResourceModelWrapper { graph_id: model_id }
    }

    /// This wrapper's core model id: instance wrappers made with it use this
    /// model (its permissions and prunes), not the graph's latest.
    #[wasm_bindgen(getter, js_name = modelId)]
    pub fn model_id(&self) -> String {
        self.graph_id.clone()
    }

    /// Get a registered model by graph_id (returns None if not found or graph_id is null)
    #[wasm_bindgen(
        js_name = getModelByGraphId,
        unchecked_return_type = "WASMResourceModelWrapper | null"
    )]
    pub fn get_model_by_graph_id(graph_id: Option<String>) -> JsValue {
        let registered = graph_id.filter(|id| MODEL_REGISTRY.with(|r| r.borrow().contains_key(id)));
        crate::js_json::or_null(registered.map(|graph_id| WASMResourceModelWrapper { graph_id }))
    }

    /// Check if a model is registered (returns false if graph_id is null)
    #[wasm_bindgen(js_name = isModelRegistered)]
    pub fn is_model_registered(graph_id: Option<String>) -> bool {
        match graph_id {
            Some(id) => MODEL_REGISTRY.with(|registry| registry.borrow().contains_key(&id)),
            None => false,
        }
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
        // Core's, as NAPI and Python use: a child records its parent's path.
        self.with_core(|core| {
            let model: &dyn alizarin_core::ModelAccess = &*core.model_access;
            let made = match (parent_pseudo, child_node) {
                (Some(parent), Some(child)) => parent.core().child(&child, model),
                (Some(_), None) => {
                    Err("Must have a child node name if passing a parent".to_string())
                }
                (None, Some(alias)) => alizarin_core::PseudoNodeCore::by_alias(&alias, model),
                (None, None) => alizarin_core::PseudoNodeCore::root(model),
            };
            made.map(PseudoNode::from_core)
                .map_err(|e| JsValue::from_str(&e))
        })
    }

    #[wasm_bindgen(js_name = getRootNode)]
    pub fn get_root_node(&mut self) -> Result<WasmStaticNode, JsValue> {
        self.with_core(|core| {
            let arc_node = core.get_root_node().map_err(|e| JsValue::from_str(&e))?;
            // Dereference Arc to clone the StaticNode, wrap for JavaScript
            Ok(WasmStaticNode((*arc_node).clone()))
        })
    }

    #[wasm_bindgen(js_name = getNodeObjects)]
    pub fn get_node_objects(&mut self) -> Result<JsValue, JsValue> {
        self.with_core(|core| {
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
        self.with_core(|core| {
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
        self.with_core(|core| {
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
        self.with_core(|core| {
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
        self.with_core(|core| {
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
        self.with_core(|core| {
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
        self.with_core(|core| {
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
        self.with_core(|core| {
            let child_nodes = core
                .get_child_nodes(&node_id)
                .map_err(|e| JsValue::from_str(&e))?;
            Ok(alizarin_core::sorted_serde::sorted_keys(child_nodes.keys()))
        })
    }

    /// Get nodegroup IDs
    /// Returns array of IDs instead of full nodegroup objects
    #[wasm_bindgen(js_name = getNodegroupIds)]
    pub fn get_nodegroup_ids(&mut self) -> Result<Vec<String>, JsValue> {
        self.with_core(|core| {
            let nodegroups = core
                .get_nodegroup_objects()
                .map_err(|e| JsValue::from_str(&e))?;
            Ok(alizarin_core::sorted_serde::sorted_keys(nodegroups.keys()))
        })
    }

    /// Get nodegroup name by ID: its grouping node's name (core's rule).
    /// Returns just the name string instead of full node object
    /// Note: nodegroup_id is actually a node ID (the root node of the nodegroup)
    #[wasm_bindgen(js_name = getNodegroupName)]
    pub fn get_nodegroup_name(&mut self, nodegroup_id: Option<String>) -> Result<String, JsValue> {
        let nodegroup_id =
            nodegroup_id.ok_or_else(|| JsValue::from_str("nodegroup_id is null or undefined"))?;
        self.with_core(|core| {
            core.model_access
                .nodegroup_name(&nodegroup_id)
                .map_err(|e| JsValue::from_str(&e))
        })
    }

    /// Get node ID from alias
    /// Returns just the node ID string instead of full node map
    #[wasm_bindgen(js_name = getNodeIdFromAlias)]
    pub fn get_node_id_from_alias(&mut self, alias: Option<String>) -> Result<String, JsValue> {
        let alias = alias.ok_or_else(|| JsValue::from_str("alias is null or undefined"))?;
        self.with_core(|core| {
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
        self.with_core(|core| {
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
        Self::or_throw(alizarin_core::set_model_default_allow(
            &self.graph_id,
            default_allow,
        ));
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
        // To JSON, then core's one strict parser, as on NAPI and Python.
        let mut entries = serde_json::Map::new();
        let mut errors: Vec<String> = Vec::new();
        // for_each, not an iterator, to avoid iterator retention.
        permissions.for_each(&mut |value, key| match key.as_string() {
            Some(key) => match crate::js_json::from_value::<serde_json::Value>(value) {
                Ok(rule) => {
                    entries.insert(key, rule);
                }
                Err(e) => errors.push(format!("'{key}': not a JSON value ({e})")),
            },
            None => errors.push(format!("Invalid key (not a string): {key:?}")),
        });
        if !errors.is_empty() {
            return Err(JsValue::from_str(&format!(
                "Permission validation errors:\n  - {}",
                errors.join("\n  - ")
            )));
        }
        let perms = PermissionRule::rules_from_json(&serde_json::Value::Object(entries))
            .map_err(|e| JsValue::from_str(&e))?;

        // Core resolves alias keys to nodegroup ids, once, for every reader.
        alizarin_core::set_model_permissions(&self.graph_id, perms)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(())
    }

    // Getters
    #[wasm_bindgen(getter = graph)]
    pub fn get_graph(&self) -> WasmStaticGraph {
        self.with_core(|core| WasmStaticGraph(core.get_graph().clone()))
    }

    /// Replace the model's graph, rebuilding its indices: core's, as NAPI's
    /// setter. Borrows the graph (a by-value setter consumed the caller's
    /// object, so it couldn't be used again).
    #[wasm_bindgen(setter = graph)]
    pub fn set_graph(&mut self, graph: &WasmStaticGraph) {
        Self::or_throw(alizarin_core::update_model(&self.graph_id, |model| {
            model.rebuild_from_graph(&graph.0);
            Ok(())
        }));
    }

    #[wasm_bindgen(getter = edges)]
    pub fn get_edges_prop(&mut self) -> JsValue {
        self.get_edges().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = nodes)]
    pub fn get_nodes_prop(&mut self) -> JsValue {
        self.get_node_objects().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = nodegroups)]
    pub fn get_nodegroups_prop(&mut self) -> JsValue {
        self.get_nodegroup_objects().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = nodesByAlias)]
    pub fn get_nodes_by_alias_prop(&mut self) -> JsValue {
        self.get_node_objects_by_alias().unwrap_or(JsValue::NULL)
    }

    /// Prune graph to only include permitted nodegroups and their dependencies
    /// Filters nodes, edges, cards, nodegroups, and functions based on permissions
    #[wasm_bindgen(js_name = pruneGraph)]
    pub fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> Result<(), JsValue> {
        alizarin_core::update_model(&self.graph_id, |model| {
            model.prune_graph(keep_functions.as_deref())
        })
        .map_err(|e| JsValue::from_str(&e))
    }
}

/// How the ORM names a field: core's `describe_field` (`"<model> - <node
/// name>"`, a list's in brackets), as NAPI and Python give it.
#[wasm_bindgen(js_name = describeField)]
pub fn describe_field(model_name: Option<String>, node_name: &str, list: bool) -> String {
    alizarin_core::describe_field(model_name.as_deref(), node_name, list)
}

/// How the ORM names a field's group: core's `describe_field_group`.
#[wasm_bindgen(js_name = describeFieldGroup)]
pub fn describe_field_group(
    model_name: Option<String>,
    node_name: &str,
    nodegroup_name: Option<String>,
    list: bool,
) -> String {
    alizarin_core::describe_field_group(
        model_name.as_deref(),
        node_name,
        nodegroup_name.as_deref(),
        list,
    )
}
