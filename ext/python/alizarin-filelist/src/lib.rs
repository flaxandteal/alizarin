//! Alizarin FileList Extension
//!
//! This extension provides the "file-list" datatype handler for file attachments
//! in Arches. It handles coercion and display rendering of file list values.

#[cfg(feature = "pyo3-ext")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use alizarin_extension_api::{
    alizarin_free_coerce_result, alizarin_free_render_display_result,
    CoerceFn, CoerceResult, FreeFn, TypeHandlerInfo,
    RenderDisplayFn, RenderDisplayResult, FreeDisplayFn,
};

// =============================================================================
// Display Serializer (requires alizarin-core)
// =============================================================================

#[cfg(feature = "mutations")]
mod display_serializer {
    use super::{FileListItem, Value};
    use alizarin_core::{
        ExtensionDisplaySerializer, SerializationOptions, SerializationResult,
        DisplaySerializerRegistry,
    };
    use std::sync::Arc;

    /// Display serializer for file-list type.
    pub struct FileListDisplaySerializer;

    impl ExtensionDisplaySerializer for FileListDisplaySerializer {
        fn serialize_display(
            &self,
            tile_data: &Value,
            options: &SerializationOptions,
        ) -> SerializationResult {
            let lang = options.language.as_str();

            match tile_data {
                Value::Null => SerializationResult::success(Value::Null),

                // Array of files
                Value::Array(arr) => {
                    let displays: Vec<String> = arr
                        .iter()
                        .filter_map(|item| {
                            serde_json::from_value::<FileListItem>(item.clone())
                                .ok()
                                .map(|f| f.to_display_string(Some(lang)))
                        })
                        .collect();

                    if displays.is_empty() {
                        return SerializationResult::success(Value::Null);
                    }
                    SerializationResult::success(Value::String(displays.join(", ")))
                }

                // Single file object
                Value::Object(_) => {
                    match serde_json::from_value::<FileListItem>(tile_data.clone()) {
                        Ok(file) => {
                            SerializationResult::success(Value::String(file.to_display_string(Some(lang))))
                        }
                        Err(_) => SerializationResult::success(tile_data.clone()),
                    }
                }

                _ => SerializationResult::success(tile_data.clone()),
            }
        }

        fn description(&self) -> &str {
            "Display serializer for file-list type"
        }
    }

    /// Create a DisplaySerializerRegistry with file-list serializers.
    pub fn create_filelist_display_registry() -> DisplaySerializerRegistry {
        let mut registry = DisplaySerializerRegistry::new();
        registry.register("file-list", Arc::new(FileListDisplaySerializer));
        registry
    }
}

#[cfg(feature = "mutations")]
pub use display_serializer::{FileListDisplaySerializer, create_filelist_display_registry};

// =============================================================================
// Localized String Types
// =============================================================================

/// A localized string with direction and value.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalizedStringValue {
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub value: String,
}

/// A map of language codes to localized string values.
pub type LocalizedString = HashMap<String, LocalizedStringValue>;

// =============================================================================
// FileListItem Type
// =============================================================================

