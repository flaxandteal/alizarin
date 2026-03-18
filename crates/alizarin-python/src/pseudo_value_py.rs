#![allow(deprecated)]
use crate::python_json::{json_to_python, python_to_json};
/// PyO3 bindings for PseudoValueCore/PseudoListCore from alizarin-core
///
/// These provide Rust-backed pseudo values for Python, matching the
/// architecture used by JS/TS (which wraps WasmPseudoValue).
///
/// Key benefits:
/// - Single source of truth for matching_entries logic
/// - Feature parity with JS/TS implementation
/// - Performance: filtering happens in Rust
use pyo3::prelude::*;
use std::sync::Arc;

use alizarin_core::{
    PseudoListCore, PseudoValueCore, StaticNode as CoreStaticNode, StaticTile as CoreStaticTile,
};

// =============================================================================
// PyPseudoValue - Wraps PseudoValueCore
// =============================================================================

/// Python wrapper for PseudoValueCore.
///
/// Provides Rust-backed pseudo values with the same API as JS/TS.
/// The Python pseudos.py PseudoValue class should wrap this.
#[pyclass(name = "RustPseudoValue")]
#[derive(Clone)]
pub struct PyPseudoValue {
    inner: PseudoValueCore,
}

#[pymethods]
impl PyPseudoValue {
    /// Create a new PyPseudoValue from node and tile JSON
    #[new]
    #[pyo3(signature = (node_json, tile_json=None, tile_data_json=None, child_node_ids=None))]
    fn new(
        node_json: &str,
        tile_json: Option<&str>,
        tile_data_json: Option<&str>,
        child_node_ids: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let node: CoreStaticNode = serde_json::from_str(node_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse node: {}", e))
        })?;

