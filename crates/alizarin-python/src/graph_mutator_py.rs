//! Python bindings for the JSON-based Graph Mutation API.
//!
//! This module exposes a simple JSON-based API for mutating graphs.

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

use alizarin_core::graph_mutator::{
    apply_mutations_create_from_json as core_apply_mutations_create,
    apply_mutations_from_json as core_apply_mutations,
    apply_mutations_from_json_with_extensions as core_apply_mutations_with_ext,
    generate_uuid_v5 as core_generate_uuid, get_mutation_schema as core_get_schema,
    ExtensionMutationHandler, ExtensionMutationRegistry, MutationConformance, MutationError,
    MutatorOptions,
};
use alizarin_core::StaticGraph as CoreStaticGraph;

// Global extension mutation registry
lazy_static::lazy_static! {
    static ref EXTENSION_REGISTRY: Mutex<ExtensionMutationRegistry> =
        Mutex::new(ExtensionMutationRegistry::new());
}

/// Handler that calls a Python function for extension mutations
struct PyExtensionHandler {
    #[allow(dead_code)]
    name: String,
    apply_fn: PyObject,
    conformance: MutationConformance,
}

impl ExtensionMutationHandler for PyExtensionHandler {
    fn apply(
        &self,
        graph: &mut CoreStaticGraph,
        params: &serde_json::Value,
        _options: &MutatorOptions,
    ) -> Result<(), MutationError> {
        Python::with_gil(|py| {
            // Serialize graph and params to JSON for Python
            let graph_json = serde_json::to_string(graph)
                .map_err(|e| MutationError::Other(format!("Failed to serialize graph: {}", e)))?;
            let params_json = serde_json::to_string(params)
                .map_err(|e| MutationError::Other(format!("Failed to serialize params: {}", e)))?;

            // Call Python function
            let result = self
                .apply_fn
                .call1(py, (graph_json, params_json))
                .map_err(|e| MutationError::Other(format!("Python handler error: {}", e)))?;

            // Parse result back as mutated graph JSON
            let result_json: String = result.extract(py).map_err(|e| {
                MutationError::Other(format!("Handler must return JSON string: {}", e))
            })?;

            let mutated: CoreStaticGraph = serde_json::from_str(&result_json)
                .map_err(|e| MutationError::Other(format!("Invalid graph JSON returned: {}", e)))?;

            // Replace graph contents
            *graph = mutated;
            Ok(())
        })
    }

    fn conformance(&self) -> MutationConformance {
        self.conformance
    }

    fn description(&self) -> &str {
        "Python extension mutation handler"
    }
}

// Make PyExtensionHandler safe to use across threads (Python GIL handles sync)
unsafe impl Send for PyExtensionHandler {}
unsafe impl Sync for PyExtensionHandler {}

/// Apply mutations to a graph from a JSON string.
///
/// Args:
///     graph_json: The source graph as JSON string
///     mutations_json: JSON string containing a MutationRequest
///
/// Returns:
///     The mutated graph as JSON string
///
/// Example mutations_json:
/// ```json
/// {
///   "mutations": [
///     {
///       "AddNode": {
///         "parent_alias": "root",
///         "alias": "child",
///         "name": "Child Node",
///         "cardinality": "N",
///         "datatype": "string",
///         "ontology_class": "E41_Appellation",
///         "parent_property": "P1_is_identified_by",
///         "options": {}
///       }
///     }
///   ],
///   "options": {
///     "autocreate_card": true,
///     "autocreate_widget": true
///   }
/// }
/// ```
#[pyfunction]
fn apply_mutations_from_json(graph_json: &str, mutations_json: &str) -> PyResult<String> {
    // Parse graph from JSON
    let graph = CoreStaticGraph::from_json_string(graph_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Apply mutations
    let mutated = core_apply_mutations(&graph, mutations_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Serialize result
    serde_json::to_string(&mutated).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize mutated graph: {}",
            e
        ))
    })
}

/// Generate a deterministic UUID v5 from group and key.
///
/// This matches the behavior of the Rust UUID generation for cross-platform
/// consistency.
///
/// Args:
///     group_type: The type of group (e.g., "graph", "node")
///     group_id: Optional ID for the group
///     key: The key to generate a UUID for
///
/// Returns:
///     A UUID string
#[pyfunction]
#[pyo3(signature = (group_type, key, group_id=None))]
fn generate_uuid_v5(group_type: &str, key: &str, group_id: Option<&str>) -> String {
    core_generate_uuid((group_type, group_id), key)
}

/// Get the JSON schema for mutation types.
///
/// Returns a documentation dict describing the structure of each mutation type.
/// Useful for building UI forms or validating mutation payloads.
#[pyfunction]
fn get_mutation_schema(py: Python) -> PyResult<PyObject> {
    let schema = core_get_schema();
    let schema_str = serde_json::to_string(&schema).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize schema: {}",
            e
        ))
    })?;

    let json_module = py.import_bound("json")?;
    let py_dict = json_module.call_method1("loads", (schema_str,))?;
    Ok(py_dict.to_object(py))
}

