//! Python bindings for type coercion functions.
//!
//! These wrappers expose the platform-agnostic type coercion logic
//! from alizarin-core to Python via PyO3.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use alizarin_core::type_coercion::{
    // Phase 1
    coerce_date, coerce_edtf, coerce_non_localized_string, coerce_number,
    // Phase 2
    coerce_geojson, coerce_string, coerce_url,
    // Phase 3
    coerce_boolean, coerce_domain_value, coerce_domain_value_list,
    // Phase 4
    coerce_concept_value, coerce_concept_list,
    // Phase 5
    coerce_resource_instance, coerce_resource_instance_list,
    // Dispatcher
    coerce_value, CoercionResult as CoreCoercionResult,
    // Language config
    get_current_language, set_current_language,
};

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
pub fn py_coerce_string(py: Python<'_>, value: &PyAny, language: Option<&str>) -> PyResult<PyCoercionResult> {
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
pub fn py_coerce_boolean(py: Python<'_>, value: &PyAny, config: Option<&PyAny>) -> PyResult<PyCoercionResult> {
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
pub fn py_coerce_domain_value(py: Python<'_>, value: &PyAny, config: Option<&PyAny>) -> PyResult<PyCoercionResult> {
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
pub fn py_coerce_domain_value_list(py: Python<'_>, value: &PyAny, config: Option<&PyAny>) -> PyResult<PyCoercionResult> {
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
pub fn py_coerce_concept_value(py: Python<'_>, value: &PyAny, config: Option<&PyAny>) -> PyResult<PyCoercionResult> {
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
pub fn py_coerce_concept_list(py: Python<'_>, value: &PyAny, config: Option<&PyAny>) -> PyResult<PyCoercionResult> {
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
pub fn py_coerce_resource_instance(py: Python<'_>, value: &PyAny, config: Option<&PyAny>) -> PyResult<PyCoercionResult> {
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
pub fn py_coerce_resource_instance_list(py: Python<'_>, value: &PyAny, config: Option<&PyAny>) -> PyResult<PyCoercionResult> {
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

// =============================================================================
// Helper functions for Python <-> JSON conversion
// =============================================================================

/// Convert a Python object to serde_json::Value
fn python_to_json(py: Python<'_>, obj: &PyAny) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        return Ok(serde_json::Value::Null);
    }

    // Try extracting as primitive types first
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }
    if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(serde_json::Value::Number(n));
        }
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Cannot convert NaN or Infinity to JSON",
        ));
    }
    if let Ok(s) = obj.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }

    // Try as list
    if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(python_to_json(py, item)?);
        }
        return Ok(serde_json::Value::Array(arr));
    }

    // Try as dict
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, python_to_json(py, v)?);
        }
        return Ok(serde_json::Value::Object(map));
    }

    // Fall back to JSON serialization via Python's json module
    let json_module = py.import("json")?;
    let json_str: String = json_module.call_method1("dumps", (obj,))?.extract()?;
    serde_json::from_str(&json_str).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })
}

/// Convert serde_json::Value to a Python object
fn json_to_python(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.to_object(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Err(pyo3::exceptions::PyValueError::new_err("Invalid number"))
            }
        }
        serde_json::Value::String(s) => Ok(s.to_object(py)),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_to_python(py, item)?)?;
            }
            Ok(list.to_object(py))
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_python(py, v)?)?;
            }
            Ok(dict.to_object(py))
        }
    }
}
