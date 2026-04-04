/// PyO3 bindings for ResourceInstanceWrapperCore from alizarin-core
///
/// Provides Python bindings for the platform-agnostic instance wrapper logic,
/// enabling Python to use the same tile processing and pseudo cache population
/// as the WASM implementation.
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use alizarin_core::{
    ModelAccess, PopulateResult as CorePopulateResult, PseudoListCore, PseudoValueCore,
    ResourceInstanceWrapperCore, SemanticChildError, SemanticChildResult, StaticGraph, StaticNode,
    StaticNodegroup, StaticResourceMetadata, StaticTile,
};

use crate::node_config_py::PyNodeConfigManager;
use crate::pseudo_value_py::{PyPseudoList, PyPseudoValue};
use crate::rdm_cache_py;

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

            reverse_edges.entry(child_id).or_default().push(parent_id);
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
        self.nodes
            .get(&self.root_node_id)
            .cloned()
            .ok_or_else(|| "Root node not found".to_string())
    }

    fn get_child_nodes(&self, node_id: &str) -> Result<HashMap<String, Arc<StaticNode>>, String> {
        let child_ids = self.edges.get(node_id).cloned().unwrap_or_default();

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
        let graph = alizarin_core::get_graph(&graph_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Graph '{}' not registered. Call register_graph() first.",
                graph_id
            ))
        })?;

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
        let tiles: Vec<StaticTile> = serde_json::from_str(tiles_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse tiles JSON: {}",
                e
            ))
        })?;

        self.inner.load_tiles(tiles);
        Ok(())
    }

    /// Set resource metadata
    ///
    /// Args:
    ///     metadata_json: JSON string of resource metadata
    fn set_resource_metadata(&mut self, metadata_json: &str) -> PyResult<()> {
        let metadata: StaticResourceMetadata =
            serde_json::from_str(metadata_json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse metadata JSON: {}",
                    e
                ))
            })?;

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
        let model = self.model_access.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Model not initialized")
        })?;

        let result = self
            .inner
            .populate(lazy, &nodegroup_ids, &root_alias, model)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Populate failed: {}", e))
            })?;

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
        self.inner
            .get_cached_pseudo(alias)
            .map(PyPseudoList::from_core)
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
        let model = self.model_access.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Model not initialized")
        })?;

        let result = self
            .inner
            .get_semantic_child_value(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                &child_alias,
                model,
            )
            .map_err(|e| match e {
                SemanticChildError::TilesNotLoaded { nodegroup_id } => {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Tiles not loaded for nodegroup: {}",
                        nodegroup_id
                    ))
                }
                SemanticChildError::ChildNotFound { alias } => {
                    PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                        "Child node not found: {}",
                        alias
                    ))
                }
                SemanticChildError::TilesNotInitialized => {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Tiles not initialized")
                }
                SemanticChildError::ModelNotInitialized(msg) => {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Model not initialized: {}",
                        msg
                    ))
                }
                SemanticChildError::Other(msg) => {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(msg)
                }
            })?;

        Ok(PySemanticChildResult::from_core(result))
    }

    /// Resolve a dot-separated path and return a PseudoList for the target node.
    ///
    /// Walks the graph edges matching node aliases at each path segment (e.g. "building.name"),
    /// then retrieves matching tiles for the target node's nodegroup. Avoids full tree
    /// materialization.
    ///
    /// Args:
    ///     path: Dot-separated path of node aliases (e.g. "building.name")
    ///
    /// Returns:
    ///     RustPseudoList for the target node
    fn get_values_at_path(&self, path: &str) -> PyResult<PyPseudoList> {
        let model = self.model_access.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Model not initialized")
        })?;

        let result = self
            .inner
            .get_values_at_path(path, model)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyPseudoList::from_core(result))
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

    /// Serialize a single card from the populated pseudo_cache.
    ///
    /// Prerequisites: populate() must have been called for the relevant nodegroups.
    /// Use nodegroup_ids_for_card() to discover which nodegroups are needed.
    ///
    /// Args:
    ///     card_id: The card UUID to serialize
    ///     parent_tile_id: Parent tile ID for nested cards (None for root cards)
    ///     parent_nodegroup_id: Parent nodegroup ID for nested cards (None for root cards)
    ///     max_depth: Max recursion depth (None = unlimited, 0 = this card only)
    #[pyo3(signature = (card_id, parent_tile_id=None, parent_nodegroup_id=None, max_depth=None))]
    fn serialize_card(
        &self,
        py: Python,
        card_id: String,
        parent_tile_id: Option<String>,
        parent_nodegroup_id: Option<String>,
        max_depth: Option<usize>,
    ) -> PyResult<PyObject> {
        let graph = alizarin_core::get_graph(&self.graph_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Graph '{}' not registered",
                self.graph_id
            ))
        })?;

        let card_index = graph.card_index().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Graph has no card index — cards may not be loaded",
            )
        })?;

        let cache = self.inner.pseudo_cache.lock().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache lock failed: {}", e))
        })?;

        let result = alizarin_core::serialize_card(
            &card_id,
            card_index,
            &cache,
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            &graph,
            max_depth,
            None,
        )
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        pythonize::pythonize(py, &result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Serialize all root cards from the populated pseudo_cache.
    ///
    /// Args:
    ///     max_depth: Max recursion depth (None = unlimited, 0 = root cards only)
    #[pyo3(signature = (max_depth=None))]
    fn serialize_root_cards(&self, py: Python, max_depth: Option<usize>) -> PyResult<PyObject> {
        let graph = alizarin_core::get_graph(&self.graph_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Graph '{}' not registered",
                self.graph_id
            ))
        })?;

        let card_index = graph.card_index().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Graph has no card index — cards may not be loaded",
            )
        })?;

        let cache = self.inner.pseudo_cache.lock().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache lock failed: {}", e))
        })?;

        let result =
            alizarin_core::serialize_root_cards(card_index, &cache, &graph, max_depth, None);

        pythonize::pythonize(py, &result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Serialize a single card in display mode (resolves UUIDs to labels
    /// using the global RDM cache and optional node config manager).
    ///
    /// Args:
    ///     card_id: The card UUID to serialize
    ///     node_config_manager: PyNodeConfigManager for domain value / boolean label lookups
    ///     parent_tile_id: Parent tile ID for nested cards (None for root cards)
    ///     parent_nodegroup_id: Parent nodegroup ID for nested cards (None for root cards)
    ///     max_depth: Max recursion depth (None = unlimited, 0 = this card only)
    ///     language: Language code for display labels (defaults to "en")
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (card_id, node_config_manager=None, parent_tile_id=None, parent_nodegroup_id=None, max_depth=None, language=None))]
    fn serialize_card_display(
        &self,
        py: Python,
        card_id: String,
        node_config_manager: Option<&PyNodeConfigManager>,
        parent_tile_id: Option<String>,
        parent_nodegroup_id: Option<String>,
        max_depth: Option<usize>,
        language: Option<String>,
    ) -> PyResult<PyObject> {
        let graph = alizarin_core::get_graph(&self.graph_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Graph '{}' not registered",
                self.graph_id
            ))
        })?;

        let card_index = graph.card_index().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Graph has no card index — cards may not be loaded",
            )
        })?;

        let cache = self.inner.pseudo_cache.lock().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache lock failed: {}", e))
        })?;

        let lang = language.unwrap_or_else(|| "en".to_string());
        let opts = alizarin_core::SerializationOptions::display(&lang);

        let global_rdm = rdm_cache_py::get_global_rdm_cache();
        let rdm_inner = global_rdm.as_ref().map(|c| c.inner());
        let ser_ctx = alizarin_core::type_serialization::SerializationContext {
            node_config: None,
            external_resolver: rdm_inner
                .map(|r| r as &dyn alizarin_core::type_serialization::ExternalResolver),
            extension_registry: None,
        };

        let ncm = node_config_manager.map(|m| m.inner());

        let result = alizarin_core::serialize_card(
            &card_id,
            card_index,
            &cache,
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            &graph,
            max_depth,
            Some(alizarin_core::card_traversal::CardSerializationParams {
                opts: &opts,
                ser_ctx: &ser_ctx,
                node_config_manager: ncm,
            }),
        )
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        pythonize::pythonize(py, &result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Serialize all root cards in display mode.
    ///
    /// Args:
    ///     node_config_manager: PyNodeConfigManager for domain value / boolean label lookups
    ///     max_depth: Max recursion depth (None = unlimited, 0 = root cards only)
    ///     language: Language code for display labels (defaults to "en")
    #[pyo3(signature = (node_config_manager=None, max_depth=None, language=None))]
    fn serialize_root_cards_display(
        &self,
        py: Python,
        node_config_manager: Option<&PyNodeConfigManager>,
        max_depth: Option<usize>,
        language: Option<String>,
    ) -> PyResult<PyObject> {
        let graph = alizarin_core::get_graph(&self.graph_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Graph '{}' not registered",
                self.graph_id
            ))
        })?;

        let card_index = graph.card_index().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Graph has no card index — cards may not be loaded",
            )
        })?;

        let cache = self.inner.pseudo_cache.lock().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache lock failed: {}", e))
        })?;

        let lang = language.unwrap_or_else(|| "en".to_string());
        let opts = alizarin_core::SerializationOptions::display(&lang);

        let global_rdm = rdm_cache_py::get_global_rdm_cache();
        let rdm_inner = global_rdm.as_ref().map(|c| c.inner());
        let ser_ctx = alizarin_core::type_serialization::SerializationContext {
            node_config: None,
            external_resolver: rdm_inner
                .map(|r| r as &dyn alizarin_core::type_serialization::ExternalResolver),
            extension_registry: None,
        };

        let ncm = node_config_manager.map(|m| m.inner());

        let result = alizarin_core::serialize_root_cards(
            card_index,
            &cache,
            &graph,
            max_depth,
            Some(alizarin_core::card_traversal::CardSerializationParams {
                opts: &opts,
                ser_ctx: &ser_ctx,
                node_config_manager: ncm,
            }),
        );

        pythonize::pythonize(py, &result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Serialize the populated pseudo_cache as a node-hierarchy display tree.
    ///
    /// This is the Python equivalent of WASM's `toDisplayJson()`. It traverses
    /// the pseudo_cache using the node/edge graph hierarchy (not card hierarchy)
    /// and serializes in display mode — resolving UUIDs to labels for concepts,
    /// domain values, booleans, etc.
    ///
    /// Prerequisites: `populate()` must have been called.
    ///
    /// Args:
    ///     node_config_manager: Optional PyNodeConfigManager for domain/boolean lookups
    ///     language: Language code for display labels (defaults to "en")
    #[pyo3(signature = (node_config_manager=None, language=None))]
    fn to_display_json(
        &self,
        py: Python,
        node_config_manager: Option<&crate::node_config_py::PyNodeConfigManager>,
        language: Option<String>,
    ) -> PyResult<PyObject> {
        let graph = alizarin_core::get_graph(&self.graph_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Graph '{}' not registered",
                self.graph_id
            ))
        })?;

        let nodes_by_alias = graph.nodes_by_alias_arc().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Graph indices not built")
        })?;

        let edges = graph.edges_map().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Graph indices not built")
        })?;

        let root_node = graph.root_node();
        let root_alias = root_node.alias.clone().unwrap_or_default();

        let cache = self.inner.pseudo_cache.lock().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache lock failed: {}", e))
        })?;

        let root_list = cache.get(&root_alias).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Root pseudo not found in cache for alias '{}' — was populate() called?",
                root_alias
            ))
        })?;

        let lang = language.unwrap_or_else(|| "en".to_string());
        let global_rdm = rdm_cache_py::get_global_rdm_cache();
        let rdm_inner = global_rdm.as_ref().map(|c| c.inner());
        let ncm = node_config_manager.map(|m| m.inner());

        let ser_ctx = alizarin_core::type_serialization::SerializationContext {
            node_config: None,
            external_resolver: rdm_inner
                .map(|r| r as &dyn alizarin_core::type_serialization::ExternalResolver),
            extension_registry: None,
        };

        let ctx = alizarin_core::pseudo_value_core::VisitorContext {
            pseudo_cache: &*cache,
            nodes_by_alias,
            edges,
            depth: 0,
            max_depth: 50,
            serialization_options: alizarin_core::SerializationOptions::display(&lang),
            serialization_context: ser_ctx,
            node_config_manager: ncm,
        };

        let json = root_list.to_json(&ctx);

        pythonize::pythonize(py, &json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Get nodegroup IDs needed to serialize a card (and its descendants).
    ///
    /// Use this to know which nodegroups to pass to populate() before calling
    /// serialize_card().
    ///
    /// Args:
    ///     card_id: The card UUID
    ///     max_depth: Max depth (None = all descendants, 0 = this card only)
    #[pyo3(signature = (card_id, max_depth=None))]
    fn nodegroup_ids_for_card(
        &self,
        card_id: String,
        max_depth: Option<usize>,
    ) -> PyResult<Vec<String>> {
        let graph = alizarin_core::get_graph(&self.graph_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Graph '{}' not registered",
                self.graph_id
            ))
        })?;

        let card_index = graph.card_index().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Graph has no card index — cards may not be loaded",
            )
        })?;

        Ok(card_index.nodegroup_ids_for_card(&card_id, max_depth))
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
        self.values
            .iter()
            .map(|(k, v)| (k.clone(), PyPseudoList::from_core(v.clone())))
            .collect()
    }

    /// Get a specific value by alias
    fn get_value(&self, alias: &str) -> Option<PyPseudoList> {
        self.values
            .get(alias)
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
        self.list_value
            .as_ref()
            .map(|v| PyPseudoList::from_core(v.clone()))
    }

    /// Get as single value (returns None if not a single value)
    fn as_single(&self) -> Option<PyPseudoValue> {
        self.single_value
            .as_ref()
            .map(|v| PyPseudoValue::from_core(v.clone()))
    }

    /// Get as list, converting single values to single-item lists
    fn to_list(&self) -> Option<PyPseudoList> {
        match (&self.list_value, &self.single_value) {
            (Some(list), _) => Some(PyPseudoList::from_core(list.clone())),
            (None, Some(single)) => {
                let alias = single.node.alias.clone().unwrap_or_default();
                let list =
                    PseudoListCore::from_values_with_cardinality(alias, vec![single.clone()], true);
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
                self.list_value
                    .as_ref()
                    .map(|v| v.values.len())
                    .unwrap_or(0)
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
