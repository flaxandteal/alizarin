/// PyO3 bindings for ResourceInstanceWrapperCore from alizarin-core
///
/// Provides Python bindings for the platform-agnostic instance wrapper logic,
/// enabling Python to use the same tile processing and pseudo cache population
/// as the WASM implementation.
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::sync::Arc;

use alizarin_core::{
    GraphModelAccess, PermissionRule, PopulateResult as CorePopulateResult, PseudoListCore,
    PseudoValueCore, ResourceInstanceWrapperCore, SemanticChildError, SemanticChildResult,
    StaticTile,
};

use crate::node_config_py::PyNodeConfigManager;
use crate::pseudo_value_py::{PyPseudoList, PyPseudoValue};

// =============================================================================
// Permission parsing (shared logic)
// =============================================================================

/// Parse a Python dict of permissions into `HashMap<String, PermissionRule>`.
///
/// Each value may be a `bool` or a `{path: str, allowed: [str]}` dict for
/// conditional rules, matching the format accepted by `PyResourceModelWrapper`.
pub(crate) fn parse_permissions_dict(
    py: Python,
    permissions: &PyObject,
) -> PyResult<HashMap<String, PermissionRule>> {
    let dict = permissions.downcast_bound::<PyDict>(py).map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyTypeError, _>("permissions must be a dict")
    })?;
    // To JSON, then core's one strict parser, as on WASM and NAPI.
    let json = crate::python_json::python_to_json(py, dict.as_any())?;
    PermissionRule::rules_from_json(&json).map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
}

// =============================================================================
// ModelAccess — uses GraphModelAccess from alizarin-core
// =============================================================================

// GraphModelAccess (imported from alizarin_core) provides the ModelAccess
// implementation built from a StaticGraph. Previously this was a local
// RegistryModelAccess struct with identical logic.

// =============================================================================
// PyResourceInstanceWrapperCore - Python wrapper
// =============================================================================

/// The shared model core holds for `graph_id`, registered by its model wrapper.
pub(crate) fn model_for(graph_id: &str) -> PyResult<Arc<GraphModelAccess>> {
    alizarin_core::get_model(graph_id).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
            "No model registered for graph '{graph_id}'. Create its model wrapper first."
        ))
    })
}

/// The resolvers and language a display-mode card serialization uses.
struct CardDisplay<'a> {
    ncm: Option<&'a alizarin_core::node_config::NodeConfigManager>,
    registry: Option<&'a alizarin_core::StaticResourceRegistry>,
    language: String,
}

