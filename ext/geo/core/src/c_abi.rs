//! C-ABI [`TypeHandlerInfo`] for the geo handler: validation and folding.
//!
//! The `extern "C"` wrappers + the process-static `TypeHandlerInfo` live here so
//! both bindings share one copy: the Python build hands the pointer to the host
//! via `PyCapsule`, the NAPI build via a `BigInt`. Coercion stays in core.
//!
//! The fold functions wrap [`crate::fold`], the same implementation the native
//! handler and the WASM build use, so every binding folds geometry identically.

use std::ffi::c_void;
use std::sync::Once;

use alizarin_core::extension_type_registry::ExtensionError;
use alizarin_extension_api::{
    FoldFn, FoldIdentityFn, FoldMergeFn, FoldRemoveFn, FoldResult, FreeFoldFn, FreeValidateFn,
    TypeHandlerInfo, ValidateFn, ValidateResult,
};
use serde_json::Value;

use crate::{fold, validate_geojson, DATATYPE_NAME};

/// A fold function's result for the host: the summary as JSON, or the error.
fn fold_result(result: Result<Value, ExtensionError>) -> FoldResult {
    match result.and_then(|v| {
        serde_json::to_vec(&v).map_err(|e| ExtensionError::new(format!("serialize: {e}")))
    }) {
        Ok(json) => FoldResult::success(json),
        Err(e) => FoldResult::error(e.to_string()),
    }
}

/// A summary the host passes by pointer, cloned.
///
/// # Safety
/// `ptr` is null or a `*const serde_json::Value` live for the call.
unsafe fn summary_arg(ptr: *const c_void) -> Result<Value, ExtensionError> {
    if ptr.is_null() {
        return Err(ExtensionError::new("null summary pointer"));
    }
    Ok((*(ptr as *const Value)).clone())
}

/// A batch of values the host passes by pointer, borrowed, not copied.
///
/// # Safety
/// `ptr` points to `len` `*const serde_json::Value`s live for the call.
unsafe fn values_arg<'a>(
    ptr: *const *const c_void,
    len: usize,
) -> Result<Vec<&'a Value>, ExtensionError> {
    if len == 0 {
        return Ok(Vec::new());
    }
    if ptr.is_null() {
        return Err(ExtensionError::new("null values pointer"));
    }
    std::slice::from_raw_parts(ptr, len)
        .iter()
        .map(|p| {
            (!p.is_null())
                .then(|| &*(*p as *const Value))
                .ok_or_else(|| ExtensionError::new("null value pointer"))
        })
        .collect()
}

unsafe extern "C" fn geo_fold_identity(_config_ptr: *const c_void) -> FoldResult {
    fold_result(Ok(fold::identity()))
}

unsafe extern "C" fn geo_fold(
    acc_ptr: *const c_void,
    values_ptr: *const *const c_void,
    values_len: usize,
    _config_ptr: *const c_void,
) -> FoldResult {
    fold_result((|| {
        fold::fold(summary_arg(acc_ptr)?, &values_arg(values_ptr, values_len)?)
    })())
}

unsafe extern "C" fn geo_fold_merge(
    a_ptr: *const c_void,
    b_ptr: *const c_void,
    _config_ptr: *const c_void,
) -> FoldResult {
    fold_result((|| fold::merge(summary_arg(a_ptr)?, summary_arg(b_ptr)?))())
}

unsafe extern "C" fn geo_fold_remove(
    acc_ptr: *const c_void,
    values_ptr: *const *const c_void,
    values_len: usize,
    _config_ptr: *const c_void,
) -> FoldResult {
    fold_result((|| {
        let (summary, exact) =
            fold::remove(summary_arg(acc_ptr)?, &values_arg(values_ptr, values_len)?)?;
        Ok(serde_json::json!({ "summary": summary, "exact": exact }))
    })())
}

/// Free a [`FoldResult`] produced by any of the geo fold functions.
///
/// # Safety
/// `result` must have been produced by one of them (this crate's allocator).
unsafe extern "C" fn geo_free_fold(result: FoldResult) {
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

/// Pointer to the process-static geo `TypeHandlerInfo` (validate + fold). A
/// binding hands this to the host for registration — Python via `PyCapsule`,
/// NAPI via a `BigInt`. The pointee outlives every capsule (it is `'static`).
pub fn geo_handler_type_info() -> *const TypeHandlerInfo {
    INIT.call_once(|| unsafe {
        let mut info = TypeHandlerInfo::new_validating(
            DATATYPE_NAME,
            geo_validate as ValidateFn,
            geo_free_validate as FreeValidateFn,
        );
        info.fold_identity_fn = Some(geo_fold_identity as FoldIdentityFn);
        info.fold_fn = Some(geo_fold as FoldFn);
        info.fold_merge_fn = Some(geo_fold_merge as FoldMergeFn);
        info.fold_remove_fn = Some(geo_fold_remove as FoldRemoveFn);
        info.free_fold_fn = Some(geo_free_fold as FreeFoldFn);
        HANDLER_INFO = Some(info);
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

    /// The host path napi and Python take: register the `TypeHandlerInfo` pointer
    /// with core, then fold through the C ABI via `datatype_fold`.
    #[test]
    fn folds_through_the_host_over_the_c_abi() {
        use alizarin_core::datatype_fold::{
            fold_identity, fold_merge, fold_remove, fold_values, is_loose,
        };

        unsafe { alizarin_core::extension_abi::register_handler_from_ptr(geo_handler_type_info()) }
            .unwrap();
        let handlers = alizarin_core::extension_abi::build_extension_registry_from_registered();
        let reg = Some(&handlers);
        let point = |x: f64, y: f64| {
            json!({ "type": "FeatureCollection", "features": [{
                "type": "Feature", "properties": {},
                "geometry": { "type": "Point", "coordinates": [x, y] } }] })
        };
        let (west, middle, east) = (point(-10.0, 54.0), point(-7.0, 54.5), point(-5.0, 55.0));

        let identity = fold_identity(DATATYPE_NAME, None, reg).expect("geo folds over the ABI");
        let acc = fold_values(
            DATATYPE_NAME,
            identity.clone(),
            &[&west, &middle],
            None,
            reg,
        )
        .unwrap()
        .summary;
        let other = fold_values(DATATYPE_NAME, identity, &[&east], None, reg)
            .unwrap()
            .summary;
        let merged = fold_merge(DATATYPE_NAME, acc, other, None, reg).unwrap();
        assert_eq!(
            merged,
            json!({ "count": 3, "bbox": [-10.0, 54.0, -5.0, 55.0], "geometry_types": ["Point"] })
        );

        let inner = fold_remove(DATATYPE_NAME, merged.clone(), &[&middle], None, reg)
            .unwrap()
            .unwrap();
        assert!(!is_loose(&inner));
        let edge = fold_remove(DATATYPE_NAME, merged, &[&west], None, reg)
            .unwrap()
            .unwrap();
        assert!(is_loose(&edge), "the westmost point touched the bbox");

        let err = fold_values(DATATYPE_NAME, edge, &[&json!("not geojson")], None, reg);
        assert!(err.is_err(), "a handler error crosses the ABI as an error");
    }
}
