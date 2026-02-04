/// PyO3 bindings for ResourceInstanceWrapperCore from alizarin-core
///
/// Provides Python bindings for the platform-agnostic instance wrapper logic,
/// enabling Python to use the same tile processing and pseudo cache population
/// as the WASM implementation.

use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use alizarin_core::{
    StaticGraph, StaticNode, StaticNodegroup, StaticTile, StaticResourceMetadata,
    PseudoListCore, PseudoValueCore,
    ResourceInstanceWrapperCore, ModelAccess, SemanticChildError, SemanticChildResult,
    PopulateResult as CorePopulateResult,
};

use crate::pseudo_value_py::{PyPseudoValue, PyPseudoList};

// =============================================================================
// ModelAccess implementation using registered graphs
// =============================================================================

/// ModelAccess implementation that uses the graph registry
struct RegistryModelAccess {
    nodes: HashMap<String, Arc<StaticNode>>,
    edges: HashMap<String, Vec<String>>,
    reverse_edges: HashMap<String, Vec<String>>,
    nodes_by_nodegroup: HashMap<String, Vec<Arc<StaticNode>>>,
    nodegroups: HashMap<String, Arc<StaticNodegroup>>,
    root_node_id: String,
    permitted_nodegroups: HashMap<String, bool>,
}

impl RegistryModelAccess {
    /// Build from a registered graph
    fn from_graph(graph: &StaticGraph) -> Self {
        let mut nodes: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut nodes_by_nodegroup: HashMap<String, Vec<Arc<StaticNode>>> = HashMap::new();
        let mut nodegroups: HashMap<String, Arc<StaticNodegroup>> = HashMap::new();
        let mut root_node_id = String::new();

        // Build node index
        for node in &graph.nodes {
            let arc_node = Arc::new(node.clone());
            nodes.insert(node.nodeid.clone(), Arc::clone(&arc_node));

            // Track root node
            if node.istopnode {
                root_node_id = node.nodeid.clone();
            }

            // Index by nodegroup
            if let Some(ref ng_id) = node.nodegroup_id {
                nodes_by_nodegroup
                    .entry(ng_id.clone())
                    .or_default()
                    .push(Arc::clone(&arc_node));
            }
        }

        // Build edge indices from graph edges
        for edge in &graph.edges {
            let parent_id = edge.domainnode_id.clone();
            let child_id = edge.rangenode_id.clone();

            edges
                .entry(parent_id.clone())
                .or_default()
                .push(child_id.clone());

            reverse_edges
                .entry(child_id)
                .or_default()
                .push(parent_id);
        }

        // Build nodegroup index
        for ng in &graph.nodegroups {
            nodegroups.insert(ng.nodegroupid.clone(), Arc::new(ng.clone()));
        }

        RegistryModelAccess {
            nodes,
            edges,
            reverse_edges,
            nodes_by_nodegroup,
            nodegroups,
            root_node_id,
            permitted_nodegroups: HashMap::new(),
        }
    }

    /// Set permitted nodegroups
    fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, bool>) {
        self.permitted_nodegroups = permissions;
    }
}

impl ModelAccess for RegistryModelAccess {
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
        self.nodes.get(&self.root_node_id)
            .cloned()
            .ok_or_else(|| "Root node not found".to_string())
    }

    fn get_child_nodes(&self, node_id: &str) -> Result<HashMap<String, Arc<StaticNode>>, String> {
        let child_ids = self.edges.get(node_id)
            .cloned()
            .unwrap_or_default();

        let mut children = HashMap::new();
        for child_id in child_ids {
            if let Some(node) = self.nodes.get(&child_id) {
                if let Some(ref alias) = node.alias {
                    children.insert(alias.clone(), Arc::clone(node));
                }
            }
        }
        Ok(children)
    }

    fn get_permitted_nodegroups(&self) -> HashMap<String, bool> {
        self.permitted_nodegroups.clone()
    }
}

// =============================================================================
// PyResourceInstanceWrapperCore - Python wrapper
// =============================================================================

/// Python wrapper for ResourceInstanceWrapperCore.
///
/// Provides Rust-backed instance wrapper with the same capabilities as WASM,
/// enabling cross-model traversal and proper tile processing in Python.
#[pyclass(name = "RustResourceInstanceWrapperCore")]
pub struct PyResourceInstanceWrapperCore {
    inner: ResourceInstanceWrapperCore,
    model_access: Option<RegistryModelAccess>,
    graph_id: String,
}