        let tile: Option<Arc<CoreStaticTile>> = match tile_json {
            Some(json) => {
                let t: CoreStaticTile = serde_json::from_str(json).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to parse tile: {}",
                        e
                    ))
                })?;
                Some(Arc::new(t))
            }
            None => None,
        };

        let tile_data: Option<serde_json::Value> = match tile_data_json {
            Some(json) => Some(serde_json::from_str(json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse tile_data: {}",
                    e
                ))
            })?),
            None => None,
        };

        let core = PseudoValueCore::from_node_and_tile(
            Arc::new(node),
            tile,
            tile_data,
            child_node_ids.unwrap_or_default(),
        );

        Ok(PyPseudoValue { inner: core })
    }

    // =========================================================================
    // Property accessors
    // =========================================================================

    /// Get the node ID
    #[getter]
    fn node_id(&self) -> String {
        self.inner.node.nodeid.clone()
    }

    /// Get the node alias
    #[getter]
    fn alias(&self) -> Option<String> {
        self.inner.node.alias.clone()
    }

    /// Get the node name
    #[getter]
    fn name(&self) -> String {
        self.inner.node.name.clone()
    }

    /// Get the datatype (may be "semantic" for inner nodes)
    #[getter]
    fn datatype(&self) -> String {
        self.inner.datatype().to_string()
    }

    /// Get the nodegroup ID
    #[getter]
    fn nodegroup_id(&self) -> Option<String> {
        self.inner.node.nodegroup_id.clone()
    }

    /// Check if this is an outer node (has inner)
    #[getter]
    fn is_outer(&self) -> bool {
        self.inner.is_outer()
    }

    /// Check if this is an inner node (datatype overridden to semantic)
    #[getter]
    fn is_inner(&self) -> bool {
        self.inner.is_inner
    }

    /// Check if this is a collector node
    #[getter]
    fn is_collector(&self) -> bool {
        self.inner.is_collector
    }

    /// Check if this value has a tile
    #[getter]
    fn has_tile(&self) -> bool {
        self.inner.has_tile()
    }

    /// Get tile ID if present
    #[getter]
    fn tile_id(&self) -> Option<String> {
        self.inner.tile_id()
    }

    /// Check if this tile is independent (not inherited from parent)
    #[getter]
    fn independent(&self) -> bool {
        self.inner.independent
    }

    /// Get child node IDs
    #[getter]
    fn child_node_ids(&self) -> Vec<String> {
        self.inner.child_node_ids.clone()
    }

    /// Get the tile as JSON string (if present)
    fn get_tile_json(&self) -> PyResult<Option<String>> {
        match &self.inner.tile {
            Some(tile) => {
                let json = serde_json::to_string(tile.as_ref()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize tile: {}",
                        e
                    ))
                })?;
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }

    /// Get the tile data as JSON string (if present)
    fn get_tile_data_json(&self) -> PyResult<Option<String>> {
        match &self.inner.tile_data {
            Some(data) => {
                let json = serde_json::to_string(data).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize tile_data: {}",
                        e
                    ))
                })?;
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }

    /// Get the tile data as a Python object
    fn get_tile_data(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner.tile_data {
            Some(data) => json_to_python(py, data),
            None => Ok(py.None()),
        }
    }

    /// Get the node as JSON string
    fn get_node_json(&self) -> PyResult<String> {
        serde_json::to_string(self.inner.node.as_ref()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize node: {}",
                e
            ))
        })
    }

    /// Get the inner PseudoValue (if this is an outer node)
    fn get_inner(&self) -> Option<PyPseudoValue> {
        self.inner.inner.as_ref().map(|inner| PyPseudoValue {
            inner: (**inner).clone(),
        })
    }

    /// Set the inner PseudoValue (for outer/inner pattern)
    fn set_inner(&mut self, inner: &PyPseudoValue) {
        self.inner.inner = Some(Box::new(inner.inner.clone()));
    }

    /// Mark this as an inner node
    fn set_is_inner(&mut self, is_inner: bool) {
        self.inner.is_inner = is_inner;
    }

    /// Set tile data from JSON
    fn set_tile_data_json(&mut self, json: &str) -> PyResult<()> {
        let data: serde_json::Value = serde_json::from_str(json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse tile_data: {}",
                e
            ))
        })?;
        self.inner.tile_data = Some(data);
        Ok(())
    }

    /// Set tile data from Python object
    fn set_tile_data(&mut self, py: Python<'_>, data: &PyAny) -> PyResult<()> {
        let json_value = python_to_json(py, data)?;
        self.inner.tile_data = Some(json_value);
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!(
            "RustPseudoValue(alias={:?}, datatype={}, has_tile={})",
            self.inner.node_alias(),
            self.inner.datatype(),
            self.inner.has_tile()
        )
    }
}

impl PyPseudoValue {
    /// Create from an existing PseudoValueCore (internal use)
    pub fn from_core(core: PseudoValueCore) -> Self {
        PyPseudoValue { inner: core }
    }

    /// Get a reference to the inner core (internal use)
    pub fn core(&self) -> &PseudoValueCore {
        &self.inner
    }

    /// Get a mutable reference to the inner core (internal use)
    pub fn core_mut(&mut self) -> &mut PseudoValueCore {
        &mut self.inner
    }
}

// =============================================================================
// PyPseudoList - Wraps PseudoListCore
// =============================================================================

/// Python wrapper for PseudoListCore.
///
/// Provides Rust-backed pseudo list with matching_entries from Rust.
#[pyclass(name = "RustPseudoList")]
#[derive(Clone)]
pub struct PyPseudoList {
    inner: PseudoListCore,
}

#[pymethods]
impl PyPseudoList {
    /// Create a new empty PyPseudoList
    #[new]
    #[pyo3(signature = (alias, is_single=false))]
    fn new(alias: String, is_single: bool) -> Self {
        PyPseudoList {
            inner: PseudoListCore::new_with_cardinality(alias, is_single),
        }
    }

    /// Create from a list of PyPseudoValue objects
    #[staticmethod]
    #[pyo3(signature = (alias, values, is_single=false))]
    fn from_values(alias: String, values: Vec<PyPseudoValue>, is_single: bool) -> Self {
        let cores: Vec<PseudoValueCore> = values.into_iter().map(|v| v.inner).collect();
        PyPseudoList {
            inner: PseudoListCore::from_values_with_cardinality(alias, cores, is_single),
        }
    }

