//! Shared CLM (Controlled List Manager) reference type logic.
//!
//! This crate provides the core types and `ExtensionTypeHandler` implementation
//! for the "reference" datatype. It is used by:
//! - `alizarin-napi` (Node.js bindings)
//! - `ext/python/alizarin-clm` (Python extension, via C ABI wrappers)
//! - `ext/js/@alizarin/clm` (WASM/JS extension, could replace TS reimplementation)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use alizarin_core::extension_type_registry::{
    ExtensionError, ExtensionTypeHandler, HandlerCapabilities,
};
use alizarin_core::type_coercion::CoercionResult;

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
    /// Get the display string for this reference.
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

/// Coerce a value to reference tile data format.
pub fn coerce_reference_value(
    value: &Value,
    config: &ReferenceNodeConfig,
) -> Result<(Value, Value), String> {
    fn coerce_single(
        value: &Value,
        config: &ReferenceNodeConfig,
    ) -> Result<(Value, Value), String> {
        match value {
            Value::Object(obj) if obj.contains_key("labels") || obj.contains_key("list_id") => {
                Err(format!(
                    "Pre-formed reference objects are not valid input. \
                     Use a label string or UUID instead. Got: {:?}",
                    value
                ))
            }

            Value::String(s) => {
                let uuid_regex = regex::Regex::new(
                    r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                )
                .expect("hardcoded UUID regex is valid");

                if uuid_regex.is_match(s) {
                    Ok((
                        json!({"__needs_rdm_lookup": true, "uuid": s}),
                        json!({"__needs_rdm_lookup": true, "uuid": s}),
                    ))
                } else {
                    Ok((
                        json!({"__needs_rdm_label_lookup": true, "label": s, "controlledList": config.controlled_list}),
                        json!({"__needs_rdm_label_lookup": true, "label": s, "controlledList": config.controlled_list}),
                    ))
                }
            }

            Value::Array(arr) => {
                let mut tile_data = Vec::new();
                let mut resolved = Vec::new();

                for item in arr {
                    let (item_tile, item_resolved) = coerce_single(item, config)?;
                    tile_data.push(item_tile);
                    resolved.push(item_resolved);
                }

                Ok((json!(tile_data), json!(resolved)))
            }

            Value::Null => Ok((Value::Null, Value::Null)),

            _ => Err(format!("Could not coerce value to reference: {:?}", value)),
        }
    }

    let (tile_data, resolved) = coerce_single(value, config)?;

    if config.multi_value == Some(true) && !matches!(tile_data, Value::Array(_) | Value::Null) {
        Ok((json!([tile_data]), json!([resolved])))
    } else {
        Ok((tile_data, resolved))
    }
}

// =============================================================================
// Display Rendering Logic
// =============================================================================

