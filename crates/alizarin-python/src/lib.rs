/// Python bindings for Alizarin
///
/// This provides Python bindings for:
/// - Tree conversion (tiles ↔ JSON)
/// - Resource instance wrapper with three-layer caching
/// - Resource model wrapper with pruning
/// - Graph management utilities
/// - Extension type handler registration
/// - Node configuration management for type coercion
use pyo3::prelude::*;

mod graph_mutator_py;
mod instance_wrapper_py;
mod node_config_py;
mod pseudo_value_py;
mod python_json;
mod rdm_cache_py;
mod skos_py;
mod type_coercion_py;
use pyo3::types::PyCapsule;
use std::collections::HashMap;
use std::sync::Arc;

use alizarin_core::rdm_cache::RdmCache as CoreRdmCache;

use alizarin_extension_api::TypeHandlerInfo;

// Shared extension-handler machinery lives in alizarin-core (extension_abi);
// Python is a thin PyCapsule front-end over it.
use alizarin_core::ExtensionTypeRegistry;

/// Build a fresh extension registry from the C-ABI handlers registered in core.
fn build_extension_registry() -> ExtensionTypeRegistry {
    alizarin_core::build_extension_registry_from_registered()
}

/// Get the list of registered extension handler datatypes.
#[pyfunction]
fn get_registered_extension_handlers() -> Vec<String> {
    alizarin_core::registered_handler_types()
}

/// Register a type handler from a PyCapsule wrapping a `*const TypeHandlerInfo`.
///
/// Delegates to core's shared C-ABI registration (ABI handshake, store, and
/// global-registry install). Called once at import time by an extension.
#[pyfunction]
#[pyo3(name = "register_type_handler")]
fn register_extension_handler(capsule: &Bound<'_, PyCapsule>) -> PyResult<()> {
    let ptr = capsule.pointer() as *const TypeHandlerInfo;
    // SAFETY: ptr comes from PyCapsule::pointer(), set by the extension when it
    // created the capsule; the TypeHandlerInfo and its buffers/fn-pointers are
    // owned by the capsule and valid for its lifetime. core verifies the ABI
    // fingerprint before reading any field or calling any function.
    let type_name = unsafe { alizarin_core::register_handler_from_ptr(ptr) }
        .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
    println!("Registered type handler for: {}", type_name);
    Ok(())
}

/// Register an extension's derive provider from a PyCapsule wrapping a
/// `*const DeriveProviderInfo`, under the provider id it carries, which is
/// returned. As NAPI's `registerDeriveProvider` (a pointer) and WASM's (JS
/// callbacks).
#[pyfunction]
fn register_derive_provider(capsule: &Bound<'_, PyCapsule>) -> PyResult<String> {
    let ptr = capsule.pointer() as *const alizarin_extension_api::DeriveProviderInfo;
    // SAFETY: ptr comes from PyCapsule::pointer(), set by the extension when
    // it created the capsule over its static DeriveProviderInfo; core checks
    // the ABI fingerprint before reading anything else.
    unsafe { alizarin_core::derive_abi::register_derive_provider_from_ptr(ptr) }
        .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)
}

/// Whether a derive provider is registered under `provider_id`. As on NAPI
/// and WASM.
#[pyfunction]
fn has_derive_provider(provider_id: &str) -> bool {
    alizarin_core::get_global_functions_registry()
        .derive(provider_id)
        .is_some()
}

/// Check if a type handler is registered
#[pyfunction]
#[pyo3(name = "has_type_handler")]
fn has_extension_handler(type_name: &str) -> bool {
    alizarin_core::has_registered_handler(type_name)
}

/// Coerce a value using a registered extension handler
///
/// Returns (tile_data_json, resolved_json) or raises an error
#[pyfunction]
fn coerce_with_extension(
    type_name: &str,
    value_json: &str,
    config_json: Option<&str>,
) -> PyResult<(String, String)> {
    let registry = build_extension_registry();

    if !registry.has(type_name) {
        return Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
            "No handler registered for type: {}",
            type_name
        )));
    }

    let value: serde_json::Value = serde_json::from_str(value_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid JSON: {}", e))
    })?;

    let config: Option<serde_json::Value> = config_json
        .filter(|s| *s != "null")
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid config JSON: {}", e))
        })?;

    let result = registry
        .coerce(type_name, &value, config.as_ref())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Handler for type '{}' does not support coercion",
                type_name
            ))
        })?;

    let tile_json = serde_json::to_string(&result.tile_data).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize: {}", e))
    })?;
    let resolved_json = serde_json::to_string(&result.display_value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize: {}", e))
    })?;

    Ok((tile_json, resolved_json))
}

/// Check if a type handler has display rendering support
#[pyfunction]
fn has_display_renderer(type_name: &str) -> bool {
    alizarin_core::build_extension_registry_from_registered()
        .get(type_name)
        .map(|h| h.capabilities().can_render_display)
        .unwrap_or(false)
}

/// Render a resolved value to a display string using a registered extension handler
///
/// Returns the display string or raises an error.
/// Raises KeyError if no handler is registered or handler doesn't support display rendering.
#[pyfunction]
fn render_display_with_extension(
    type_name: &str,
    resolved_json: &str,
    lang: &str,
) -> PyResult<String> {
    let registry = build_extension_registry();

    if !registry.has(type_name) {
        return Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
            "No handler registered for type: {}",
            type_name
        )));
    }

    let tile_data: serde_json::Value = serde_json::from_str(resolved_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid JSON: {}", e))
    })?;

    match registry.render_display(type_name, &tile_data, lang, None) {
        Ok(Some(display)) => Ok(display),
        Ok(None) => Ok(String::new()),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            e.to_string(),
        )),
    }
}

// Import core types and functions directly (no WASM dependency needed)
use alizarin_core::{
    // Semantic child matching
    matches_semantic_child as core_matches_semantic_child,
    // Label resolution
    resolve_labels as core_resolve_labels,
    // Graph model access
    GraphModelAccess,
    MergeAccumulator,
    StaticGraph as AlizarinCoreStaticGraph,
    StaticNode as AlizarinStaticNode,
    StaticResource as AlizarinStaticResource,
    // Resource registry for relationship resolution
    StaticResourceRegistry as CoreStaticResourceRegistry,
    StaticResourceSummary as CoreStaticResourceSummary,
    // Graph types
    StaticTile as AlizarinStaticTile,
};

/// Build alias-to-collection mapping directly from StaticGraph.
///
/// Includes core resolvable datatypes (concept, concept-list) plus any
/// extension-registered datatypes that have marker resolution support.
fn build_alias_to_collection_from_graph(
    graph: &AlizarinCoreStaticGraph,
    ext_registry: Option<&ExtensionTypeRegistry>,
) -> HashMap<String, String> {
    alizarin_core::alias_to_collection_for_graph(graph, ext_registry)
}

/// Helper to get a graph from the registry
fn get_registered_graph(graph_id: &str) -> PyResult<std::sync::Arc<AlizarinCoreStaticGraph>> {
    alizarin_core::get_graph(graph_id).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
            "Graph '{}' not registered. Call register_graph() first.",
            graph_id
        ))
    })
}

/// Convert tiled resource to nested JSON tree
///
/// Args:
///     resource_json: Resource as JSON string: StaticResource, or flat
///         `{resourceinstanceid, graph_id, tiles}` (see core's `resource_for_tree`).
///         Its graph must be registered.
///
/// Returns:
///     Nested dict structure representing the resource tree with resourceinstanceid and graph_id
#[pyfunction]
#[pyo3(signature = (resource_json))]
fn tiles_to_json_tree(py: Python, resource_json: String) -> PyResult<PyObject> {
    let result = alizarin_core::tiles_to_tree_from_json_registered(&resource_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert result to Python: {}",
            e
        ))
    })
}

/// Build a tree from pre-extracted tiles (no resource wrapper needed).
///
/// Thin shim around `alizarin_core::build_tree_from_tiles` — all logic
/// lives in core so NAPI/WASM consumers get the same function for free.
///
/// Args:
///     tiles_json: JSON array of tile objects
///     resource_id: Resource instance ID
///     graph_id: Must be registered via register_graph()
///
/// Returns:
///     Nested dict structure representing the resource tree
#[pyfunction]
#[pyo3(signature = (tiles_json, resource_id, graph_id))]
fn build_tree_from_tiles(
    py: Python,
    tiles_json: String,
    resource_id: String,
    graph_id: String,
) -> PyResult<PyObject> {
    let result = alizarin_core::build_tree_from_tiles(&tiles_json, &resource_id, &graph_id)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    pythonize::pythonize(py, &result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert result to Python: {}",
            e
        ))
    })
}

/// Convert tiles to a card-structured tree.
///
/// Groups data by the card/widget hierarchy (UI structure) rather than by
/// node edges (data structure). Each card contains its widgets' values and
/// nested child cards.
///
/// Args:
///     resource_json: Resource JSON string (same format as tiles_to_json_tree)
///
/// Returns:
///     Card-structured dict with widgets and nested cards
#[pyfunction]
#[pyo3(signature = (resource_json))]
fn cards_to_json_tree(py: Python, resource_json: String) -> PyResult<PyObject> {
    let result = alizarin_core::cards_to_tree_from_json_registered(&resource_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert result to Python: {}",
            e
        ))
    })
}

