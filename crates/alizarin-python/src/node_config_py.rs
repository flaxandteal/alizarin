/// Python bindings for Node configuration types.
///
/// This module wraps the platform-agnostic types from alizarin-core
/// with PyO3 bindings for Python access.
use pyo3::prelude::*;
use std::collections::HashMap;

// Import core types
use alizarin_core::node_config::{
    NodeConfigBoolean as CoreNodeConfigBoolean,
    NodeConfigConcept as CoreNodeConfigConcept,
    NodeConfigDomain as CoreNodeConfigDomain,
    NodeConfigManager as CoreNodeConfigManager,
    NodeConfigReference as CoreNodeConfigReference,
    StaticDomainValue as CoreStaticDomainValue,
};

// =============================================================================
// StaticDomainValue Wrapper
// =============================================================================

/// Python wrapper for StaticDomainValue
#[derive(Clone)]
#[pyclass(name = "RustStaticDomainValue")]
pub struct PyStaticDomainValue {
    inner: CoreStaticDomainValue,
}

impl From<CoreStaticDomainValue> for PyStaticDomainValue {
    fn from(inner: CoreStaticDomainValue) -> Self {
        Self { inner }
    }
}

impl From<&CoreStaticDomainValue> for PyStaticDomainValue {
    fn from(inner: &CoreStaticDomainValue) -> Self {
        Self { inner: inner.clone() }
    }
}

#[pymethods]
impl PyStaticDomainValue {
    #[new]
    fn new(id: String, selected: bool, text: HashMap<String, String>) -> Self {
        Self {
            inner: CoreStaticDomainValue::new(id, selected, text),
        }
    }

    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }

    #[getter]
    fn selected(&self) -> bool {
        self.inner.selected
    }

    #[getter]
    fn text(&self) -> HashMap<String, String> {
        self.inner.text.clone()
    }

    /// Get text for a specific language
    fn lang(&self, language: &str) -> Option<String> {
        self.inner.lang(language).map(|s| s.to_string())
    }

    fn __str__(&self) -> String {
        self.inner.display().to_string()
    }

    fn __repr__(&self) -> String {
        format!("RustStaticDomainValue(id={}, text={:?})", self.inner.id, self.inner.text)
    }

    /// Convert to Python dict
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let json_module = py.import_bound("json")?;
        let py_dict = json_module.call_method1("loads", (json_str,))?;
        Ok(py_dict.to_object(py))
    }
}

// =============================================================================
// NodeConfigBoolean Wrapper
// =============================================================================

#[derive(Clone)]
#[pyclass(name = "RustNodeConfigBoolean")]
pub struct PyNodeConfigBoolean {
    inner: CoreNodeConfigBoolean,
}

impl From<CoreNodeConfigBoolean> for PyNodeConfigBoolean {
    fn from(inner: CoreNodeConfigBoolean) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigBoolean> for PyNodeConfigBoolean {
    fn from(inner: &CoreNodeConfigBoolean) -> Self {
        Self { inner: inner.clone() }
    }
}

#[pymethods]
impl PyNodeConfigBoolean {
    #[new]
    fn new(true_label: HashMap<String, String>, false_label: HashMap<String, String>) -> Self {
        Self {
            inner: CoreNodeConfigBoolean::new(true_label, false_label),
        }
    }

    #[getter]
    fn true_label(&self) -> HashMap<String, String> {
        self.inner.true_label.clone()
    }

    #[getter]
    fn false_label(&self) -> HashMap<String, String> {
        self.inner.false_label.clone()
    }

    /// Get label for a boolean value in a specific language
    fn get_label(&self, value: bool, language: &str) -> Option<String> {
        self.inner.get_label(value, language).map(|s| s.to_string())
    }
}

// =============================================================================
// NodeConfigConcept Wrapper
// =============================================================================

#[derive(Clone)]
#[pyclass(name = "RustNodeConfigConcept")]
pub struct PyNodeConfigConcept {
    inner: CoreNodeConfigConcept,
}

impl From<CoreNodeConfigConcept> for PyNodeConfigConcept {
    fn from(inner: CoreNodeConfigConcept) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigConcept> for PyNodeConfigConcept {
    fn from(inner: &CoreNodeConfigConcept) -> Self {
        Self { inner: inner.clone() }
    }
}

#[pymethods]
impl PyNodeConfigConcept {
    #[new]
    fn new(rdm_collection: String) -> Self {
        Self {
            inner: CoreNodeConfigConcept::new(rdm_collection),
        }
    }

    #[getter]
    fn rdm_collection(&self) -> &str {
        &self.inner.rdm_collection
    }
}

// =============================================================================
// NodeConfigReference Wrapper
// =============================================================================

#[derive(Clone)]
#[pyclass(name = "RustNodeConfigReference")]
pub struct PyNodeConfigReference {
    inner: CoreNodeConfigReference,
}

impl From<CoreNodeConfigReference> for PyNodeConfigReference {
    fn from(inner: CoreNodeConfigReference) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigReference> for PyNodeConfigReference {
    fn from(inner: &CoreNodeConfigReference) -> Self {
        Self { inner: inner.clone() }
    }
}

#[pymethods]
impl PyNodeConfigReference {
    #[new]
    fn new(controlled_list: String, rdm_collection: String, multi_value: bool) -> Self {
        Self {
            inner: CoreNodeConfigReference::new(controlled_list, rdm_collection, multi_value),
        }
    }

