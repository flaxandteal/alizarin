//! Alizarin FileList Extension
//!
//! This extension provides the "file-list" datatype handler for file attachments
//! in Arches. It handles coercion and display rendering of file list values.

// Re-export core types so downstream users don't need to depend on alizarin-filelist-core directly
pub use alizarin_filelist_core::{
    FileListItem, LocalizedString, LocalizedStringValue,
    coerce_filelist_value, coerce_single_file, render_filelist_display_value,
    FileListTypeHandler, create_filelist_handler, DATATYPE_NAME,
};

// =============================================================================
// Python Module (C ABI + pyo3)
// =============================================================================

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use serde_json::Value;
    use alizarin_extension_api::{
        alizarin_free_coerce_result, alizarin_free_render_display_result,
        CoerceFn, CoerceResult, FreeFn, TypeHandlerInfo,
        RenderDisplayFn, RenderDisplayResult, FreeDisplayFn,
    };
    use pyo3::{pyfunction, pymodule, wrap_pyfunction, Bound, Py, PyErr, PyResult, Python};
    use pyo3::types::{PyCapsule, PyModule};
    use std::ffi::{c_void, CString};

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
                match (serde_json::to_vec(&tile_data), serde_json::to_vec(&resolved)) {
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
            Err(e) => return RenderDisplayResult::error(format!("Invalid UTF-8 in resolved: {}", e)),
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
    use std::sync::Once;

    /// Static storage for the TypeHandlerInfo
    static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
    static INIT: Once = Once::new();

    /// Get the type handler capsule for registration with alizarin.
    #[pyfunction]
    pub fn get_filelist_handler_capsule(py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        static TYPE_NAME: &[u8] = b"file-list";

        // Initialize the static handler info once
        INIT.call_once(|| {
            unsafe {
                HANDLER_INFO = Some(TypeHandlerInfo {
                    type_name_ptr: TYPE_NAME.as_ptr(),
                    type_name_len: TYPE_NAME.len(),
                    coerce_fn: coerce_filelist as CoerceFn,
                    free_fn: alizarin_free_coerce_result as FreeFn,
                    render_display_fn: Some(render_filelist_display as RenderDisplayFn),
                    free_display_fn: Some(alizarin_free_render_display_result as FreeDisplayFn),
                    resolve_markers_fn: None,
                    free_resolve_markers_fn: None,
                    user_data: std::ptr::null_mut(),
                });
            }
        });

        // Get pointer to the static handler info
        // SAFETY: HANDLER_INFO is initialized unconditionally in Once::call_once above
        #[allow(static_mut_refs)]
        let ptr = unsafe {
            HANDLER_INFO.as_ref().expect("HANDLER_INFO initialized in Once::call_once above")
                as *const TypeHandlerInfo
        };

        // SAFETY: Hardcoded string with no null bytes
        let name = CString::new("alizarin_filelist.filelist_handler")
            .expect("handler name contains no null bytes");

        unsafe {
            let capsule = pyo3::ffi::PyCapsule_New(
                ptr as *mut c_void,
                name.as_ptr(),
                None,
            );

            if capsule.is_null() {
                return Err(PyErr::fetch(py));
            }

            Ok(Py::from_owned_ptr(py, capsule))
        }
    }

    /// Python module definition
    #[pymodule]
    pub fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(get_filelist_handler_capsule, m)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_coerce_file_object() {
        let value = json!({
            "name": "test.png",
            "file_id": "123e4567-e89b-12d3-a456-426614174000",
            "size": 12345,
            "type": "image/png",
            "url": "/files/123e4567-e89b-12d3-a456-426614174000",
            "status": "uploaded"
        });

        let result = coerce_filelist_value(&value);

        assert!(result.is_ok());
        let (tile_data, _resolved) = result.unwrap();
        assert!(tile_data.is_array());
        assert_eq!(tile_data.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_coerce_file_array() {
        let value = json!([
            {"name": "file1.png", "url": "/files/1"},
            {"name": "file2.jpg", "url": "/files/2"}
        ]);

        let result = coerce_filelist_value(&value);

        assert!(result.is_ok());
        let (tile_data, _) = result.unwrap();
        assert!(tile_data.is_array());
        assert_eq!(tile_data.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_file_display_string() {
        let file = FileListItem {
            name: "test-file.png".to_string(),
            title: Some({
                let mut m = HashMap::new();
                m.insert("en".to_string(), LocalizedStringValue {
                    direction: "ltr".to_string(),
                    value: "My Test Image".to_string(),
                });
                m
            }),
            ..Default::default()
        };

        assert_eq!(file.to_display_string(Some("en")), "My Test Image");
        assert_eq!(file.to_display_string(Some("fr")), "My Test Image"); // Falls back
    }

    #[test]
    fn test_file_display_string_fallback() {
        let file = FileListItem {
            name: "test-file.png".to_string(),
            title: None,
            ..Default::default()
        };

        assert_eq!(file.to_display_string(Some("en")), "test-file.png");
    }

    #[test]
    fn test_is_image() {
        let image = FileListItem {
            file_type: Some("image/png".to_string()),
            ..Default::default()
        };
        assert!(image.is_image());

        let pdf = FileListItem {
            file_type: Some("application/pdf".to_string()),
            ..Default::default()
        };
        assert!(!pdf.is_image());
    }

    #[test]
    fn test_render_filelist_display() {
        let resolved = json!([
            {"name": "file1.png", "title": {"en": {"direction": "ltr", "value": "First File"}}},
            {"name": "file2.jpg"}
        ]);
        assert_eq!(
            render_filelist_display_value(&resolved, Some("en")).unwrap(),
            "First File, file2.jpg"
        );
    }
}