/// Convert nested JSON tree to tiled resource
///
/// Args:
///     tree_json: Nested JSON tree as string
///     resource_id: Resource instance ID
///     graph_id: Graph ID (must be registered via register_graph first)
///     from_camel: If True, convert keys from camelCase to snake_case before resolving
///     strict: If True, raise an error for any validation failures or unparseable data
///
/// Returns:
///     Dict with 'resourceinstanceid', 'graph_id', and 'tiles'
///
/// If a global RDM cache is set (via set_global_rdm_cache), label strings
/// will automatically be resolved to UUIDs before conversion.
///
/// If id_key is provided and the tree does not contain a resourceinstanceid,
/// a deterministic UUID v5 will be generated using the key and graph ID.
///
/// If scopes is provided (as JSON string), it will be set on all resulting resources.
///
/// If tile_namespace (a UUID) is provided, cardinality-n tile ids are derived so
/// the same tree yields the same ids; each layer must use its own. Without it
/// they stay random.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (tree_json, resource_id, graph_id, from_camel=false, strict=true, rdm_cache=None, id_key=None, scopes=None, tile_namespace=None))]
fn json_tree_to_tiles(
    py: Python,
    tree_json: String,
    resource_id: String,
    graph_id: String,
    from_camel: bool,
    strict: bool,
    rdm_cache: Option<&rdm_cache_py::RdmCache>,
    id_key: Option<String>,
    scopes: Option<String>,
    tile_namespace: Option<String>,
) -> PyResult<PyObject> {
    let scopes: Option<serde_json::Value> = scopes
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse scopes: {}",
                e
            ))
        })?;
    let graph = get_registered_graph(&graph_id)?;
    let opts = alizarin_core::TreeToTilesOptions {
        resource_id: Some(&resource_id),
        id_key: id_key.as_deref(),
        from_camel,
        strict,
        random_ids: false,
        scopes: scopes.as_ref(),
        tile_namespace: tile_namespace.as_deref(),
    };
    let ext_registry = build_extension_registry();
    let output = alizarin_core::tree_to_tiles_from_json(
        &tree_json,
        &graph,
        &opts,
        Some(&ext_registry),
        rdm_cache.map(|c| Arc::new(c.inner().clone())),
    )
    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    pythonize::pythonize(py, &output).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert result to Python: {}",
            e
        ))
    })
}

/// Batch convert multiple JSON trees to tiles in parallel.
///
/// Applies registered extension handlers (e.g. CLM "reference" type) by default.
/// Set `skip_extensions=True` to bypass extension coercion for a minor performance gain
/// when your graph has no extension-type nodes.
///
/// Args:
///     trees_json: Array of trees as JSON string
///     graph_id: Graph ID (must be registered via register_graph first)
///     from_camel: If True, convert keys from camelCase to snake_case
///     strict: If True, fail on first error (including extension coercion errors)
///     id_keys: Optional list of keys for deterministic UUID generation
///     scopes: Optional JSON string to set as scopes on all resources
///     resolve_markers: If True, resolve extension markers (e.g. label→UUID) via RDM cache
///     random_ids: If True, generate random UUIDs instead of slug-based IDs
///     skip_extensions: If True, skip extension coercion (unknown datatypes will error in strict mode)
///     tile_namespace: Optional layer namespace (a UUID) for derived cardinality-n
///         tile ids; each layer must use its own. Without it they stay random.
///
/// Returns:
///     BusinessDataWrapper format: {business_data: {resources: [...]}, errors: [...], error_count: int}
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (trees_json, graph_id, from_camel=false, strict=true, id_keys=None, scopes=None, resolve_markers=true, random_ids=false, skip_extensions=false, tile_namespace=None))]
fn batch_trees_to_tiles(
    py: Python,
    trees_json: String,
    graph_id: String,
    from_camel: bool,
    strict: bool,
    id_keys: Option<Vec<String>>,
    scopes: Option<String>,
    resolve_markers: bool,
    random_ids: bool,
    skip_extensions: bool,
    tile_namespace: Option<String>,
) -> PyResult<PyObject> {
    let scopes: Option<serde_json::Value> = scopes
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse scopes: {}",
                e
            ))
        })?;
    let graph = get_registered_graph(&graph_id)?;
    let opts = alizarin_core::BatchOptions {
        graph_id: &graph_id,
        from_camel,
        strict,
        random_ids,
        resolve_markers,
        id_keys: id_keys.as_deref(),
        scopes: scopes.as_ref(),
        tile_namespace: tile_namespace.as_deref(),
    };
    let ext_registry = (!skip_extensions).then(build_extension_registry);
    let output = alizarin_core::batch_trees_to_tiles_from_json(
        &trees_json,
        &graph,
        &opts,
        ext_registry.as_ref(),
        None,
    )
    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    pythonize::pythonize(py, &output).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

/// A static type's dict as core serialises it: the one definition of
/// alizarin's output, which WASM and NAPI give too (pinned by
/// tests/data/output-parity/expected.json). The dataclasses' `to_dict`s return
/// this, so Python can't drift from it: empty fields core leaves out are left
/// out, and keys come in core's order.
///
/// Args:
///     kind: the core type: "node", "nodegroup", "edge", "card", "constraint",
///         "cards_x_nodes_x_widgets", "functions_x_graphs", "publication",
///         "graph_meta", "tile", "resource", "resource_descriptors" or "graph".
///     value: the dict to normalise.
#[pyfunction]
fn canonical_static(py: Python, kind: &str, value: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let canonical = alizarin_core::canonical_static(kind, python_json::python_to_json(py, value)?)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &canonical)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// A static type built from a dict, as every binding's constructor builds it:
/// `canonical_static`, except that a resource's `tiles_loaded` is settled (as
/// given, else whether it has tiles).
///
/// Args:
///     kind: as for `canonical_static`.
///     value: the dict to build from.
#[pyfunction]
fn construct_static(py: Python, kind: &str, value: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let built = alizarin_core::construct_static(kind, python_json::python_to_json(py, value)?)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &built)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// A number as JavaScript's `String(n)` shows it (`7`, not `7.0`; `1e+21`):
/// core's, so number view models display alike on every backend.
#[pyfunction]
fn number_display(n: f64) -> String {
    alizarin_core::string_utils::number_display(n)
}

/// Whether two nodes (whole or partial dicts) are the same apart from their
/// ids: core's `compare_nodes`, as WASM's `StaticNode.compare` and NAPI's
/// `compareNodes`. One of "same", "same_ids_partial", "nodeid_conflict",
/// "nodegroup_id_conflict", "graph_id_conflict", "different".
#[pyfunction]
fn compare_nodes(py: Python, a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>) -> PyResult<String> {
    let a = python_json::python_to_json(py, a)?;
    let b = python_json::python_to_json(py, b)?;
    Ok(alizarin_core::compare_nodes(&a, &b).as_str().to_string())
}

/// The resources of a business data file (`{"business_data": {"resources":
/// [...]}}`), each as `construct_static("resource", ...)` builds it.
#[pyfunction]
fn resources_from_business_data_json(py: Python, json: &str) -> PyResult<PyObject> {
    let resources = alizarin_core::resources_from_business_data_json(json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &resources)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

fn summary_arg(py: Python, summary: &Bound<'_, PyAny>) -> PyResult<CoreStaticResourceSummary> {
    serde_json::from_value(python_json::python_to_json(py, summary)?).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid summary: {e}"))
    })
}

/// A registry summary's resource metadata, as core builds it.
///
/// Args:
///     summary: a summary dict, as `ResourceRegistry.get_summary` gives.
#[pyfunction]
fn summary_to_metadata(py: Python, summary: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    pythonize::pythonize(py, &summary_arg(py, summary)?.to_metadata())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// A lightweight resource (no tiles, `tiles_loaded: False`) from a registry
/// summary, as core builds it.
///
/// Args:
///     summary: a summary dict, as `ResourceRegistry.get_summary` gives.
#[pyfunction]
fn static_resource_from_summary(py: Python, summary: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let resource = alizarin_core::StaticResource::from_summary(&summary_arg(py, summary)?);
    pythonize::pythonize(py, &resource)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Compose one resource across an ordered layer stack, in memory — the binding
/// form of the substrate's `hydrate_layers` (no DuckDB).
///
/// Args:
///     resources_json: JSON array of the same resource as it exists in each
///         layer, TOPMOST-FIRST (highest-priority layer first). Tiles are merged
///         (identical tiles deduped, topmost wins), then cardinality-1 nodegroups
///         unified PerNodegroup (topmost overrides a single-valued group whole;
///         multi-valued groups accumulate).
///     base_graph_json: The base model StaticGraph JSON.
///     overlay_graphs_json: JSON array of overlay StaticGraphs bottom-to-top (may
///         be "[]"); composition runs against the merged model via LayeredGraph.
///     strict: Make a cross-layer conflict on a single-valued group an error.
///
/// Returns:
///     {resource: ..., warnings: [...]}
#[pyfunction]
#[pyo3(signature = (resources_json, base_graph_json, overlay_graphs_json, strict=false))]
fn compose_resource_layers(
    py: Python,
    resources_json: String,
    base_graph_json: String,
    overlay_graphs_json: String,
    strict: bool,
) -> PyResult<PyObject> {
    let composed = alizarin_core::graph::compose_resource_layers_from_json(
        &resources_json,
        &base_graph_json,
        &overlay_graphs_json,
        strict,
    )
    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    pythonize::pythonize(py, &composed).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to convert result: {}", e))
    })
}

