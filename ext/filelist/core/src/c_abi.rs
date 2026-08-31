//! C-ABI [`TypeHandlerInfo`] for the file-list handler.
//!
//! The `extern "C"` wrappers (coerce / render) + the process-static
//! `TypeHandlerInfo` live here so both bindings share one copy: Python via
//! `PyCapsule`, NAPI via a `BigInt`.

use std::sync::Once;

use serde_json::Value;

use alizarin_extension_api::{
    abi_fingerprint, alizarin_free_coerce_result, alizarin_free_render_display_result, CoerceFn,
    CoerceResult, FreeDisplayFn, FreeFn, RenderDisplayFn, RenderDisplayResult, TypeHandlerInfo,
};

use crate::{coerce_filelist_value, render_filelist_display_value};

    /// C ABI coercion function for file-list type.
    unsafe extern "C" fn coerce_filelist(
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

        let _ = (config_ptr, config_len);

        match coerce_filelist_value(&value) {
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

    /// C ABI display render function for file-list type.
    unsafe extern "C" fn render_filelist_display(
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

        match render_filelist_display_value(&resolved, lang) {
            Ok(s) => RenderDisplayResult::success(s),
            Err(e) => RenderDisplayResult::error(e),
        }
    }

static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
static INIT: Once = Once::new();

/// Pointer to the process-static file-list `TypeHandlerInfo` (coerce + display).
/// A binding hands this to the host for registration.
pub fn filelist_handler_type_info() -> *const TypeHandlerInfo {
    static TYPE_NAME: &[u8] = b"file-list";
    INIT.call_once(|| unsafe {
        HANDLER_INFO = Some(TypeHandlerInfo {
            type_name_ptr: TYPE_NAME.as_ptr(),
            type_name_len: TYPE_NAME.len(),
            coerce_fn: Some(coerce_filelist as CoerceFn),
            free_fn: Some(alizarin_free_coerce_result as FreeFn),
            render_display_fn: Some(render_filelist_display as RenderDisplayFn),
            free_display_fn: Some(alizarin_free_render_display_result as FreeDisplayFn),
            resolve_markers_fn: None,
            free_resolve_markers_fn: None,
            validate_fn: None,
            free_validate_fn: None,
            render_search_fn: None,
            free_render_search_fn: None,
            index_spec_fn: None,
            free_index_spec_fn: None,
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