/// A semantic-child error as a Python exception, keeping core's text
/// (`TILES_NOT_LOADED:<nodegroups>` for the lazy loader), as on NAPI/WASM.
fn sc_err(e: SemanticChildError) -> PyErr {
    match e {
        e @ SemanticChildError::TilesNotLoaded { .. } => {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
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
        SemanticChildError::Other(msg) => PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(msg),
    }
}

/// Python wrapper for ResourceInstanceWrapperCore.
///
/// It holds no model of its own: it looks up its graph's shared model on each
/// use, so it sees permission changes and prunes made after it was created.
///
/// Provides Rust-backed instance wrapper with the same capabilities as WASM,
/// enabling cross-model traversal and proper tile processing in Python.
#[pyclass(name = "RustResourceInstanceWrapperCore")]
pub struct PyResourceInstanceWrapperCore {
    inner: ResourceInstanceWrapperCore,
    graph_id: String,
}

impl PyResourceInstanceWrapperCore {
    /// This instance's graph's shared model, as it is now.
    fn model(&self) -> PyResult<Arc<GraphModelAccess>> {
        model_for(&self.graph_id)
    }

    /// Set a compiled-in tile source, tried before the host's loader
    /// (`fetch_from_tile_source`). Rust-only, as on NAPI and WASM: for a
    /// binary that compiles alizarin and a tile backend together.
    pub fn set_tile_source(&mut self, source: Arc<dyn alizarin_core::TileSource>) {
        self.inner.tile_source = Some(source);
    }

    /// Remove the compiled-in tile source.
    pub fn clear_tile_source(&mut self) {
        self.inner.tile_source = None;
    }

    /// Core's `serialize_cards`, in display mode with `display`'s resolvers
    /// (concept labels through the global RDM cache): what the
    /// card-serialization methods share, as on NAPI and WASM.
    fn serialize_cards_with(
        &self,
        py: Python,
        card_id: Option<&str>,
        parent_tile_id: Option<&str>,
        parent_nodegroup_id: Option<&str>,
        max_depth: Option<usize>,
        display: Option<CardDisplay>,
    ) -> PyResult<PyObject> {
        let model = self.model()?;
        let result = match display {
            None => self.inner.serialize_cards(
                card_id,
                parent_tile_id,
                parent_nodegroup_id,
                max_depth,
                &model,
                None,
            ),
            Some(display) => {
                let opts = alizarin_core::SerializationOptions::display(&display.language);
                // Shared, not copied (get_global_rdm_cache copies it).
                let global_rdm = alizarin_core::get_global_rdm_cache_arc();
                let ext_registry = crate::build_extension_registry();
                let ser_ctx = alizarin_core::type_serialization::SerializationContext {
                    node_config: None,
                    external_resolver: global_rdm
                        .as_deref()
                        .map(|c| c as &dyn alizarin_core::type_serialization::ExternalResolver),
                    concept_lookup: None,
                    resource_resolver: display.registry.map(|r| {
                        r as &dyn alizarin_core::type_serialization::ResourceDisplayResolver
                    }),
                    extension_registry: Some(&ext_registry),
                };
                self.inner.serialize_cards(
                    card_id,
                    parent_tile_id,
                    parent_nodegroup_id,
                    max_depth,
                    &model,
                    Some(alizarin_core::card_traversal::CardSerializationParams {
                        opts: &opts,
                        ser_ctx: &ser_ctx,
                        node_config_manager: display.ncm,
                    }),
                )
            }
        }
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
        pythonize::pythonize(py, &result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Serialise the resource as a node-hierarchy tree with `options`,
    /// populating from the loaded tiles first if needed: what `to_json` and
    /// `to_display_json` share. `resolve` resolves concept labels through the
    /// global RDM cache, as display mode needs.
    fn serialize(
        &self,
        py: Python,
        options: alizarin_core::SerializationOptions,
        node_config_manager: Option<&alizarin_core::node_config::NodeConfigManager>,
        resource_registry: Option<&alizarin_core::StaticResourceRegistry>,
        resolve: bool,
    ) -> PyResult<PyObject> {
        // The model's graph: the registered one, or as pruned since.
        let model = self.model()?;
        let graph = model.get_graph_arc();

        // Populated from the loaded tiles first if needed, as on NAPI and WASM.
        self.inner
            .ensure_populated(&*model)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let nodes_by_alias = graph.nodes_by_alias_arc().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Graph indices not built")
        })?;
        let edges = graph.edges_map().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Graph indices not built")
        })?;
        let root_alias = graph.root_node().alias.clone().unwrap_or_default();

        let cache = self.inner.pseudo_cache.read().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache lock failed: {}", e))
        })?;
        let root_list = cache.get(&root_alias).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Root pseudo not found in cache for alias '{}'",
                root_alias
            ))
        })?;

        // Shared, not copied (get_global_rdm_cache copies it).
        let global_rdm = if resolve {
            alizarin_core::get_global_rdm_cache_arc()
        } else {
            None
        };
        let ext_registry = crate::build_extension_registry();
        let ser_ctx = alizarin_core::type_serialization::SerializationContext {
            node_config: None,
            external_resolver: global_rdm
                .as_deref()
                .map(|c| c as &dyn alizarin_core::type_serialization::ExternalResolver),
            concept_lookup: None,
            resource_resolver: resource_registry
                .map(|r| r as &dyn alizarin_core::type_serialization::ResourceDisplayResolver),
            extension_registry: Some(&ext_registry),
        };
        let ctx = alizarin_core::pseudo_value_core::VisitorContext {
            pseudo_cache: &*cache,
            nodes_by_alias,
            edges,
            depth: 0,
            max_depth: 50,
            serialization_options: options,
            serialization_context: ser_ctx,
            node_config_manager,
        };

        let json = root_list.to_json(&ctx);
        pythonize::pythonize(py, &json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }
}

#[pymethods]
impl PyResourceInstanceWrapperCore {
    /// Create a new PyResourceInstanceWrapperCore
    ///
    /// Args:
    ///     graph_id: The graph ID (its model wrapper must exist)
    #[new]
    fn new(graph_id: String) -> PyResult<Self> {
        // Its graph's model must be registered; the instance builds nothing.
        model_for(&graph_id)?;

        Ok(PyResourceInstanceWrapperCore {
            inner: ResourceInstanceWrapperCore::new(graph_id.clone()),
            graph_id,
        })
    }

