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

mod node_config;
mod rdm_cache;
mod type_coercion;
use pyo3::types::PyCapsule;
use rayon::prelude::*;
use serde_json;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use alizarin_extension_api::{TypeHandlerInfo, CoerceFn, FreeFn};

lazy_static::lazy_static! {
    /// Global registry of extension type handlers
    static ref TYPE_HANDLERS: RwLock<HashMap<String, RegisteredHandler>> =
        RwLock::new(HashMap::new());
}

/// Internal representation of a registered handler
struct RegisteredHandler {
    coerce_fn: CoerceFn,
    free_fn: FreeFn,
}

/// Register a type handler from a PyCapsule
///
/// This is called once at import time by extensions to register their handlers.
/// After registration, the handler can be called directly from Rust without
/// crossing the Python boundary.
#[pyfunction]
fn register_type_handler(capsule: &PyCapsule) -> PyResult<()> {
    let ptr = capsule.pointer() as *const TypeHandlerInfo;

    unsafe {
        let info = &*ptr;
        let type_name = std::str::from_utf8_unchecked(
            std::slice::from_raw_parts(info.type_name_ptr, info.type_name_len)
        ).to_string();

        TYPE_HANDLERS.write().unwrap().insert(type_name.clone(), RegisteredHandler {
            coerce_fn: info.coerce_fn,
            free_fn: info.free_fn,
        });

        println!("Registered type handler for: {}", type_name);
    }

    Ok(())
}

/// Check if a type handler is registered
#[pyfunction]
fn has_type_handler(type_name: &str) -> bool {
    TYPE_HANDLERS.read().unwrap().contains_key(type_name)
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
    let handlers = TYPE_HANDLERS.read().unwrap();

    if let Some(handler) = handlers.get(type_name) {
        let config = config_json.unwrap_or("null");

        unsafe {
            let result = (handler.coerce_fn)(
                value_json.as_ptr(),
                value_json.len(),
                config.as_ptr(),
                config.len(),
            );

            if result.error_ptr.is_null() {
                let tile_json = std::str::from_utf8_unchecked(
                    std::slice::from_raw_parts(result.json_ptr, result.json_len)
                ).to_string();

                let resolved_json = std::str::from_utf8_unchecked(
                    std::slice::from_raw_parts(result.resolved_ptr, result.resolved_len)
                ).to_string();

                (handler.free_fn)(result);
                Ok((tile_json, resolved_json))
            } else {
                let error = std::str::from_utf8_unchecked(
                    std::slice::from_raw_parts(result.error_ptr, result.error_len)
                ).to_string();

                (handler.free_fn)(result);
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(error))
            }
        }
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(
            format!("No handler registered for type: {}", type_name)
        ))
    }
}

// Import shared conversion logic from parent crate
// Use ::alizarin to refer to the external crate, not the pymodule
use ::alizarin::json_conversion::{tiles_to_tree, tree_to_tiles, tree_to_tiles_strict, ResourceData};
// Import graph wrapper types (for json_conversion graph parameter)
use ::alizarin::graph::StaticGraph as AlizarinStaticGraph;
// Import core types (for ResourceData tiles and matching logic)
use ::alizarin::core::{
    StaticTile as AlizarinStaticTile,
    StaticNode as AlizarinStaticNode,
    StaticResource as AlizarinStaticResource,
    StaticGraph as AlizarinCoreStaticGraph,
};
use ::alizarin::instance_wrapper::ResourceInstanceWrapperCore as AlizarinInstanceWrapperCore;

/// Convert tiled resource to nested JSON tree
///
/// Args:
///     tiles: List of tile dicts
///     resource_id: Resource instance ID
///     graph_id: Graph ID
///     graph_json: Graph model as JSON string
///
/// Returns:
///     Nested dict structure representing the resource tree
#[pyfunction]
#[pyo3(signature = (tiles_json, resource_id, graph_id, graph_json))]
fn tiles_to_json_tree(
    py: Python,
    tiles_json: String,
    resource_id: String,
    graph_id: String,
    graph_json: String,
) -> PyResult<PyObject> {
    // Parse tiles from JSON
    let tiles: Vec<AlizarinStaticTile> = serde_json::from_str(&tiles_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse tiles: {}", e)
        ))?;

    // Parse graph from JSON (uses from_json_string which calls build_indices)
    let core_graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;
    let graph = AlizarinStaticGraph::from(core_graph);

    // Create ResourceData
    let resource_data = ResourceData {
        resourceinstanceid: resource_id,
        graph_id,
        tiles,
    };

    // Call shared Rust conversion function
    let json_tree = tiles_to_tree(&resource_data, &graph)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Wrap tree with metadata for consistency with json_tree_to_tiles
    let mut result = json_tree.clone();
    if let serde_json::Value::Object(ref mut map) = result {
        map.insert("resourceinstanceid".to_string(), serde_json::Value::String(resource_data.resourceinstanceid));
        map.insert("graph_id".to_string(), serde_json::Value::String(resource_data.graph_id));
    }

    // Convert to Python dict
    let py_str = serde_json::to_string(&result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to serialize result: {}", e)
        ))?;

    // Parse as Python dict
    let json_module = py.import("json")?;
    let py_dict = json_module.call_method1("loads", (py_str,))?;

    Ok(py_dict.to_object(py))
}

