//! Shared CLM (Controlled List Manager) reference type logic.
//!
//! This crate provides the core types and `ExtensionTypeHandler` implementation
//! for the "reference" datatype. It is used by:
//! - `alizarin-napi` (Node.js bindings)
//! - `ext/alizarin-clm/python` (Python extension, via C ABI wrappers)
//! - `ext/alizarin-clm/js` (JS extension)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};

use alizarin_core::extension_type_registry::{
    ExtensionError, ExtensionTypeHandler, HandlerCapabilities,
};
use alizarin_core::type_coercion::CoercionResult;
use alizarin_core::type_serialization::{ExternalResolver, SerializationContext};

// =============================================================================
// CLM Base URI Configuration
// =============================================================================

lazy_static::lazy_static! {
    static ref CLM_BASE_URI: RwLock<Option<String>> = RwLock::new(None);
}

pub fn set_clm_base_uri(uri: &str) {
    let normalized = if uri.ends_with('/') {
        uri.to_string()
    } else {
        format!("{}/", uri)
    };
    if let Ok(mut guard) = CLM_BASE_URI.write() {
        *guard = Some(normalized);
    }
}

/// Get the current CLM base URI, if set.
pub fn get_clm_base_uri() -> Option<String> {
    CLM_BASE_URI.read().ok().and_then(|guard| guard.clone())
}

/// Clear the CLM base URI.
pub fn clear_clm_base_uri() {
    if let Ok(mut guard) = CLM_BASE_URI.write() {
        *guard = None;
    }
}

/// Build the URI for a list item given its UUID.
///
/// Uses the configured CLM base URI. Panics if not set — call `set_clm_base_uri`
/// before generating references.
pub fn build_item_uri(item_id: &str) -> String {
    match get_clm_base_uri() {
        Some(base) => format!("{}{}", base, item_id),
        None => panic!(
            "CLM base URI not configured. Call set_clm_base_uri() with \
             PUBLIC_SERVER_ADDRESS before generating reference values."
        ),
    }
}

/// Build the URI for a list item, returning an error if CLM base URI is not configured.
pub fn try_build_item_uri(item_id: &str) -> Result<String, String> {
    match get_clm_base_uri() {
        Some(base) => Ok(format!("{}{}", base, item_id)),
        None => Err("CLM base URI not configured. Call set_clm_base_uri() with \
             PUBLIC_SERVER_ADDRESS before generating reference values."
            .to_string()),
    }
}

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
                        json!({"__needs_rdm_lookup": true, "uuid": s, "controlledList": config.controlled_list}),
                        json!({"__needs_rdm_lookup": true, "uuid": s, "controlledList": config.controlled_list}),
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

    // Always wrap in array for tile data, even for multiValue=false
    if !matches!(tile_data, Value::Array(_) | Value::Null) {
        Ok((json!([tile_data]), json!([resolved])))
    } else {
        Ok((tile_data, resolved))
    }
}

// =============================================================================
// Index Key Extraction
// =============================================================================

#[cfg(test)]
fn is_uuid(s: &str) -> bool {
    let uuid_regex =
        regex::Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .expect("hardcoded UUID regex is valid");
    uuid_regex.is_match(s)
}

