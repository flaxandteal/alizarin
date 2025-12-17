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

mod node_config_py;
mod pseudo_value_py;
mod rdm_cache_py;
mod type_coercion_py;
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

// Import core types and functions directly (no WASM dependency needed)
use alizarin_core::{
    // JSON conversion
    tiles_to_tree, tree_to_tiles, tree_to_tiles_strict, ResourceData, BusinessDataWrapper,
    // Graph types
    StaticTile as AlizarinStaticTile,
    StaticNode as AlizarinStaticNode,
    StaticResource as AlizarinStaticResource,
    StaticGraph as AlizarinCoreStaticGraph,
    StaticResourceMetadata, StaticResourceDescriptors,
    // Label resolution
    resolve_labels as core_resolve_labels,
    DEFAULT_RESOLVABLE_DATATYPES,
    DEFAULT_CONFIG_KEYS,
    // Semantic child matching
    matches_semantic_child as core_matches_semantic_child,
};

/// Build alias-to-collection mapping directly from StaticGraph.
///
/// This is more efficient than serializing to JSON and using build_alias_to_collection_map.
fn build_alias_to_collection_from_graph(
    graph: &AlizarinCoreStaticGraph,
) -> HashMap<String, String> {
    use std::collections::HashSet;

    let resolvable_set: HashSet<&str> = DEFAULT_RESOLVABLE_DATATYPES
        .iter()
        .copied()
        .collect();

    let mut alias_to_collection = HashMap::new();
    for node in &graph.nodes {
        if let Some(alias) = &node.alias {
            if !resolvable_set.contains(node.datatype.as_str()) {
                continue;
            }
            for key in DEFAULT_CONFIG_KEYS {
                if let Some(collection_id) = node.config.get(*key).and_then(|v| v.as_str()) {
                    alias_to_collection.insert(alias.clone(), collection_id.to_string());
                    break;
                }
            }
        }
    }
    alias_to_collection
}

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
    let graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;

    // Create ResourceData and convert to StaticResource for the new API
    let resource_data = ResourceData {
        resourceinstanceid: resource_id,
        graph_id,
        tiles,
    };
    let static_resource = resource_data.to_static_resource(&graph);

    // Convert to JSON Value for tiles_to_tree
    let input_json = serde_json::to_value(&static_resource)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to serialize resource: {}", e)
        ))?;

    // Call shared Rust conversion function (returns array)
    let json_tree_array = tiles_to_tree(&input_json, &graph)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Extract first element (single resource case)
    let json_tree = json_tree_array.as_array()
        .and_then(|arr| arr.first().cloned())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    // Wrap tree with metadata for consistency with json_tree_to_tiles
    let mut result = json_tree.clone();
    if let serde_json::Value::Object(ref mut map) = result {
        map.insert("resourceinstanceid".to_string(), serde_json::Value::String(static_resource.resourceinstance.resourceinstanceid.clone()));
        map.insert("graph_id".to_string(), serde_json::Value::String(static_resource.resourceinstance.graph_id.clone()));
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
    rdm_cache: Option<&rdm_cache_py::RdmCache>,
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
    let graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;

    // Resolve labels to UUIDs if an RDM cache is available
    // Use explicit cache if provided, otherwise try global cache
    let global_cache = rdm_cache_py::get_global_rdm_cache();
    let cache_to_use: Option<&rdm_cache_py::RdmCache> = rdm_cache.or(global_cache.as_ref());

    if let Some(cache) = cache_to_use {
        let alias_map = build_alias_to_collection_from_graph(&graph);
        tree = core_resolve_labels(tree, &alias_map, cache, strict)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.message))?;
    }

    // Add resourceinstanceid and graph_id to tree for new API
    if let serde_json::Value::Object(ref mut map) = tree {
        map.insert("resourceinstanceid".to_string(), serde_json::Value::String(resource_id.clone()));
        map.insert("graph_id".to_string(), serde_json::Value::String(graph_id.clone()));
    }

    // Call shared Rust conversion function (strict or non-strict)
    let business_data = if strict {
        tree_to_tiles_strict(&tree, &graph)
    } else {
        tree_to_tiles(&tree, &graph)
    }.map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Extract first resource (full StaticResource with resourceinstance metadata)
    let resource = business_data.business_data.resources.into_iter().next()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No resources returned"))?;

    // Convert to Python dict
    let result_json = serde_json::to_string(&resource)
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

    let graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse graph: {}", e)))?;
    let graph_id = graph.graph_id().to_string();

    let results: Vec<Result<serde_json::Value, String>> = trees
        .into_par_iter()
        .enumerate()
        .map(|(i, mut tree)| {
            if from_camel { tree = transform_keys_to_snake(tree); }

            // Add graph_id to tree if not present
            if let serde_json::Value::Object(ref mut map) = tree {
                if !map.contains_key("graph_id") {
                    map.insert("graph_id".to_string(), serde_json::Value::String(graph_id.clone()));
                }
            }

            let business_data = if strict {
                tree_to_tiles_strict(&tree, &graph)
            } else {
                tree_to_tiles(&tree, &graph)
            }.map_err(|e| format!("Tree {}: {}", i, e))?;

            // Extract first resource (full StaticResource with resourceinstance metadata)
            let resource = business_data.business_data.resources.into_iter().next()
                .ok_or_else(|| format!("Tree {}: No resources returned", i))?;

            serde_json::to_value(&resource)
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

    let graph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse graph: {}", e)))?;
    let graph_id = graph.graph_id().to_string();

    let results: Vec<Result<serde_json::Value, String>> = resources
        .into_par_iter()
        .enumerate()
        .map(|(i, resource)| {
            // Extract resource_id from either format:
            // - New format: resource.resourceinstance.resourceinstanceid
            // - Old format: resource.resourceinstanceid
            let resource_id = resource.get("resourceinstance")
                .and_then(|ri| ri.get("resourceinstanceid"))
                .or_else(|| resource.get("resourceinstanceid"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            // Check if input is already in StaticResource format (has resourceinstance)
            let input_json = if resource.get("resourceinstance").is_some() {
                // Already in StaticResource format, use directly
                resource.clone()
            } else {
                // Old format - convert to StaticResource
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
                let static_resource = resource_data.to_static_resource(&graph);

                serde_json::to_value(&static_resource)
                    .map_err(|e| format!("Resource {}: Failed to serialize: {}", i, e))?
            };

            // Call tiles_to_tree (returns array)
            let json_tree_array = tiles_to_tree(&input_json, &graph)
                .map_err(|e| format!("Resource {}: {}", i, e))?;

            // Extract first element and add metadata
            let mut tree = json_tree_array.as_array()
                .and_then(|arr| arr.first().cloned())
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

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
    graph: Arc<AlizarinCoreStaticGraph>,
    permitted_nodegroups: HashMap<String, bool>,
}

#[pymethods]
impl PyResourceModelWrapper {
    #[new]
    fn new(graph_json: String) -> PyResult<Self> {
        let graph: AlizarinCoreStaticGraph = serde_json::from_str(&graph_json)
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

    Ok(core_matches_semantic_child(
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
    // Convert resource to JSON Value for the new API
    let input_json = serde_json::to_value(&resource.inner)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to serialize resource: {}", e)
        ))?;

    // Call shared Rust conversion function (returns array)
    let json_tree_array = tiles_to_tree(&input_json, &graph.inner)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Extract first element (single resource case)
    let json_tree = json_tree_array.as_array()
        .and_then(|arr| arr.first().cloned())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

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
    let mut tree: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse JSON: {}", e)
        ))?;

    // Get graph_id from the graph
    let graph_id = graph.inner.graphid.clone();

    // Add resourceinstanceid and graph_id to tree for new API
    if let serde_json::Value::Object(ref mut map) = tree {
        map.insert("resourceinstanceid".to_string(), serde_json::Value::String(resource_id));
        map.insert("graph_id".to_string(), serde_json::Value::String(graph_id));
    }

    // Call shared Rust conversion function
    let business_data = tree_to_tiles(&tree, &graph.inner)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    // Extract first resource
    let resource = business_data.business_data.resources.into_iter().next()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No resources returned"))?;

    Ok(PyStaticResource { inner: resource })
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
    rdm_cache: Option<&rdm_cache_py::RdmCache>,
    strict: bool,
) -> PyResult<String> {
    // Parse tree and graph
    let tree: serde_json::Value = serde_json::from_str(&tree_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse tree: {}", e)
        ))?;

    let graph: AlizarinCoreStaticGraph = AlizarinCoreStaticGraph::from_json_string(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;

    // Use explicit cache or fall back to global
    let global_cache = rdm_cache_py::get_global_rdm_cache();
    let cache_to_use: Option<&rdm_cache_py::RdmCache> = rdm_cache.or(global_cache.as_ref());

    let resolved_tree = if let Some(cache) = cache_to_use {
        let alias_map = build_alias_to_collection_from_graph(&graph);
        core_resolve_labels(tree, &alias_map, cache, strict)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.message))?
    } else {
        tree
    };

    serde_json::to_string(&resolved_tree)
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
    node_config_py::register_module(m)?;

    // RDM cache for concept collections (Phase 0 of type coercion)
    rdm_cache_py::register_module(m)?;

    // Type coercion functions (Phase 1: simple scalars)
    type_coercion_py::register_module(m)?;

    // Rust-backed pseudo values (single source of truth for matching_entries)
    pseudo_value_py::register_module(m)?;

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