/// Convert nested JSON tree to tiled resource
///
/// Args:
///     tree_json: Nested JSON tree as string
///     resource_id: Resource instance ID
///     graph_id: Graph ID
///     graph_json: Graph model as JSON string
///     from_camel: If True, convert keys from camelCase to snake_case before resolving
///     strict: If True, raise an error for any validation failures or unparseable data
///
/// Returns:
///     Dict with 'resourceinstanceid', 'graph_id', and 'tiles'
///
/// If a global RDM cache is set (via set_global_rdm_cache), label strings
/// will automatically be resolved to UUIDs before conversion.
#[pyfunction]
#[pyo3(signature = (tree_json, resource_id, graph_id, graph_json, from_camel=false, strict=false, rdm_cache=None))]
fn json_tree_to_tiles(
    py: Python,
    tree_json: String,
    resource_id: String,
    graph_id: String,
    graph_json: String,
    from_camel: bool,
    strict: bool,
    rdm_cache: Option<&rdm_cache::RdmCache>,
) -> PyResult<PyObject> {
    // Parse tree from JSON
    let mut tree: serde_json::Value = serde_json::from_str(&tree_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse tree: {}", e)
        ))?;

    // Convert keys from camelCase to snake_case if requested
    if from_camel {
        tree = transform_keys_to_snake(tree);
    }

    // Parse graph from JSON (uses from_json_string which calls build_indices)
    let core_graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;

    // Resolve labels to UUIDs if an RDM cache is available
    // Use explicit cache if provided, otherwise try global cache
    // Must happen before converting core_graph to wrapper (which moves it)
    let global_cache = rdm_cache::get_global_rdm_cache();
    let cache_to_use: Option<&rdm_cache::RdmCache> = rdm_cache.or(global_cache.as_ref());

    if let Some(cache) = cache_to_use {
        resolve_labels_in_value(&mut tree, &core_graph, cache, strict)?;
    }

    let graph = AlizarinStaticGraph::from(core_graph);

    // Call shared Rust conversion function (strict or non-strict)
    let resource_data = if strict {
        tree_to_tiles_strict(&tree, &graph, &resource_id, &graph_id)
    } else {
        tree_to_tiles(&tree, &graph, &resource_id, &graph_id)
    }.map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Convert to Python dict
    let result_json = serde_json::to_string(&resource_data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to serialize result: {}", e)
        ))?;

    // Parse as Python dict
    let json_module = py.import("json")?;
    let py_dict = json_module.call_method1("loads", (result_json,))?;

    Ok(py_dict.to_object(py))
}

/// Helper to build batch result output and convert to Python
fn batch_result_to_python(
    py: Python,
    results: Vec<Result<serde_json::Value, String>>,
    strict: bool,
) -> PyResult<PyObject> {
    if strict {
        // In strict mode, fail on first error
        let successes: Result<Vec<_>, _> = results.into_iter().collect();
        let successes = successes.map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Strict mode error: {}", e))
        })?;

        let output = serde_json::json!({
            "success": true,
            "results": successes,
            "count": successes.len()
        });

        let json_module = py.import("json")?;
        let py_dict = json_module.call_method1("loads", (serde_json::to_string(&output).unwrap(),))?;
        return Ok(py_dict.to_object(py));
    }

    // Non-strict: separate successes and errors
    let (successes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
    let successes: Vec<_> = successes.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(|e| e.unwrap_err()).collect();

    let output = if errors.is_empty() {
        serde_json::json!({ "success": true, "results": successes, "count": successes.len() })
    } else {
        serde_json::json!({
            "success": false, "results": successes, "count": successes.len(),
            "errors": errors, "error_count": errors.len()
        })
    };

    let json_module = py.import("json")?;
    let py_dict = json_module.call_method1("loads", (serde_json::to_string(&output).unwrap(),))?;
    Ok(py_dict.to_object(py))
}