    #[getter]
    fn controlled_list(&self) -> &str {
        &self.inner.controlled_list
    }

    #[getter]
    fn rdm_collection(&self) -> &str {
        &self.inner.rdm_collection
    }

    #[getter]
    fn multi_value(&self) -> bool {
        self.inner.multi_value
    }

    /// Get the collection ID (controlledList or rdmCollection)
    fn get_collection_id(&self) -> Option<String> {
        self.inner.get_collection_id().map(|s| s.to_string())
    }
}

// =============================================================================
// NodeConfigDomain Wrapper
// =============================================================================

#[derive(Clone)]
#[pyclass(name = "RustNodeConfigDomain")]
pub struct PyNodeConfigDomain {
    inner: CoreNodeConfigDomain,
}

impl From<CoreNodeConfigDomain> for PyNodeConfigDomain {
    fn from(inner: CoreNodeConfigDomain) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigDomain> for PyNodeConfigDomain {
    fn from(inner: &CoreNodeConfigDomain) -> Self {
        Self { inner: inner.clone() }
    }
}

#[pymethods]
impl PyNodeConfigDomain {
    #[new]
    fn new(options: Vec<PyStaticDomainValue>) -> Self {
        let core_options: Vec<CoreStaticDomainValue> = options
            .into_iter()
            .map(|o| o.inner)
            .collect();
        Self {
            inner: CoreNodeConfigDomain::new(core_options),
        }
    }

    #[getter]
    fn options(&self) -> Vec<PyStaticDomainValue> {
        self.inner.options.iter().map(PyStaticDomainValue::from).collect()
    }

    /// Get the selected option (if any)
    fn get_selected(&self) -> Option<PyStaticDomainValue> {
        self.inner.get_selected().map(PyStaticDomainValue::from)
    }

    /// Find option by ID
    fn value_from_id(&self, id: &str) -> Option<PyStaticDomainValue> {
        self.inner.value_from_id(id).map(PyStaticDomainValue::from)
    }

    /// Get all option IDs
    fn get_option_ids(&self) -> Vec<String> {
        self.inner.get_option_ids().iter().map(|s| s.to_string()).collect()
    }
}

// =============================================================================
// NodeConfigManager Wrapper
// =============================================================================

#[pyclass(name = "RustNodeConfigManager")]
pub struct PyNodeConfigManager {
    inner: CoreNodeConfigManager,
}

#[pymethods]
impl PyNodeConfigManager {
    #[new]
    fn new() -> Self {
        Self {
            inner: CoreNodeConfigManager::new(),
        }
    }

    /// Build configs from a graph JSON string
    fn load_graph_json(&mut self, graph_json: &str) -> PyResult<()> {
        self.inner.from_graph_json(graph_json)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    /// Get boolean config for a node
    fn get_boolean(&self, nodeid: &str) -> Option<PyNodeConfigBoolean> {
        self.inner.get_boolean(nodeid).map(PyNodeConfigBoolean::from)
    }

    /// Get concept config for a node
    fn get_concept(&self, nodeid: &str) -> Option<PyNodeConfigConcept> {
        self.inner.get_concept(nodeid).map(PyNodeConfigConcept::from)
    }

    /// Get reference config for a node
    fn get_reference(&self, nodeid: &str) -> Option<PyNodeConfigReference> {
        self.inner.get_reference(nodeid).map(PyNodeConfigReference::from)
    }

    /// Get domain config for a node
    fn get_domain(&self, nodeid: &str) -> Option<PyNodeConfigDomain> {
        self.inner.get_domain(nodeid).map(PyNodeConfigDomain::from)
    }

    /// Look up domain value by ID
    fn lookup_domain_value(&self, nodeid: &str, value_id: &str) -> Option<PyStaticDomainValue> {
        self.inner.lookup_domain_value(nodeid, value_id).map(PyStaticDomainValue::from)
    }

    /// Check if a node has config
    fn has_config(&self, nodeid: &str) -> bool {
        self.inner.has_config(nodeid)
    }

    /// Get the datatype for which this node has config
    fn get_config_type(&self, nodeid: &str) -> Option<String> {
        self.inner.get_config_type(nodeid).map(|s| s.to_string())
    }

    /// Clear all cached configs
    fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get number of cached configs
    fn __len__(&self) -> usize {
        self.inner.len()
    }
}

impl PyNodeConfigManager {
    /// Get inner manager reference (for Rust-side access)
    pub fn inner(&self) -> &CoreNodeConfigManager {
        &self.inner
    }
}

// =============================================================================
// Module Registration
// =============================================================================

/// Register node config types with the Python module
pub fn register_module(m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStaticDomainValue>()?;
    m.add_class::<PyNodeConfigBoolean>()?;
    m.add_class::<PyNodeConfigConcept>()?;
    m.add_class::<PyNodeConfigReference>()?;
    m.add_class::<PyNodeConfigDomain>()?;
    m.add_class::<PyNodeConfigManager>()?;
    Ok(())
}