/// Extract the list-item UUID(s) a reference tile value indexes on.
#[cfg(test)]
fn reference_index_keys(value: &Value) -> Vec<String> {
    fn push_from(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::String(s) if is_uuid(s) => out.push(s.clone()),
            Value::String(_) => {}
            Value::Object(m) => {
                if let Some(u) = m.get("uuid").and_then(|v| v.as_str()) {
                    if is_uuid(u) {
                        out.push(u.to_string());
                    }
                } else if let Some(labels) = m.get("labels").and_then(|v| v.as_array()) {
                    // StaticReference: extract concept ID from first label's list_item_id
                    if let Some(list_item_id) = labels
                        .first()
                        .and_then(|l| l.get("list_item_id"))
                        .and_then(|v| v.as_str())
                    {
                        out.push(list_item_id.to_string());
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    push_from(item, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    push_from(value, &mut out);
    out
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

/// Render a reference value to a display string, resolving bare list-item UUIDs
/// and `__needs_rdm_lookup` markers through `resolver` when the value has not
/// already been resolved to a [`StaticReference`].
pub fn render_reference_display_value_resolved(
    tile_data: &Value,
    lang: Option<&str>,
    resolver: Option<&dyn ExternalResolver>,
    collection: Option<&str>,
) -> String {
    fn resolve_id(
        id: &str,
        lang: Option<&str>,
        resolver: Option<&dyn ExternalResolver>,
        collection: Option<&str>,
    ) -> String {
        resolver
            .and_then(|r| r.resolve_concept(collection.unwrap_or(""), id, lang.unwrap_or("en")))
            .unwrap_or_else(|| id.to_string())
    }

    fn render_one(
        value: &Value,
        lang: Option<&str>,
        resolver: Option<&dyn ExternalResolver>,
        collection: Option<&str>,
    ) -> Option<String> {
        match value {
            Value::Null => None,
            Value::String(id) => Some(resolve_id(id, lang, resolver, collection)),
            Value::Object(obj) => {
                if obj.contains_key("labels") {
                    serde_json::from_value::<StaticReference>(value.clone())
                        .ok()
                        .map(|r| r.to_display_string(lang))
                } else if let Some(id) = obj.get("uuid").and_then(|v| v.as_str()) {
                    Some(resolve_id(id, lang, resolver, collection))
                } else {
                    obj.get("label")
                        .and_then(|v| v.as_str())
                        .map(|label| label.to_string())
                }
            }
            _ => None,
        }
    }

    match tile_data {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| render_one(item, lang, resolver, collection))
            .collect::<Vec<_>>()
            .join(", "),
        other => render_one(other, lang, resolver, collection).unwrap_or_default(),
    }
}

// =============================================================================
// Marker resolution (build/index side)
// =============================================================================

/// Build a StaticReference JSON value from concept JSON (as returned by RDM callbacks).
///
/// Thin wrapper around `build_static_reference_from_concept` that returns
/// serialized JSON rather than a typed struct.
pub fn build_reference_from_concept_json(
    concept_json: &Value,
    collection_id: &str,
) -> Option<Value> {
    build_static_reference_from_concept(concept_json, collection_id)
        .ok()
        .and_then(|sr| serde_json::to_value(sr).ok())
}

/// Resolve reference markers to full `StaticReference` objects using provided lookup functions.
///
/// This is the primary resolution function, used by the FFI layer where the RDM cache
/// is accessed via callbacks rather than a global static.
///
/// Per element of the (array-wrapped) tile value:
/// - `__needs_rdm_lookup{uuid, controlledList}` -> full `StaticReference` built from
///   the concept returned by `lookup_by_id`;
/// - `__needs_rdm_label_lookup{label, controlledList}` -> full `StaticReference` built
///   from the concept returned by `lookup_by_label`;
/// - a bare id string -> unchanged;
/// - anything already resolved (e.g. a `StaticReference` object) -> left as-is.
///
/// When lookup fails, markers pass through unchanged.
pub fn resolve_reference_markers_with_lookups<F1, F2>(
    tile_data: &Value,
    _language: &str,
    lookup_by_id: F1,
    lookup_by_label: F2,
) -> Value
where
    F1: Fn(&str, &str) -> Option<Value>,
    F2: Fn(&str, &str) -> Option<Value>,
{
    let resolve_one = |v: &Value| -> Option<Value> {
        match v {
            Value::Null => None,
            Value::String(_) => Some(v.clone()),
            Value::Object(obj) => {
                if obj.get("__needs_rdm_lookup").is_some() {
                    let uuid = obj.get("uuid").and_then(|u| u.as_str())?;
                    let collection = obj.get("controlledList").and_then(|c| c.as_str());

                    if let Some(coll_id) = collection {
                        if let Some(concept_json) = lookup_by_id(coll_id, uuid) {
                            if let Some(ref_val) =
                                build_reference_from_concept_json(&concept_json, coll_id)
                            {
                                return Some(ref_val);
                            }
                        }
                    }
                    return Some(v.clone());
                }
                if obj.get("__needs_rdm_label_lookup").is_some() {
                    let label = obj.get("label").and_then(|l| l.as_str())?;
                    let collection = obj.get("controlledList").and_then(|c| c.as_str())?;
                    if let Some(concept_json) = lookup_by_label(collection, label) {
                        if let Some(ref_val) =
                            build_reference_from_concept_json(&concept_json, collection)
                        {
                            return Some(ref_val);
                        }
                    }
                    return Some(v.clone());
                }
                Some(v.clone())
            }
            _ => Some(v.clone()),
        }
    };

    match tile_data {
        Value::Array(items) => Value::Array(items.iter().filter_map(resolve_one).collect()),
        Value::Null => Value::Null,
        other => resolve_one(other).unwrap_or(Value::Null),
    }
}

/// Resolve reference markers using the global RDM cache.
///
/// Convenience wrapper around `resolve_reference_markers_with_lookups` that uses
/// `alizarin_core::with_global_rdm_cache` for lookups. Suitable for standalone
/// (non-FFI) use where the global cache is in the same binary.
pub fn resolve_reference_markers(tile_data: &Value, language: &str) -> Value {
    resolve_reference_markers_with_lookups(
        tile_data,
        language,
        |collection_id, concept_id| {
            alizarin_core::with_global_rdm_cache(|cache| {
                cache.lookup_concept(collection_id, concept_id).map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "pref_label": c.pref_label,
                    })
                })
            })
            .flatten()
        },
        |collection_id, label| {
            alizarin_core::with_global_rdm_cache(|cache| {
                cache.lookup_by_label(collection_id, label).map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "pref_label": c.pref_label,
                    })
                })
            })
            .flatten()
        },
    )
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

    let uri = try_build_item_uri(concept_id)?;

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
            can_resolve_markers: true,
            can_index: false,
        }
    }

    fn coerce(
        &self,
        value: &Value,
        config: Option<&Value>,
    ) -> Result<CoercionResult, ExtensionError> {
        let node_config: ReferenceNodeConfig = match config {
            Some(c) => serde_json::from_value(c.clone())
                .map_err(|e| ExtensionError::new(format!("Invalid CLM node config: {}", e)))?,
            None => ReferenceNodeConfig::default(),
        };

        match coerce_reference_value(value, &node_config) {
            Ok((tile_data, display_value)) => Ok(CoercionResult::success(tile_data, display_value)),
            Err(e) => Err(ExtensionError::new(e)),
        }
    }

    fn render_display(
        &self,
        tile_data: &Value,
        language: &str,
        ctx: Option<&SerializationContext>,
    ) -> Result<Option<String>, ExtensionError> {
        let resolver = ctx.and_then(|c| c.external_resolver);
        let collection = ctx
            .and_then(|c| c.node_config)
            .and_then(|nc| nc.as_reference())
            .and_then(|r| r.get_collection_id());
        let display = render_reference_display_value_resolved(
            tile_data,
            Some(language),
            resolver,
            collection,
        );
        if display.is_empty() {
            Ok(None)
        } else {
            Ok(Some(display))
        }
    }

    fn resolve_markers(&self, tile_data: &Value, language: &str) -> Result<Value, ExtensionError> {
        Ok(resolve_reference_markers(tile_data, language))
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
        // Even multiValue=false should produce a one-element array
        assert!(tile_data.is_array());
        let arr = tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(
            arr[0].get("__needs_rdm_lookup").and_then(|v| v.as_bool()),
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
    fn test_render_resolves_bare_uuid_via_resolver() {
        struct MockResolver;
        impl ExternalResolver for MockResolver {
            fn resolve_concept(&self, _c: &str, id: &str, _l: &str) -> Option<String> {
                match id {
                    "9239cbe9-9571-4e5a-9c7f-000000000001" => Some("CC BY-SA 4.0".to_string()),
                    "9239cbe9-9571-4e5a-9c7f-000000000002" => Some("Public Domain".to_string()),
                    _ => None,
                }
            }
        }
        let r = MockResolver;

        let single = json!(["9239cbe9-9571-4e5a-9c7f-000000000001"]);
        assert_eq!(
            render_reference_display_value_resolved(&single, Some("en"), Some(&r), None),
            "CC BY-SA 4.0"
        );

        let multi = json!([
            "9239cbe9-9571-4e5a-9c7f-000000000001",
            "9239cbe9-9571-4e5a-9c7f-000000000002"
        ]);
        assert_eq!(
            render_reference_display_value_resolved(&multi, Some("en"), Some(&r), None),
            "CC BY-SA 4.0, Public Domain"
        );

        let unknown = json!(["deadbeef-0000-0000-0000-000000000000"]);
        assert_eq!(
            render_reference_display_value_resolved(&unknown, Some("en"), Some(&r), None),
            "deadbeef-0000-0000-0000-000000000000"
        );

        assert_eq!(
            render_reference_display_value_resolved(&single, Some("en"), None, None),
            "9239cbe9-9571-4e5a-9c7f-000000000001"
        );

        let resolved = json!([{
            "labels": [{"id": "1", "language_id": "en", "list_item_id": "i", "value": "Pre-Resolved", "valuetype_id": "prefLabel"}],
            "list_id": "l",
            "uri": "u"
        }]);
        assert_eq!(
            render_reference_display_value_resolved(&resolved, Some("en"), Some(&r), None),
            "Pre-Resolved"
        );
    }

    #[test]
    fn test_handler_trait() {
        let handler = ReferenceTypeHandler;
        let caps = handler.capabilities();
        assert!(caps.can_coerce);
        assert!(caps.can_render_display);
        assert!(caps.can_resolve_markers);

        let resolved = json!({
            "labels": [{"id": "1", "language_id": "en", "list_item_id": "i", "value": "Display", "valuetype_id": "prefLabel"}],
            "list_id": "l",
            "uri": "u"
        });
        let display = handler.render_display(&resolved, "en", None).unwrap();
        assert_eq!(display, Some("Display".to_string()));
    }

    #[test]
    fn test_resolve_markers_to_static_references() {
        use alizarin_core::{set_global_rdm_cache, RdmCache, RdmCollection};

        set_clm_base_uri("http://localhost:8000/plugins/controlled-list-manager/item/");

        const NOUN: &str = "1052ed22-def2-5e6b-a5a2-ddff79e08e70";
        let coll = RdmCollection::from_concepts_json(
            "coll-pos".to_string(),
            r#"[{"id":"1052ed22-def2-5e6b-a5a2-ddff79e08e70","pref_label":{"en":{"id":"v1","value":"noun"}}}]"#,
        )
        .unwrap();
        let mut cache = RdmCache::new();
        cache.add_collection(coll);
        set_global_rdm_cache(cache);

        let handler = ReferenceTypeHandler;

        // Label marker → full StaticReference
        let label_marker = json!([{"__needs_rdm_label_lookup": true, "label": "noun", "controlledList": "coll-pos"}]);
        let resolved = handler.resolve_markers(&label_marker, "en").unwrap();
        let resolved_arr = resolved.as_array().unwrap();
        assert_eq!(resolved_arr.len(), 1);
        assert!(
            resolved_arr[0].get("labels").is_some(),
            "Should be StaticReference"
        );
        assert_eq!(resolved_arr[0]["labels"][0]["list_item_id"], NOUN);
        assert_eq!(resolved_arr[0]["labels"][0]["value"], "noun");
        assert_eq!(resolved_arr[0]["list_id"], "coll-pos");
        // reference_index_keys should extract concept ID from StaticReference
        assert_eq!(reference_index_keys(&resolved), vec![NOUN.to_string()]);

        // UUID marker with controlledList → full StaticReference
        let uuid_marker =
            json!([{"__needs_rdm_lookup": true, "uuid": NOUN, "controlledList": "coll-pos"}]);
        let uuid_resolved = handler.resolve_markers(&uuid_marker, "en").unwrap();
        assert!(
            uuid_resolved[0].get("labels").is_some(),
            "Should be StaticReference"
        );
        assert_eq!(reference_index_keys(&uuid_resolved), vec![NOUN.to_string()]);

        // Bare UUID string passes through unchanged
        assert_eq!(
            handler.resolve_markers(&json!([NOUN]), "en").unwrap(),
            json!([NOUN])
        );

        // Missing label → marker passed through (not dropped)
        let miss = json!([{"__needs_rdm_label_lookup": true, "label": "verb", "controlledList": "coll-pos"}]);
        let miss_resolved = handler.resolve_markers(&miss, "en").unwrap();
        let miss_arr = miss_resolved.as_array().unwrap();
        assert_eq!(miss_arr.len(), 1);
        assert_eq!(
            miss_arr[0]
                .get("__needs_rdm_label_lookup")
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        // Coerced label → resolves to StaticReference
        let cfg = ReferenceNodeConfig {
            controlled_list: Some("coll-pos".to_string()),
            ..Default::default()
        };
        let (from_name, _) = coerce_reference_value(&json!("noun"), &cfg).unwrap();
        let from_name_resolved = handler.resolve_markers(&from_name, "en").unwrap();
        assert!(from_name_resolved[0].get("labels").is_some());
        assert_eq!(
            reference_index_keys(&from_name_resolved),
            vec![NOUN.to_string()]
        );

        // Coerced UUID → resolves to StaticReference (now includes controlledList)
        let (from_id, _) = coerce_reference_value(&json!(NOUN), &cfg).unwrap();
        assert!(from_id.is_array());
        let from_id_resolved = handler.resolve_markers(&from_id, "en").unwrap();
        assert!(from_id_resolved[0].get("labels").is_some());
        assert_eq!(
            reference_index_keys(&from_id_resolved),
            vec![NOUN.to_string()]
        );
    }

    #[test]
    fn test_single_value_also_wraps() {
        // Even multiValue=false produces array tile data
        let value = json!("550e8400-e29b-41d4-a716-446655440000");
        let config = ReferenceNodeConfig {
            multi_value: Some(false),
            ..Default::default()
        };
        let (tile_data, _) = coerce_reference_value(&value, &config).unwrap();
        assert!(tile_data.is_array());
        assert_eq!(tile_data.as_array().unwrap().len(), 1);
    }
}
