//! C-ABI [`TypeHandlerInfo`] for the CLM `reference` handler.
//!
//! The `extern "C"` wrappers (coerce / render / resolve-markers) + the
//! process-static `TypeHandlerInfo` live here so both bindings share one copy:
//! the Python build hands the pointer to the host via `PyCapsule`, the NAPI build
//! via a `BigInt`. Values cross by pointer/JSON as the ABI requires — no extra
//! serialization.

use std::ffi::c_void;
use std::sync::Once;

use serde_json::Value;

use alizarin_extension_api::{
    abi_fingerprint, alizarin_free_coerce_result, alizarin_free_render_display_result,
    alizarin_free_resolve_markers_result, CoerceFn, CoerceResult, ConceptLookupByIdFn,
    ConceptLookupByLabelFn, FreeConceptJsonFn, FreeDisplayFn, FreeFn, FreeResolveMarkersFn,
    FreeIndexSpecFn, HasCollectionFn, IndexSpecFn, IndexSpecResult, RenderDisplayFn,
    RenderDisplayResult, ResolveMarkersFn, ResolveMarkersResult, TypeHandlerInfo,
};

use crate::{
    coerce_reference_value, reference_index_spec, render_reference_display_value, ReferenceNodeConfig,
};

    unsafe extern "C" fn coerce_reference(
        value_ptr: *const u8,
        value_len: usize,
        config_ptr: *const u8,
        config_len: usize,
    ) -> CoerceResult {
        let value_slice = std::slice::from_raw_parts(value_ptr, value_len);
        let value_str = match std::str::from_utf8(value_slice) {
            Ok(s) => s,
            Err(e) => return CoerceResult::error(format!("Invalid UTF-8 in value: {}", e)),
        };

        let value: Value = match serde_json::from_str(value_str) {
            Ok(v) => v,
            Err(e) => return CoerceResult::error(format!("Invalid JSON value: {}", e)),
        };

        let config: ReferenceNodeConfig = if config_len > 0 && !config_ptr.is_null() {
            let config_slice = std::slice::from_raw_parts(config_ptr, config_len);
            let config_str = match std::str::from_utf8(config_slice) {
                Ok(s) => s,
                Err(_) => return CoerceResult::error("Invalid UTF-8 in config".to_string()),
            };
            serde_json::from_str::<ReferenceNodeConfig>(config_str).unwrap_or_default()
        } else {
            ReferenceNodeConfig::default()
        };

        match coerce_reference_value(&value, &config) {
            Ok((tile_data, resolved)) => {
                match (
                    serde_json::to_vec(&tile_data),
                    serde_json::to_vec(&resolved),
                ) {
                    (Ok(tile_json), Ok(resolved_json)) => {
                        CoerceResult::success(tile_json, resolved_json)
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        CoerceResult::error(format!("Failed to serialize coerced value: {}", e))
                    }
                }
            }
            Err(e) => CoerceResult::error(e),
        }
    }

    /// C ABI display render function for reference type
    unsafe extern "C" fn render_reference_display(
        resolved_ptr: *const u8,
        resolved_len: usize,
        lang_ptr: *const u8,
        lang_len: usize,
    ) -> RenderDisplayResult {
        let resolved_slice = std::slice::from_raw_parts(resolved_ptr, resolved_len);
        let resolved_str = match std::str::from_utf8(resolved_slice) {
            Ok(s) => s,
            Err(e) => {
                return RenderDisplayResult::error(format!("Invalid UTF-8 in resolved: {}", e))
            }
        };

        let lang_slice = std::slice::from_raw_parts(lang_ptr, lang_len);
        let lang = std::str::from_utf8(lang_slice).ok();

        let resolved: Value = match serde_json::from_str(resolved_str) {
            Ok(v) => v,
            Err(e) => return RenderDisplayResult::error(format!("Invalid JSON: {}", e)),
        };

        match render_reference_display_value(&resolved, lang) {
            Ok(s) => RenderDisplayResult::success(s),
            Err(e) => RenderDisplayResult::error(e),
        }
    }

    // =========================================================================
    // Marker Resolution (delegates to core's resolve_reference_markers)
    // =========================================================================

    /// Call an FFI concept lookup callback and parse the returned JSON.
    ///
    /// # Safety
    /// All pointer arguments must be valid for the lifetime of this call.
    unsafe fn call_concept_lookup(
        lookup_fn: unsafe extern "C" fn(
            *mut c_void,
            *const u8,
            usize,
            *const u8,
            usize,
            *mut *mut u8,
            *mut usize,
        ) -> bool,
        free_fn: FreeConceptJsonFn,
        user_data: *mut c_void,
        collection_id: &str,
        key: &str,
    ) -> Option<Value> {
        let mut json_ptr: *mut u8 = std::ptr::null_mut();
        let mut json_len: usize = 0;

        let found = lookup_fn(
            user_data,
            collection_id.as_ptr(),
            collection_id.len(),
            key.as_ptr(),
            key.len(),
            &mut json_ptr,
            &mut json_len,
        );

        if !found || json_ptr.is_null() || json_len == 0 {
            return None;
        }

        let json_slice = std::slice::from_raw_parts(json_ptr, json_len);
        let result = std::str::from_utf8(json_slice)
            .ok()
            .and_then(|s| serde_json::from_str(s).ok());

        free_fn(json_ptr, json_len);
        result
    }

    /// C ABI marker resolution function for reference type.
    ///
    /// Uses the provided RDM lookup callbacks to resolve markers, bridging
    /// the shared library boundary (the CLM .so has its own copy of the global
    /// RDM cache, so we must use the callbacks from the alizarin Python binding
    /// which has the populated cache).
    unsafe extern "C" fn resolve_reference_markers(
        value_ptr: *const u8,
        value_len: usize,
        lang_ptr: *const u8,
        lang_len: usize,
        _has_collection: HasCollectionFn,
        lookup_by_id: ConceptLookupByIdFn,
        lookup_by_label: ConceptLookupByLabelFn,
        free_concept_json: FreeConceptJsonFn,
        lookup_user_data: *mut c_void,
    ) -> ResolveMarkersResult {
        let value_slice = std::slice::from_raw_parts(value_ptr, value_len);
        let value_str = match std::str::from_utf8(value_slice) {
            Ok(s) => s,
            Err(e) => return ResolveMarkersResult::error(format!("Invalid UTF-8 in value: {}", e)),
        };

        let value: Value = match serde_json::from_str(value_str) {
            Ok(v) => v,
            Err(e) => return ResolveMarkersResult::error(format!("Invalid JSON value: {}", e)),
        };

        let language = if lang_len > 0 && !lang_ptr.is_null() {
            std::str::from_utf8(std::slice::from_raw_parts(lang_ptr, lang_len)).unwrap_or("en")
        } else {
            "en"
        };

        let resolved = match crate::resolve_reference_markers_with_lookups(
            &value,
            language,
            |collection_id, concept_id| unsafe {
                call_concept_lookup(
                    lookup_by_id,
                    free_concept_json,
                    lookup_user_data,
                    collection_id,
                    concept_id,
                )
            },
            |collection_id, label| unsafe {
                call_concept_lookup(
                    lookup_by_label,
                    free_concept_json,
                    lookup_user_data,
                    collection_id,
                    label,
                )
            },
        ) {
            Ok(v) => v,
            // Malformed reference data (e.g. a non-UUID concept id) is an input
            // problem — surface it rather than emitting/keeping bad data.
            Err(e) => return ResolveMarkersResult::error(e),
        };

        if resolved == value {
            ResolveMarkersResult::unchanged()
        } else {
            match serde_json::to_vec(&resolved) {
                Ok(json) => ResolveMarkersResult::success(json),
                Err(e) => ResolveMarkersResult::error(format!(
                    "Failed to serialize resolved value: {}",
                    e
                )),
            }
        }
    }

    // =========================================================================
    // Index Spec (delegates to core's reference_index_spec)
    // =========================================================================

    /// C ABI index-spec function for the reference type. `value_ptr`/`config_ptr`
    /// cross by pointer (`*const serde_json::Value`, config null for none); returns
    /// a serialized `Option<IndexSpec>` (always `Some` — a ConceptHierarchical spec).
    unsafe extern "C" fn index_spec_reference(
        value_ptr: *const c_void,
        config_ptr: *const c_void,
    ) -> IndexSpecResult {
        if value_ptr.is_null() {
            return IndexSpecResult::error("null value pointer".to_string());
        }
        let value = &*(value_ptr as *const Value);
        let config = if config_ptr.is_null() {
            None
        } else {
            Some(&*(config_ptr as *const Value))
        };

        let spec = Some(reference_index_spec(value, config));
        match serde_json::to_vec(&spec) {
            Ok(json) => IndexSpecResult::success(json),
            Err(e) => IndexSpecResult::error(format!("Failed to serialize index spec: {}", e)),
        }
    }

    /// Free an [`IndexSpecResult`] allocated by [`index_spec_reference`].
    unsafe extern "C" fn free_index_spec_reference(result: IndexSpecResult) {
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

    // =========================================================================
    // PyCapsule Registration
    // =========================================================================


static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
static INIT: Once = Once::new();

/// Pointer to the process-static CLM `reference` `TypeHandlerInfo` (coerce +
/// display + resolve-markers + index-spec). A binding hands this to the host for
/// registration.
pub fn reference_handler_type_info() -> *const TypeHandlerInfo {
    static TYPE_NAME: &[u8] = b"reference";
    INIT.call_once(|| unsafe {
        HANDLER_INFO = Some(TypeHandlerInfo {
            type_name_ptr: TYPE_NAME.as_ptr(),
            type_name_len: TYPE_NAME.len(),
            coerce_fn: Some(coerce_reference as CoerceFn),
            free_fn: Some(alizarin_free_coerce_result as FreeFn),
            render_display_fn: Some(render_reference_display as RenderDisplayFn),
            free_display_fn: Some(alizarin_free_render_display_result as FreeDisplayFn),
            resolve_markers_fn: Some(resolve_reference_markers as ResolveMarkersFn),
            free_resolve_markers_fn: Some(
                alizarin_free_resolve_markers_result as FreeResolveMarkersFn,
            ),
            validate_fn: None,
            free_validate_fn: None,
            render_search_fn: None,
            free_render_search_fn: None,
            index_spec_fn: Some(index_spec_reference as IndexSpecFn),
            free_index_spec_fn: Some(free_index_spec_reference as FreeIndexSpecFn),
            abi: abi_fingerprint(),
            user_data: std::ptr::null_mut(),
        });
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