/// A resource's lifecycle state resolved against its graph's lifecycle.
///
/// Args:
///     resource_json: A StaticResource as JSON.
///     lifecycle_json: The graph's resource_instance_lifecycle as JSON ("null" for none).
///
/// Returns:
///     {status: "state" | "unknown" | "conflict", state_id, is_initial, conflict}
#[pyfunction]
fn resource_lifecycle_state(
    py: Python,
    resource_json: String,
    lifecycle_json: String,
) -> PyResult<PyObject> {
    let report = alizarin_core::graph::lifecycle_state_from_json(&resource_json, &lifecycle_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &report).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to convert result: {}", e))
    })
}

/// Move a resource to lifecycle state `target`.
///
/// Args:
///     resource_json: A StaticResource as JSON.
///     lifecycle_json: The graph's resource_instance_lifecycle as JSON ("null" for none).
///     target: The state id to move to.
///     strict: Require a permitted edge (next or previous) and refuse to leave a
///         conflict or unknown state. Pass False explicitly for Arches'
///         membership-only check.
///
/// Returns:
///     {resource: <tiles-free overlay record>, event: {resourceinstanceid, from, to, ...}}
#[pyfunction]
#[pyo3(signature = (resource_json, lifecycle_json, target, strict=true))]
fn transition_resource_lifecycle(
    py: Python,
    resource_json: String,
    lifecycle_json: String,
    target: String,
    strict: bool,
) -> PyResult<PyObject> {
    let result = alizarin_core::graph::transition_from_json(
        &resource_json,
        &lifecycle_json,
        &target,
        strict,
    )
    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to convert result: {}", e))
    })
}

// =============================================================================
// ResourceRegistry - For relationship resolution and cache population
// =============================================================================

/// Registry of known resources for relationship resolution
///
/// Used to:
/// - Look up graph_id for referenced resources
/// - Populate __cache on resources with related resource summaries
/// - Enrich resource-instance tile data with ontologyProperty from node config
/// - Validate that referenced resources exist (in strict mode)
///
/// Usage:
///     registry = ResourceRegistry()
///     registry.merge_from_resources(resources_json)  # Load known resources
///     result = batch_trees_to_tiles(..., registry=registry)
#[pyclass]
pub struct ResourceRegistry {
    inner: CoreStaticResourceRegistry,
}

impl ResourceRegistry {
    /// Get the inner core registry (for internal Rust use)
    pub(crate) fn inner(&self) -> &CoreStaticResourceRegistry {
        &self.inner
    }
}

#[pymethods]
impl ResourceRegistry {
    #[new]
    fn new() -> Self {
        ResourceRegistry {
            inner: CoreStaticResourceRegistry::new(),
        }
    }

    /// Number of resources in the registry
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Check if a resource ID is known
    fn __contains__(&self, resource_id: &str) -> bool {
        self.inner.contains(resource_id)
    }

    /// Check if registry is empty
    #[getter]
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the graph_id for a resource (or None if not known)
    fn get_graph_id(&self, resource_id: &str) -> Option<String> {
        self.inner.get_graph_id(resource_id).map(|s| s.to_string())
    }