#[pymethods]
impl PyResourceInstanceWrapperCore {
    /// Create a new PyResourceInstanceWrapperCore
    ///
    /// Args:
    ///     graph_id: The graph ID (must be registered via register_graph first)
    #[new]
    fn new(graph_id: String) -> PyResult<Self> {
        // Get graph from registry
        let graph = alizarin_core::get_graph(&graph_id)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                format!("Graph '{}' not registered. Call register_graph() first.", graph_id)
            ))?;

        // Build model access
        let model_access = RegistryModelAccess::from_graph(&graph);

        Ok(PyResourceInstanceWrapperCore {
            inner: ResourceInstanceWrapperCore::new(graph_id.clone()),
            model_access: Some(model_access),
            graph_id,
        })
    }

    /// Get the graph ID
    #[getter]
    fn graph_id(&self) -> String {
        self.graph_id.clone()
    }

    /// Load tiles into the wrapper
    ///
    /// Args:
    ///     tiles_json: JSON string of tiles array
    fn load_tiles(&mut self, tiles_json: &str) -> PyResult<()> {
        let tiles: Vec<StaticTile> = serde_json::from_str(tiles_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to parse tiles JSON: {}", e)
            ))?;

        self.inner.load_tiles(tiles);
        Ok(())
    }

    /// Set resource metadata
    ///
    /// Args:
    ///     metadata_json: JSON string of resource metadata
    fn set_resource_metadata(&mut self, metadata_json: &str) -> PyResult<()> {
        let metadata: StaticResourceMetadata = serde_json::from_str(metadata_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to parse metadata JSON: {}", e)
            ))?;

        self.inner.resource_instance = Some(metadata);
        Ok(())
    }

    /// Set permitted nodegroups
    ///
    /// Args:
    ///     permissions: Dict of nodegroup_id -> is_permitted
    fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, bool>) -> PyResult<()> {
        if let Some(ref mut model_access) = self.model_access {
            model_access.set_permitted_nodegroups(permissions);
        }
        Ok(())
    }

    /// Populate the pseudo cache
    ///
    /// Args:
    ///     lazy: If True, only populate on-demand
    ///     nodegroup_ids: List of nodegroup IDs to process
    ///     root_alias: Alias of the root node
    ///
    /// Returns:
    ///     Dict with 'values' (dict of alias -> RustPseudoList)
    fn populate(
        &mut self,
        lazy: bool,
        nodegroup_ids: Vec<String>,
        root_alias: String,
    ) -> PyResult<PyPopulateResult> {
        let model = self.model_access.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Model not initialized"
            ))?;

        let result = self.inner.populate(lazy, &nodegroup_ids, &root_alias, model)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Populate failed: {}", e)
            ))?;

        Ok(PyPopulateResult::from_core(result))
    }

    /// Get a cached pseudo list by alias
    ///
    /// Args:
    ///     alias: The node alias
    ///
    /// Returns:
    ///     RustPseudoList if found, None otherwise
    fn get_cached_pseudo(&self, alias: &str) -> Option<PyPseudoList> {
        self.inner.get_cached_pseudo(alias)
            .map(|core| PyPseudoList::from_core(core))
    }

    /// Check if a nodegroup is loaded
    fn is_nodegroup_loaded(&self, nodegroup_id: &str) -> bool {
        self.inner.is_nodegroup_loaded(nodegroup_id)
    }

    /// Mark a nodegroup as loaded
    fn mark_nodegroup_loaded(&self, nodegroup_id: &str) {
        self.inner.mark_nodegroup_loaded(nodegroup_id)
    }

    /// Get semantic child value
    ///
    /// Args:
    ///     parent_tile_id: Tile ID of the parent (or None for root)
    ///     parent_node_id: Node ID of the parent
    ///     parent_nodegroup_id: Nodegroup ID of the parent (or None)
    ///     child_alias: Alias of the child node
    ///
    /// Returns:
    ///     PySemanticChildResult containing the values
    #[pyo3(signature = (parent_tile_id, parent_node_id, parent_nodegroup_id, child_alias))]
    fn get_semantic_child_value(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
        child_alias: String,
    ) -> PyResult<PySemanticChildResult> {
        let model = self.model_access.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Model not initialized"
            ))?;

        let result = self.inner.get_semantic_child_value(
            parent_tile_id.as_ref(),
            &parent_node_id,
            parent_nodegroup_id.as_ref(),
            &child_alias,
            model,
        ).map_err(|e| match e {
            SemanticChildError::TilesNotLoaded { nodegroup_id } => {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Tiles not loaded for nodegroup: {}", nodegroup_id)
                )
            }
            SemanticChildError::ChildNotFound { alias } => {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                    format!("Child node not found: {}", alias)
                )
            }
            SemanticChildError::TilesNotInitialized => {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Tiles not initialized"
                )
            }
            SemanticChildError::ModelNotInitialized(msg) => {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Model not initialized: {}", msg)
                )
            }
            SemanticChildError::Other(msg) => {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(msg)
            }
        })?;

        Ok(PySemanticChildResult::from_core(result))
    }

    /// Get all nodegroup IDs that have tiles
    fn get_loaded_nodegroup_ids(&self) -> Vec<String> {
        self.inner.nodegroup_index.keys().cloned().collect()
    }

    /// Get count of tiles
    fn tile_count(&self) -> usize {
        self.inner.tiles.as_ref().map(|t| t.len()).unwrap_or(0)
    }

    /// Get count of cached pseudos
    fn cache_size(&self) -> usize {
        if let Ok(cache) = self.inner.pseudo_cache.lock() {
            cache.len()
        } else {
            0
        }
    }

    /// Clear the pseudo cache
    fn clear_cache(&self) {
        if let Ok(mut cache) = self.inner.pseudo_cache.lock() {
            cache.clear();
        }
        if let Ok(mut loaded) = self.inner.loaded_nodegroups.lock() {
            loaded.clear();
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "RustResourceInstanceWrapperCore(graph_id={}, tiles={}, cached={})",
            self.graph_id,
            self.tile_count(),
            self.cache_size()
        )
    }
}

