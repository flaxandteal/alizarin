//! Python bindings for the JSON-based Graph Mutation API.
//!
//! This module exposes a simple JSON-based API for mutating graphs.

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

use alizarin_core::graph_mutator::{
    apply_mutations_create_from_json as core_apply_mutations_create,
    apply_mutations_from_json as core_apply_mutations,
    apply_mutations_from_json_with_extensions as core_apply_mutations_with_ext,
    build_graph_from_instructions_json as core_build_from_instructions,
    build_graph_from_instructions_with_extensions, generate_uuid_v5 as core_generate_uuid,
    get_mutation_schema as core_get_schema, parse_instructions_from_csv, ExtensionMutationHandler,
    ExtensionMutationRegistry, MutationConformance, MutationError, MutatorOptions,
};
use alizarin_core::ontology::OntologyValidator;
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
#[pyo3(name = "list_extension_mutations")]
fn get_registered_extension_mutations() -> PyResult<Vec<String>> {
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

/// Build a graph from scratch using instruction-based format.
///
/// Instructions use a CSV-friendly triple format (action, subject, object)
/// with additional params. The first instruction must be `create_model` or
/// `create_branch`.
///
/// Branches referenced by graph ID in `add_subgraph` instructions are looked
/// up from the global registry (see `register_graph`).
///
/// Args:
///     instructions_json: JSON string containing instructions and options
///
/// Example:
/// ```json
/// {
///   "instructions": [
///     {"action": "create_model", "subject": "person", "object": "", "params": {"name": "Person", "ontology_class": "E21_Person"}},
///     {"action": "add_node", "subject": "person", "object": "name", "params": {"datatype": "string", "name": "Name"}}
///   ],
///   "options": {"autocreate_card": true, "autocreate_widget": true}
/// }
/// ```
///
/// Returns:
///     The built graph as JSON string
#[pyfunction]
fn build_graph_from_instructions(instructions_json: &str) -> PyResult<String> {
    let graph = core_build_from_instructions(instructions_json)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    serde_json::to_string(&graph).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize built graph: {}",
            e
        ))
    })
}

/// Build or mutate a graph from CSV instructions.
///
/// Parses CSV text with columns `action`, `subject`, `object`, and `params.*`.
/// The first row must be `create_model`, `create_branch`, or `load_graph`.
///
/// Use `load_graph` with the graph ID as `subject` to load a registered graph
/// and apply subsequent instructions to it. Branches referenced by graph ID
/// in `add_subgraph` instructions are also looked up from the registry.
///
/// Unrecognized actions are treated as extension mutations and dispatched to
/// handlers registered via `register_extension_mutation`.
///
/// Args:
///     csv_text: CSV string with header row and instruction rows
///     autocreate_card: Whether to auto-create cards for nodegroups (default True)
///     autocreate_widget: Whether to auto-create widgets for nodes (default True)
///     ontology_validator: Optional OntologyValidator for class/property validation
///
/// Returns:
///     The built graph as JSON string
#[pyfunction]
#[pyo3(signature = (csv_text, autocreate_card=true, autocreate_widget=true, ontology_validator=None))]
fn build_graph_from_csv(
    csv_text: &str,
    autocreate_card: bool,
    autocreate_widget: bool,
    ontology_validator: Option<PyOntologyValidator>,
) -> PyResult<String> {
    let options = MutatorOptions {
        autocreate_card,
        autocreate_widget,
        ontology_validator: ontology_validator.map(|v| v.inner),
    };

    let instructions = parse_instructions_from_csv(csv_text)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Use extension registry if any extensions are registered
    let registry = EXTENSION_REGISTRY.lock().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Failed to acquire extension registry lock",
        )
    })?;

    let ext_registry = if registry.list().is_empty() {
        None
    } else {
        Some(&*registry)
    };

    let graph = build_graph_from_instructions_with_extensions(instructions, options, ext_registry)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    serde_json::to_string(&graph).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize built graph: {}",
            e
        ))
    })
}

/// Python wrapper for OntologyValidator
#[pyclass(name = "OntologyValidator")]
#[derive(Clone)]
pub struct PyOntologyValidator {
    inner: OntologyValidator,
}

impl PyOntologyValidator {
    pub fn from_inner(validator: OntologyValidator) -> Self {
        Self { inner: validator }
    }
}

#[pymethods]
impl PyOntologyValidator {
    /// Load an ontology validator from one or more RDFS/XML file paths.
    ///
    /// Args:
    ///     file_paths: List of paths to RDFS/XML ontology files.
    ///                 The base ontology should be first, extensions after.
    ///
    /// Returns:
    ///     An OntologyValidator instance
    #[new]
    fn new(file_paths: Vec<String>) -> PyResult<Self> {
        let contents: Vec<String> = file_paths
            .iter()
            .map(|p| {
                std::fs::read_to_string(p).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}: {}", p, e))
                })
            })
            .collect::<PyResult<Vec<String>>>()?;

        let refs: Vec<&str> = contents.iter().map(|s| s.as_str()).collect();
        let validator = OntologyValidator::from_rdfs_xml(&refs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

        Ok(Self { inner: validator })
    }

    /// Check if a class URI is known in the loaded ontology.
    fn is_valid_class(&self, class_uri: &str) -> bool {
        self.inner.is_valid_class(class_uri)
    }

    /// Validate that a property is valid between domain and range classes.
    ///
    /// Raises ValueError if the edge is invalid.
    fn validate_edge(&self, domain_class: &str, property: &str, range_class: &str) -> PyResult<()> {
        self.inner
            .validate_edge(domain_class, property, range_class)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }

    /// Number of known classes
    #[getter]
    fn class_count(&self) -> usize {
        self.inner.class_count()
    }
}

