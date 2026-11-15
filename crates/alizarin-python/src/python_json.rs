//! Shared utilities for converting between Python objects and serde_json::Value.
//!
//! These helpers are used by multiple Python binding modules to bridge
//! between PyO3's Python types and Rust's serde_json types.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Convert a Python object to serde_json::Value
///
/// Handles conversion of Python primitives, lists, and dicts to JSON.
/// Falls back to Python's json.dumps() for complex types. A list or dict that
/// contains itself raises ValueError, as json.dumps does, rather than recursing
/// until the process overflows its stack.
pub fn python_to_json(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    to_json_tracked(py, obj, &mut Vec::new())
}

/// `python_to_json`, with `open` the lists and dicts being converted around
/// `obj` (by address), so a cycle is caught.
fn to_json_tracked(
    py: Python<'_>,
    obj: &Bound<'_, PyAny>,
    open: &mut Vec<usize>,
) -> PyResult<serde_json::Value> {
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

    let is_list = obj.downcast::<PyList>().is_ok();
    let is_dict = !is_list && obj.downcast::<PyDict>().is_ok();
    if is_list || is_dict {
        let address = obj.as_ptr() as usize;
        if open.contains(&address) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Circular reference detected",
            ));
        }
        open.push(address);
        let converted = if let Ok(list) = obj.downcast::<PyList>() {
            let mut arr = Vec::new();
            for item in list.iter() {
                arr.push(to_json_tracked(py, &item, open)?);
            }
            serde_json::Value::Array(arr)
        } else {
            let mut map = serde_json::Map::new();
            for (k, v) in obj.downcast::<PyDict>()?.iter() {
                let key: String = k.extract()?;
                map.insert(key, to_json_tracked(py, &v, open)?);
            }
            serde_json::Value::Object(map)
        };
        open.pop();
        return Ok(converted);
    }

    // Fall back to JSON serialization via Python's json module
    let json_module = py.import_bound("json")?;
    let json_str: String = json_module.call_method1("dumps", (obj,))?.extract()?;
    serde_json::from_str(&json_str).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })
}

/// Convert serde_json::Value to a Python object
///
/// Recursively converts JSON values to their Python equivalents.
pub fn json_to_python(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
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
            let list = PyList::empty_bound(py);
            for item in arr {
                list.append(json_to_python(py, item)?)?;
            }
            Ok(list.to_object(py))
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new_bound(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_python(py, v)?)?;
            }
            Ok(dict.to_object(py))
        }
    }
}
