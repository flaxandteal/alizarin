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
use ::alizarin::json_conversion::{tiles_to_tree, tree_to_tiles, ResourceData};
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

    // Parse graph from JSON
    let graph: AlizarinStaticGraph = serde_json::from_str(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;

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
///
/// Returns:
///     Dict with 'resourceinstanceid', 'graph_id', and 'tiles'
#[pyfunction]
#[pyo3(signature = (tree_json, resource_id, graph_id, graph_json))]
fn json_tree_to_tiles(
    py: Python,
    tree_json: String,
    resource_id: String,
    graph_id: String,
    graph_json: String,
) -> PyResult<PyObject> {
    // Parse tree from JSON
    let tree: serde_json::Value = serde_json::from_str(&tree_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse tree: {}", e)
        ))?;

    // Parse graph from JSON
    let graph: AlizarinStaticGraph = serde_json::from_str(&graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to parse graph: {}", e)
        ))?;

    // Call shared Rust conversion function
    let resource_data = tree_to_tiles(&tree, &graph, &resource_id, &graph_id)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

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
