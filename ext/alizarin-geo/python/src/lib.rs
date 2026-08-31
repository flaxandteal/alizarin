//! Alizarin Geo Extension (Python binding)
//!
//! Registers a validate-only handler for the `geojson-feature-collection`
//! datatype. Coercion stays in core — this extension only *validates*, so the
//! host dispatches validation here while continuing to coerce itself.
//!
//! Validation runs in Rust (`alizarin-geo-core`). The tile value is passed to
//! the handler **by pointer** (`*const serde_json::Value`) — nothing is
//! serialized on the hot path. This is sound because the host confirms the
//! extension [`AbiFingerprint`](alizarin_extension_api::AbiFingerprint) when the
//! capsule is registered, guaranteeing both `.so`s agree on the layout of
//! `serde_json::Value` and `TypeHandlerInfo`.

// Re-export core so downstream users needn't depend on alizarin-geo-core directly.
pub use alizarin_geo_core::{
    get_coord_limit, reset_coord_limit, set_coord_limit, validate_geojson, DATATYPE_NAME,
    DEFAULT_COORD_LIMIT,
};


// =============================================================================
// Python Module (C ABI + pyo3)
// =============================================================================

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use std::ffi::c_void;

    use pyo3::types::{PyCapsule, PyModule};
    use pyo3::{pyfunction, pymodule, wrap_pyfunction, Bound, Py, PyErr, PyResult, Python};

    /// Capsule name: null-terminated and `'static` so the pointer handed to
    /// `PyCapsule_New` (which does not copy it) outlives the capsule.
    static CAPSULE_NAME: &[u8] = b"alizarin_geo.geo_handler\0";

    /// Get the type handler capsule for registration with alizarin. The
    /// `TypeHandlerInfo` and its extern-"C" wrappers live in geo-core
    /// ([`alizarin_geo_core::c_abi`]), shared with the napi binding.
    #[pyfunction]
    pub fn get_geo_handler_capsule(py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        let ptr = alizarin_geo_core::c_abi::geo_handler_type_info();

        // SAFETY: `ptr` points at geo-core's 'static HANDLER_INFO, and
        // CAPSULE_NAME is 'static and null-terminated.
        unsafe {
            let capsule = pyo3::ffi::PyCapsule_New(
                ptr as *mut c_void,
                CAPSULE_NAME.as_ptr() as *const std::os::raw::c_char,
                None,
            );

            if capsule.is_null() {
                return Err(PyErr::fetch(py));
            }

            Ok(Py::from_owned_ptr(py, capsule))
        }
    }

    /// Set the maximum coordinate count per feature collection, or `None` to
    /// disable the check. Applies to the geo-core linked into this extension —
    /// i.e. the one that actually validates. Defaults to
    /// [`DEFAULT_COORD_LIMIT`] (1500, matching Arches' Elasticsearch limit).
    #[pyfunction]
    #[pyo3(name = "set_coord_limit")]
    pub fn py_set_coord_limit(limit: Option<usize>) {
        set_coord_limit(limit);
    }

    /// Get the current coordinate limit, or `None` if the check is disabled.
    #[pyfunction]
    #[pyo3(name = "get_coord_limit")]
    pub fn py_get_coord_limit() -> Option<usize> {
        get_coord_limit()
    }

    /// Reset the coordinate limit to [`DEFAULT_COORD_LIMIT`].
    #[pyfunction]
    #[pyo3(name = "reset_coord_limit")]
    pub fn py_reset_coord_limit() {
        reset_coord_limit();
    }

    /// Python module definition.
    #[pymodule]
    pub fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(get_geo_handler_capsule, m)?)?;
        m.add_function(wrap_pyfunction!(py_set_coord_limit, m)?)?;
        m.add_function(wrap_pyfunction!(py_get_coord_limit, m)?)?;
        m.add_function(wrap_pyfunction!(py_reset_coord_limit, m)?)?;
        Ok(())
    }
}