    /// Get the summary for a resource (or None if not known)
    ///
    /// Returns a JSON string with {resourceinstanceid, graph_id, name, ...}
    fn get_summary(&self, py: Python, resource_id: &str) -> PyResult<Option<PyObject>> {
        match self.inner.get_summary(resource_id) {
            Some(summary) => {
                let json = serde_json::to_value(&summary).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize summary: {}",
                        e
                    ))
                })?;
                Ok(Some(pythonize::pythonize(py, &json)?))
            }
            None => Ok(None),
        }
    }

    /// Get the full resource if stored (or None if only summary or not known)
    ///
    /// Returns a JSON dict with full resource data including tiles
    fn get_full(&self, py: Python, resource_id: &str) -> PyResult<Option<PyObject>> {
        match self.inner.get_full(resource_id) {
            Some(resource) => {
                let json = serde_json::to_value(resource).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize resource: {}",
                        e
                    ))
                })?;
                Ok(Some(pythonize::pythonize(py, &json)?))
            }
            None => Ok(None),
        }
    }

    /// Check if a resource has full data stored (not just summary)
    fn has_full(&self, resource_id: &str) -> bool {
        self.inner.has_full(resource_id)
    }

    /// Get all full resources as a list of JSON dicts
    fn get_all_full(&self, py: Python) -> PyResult<PyObject> {
        let resources: Vec<&AlizarinStaticResource> =
            self.inner.iter_full().map(|(_, r)| r).collect();
        let json = serde_json::to_value(&resources).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize resources: {}",
                e
            ))
        })?;
        pythonize::pythonize(py, &json).map_err(|e| e.into())
    }

    /// Run a resource's Aggregate functions, with this registry as the slice.
    ///
    /// The resource must be loaded in full here, and its graph and its members'
    /// graphs registered (`register_graph`). Its Aggregate functions (e.g. the
    /// built-in fold) write node values into its tiles, which are updated in
    /// place; the resource itself is not part of the slice.
    ///
    /// Args:
    ///     target_id: The resource whose Aggregate functions to run (e.g. a Layer resource)
    ///
    /// Returns:
    ///     The node values that changed, as `[{"tileid", "nodeid", "value"}]`
    ///     (empty if nothing did), for a consumer caching them
    fn apply_aggregate_functions(&mut self, py: Python, target_id: &str) -> PyResult<PyObject> {
        let changed = alizarin_core::apply_aggregate_functions_in_registry(
            &mut self.inner,
            target_id,
            &build_extension_registry(),
        )
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
        Ok(pythonize::pythonize(py, &changed)?)
    }

    /// Expand a resource's Aggregate node values with members added since the
    /// last call, without revisiting earlier ones.
    ///
    /// Call it when the node values are about to be consumed: additions compose,
    /// so the ids added since the last call go in one batch. Add each member
    /// once; re-adding one counts it again. Raises if a provider can't add
    /// incrementally (run `apply_aggregate_functions` then).
    ///
    /// Args:
    ///     target_id: The resource whose Aggregate node values to expand
    ///     added_ids: Members added since the last call, already loaded in full here
    ///
    /// Returns:
    ///     The node values that changed, as `apply_aggregate_functions` does
    fn apply_aggregate_additions(
        &mut self,
        py: Python,
        target_id: &str,
        added_ids: Vec<String>,
    ) -> PyResult<PyObject> {
        let changed = alizarin_core::apply_aggregate_additions_in_registry(
            &mut self.inner,
            target_id,
            &added_ids,
            &build_extension_registry(),
        )
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
        Ok(pythonize::pythonize(py, &changed)?)
    }

    /// Add `count` to the pending count each of a resource's Aggregate functions
    /// declares (`pending` in its entry): members added but not yet applied.
    ///
    /// A consumer can then tell the node values are stale, and by how much.
    /// Runs and additions reset it to 0.
    ///
    /// Args:
    ///     target_id: The resource (e.g. a Layer resource) whose count to add to
    ///     count: How many members were added
    ///
    /// Returns:
    ///     The node values that changed, as `apply_aggregate_functions` does
    fn mark_aggregate_pending(
        &mut self,
        py: Python,
        target_id: &str,
        count: u64,
    ) -> PyResult<PyObject> {
        let changed =
            alizarin_core::mark_aggregate_pending_in_registry(&mut self.inner, target_id, count)
                .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
        Ok(pythonize::pythonize(py, &changed)?)
    }

    /// Get all full resources for a specific graph
    fn get_all_full_for_graph(&self, py: Python, graph_id: &str) -> PyResult<PyObject> {
        let resources: Vec<&AlizarinStaticResource> = self
            .inner
            .iter_full()
            .filter(|(_, r)| r.resourceinstance.graph_id == graph_id)
            .map(|(_, r)| r)
            .collect();
        let json = serde_json::to_value(&resources).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize resources: {}",
                e
            ))
        })?;
        pythonize::pythonize(py, &json).map_err(|e| e.into())
    }

    /// Add a single resource summary to the registry
    ///
    /// Args:
    ///     summary_json: JSON string with {resourceinstanceid, graph_id, name, ...}
    fn insert(&mut self, summary_json: &str) -> PyResult<()> {
        let summary: CoreStaticResourceSummary =
            serde_json::from_str(summary_json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse summary: {}",
                    e
                ))
            })?;
        self.inner.insert(summary);
        Ok(())
    }

    /// Merge resources into the registry
    ///
    /// Registers each resource's ID → summary mapping or full resources.
    /// If store_full is True, stores full resources (for traversal).
    /// If store_full is False (default), stores only summaries (memory efficient).
    /// If include_caches is True (default), also merges any __cache.relatedResources as summaries.
    ///
    /// Args:
    ///     resources_json: JSON string containing array of StaticResource objects
    ///     store_full: If True, store full resources; if False, store only summaries
    ///     include_caches: If True, also merge related resources from __cache
    #[pyo3(signature = (resources_json, store_full=false, include_caches=true))]
    fn merge_from_resources(
        &mut self,
        resources_json: &str,
        store_full: bool,
        include_caches: bool,
    ) -> PyResult<()> {
        let resources: Vec<AlizarinStaticResource> =
            serde_json::from_str(resources_json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse resources: {}",
                    e
                ))
            })?;
        self.inner
            .merge_from_resources(&resources, store_full, include_caches);
        Ok(())
    }

    /// Load a business data file from raw bytes, merge its resources, and
    /// return a ref for each, in file order.
    ///
    /// Args:
    ///     data: The file's bytes (a `{"business_data": {"resources": [...]}}`
    ///         wrapper, or one bare resource)
    ///     store_full: If True (default), store full resources; else summaries
    ///     include_caches: If True (default), also merge related resources from __cache
    ///
    /// Returns:
    ///     List of `{"resourceinstanceid", "graph_id", "isPublic"}` dicts, where
    ///     `isPublic` is whether the resource's `__scopes` include "public"
    /// Load every business data file under a prebuild's `business_data/`
    /// (recursively, in path order), as `load_from_business_data_bytes`
    /// loads one, and return all their refs in that order.
    #[pyo3(signature = (prebuild_dir, store_full=true, include_caches=true))]
    fn load_prebuild_business_data(
        &mut self,
        py: Python,
        prebuild_dir: &str,
        store_full: bool,
        include_caches: bool,
    ) -> PyResult<PyObject> {
        let refs = self
            .inner
            .load_prebuild_business_data(prebuild_dir, store_full, include_caches)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        pythonize::pythonize(py, &refs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    #[pyo3(signature = (data, store_full=true, include_caches=true))]
    fn load_from_business_data_bytes(
        &mut self,
        py: Python,
        data: &[u8],
        store_full: bool,
        include_caches: bool,
    ) -> PyResult<PyObject> {
        let refs = self
            .inner
            .load_from_business_data_bytes(data, store_full, include_caches)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse business data: {}",
                    e
                ))
            })?;
        pythonize::pythonize(py, &refs).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert result to Python: {}",
                e
            ))
        })
    }

    /// Populate __cache on resources with summaries for referenced resources
    ///
    /// Uses the graph to identify resource-instance nodes, then populates
    /// cache entries for each referenced resource found in this registry.
    ///
    /// If enrich_relationships is True, also adds ontologyProperty/inverseOntologyProperty
    /// to tile data based on node config and the target resource's graph.
    ///
    /// Args:
    ///     resources_json: JSON string containing array of StaticResource objects
    ///     graph_id: ID of the graph to use for node lookup
    ///     enrich_relationships: If True, add ontologyProperty to tile data
    ///
    /// Returns:
    ///     Dict with, as on WASM and NAPI:
    ///     - resources: Updated resources with __cache populated
    ///     - unknownReferences: List of {source_resource_id, node_id, node_alias, referenced_id}
    ///     - hasUnknown: whether there are any
    #[pyo3(signature = (resources_json, graph_id, enrich_relationships=true, strict=false, recompute_descriptors=false))]
    fn populate_caches(
        &self,
        py: Python,
        resources_json: &str,
        graph_id: &str,
        enrich_relationships: bool,
        strict: bool,
        recompute_descriptors: bool,
    ) -> PyResult<PyObject> {
        let graph = get_registered_graph(graph_id)?;
        let output = self
            .inner
            .populate_caches_from_json(
                resources_json,
                &graph,
                enrich_relationships,
                strict,
                recompute_descriptors,
            )
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        pythonize::pythonize(py, &output).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Extract node values from all resources, returning a map from resource ID to values.
    ///
    /// Efficiently iterates through tiles in Rust, filtering by nodegroup and extracting
    /// values for the specified node.
    ///
    /// Args:
    ///     graph_id: ID of the graph (must be registered via register_graph)
    ///     node_identifier: Node alias or node ID to extract values for
    ///
    /// Returns:
    ///     Dict mapping resource_id -> list of values found for that node.
    ///     Values are returned as-is (may be strings, dicts for localized strings, etc.)
    ///
    /// Example:
    ///     >>> index = registry.get_node_values_index(graph_id, "resource_id")
    ///     >>> # Returns: {"uuid-1": ["REF-001"], "uuid-2": ["REF-002", "REF-003"]}
    fn get_node_values_index(
        &self,
        py: Python,
        graph_id: &str,
        node_identifier: &str,
    ) -> PyResult<PyObject> {
        let graph = get_registered_graph(graph_id)?;

        let index = self
            .inner
            .get_node_values_index(&graph, node_identifier)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        pythonize::pythonize(py, &index).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Extract node values and return an inverted index: value -> list of resource IDs.
    ///
    /// Useful for looking up resources by a field value (e.g., find resource by external ID).
    ///
    /// Args:
    ///     graph_id: ID of the graph (must be registered via register_graph)
    ///     node_identifier: Node alias or node ID to extract values for
    ///     flatten_localized: If True, extract string from localized values {"en": "value"}
    ///
    /// Returns:
    ///     Dict mapping value -> list of resource_ids that have that value.
    ///
    /// Example:
    ///     >>> index = registry.get_value_to_resources_index(graph_id, "resource_id", True)
    ///     >>> # Returns: {"REF-001": ["uuid-1"], "REF-002": ["uuid-2", "uuid-3"]}
    #[pyo3(signature = (graph_id, node_identifier, flatten_localized=true))]
    fn get_value_to_resources_index(
        &self,
        py: Python,
        graph_id: &str,
        node_identifier: &str,
        flatten_localized: bool,
    ) -> PyResult<PyObject> {
        let graph = get_registered_graph(graph_id)?;

        let index = self
            .inner
            .get_value_to_resources_index(&graph, node_identifier, flatten_localized)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        pythonize::pythonize(py, &index).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }
}

/// Iterator that processes trees to tiles one at a time (low memory)
///
/// Usage:
///     iter = TreeToTilesIterator(tree_strings, graph_id, from_camel=True, scopes='{"read": true}')
///     for result in iter:
///         # result is {"resource": {...}, "error": None} or {"resource": None, "error": "..."}
///         if result["resource"]:
///             process(result["resource"])
#[pyclass]
struct TreeToTilesIterator {
    trees: Vec<String>,
    graph: std::sync::Arc<AlizarinCoreStaticGraph>,
    graph_id: String,
    from_camel: bool,
    strict: bool,
    id_keys: Option<Vec<String>>,
    scopes: Option<serde_json::Value>,
    random_ids: bool,
    tile_namespace: Option<String>,
    index: usize,
    ext_registry: Option<ExtensionTypeRegistry>,
}

#[pymethods]
impl TreeToTilesIterator {
    #[new]
    #[pyo3(signature = (tree_strings, graph_id, from_camel=false, strict=true, id_keys=None, scopes=None, random_ids=false, tile_namespace=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        tree_strings: Vec<String>,
        graph_id: String,
        from_camel: bool,
        strict: bool,
        id_keys: Option<Vec<String>>,
        scopes: Option<String>,
        random_ids: bool,
        tile_namespace: Option<String>,
    ) -> PyResult<Self> {
        // Get graph from registry
        let graph = get_registered_graph(&graph_id)?;

        // Parse scopes if provided
        let scopes_value: Option<serde_json::Value> = scopes
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse scopes: {}",
                    e
                ))
            })?;

        // Validate id_keys length if provided
        if let Some(ref keys) = id_keys {
            if keys.len() != tree_strings.len() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "id_keys length ({}) must match trees length ({})",
                    keys.len(),
                    tree_strings.len()
                )));
            }
        }

        let ext_registry = build_extension_registry();
        let ext_registry = if ext_registry.is_empty() {
            None
        } else {
            Some(ext_registry)
        };

        Ok(TreeToTilesIterator {
            trees: tree_strings,
            graph,
            graph_id,
            from_camel,
            strict,
            id_keys,
            scopes: scopes_value,
            random_ids,
            tile_namespace,
            index: 0,
            ext_registry,
        })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        if self.index >= self.trees.len() {
            return Ok(None);
        }

        let tree_str = &self.trees[self.index];
        let id_key = self.id_keys.as_ref().map(|keys| keys[self.index].as_str());
        let i = self.index;
        self.index += 1;

        let tree: serde_json::Value = match serde_json::from_str(tree_str) {
            Ok(t) => t,
            Err(e) => {
                let output = serde_json::json!({
                    "resource": null,
                    "error": format!("Tree {}: Failed to parse: {}", i, e),
                    "index": i
                });
                return pythonize::pythonize(py, &output)
                    .map(Some)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()));
            }
        };

        let output = match alizarin_core::convert_single_tree::<CoreRdmCache>(
            &tree,
            &self.graph,
            &self.graph_id,
            id_key,
            self.from_camel,
            self.strict,
            self.random_ids,
            self.ext_registry.as_ref(),
            None,
            true,
            self.tile_namespace.as_deref(),
        ) {
            Ok(mut resource) => {
                if let Some(ref scopes_val) = self.scopes {
                    resource.scopes = Some(scopes_val.clone());
                }
                serde_json::json!({
                    "resource": resource,
                    "error": null,
                    "index": i
                })
            }
            Err(e) => {
                serde_json::json!({
                    "resource": null,
                    "error": format!("Tree {}: {}", i, e),
                    "index": i
                })
            }
        };

        pythonize::pythonize(py, &output)
            .map(Some)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Get remaining count
    fn __len__(&self) -> usize {
        self.trees.len() - self.index
    }

    /// Total number of trees
    #[getter]
    fn total(&self) -> usize {
        self.trees.len()
    }

    /// Current index
    #[getter]
    fn current_index(&self) -> usize {
        self.index
    }
}

/// Batch convert multiple tiled resources to JSON trees in parallel
///
/// Resources must contain graph_id in resourceinstance.graph_id or graph_id field.
/// Graphs must be registered via register_graph first.
#[pyfunction]
#[pyo3(signature = (resources_json, strict=true))]
fn batch_tiles_to_trees(py: Python, resources_json: String, strict: bool) -> PyResult<PyObject> {
    let output = alizarin_core::batch_tiles_to_trees_from_json(&resources_json, None, strict)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &output).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

// camel_to_snake and transform_keys_to_snake are now in alizarin-core/src/string_utils.rs

/// A graph node for navigating a model's structure (`NodeViewModel`): core's
/// `PseudoNodeCore`, as WASM's `PseudoNode` and NAPI's `NapiPseudoNode` are.
#[pyclass(name = "PseudoNode")]
struct PyPseudoNode {
    core: alizarin_core::PseudoNodeCore,
}

