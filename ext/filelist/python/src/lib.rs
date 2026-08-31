//! Alizarin FileList Extension
//!
//! This extension provides the "file-list" datatype handler for file attachments
//! in Arches. It handles coercion and display rendering of file list values.

// Re-export core types so downstream users don't need to depend on alizarin-filelist-core directly
pub use alizarin_filelist_core::{
    coerce_filelist_value, coerce_single_file, create_filelist_handler,
    render_filelist_display_value, FileListItem, FileListTypeHandler, LocalizedString,
    LocalizedStringValue, DATATYPE_NAME,
};

// =============================================================================
// Python Module (C ABI + pyo3)
// =============================================================================

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use pyo3::types::{PyCapsule, PyModule};
    use pyo3::{pyfunction, pymodule, wrap_pyfunction, Bound, Py, PyErr, PyResult, Python};
    use std::ffi::{c_void, CString};

    /// Get the type handler capsule for registration with alizarin.
    #[pyfunction]
    pub fn get_filelist_handler_capsule(py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        // The TypeHandlerInfo + its extern-"C" wrappers live in filelist-core
        // (alizarin_filelist_core::c_abi), shared with the napi binding.
        let ptr = alizarin_filelist_core::c_abi::filelist_handler_type_info();

        // SAFETY: Hardcoded string with no null bytes
        let name = CString::new("alizarin_filelist.filelist_handler")
            .expect("handler name contains no null bytes");

        unsafe {
            let capsule = pyo3::ffi::PyCapsule_New(ptr as *mut c_void, name.as_ptr(), None);

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
                m.insert(
                    "en".to_string(),
                    LocalizedStringValue {
                        direction: "ltr".to_string(),
                        value: "My Test Image".to_string(),
                    },
                );
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