/// Batch convert multiple JSON trees to tiles in parallel
#[pyfunction]
#[pyo3(signature = (trees_json, graph_json, from_camel=false, strict=false))]
fn batch_trees_to_tiles(
    py: Python,
    trees_json: String,
    graph_json: String,
    from_camel: bool,
    strict: bool,
) -> PyResult<PyObject> {
    let trees: Vec<serde_json::Value> = serde_json::from_str(&trees_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse trees: {}", e)))?;

    let core_graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse graph: {}", e)))?;
    let graph = AlizarinStaticGraph::from(core_graph);
    let graph_id = graph.graph_id().to_string();

    let results: Vec<Result<serde_json::Value, String>> = trees
        .into_par_iter()
        .enumerate()
        .map(|(i, mut tree)| {
            if from_camel { tree = transform_keys_to_snake(tree); }

            let resource_id = tree.get("resourceinstanceid")
                .and_then(|v| v.as_str()).map(String::from)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let resource_data = if strict {
                tree_to_tiles_strict(&tree, &graph, &resource_id, &graph_id)
            } else {
                tree_to_tiles(&tree, &graph, &resource_id, &graph_id)
            }.map_err(|e| format!("Tree {}: {}", i, e))?;

            serde_json::to_value(&resource_data)
                .map_err(|e| format!("Tree {}: Failed to serialize: {}", i, e))
        })
        .collect();

    batch_result_to_python(py, results, strict)
}

/// Batch convert multiple tiled resources to JSON trees in parallel
#[pyfunction]
#[pyo3(signature = (resources_json, graph_json, strict=false))]
fn batch_tiles_to_trees(
    py: Python,
    resources_json: String,
    graph_json: String,
    strict: bool,
) -> PyResult<PyObject> {
    let resources: Vec<serde_json::Value> = serde_json::from_str(&resources_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse resources: {}", e)))?;

    let core_graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse graph: {}", e)))?;
    let graph = AlizarinStaticGraph::from(core_graph);
    let graph_id = graph.graph_id().to_string();

    let results: Vec<Result<serde_json::Value, String>> = resources
        .into_par_iter()
        .enumerate()
        .map(|(i, resource)| {
            let resource_id = resource.get("resourceinstanceid")
                .and_then(|v| v.as_str()).map(String::from)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let tiles: Vec<AlizarinStaticTile> = resource.get("tiles")
                .map(|t| serde_json::from_value(t.clone()))
                .transpose()
                .map_err(|e| format!("Resource {}: Failed to parse tiles: {}", i, e))?
                .unwrap_or_default();

            let resource_data = ResourceData {
                resourceinstanceid: resource_id.clone(),
                graph_id: graph_id.clone(),
                tiles,
            };

            let mut tree = tiles_to_tree(&resource_data, &graph)
                .map_err(|e| format!("Resource {}: {}", i, e))?;

            if let serde_json::Value::Object(ref mut map) = tree {
                map.insert("resourceinstanceid".to_string(), serde_json::Value::String(resource_id));
                map.insert("graph_id".to_string(), serde_json::Value::String(graph_id.clone()));
            }

            Ok(tree)
        })
        .collect();

    batch_result_to_python(py, results, strict)
}

/// Convert a camelCase string to snake_case
fn camel_to_snake(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 10);
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            // Don't add underscore at the start
            if !result.is_empty() {
                // Check if we're in an acronym (current is upper, next is upper or end)
                let next_is_upper_or_end = chars.peek().map(|n| n.is_uppercase()).unwrap_or(true);
                let prev_was_upper = result.chars().last().map(|p| p.is_uppercase()).unwrap_or(false);

                // Add underscore if:
                // - Previous was lowercase, or
                // - We're at the end of an acronym (next is lowercase)
                if !prev_was_upper || !next_is_upper_or_end {
                    result.push('_');
                }
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

/// Recursively convert all keys in a JSON value from camelCase to snake_case
fn transform_keys_to_snake(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                let new_key = camel_to_snake(&key);
                let new_val = transform_keys_to_snake(val);
                new_map.insert(new_key, new_val);
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(transform_keys_to_snake).collect())
        }
        other => other,
    }
}