#[pymethods]
impl PyPseudoNode {
    fn is_iterable(&self) -> bool {
        self.core.is_iterable()
    }

    /// The path from where navigation started (`.group.basic_info`), with
    /// `[*]` after a list node.
    fn get_node_placeholder(&self) -> String {
        self.core.placeholder()
    }

    /// The node, as a dict.
    #[getter]
    fn node(&self, py: Python) -> PyResult<PyObject> {
        pythonize::pythonize(py, &*self.core.node).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to convert node: {e}"))
        })
    }

    #[getter]
    fn size(&self) -> usize {
        self.core.child_nodes.len()
    }

    #[getter]
    fn child_node_aliases(&self) -> Vec<String> {
        self.core.child_nodes.keys().cloned().collect()
    }

    /// Children by alias, each as a dict.
    #[getter]
    fn child_nodes(&self, py: Python) -> PyResult<PyObject> {
        let children: std::collections::BTreeMap<&str, &alizarin_core::StaticNode> = self
            .core
            .child_nodes
            .iter()
            .map(|(alias, node)| (alias.as_str(), &**node))
            .collect();
        pythonize::pythonize(py, &children).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert child nodes: {e}"
            ))
        })
    }

    #[getter]
    fn nodeid(&self) -> String {
        self.core.node.nodeid.clone()
    }

    #[getter]
    fn alias(&self) -> String {
        self.core.node.alias.clone().unwrap_or_default()
    }

    #[getter]
    fn name(&self) -> String {
        self.core.node.name.clone()
    }

    #[getter]
    fn nodegroup_id(&self) -> Option<String> {
        self.core.node.nodegroup_id.clone()
    }

    /// The datatype, read as `semantic` on an inner node.
    #[getter]
    fn datatype(&self) -> String {
        self.core.datatype().to_string()
    }

    #[getter]
    fn is_collector(&self) -> bool {
        self.core.node.is_collector
    }

    #[getter]
    fn is_outer(&self) -> bool {
        self.core.is_outer()
    }

    #[getter]
    fn inner(&self) -> PyResult<PyPseudoNode> {
        self.core
            .inner
            .as_ref()
            .map(|inner| PyPseudoNode {
                core: (**inner).clone(),
            })
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No inner node"))
    }

    #[getter]
    fn is_inner(&self) -> bool {
        self.core.is_inner
    }

    fn to_json(&self, py: Python) -> PyResult<PyObject> {
        pythonize::pythonize(py, &self.core.node.to_json()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to convert node: {e}"))
        })
    }
}

/// Python wrapper for ResourceModelWrapperCore
///
/// Provides graph pruning and permission checking.
/// Graph must be registered via register_graph first.
///
/// A handle on its graph's model in core's registry, which instance wrappers
/// share: it reads that model and changes it only through core.
#[pyclass]
struct PyResourceModelWrapper {
    graph_id: String,
}

impl PyResourceModelWrapper {
    /// The graph's shared model, as it is now.
    fn model(&self) -> PyResult<Arc<GraphModelAccess>> {
        instance_wrapper_py::model_for(&self.graph_id)
    }
}

#[pymethods]
impl PyResourceModelWrapper {
    #[new]
    fn new(graph_id: String, default_allow: Option<bool>) -> PyResult<Self> {
        // Get graph from registry
        let graph = get_registered_graph(&graph_id)?;
        let default_allow = default_allow.unwrap_or(true);

        // The graph's shared model, which every instance wrapper uses.
        alizarin_core::register_model(graph, default_allow)
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;

        Ok(PyResourceModelWrapper { graph_id })
    }

    /// Get child nodes for a given node ID, keyed by alias.
    /// Returns a dict of {alias: node_json}.
    fn get_child_nodes(&self, py: Python, node_id: &str) -> PyResult<PyObject> {
        let model = self.model()?;
        let children = model.get_child_nodes(node_id);
        let dict = pyo3::types::PyDict::new_bound(py);
        for (alias, node) in &children {
            let node_json = serde_json::to_string(node.as_ref()).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize node: {}",
                    e
                ))
            })?;
            dict.set_item(alias, node_json)?;
        }
        Ok(dict.to_object(py))
    }

    /// A pseudo node to navigate from: the root, or the node with `alias`.
    /// Core's, as WASM and NAPI use.
    #[pyo3(signature = (alias=None))]
    fn create_pseudo_node(&self, alias: Option<&str>) -> PyResult<PyPseudoNode> {
        let model = self.model()?;
        match alias {
            Some(alias) => alizarin_core::PseudoNodeCore::by_alias(alias, &*model),
            None => alizarin_core::PseudoNodeCore::root(&*model),
        }
        .map(|core| PyPseudoNode { core })
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    /// `parent`'s child `child`, which records `parent` as its parent.
    fn create_pseudo_node_child(
        &self,
        child: &str,
        parent: PyRef<'_, PyPseudoNode>,
    ) -> PyResult<PyPseudoNode> {
        let model = self.model()?;
        parent
            .core
            .child(child, &*model)
            .map(|core| PyPseudoNode { core })
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    /// Get all nodes as a dict of {node_id: node_json}. As WASM's and NAPI's
    /// `getNodeObjects`.
    fn get_node_objects(&self, py: Python) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        let model = self.model()?;
        if let Some(nodes) = alizarin_core::ModelAccess::get_nodes(&*model) {
            for (id, node) in nodes {
                let node_json = serde_json::to_string(node.as_ref()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize node: {}",
                        e
                    ))
                })?;
                dict.set_item(id, node_json)?;
            }
        }
        Ok(dict.to_object(py))
    }

    /// Get all edges as a dict of {node_id: [child_node_id, ...]}.
    fn get_edges(&self, py: Python) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        let model = self.model()?;
        if let Some(edges) = alizarin_core::ModelAccess::get_edges(&*model) {
            for (id, children) in edges {
                let py_list =
                    pyo3::types::PyList::new_bound(py, children.iter().map(|s| s.as_str()));
                dict.set_item(id, py_list)?;
            }
        }
        Ok(dict.to_object(py))
    }

    /// Get nodes indexed by alias as a dict of {alias: node_json}. As WASM's
    /// and NAPI's `getNodeObjectsByAlias`.
    fn get_node_objects_by_alias(&self, py: Python) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        let model = self.model()?;
        if let Some(nodes_by_alias) = model.get_nodes_by_alias() {
            for (alias, node) in nodes_by_alias {
                let node_json = serde_json::to_string(node.as_ref()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize node: {}",
                        e
                    ))
                })?;
                dict.set_item(alias, node_json)?;
            }
        }
        Ok(dict.to_object(py))
    }

    /// Set permitted nodegroups with support for both boolean and conditional rules.
    ///
    /// Accepts a dict where values can be:
    /// - bool: True/False for simple allow/deny
    /// - dict: {"path": ".data.uuid.field", "allowed": ["value1", "value2"]}
    ///   for conditional filtering based on tile data
    ///
    /// Raises ValueError if any permission rule is invalid.
    fn set_permitted_nodegroups(&mut self, py: Python, permissions: PyObject) -> PyResult<()> {
        let perms = instance_wrapper_py::parse_permissions_dict(py, &permissions)?;
        alizarin_core::set_model_permissions(&self.graph_id, perms)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    /// The permitted nodegroups as a nodegroup ID -> bool dict (conditional
    /// rules count as permitted). As NAPI's and WASM's `getPermittedNodegroups`.
    fn get_permitted_nodegroups(&self) -> PyResult<HashMap<String, bool>> {
        Ok(self.model()?.get_permitted_nodegroups_bool())
    }

    /// Whether nodegroups without an explicit rule are permitted. As NAPI's
    /// and WASM's `setDefaultAllowAllNodegroups`.
    fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) -> PyResult<()> {
        alizarin_core::set_model_default_allow(&self.graph_id, default_allow)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    /// Check if a nodegroup is permitted (for nodegroup-level filtering)
    fn is_nodegroup_permitted(&self, nodegroup_id: &str) -> bool {
        self.model()
            .is_ok_and(|model| model.is_nodegroup_permitted(nodegroup_id))
    }

    /// Prune graph to only include permitted nodegroups.
    fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> PyResult<()> {
        alizarin_core::update_model(&self.graph_id, |model| {
            model.prune_graph(keep_functions.as_deref())
        })
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to prune graph: {}",
                e
            ))
        })
    }

    /// The model's graph, as a dict. As WASM's and NAPI's `graph` getter.
    #[getter]
    fn graph(&self, py: Python) -> PyResult<PyObject> {
        let model = self.model()?;
        let graph = model.get_graph();
        pythonize::pythonize(py, graph).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert graph to Python: {}",
                e
            ))
        })
    }
}

// parse_conditional_rule removed — permission parsing is now centralised
// in instance_wrapper_py::parse_permissions_dict

/// Static method: Check if a tile/node matches as a semantic child
///
/// Note: `parent_nodegroup_id` was previously named `parent_node_id` - callers
/// should pass the nodegroup ID of the parent node, not the node ID.
#[pyfunction]
#[pyo3(signature = (parent_tile_id, parent_nodegroup_id, child_node_json, tile_json))]
fn matches_semantic_child(
    parent_tile_id: Option<String>,
    parent_nodegroup_id: Option<String>,
    child_node_json: String,
    tile_json: String,
) -> PyResult<bool> {
    let child_node: AlizarinStaticNode = serde_json::from_str(&child_node_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to parse child node: {}",
            e
        ))
    })?;

    let tile: AlizarinStaticTile = serde_json::from_str(&tile_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse tile: {}", e))
    })?;

    Ok(core_matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        &tile,
    ))
}

