//! Python bindings for type coercion functions.
//!
//! These wrappers expose the platform-agnostic type coercion logic
//! from alizarin-core to Python via PyO3.
use crate::python_json::{json_to_python, python_to_json};
use alizarin_core::type_coercion::{
    // Phase 3
    coerce_boolean,
    coerce_concept_list,
    // Phase 4
    coerce_concept_value,
    // Phase 1
    coerce_date,
    coerce_domain_value,
    coerce_domain_value_list,
    coerce_edtf,
    // Phase 2
    coerce_geojson,
    coerce_non_localized_string,
    coerce_number,
    // Phase 5
    coerce_resource_instance,
    coerce_resource_instance_list,
    coerce_string,
    coerce_url,
    // Dispatcher
    coerce_value,
    // Language config
    get_current_language,
    set_current_language,
    CoercionResult as CoreCoercionResult,
};
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Register type coercion functions with the Python module
pub fn register_module(m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCoercionResult>()?;
    // Phase 1: Simple scalars
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_number, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_non_localized_string, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_edtf, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_date, m)?)?;
    // Phase 2: Dict types
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_string, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_url, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_geojson, m)?)?;
    // Phase 3: Config-dependent types
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_boolean, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_domain_value, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_domain_value_list, m)?)?;
    // Phase 4: RDM-dependent types
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_concept_value, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_concept_list, m)?)?;
    // Phase 5: Format normalization
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_resource_instance, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_resource_instance_list, m)?)?;
    // Dispatcher
    m.add_function(pyo3::wrap_pyfunction!(py_coerce_value, m)?)?;
    // Language config
    m.add_function(pyo3::wrap_pyfunction!(py_get_current_language, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_set_current_language, m)?)?;
    Ok(())
}

/// Python wrapper for CoercionResult
#[pyclass(name = "CoercionResult")]
#[derive(Clone)]
pub struct PyCoercionResult {
    inner: CoreCoercionResult,
}

#[pymethods]
impl PyCoercionResult {
    /// Get the tile data (value for tile.data[nodeid])
    #[getter]
    fn tile_data(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_python(py, &self.inner.tile_data)
    }

    /// Get the display value (for ViewModel construction)
    #[getter]
    fn display_value(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_python(py, &self.inner.display_value)
    }

    /// Check if the result is an error
    #[getter]
    fn is_error(&self) -> bool {
        self.inner.is_error()
    }

    /// Get the error message (if any)
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }

    /// Check if the result is null (no value, no error)
    #[getter]
    fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    fn __repr__(&self) -> String {
        if let Some(ref error) = self.inner.error {
            format!("CoercionResult(error='{}')", error)
        } else if self.inner.is_null() {
            "CoercionResult(null)".to_string()
        } else {
            format!("CoercionResult(tile_data={:?})", self.inner.tile_data)
        }
    }
}

/// Coerce a value to a number
#[pyfunction]
#[pyo3(name = "coerce_number")]
pub fn py_coerce_number(py: Python<'_>, value: &PyAny) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    Ok(PyCoercionResult {
        inner: coerce_number(&val),
    })
}

/// Coerce a value to a non-localized string
#[pyfunction]
#[pyo3(name = "coerce_non_localized_string")]
pub fn py_coerce_non_localized_string(py: Python<'_>, value: &PyAny) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    Ok(PyCoercionResult {
        inner: coerce_non_localized_string(&val),
    })
}

/// Coerce a value to an EDTF string
#[pyfunction]
#[pyo3(name = "coerce_edtf")]
pub fn py_coerce_edtf(py: Python<'_>, value: &PyAny) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    Ok(PyCoercionResult {
        inner: coerce_edtf(&val),
    })
}

/// Coerce a value to a date
#[pyfunction]
#[pyo3(name = "coerce_date")]
pub fn py_coerce_date(py: Python<'_>, value: &PyAny) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    Ok(PyCoercionResult {
        inner: coerce_date(&val),
    })
}