/// Python wrapper for ResourceModelWrapperCore
///
/// Provides graph pruning and permission checking
#[pyclass]
struct PyResourceModelWrapper {
    graph: Arc<AlizarinStaticGraph>,
    permitted_nodegroups: HashMap<String, bool>,
}

#[pymethods]
impl PyResourceModelWrapper {
    #[new]
    fn new(graph_json: String) -> PyResult<Self> {
        let graph: AlizarinStaticGraph = serde_json::from_str(&graph_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to parse graph: {}", e)
            ))?;

        Ok(PyResourceModelWrapper {
            graph: Arc::new(graph),
            permitted_nodegroups: HashMap::new(),
        })
    }

    /// Set permitted nodegroups
    fn set_permitted_nodegroups(&mut self, permissions: HashMap<String, bool>) {
        self.permitted_nodegroups = permissions;
    }

    /// Check if a nodegroup is permitted
    fn is_nodegroup_permitted(&self, nodegroup_id: &str) -> bool {
        self.permitted_nodegroups.get(nodegroup_id).copied().unwrap_or(true)
    }

    /// Export graph as JSON
    fn export_graph(&self, py: Python) -> PyResult<PyObject> {
        let graph_json = serde_json::to_string(&*self.graph)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to serialize graph: {}", e)
            ))?;

        let json_module = py.import("json")?;
        let py_dict = json_module.call_method1("loads", (graph_json,))?;
        Ok(py_dict.to_object(py))
    }
}

/// Static method: Check if a tile/node matches as a semantic child
#[pyfunction]
#[pyo3(signature = (parent_tile_id, parent_node_id, child_node_json, tile_json))]
fn matches_semantic_child(
    parent_tile_id: Option<String>,
    parent_node_id: String,
    child_node_json: String,
    tile_json: String,
) -> PyResult<bool> {
    let child_node: AlizarinStaticNode = serde_json::from_str(&child_node_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse child node: {}", e)
        ))?;

    let tile: AlizarinStaticTile = serde_json::from_str(&tile_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse tile: {}", e)
        ))?;

    Ok(AlizarinInstanceWrapperCore::matches_semantic_child(
        parent_tile_id.as_ref(),
        &parent_node_id,
        &child_node,
        &tile,
    ))
}

/// Python wrapper for StaticGraph (core type)
#[pyclass(name = "StaticGraphCore")]
struct PyStaticGraph {
    inner: AlizarinCoreStaticGraph,
}

#[pymethods]
impl PyStaticGraph {
    /// Create from JSON string
    /// Handles both direct graph objects and wrapped format: {"graph": [...]}
    #[new]
    fn new(json_str: String) -> PyResult<Self> {
        let graph = AlizarinCoreStaticGraph::from_json_string(&json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(PyStaticGraph { inner: graph })
    }

    /// Get graph ID
    #[getter]
    fn graphid(&self) -> String {
        self.inner.graphid.clone()
    }

    /// Get graph name as JSON string
    fn name_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to serialize name: {}", e)
            ))
    }

    /// Export full graph as JSON string
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to serialize graph: {}", e)
            ))
    }
}

/// Python wrapper for StaticResource (core type)
#[pyclass(name = "StaticResourceCore")]
struct PyStaticResource {
    inner: AlizarinStaticResource,
}

#[pymethods]
impl PyStaticResource {
    /// Create from JSON string
    #[new]
    fn new(json_str: String) -> PyResult<Self> {
        let resource: AlizarinStaticResource = serde_json::from_str(&json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to parse resource: {}", e)
            ))?;
        Ok(PyStaticResource { inner: resource })
    }

    /// Get resource instance ID
    #[getter]
    fn resourceinstanceid(&self) -> String {
        self.inner.resourceinstance.resourceinstanceid.clone()
    }

    /// Get graph ID
    #[getter]
    fn graph_id(&self) -> String {
        self.inner.resourceinstance.graph_id.clone()
    }

    /// Get tiles as JSON string
    fn get_tiles_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.tiles)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to serialize tiles: {}", e)
            ))
    }

    /// Export full resource as JSON string
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to serialize resource: {}", e)
            ))
    }
}