    /// Get the node alias
    #[getter]
    fn alias(&self) -> String {
        self.inner.node_alias.clone()
    }

    /// Check if this list represents a cardinality-1 node
    #[getter]
    fn is_single(&self) -> bool {
        self.inner.is_single
    }

    /// Set is_single flag
    #[setter]
    fn set_is_single(&mut self, value: bool) {
        self.inner.is_single = value;
    }

    /// Check if the list has been fully loaded
    #[getter]
    fn is_loaded(&self) -> bool {
        self.inner.is_loaded
    }

    /// Get all values
    fn get_all_values(&self) -> Vec<PyPseudoValue> {
        self.inner
            .values
            .iter()
            .map(|v| PyPseudoValue { inner: v.clone() })
            .collect()
    }

    /// Add a value to the list
    fn add_value(&mut self, value: PyPseudoValue) {
        self.inner.values.push(value.inner);
    }

    /// Get the number of values
    fn __len__(&self) -> usize {
        self.inner.values.len()
    }

    /// Get a value by index
    fn __getitem__(&self, index: usize) -> PyResult<PyPseudoValue> {
        self.inner
            .values
            .get(index)
            .map(|v| PyPseudoValue { inner: v.clone() })
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyIndexError, _>("Index out of range"))
    }

    /// Filter entries matching the given tile and nodegroup context.
    ///
    /// This is the CRITICAL method that was previously duplicated in Python.
    /// Now it calls the Rust implementation directly.
    ///
    /// Args:
    ///     parent_tile_id: The parent tile ID to match against (or None for root)
    ///     nodegroup_id: The nodegroup ID to match against
    ///     parent_nodegroup_id: The parent's nodegroup ID (used to distinguish same vs different nodegroup children)
    ///
    /// Returns:
    ///     List of matching RustPseudoValue objects
    #[pyo3(signature = (parent_tile_id=None, nodegroup_id=None, parent_nodegroup_id=None))]
    fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>,
        parent_nodegroup_id: Option<String>,
    ) -> Vec<PyPseudoValue> {
        self.inner
            .matching_entries(parent_tile_id, nodegroup_id, parent_nodegroup_id)
            .into_iter()
            .map(|v| PyPseudoValue { inner: v.clone() })
            .collect()
    }

    /// Merge another list into this one
    fn merge(&mut self, other: &PyPseudoList) {
        let other_clone = other.inner.clone();
        self.inner.merge(other_clone);
    }

    fn __repr__(&self) -> String {
        format!(
            "RustPseudoList(alias={}, len={}, is_single={})",
            self.inner.node_alias,
            self.inner.values.len(),
            self.inner.is_single
        )
    }

    /// Iterate over values
    fn __iter__(slf: PyRef<'_, Self>) -> PyPseudoListIterator {
        PyPseudoListIterator {
            values: slf.inner.values.clone(),
            index: 0,
        }
    }
}

impl PyPseudoList {
    /// Create from an existing PseudoListCore (internal use)
    pub fn from_core(core: PseudoListCore) -> Self {
        PyPseudoList { inner: core }
    }

    /// Get a reference to the inner core (internal use)
    pub fn core(&self) -> &PseudoListCore {
        &self.inner
    }
}

// =============================================================================
// Iterator for PyPseudoList
// =============================================================================

#[pyclass]
pub struct PyPseudoListIterator {
    values: Vec<PseudoValueCore>,
    index: usize,
}

#[pymethods]
impl PyPseudoListIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyPseudoValue> {
        if slf.index < slf.values.len() {
            let value = PyPseudoValue {
                inner: slf.values[slf.index].clone(),
            };
            slf.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

// =============================================================================
// Module Registration
// =============================================================================

/// Register pseudo value types with the Python module
pub fn register_module(m: &pyo3::types::PyModule) -> PyResult<()> {
    m.add_class::<PyPseudoValue>()?;
    m.add_class::<PyPseudoList>()?;
    m.add_class::<PyPseudoListIterator>()?;
    Ok(())
}
