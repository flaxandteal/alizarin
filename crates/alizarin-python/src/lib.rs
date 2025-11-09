/// Python bindings for Alizarin tree conversion
///
/// This provides Python bindings for converting between:
/// - Tiled resource format (flat tiles grouped by nodegroup)
/// - Tree format (nested JSON using node aliases as keys)

use pyo3::prelude::*;
use serde_json;

// Import shared conversion logic from parent crate
// Use ::alizarin to refer to the external crate, not the pymodule
use ::alizarin::json_conversion::{tiles_to_tree, tree_to_tiles, ResourceData};
use ::alizarin::graph::StaticGraph as AlizarinStaticGraph;
use ::alizarin::graph::StaticTile as AlizarinStaticTile;

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

    // Convert to Python dict
    let py_str = serde_json::to_string(&json_tree)
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
///     graph_json: Graph model as JSON string
///
/// Returns:
///     Dict with 'resourceinstanceid', 'graph_id', and 'tiles'
#[pyfunction]
fn json_tree_to_tiles(
    py: Python,
    tree_json: String,
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
    let resource_data = tree_to_tiles(&tree, &graph)
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

/// Python module definition
#[pymodule]
fn alizarin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tiles_to_json_tree, m)?)?;
    m.add_function(wrap_pyfunction!(json_tree_to_tiles, m)?)?;
    Ok(())
}
