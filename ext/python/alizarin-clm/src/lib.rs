//! Alizarin CLM Extension
//!
//! This extension provides the "reference" datatype handler for Controlled List Manager
//! integration. It handles coercion of reference values to tile data format.
//!
//! ## Mutations
//!
//! When the `mutations` feature is enabled, this crate also provides:
//! - `ReferenceChangeCollectionHandler` - mutation to change a reference node's collection

#[cfg(feature = "pyo3-ext")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3-ext")]
use pyo3::types::PyCapsule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[cfg(feature = "pyo3-ext")]
use std::ffi::{c_void, CString};

use alizarin_extension_api::{
    alizarin_free_coerce_result, alizarin_free_render_display_result,
    CoerceFn, CoerceResult, FreeFn, TypeHandlerInfo,
    RenderDisplayFn, RenderDisplayResult, FreeDisplayFn,
};

// Re-export mutation types when feature is enabled
#[cfg(feature = "mutations")]
pub mod mutations;

// =============================================================================
// Static Reference Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticReferenceLabel {
    pub id: String,
    pub language_id: String,
    pub list_item_id: String,
    pub value: String,
    pub valuetype_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticReference {
    pub labels: Vec<StaticReferenceLabel>,
    pub list_id: String,
    pub uri: String,
}

impl StaticReference {
    /// Get the display string for this reference
    pub fn to_display_string(&self, lang: Option<&str>) -> String {
        if self.labels.len() == 1 {
            return self.labels[0].value.clone();
        }

        let target_lang = lang.unwrap_or("en");
        let mut pref_label: Option<&str> = None;

        for label in &self.labels {
            if label.valuetype_id == "prefLabel" {
                pref_label = Some(&label.value);
                if label.language_id == target_lang {
                    return label.value.clone();
                }
            }
        }

        pref_label.unwrap_or("(undefined)").to_string()
    }
}

// =============================================================================
// Node Config for Reference Type
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReferenceNodeConfig {
    #[serde(rename = "controlledList")]
    pub controlled_list: Option<String>,
    #[serde(rename = "rdmCollection")]
    pub rdm_collection: Option<String>,
    #[serde(rename = "multiValue")]
    pub multi_value: Option<bool>,
}

impl ReferenceNodeConfig {
    pub fn get_collection_id(&self) -> Option<&str> {
        self.controlled_list
            .as_deref()
            .or(self.rdm_collection.as_deref())
    }

    pub fn is_multi_value(&self) -> bool {
        self.multi_value.unwrap_or(false)
    }
}

// =============================================================================
// Coercion Logic
// =============================================================================

/// Coerce a value to reference tile data format
fn coerce_reference_value(value: &Value, _config: &ReferenceNodeConfig) -> Result<(Value, Value), String> {
    match value {
        // Already a StaticReference object
        Value::Object(obj) if obj.contains_key("labels") && obj.contains_key("list_id") => {
            // Validate and pass through
            let reference: StaticReference = serde_json::from_value(value.clone())
                .map_err(|e| format!("Invalid reference object: {}", e))?;

            let tile_data = serde_json::to_value(&reference)
                .map_err(|e| format!("Failed to serialize reference: {}", e))?;

            Ok((tile_data.clone(), tile_data))
        }

        // UUID string - needs RDM lookup (return marker for Python to handle)
        Value::String(s) => {
            let uuid_regex = regex::Regex::new(
                r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            ).unwrap();

            if uuid_regex.is_match(s) {
                // Return a marker that Python should do RDM lookup
                Ok((
                    json!({"__needs_rdm_lookup": true, "uuid": s}),
                    json!({"__needs_rdm_lookup": true, "uuid": s}),
                ))
            } else {
                Err(format!("Set references using UUIDs from collections, not arbitrary strings: {}", s))
            }
        }

        // Array of references (for list type)
        Value::Array(arr) => {
            let mut tile_data = Vec::new();
            let mut resolved = Vec::new();

            for item in arr {
                let (item_tile, item_resolved) = coerce_reference_value(item, _config)?;
                tile_data.push(item_tile);
                resolved.push(item_resolved);
            }

            Ok((json!(tile_data), json!(resolved)))
        }

        Value::Null => Ok((Value::Null, Value::Null)),

        _ => Err(format!("Could not coerce value to reference: {:?}", value)),
    }
}