    /// Load tiles into the wrapper
    ///
    /// Args:
    ///     tiles_json: JSON string of tiles array
    ///     assume_tiles_comprehensive_for_nodegroup: the resource's complete
    ///         set, so every nodegroup counts as loaded (as on NAPI/WASM)
    #[pyo3(signature = (tiles_json, assume_tiles_comprehensive_for_nodegroup=true))]
    fn load_tiles(
        &mut self,
        tiles_json: &str,
        assume_tiles_comprehensive_for_nodegroup: bool,
    ) -> PyResult<()> {
        let tiles: Vec<StaticTile> = serde_json::from_str(tiles_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse tiles JSON: {}",
                e
            ))
        })?;

        let model = self.model()?;
        self.inner.load_permitted_tiles(tiles, &model);
        if assume_tiles_comprehensive_for_nodegroup {
            self.inner.mark_loaded_comprehensively(&model);
        }
        Ok(())
    }

    /// Append the tiles a lazy fetch returned for `nodegroup_id` (None: the
    /// whole resource), marking just that nodegroup loaded, even if it had no
    /// tiles (as on NAPI/WASM).
    #[pyo3(signature = (tiles_json, nodegroup_id=None))]
    fn append_fetched_tiles(
        &mut self,
        tiles_json: &str,
        nodegroup_id: Option<&str>,
    ) -> PyResult<()> {
        let tiles: Vec<StaticTile> = serde_json::from_str(tiles_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse tiles JSON: {}",
                e
            ))
        })?;
        let model = self.model()?;
        self.inner.merge_fetched_tiles(tiles, nodegroup_id, &model);
        Ok(())
    }

    /// Append tiles, keeping those already loaded: the incremental counterpart
    /// to `load_tiles` (which replaces). Idempotent per `tileid`. Unless
    /// `assume_tiles_comprehensive_for_nodegroup` is false, they complete the
    /// resource, so every nodegroup counts as loaded. As NAPI's and WASM's
    /// `appendTiles`.
    ///
    /// Args:
    ///     tiles_json: JSON string of tiles array
    #[pyo3(signature = (tiles_json, assume_tiles_comprehensive_for_nodegroup=true))]
    fn append_tiles(
        &mut self,
        tiles_json: &str,
        assume_tiles_comprehensive_for_nodegroup: bool,
    ) -> PyResult<()> {
        let tiles: Vec<StaticTile> = serde_json::from_str(tiles_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse tiles JSON: {}",
                e
            ))
        })?;

        let model = self.model()?;
        self.inner.merge_permitted_tiles(tiles, &model);
        if assume_tiles_comprehensive_for_nodegroup {
            self.inner.mark_loaded_comprehensively(&model);
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
        let model = self.model()?;
        let model = &*model;

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

    /// Switch lazy tile mode, as on NAPI/WASM: a nodegroup not yet loaded is
    /// then unknown, so semantic child lookups on it raise
    /// `TILES_NOT_LOADED:<nodegroup>` until its tiles are fetched
    /// (`append_fetched_tiles`). The Python wrapper's `populate(lazy=...)` sets it, and
    /// fetches what lookups list (`fetch_nodegroups`), as TypeScript does.
    fn set_lazy(&mut self, lazy: bool) {
        self.inner.set_lazy(lazy);
    }

    /// Whether lazy tile mode is on.
    fn is_lazy(&self) -> bool {
        self.inner.is_lazy()
    }

    /// The nodegroups to fetch before the children of `parent_node_id` can be
    /// read (in lazy mode; none otherwise), each once. Core's, as on NAPI and
    /// WASM.
    fn get_missing_nodegroups_for_children(&self, parent_node_id: &str) -> PyResult<Vec<String>> {
        let model = self.model()?;
        Ok(self.inner.missing_nodegroups_for_children(
            parent_node_id,
            &*model,
            self.inner.is_lazy(),
        ))
    }

    /// The nodegroups to fetch before `nodegroup_id` can be read (in lazy
    /// mode; none otherwise): itself and, for an on-load Derive target, its
    /// unloaded sources. Core's, as on NAPI and WASM.
    fn nodegroups_to_fetch(&self, nodegroup_id: &str) -> PyResult<Vec<String>> {
        let model = self.model()?;
        Ok(self.inner.nodegroups_to_fetch(nodegroup_id, &*model))
    }

    /// Raises ValueError if node `alias` can't be given a value here (unknown,
    /// semantic, derived, or a lazy nodegroup not loaded): what the ORM checks
    /// on assignment. Core's, as on NAPI and WASM.
    fn check_editable(&self, alias: &str) -> PyResult<()> {
        let model = self.model()?;
        self.inner
            .check_editable(alias, &model)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    /// Apply and commit a batch of edits as the ORM recorded them (a JSON
    /// list of `{"op": "set" | "add" | "remove", ...}`), all or nothing,
    /// returning `{"upserted", "removed"}` as JSON. Core's, as on NAPI and
    /// WASM.
    fn apply_edits(&mut self, ops_json: &str) -> PyResult<String> {
        let ops: Vec<alizarin_core::EditOp> = serde_json::from_str(ops_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("invalid edits: {e}"))
        })?;
        let model = self.model()?;
        let delta = self
            .inner
            .apply_edits(&ops, &model)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
        serde_json::to_string(&delta)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Fetch `nodegroup_ids` from the compiled-in tile source, if there is
    /// one, returning those still for the host's loader. Core's, as on NAPI
    /// and WASM.
    fn fetch_from_tile_source(&mut self, nodegroup_ids: Vec<String>) -> PyResult<Vec<String>> {
        let model = self.model()?;
        self.inner
            .fetch_from_tile_source(&nodegroup_ids, &model)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("TileSource error: {e}"))
            })
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
        let model = self.model()?;
        let model = &*model;

        let result = self
            .inner
            .get_semantic_child_value(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                &child_alias,
                model,
            )
            .map_err(sc_err)?;

        Ok(PySemanticChildResult::from_core(result))
    }

    /// Resolve a dot-separated path and return a PseudoList for the target node.
    /// Resolve a dot-separated path and return matching tile values.
    ///
    /// Walks the graph edges matching node aliases at each path segment (e.g. "building.name"),
    /// then retrieves matching tiles for the target node's nodegroup. Avoids full tree
    /// materialization.
    ///
    /// Args:
    ///     path: Dot-separated path of node aliases (e.g. "building.name")
    ///     filter_tile_id: Optional parent tile ID to filter results by parent-child relationship
    ///
    /// Returns:
    ///     RustPseudoList for the target node
    #[pyo3(signature = (path, filter_tile_id=None))]
    fn get_values_at_path(
        &self,
        path: &str,
        filter_tile_id: Option<&str>,
    ) -> PyResult<PyPseudoList> {
        let model = self.model()?;
        let model = &*model;

        let result = self
            .inner
            .get_values_at_path(path, model, filter_tile_id)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyPseudoList::from_core(result))
    }

    /// Resolve a dot-separated path to its target node metadata without needing tiles.
    ///
    /// Returns a dict with { nodegroup_id, is_single, target_node_id } — enough
    /// for the caller to lazy-load just that nodegroup's tiles.
    fn resolve_path(&self, path: &str) -> PyResult<pyo3::Py<pyo3::types::PyDict>> {
        let model = self.model()?;
        let model = &*model;

        let info = self
            .inner
            .resolve_path(path, model)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        pyo3::Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new_bound(py);
            dict.set_item("nodegroup_id", &info.nodegroup_id)?;
            dict.set_item("is_single", info.is_single)?;
            dict.set_item("target_node_id", &info.target_node.nodeid)?;
            Ok(dict.into())
        })
    }

    /// Drop everything loaded for this resource: its tiles, the pseudo cache
    /// and the nodegroup load states. Core's, as on NAPI and WASM.
    fn release(&mut self) {
        self.inner.release();
    }

    /// Load a whole StaticResource (`{resourceinstance, tiles}`) as a JSON
    /// string: its metadata and permitted tiles, comprehensive unless told
    /// otherwise. As NAPI's and WASM's `loadTilesFromResource`.
    #[pyo3(signature = (resource_json, assume_tiles_comprehensive_for_nodegroup=true))]
    fn load_tiles_from_resource(
        &mut self,
        resource_json: &str,
        assume_tiles_comprehensive_for_nodegroup: bool,
    ) -> PyResult<()> {
        let resource: alizarin_core::StaticResource =
            serde_json::from_str(resource_json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid resource JSON: {e}"
                ))
            })?;
        let model = self.model()?;
        self.inner.resource_instance = Some(resource.resourceinstance.clone());
        if let Some(tiles) = resource.tiles {
            self.inner.load_permitted_tiles(tiles, &model);
            if assume_tiles_comprehensive_for_nodegroup {
                self.inner.mark_loaded_comprehensively(&model);
            }
        }
        Ok(())
    }

    /// The stored tiles as they now stand (committed edits included, on-load
    /// derived tiles left out), as a JSON string. As NAPI's and WASM's
    /// `exportTilesJson`.
    fn export_tiles_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.stored_tiles())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Drop the tiles (and their index entries) the model no longer permits.
    /// Core's, as NAPI's and WASM's `pruneResourceTiles`.
    fn prune_resource_tiles(&mut self) -> PyResult<()> {
        let model = self.model()?;
        self.inner
            .prune_to_permitted(&model)
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)
    }

    /// Whether tiles have been loaded. Core's, as on NAPI and WASM.
    fn tiles_loaded(&self) -> bool {
        self.inner.tiles_loaded()
    }

    /// The tile `tile_id`, as a dict; KeyError if not loaded.
    fn get_tile(&self, py: Python, tile_id: &str) -> PyResult<PyObject> {
        let tile = self.inner.get_tile(tile_id).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!("Tile '{}' not found", tile_id))
        })?;
        pythonize::pythonize(py, tile)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// The resource instance ID, if metadata is loaded.
    fn get_resource_id(&self) -> Option<String> {
        self.inner
            .resource_instance
            .as_ref()
            .map(|ri| ri.resourceinstanceid.clone())
    }

    /// The resource's name: its metadata's, else its descriptors', else
    /// `<Unnamed>`. Core's, as on NAPI and WASM.
    fn get_name(&self) -> String {
        self.inner.display_name()
    }

    /// The cached descriptors, built from the tiles (and cached) if
    /// `recompute` or none are cached. Core's, as on NAPI and WASM.
    #[pyo3(signature = (recompute=false))]
    fn get_descriptors(&mut self, py: Python, recompute: bool) -> PyResult<PyObject> {
        let graph_id = self.graph_id.clone();
        let descriptors = self
            .inner
            .descriptors(recompute, || {
                let graph = model_for(&graph_id)
                    .map_err(|e| e.to_string())?
                    .get_graph_arc();
                Ok((graph, crate::build_extension_registry()))
            })
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
        pythonize::pythonize(py, &descriptors)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Every semantic child of a parent node, by alias. Errors propagate: in
    /// lazy mode, `TILES_NOT_LOADED:<nodegroups>`. As NAPI's and WASM's
    /// `getAllSemanticChildValues`.
    #[pyo3(signature = (parent_tile_id, parent_node_id, parent_nodegroup_id))]
    fn get_all_semantic_child_values(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
    ) -> PyResult<HashMap<String, PySemanticChildResult>> {
        let model = self.model()?;
        let values = self
            .inner
            .get_all_semantic_child_values(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                &*model,
            )
            .map_err(sc_err)?;
        Ok(values
            .into_iter()
            .map(|(alias, value)| (alias, PySemanticChildResult::from_core(value)))
            .collect())
    }

    /// The root pseudo value from the cache, if populated.
    fn get_root_pseudo(&self) -> Option<PyPseudoValue> {
        let model = self.model().ok()?;
        let root =
            alizarin_core::instance_wrapper_core::ModelAccess::get_root_node(&*model).ok()?;
        let cached = self.inner.get_cached_pseudo(root.alias.as_deref()?)?;
        cached
            .values
            .first()
            .map(|v| PyPseudoValue::from_core(v.clone()))
    }

    /// Construct the pseudo for `alias`: a list, a value, or empty when not
    /// permitted. Core's, as NAPI's and WASM's `makePseudoValue`.
    #[pyo3(signature = (alias, tile_id, is_permitted, is_single))]
    fn make_pseudo_value(
        &self,
        alias: &str,
        tile_id: Option<&str>,
        is_permitted: bool,
        is_single: bool,
    ) -> PyResult<PySemanticChildResult> {
        let model = self.model()?;
        let result = self
            .inner
            .make_pseudo_value(alias, tile_id, is_permitted, is_single, &*model)
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
        Ok(PySemanticChildResult::from_core(result))
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
        self.serialize_cards_with(
            py,
            Some(&card_id),
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            max_depth,
            None,
        )
    }

    /// Serialize all root cards from the populated pseudo_cache.
    ///
    /// Args:
    ///     max_depth: Max recursion depth (None = unlimited, 0 = root cards only)
    #[pyo3(signature = (max_depth=None))]
    fn serialize_root_cards(&self, py: Python, max_depth: Option<usize>) -> PyResult<PyObject> {
        self.serialize_cards_with(py, None, None, None, max_depth, None)
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
    #[pyo3(signature = (card_id, node_config_manager=None, parent_tile_id=None, parent_nodegroup_id=None, max_depth=None, language=None, resource_registry=None))]
    fn serialize_card_display(
        &self,
        py: Python,
        card_id: String,
        node_config_manager: Option<&PyNodeConfigManager>,
        parent_tile_id: Option<String>,
        parent_nodegroup_id: Option<String>,
        max_depth: Option<usize>,
        language: Option<String>,
        resource_registry: Option<&crate::ResourceRegistry>,
    ) -> PyResult<PyObject> {
        self.serialize_cards_with(
            py,
            Some(&card_id),
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            max_depth,
            Some(CardDisplay {
                ncm: node_config_manager.map(|m| m.inner()),
                registry: resource_registry.map(|r| r.inner()),
                language: language.unwrap_or_else(|| "en".to_string()),
            }),
        )
    }

    /// Serialize all root cards in display mode.
    ///
    /// Args:
    ///     node_config_manager: PyNodeConfigManager for domain value / boolean label lookups
    ///     max_depth: Max recursion depth (None = unlimited, 0 = root cards only)
    ///     language: Language code for display labels (defaults to "en")
    #[pyo3(signature = (node_config_manager=None, max_depth=None, language=None, resource_registry=None))]
    fn serialize_root_cards_display(
        &self,
        py: Python,
        node_config_manager: Option<&PyNodeConfigManager>,
        max_depth: Option<usize>,
        language: Option<String>,
        resource_registry: Option<&crate::ResourceRegistry>,
    ) -> PyResult<PyObject> {
        self.serialize_cards_with(
            py,
            None,
            None,
            None,
            max_depth,
            Some(CardDisplay {
                ncm: node_config_manager.map(|m| m.inner()),
                registry: resource_registry.map(|r| r.inner()),
                language: language.unwrap_or_else(|| "en".to_string()),
            }),
        )
    }

    /// The resource as a node-hierarchy tree in display mode: UUIDs resolved
    /// to labels for concepts, domain values, booleans, etc. Populates from
    /// the loaded tiles first if needed. As NAPI's and WASM's `toDisplayJson`.
    ///
    /// Args:
    ///     node_config_manager: Optional PyNodeConfigManager for domain/boolean lookups
    ///     language: Language code for display labels (defaults to "en")
    #[pyo3(signature = (node_config_manager=None, language=None, resource_registry=None))]
    fn to_display_json(
        &self,
        py: Python,
        node_config_manager: Option<&crate::node_config_py::PyNodeConfigManager>,
        language: Option<String>,
        resource_registry: Option<&crate::ResourceRegistry>,
    ) -> PyResult<PyObject> {
        let lang = language.unwrap_or_else(|| "en".to_string());
        self.serialize(
            py,
            alizarin_core::SerializationOptions::display(&lang),
            node_config_manager.map(|m| m.inner()),
            resource_registry.map(|r| r.inner()),
            true,
        )
    }

    /// The resource as a node-hierarchy tree of raw tile values. Populates
    /// from the loaded tiles first if needed. As NAPI's and WASM's `toJson`.
    fn to_json(&self, py: Python) -> PyResult<PyObject> {
        self.serialize(
            py,
            alizarin_core::SerializationOptions::tile_data(),
            None,
            None,
            false,
        )
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
        let model = self.model()?;
        ResourceInstanceWrapperCore::nodegroup_ids_for_card(&card_id, max_depth, &model)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    fn __repr__(&self) -> String {
        format!(
            "RustResourceInstanceWrapperCore(graph_id={}, tiles={}, cached={})",
            self.graph_id,
            self.inner.tiles.as_ref().map_or(0, |t| t.len()),
            self.inner.pseudo_cache.read().map_or(0, |c| c.len())
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
        alizarin_core::sorted_serde::sorted_keys(self.values.keys())
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
pub fn register_module(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<PyResourceInstanceWrapperCore>()?;
    m.add_class::<PyPopulateResult>()?;
    m.add_class::<PySemanticChildResult>()?;
    Ok(())
}
