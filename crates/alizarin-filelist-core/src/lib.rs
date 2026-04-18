//! Shared file-list type logic.
//!
//! This crate provides the core types and `ExtensionTypeHandler` implementation
//! for the "file-list" datatype. It is used by:
//! - `alizarin-napi` (Node.js bindings)
//! - `ext/python/alizarin-filelist` (Python extension, via C ABI wrappers)
//! - `ext/js/@alizarin/filelist` (WASM/JS extension, could replace TS reimplementation)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use alizarin_core::extension_type_registry::{
    ExtensionError, ExtensionTypeHandler, HandlerCapabilities,
};
use alizarin_core::type_coercion::CoercionResult;

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileListItem {
    #[serde(default)]
    pub accepted: bool,

    #[serde(default)]
    pub alt_text: Option<LocalizedString>,

    #[serde(default)]
    pub attribution: Option<LocalizedString>,

    #[serde(default)]
    pub content: Option<String>,

    #[serde(default)]
    pub description: Option<LocalizedString>,

    #[serde(default)]
    pub file_id: Option<String>,

    #[serde(default)]
    pub index: Option<i64>,

    #[serde(default)]
    pub last_modified: Option<i64>,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub selected: bool,

    #[serde(default)]
    pub size: Option<i64>,

    #[serde(default)]
    pub status: Option<String>,

    #[serde(default)]
    pub title: Option<LocalizedString>,

    #[serde(default, rename = "type")]
    pub file_type: Option<String>,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub renderer: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl FileListItem {
    /// Get the display string for this file item.
    pub fn to_display_string(&self, lang: Option<&str>) -> String {
        let target_lang = lang.unwrap_or("en");

        if let Some(ref title) = self.title {
            if let Some(localized) = title.get(target_lang) {
                if !localized.value.is_empty() {
                    return localized.value.clone();
                }
            }
            for localized in title.values() {
                if !localized.value.is_empty() {
                    return localized.value.clone();
                }
            }
        }

        if !self.name.is_empty() {
            return self.name.clone();
        }

        self.file_id
            .clone()
            .unwrap_or_else(|| "(unnamed file)".to_string())
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
pub fn coerce_filelist_value(value: &Value) -> Result<(Value, Value), String> {
    match value {
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

        Value::Object(_) => {
            let (item_tile, item_resolved) = coerce_single_file(value, 0)?;
            Ok((json!([item_tile]), json!([item_resolved])))
        }

        Value::Null => Ok((Value::Null, Value::Null)),

        _ => Err(format!("Could not coerce value to file-list: {:?}", value)),
    }
}

/// Coerce a single file value.
pub fn coerce_single_file(value: &Value, index: usize) -> Result<(Value, Value), String> {
    match value {
        Value::Object(obj) => {
            let mut file: FileListItem = serde_json::from_value(Value::Object(obj.clone()))
                .map_err(|e| format!("Invalid file object: {}", e))?;

            if file.index.is_none() {
                file.index = Some(index as i64);
            }

            let tile_data = serde_json::to_value(&file)
                .map_err(|e| format!("Failed to serialize file: {}", e))?;

            Ok((tile_data.clone(), tile_data))
        }

        Value::String(url) => {
            let file = FileListItem {
                url: Some(url.clone()),
                name: url.split('/').next_back().unwrap_or("file").to_string(),
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
// Display Rendering Logic
// =============================================================================

/// Render a file-list value to a display string.
pub fn render_filelist_display_value(
    resolved: &Value,
    lang: Option<&str>,
) -> Result<String, String> {
    match resolved {
        Value::Object(_) => {
            let file: FileListItem = serde_json::from_value(resolved.clone())
                .map_err(|e| format!("Invalid file: {}", e))?;
            Ok(file.to_display_string(lang))
        }

        Value::Array(arr) => {
            let mut displays = Vec::new();
            for item in arr {
                match serde_json::from_value::<FileListItem>(item.clone()) {
                    Ok(file) => displays.push(file.to_display_string(lang)),
                    Err(_) => continue,
                }
            }
            Ok(displays.join(", "))
        }

        Value::Null => Ok(String::new()),

        _ => Err(format!("Unexpected resolved type: {:?}", resolved)),
    }
}

// =============================================================================
// ExtensionTypeHandler Implementation
// =============================================================================

/// File-list type handler implementing `ExtensionTypeHandler`.
pub struct FileListTypeHandler;

impl ExtensionTypeHandler for FileListTypeHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        HandlerCapabilities {
            can_coerce: true,
            can_render_display: true,
            can_render_search: false,
            can_resolve_markers: false,
        }
    }

    fn coerce(
        &self,
        value: &Value,
        _config: Option<&Value>,
    ) -> Result<CoercionResult, ExtensionError> {
        match coerce_filelist_value(value) {
            Ok((tile_data, display_value)) => Ok(CoercionResult::success(tile_data, display_value)),
            Err(e) => Err(ExtensionError::new(e)),
        }
    }

    fn render_display(
        &self,
        tile_data: &Value,
        language: &str,
    ) -> Result<Option<String>, ExtensionError> {
        match render_filelist_display_value(tile_data, Some(language)) {
            Ok(s) if s.is_empty() => Ok(None),
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(ExtensionError::new(e)),
        }
    }

    fn description(&self) -> &str {
        "File-list type handler"
    }
}

/// Create an `Arc<dyn ExtensionTypeHandler>` for the file-list type.
pub fn create_filelist_handler() -> Arc<dyn ExtensionTypeHandler> {
    Arc::new(FileListTypeHandler)
}

/// The datatype name this handler registers for.
pub const DATATYPE_NAME: &str = "file-list";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_display_with_title() {
        let file = FileListItem {
            name: "test.png".to_string(),
            title: Some({
                let mut m = HashMap::new();
                m.insert(
                    "en".to_string(),
                    LocalizedStringValue {
                        direction: "ltr".to_string(),
                        value: "My Image".to_string(),
                    },
                );
                m
            }),
            ..Default::default()
        };

        assert_eq!(file.to_display_string(Some("en")), "My Image");
    }

    #[test]
    fn test_file_display_fallback() {
        let file = FileListItem {
            name: "test.png".to_string(),
            ..Default::default()
        };
        assert_eq!(file.to_display_string(Some("en")), "test.png");
    }

    #[test]
    fn test_coerce_single_object() {
        let value = json!({"name": "file.png", "url": "/files/1"});
        let (tile_data, _) = coerce_filelist_value(&value).unwrap();
        assert!(tile_data.is_array());
        assert_eq!(tile_data.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_coerce_array() {
        let value = json!([
            {"name": "a.png", "url": "/a"},
            {"name": "b.jpg", "url": "/b"}
        ]);
        let (tile_data, _) = coerce_filelist_value(&value).unwrap();
        assert!(tile_data.is_array());
        assert_eq!(tile_data.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_render_display() {
        let resolved = json!([
            {"name": "file1.png", "title": {"en": {"direction": "ltr", "value": "First"}}},
            {"name": "file2.jpg"}
        ]);
        assert_eq!(
            render_filelist_display_value(&resolved, Some("en")).unwrap(),
            "First, file2.jpg"
        );
    }

    #[test]
    fn test_handler_trait() {
        let handler = FileListTypeHandler;
        let caps = handler.capabilities();
        assert!(caps.can_coerce);
        assert!(caps.can_render_display);

        let resolved = json!([{"name": "test.png"}]);
        let display = handler.render_display(&resolved, "en").unwrap();
        assert_eq!(display, Some("test.png".to_string()));
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
}