/// Resolve labels to UUIDs in a JSON tree using an RDM cache
///
/// This pre-processes a tree by looking up label strings in the appropriate
/// RDM collection (based on node config) and replacing them with UUIDs.
///
/// Args:
///     tree_json: The input tree as JSON string
///     graph_id: Graph ID (must be registered via register_graph first)
///     rdm_cache: The RDM cache containing collections (optional, uses global if not provided)
///     strict: If true, fail on unresolved labels; if false, leave them as-is
///
/// Returns:
///     The tree with labels resolved to UUIDs (as JSON string)
#[pyfunction]
#[pyo3(signature = (tree_json, graph_id, rdm_cache=None, strict = false))]
fn resolve_labels_in_tree(
    tree_json: String,
    graph_id: String,
    rdm_cache: Option<&rdm_cache_py::RdmCache>,
    strict: bool,
) -> PyResult<String> {
    // Parse tree
    let tree: serde_json::Value = serde_json::from_str(&tree_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse tree: {}", e))
    })?;

    // Get graph from registry
    let graph = get_registered_graph(&graph_id)?;

    // Use explicit cache or fall back to global
    // The global cache shared, not copied (get_global_rdm_cache copies it).
    let global_cache = alizarin_core::get_global_rdm_cache_arc();
    let cache_to_use: Option<&CoreRdmCache> =
        rdm_cache.map(|c| c.inner()).or(global_cache.as_deref());

    let resolved_tree = if let Some(cache) = cache_to_use {
        let ext_registry = build_extension_registry();
        let ext_registry_ref = if ext_registry.is_empty() {
            None
        } else {
            Some(&ext_registry)
        };
        let alias_map = build_alias_to_collection_from_graph(&graph, ext_registry_ref);
        core_resolve_labels(tree, &alias_map, cache, strict)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.message))?
    } else {
        tree
    };

    serde_json::to_string(&resolved_tree).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize result: {}",
            e
        ))
    })
}

/// Register a graph in the core registry for batch_merge_resources descriptor computation
///
/// Call this once per graph before using batch_merge_resources with recompute_descriptors=True.
/// The graph will be looked up by its graph_id when computing descriptors.
///
/// Args:
///     graph_json: Graph model as JSON string (can be direct object or {"graph": [...]})
///
/// Returns:
///     The graph_id that was registered
#[pyfunction]
#[pyo3(signature = (graph_json,))]
fn register_graph(graph_json: String) -> PyResult<String> {
    let graph = AlizarinCoreStaticGraph::from_json_string(&graph_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse graph: {}", e))
    })?;

    let graph_id = graph.graphid.clone();
    alizarin_core::register_graph_owned(graph);

    Ok(graph_id)
}

/// Run a resource's Aggregate functions over a business-data file, streamed one
/// resource at a time (so a large layer is never held in memory).
///
/// The resource's graph and its members' graphs must be registered
/// (`register_graph`). The resource itself is left out of the slice if the file
/// contains it.
///
/// Args:
///     resource_json: The resource (e.g. a Layer resource) as JSON
///     business_data_path: A `{"business_data": {"resources": [...]}}` file
///
/// Returns:
///     The resource as JSON, with its Aggregate node values written
#[pyfunction]
fn apply_aggregate_functions_from_file(
    resource_json: &str,
    business_data_path: &str,
) -> PyResult<String> {
    let invalid = |e: String| PyErr::new::<pyo3::exceptions::PyValueError, _>(e);
    let mut resource: AlizarinStaticResource = serde_json::from_str(resource_json)
        .map_err(|e| invalid(format!("invalid resource: {e}")))?;
    alizarin_core::apply_aggregate_functions_to_resource(
        &mut resource,
        &alizarin_core::BusinessDataFileSlice::new(business_data_path),
        &build_extension_registry(),
    )
    .map_err(invalid)?;
    serde_json::to_string(&resource).map_err(|e| invalid(e.to_string()))
}

/// Whether any value folded into a summary could match a predicate.
///
/// Conservative: False only when certain, so whatever the summary covers (e.g.
/// a Layer whose Aggregate node value it is) can be skipped. Predicates:
/// `{"range": {"min", "max"}}` for number/date/edtf, or a handler's own (geo:
/// `{"bbox": [minx, miny, maxx, maxy]}`); anything undecidable gives True. A
/// null summary folded nothing. Equivalent to WASM's and NAPI's `foldMayMatch`.
///
/// Args:
///     datatype: The datatype the summary folds (e.g. "number")
///     summary_json: The fold summary as JSON
///     predicate_json: The predicate as JSON
///     config_json: The node's config as JSON, if any
#[pyfunction]
#[pyo3(signature = (datatype, summary_json, predicate_json, config_json=None))]
fn fold_may_match(
    datatype: &str,
    summary_json: &str,
    predicate_json: &str,
    config_json: Option<&str>,
) -> PyResult<bool> {
    let parse = |what: &str, json: &str| -> PyResult<serde_json::Value> {
        serde_json::from_str(json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("invalid {what}: {e}"))
        })
    };
    let config = config_json.map(|c| parse("config", c)).transpose()?;
    Ok(alizarin_core::datatype_fold::fold_may_match(
        datatype,
        &parse("summary", summary_json)?,
        &parse("predicate", predicate_json)?,
        config.as_ref(),
        Some(&build_extension_registry()),
    ))
}

/// Set a descriptor template on a registered graph.
///
/// Creates or updates the descriptor function config for the given type.
/// The nodegroup_id is inferred from the <Node Name> placeholders in the template.
/// All placeholder nodes must belong to exactly one nodegroup.
///
/// Args:
///     graph_id: The graph ID to update
///     descriptor_type: Type name (e.g. "slug", "name", "description", "map_popup")
///     string_template: Template string with <Node Name> placeholders
#[pyfunction]
#[pyo3(signature = (graph_id, descriptor_type, string_template))]
fn set_descriptor_template(
    graph_id: &str,
    descriptor_type: &str,
    string_template: &str,
) -> PyResult<()> {
    let graph_arc = alizarin_core::get_graph(graph_id).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Graph '{}' not found in registry",
            graph_id
        ))
    })?;
    let mut graph = (*graph_arc).clone();
    graph
        .set_descriptor_template(descriptor_type, string_template)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    alizarin_core::register_graph_owned(graph);
    Ok(())
}

/// Register a datatype as a list type.
///
/// List types are datatypes where the array IS the value (not multiple items).
/// Extensions should call this at initialization for their custom list datatypes.
///
/// Example:
///     alizarin.register_list_datatype("reference")  # CLM reference type
#[pyfunction]
fn register_list_datatype(datatype: &str) {
    alizarin_core::register_list_datatype(datatype);
}

/// Check if a datatype is a registered list type.
///
/// Returns True if arrays for this datatype should be treated as single values
/// rather than iterated over during tree-to-tiles conversion.
#[pyfunction]
fn is_list_datatype(datatype: &str) -> bool {
    alizarin_core::is_list_datatype(datatype)
}

/// Get all registered list datatypes.
#[pyfunction]
fn list_datatypes() -> Vec<String> {
    alizarin_core::list_datatypes()
}

// =============================================================================
// Widget Registry Functions
// =============================================================================

/// Register a widget mapping for a datatype.
///
/// Extensions should call this at initialization to register their custom
/// datatype-to-widget mappings. This allows `get_default_widget_for_datatype`
/// in graph mutations to find the correct widget for extension datatypes.
///
/// Example:
///     # In CLM extension initialization:
///     alizarin.register_widget_for_datatype("reference", "reference-select-widget")
///     alizarin.register_widget_for_datatype("reference-list", "reference-multiselect-widget")
#[pyfunction]
fn register_widget_for_datatype(datatype: &str, widget_name: &str) {
    alizarin_core::register_widget_for_datatype(datatype, widget_name);
}

/// Get the registered widget name for a datatype.
///
/// Returns None if no widget is registered for this datatype.
#[pyfunction]
fn get_widget_for_datatype(datatype: &str) -> Option<String> {
    alizarin_core::get_widget_for_datatype(datatype)
}

/// Register a widget definition.
///
/// Extensions should call this at initialization to register their custom widgets.
/// This allows the mutation system to find extension widgets when creating
/// nodes with extension datatypes.
///
/// Args:
///     widget_id: UUID of the widget (should be a valid UUID string)
///     widget_name: Name of the widget (e.g., "reference-select-widget")
///     datatype: The datatype this widget handles
///     default_config_json: JSON string of default config (or "{}" for empty)
///
/// Example:
///     # In CLM extension initialization:
///     alizarin.register_widget(
///         "10000000-0000-0000-0000-000000000017",
///         "reference-select-widget",
///         "reference",
///         '{"placeholder": "Select a reference"}'
///     )
#[pyfunction]
fn register_widget(widget_id: &str, widget_name: &str, datatype: &str, default_config_json: &str) {
    let widget =
        alizarin_core::RegisteredWidget::new(widget_id, widget_name, datatype, default_config_json);
    alizarin_core::register_widget(widget);
}

/// Get all registered widget names.
#[pyfunction]
fn registered_widgets() -> Vec<String> {
    alizarin_core::registered_widgets()
}

/// Get all registered widget mappings (datatype -> widget_name).
#[pyfunction]
fn widget_mappings() -> Vec<(String, String)> {
    alizarin_core::widget_mappings()
}