/// Convert resource to nested JSON string (convenience function)
#[pyfunction]
fn resource_to_json_string(resource: &PyStaticResource, graph: &PyStaticGraph) -> PyResult<String> {
    // Extract tiles from resource
    let tiles = resource.inner.tiles.as_ref()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Resource has no tiles"))?;

    // Create ResourceData
    let resource_data = ResourceData {
        resourceinstanceid: resource.inner.resourceinstance.resourceinstanceid.clone(),
        graph_id: resource.inner.resourceinstance.graph_id.clone(),
        tiles: tiles.clone(),
    };

    // Convert core graph to wrapper graph for json_conversion
    let wrapper_graph: AlizarinStaticGraph = serde_json::from_str(&serde_json::to_string(&graph.inner).unwrap())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to convert graph: {}", e)
        ))?;

    // Call shared Rust conversion function
    let json_tree = tiles_to_tree(&resource_data, &wrapper_graph)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Return as JSON string
    serde_json::to_string(&json_tree)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to serialize result: {}", e)
        ))
}

/// Convert nested JSON string to resource (convenience function)
#[pyfunction]
fn json_string_to_resource(
    json_str: String,
    resource_id: String,
    graph: &PyStaticGraph,
) -> PyResult<PyStaticResource> {
    // Parse as JSON value
    let tree: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse JSON: {}", e)
        ))?;

    // Get graph_id from the graph
    let graph_id = graph.inner.graphid.clone();

    // Convert core graph to wrapper graph for json_conversion
    let wrapper_graph: AlizarinStaticGraph = serde_json::from_str(&serde_json::to_string(&graph.inner).unwrap())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to convert graph: {}", e)
        ))?;

    // Call shared Rust conversion function
    let resource_data = tree_to_tiles(&tree, &wrapper_graph, &resource_id, &graph_id)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Create StaticResource from ResourceData
    let resource = AlizarinStaticResource {
        resourceinstance: ::alizarin::core::StaticResourceMetadata {
            resourceinstanceid: resource_data.resourceinstanceid.clone(),
            graph_id: resource_data.graph_id.clone(),
            name: String::new(),
            descriptors: ::alizarin::core::StaticResourceDescriptors::default(),
            publication_id: None,
            principaluser_id: None,
            legacyid: None,
            graph_publication_id: None,
            createdtime: None,
            lastmodified: None,
        },
        tiles: Some(resource_data.tiles),
        metadata: HashMap::new(),
        cache: None,
        scopes: None,
        tiles_loaded: Some(true),
    };

    Ok(PyStaticResource { inner: resource })
}

/// Check if a string looks like a UUID
fn is_uuid_format(s: &str) -> bool {
    let uuid_pattern = regex::Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    ).unwrap();
    uuid_pattern.is_match(s)
}