// =============================================================================
// Result types
// =============================================================================

/// Python wrapper for PopulateResult
#[pyclass(name = "RustPopulateResult")]
pub struct PyPopulateResult {
    values: HashMap<String, PseudoListCore>,
    /// Stored for parity with WASM; expose via getter if Python needs access
    #[allow(dead_code)]
    all_values_map: HashMap<String, Option<bool>>,
    all_nodegroups_map: HashMap<String, bool>,
}

#[pymethods]
impl PyPopulateResult {
    /// Get the values dict (alias -> RustPseudoList)
    fn get_values(&self) -> HashMap<String, PyPseudoList> {
        self.values.iter()
            .map(|(k, v)| (k.clone(), PyPseudoList::from_core(v.clone())))
            .collect()
    }

    /// Get a specific value by alias
    fn get_value(&self, alias: &str) -> Option<PyPseudoList> {
        self.values.get(alias)
            .map(|v| PyPseudoList::from_core(v.clone()))
    }

    /// Get all aliases
    fn aliases(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    /// Get all nodegroups map
    fn get_nodegroups_map(&self) -> HashMap<String, bool> {
        self.all_nodegroups_map.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "RustPopulateResult(aliases={}, nodegroups={})",
            self.values.len(),
            self.all_nodegroups_map.len()
        )
    }
}

impl PyPopulateResult {
    fn from_core(result: CorePopulateResult) -> Self {
        PyPopulateResult {
            values: result.values,
            all_values_map: result.all_values_map,
            all_nodegroups_map: result.all_nodegroups_map,
        }
    }
}

/// Python wrapper for SemanticChildResult
#[pyclass(name = "RustSemanticChildResult")]
pub struct PySemanticChildResult {
    result_type: String,
    list_value: Option<PseudoListCore>,
    single_value: Option<PseudoValueCore>,
}

#[pymethods]
impl PySemanticChildResult {
    /// Get the result type: "list", "single", or "empty"
    #[getter]
    fn result_type(&self) -> String {
        self.result_type.clone()
    }

    /// Check if result is empty
    #[getter]
    fn is_empty(&self) -> bool {
        self.result_type == "empty"
    }

    /// Get as list (returns None if not a list type)
    fn as_list(&self) -> Option<PyPseudoList> {
        self.list_value.as_ref()
            .map(|v| PyPseudoList::from_core(v.clone()))
    }

    /// Get as single value (returns None if not a single value)
    fn as_single(&self) -> Option<PyPseudoValue> {
        self.single_value.as_ref()
            .map(|v| PyPseudoValue::from_core(v.clone()))
    }

    /// Get as list, converting single values to single-item lists
    fn to_list(&self) -> Option<PyPseudoList> {
        match (&self.list_value, &self.single_value) {
            (Some(list), _) => Some(PyPseudoList::from_core(list.clone())),
            (None, Some(single)) => {
                let alias = single.node.alias.clone().unwrap_or_default();
                let list = PseudoListCore::from_values_with_cardinality(
                    alias,
                    vec![single.clone()],
                    true,
                );
                Some(PyPseudoList::from_core(list))
            }
            _ => None,
        }
    }

    fn __repr__(&self) -> String {
        match self.result_type.as_str() {
            "empty" => "RustSemanticChildResult(empty)".to_string(),
            "single" => format!(
                "RustSemanticChildResult(single, alias={:?})",
                self.single_value.as_ref().map(|v| v.node.alias.clone())
            ),
            "list" => format!(
                "RustSemanticChildResult(list, len={})",
                self.list_value.as_ref().map(|v| v.values.len()).unwrap_or(0)
            ),
            _ => "RustSemanticChildResult(unknown)".to_string(),
        }
    }
}

impl PySemanticChildResult {
    fn from_core(result: SemanticChildResult) -> Self {
        match result {
            SemanticChildResult::List(list) => PySemanticChildResult {
                result_type: "list".to_string(),
                list_value: Some(list),
                single_value: None,
            },
            SemanticChildResult::Single(single) => PySemanticChildResult {
                result_type: "single".to_string(),
                list_value: None,
                single_value: Some(single),
            },
            SemanticChildResult::Empty => PySemanticChildResult {
                result_type: "empty".to_string(),
                list_value: None,
                single_value: None,
            },
        }
    }
}

// =============================================================================
// Module Registration
// =============================================================================

/// Register instance wrapper types with the Python module
pub fn register_module(m: &pyo3::types::PyModule) -> PyResult<()> {
    m.add_class::<PyResourceInstanceWrapperCore>()?;
    m.add_class::<PyPopulateResult>()?;
    m.add_class::<PySemanticChildResult>()?;
    Ok(())
}