// sort_json_keys is provided by alizarin_core::sort_json_keys
use alizarin_core::sort_json_keys;

/// Get the JSON representation of a registered graph.
///
/// The output is deterministic (sorted keys) for reliable git diffing.
///
/// Args:
///     graph_id: The UUID of the graph to retrieve
///
/// Returns:
///     The graph as a JSON string
///
/// Raises:
///     ValueError: If the graph is not registered
///
/// Example:
///     graph_json = get_graph_json("12345678-1234-1234-1234-123456789012")
///     data = json.loads(graph_json)
#[pyfunction]
#[pyo3(signature = (graph_id,))]
fn get_graph_json(graph_id: String) -> PyResult<String> {
    let graph = alizarin_core::get_graph(&graph_id).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Graph '{}' not registered. Call register_graph() first.",
            graph_id
        ))
    })?;

    // Serialize to Value first, then sort all keys for deterministic output
    let value = serde_json::to_value(&*graph).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize graph: {}", e))
    })?;

    let sorted = sort_json_keys(value);

    serde_json::to_string_pretty(&sorted).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize graph: {}", e))
    })
}

/// Get a simplified schema view of a registered graph showing node aliases and structure.
///
/// Returns a nested dictionary representing the tree structure with:
/// - Keys are node aliases (or nodeid if no alias)
/// - Values are dicts with 'datatype', 'nodeid', optionally 'required', and 'children'
///
/// Useful for understanding what keys are available in batch_tiles_to_trees output.
///
/// Args:
///     graph_id: The graph UUID that was returned from register_graph()
///
/// Returns:
///     dict: Nested structure showing node hierarchy with aliases and datatypes
///
/// Example:
///     schema = get_graph_schema("12345678-1234-1234-1234-123456789012")
///     print(json.dumps(schema, indent=2))
#[pyfunction]
#[pyo3(signature = (graph_id,))]
fn get_graph_schema(py: Python, graph_id: String) -> PyResult<PyObject> {
    let graph = get_registered_graph(&graph_id)?;
    let schema = graph.get_schema();
    pythonize::pythonize(py, &schema).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

/// Merge multiple resources with the same resourceinstanceid into one
///
/// Concatenates tiles from all resources, detecting and warning about duplicate tileids.
/// All resources must have the same resourceinstanceid.
///
/// Args:
///     resources_json: JSON string containing array of StaticResource objects
///
/// Returns:
///     Dict with 'resource' (merged StaticResource) and 'warnings' (list of duplicate warnings)
#[pyfunction]
#[pyo3(signature = (resources_json,))]
fn merge_resources(py: Python, resources_json: String) -> PyResult<PyObject> {
    let result = alizarin_core::merge_resources_from_json(&resources_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    pythonize::pythonize(py, &result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

/// Batch merge resources from multiple JSON strings
///
/// Takes an array of JSON strings. Each string can be:
/// - An array of StaticResource objects: [{...}, {...}]
/// - A single StaticResource object: {resourceinstance: {...}, tiles: [...]}
/// - A BusinessDataWrapper: {business_data: {resources: [...]}}
///
/// Groups all resources by resourceinstanceid and merges each group.
///
/// Args:
///     batches_json: List of JSON strings containing resources in any supported format
///     recompute_descriptors: If True, recomputes descriptors from merged tiles using
///         graphs from the registry. Graphs must be registered first.
///     strict: If True (default), raises ValueError on conflicting data when unifying
///         cardinality-1 tiles.
///
/// Returns:
///     Dict with 'resources' (list of merged StaticResources) and 'warnings' (list of warnings)
#[pyfunction]
#[pyo3(signature = (batches_json, recompute_descriptors=false, strict=true))]
fn batch_merge_resources(
    py: Python,
    batches_json: Vec<String>,
    recompute_descriptors: bool,
    strict: bool,
) -> PyResult<PyObject> {
    let result = alizarin_core::batch_merge_resources_from_json(
        &batches_json,
        recompute_descriptors,
        strict,
    )
    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    batch_merge_result_to_py(py, result)
}

/// Merge resources from multiple JSON files, reading one file at a time.
///
/// Memory-efficient alternative to loading all files and calling batch_merge_resources.
/// Each file is read from disk, parsed, and the raw JSON is freed before the next file
/// is processed. Only the accumulated StaticResource structs persist between files.
///
/// If output_path is provided, writes the result directly to a JSON file from Rust
/// (avoiding the large Python dict conversion) and returns only warnings.
///
/// Args:
///     file_paths: List of paths to JSON files containing resources
///     chunk_size: Number of files to merge per flush (default 1 = flush after every file)
///     recompute_descriptors: If True, recomputes descriptors in a final pass
///     strict: If True (default), raises ValueError on conflicting data
///     output_path: If provided, writes result JSON to this path and returns warnings only
///
/// Returns:
///     If output_path is None: Dict with 'resources' and 'warnings'
///     If output_path is set: Dict with 'warnings' and 'output_path'
#[pyfunction]
#[pyo3(signature = (file_paths, chunk_size=1, recompute_descriptors=true, strict=true, output_path=None))]
fn streamed_merge_from_files(
    py: Python,
    file_paths: Vec<String>,
    chunk_size: usize,
    recompute_descriptors: bool,
    strict: bool,
    output_path: Option<String>,
) -> PyResult<PyObject> {
    let mut accumulator = MergeAccumulator::new(chunk_size, strict);

    for (i, path) in file_paths.iter().enumerate() {
        let content = std::fs::read_to_string(path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to read file '{}' (index {}): {}",
                path, i, e
            ))
        })?;

        accumulator.add_json(&content).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse file '{}' (index {}): {}",
                path, i, e
            ))
        })?;
        // `content` dropped here — raw JSON string freed before next file
    }

    let result = accumulator.finish(recompute_descriptors);

    if let Some(ref error) = result.error {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            error.clone(),
        ));
    }

    if let Some(ref out_path) = output_path {
        // Write directly to file from Rust — serializes straight from the structs
        // to disk, avoiding any intermediate serde_json::Value or Python dict.
        let file = std::fs::File::create(out_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create output file '{}': {}",
                out_path, e
            ))
        })?;
        let writer = std::io::BufWriter::new(file);

        #[derive(serde::Serialize)]
        struct BusinessDataWrapper<'a> {
            business_data: ResourcesWrapper<'a>,
        }
        #[derive(serde::Serialize)]
        struct ResourcesWrapper<'a> {
            resources: &'a [AlizarinStaticResource],
        }

        let wrapper = BusinessDataWrapper {
            business_data: ResourcesWrapper {
                resources: &result.resources,
            },
        };
        serde_json::to_writer_pretty(writer, &wrapper).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to write output file '{}': {}",
                out_path, e
            ))
        })?;

        let warnings_output = serde_json::json!({
            "warnings": result.warnings,
            "output_path": out_path,
        });
        pythonize::pythonize(py, &warnings_output).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert warnings to Python: {}",
                e
            ))
        })
    } else {
        batch_merge_result_to_py(py, result)
    }
}