/// A single file in a file-list.
///
/// This matches the Arches file-list datatype structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileListItem {
    /// Whether the file has been accepted/approved
    #[serde(default)]
    pub accepted: bool,

    /// Alternative text for accessibility (localized)
    #[serde(default, rename = "altText")]
    pub alt_text: Option<LocalizedString>,

    /// Attribution/credits (localized)
    #[serde(default)]
    pub attribution: Option<LocalizedString>,

    /// Blob content URL (for uploads in progress)
    #[serde(default)]
    pub content: Option<String>,

    /// Description of the file (localized)
    #[serde(default)]
    pub description: Option<LocalizedString>,

    /// Unique file identifier (UUID)
    #[serde(default)]
    pub file_id: Option<String>,

    /// Index/order in the list
    #[serde(default)]
    pub index: Option<i64>,

    /// Last modified timestamp (milliseconds)
    #[serde(default, rename = "lastModified")]
    pub last_modified: Option<i64>,

    /// Original filename
    #[serde(default)]
    pub name: String,

    /// Storage path
    #[serde(default)]
    pub path: Option<String>,

    /// Whether this file is currently selected in UI
    #[serde(default)]
    pub selected: bool,

    /// File size in bytes
    #[serde(default)]
    pub size: Option<i64>,

    /// Upload status (e.g., "uploaded", "pending")
    #[serde(default)]
    pub status: Option<String>,

    /// Title/caption (localized)
    #[serde(default)]
    pub title: Option<LocalizedString>,

    /// MIME type
    #[serde(default, rename = "type")]
    pub file_type: Option<String>,

    /// URL to access the file
    #[serde(default)]
    pub url: Option<String>,

    /// Renderer for display
    #[serde(default)]
    pub renderer: Option<String>,

    /// Additional fields we might not know about
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl FileListItem {
    /// Get the display string for this file item.
    ///
    /// Uses title if available (in specified language), otherwise falls back to filename.
    pub fn to_display_string(&self, lang: Option<&str>) -> String {
        let target_lang = lang.unwrap_or("en");

        // Try title first
        if let Some(ref title) = self.title {
            if let Some(localized) = title.get(target_lang) {
                if !localized.value.is_empty() {
                    return localized.value.clone();
                }
            }
            // Try any language
            for (_lang, localized) in title {
                if !localized.value.is_empty() {
                    return localized.value.clone();
                }
            }
        }

        // Fall back to filename
        if !self.name.is_empty() {
            return self.name.clone();
        }

        // Last resort
        self.file_id.clone().unwrap_or_else(|| "(unnamed file)".to_string())
    }

    /// Get the alt text in a specific language.
    pub fn get_alt_text(&self, lang: Option<&str>) -> Option<String> {
        let target_lang = lang.unwrap_or("en");

        if let Some(ref alt_text) = self.alt_text {
            if let Some(localized) = alt_text.get(target_lang) {
                if !localized.value.is_empty() {
                    return Some(localized.value.clone());
                }
            }
        }
        None
    }

    /// Check if this is an image file based on MIME type.
    pub fn is_image(&self) -> bool {
        self.file_type
            .as_ref()
            .map(|t| t.starts_with("image/"))
            .unwrap_or(false)
    }
}

// =============================================================================
// Coercion Logic
// =============================================================================

/// Coerce a value to file-list tile data format.
fn coerce_filelist_value(value: &Value) -> Result<(Value, Value), String> {
    match value {
        // Already a file list array
        Value::Array(arr) => {
            let mut tile_data = Vec::new();
            let mut resolved = Vec::new();

            for (idx, item) in arr.iter().enumerate() {
                let (item_tile, item_resolved) = coerce_single_file(item, idx)?;
                tile_data.push(item_tile);
                resolved.push(item_resolved);
            }

            Ok((json!(tile_data), json!(resolved)))
        }

        // Single file object - wrap in array
        Value::Object(_) => {
            let (item_tile, item_resolved) = coerce_single_file(value, 0)?;
            Ok((json!([item_tile]), json!([item_resolved])))
        }

        Value::Null => Ok((Value::Null, Value::Null)),

        _ => Err(format!("Could not coerce value to file-list: {:?}", value)),
    }
}

/// Coerce a single file value.
fn coerce_single_file(value: &Value, index: usize) -> Result<(Value, Value), String> {
    match value {
        Value::Object(obj) => {
            // Already a FileListItem object - validate and pass through
            let mut file: FileListItem = serde_json::from_value(Value::Object(obj.clone()))
                .map_err(|e| format!("Invalid file object: {}", e))?;

            // Ensure index is set
            if file.index.is_none() {
                file.index = Some(index as i64);
            }

            let tile_data = serde_json::to_value(&file)
                .map_err(|e| format!("Failed to serialize file: {}", e))?;

            Ok((tile_data.clone(), tile_data))
        }

        // String URL - create minimal file object
        Value::String(url) => {
            let file = FileListItem {
                url: Some(url.clone()),
                name: url.split('/').last().unwrap_or("file").to_string(),
                index: Some(index as i64),
                status: Some("uploaded".to_string()),
                ..Default::default()
            };

            let tile_data = serde_json::to_value(&file)
                .map_err(|e| format!("Failed to serialize file: {}", e))?;

            Ok((tile_data.clone(), tile_data))
        }

        _ => Err(format!("Could not coerce single file from: {:?}", value)),
    }
}