/// Render a reference value (single object, array, or null) to a display string.
pub fn render_reference_display_value(
    resolved: &Value,
    lang: Option<&str>,
) -> Result<String, String> {
    match resolved {
        Value::Object(_) => {
            let reference: StaticReference = serde_json::from_value(resolved.clone())
                .map_err(|e| format!("Invalid reference: {}", e))?;
            Ok(reference.to_display_string(lang))
        }

        Value::Array(arr) => {
            let mut displays = Vec::new();
            for item in arr {
                match serde_json::from_value::<StaticReference>(item.clone()) {
                    Ok(reference) => displays.push(reference.to_display_string(lang)),
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
// Concept Building
// =============================================================================

/// Build a StaticReference from RDM concept JSON.
pub fn build_static_reference_from_concept(
    concept: &Value,
    collection_id: &str,
) -> Result<StaticReference, String> {
    let concept_id = concept
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id in concept")?;

    let uri = format!("urn:uuid:{}", concept_id);

    let mut labels = Vec::new();
    if let Some(pref_label) = concept.get("pref_label").and_then(|v| v.as_object()) {
        for (lang_id, value) in pref_label {
            let label_value = if let Some(s) = value.as_str() {
                Some(s.to_string())
            } else if let Some(obj) = value.as_object() {
                obj.get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };

            if let Some(label_text) = label_value {
                let label_id = value
                    .as_object()
                    .and_then(|obj| obj.get("id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{}-{}", concept_id, lang_id));

                labels.push(StaticReferenceLabel {
                    id: label_id,
                    language_id: lang_id.clone(),
                    list_item_id: concept_id.to_string(),
                    value: label_text,
                    valuetype_id: "prefLabel".to_string(),
                });
            }
        }
    }

    if labels.is_empty() {
        if let Some(label) = concept.get("label").and_then(|v| v.as_str()) {
            labels.push(StaticReferenceLabel {
                id: format!("{}-en", concept_id),
                language_id: "en".to_string(),
                list_item_id: concept_id.to_string(),
                value: label.to_string(),
                valuetype_id: "prefLabel".to_string(),
            });
        }
    }

    Ok(StaticReference {
        labels,
        list_id: collection_id.to_string(),
        uri,
    })
}

// =============================================================================
// ExtensionTypeHandler Implementation
// =============================================================================

/// Reference type handler implementing `ExtensionTypeHandler`.
///
/// Provides coercion and display rendering for the "reference" datatype.
/// Marker resolution is not included here as it requires external callbacks
/// (RDM lookups) that are platform-specific.
pub struct ReferenceTypeHandler;

impl ExtensionTypeHandler for ReferenceTypeHandler {
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
        config: Option<&Value>,
    ) -> Result<CoercionResult, ExtensionError> {
        let node_config: ReferenceNodeConfig = config
            .map(|c| serde_json::from_value(c.clone()).unwrap_or_default())
            .unwrap_or_default();

        match coerce_reference_value(value, &node_config) {
            Ok((tile_data, display_value)) => Ok(CoercionResult::success(tile_data, display_value)),
            Err(e) => Err(ExtensionError::new(e)),
        }
    }

    fn render_display(
        &self,
        tile_data: &Value,
        language: &str,
    ) -> Result<Option<String>, ExtensionError> {
        match render_reference_display_value(tile_data, Some(language)) {
            Ok(s) if s.is_empty() => Ok(None),
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(ExtensionError::new(e)),
        }
    }

    fn description(&self) -> &str {
        "CLM reference type handler"
    }
}

/// Create an `Arc<dyn ExtensionTypeHandler>` for the reference type.
pub fn create_reference_handler() -> Arc<dyn ExtensionTypeHandler> {
    Arc::new(ReferenceTypeHandler)
}

/// The datatype name this handler registers for.
pub const DATATYPE_NAME: &str = "reference";

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(reference.to_display_string(None), "English Label");
    }

    #[test]
    fn test_coerce_rejects_preformed() {
        let value = json!({
            "labels": [],
            "list_id": "list-1",
            "uri": "http://example.com"
        });
        let config = ReferenceNodeConfig::default();
        assert!(coerce_reference_value(&value, &config).is_err());
    }

    #[test]
    fn test_coerce_uuid_string() {
        let value = json!("550e8400-e29b-41d4-a716-446655440000");
        let config = ReferenceNodeConfig::default();
        let (tile_data, _) = coerce_reference_value(&value, &config).unwrap();
        assert_eq!(
            tile_data
                .get("__needs_rdm_lookup")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_render_display_single() {
        let resolved = json!({
            "labels": [{"id": "1", "language_id": "en", "list_item_id": "item-1", "value": "Test Label", "valuetype_id": "prefLabel"}],
            "list_id": "list-1",
            "uri": "http://example.com"
        });
        assert_eq!(
            render_reference_display_value(&resolved, Some("en")).unwrap(),
            "Test Label"
        );
    }

    #[test]
    fn test_render_display_array() {
        let resolved = json!([
            {"labels": [{"id": "1", "language_id": "en", "list_item_id": "a", "value": "Label A", "valuetype_id": "prefLabel"}], "list_id": "l", "uri": "u"},
            {"labels": [{"id": "2", "language_id": "en", "list_item_id": "b", "value": "Label B", "valuetype_id": "prefLabel"}], "list_id": "l", "uri": "u"}
        ]);
        assert_eq!(
            render_reference_display_value(&resolved, Some("en")).unwrap(),
            "Label A, Label B"
        );
    }

    #[test]
    fn test_handler_trait() {
        let handler = ReferenceTypeHandler;
        let caps = handler.capabilities();
        assert!(caps.can_coerce);
        assert!(caps.can_render_display);
        assert!(!caps.can_resolve_markers);

        let resolved = json!({
            "labels": [{"id": "1", "language_id": "en", "list_item_id": "i", "value": "Display", "valuetype_id": "prefLabel"}],
            "list_id": "l",
            "uri": "u"
        });
        let display = handler.render_display(&resolved, "en").unwrap();
        assert_eq!(display, Some("Display".to_string()));
    }

    #[test]
    fn test_multivalue_wraps() {
        let value = json!("550e8400-e29b-41d4-a716-446655440000");
        let config = ReferenceNodeConfig {
            multi_value: Some(true),
            ..Default::default()
        };
        let (tile_data, _) = coerce_reference_value(&value, &config).unwrap();
        assert!(tile_data.is_array());
        assert_eq!(tile_data.as_array().unwrap().len(), 1);
    }
}