/// Apply mutations with extension support.
///
/// This version uses the global extension mutation registry to handle
/// Extension mutations. Register handlers with `register_extension_mutation`.
///
/// Args:
///     graph_json: The source graph as JSON string
///     mutations_json: JSON string containing a MutationRequest
///
/// Returns:
///     The mutated graph as JSON string
#[pyfunction]
fn apply_mutations_with_extensions(graph_json: &str, mutations_json: &str) -> PyResult<String> {
    // Parse graph from JSON
    let graph = CoreStaticGraph::from_json_string(graph_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Get registry
    let registry = EXTENSION_REGISTRY.lock().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Failed to acquire extension registry lock",
        )
    })?;

    // Apply mutations with extensions
    let mutated = core_apply_mutations_with_ext(&graph, mutations_json, Some(&registry))
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Serialize result
    serde_json::to_string(&mutated).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize mutated graph: {}",
            e
        ))
    })
}

/// Register a Python function as an extension mutation handler.
///
/// The handler function should have the signature:
///     def handler(graph_json: str, params_json: str) -> str:
///         # Modify graph based on params
///         # Return mutated graph as JSON string
///
/// Args:
///     name: The mutation name (e.g., "clm.reference_change_collection")
///     handler: Python function to handle the mutation
///     conformance: Conformance level ("AlwaysConformant", "BranchConformant", "ModelConformant")
///
/// Example:
///     def my_handler(graph_json: str, params_json: str) -> str:
///         import json
///         graph = json.loads(graph_json)
///         params = json.loads(params_json)
///         # Modify graph...
///         return json.dumps(graph)
///
///     register_extension_mutation("my.custom_mutation", my_handler, "AlwaysConformant")
#[pyfunction]
#[pyo3(signature = (name, handler, conformance="AlwaysConformant"))]
fn register_extension_mutation(
    py: Python,
    name: &str,
    handler: PyObject,
    conformance: &str,
) -> PyResult<()> {
    // Parse conformance level
    let conformance_level = match conformance {
        "AlwaysConformant" => MutationConformance::AlwaysConformant,
        "BranchConformant" => MutationConformance::BranchConformant,
        "ModelConformant" => MutationConformance::ModelConformant,
        "NonConformant" => MutationConformance::NonConformant,
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid conformance level: {}",
                conformance
            )))
        }
    };

    // Verify handler is callable
    if !handler.bind(py).is_callable() {
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Handler must be callable",
        ));
    }

    // Create handler
    let py_handler = PyExtensionHandler {
        name: name.to_string(),
        apply_fn: handler,
        conformance: conformance_level,
    };

    // Register with global registry
    let mut registry = EXTENSION_REGISTRY.lock().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Failed to acquire extension registry lock",
        )
    })?;

    registry.register(name, Arc::new(py_handler));
    Ok(())
}

/// Check if an extension mutation handler is registered.
///
/// Args:
///     name: The mutation name to check
///
/// Returns:
///     True if the handler is registered
#[pyfunction]
fn has_extension_mutation(name: &str) -> PyResult<bool> {
    let registry = EXTENSION_REGISTRY.lock().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Failed to acquire extension registry lock",
        )
    })?;

    Ok(registry.has(name))
}

/// List all registered extension mutation names.
///
/// Returns:
///     List of registered mutation names
#[pyfunction]
fn list_extension_mutations() -> PyResult<Vec<String>> {
    let registry = EXTENSION_REGISTRY.lock().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Failed to acquire extension registry lock",
        )
    })?;

    Ok(registry.list().into_iter().map(|s| s.to_string()).collect())
}

/// Apply mutations that may create a new graph.
///
/// If graph_json is None, the first mutation must be CreateGraph which creates
/// a new graph from scratch. Remaining mutations are applied to the new graph.
///
/// If graph_json is provided, the first mutation must NOT be CreateGraph,
/// and all mutations are applied to the existing graph.
///
/// Args:
///     mutations_json: JSON string containing a MutationRequest
///     graph_json: Optional existing graph as JSON string
///
/// Returns:
///     The resulting graph as JSON string
#[pyfunction]
#[pyo3(signature = (mutations_json, graph_json=None))]
fn apply_mutations_create(mutations_json: &str, graph_json: Option<&str>) -> PyResult<String> {
    // Parse existing graph if provided
    let existing_graph = match graph_json {
        Some(json) => {
            let graph = CoreStaticGraph::from_json_string(json)
                .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
            Some(graph)
        }
        None => None,
    };

    // Apply mutations
    let result = core_apply_mutations_create(mutations_json, existing_graph.as_ref())
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Serialize result
    serde_json::to_string(&result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize result graph: {}",
            e
        ))
    })
}

/// Register graph mutator functions with the Python module
pub fn register_module(m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(apply_mutations_from_json, m)?)?;
    m.add_function(wrap_pyfunction!(apply_mutations_create, m)?)?;
    m.add_function(wrap_pyfunction!(apply_mutations_with_extensions, m)?)?;
    m.add_function(wrap_pyfunction!(register_extension_mutation, m)?)?;
    m.add_function(wrap_pyfunction!(has_extension_mutation, m)?)?;
    m.add_function(wrap_pyfunction!(list_extension_mutations, m)?)?;
    m.add_function(wrap_pyfunction!(generate_uuid_v5, m)?)?;
    m.add_function(wrap_pyfunction!(get_mutation_schema, m)?)?;
    Ok(())
}
