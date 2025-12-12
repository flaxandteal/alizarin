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

/// Handler registration info passed via PyCapsule
#[repr(C)]
pub struct TypeHandlerInfo {
    /// Datatype name (e.g., "reference")
    pub type_name_ptr: *const u8,
    pub type_name_len: usize,

    /// Coercion function
    pub coerce_fn: CoerceFn,

    /// Free function (to free CoerceResult)
    pub free_fn: FreeFn,

    /// Opaque data pointer (for extension use)
    pub user_data: *mut c_void,
}

/// Helper to create a TypeHandlerInfo
impl TypeHandlerInfo {
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
            user_data: std::ptr::null_mut(),
        }
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