// =============================================================================
// C ABI Handler
// =============================================================================

/// C ABI coercion function for file-list type.
unsafe extern "C" fn coerce_filelist(
    value_ptr: *const u8,
    value_len: usize,
    config_ptr: *const u8,
    config_len: usize,
) -> CoerceResult {
    // Parse value JSON
    let value_slice = std::slice::from_raw_parts(value_ptr, value_len);
    let value_str = match std::str::from_utf8(value_slice) {
        Ok(s) => s,
        Err(e) => return CoerceResult::error(format!("Invalid UTF-8 in value: {}", e)),
    };

    let value: Value = match serde_json::from_str(value_str) {
        Ok(v) => v,
        Err(e) => return CoerceResult::error(format!("Invalid JSON value: {}", e)),
    };

    // Parse config JSON
    // Config is accepted by the ABI but not currently used for coercion
    let _ = (config_ptr, config_len);

    // Perform coercion
    match coerce_filelist_value(&value) {
        Ok((tile_data, resolved)) => {
            let tile_json = serde_json::to_vec(&tile_data).unwrap_or_default();
            let resolved_json = serde_json::to_vec(&resolved).unwrap_or_default();
            CoerceResult::success(tile_json, resolved_json)
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
    // Parse resolved JSON
    let resolved_slice = std::slice::from_raw_parts(resolved_ptr, resolved_len);
    let resolved_str = match std::str::from_utf8(resolved_slice) {
        Ok(s) => s,
        Err(e) => return RenderDisplayResult::error(format!("Invalid UTF-8 in resolved: {}", e)),
    };

    // Parse language
    let lang_slice = std::slice::from_raw_parts(lang_ptr, lang_len);
    let lang = std::str::from_utf8(lang_slice).ok();

    // Handle arrays of files
    let resolved: Value = match serde_json::from_str(resolved_str) {
        Ok(v) => v,
        Err(e) => return RenderDisplayResult::error(format!("Invalid JSON: {}", e)),
    };

    match &resolved {
        // Single file object
        Value::Object(_) => {
            let file: FileListItem = match serde_json::from_value(resolved.clone()) {
                Ok(f) => f,
                Err(e) => return RenderDisplayResult::error(format!("Invalid file: {}", e)),
            };
            RenderDisplayResult::success(file.to_display_string(lang))
        }

        // Array of files - join names with ", "
        Value::Array(arr) => {
            let mut displays = Vec::new();
            for item in arr {
                match serde_json::from_value::<FileListItem>(item.clone()) {
                    Ok(file) => displays.push(file.to_display_string(lang)),
                    Err(_) => continue,
                }
            }
            RenderDisplayResult::success(displays.join(", "))
        }

        Value::Null => RenderDisplayResult::success(String::new()),

        _ => RenderDisplayResult::error(format!("Unexpected resolved type: {:?}", resolved)),
    }
}

// =============================================================================
// Python Module
// =============================================================================

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::PyCapsule;
    use std::ffi::{c_void, CString};
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
        let ptr = unsafe { HANDLER_INFO.as_ref().unwrap() as *const TypeHandlerInfo };

        let name = CString::new("alizarin_filelist.filelist_handler").unwrap();

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
    pub fn _rust(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(get_filelist_handler_capsule, m)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let files_json = r#"[
            {"name": "file1.png", "title": {"en": {"direction": "ltr", "value": "First File"}}},
            {"name": "file2.jpg"}
        ]"#;

        unsafe {
            let result = render_filelist_display(
                files_json.as_ptr(),
                files_json.len(),
                "en".as_ptr(),
                2,
            );

            assert!(!result.is_error());
            let display = std::str::from_utf8(
                std::slice::from_raw_parts(result.display_ptr, result.display_len)
            ).unwrap();
            assert_eq!(display, "First File, file2.jpg");

            alizarin_free_render_display_result(result);
        }
    }
}
