//! Alizarin Extension API
//!
//! This crate defines the C ABI types and function signatures for Alizarin extensions.
//! Extensions can implement custom datatype handlers that are called directly from Rust
//! without crossing the Python/WASM boundary at runtime.
//!
//! # Usage
//!
//! Extensions implement coercion handlers and register them via PyCapsule at import time.
//! The handlers are then called directly from Rust during tile processing.
//!
//! # Display Rendering
//!
//! Extensions can optionally implement a display renderer to convert resolved values
//! to human-readable strings. This is used by `toDisplayJson()` for export/indexing.

use std::ffi::c_void;

/// Result of a coercion operation
///
/// All pointers are owned by the extension and must be freed via `free_fn`.
#[repr(C)]
pub struct CoerceResult {
    /// JSON string output (null if error)
    pub json_ptr: *mut u8,
    pub json_len: usize,

    /// Resolved object JSON (for ViewModel constructor, may be same as json_ptr)
    pub resolved_ptr: *mut u8,
    pub resolved_len: usize,

    /// Error message (null if success)
    pub error_ptr: *mut u8,
    pub error_len: usize,
}

impl CoerceResult {
    /// Create a successful result with tile data and resolved object
    pub fn success(tile_json: Vec<u8>, resolved_json: Vec<u8>) -> Self {
        let json_len = tile_json.len();
        let resolved_len = resolved_json.len();

        let json_ptr = Box::into_raw(tile_json.into_boxed_slice()) as *mut u8;
        let resolved_ptr = Box::into_raw(resolved_json.into_boxed_slice()) as *mut u8;

        CoerceResult {
            json_ptr,
            json_len,
            resolved_ptr,
            resolved_len,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create a successful result where tile data and resolved are the same
    pub fn success_same(json: Vec<u8>) -> Self {
        let len = json.len();
        let ptr = Box::into_raw(json.into_boxed_slice()) as *mut u8;

        CoerceResult {
            json_ptr: ptr,
            json_len: len,
            resolved_ptr: ptr,
            resolved_len: len,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create an error result
    pub fn error(message: String) -> Self {
        let bytes = message.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        CoerceResult {
            json_ptr: std::ptr::null_mut(),
            json_len: 0,
            resolved_ptr: std::ptr::null_mut(),
            resolved_len: 0,
            error_ptr: ptr,
            error_len: len,
        }
    }

    /// Check if this result is an error
    pub fn is_error(&self) -> bool {
        !self.error_ptr.is_null()
    }
}

/// Function signature for coercion handlers
///
/// # Arguments
/// * `value_ptr` - JSON string of the value to coerce
/// * `value_len` - Length of value JSON
/// * `config_ptr` - JSON string of node config (may be null)
/// * `config_len` - Length of config JSON
///
/// # Returns
/// * `CoerceResult` - The coerced value and/or error
pub type CoerceFn = unsafe extern "C" fn(
    value_ptr: *const u8,
    value_len: usize,
    config_ptr: *const u8,
    config_len: usize,
) -> CoerceResult;

/// Function signature for freeing CoerceResult
pub type FreeFn = unsafe extern "C" fn(result: CoerceResult);

// =============================================================================
// Display Rendering API
// =============================================================================

/// Result of a display render operation
///
/// Returns the human-readable display string for a resolved value.
/// All pointers are owned by the extension and must be freed via `free_display_fn`.
#[repr(C)]
pub struct RenderDisplayResult {
    /// Display string (null if error)
    pub display_ptr: *mut u8,
    pub display_len: usize,

    /// Error message (null if success)
    pub error_ptr: *mut u8,
    pub error_len: usize,
}

impl RenderDisplayResult {
    /// Create a successful result with display string
    pub fn success(display: String) -> Self {
        let bytes = display.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        RenderDisplayResult {
            display_ptr: ptr,
            display_len: len,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create an error result
    pub fn error(message: String) -> Self {
        let bytes = message.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        RenderDisplayResult {
            display_ptr: std::ptr::null_mut(),
            display_len: 0,
            error_ptr: ptr,
            error_len: len,
        }
    }

    /// Check if this result is an error
    pub fn is_error(&self) -> bool {
        !self.error_ptr.is_null()
    }
}

/// Function signature for display rendering
///
/// Takes a resolved value (from CoerceResult.resolved_ptr) and returns
/// a human-readable display string.
///
/// # Arguments
/// * `resolved_ptr` - JSON string of the resolved value
/// * `resolved_len` - Length of resolved JSON
/// * `lang_ptr` - Language code (e.g., "en")
/// * `lang_len` - Length of language code
///
/// # Returns
/// * `RenderDisplayResult` - The display string and/or error
pub type RenderDisplayFn = unsafe extern "C" fn(
    resolved_ptr: *const u8,
    resolved_len: usize,
    lang_ptr: *const u8,
    lang_len: usize,
) -> RenderDisplayResult;

/// Function signature for freeing RenderDisplayResult
pub type FreeDisplayFn = unsafe extern "C" fn(result: RenderDisplayResult);

/// Standard free function for RenderDisplayResult
#[no_mangle]
pub unsafe extern "C" fn alizarin_free_render_display_result(result: RenderDisplayResult) {
    if !result.display_ptr.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(
            result.display_ptr,
            result.display_len,
        ));
    }
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

// =============================================================================
// Handler Registration
// =============================================================================

/// Handler registration info passed via PyCapsule
///
/// Extensions must provide coerce_fn and free_fn.
/// render_display_fn and free_display_fn are optional (null if not implemented).
#[repr(C)]
pub struct TypeHandlerInfo {
    /// Datatype name (e.g., "reference")
    pub type_name_ptr: *const u8,
    pub type_name_len: usize,

    /// Coercion function (required)
    pub coerce_fn: CoerceFn,

    /// Free function for CoerceResult (required)
    pub free_fn: FreeFn,

    /// Display render function (optional - null if not implemented)
    /// Used by toDisplayJson() to get human-readable strings
    pub render_display_fn: Option<RenderDisplayFn>,

    /// Free function for RenderDisplayResult (optional - null if render_display_fn is null)
    pub free_display_fn: Option<FreeDisplayFn>,

    /// Opaque data pointer (for extension use)
    pub user_data: *mut c_void,
}

/// Helper to create a TypeHandlerInfo
impl TypeHandlerInfo {
    /// Create a handler without display rendering support
    pub fn new(
        type_name: &'static str,
        coerce_fn: CoerceFn,
        free_fn: FreeFn,
    ) -> Self {
        TypeHandlerInfo {
            type_name_ptr: type_name.as_ptr(),
            type_name_len: type_name.len(),
            coerce_fn,
            free_fn,
            render_display_fn: None,
            free_display_fn: None,
            user_data: std::ptr::null_mut(),
        }
    }

    /// Create a handler with display rendering support
    pub fn with_display_renderer(
        type_name: &'static str,
        coerce_fn: CoerceFn,
        free_fn: FreeFn,
        render_display_fn: RenderDisplayFn,
        free_display_fn: FreeDisplayFn,
    ) -> Self {
        TypeHandlerInfo {
            type_name_ptr: type_name.as_ptr(),
            type_name_len: type_name.len(),
            coerce_fn,
            free_fn,
            render_display_fn: Some(render_display_fn),
            free_display_fn: Some(free_display_fn),
            user_data: std::ptr::null_mut(),
        }
    }

    /// Check if this handler supports display rendering
    pub fn has_display_renderer(&self) -> bool {
        self.render_display_fn.is_some()
    }
}

/// Standard free function for CoerceResult
///
/// Extensions can use this if they allocate results using the standard methods.
#[no_mangle]
pub unsafe extern "C" fn alizarin_free_coerce_result(result: CoerceResult) {
    if !result.json_ptr.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(
            result.json_ptr,
            result.json_len,
        ));
    }
    // Only free resolved if different from json
    if !result.resolved_ptr.is_null() && result.resolved_ptr != result.json_ptr {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(
            result.resolved_ptr,
            result.resolved_len,
        ));
    }
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coerce_result_success() {
        let result = CoerceResult::success(
            b"\"tile_data\"".to_vec(),
            b"{\"resolved\": true}".to_vec(),
        );
        assert!(!result.is_error());
        assert!(!result.json_ptr.is_null());
        assert!(!result.resolved_ptr.is_null());

        // Clean up
        unsafe { alizarin_free_coerce_result(result) };
    }

    #[test]
    fn test_coerce_result_error() {
        let result = CoerceResult::error("Test error".to_string());
        assert!(result.is_error());
        assert!(result.json_ptr.is_null());
        assert!(!result.error_ptr.is_null());

        // Clean up
        unsafe { alizarin_free_coerce_result(result) };
    }
}