// =============================================================================
// C ABI Handler
// =============================================================================

/// C ABI coercion function for reference type
unsafe extern "C" fn coerce_reference(
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
    let config: ReferenceNodeConfig = if config_len > 0 && !config_ptr.is_null() {
        let config_slice = std::slice::from_raw_parts(config_ptr, config_len);
        let config_str = match std::str::from_utf8(config_slice) {
            Ok(s) => s,
            Err(_) => return CoerceResult::error("Invalid UTF-8 in config".to_string()),
        };

        match serde_json::from_str(config_str) {
            Ok(c) => c,
            Err(_) => ReferenceNodeConfig::default(),
        }
    } else {
        ReferenceNodeConfig::default()
    };

    // Perform coercion
    match coerce_reference_value(&value, &config) {
        Ok((tile_data, resolved)) => {
            let tile_json = serde_json::to_vec(&tile_data).unwrap_or_default();
            let resolved_json = serde_json::to_vec(&resolved).unwrap_or_default();
            CoerceResult::success(tile_json, resolved_json)
        }
        Err(e) => CoerceResult::error(e),
    }
}

/// C ABI display render function for reference type
///
/// Takes a resolved StaticReference JSON and returns the display string
/// for the specified language.
unsafe extern "C" fn render_reference_display(
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

    // Handle arrays of references
    let resolved: Value = match serde_json::from_str(resolved_str) {
        Ok(v) => v,
        Err(e) => return RenderDisplayResult::error(format!("Invalid JSON: {}", e)),
    };

    match &resolved {
        // Single reference object
        Value::Object(_) => {
            let reference: StaticReference = match serde_json::from_value(resolved.clone()) {
                Ok(r) => r,
                Err(e) => return RenderDisplayResult::error(format!("Invalid reference: {}", e)),
            };
            RenderDisplayResult::success(reference.to_display_string(lang))
        }

        // Array of references - join with ", "
        Value::Array(arr) => {
            let mut displays = Vec::new();
            for item in arr {
                match serde_json::from_value::<StaticReference>(item.clone()) {
                    Ok(reference) => displays.push(reference.to_display_string(lang)),
                    Err(_) => continue, // Skip invalid items
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
    /// This must be static because the capsule needs a stable pointer
    static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
    static INIT: Once = Once::new();

    /// Get the type handler capsule for registration with alizarin
    ///
    /// This handler includes display rendering support for toDisplayJson().
    #[pyfunction]
    pub fn get_reference_handler_capsule(py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        static TYPE_NAME: &[u8] = b"reference";

        // Initialize the static handler info once
        INIT.call_once(|| {
            unsafe {
                HANDLER_INFO = Some(TypeHandlerInfo {
                    type_name_ptr: TYPE_NAME.as_ptr(),
                    type_name_len: TYPE_NAME.len(),
                    coerce_fn: coerce_reference as CoerceFn,
                    free_fn: alizarin_free_coerce_result as FreeFn,
                    render_display_fn: Some(render_reference_display as RenderDisplayFn),
                    free_display_fn: Some(alizarin_free_render_display_result as FreeDisplayFn),
                    user_data: std::ptr::null_mut(),
                });
            }
        });

        // Get pointer to the static handler info
        let ptr = unsafe { HANDLER_INFO.as_ref().unwrap() as *const TypeHandlerInfo };

        // Note: The capsule name must be a C string (null-terminated)
        let name = CString::new("alizarin_clm.reference_handler").unwrap();

        // Create capsule using unsafe raw pointer approach
        // PyCapsule::new expects something that implements Send, so we use the raw FFI
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
        m.add_function(wrap_pyfunction!(get_reference_handler_capsule, m)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coerce_reference_object() {
        let value = json!({
            "labels": [{
                "id": "label-1",
                "language_id": "en",
                "list_item_id": "item-1",
                "value": "Test Item",
                "valuetype_id": "prefLabel"
            }],
            "list_id": "list-1",
            "uri": "http://example.com/item/1"
        });

        let config = ReferenceNodeConfig::default();
        let result = coerce_reference_value(&value, &config);

        assert!(result.is_ok());
        let (tile_data, _resolved) = result.unwrap();
        assert!(tile_data.get("labels").is_some());
    }

    #[test]
    fn test_coerce_uuid_string() {
        let value = json!("550e8400-e29b-41d4-a716-446655440000");
        let config = ReferenceNodeConfig::default();
        let result = coerce_reference_value(&value, &config);

        assert!(result.is_ok());
        let (tile_data, _) = result.unwrap();
        assert_eq!(tile_data.get("__needs_rdm_lookup").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_static_reference_display() {
        let reference = StaticReference {
            labels: vec![
                StaticReferenceLabel {
                    id: "1".to_string(),
                    language_id: "en".to_string(),
                    list_item_id: "item-1".to_string(),
                    value: "English Label".to_string(),
                    valuetype_id: "prefLabel".to_string(),
                },
                StaticReferenceLabel {
                    id: "2".to_string(),
                    language_id: "es".to_string(),
                    list_item_id: "item-1".to_string(),
                    value: "Etiqueta Española".to_string(),
                    valuetype_id: "prefLabel".to_string(),
                },
            ],
            list_id: "list-1".to_string(),
            uri: "http://example.com".to_string(),
        };

        assert_eq!(reference.to_display_string(Some("en")), "English Label");
        assert_eq!(reference.to_display_string(Some("es")), "Etiqueta Española");
        assert_eq!(reference.to_display_string(None), "English Label"); // default to en
    }

    #[test]
    fn test_render_reference_display() {
        let reference_json = r#"{
            "labels": [{
                "id": "1",
                "language_id": "en",
                "list_item_id": "item-1",
                "value": "Test Label",
                "valuetype_id": "prefLabel"
            }],
            "list_id": "list-1",
            "uri": "http://example.com"
        }"#;

        unsafe {
            let result = render_reference_display(
                reference_json.as_ptr(),
                reference_json.len(),
                "en".as_ptr(),
                2,
            );

            assert!(!result.is_error());
            let display = std::str::from_utf8(
                std::slice::from_raw_parts(result.display_ptr, result.display_len)
            ).unwrap();
            assert_eq!(display, "Test Label");

            alizarin_free_render_display_result(result);
        }
    }

    #[test]
    fn test_render_reference_display_array() {
        let references_json = r#"[
            {
                "labels": [{"id": "1", "language_id": "en", "list_item_id": "item-1", "value": "Label A", "valuetype_id": "prefLabel"}],
                "list_id": "list-1",
                "uri": "http://example.com/a"
            },
            {
                "labels": [{"id": "2", "language_id": "en", "list_item_id": "item-2", "value": "Label B", "valuetype_id": "prefLabel"}],
                "list_id": "list-1",
                "uri": "http://example.com/b"
            }
        ]"#;

        unsafe {
            let result = render_reference_display(
                references_json.as_ptr(),
                references_json.len(),
                "en".as_ptr(),
                2,
            );

            assert!(!result.is_error());
            let display = std::str::from_utf8(
                std::slice::from_raw_parts(result.display_ptr, result.display_len)
            ).unwrap();
            assert_eq!(display, "Label A, Label B");

            alizarin_free_render_display_result(result);
        }
    }
}
