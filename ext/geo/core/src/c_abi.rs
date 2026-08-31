//! C-ABI [`TypeHandlerInfo`] for the geo validate handler.
//!
//! The `extern "C"` wrappers + the process-static `TypeHandlerInfo` live here so
//! both bindings share one copy: the Python build hands the pointer to the host
//! via `PyCapsule`, the NAPI build via a `BigInt`. Validate-only (coercion stays
//! in core).

use std::ffi::c_void;
use std::sync::Once;

use alizarin_extension_api::{FreeValidateFn, TypeHandlerInfo, ValidateFn, ValidateResult};

use crate::{validate_geojson, DATATYPE_NAME};

/// # Safety
/// The host confirms the ABI fingerprint before registering this handler, so
/// `value_ptr` references a `serde_json::Value` whose layout matches this crate's.
/// Ownership stays with the host — do not free the value.
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
/// `result` must have been produced by [`geo_validate`] (this crate's allocator).
unsafe extern "C" fn geo_free_validate(result: ValidateResult) {
    if !result.json_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.json_ptr,
            result.json_len,
        ));
    }
}

static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
static INIT: Once = Once::new();

/// Pointer to the process-static geo `TypeHandlerInfo` (validate-only). A binding
/// hands this to the host for registration — Python via `PyCapsule`, NAPI via a
/// `BigInt`. The pointee outlives every capsule (it is `'static`).
pub fn geo_handler_type_info() -> *const TypeHandlerInfo {
    INIT.call_once(|| unsafe {
        HANDLER_INFO = Some(TypeHandlerInfo::new_validating(
            DATATYPE_NAME,
            geo_validate as ValidateFn,
            geo_free_validate as FreeValidateFn,
        ));
    });
    // SAFETY: HANDLER_INFO is initialised unconditionally in the Once above and
    // never mutated afterwards.
    #[allow(static_mut_refs)]
    unsafe {
        HANDLER_INFO
            .as_ref()
            .expect("HANDLER_INFO initialised in Once::call_once")
            as *const TypeHandlerInfo
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
                "type": "Feature", "properties": {},
                "geometry": { "type": "Point", "coordinates": [-6.25, 54.6] }
            }]
        });
        // Drive the C-ABI fn the way the host would: value by pointer.
        let result =
            unsafe { geo_validate(&value as *const serde_json::Value as *const c_void, std::ptr::null()) };
        assert!(!result.is_error());
        let slice = unsafe { std::slice::from_raw_parts(result.json_ptr, result.json_len) };
        let out: serde_json::Value = serde_json::from_slice(slice).unwrap();
        assert_eq!(out["valid"], json!(true));
        unsafe { geo_free_validate(result) };
    }

    #[test]
    fn bare_geometry_is_rejected() {
        let value = json!({ "type": "Point", "coordinates": [-6.25, 54.6] });
        let result =
            unsafe { geo_validate(&value as *const serde_json::Value as *const c_void, std::ptr::null()) };
        assert!(!result.is_error());
        let slice = unsafe { std::slice::from_raw_parts(result.json_ptr, result.json_len) };
        let out: serde_json::Value = serde_json::from_slice(slice).unwrap();
        assert_eq!(out["valid"], json!(false));
        unsafe { geo_free_validate(result) };
    }

    #[test]
    fn null_value_pointer_is_a_handler_error() {
        let result = unsafe { geo_validate(std::ptr::null(), std::ptr::null()) };
        assert!(result.is_error());
        unsafe { geo_free_validate(result) };
    }

    #[test]
    fn type_info_pointer_is_stable() {
        assert_eq!(geo_handler_type_info(), geo_handler_type_info());
    }
}