// =============================================================================
// Phase 2: Dict types
// =============================================================================

/// Coerce a value to a localized string
#[pyfunction]
#[pyo3(name = "coerce_string")]
pub fn py_coerce_string(
    py: Python<'_>,
    value: &PyAny,
    language: Option<&str>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    Ok(PyCoercionResult {
        inner: coerce_string(&val, language),
    })
}

/// Coerce a value to a URL
#[pyfunction]
#[pyo3(name = "coerce_url")]
pub fn py_coerce_url(py: Python<'_>, value: &PyAny) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    Ok(PyCoercionResult {
        inner: coerce_url(&val),
    })
}

/// Coerce a value to GeoJSON
#[pyfunction]
#[pyo3(name = "coerce_geojson")]
pub fn py_coerce_geojson(py: Python<'_>, value: &PyAny) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    Ok(PyCoercionResult {
        inner: coerce_geojson(&val),
    })
}

// =============================================================================
// Phase 3: Config-dependent types
// =============================================================================

/// Coerce a value to a boolean
#[pyfunction]
#[pyo3(name = "coerce_boolean")]
pub fn py_coerce_boolean(
    py: Python<'_>,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_boolean(&val, cfg.as_ref()),
    })
}

/// Coerce a value to a domain value
#[pyfunction]
#[pyo3(name = "coerce_domain_value")]
pub fn py_coerce_domain_value(
    py: Python<'_>,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_domain_value(&val, cfg.as_ref()),
    })
}

/// Coerce a value to a domain value list
#[pyfunction]
#[pyo3(name = "coerce_domain_value_list")]
pub fn py_coerce_domain_value_list(
    py: Python<'_>,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_domain_value_list(&val, cfg.as_ref()),
    })
}

// =============================================================================
// Phase 4: RDM-dependent types
// =============================================================================

/// Coerce a value to a concept value
#[pyfunction]
#[pyo3(name = "coerce_concept_value")]
pub fn py_coerce_concept_value(
    py: Python<'_>,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_concept_value(&val, cfg.as_ref()),
    })
}

/// Coerce a value to a concept list
#[pyfunction]
#[pyo3(name = "coerce_concept_list")]
pub fn py_coerce_concept_list(
    py: Python<'_>,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_concept_list(&val, cfg.as_ref()),
    })
}

// =============================================================================
// Phase 5: Format normalization
// =============================================================================

/// Coerce a value to a resource instance
#[pyfunction]
#[pyo3(name = "coerce_resource_instance")]
pub fn py_coerce_resource_instance(
    py: Python<'_>,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_resource_instance(&val, cfg.as_ref()),
    })
}

/// Coerce a value to a resource instance list
#[pyfunction]
#[pyo3(name = "coerce_resource_instance_list")]
pub fn py_coerce_resource_instance_list(
    py: Python<'_>,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_resource_instance_list(&val, cfg.as_ref()),
    })
}

// =============================================================================
// Language configuration
// =============================================================================

/// Get the current language setting
#[pyfunction]
#[pyo3(name = "get_current_language")]
pub fn py_get_current_language() -> String {
    get_current_language()
}

/// Set the current language
#[pyfunction]
#[pyo3(name = "set_current_language")]
pub fn py_set_current_language(language: &str) {
    set_current_language(language);
}

/// Coerce a value based on datatype
#[pyfunction]
#[pyo3(name = "coerce_value")]
pub fn py_coerce_value(
    py: Python<'_>,
    datatype: &str,
    value: &PyAny,
    config: Option<&PyAny>,
) -> PyResult<PyCoercionResult> {
    let val = python_to_json(py, value)?;
    let cfg = match config {
        Some(c) => Some(python_to_json(py, c)?),
        None => None,
    };
    Ok(PyCoercionResult {
        inner: coerce_value(datatype, &val, cfg.as_ref()),
    })
}