/// Build a graph and collections from the 3-CSV model format.
///
/// Parses graph.csv, nodes.csv, and collections.csv, validates them,
/// and builds the graph via the standard mutation pipeline.
///
/// Args:
///     graph_csv: Contents of graph.csv
///     nodes_csv: Contents of nodes.csv
///     rdm_namespace: RDM namespace string (UUID or URL) for deterministic ID generation
///     collections_csv: Contents of collections.csv (optional)
///     autocreate_card: Whether to auto-create cards for nodegroups (default True)
///     autocreate_widget: Whether to auto-create widgets for nodes (default True)
///
/// Returns:
///     JSON string containing { "graph": <graph>, "collections": [<collection>, ...] }
#[pyfunction]
#[pyo3(signature = (graph_csv, nodes_csv, rdm_namespace, collections_csv=None, autocreate_card=true, autocreate_widget=true))]
fn build_graph_from_model_csvs(
    graph_csv: &str,
    nodes_csv: &str,
    rdm_namespace: &str,
    collections_csv: Option<&str>,
    autocreate_card: bool,
    autocreate_widget: bool,
) -> PyResult<String> {
    use alizarin_core::csv_model_loader;

    let options = MutatorOptions {
        autocreate_card,
        autocreate_widget,
        ontology_validator: None,
    };

    let (graph, collections) = csv_model_loader::build_graph_from_model_csvs(
        graph_csv,
        nodes_csv,
        collections_csv,
        rdm_namespace,
        options,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

    let result = serde_json::json!({
        "graph": graph,
        "collections": collections,
    });

    serde_json::to_string(&result).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize result: {}",
            e
        ))
    })
}

/// Validate 3-CSV model files without building.
///
/// Args:
///     graph_csv: Contents of graph.csv
///     nodes_csv: Contents of nodes.csv
///     collections_csv: Contents of collections.csv (optional)
///
/// Returns:
///     JSON string containing array of { level, file, line, message } diagnostics
#[pyfunction]
#[pyo3(signature = (graph_csv, nodes_csv, collections_csv=None))]
fn validate_model_csvs(
    graph_csv: &str,
    nodes_csv: &str,
    collections_csv: Option<&str>,
) -> PyResult<String> {
    use alizarin_core::csv_model_loader;

    let diagnostics =
        csv_model_loader::validate_model_csvs_from_strings(graph_csv, nodes_csv, collections_csv);

    serde_json::to_string(&diagnostics).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize diagnostics: {}",
            e
        ))
    })
}

/// Build resource instances from a business data CSV.
///
/// Resolves node aliases to UUIDs from the graph, concept labels to UUIDs
/// from the collections. Generates deterministic UUIDs for resources and tiles.
///
/// Args:
///     csv_data: The CSV string with ResourceID as first column, node aliases as remaining columns
///     graph_json: JSON string of the built StaticGraph
///     collections_json: JSON string of the built collections array
///     default_language: Default language code (default "en")
///     strict_concepts: Whether to error on unresolved concept labels (default true)
///
/// Returns:
///     JSON string: { "business_data": { "resources": [...] } }
#[pyfunction]
#[pyo3(signature = (csv_data, graph_json, collections_json, default_language="en", strict_concepts=true))]
fn build_resources_from_business_csv(
    csv_data: &str,
    graph_json: &str,
    collections_json: &str,
    default_language: &str,
    strict_concepts: bool,
) -> PyResult<String> {
    use alizarin_core::csv_business_data_loader;

    let graph: alizarin_core::graph::StaticGraph =
        serde_json::from_str(graph_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse graph JSON: {}",
                e
            ))
        })?;

    let collections: Vec<alizarin_core::skos::SkosCollection> =
        serde_json::from_str(collections_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse collections JSON: {}",
                e
            ))
        })?;

    let options = csv_business_data_loader::BusinessDataCsvOptions {
        default_language: default_language.to_string(),
        strict_concepts,
    };

    let resources = csv_business_data_loader::build_resources_from_business_csv(
        csv_data,
        &graph,
        &collections,
        options,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

    let wrapped = csv_business_data_loader::wrap_business_data(&resources);
    serde_json::to_string(&wrapped).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize result: {}",
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
    m.add_function(wrap_pyfunction!(get_registered_extension_mutations, m)?)?;
    m.add_function(wrap_pyfunction!(generate_uuid_v5, m)?)?;
    m.add_function(wrap_pyfunction!(get_mutation_schema, m)?)?;
    m.add_function(wrap_pyfunction!(build_graph_from_instructions, m)?)?;
    m.add_function(wrap_pyfunction!(build_graph_from_csv, m)?)?;
    m.add_function(wrap_pyfunction!(build_graph_from_model_csvs, m)?)?;
    m.add_function(wrap_pyfunction!(validate_model_csvs, m)?)?;
    m.add_function(wrap_pyfunction!(build_resources_from_business_csv, m)?)?;
    m.add_class::<PyOntologyValidator>()?;
    Ok(())
}
