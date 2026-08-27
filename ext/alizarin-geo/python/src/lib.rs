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

use std::ffi::c_void;

use alizarin_extension_api::ValidateResult;

/// C ABI validation function for `geojson-feature-collection`.
///
/// `value_ptr` and `config_ptr` are `*const serde_json::Value` (config is null
/// when the node has no config). The value is validated in place — no
/// serialization on the way in.
///
/// # Safety
/// The host confirms the ABI fingerprint before registering this handler, so
/// `value_ptr` references a `serde_json::Value` whose layout matches this
/// crate's. The caller must not free the value; ownership stays with the host.
unsafe extern "C" fn geo_validate(
    value_ptr: *const c_void,
    _config_ptr: *const c_void,
) -> ValidateResult {
    if value_ptr.is_null() {
        return ValidateResult::error("null value pointer".to_string());
    }
    let value = &*(value_ptr as *const serde_json::Value);
    let result = validate_geojson(value);
    match serde_json::to_vec(&result) {
        Ok(json) => ValidateResult::success(json),
        Err(e) => ValidateResult::error(format!("failed to serialize ValidationResult: {e}")),
    }
}

/// Free a [`ValidateResult`] produced by [`geo_validate`].
///
/// # Safety
/// `result` must have been produced by [`geo_validate`] (this crate's
/// allocator); the pointers are boxed slices reclaimed here.
unsafe extern "C" fn geo_free_validate(result: ValidateResult) {
    if !result.json_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.json_ptr,
            result.json_len,
        ));
    }
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

// =============================================================================
// Python Module (C ABI + pyo3)
// =============================================================================

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use std::ffi::c_void;
    use std::sync::Once;

    use alizarin_extension_api::{FreeValidateFn, TypeHandlerInfo, ValidateFn};
    use pyo3::types::{PyCapsule, PyModule};
    use pyo3::{pyfunction, pymodule, wrap_pyfunction, Bound, Py, PyErr, PyResult, Python};

    /// Static storage for the TypeHandlerInfo (must outlive every capsule).
    static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
    static INIT: Once = Once::new();

    /// Capsule name: null-terminated and `'static` so the pointer handed to
    /// `PyCapsule_New` (which does not copy it) outlives the capsule.
    static CAPSULE_NAME: &[u8] = b"alizarin_geo.geo_handler\0";

    /// Get the type handler capsule for registration with alizarin.
    #[pyfunction]
    pub fn get_geo_handler_capsule(py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        INIT.call_once(|| unsafe {
            HANDLER_INFO = Some(TypeHandlerInfo::new_validating(
                DATATYPE_NAME,
                geo_validate as ValidateFn,
                geo_free_validate as FreeValidateFn,
            ));
        });

        // SAFETY: HANDLER_INFO is initialized unconditionally in the Once above.
        #[allow(static_mut_refs)]
        let ptr = unsafe {
            HANDLER_INFO
                .as_ref()
                .expect("HANDLER_INFO initialized in Once::call_once above")
                as *const TypeHandlerInfo
        };

        // SAFETY: `ptr` points at the 'static HANDLER_INFO (its type name is a
        // 'static const), and CAPSULE_NAME is 'static and null-terminated.
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_feature_collection_round_trips_through_c_abi() {
        let value = json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": {},
                "geometry": { "type": "Point", "coordinates": [-6.25, 54.6] }
            }]
        });

        // Drive the C ABI fn the way the host would: value by pointer.
        let result = unsafe {
            geo_validate(
                &value as *const serde_json::Value as *const c_void,
                std::ptr::null(),
            )
        };
        assert!(!result.is_error(), "handler-level error unexpectedly set");
        assert!(!result.json_ptr.is_null());

        let json: serde_json::Value = {
            let slice = unsafe { std::slice::from_raw_parts(result.json_ptr, result.json_len) };
            serde_json::from_slice(slice).expect("valid JSON out")
        };
        assert_eq!(json["valid"], json!(true));
        unsafe { geo_free_validate(result) };
    }

    #[test]
    fn bare_geometry_is_rejected_as_invalid() {
        let value = json!({ "type": "Point", "coordinates": [-6.25, 54.6] });
        let result = unsafe {
            geo_validate(
                &value as *const serde_json::Value as *const c_void,
                std::ptr::null(),
            )
        };
        assert!(!result.is_error());
        let json: serde_json::Value = {
            let slice = unsafe { std::slice::from_raw_parts(result.json_ptr, result.json_len) };
            serde_json::from_slice(slice).unwrap()
        };
        assert_eq!(json["valid"], json!(false));
        unsafe { geo_free_validate(result) };
    }

    #[test]
    fn null_value_pointer_is_a_handler_error() {
        let result = unsafe { geo_validate(std::ptr::null(), std::ptr::null()) };
        assert!(result.is_error());
        unsafe { geo_free_validate(result) };
    }
}