/// Convert a BatchMergeResult to a Python dict.
fn batch_merge_result_to_py(
    py: Python,
    result: alizarin_core::BatchMergeResult,
) -> PyResult<PyObject> {
    if let Some(ref error) = result.error {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            error.clone(),
        ));
    }

    // Core's serialisation: {resources, warnings} (no error, as checked above).
    pythonize::pythonize(py, &result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

// ============================================================================
// Prebuild Exporter
// ============================================================================

/// Export registered graphs to a directory.
///
/// Classifies graphs as resource_models or branches based on `isresource`,
/// writes as `{"graph": [graph_data]}` JSON files with deterministically sorted keys.
#[pyfunction]
#[pyo3(signature = (graph_ids, out_dir))]
fn export_graphs_to_dir(graph_ids: Vec<String>, out_dir: String) -> PyResult<Vec<String>> {
    let files = alizarin_core::export_graphs(&graph_ids)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    let export_data = alizarin_core::PrebuildExportData {
        graph_files: files,
        ..Default::default()
    };
    alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Export all registered graphs to a directory.
#[pyfunction]
#[pyo3(signature = (out_dir,))]
fn export_all_graphs_to_dir(out_dir: String) -> PyResult<Vec<String>> {
    let files = alizarin_core::export_all_graphs()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    let export_data = alizarin_core::PrebuildExportData {
        graph_files: files,
        ..Default::default()
    };
    alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Export RDM collections from the global cache as SKOS XML files.
#[pyfunction]
#[pyo3(signature = (out_dir, base_uri))]
fn export_collections_to_dir(out_dir: String, base_uri: String) -> PyResult<Vec<String>> {
    let cache = alizarin_core::get_global_rdm_cache_arc().ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Global RDM cache not set. Call set_global_rdm_cache() first.",
        )
    })?;

    let files = alizarin_core::export_collections(&cache, &base_uri)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let export_data = alizarin_core::PrebuildExportData {
        reference_data_files: files,
        ..Default::default()
    };
    alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Export a complete prebuild package (graphs + collections) to a directory.
#[pyfunction]
#[pyo3(signature = (out_dir, base_uri, graph_ids=None))]
fn export_prebuild(
    out_dir: String,
    base_uri: String,
    graph_ids: Option<Vec<String>>,
) -> PyResult<Vec<String>> {
    let core_cache = alizarin_core::get_global_rdm_cache_arc();

    alizarin_core::export_prebuild_to_directory(
        graph_ids.as_deref(),
        core_cache.as_deref(),
        &base_uri,
        std::path::Path::new(&out_dir),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Import a prebuild/pkg directory: register graphs, load SKOS collections, and load ontologies.
///
/// This is the inverse of `export_prebuild`. It reads the directory structure and:
/// 1. Loads and registers all graphs from graphs/resource_models/ and graphs/branches/
/// 2. Parses SKOS XML from reference_data/collections/ and adds to the global RDM cache
/// 3. Loads ontology RDFS files from ontologies/ (if present) and returns a validator
///
/// Args:
///     prebuild_dir: Path to the prebuild or pkg directory
///     base_uri: Base URI for SKOS XML parsing (default: "http://localhost:8000/")
///
/// Returns:
///     dict with keys:
///         graph_ids (list[str]): IDs of registered graphs
///         collection_ids (list[str]): IDs of loaded collections
///         ontology_validator (OntologyValidator | None): validator if ontologies were found
#[pyfunction]
#[pyo3(signature = (prebuild_dir, base_uri="http://localhost:8000/", extra_reference_data_dirs=None, extra_ontology_dirs=None))]
fn import_prebuild(
    py: Python,
    prebuild_dir: String,
    base_uri: &str,
    extra_reference_data_dirs: Option<Vec<String>>,
    extra_ontology_dirs: Option<Vec<String>>,
) -> PyResult<PyObject> {
    let extra_ref_strs: Option<Vec<&str>> = extra_reference_data_dirs
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());
    let extra_ont_strs: Option<Vec<&str>> = extra_ontology_dirs
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let result = alizarin_core::import_prebuild(
        &prebuild_dir,
        base_uri,
        extra_ref_strs.as_deref(),
        extra_ont_strs.as_deref(),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let ontology_validator: Option<graph_mutator_py::PyOntologyValidator> = result
        .ontology_validators
        .into_iter()
        .next()
        .map(graph_mutator_py::PyOntologyValidator::from_inner);

    let dict = pyo3::types::PyDict::new_bound(py);
    dict.set_item("graph_ids", result.graph_ids)?;
    dict.set_item("collection_ids", result.collection_ids)?;
    dict.set_item("ontology_validator", ontology_validator.into_py(py))?;
    Ok(dict.into())
}

/// Get IDs of all registered graphs.
#[pyfunction]
fn get_registered_graph_ids() -> Vec<String> {
    alizarin_core::get_registered_graph_ids()
}

/// Python module definition
#[pymodule]
fn alizarin(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Graph registry (for batch_merge_resources descriptor computation)
    m.add_function(wrap_pyfunction!(register_graph, m)?)?;
    m.add_function(wrap_pyfunction!(apply_aggregate_functions_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(fold_may_match, m)?)?;
    m.add_function(wrap_pyfunction!(get_graph_json, m)?)?;
    m.add_function(wrap_pyfunction!(get_graph_schema, m)?)?;
    m.add_function(wrap_pyfunction!(set_descriptor_template, m)?)?;
    m.add_function(wrap_pyfunction!(get_registered_graph_ids, m)?)?;

    // Prebuild exporter / importer
    m.add_function(wrap_pyfunction!(export_graphs_to_dir, m)?)?;
    m.add_function(wrap_pyfunction!(export_all_graphs_to_dir, m)?)?;
    m.add_function(wrap_pyfunction!(export_collections_to_dir, m)?)?;
    m.add_function(wrap_pyfunction!(export_prebuild, m)?)?;
    m.add_function(wrap_pyfunction!(import_prebuild, m)?)?;

    // List datatype registry (for datatypes where array IS the value)
    m.add_function(wrap_pyfunction!(register_list_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(is_list_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(list_datatypes, m)?)?;

    // Widget registry (for extension datatype -> widget mappings)
    m.add_function(wrap_pyfunction!(register_widget_for_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(get_widget_for_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(register_widget, m)?)?;
    m.add_function(wrap_pyfunction!(registered_widgets, m)?)?;
    m.add_function(wrap_pyfunction!(widget_mappings, m)?)?;

    // Low-level tree conversion functions (for compatibility)
    m.add_function(wrap_pyfunction!(tiles_to_json_tree, m)?)?;
    m.add_function(wrap_pyfunction!(build_tree_from_tiles, m)?)?;
    m.add_function(wrap_pyfunction!(json_tree_to_tiles, m)?)?;

    // Batch conversion functions (parallel processing with Rayon)
    m.add_function(wrap_pyfunction!(batch_trees_to_tiles, m)?)?;
    m.add_function(wrap_pyfunction!(batch_tiles_to_trees, m)?)?;
    m.add_function(wrap_pyfunction!(compose_resource_layers, m)?)?;
    m.add_function(wrap_pyfunction!(resource_lifecycle_state, m)?)?;
    m.add_function(wrap_pyfunction!(transition_resource_lifecycle, m)?)?;
    m.add_function(wrap_pyfunction!(canonical_static, m)?)?;
    m.add_function(wrap_pyfunction!(construct_static, m)?)?;
    m.add_function(wrap_pyfunction!(compare_nodes, m)?)?;
    m.add_function(wrap_pyfunction!(number_display, m)?)?;
    m.add_function(wrap_pyfunction!(resources_from_business_data_json, m)?)?;
    m.add_function(wrap_pyfunction!(summary_to_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(static_resource_from_summary, m)?)?;
    m.add_function(wrap_pyfunction!(cards_to_json_tree, m)?)?;

    // Streaming iterator (low memory, processes one tree at a time)
    m.add_class::<TreeToTilesIterator>()?;

    // Resource registry (for relationship resolution and cache population)
    m.add_class::<ResourceRegistry>()?;

    // Resource merging (combine tiles from multiple resources)
    m.add_function(wrap_pyfunction!(merge_resources, m)?)?;
    m.add_function(wrap_pyfunction!(batch_merge_resources, m)?)?;
    m.add_function(wrap_pyfunction!(streamed_merge_from_files, m)?)?;

    // Label resolution (resolve concept labels to UUIDs before conversion)
    m.add_function(wrap_pyfunction!(resolve_labels_in_tree, m)?)?;

    // Wrapper classes
    m.add_class::<PyResourceModelWrapper>()?;
    m.add_class::<PyPseudoNode>()?;

    // Utility functions
    m.add_function(wrap_pyfunction!(matches_semantic_child, m)?)?;

    // Extension type handler registration
    m.add_function(wrap_pyfunction!(register_extension_handler, m)?)?;
    m.add_function(wrap_pyfunction!(register_derive_provider, m)?)?;
    m.add_function(wrap_pyfunction!(has_derive_provider, m)?)?;
    m.add_function(wrap_pyfunction!(has_extension_handler, m)?)?;
    m.add_function(wrap_pyfunction!(coerce_with_extension, m)?)?;
    m.add_function(wrap_pyfunction!(has_display_renderer, m)?)?;
    m.add_function(wrap_pyfunction!(render_display_with_extension, m)?)?;
    m.add_function(wrap_pyfunction!(get_registered_extension_handlers, m)?)?;

    // Node configuration management (Phase 0 of type coercion)
    node_config_py::register_module(m)?;

    // RDM cache for concept collections (Phase 0 of type coercion)
    rdm_cache_py::register_module(m)?;

    // Type coercion functions (Phase 1: simple scalars)
    type_coercion_py::register_module(m)?;

    // Rust-backed pseudo values (single source of truth for matching_entries)
    pseudo_value_py::register_module(m)?;

    // Rust-backed instance wrapper (tile processing, pseudo cache population)
    instance_wrapper_py::register_module(m)?;

    // Graph mutation API (JSON-based)
    graph_mutator_py::register_module(m)?;

    // SKOS RDF/XML parsing and serialization
    skos_py::register_module(m)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use alizarin_core::string_utils::{camel_to_snake, transform_keys_to_snake};

    #[test]
    fn test_camel_to_snake_simple() {
        assert_eq!(camel_to_snake("camelCase"), "camel_case");
        assert_eq!(camel_to_snake("firstName"), "first_name");
        assert_eq!(camel_to_snake("lastName"), "last_name");
    }

    #[test]
    fn test_camel_to_snake_already_snake() {
        assert_eq!(camel_to_snake("snake_case"), "snake_case");
        assert_eq!(camel_to_snake("first_name"), "first_name");
    }

    #[test]
    fn test_camel_to_snake_single_word() {
        assert_eq!(camel_to_snake("name"), "name");
        assert_eq!(camel_to_snake("Name"), "name");
    }

    #[test]
    fn test_camel_to_snake_mixed() {
        // Note: The current implementation treats each uppercase letter as a word boundary
        // This is correct for standard camelCase used in JSON keys
        assert_eq!(camel_to_snake("graphId"), "graph_id");
        assert_eq!(camel_to_snake("resourceInstanceId"), "resource_instance_id");
        assert_eq!(camel_to_snake("nodeGroupId"), "node_group_id");
    }

    #[test]
    fn test_transform_keys_nested() {
        let input = serde_json::json!({
            "firstName": "John",
            "lastName": "Doe",
            "contactInfo": {
                "emailAddress": "john@example.com",
                "phoneNumber": "123-456"
            },
            "addresses": [
                {"streetName": "Main St", "zipCode": "12345"}
            ]
        });

        let result = transform_keys_to_snake(input);

        assert_eq!(result["first_name"], "John");
        assert_eq!(result["last_name"], "Doe");
        assert_eq!(result["contact_info"]["email_address"], "john@example.com");
        assert_eq!(result["contact_info"]["phone_number"], "123-456");
        assert_eq!(result["addresses"][0]["street_name"], "Main St");
        assert_eq!(result["addresses"][0]["zip_code"], "12345");
    }
}