/// Internal helper: resolve labels in a mutable JSON value using graph and cache
fn resolve_labels_in_value(
    tree: &mut serde_json::Value,
    graph: &AlizarinCoreStaticGraph,
    rdm_cache: &rdm_cache::RdmCache,
    strict: bool,
) -> PyResult<()> {
    // Build a mapping of alias → (datatype, rdmCollection)
    let mut alias_to_config: HashMap<String, (String, Option<String>)> = HashMap::new();
    for node in &graph.nodes {
        if let Some(alias) = &node.alias {
            let rdm_collection = node.config.get("rdmCollection")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            alias_to_config.insert(alias.clone(), (node.datatype.clone(), rdm_collection));
        }
    }

    // Recursively resolve labels
    fn resolve_value_recursive(
        value: &mut serde_json::Value,
        alias: Option<&str>,
        alias_to_config: &HashMap<String, (String, Option<String>)>,
        rdm_cache: &rdm_cache::RdmCache,
        strict: bool,
        errors: &mut Vec<String>,
    ) {
        match value {
            serde_json::Value::Object(obj) => {
                // Check for _value wrapper
                if let Some(inner) = obj.get_mut("_value") {
                    resolve_value_recursive(inner, alias, alias_to_config, rdm_cache, strict, errors);
                    return;
                }

                // Process each field
                let keys: Vec<String> = obj.keys().cloned().collect();
                for key in keys {
                    if let Some(field_value) = obj.get_mut(&key) {
                        resolve_value_recursive(field_value, Some(&key), alias_to_config, rdm_cache, strict, errors);
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr.iter_mut() {
                    resolve_value_recursive(item, alias, alias_to_config, rdm_cache, strict, errors);
                }
            }
            serde_json::Value::String(s) => {
                // Check if this alias is a concept field with an RDM collection
                if let Some(alias_str) = alias {
                    if let Some((datatype, Some(collection_id))) = alias_to_config.get(alias_str) {
                        if (datatype == "concept" || datatype == "concept-list") && !is_uuid_format(s) {
                            // Try to resolve the label
                            if let Some(concept) = rdm_cache.find_concept_by_label(collection_id, s) {
                                *value = serde_json::Value::String(concept.id.clone());
                            } else if strict {
                                let matches = rdm_cache.find_all_concepts_by_label(collection_id, s);
                                if matches.len() > 1 {
                                    errors.push(format!(
                                        "Ambiguous label '{}' in collection '{}' matches {} concepts",
                                        s, collection_id, matches.len()
                                    ));
                                } else {
                                    errors.push(format!(
                                        "Label '{}' not found in collection '{}'",
                                        s, collection_id
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut errors = Vec::new();
    resolve_value_recursive(tree, None, &alias_to_config, rdm_cache, strict, &mut errors);

    if !errors.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to resolve labels:\n  {}", errors.join("\n  "))
        ));
    }

    Ok(())
}

/// Resolve labels to UUIDs in a JSON tree using an RDM cache
///
/// This pre-processes a tree by looking up label strings in the appropriate
/// RDM collection (based on node config) and replacing them with UUIDs.
///
/// Args:
///     tree_json: The input tree as JSON string
///     graph_json: The graph model as JSON string
///     rdm_cache: The RDM cache containing collections (optional, uses global if not provided)
///     strict: If true, fail on unresolved labels; if false, leave them as-is
///
/// Returns:
///     The tree with labels resolved to UUIDs (as JSON string)
#[pyfunction]
#[pyo3(signature = (tree_json, graph_json, rdm_cache=None, strict = false))]
fn resolve_labels_in_tree(
    tree_json: String,
    graph_json: String,
    rdm_cache: Option<&rdm_cache::RdmCache>,
    strict: bool,
) -> PyResult<String> {
    // Parse tree and graph
    let mut tree: serde_json::Value = serde_json::from_str(&tree_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse tree: {}", e)
        ))?;

    let graph: AlizarinCoreStaticGraph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;

    // Use explicit cache or fall back to global
    let global_cache = rdm_cache::get_global_rdm_cache();
    let cache_to_use: Option<&rdm_cache::RdmCache> = rdm_cache.or(global_cache.as_ref());

    if let Some(cache) = cache_to_use {
        resolve_labels_in_value(&mut tree, &graph, cache, strict)?;
    }

    serde_json::to_string(&tree)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to serialize result: {}", e)
        ))
}

/// Python module definition
#[pymodule]
fn alizarin(_py: Python, m: &PyModule) -> PyResult<()> {
    // Core Rust-backed types (named with "Core" suffix, Python will wrap them)
    m.add_class::<PyStaticGraph>()?;
    m.add_class::<PyStaticResource>()?;

    // Convenience functions (work with JSON strings)
    m.add_function(wrap_pyfunction!(resource_to_json_string, m)?)?;
    m.add_function(wrap_pyfunction!(json_string_to_resource, m)?)?;

    // Low-level tree conversion functions (for compatibility)
    m.add_function(wrap_pyfunction!(tiles_to_json_tree, m)?)?;
    m.add_function(wrap_pyfunction!(json_tree_to_tiles, m)?)?;

    // Batch conversion functions (parallel processing with Rayon)
    m.add_function(wrap_pyfunction!(batch_trees_to_tiles, m)?)?;
    m.add_function(wrap_pyfunction!(batch_tiles_to_trees, m)?)?;

    // Label resolution (resolve concept labels to UUIDs before conversion)
    m.add_function(wrap_pyfunction!(resolve_labels_in_tree, m)?)?;

    // Wrapper classes
    m.add_class::<PyResourceModelWrapper>()?;

    // Utility functions
    m.add_function(wrap_pyfunction!(matches_semantic_child, m)?)?;

    // Extension type handler registration
    m.add_function(wrap_pyfunction!(register_type_handler, m)?)?;
    m.add_function(wrap_pyfunction!(has_type_handler, m)?)?;
    m.add_function(wrap_pyfunction!(coerce_with_extension, m)?)?;

    // Node configuration management (Phase 0 of type coercion)
    node_config::register_module(m)?;

    // RDM cache for concept collections (Phase 0 of type coercion)
    rdm_cache::register_module(m)?;

    // Type coercion functions (Phase 1: simple scalars)
    type_coercion::register_module(m)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
