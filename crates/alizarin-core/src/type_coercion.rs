//! Type coercion handlers for Alizarin datatypes.
//!
//! This module provides platform-agnostic type coercion/validation logic
//! that can be used from TypeScript (via WASM), Python (via PyO3), or native Rust.
//!
//! Phase 1: Simple scalars (Date, EDTF, NonLocalizedString, Number)
//! Phase 2: Dict types (String, Url, GeoJSON)

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::RwLock;

use crate::label_resolution::is_valid_uuid;

// =============================================================================
// Language configuration (mirrors JS getCurrentLanguage/setCurrentLanguage)
// =============================================================================

/// Default language when none is set
pub const DEFAULT_LANGUAGE: &str = "en";

lazy_static::lazy_static! {
    /// Global current language setting
    static ref CURRENT_LANGUAGE: RwLock<Option<String>> = RwLock::new(None);
}

/// Get the current language, defaulting to "en" if not set.
pub fn get_current_language() -> String {
    CURRENT_LANGUAGE
        .read()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
}

/// Set the current language.
pub fn set_current_language(lang: &str) {
    if let Ok(mut guard) = CURRENT_LANGUAGE.write() {
        *guard = Some(lang.to_string());
    }
}

/// Result of a coercion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoercionResult {
    /// The value suitable for tile storage (tile.data[nodeid])
    pub tile_data: Value,
    /// The "display" or resolved value (for ViewModel construction)
    pub display_value: Value,
    /// Error message if coercion failed
    pub error: Option<String>,
}

impl CoercionResult {
    pub fn success(tile_data: Value, display_value: Value) -> Self {
        CoercionResult {
            tile_data,
            display_value,
            error: None,
        }
    }

    pub fn success_same(value: Value) -> Self {
        CoercionResult {
            tile_data: value.clone(),
            display_value: value,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        CoercionResult {
            tile_data: Value::Null,
            display_value: Value::Null,
            error: Some(message.into()),
        }
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_null(&self) -> bool {
        self.tile_data.is_null() && self.error.is_none()
    }
}

// =============================================================================
// Number coercion
// =============================================================================

/// Coerce a value to a number.
///
/// Accepts: number, string (parseable as number), null
/// Returns: the numeric value or null
pub fn coerce_number(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::Number(n) => {
            CoercionResult::success_same(Value::Number(n.clone()))
        }
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(s) => {
            // Try to parse as number
            if let Ok(n) = s.parse::<i64>() {
                CoercionResult::success_same(Value::Number(n.into()))
            } else if let Ok(n) = s.parse::<f64>() {
                match serde_json::Number::from_f64(n) {
                    Some(num) => CoercionResult::success_same(Value::Number(num)),
                    None => CoercionResult::error(format!("Invalid number: {} (NaN or Infinity)", s)),
                }
            } else {
                CoercionResult::error(format!("Cannot parse '{}' as number", s))
            }
        }
        _ => CoercionResult::error(format!(
            "Expected number or numeric string, got {:?}",
            value_type_name(value)
        )),
    }
}

// =============================================================================
// NonLocalizedString coercion
// =============================================================================

/// Coerce a value to a non-localized string.
///
/// Accepts: string, null
/// Returns: the string value or null
pub fn coerce_non_localized_string(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(_) => CoercionResult::success_same(value.clone()),
        Value::Number(n) => {
            // Convert number to string
            CoercionResult::success_same(Value::String(n.to_string()))
        }
        Value::Bool(b) => {
            // Convert bool to string
            CoercionResult::success_same(Value::String(b.to_string()))
        }
        _ => CoercionResult::error(format!(
            "Expected string, got {:?}",
            value_type_name(value)
        )),
    }
}

// =============================================================================
// EDTF coercion (Extended Date/Time Format)
// =============================================================================

/// Coerce a value to an EDTF string.
///
/// EDTF is a string format, so this is similar to non-localized string
/// but could include validation in the future.
///
/// Accepts: string (EDTF format), null
/// Returns: the EDTF string or null
pub fn coerce_edtf(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(s) => {
            // Basic EDTF validation could be added here
            // For now, accept any non-empty string
            // EDTF examples: "2023", "2023-05", "2023-05-15", "2023?", "2023~", etc.
            if is_valid_edtf(s) {
                CoercionResult::success_same(value.clone())
            } else {
                // Be permissive - accept the string but log warning
                // In strict mode this could return an error
                CoercionResult::success_same(value.clone())
            }
        }
        _ => CoercionResult::error(format!(
            "Expected EDTF string, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Basic EDTF validation.
///
/// This is a simplified check - full EDTF validation would be more complex.
/// Returns true for strings that look like EDTF dates.
fn is_valid_edtf(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // EDTF can be quite complex. Basic patterns:
    // - Year: "2023", "2023?", "2023~", "-0500"
    // - Year-Month: "2023-05"
    // - Full date: "2023-05-15"
    // - Intervals: "2023/2024", "2023-05/2023-06"
    // - Uncertain: "2023?", "2023~", "2023%"
    // - Approximate: "2023~"
    // - Unknown: "XXXX", "2023-XX"
    // - Season: "2023-21" (spring), "2023-22" (summer), etc.

    // Simple heuristic: must start with digit, negative sign, or X
    let first = s.chars().next().unwrap();
    first.is_ascii_digit() || first == '-' || first == 'X' || first == 'x'
}

// =============================================================================
// Date coercion
// =============================================================================

/// Coerce a value to a date.
///
/// Accepts: ISO date string, null, object with 'en' key (legacy format)
/// Returns: ISO date string or null
pub fn coerce_date(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(s) => {
            // Validate and normalize the date
            match normalize_date_string(s) {
                Ok(normalized) => {
                    let tile_val = Value::String(normalized.clone());
                    CoercionResult::success(tile_val.clone(), tile_val)
                }
                Err(_) => {
                    // Be permissive - accept the string but note the issue
                    // The JS Date constructor will handle parsing
                    CoercionResult::success_same(value.clone())
                }
            }
        }
        Value::Object(obj) => {
            // Handle legacy format: {"en": "2023-05-15"}
            if let Some(Value::String(s)) = obj.get("en") {
                if s.is_empty() {
                    CoercionResult::success_same(Value::Null)
                } else {
                    // Extract the 'en' value
                    match normalize_date_string(s) {
                        Ok(normalized) => {
                            let tile_val = Value::String(normalized.clone());
                            CoercionResult::success(tile_val.clone(), tile_val)
                        }
                        Err(_) => {
                            CoercionResult::success_same(Value::String(s.clone()))
                        }
                    }
                }
            } else {
                CoercionResult::error("Date object must have 'en' key with string value")
            }
        }
        _ => CoercionResult::error(format!(
            "Expected date string, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Normalize a date string to ISO 8601 format.
///
/// This is a basic implementation - could be enhanced with chrono crate.
fn normalize_date_string(s: &str) -> Result<String, String> {
    // ISO 8601 formats we accept:
    // - "2023-05-15" (date only)
    // - "2023-05-15T10:30:00" (datetime)
    // - "2023-05-15T10:30:00Z" (datetime with Z)
    // - "2023-05-15T10:30:00+00:00" (datetime with offset)
    // - "2023-05-15T10:30:00.000Z" (datetime with milliseconds)

    let s = s.trim();
    if s.is_empty() {
        return Err("Empty date string".to_string());
    }

    // Basic validation: should start with a year-like pattern
    // More thorough validation would use a date parsing library
    if s.len() >= 4 {
        let year_part = &s[..4];
        if year_part.chars().all(|c| c.is_ascii_digit()) ||
           (s.starts_with('-') && s.len() >= 5 && s[1..5].chars().all(|c| c.is_ascii_digit())) {
            return Ok(s.to_string());
        }
    }

    Err(format!("Invalid date format: {}", s))
}

// =============================================================================
// Phase 2: Dict types (String, Url, GeoJSON)
// These receive tile data format, not plain coerceable values
// =============================================================================

/// Coerce a value to a localized string (dict of language -> value).
///
/// Tile data format: `{"en": "Hello", "es": "Hola", ...}`
///
/// If input is a plain string, wraps it in the current language key.
/// If input is already a dict, passes through with validation.
pub fn coerce_string(value: &Value, language: Option<&str>) -> CoercionResult {
    let lang: String = language
        .map(|s| s.to_string())
        .unwrap_or_else(get_current_language);

    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            // Empty string -> empty object for that language
            let mut obj = serde_json::Map::new();
            obj.insert(lang.clone(), Value::String(String::new()));
            CoercionResult::success_same(Value::Object(obj))
        }
        Value::String(s) => {
            // Plain string -> wrap in language key
            let mut obj = serde_json::Map::new();
            obj.insert(lang, Value::String(s.clone()));
            CoercionResult::success_same(Value::Object(obj))
        }
        Value::Object(obj) => {
            // Already a dict - validate all values are strings
            for (k, v) in obj {
                if !v.is_string() && !v.is_null() {
                    // Some values might be StringTranslatedLanguage objects
                    if let Value::Object(inner) = v {
                        if !inner.contains_key("value") {
                            return CoercionResult::error(format!(
                                "String dict value for '{}' must be a string or {{value: string}}, got {:?}",
                                k, v
                            ));
                        }
                    } else {
                        return CoercionResult::error(format!(
                            "String dict value for '{}' must be a string, got {:?}",
                            k, value_type_name(v)
                        ));
                    }
                }
            }
            CoercionResult::success_same(value.clone())
        }
        _ => CoercionResult::error(format!(
            "Expected string or language dict, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Coerce a value to a URL (dict with url and optional url_label).
///
/// Tile data format: `{"url": "https://...", "url_label": "Link text"}`
///
/// If input is a plain string, sets both url and url_label to that value.
/// If input is already an object with url field, passes through.
pub fn coerce_url(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(s) => {
            // Plain string -> set as both url and url_label
            let mut obj = serde_json::Map::new();
            obj.insert("url".to_string(), Value::String(s.clone()));
            obj.insert("url_label".to_string(), Value::String(s.clone()));
            CoercionResult::success_same(Value::Object(obj))
        }
        Value::Object(obj) => {
            // Must have 'url' field
            if !obj.contains_key("url") {
                return CoercionResult::error("URL object must have 'url' field");
            }
            let url = obj.get("url").unwrap();
            if !url.is_string() {
                return CoercionResult::error(format!(
                    "URL 'url' field must be a string, got {:?}",
                    value_type_name(url)
                ));
            }
            // url_label is optional but must be string if present
            if let Some(label) = obj.get("url_label") {
                if !label.is_string() && !label.is_null() {
                    return CoercionResult::error(format!(
                        "URL 'url_label' field must be a string, got {:?}",
                        value_type_name(label)
                    ));
                }
            }
            CoercionResult::success_same(value.clone())
        }
        _ => CoercionResult::error(format!(
            "Expected URL string or object, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Coerce a value to GeoJSON.
///
/// Tile data format: GeoJSON FeatureCollection, Feature, or Geometry object
///
/// Validates basic GeoJSON structure (must be an object with 'type' field).
pub fn coerce_geojson(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::Object(obj) => {
            // Basic GeoJSON validation - must have 'type' field
            if !obj.contains_key("type") {
                return CoercionResult::error("GeoJSON must have 'type' field");
            }
            let geo_type = obj.get("type").unwrap();
            if !geo_type.is_string() {
                return CoercionResult::error(format!(
                    "GeoJSON 'type' must be a string, got {:?}",
                    value_type_name(geo_type)
                ));
            }

            // Validate known GeoJSON types (permissive - just log unknown)
            let type_str = geo_type.as_str().unwrap();
            let valid_types = [
                "Point", "MultiPoint", "LineString", "MultiLineString",
                "Polygon", "MultiPolygon", "GeometryCollection",
                "Feature", "FeatureCollection"
            ];
            if !valid_types.contains(&type_str) {
                // Be permissive - some systems use custom types
            }

            CoercionResult::success_same(value.clone())
        }
        _ => CoercionResult::error(format!(
            "GeoJSON must be an object, got {:?}",
            value_type_name(value)
        )),
    }
}

// =============================================================================
// Phase 3: Config-dependent types (Boolean, DomainValue, DomainValueList)
// These require node config for full resolution
// =============================================================================

/// Coerce a value to boolean.
///
/// Tile data format: `true` or `false`
///
/// Accepts: boolean, 0, 1
/// Config (optional): `{"trueLabel": {"en": "Yes"}, "falseLabel": {"en": "No"}}`
///
/// The display_value includes the config labels for rendering.
pub fn coerce_boolean(value: &Value, config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::Bool(b) => {
            // Build display value with labels if config provided
            let display = if let Some(cfg) = config {
                let mut obj = serde_json::Map::new();
                obj.insert("value".to_string(), Value::Bool(*b));
                if let Some(labels) = cfg.get(if *b { "trueLabel" } else { "falseLabel" }) {
                    obj.insert("labels".to_string(), labels.clone());
                }
                Value::Object(obj)
            } else {
                Value::Bool(*b)
            };
            CoercionResult::success(Value::Bool(*b), display)
        }
        Value::Number(n) => {
            // Accept 0 and 1
            if let Some(i) = n.as_i64() {
                if i == 0 || i == 1 {
                    let b = i == 1;
                    let display = if let Some(cfg) = config {
                        let mut obj = serde_json::Map::new();
                        obj.insert("value".to_string(), Value::Bool(b));
                        if let Some(labels) = cfg.get(if b { "trueLabel" } else { "falseLabel" }) {
                            obj.insert("labels".to_string(), labels.clone());
                        }
                        Value::Object(obj)
                    } else {
                        Value::Bool(b)
                    };
                    return CoercionResult::success(Value::Bool(b), display);
                }
            }
            CoercionResult::error(format!(
                "Boolean must be true/false or 0/1, got {}",
                n
            ))
        }
        _ => CoercionResult::error(format!(
            "Expected boolean, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Coerce a value to a domain value.
///
/// Tile data format: UUID string (domain value ID)
///
/// Input: UUID string or StaticDomainValue object
/// Config (required for UUID lookup): `{"options": [{"id": "...", "text": {...}}, ...]}`
///
/// Returns: tile_data is the UUID, display_value is the resolved StaticDomainValue
pub fn coerce_domain_value(value: &Value, config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(uuid) => {
            // Validate UUID format (basic check)
            if !is_valid_uuid(uuid) {
                return CoercionResult::error(format!(
                    "Domain value must be a UUID, got '{}'",
                    uuid
                ));
            }

            // Look up in config options if provided
            if let Some(cfg) = config {
                if let Some(options) = cfg.get("options").and_then(|o| o.as_array()) {
                    for opt in options {
                        if opt.get("id").and_then(|id| id.as_str()) == Some(uuid.as_str()) {
                            // Found - return UUID as tile_data, full object as display
                            return CoercionResult::success(
                                Value::String(uuid.clone()),
                                opt.clone(),
                            );
                        }
                    }
                    // UUID not found in options
                    return CoercionResult::error(format!(
                        "Domain value '{}' not found in options",
                        uuid
                    ));
                }
            }

            // No config - just return UUID (can't resolve)
            CoercionResult::success_same(Value::String(uuid.clone()))
        }
        Value::Object(obj) => {
            // Already a resolved domain value object - extract id for tile_data
            if let Some(id) = obj.get("id").and_then(|i| i.as_str()) {
                CoercionResult::success(
                    Value::String(id.to_string()),
                    value.clone(),
                )
            } else {
                CoercionResult::error("Domain value object must have 'id' field")
            }
        }
        _ => CoercionResult::error(format!(
            "Expected domain value UUID or object, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Coerce a value to a domain value list.
///
/// Tile data format: Array of UUID strings
///
/// Input: Array of UUIDs or StaticDomainValue objects
/// Config (required for UUID lookup): `{"options": [{"id": "...", "text": {...}}, ...]}`
///
/// Returns: tile_data is array of UUIDs, display_value is array of resolved objects
pub fn coerce_domain_value_list(value: &Value, config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::Array(arr) if arr.is_empty() => {
            CoercionResult::success_same(Value::Array(vec![]))
        }
        Value::Array(arr) => {
            let mut tile_data = Vec::new();
            let mut display_values = Vec::new();
            let mut errors = Vec::new();

            for (i, item) in arr.iter().enumerate() {
                let result = coerce_domain_value(item, config);
                if result.is_error() {
                    errors.push(format!("[{}]: {}", i, result.error.unwrap_or_default()));
                } else if !result.tile_data.is_null() {
                    tile_data.push(result.tile_data);
                    display_values.push(result.display_value);
                }
            }

            if !errors.is_empty() {
                return CoercionResult::error(format!(
                    "Domain value list errors: {}",
                    errors.join(", ")
                ));
            }

            CoercionResult::success(
                Value::Array(tile_data),
                Value::Array(display_values),
            )
        }
        _ => CoercionResult::error(format!(
            "Expected domain value list (array), got {:?}",
            value_type_name(value)
        )),
    }
}

// =============================================================================
// Phase 4: RDM-dependent types (ConceptValue, ConceptList)
// These require RDM collection lookup for full resolution (async in JS)
// =============================================================================

/// Coerce a value to a concept value.
///
/// Tile data format: UUID string (value ID in the RDM collection)
///
/// Input: UUID string, StaticValue object, or StaticConcept object
/// Config: `{"rdmCollection": "uuid-of-collection"}` (used for validation only)
///
/// Note: Full concept resolution requires async RDM lookup, which happens
/// in the ViewModel layer. This coercion just validates and extracts the UUID.
pub fn coerce_concept_value(value: &Value, _config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(uuid) => {
            // Validate UUID format
            if !is_valid_uuid(uuid) {
                return CoercionResult::error(format!(
                    "Concept value must be a UUID, got '{}'",
                    uuid
                ));
            }
            // UUID is the value ID in the collection - tile stores this
            CoercionResult::success_same(Value::String(uuid.clone()))
        }
        Value::Object(obj) => {
            // Could be StaticValue or StaticConcept
            // StaticValue has: id, value, __concept, __conceptId
            // StaticConcept has: id, prefLabels, children, source, sortOrder

            if let Some(id) = obj.get("id").and_then(|i| i.as_str()) {
                // Check if this looks like a StaticConcept (has prefLabels)
                if obj.contains_key("prefLabels") {
                    // It's a StaticConcept - we need to get the prefLabel value ID
                    // For now, return the concept ID and let JS handle resolution
                    // The concept ID isn't what we store in tile data though...
                    // Actually, when you set a concept, you store the value ID, not concept ID
                    return CoercionResult::error(
                        "Cannot coerce StaticConcept directly - use its value ID instead"
                    );
                }

                // It's a StaticValue-like object - extract the ID
                if !is_valid_uuid(id) {
                    return CoercionResult::error(format!(
                        "Concept value ID must be a UUID, got '{}'",
                        id
                    ));
                }

                // tile_data is just the UUID, display_value is the whole object
                CoercionResult::success(
                    Value::String(id.to_string()),
                    value.clone(),
                )
            } else {
                CoercionResult::error("Concept value object must have 'id' field")
            }
        }
        _ => CoercionResult::error(format!(
            "Expected concept value UUID or object, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Coerce a value to a concept value list.
///
/// Tile data format: Array of UUID strings (value IDs in the RDM collection)
///
/// Input: Array of UUIDs or StaticValue objects
/// Config: `{"rdmCollection": "uuid-of-collection"}` (used for validation only)
pub fn coerce_concept_list(value: &Value, config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::Array(arr) if arr.is_empty() => {
            CoercionResult::success_same(Value::Array(vec![]))
        }
        Value::Array(arr) => {
            let mut tile_data = Vec::new();
            let mut display_values = Vec::new();
            let mut errors = Vec::new();

            for (i, item) in arr.iter().enumerate() {
                let result = coerce_concept_value(item, config);
                if result.is_error() {
                    errors.push(format!("[{}]: {}", i, result.error.unwrap_or_default()));
                } else if !result.tile_data.is_null() {
                    tile_data.push(result.tile_data);
                    display_values.push(result.display_value);
                }
            }

            if !errors.is_empty() {
                return CoercionResult::error(format!(
                    "Concept list errors: {}",
                    errors.join(", ")
                ));
            }

            CoercionResult::success(
                Value::Array(tile_data),
                Value::Array(display_values),
            )
        }
        _ => CoercionResult::error(format!(
            "Expected concept list (array), got {:?}",
            value_type_name(value)
        )),
    }
}

// =============================================================================
// Phase 5: Format normalization (ResourceInstance, ResourceInstanceList)
// These normalize various input formats to the standard tile data format
// =============================================================================

/// Coerce a value to a resource instance.
///
/// Tile data format: `[{resourceId: "uuid"}]` (array with single object)
///
/// Input: UUID string, object with `resourceId`, or array with single element
/// Config: Not used
///
/// Returns: tile_data is `[{resourceId: uuid}]`, display_value is the UUID string
pub fn coerce_resource_instance(value: &Value, _config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => {
            CoercionResult::success_same(Value::Null)
        }
        Value::String(uuid) => {
            // Validate UUID format
            if !is_valid_uuid(uuid) {
                return CoercionResult::error(format!(
                    "Resource instance must be a UUID, got '{}'",
                    uuid
                ));
            }
            // Normalize to [{resourceId: uuid}]
            let tile_data = Value::Array(vec![
                serde_json::json!({"resourceId": uuid})
            ]);
            CoercionResult::success(tile_data, Value::String(uuid.clone()))
        }
        Value::Object(obj) => {
            // Object with resourceId field
            if let Some(resource_id) = obj.get("resourceId").and_then(|r| r.as_str()) {
                if !is_valid_uuid(resource_id) {
                    return CoercionResult::error(format!(
                        "Resource instance resourceId must be a UUID, got '{}'",
                        resource_id
                    ));
                }
                // Normalize to [{resourceId: uuid}]
                let tile_data = Value::Array(vec![
                    serde_json::json!({"resourceId": resource_id})
                ]);
                CoercionResult::success(tile_data, Value::String(resource_id.to_string()))
            } else {
                CoercionResult::error("Resource instance object must have 'resourceId' field")
            }
        }
        Value::Array(arr) => {
            // Array - should contain single element
            if arr.is_empty() {
                return CoercionResult::success_same(Value::Null);
            }
            if arr.len() > 1 {
                return CoercionResult::error(
                    "Resource instance array must have at most one element (use resource-instance-list for multiple)"
                );
            }
            // Recurse on the single element
            coerce_resource_instance(&arr[0], _config)
        }
        _ => CoercionResult::error(format!(
            "Expected resource instance UUID, object, or array, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Coerce a value to a resource instance list.
///
/// Tile data format: `[{resourceId: "uuid1"}, {resourceId: "uuid2"}, ...]`
///
/// Input: Array of UUIDs or objects with `resourceId`
/// Config: Not used
///
/// Returns: tile_data is array of `{resourceId: uuid}` objects
pub fn coerce_resource_instance_list(value: &Value, _config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::Array(arr) if arr.is_empty() => {
            CoercionResult::success_same(Value::Array(vec![]))
        }
        Value::Array(arr) => {
            let mut tile_data = Vec::new();
            let mut display_values = Vec::new();
            let mut errors = Vec::new();

            for (i, item) in arr.iter().enumerate() {
                match item {
                    Value::String(uuid) if !uuid.is_empty() => {
                        if !is_valid_uuid(uuid) {
                            errors.push(format!("[{}]: invalid UUID '{}'", i, uuid));
                        } else {
                            tile_data.push(serde_json::json!({"resourceId": uuid}));
                            display_values.push(Value::String(uuid.clone()));
                        }
                    }
                    Value::Object(obj) => {
                        if let Some(resource_id) = obj.get("resourceId").and_then(|r| r.as_str()) {
                            if !is_valid_uuid(resource_id) {
                                errors.push(format!("[{}]: invalid resourceId UUID '{}'", i, resource_id));
                            } else {
                                tile_data.push(serde_json::json!({"resourceId": resource_id}));
                                display_values.push(Value::String(resource_id.to_string()));
                            }
                        } else {
                            errors.push(format!("[{}]: object missing 'resourceId' field", i));
                        }
                    }
                    Value::Null | Value::String(_) => {
                        // Skip null or empty string entries
                    }
                    _ => {
                        errors.push(format!("[{}]: expected UUID or object, got {:?}", i, value_type_name(item)));
                    }
                }
            }

            if !errors.is_empty() {
                return CoercionResult::error(format!(
                    "Resource instance list errors: {}",
                    errors.join(", ")
                ));
            }

            CoercionResult::success(
                Value::Array(tile_data),
                Value::Array(display_values),
            )
        }
        _ => CoercionResult::error(format!(
            "Expected resource instance list (array), got {:?}",
            value_type_name(value)
        )),
    }
}

// =============================================================================
// Dispatcher
// =============================================================================

/// Coerce a value based on datatype.
///
/// This is the main entry point for type coercion.
///
/// Config parameter usage by type:
/// - string: `{"language": "en"}` - language for wrapping plain strings
/// - boolean: `{"trueLabel": {...}, "falseLabel": {...}}` - labels for display
/// - domain-value/domain-value-list: `{"options": [...]}` - domain options for lookup
pub fn coerce_value(datatype: &str, value: &Value, config: Option<&Value>) -> CoercionResult {
    match datatype {
        // Phase 1: Simple scalars
        "number" => coerce_number(value),
        "non-localized-string" => coerce_non_localized_string(value),
        "edtf" => coerce_edtf(value),
        "date" => coerce_date(value),
        // Phase 2: Dict types
        "string" => {
            let language = config
                .and_then(|c| c.get("language"))
                .and_then(|l| l.as_str());
            coerce_string(value, language)
        }
        "url" => coerce_url(value),
        "geojson-feature-collection" => coerce_geojson(value),
        // Phase 3: Config-dependent types
        "boolean" => coerce_boolean(value, config),
        "domain-value" => coerce_domain_value(value, config),
        "domain-value-list" => coerce_domain_value_list(value, config),
        // Phase 4: RDM-dependent types
        "concept" => coerce_concept_value(value, config),
        "concept-list" => coerce_concept_list(value, config),
        // Phase 5: Format normalization
        "resource-instance" => coerce_resource_instance(value, config),
        "resource-instance-list" => coerce_resource_instance_list(value, config),
        _ => {
            // Unknown type - pass through unchanged
            CoercionResult::success_same(value.clone())
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Number tests
    #[test]
    fn test_coerce_number_from_number() {
        let result = coerce_number(&json!(42));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(42));
    }

    #[test]
    fn test_coerce_number_from_float() {
        let result = coerce_number(&json!(3.14));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(3.14));
    }

    #[test]
    fn test_coerce_number_from_string() {
        let result = coerce_number(&json!("42"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(42));
    }

    #[test]
    fn test_coerce_number_from_float_string() {
        let result = coerce_number(&json!("3.14"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(3.14));
    }

    #[test]
    fn test_coerce_number_from_null() {
        let result = coerce_number(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_number_from_empty_string() {
        let result = coerce_number(&json!(""));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_number_invalid_string() {
        let result = coerce_number(&json!("not a number"));
        assert!(result.is_error());
    }

    // NonLocalizedString tests
    #[test]
    fn test_coerce_non_localized_string() {
        let result = coerce_non_localized_string(&json!("hello"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("hello"));
    }

    #[test]
    fn test_coerce_non_localized_string_null() {
        let result = coerce_non_localized_string(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_non_localized_string_empty() {
        let result = coerce_non_localized_string(&json!(""));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_non_localized_string_from_number() {
        let result = coerce_non_localized_string(&json!(42));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("42"));
    }

    // EDTF tests
    #[test]
    fn test_coerce_edtf_year() {
        let result = coerce_edtf(&json!("2023"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023"));
    }

    #[test]
    fn test_coerce_edtf_year_month() {
        let result = coerce_edtf(&json!("2023-05"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05"));
    }

    #[test]
    fn test_coerce_edtf_uncertain() {
        let result = coerce_edtf(&json!("2023?"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023?"));
    }

    #[test]
    fn test_coerce_edtf_null() {
        let result = coerce_edtf(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    // Date tests
    #[test]
    fn test_coerce_date_iso() {
        let result = coerce_date(&json!("2023-05-15"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05-15"));
    }

    #[test]
    fn test_coerce_date_iso_datetime() {
        let result = coerce_date(&json!("2023-05-15T10:30:00Z"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05-15T10:30:00Z"));
    }

    #[test]
    fn test_coerce_date_legacy_object() {
        let result = coerce_date(&json!({"en": "2023-05-15"}));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05-15"));
    }

    #[test]
    fn test_coerce_date_null() {
        let result = coerce_date(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_date_empty_string() {
        let result = coerce_date(&json!(""));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    // Dispatcher tests
    #[test]
    fn test_coerce_value_dispatcher() {
        assert!(!coerce_value("number", &json!(42), None).is_error());
        assert!(!coerce_value("non-localized-string", &json!("test"), None).is_error());
        assert!(!coerce_value("edtf", &json!("2023"), None).is_error());
        assert!(!coerce_value("date", &json!("2023-05-15"), None).is_error());
    }

    #[test]
    fn test_coerce_value_unknown_type() {
        // Unknown types pass through unchanged
        let result = coerce_value("unknown-type", &json!({"foo": "bar"}), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"foo": "bar"}));
    }

    // Phase 2: String tests
    #[test]
    fn test_coerce_string_plain() {
        let result = coerce_string(&json!("Hello"), Some("en"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"en": "Hello"}));
    }

    #[test]
    fn test_coerce_string_with_language() {
        let result = coerce_string(&json!("Hola"), Some("es"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"es": "Hola"}));
    }

    #[test]
    fn test_coerce_string_dict_passthrough() {
        let result = coerce_string(&json!({"en": "Hello", "es": "Hola"}), Some("en"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"en": "Hello", "es": "Hola"}));
    }

    #[test]
    fn test_coerce_string_null() {
        let result = coerce_string(&json!(null), Some("en"));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_string_uses_current_language() {
        set_current_language("fr");
        let result = coerce_string(&json!("Bonjour"), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"fr": "Bonjour"}));
        // Reset
        set_current_language("en");
    }

    // Phase 2: URL tests
    #[test]
    fn test_coerce_url_plain_string() {
        let result = coerce_url(&json!("https://example.com"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({
            "url": "https://example.com",
            "url_label": "https://example.com"
        }));
    }

    #[test]
    fn test_coerce_url_object() {
        let result = coerce_url(&json!({"url": "https://example.com", "url_label": "Example"}));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"url": "https://example.com", "url_label": "Example"}));
    }

    #[test]
    fn test_coerce_url_object_no_label() {
        let result = coerce_url(&json!({"url": "https://example.com"}));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"url": "https://example.com"}));
    }

    #[test]
    fn test_coerce_url_null() {
        let result = coerce_url(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_url_empty_string() {
        let result = coerce_url(&json!(""));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_url_missing_url_field() {
        let result = coerce_url(&json!({"url_label": "Example"}));
        assert!(result.is_error());
    }

    // Phase 2: GeoJSON tests
    #[test]
    fn test_coerce_geojson_point() {
        let result = coerce_geojson(&json!({
            "type": "Point",
            "coordinates": [0.0, 0.0]
        }));
        assert!(!result.is_error());
    }

    #[test]
    fn test_coerce_geojson_feature_collection() {
        let result = coerce_geojson(&json!({
            "type": "FeatureCollection",
            "features": []
        }));
        assert!(!result.is_error());
    }

    #[test]
    fn test_coerce_geojson_null() {
        let result = coerce_geojson(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_geojson_missing_type() {
        let result = coerce_geojson(&json!({"coordinates": [0.0, 0.0]}));
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_geojson_not_object() {
        let result = coerce_geojson(&json!("not an object"));
        assert!(result.is_error());
    }

    // Dispatcher tests for Phase 2
    #[test]
    fn test_coerce_value_string() {
        let result = coerce_value("string", &json!("test"), Some(&json!({"language": "de"})));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!({"de": "test"}));
    }

    #[test]
    fn test_coerce_value_url() {
        let result = coerce_value("url", &json!("https://test.com"), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data["url"], json!("https://test.com"));
    }

    #[test]
    fn test_coerce_value_geojson() {
        let result = coerce_value("geojson-feature-collection", &json!({"type": "Point", "coordinates": [1, 2]}), None);
        assert!(!result.is_error());
    }

    // Phase 3: Boolean tests
    #[test]
    fn test_coerce_boolean_true() {
        let result = coerce_boolean(&json!(true), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(true));
        assert_eq!(result.display_value, json!(true));
    }

    #[test]
    fn test_coerce_boolean_false() {
        let result = coerce_boolean(&json!(false), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(false));
        assert_eq!(result.display_value, json!(false));
    }

    #[test]
    fn test_coerce_boolean_from_1() {
        let result = coerce_boolean(&json!(1), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(true));
    }

    #[test]
    fn test_coerce_boolean_from_0() {
        let result = coerce_boolean(&json!(0), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(false));
    }

    #[test]
    fn test_coerce_boolean_with_config() {
        let config = json!({
            "trueLabel": {"en": "Yes", "es": "Sí"},
            "falseLabel": {"en": "No", "es": "No"}
        });
        let result = coerce_boolean(&json!(true), Some(&config));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(true));
        assert_eq!(result.display_value["value"], json!(true));
        assert_eq!(result.display_value["labels"]["en"], json!("Yes"));
    }

    #[test]
    fn test_coerce_boolean_false_with_config() {
        let config = json!({
            "trueLabel": {"en": "Yes"},
            "falseLabel": {"en": "No"}
        });
        let result = coerce_boolean(&json!(false), Some(&config));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(false));
        assert_eq!(result.display_value["value"], json!(false));
        assert_eq!(result.display_value["labels"]["en"], json!("No"));
    }

    #[test]
    fn test_coerce_boolean_null() {
        let result = coerce_boolean(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_boolean_invalid_number() {
        let result = coerce_boolean(&json!(5), None);
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_boolean_invalid_type() {
        let result = coerce_boolean(&json!("true"), None);
        assert!(result.is_error());
    }

    // Phase 3: DomainValue tests
    #[test]
    fn test_coerce_domain_value_uuid_no_config() {
        let result = coerce_domain_value(&json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_domain_value_with_config() {
        let config = json!({
            "options": [
                {"id": "550e8400-e29b-41d4-a716-446655440000", "text": {"en": "Option A"}},
                {"id": "550e8400-e29b-41d4-a716-446655440001", "text": {"en": "Option B"}}
            ]
        });
        let result = coerce_domain_value(&json!("550e8400-e29b-41d4-a716-446655440000"), Some(&config));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("550e8400-e29b-41d4-a716-446655440000"));
        assert_eq!(result.display_value["id"], json!("550e8400-e29b-41d4-a716-446655440000"));
        assert_eq!(result.display_value["text"]["en"], json!("Option A"));
    }

    #[test]
    fn test_coerce_domain_value_not_found() {
        let config = json!({
            "options": [
                {"id": "550e8400-e29b-41d4-a716-446655440001", "text": {"en": "Option B"}}
            ]
        });
        let result = coerce_domain_value(&json!("550e8400-e29b-41d4-a716-446655440000"), Some(&config));
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_domain_value_object_input() {
        let result = coerce_domain_value(&json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "text": {"en": "Existing"}
        }), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_domain_value_null() {
        let result = coerce_domain_value(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_domain_value_empty_string() {
        let result = coerce_domain_value(&json!(""), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_domain_value_invalid_uuid() {
        let result = coerce_domain_value(&json!("not-a-uuid"), None);
        assert!(result.is_error());
    }

    // Phase 3: DomainValueList tests
    #[test]
    fn test_coerce_domain_value_list_empty() {
        let result = coerce_domain_value_list(&json!([]), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!([]));
    }

    #[test]
    fn test_coerce_domain_value_list_uuids() {
        let result = coerce_domain_value_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001"
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_coerce_domain_value_list_with_config() {
        let config = json!({
            "options": [
                {"id": "550e8400-e29b-41d4-a716-446655440000", "text": {"en": "A"}},
                {"id": "550e8400-e29b-41d4-a716-446655440001", "text": {"en": "B"}}
            ]
        });
        let result = coerce_domain_value_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001"
        ]), Some(&config));
        assert!(!result.is_error());
        let display = result.display_value.as_array().unwrap();
        assert_eq!(display[0]["text"]["en"], json!("A"));
        assert_eq!(display[1]["text"]["en"], json!("B"));
    }

    #[test]
    fn test_coerce_domain_value_list_null() {
        let result = coerce_domain_value_list(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_domain_value_list_invalid_item() {
        let result = coerce_domain_value_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "not-a-uuid"
        ]), None);
        assert!(result.is_error());
    }

    // Dispatcher tests for Phase 3
    #[test]
    fn test_coerce_value_boolean() {
        let result = coerce_value("boolean", &json!(true), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(true));
    }

    #[test]
    fn test_coerce_value_domain_value() {
        let result = coerce_value("domain-value", &json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
    }

    #[test]
    fn test_coerce_value_domain_value_list() {
        let result = coerce_value("domain-value-list", &json!(["550e8400-e29b-41d4-a716-446655440000"]), None);
        assert!(!result.is_error());
    }

    // Phase 4: ConceptValue tests
    #[test]
    fn test_coerce_concept_value_uuid() {
        let result = coerce_concept_value(&json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_concept_value_static_value_object() {
        let result = coerce_concept_value(&json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "value": "Some Label",
            "__conceptId": "660e8400-e29b-41d4-a716-446655440000"
        }), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("550e8400-e29b-41d4-a716-446655440000"));
        // display_value should be the full object
        assert_eq!(result.display_value["value"], json!("Some Label"));
    }

    #[test]
    fn test_coerce_concept_value_null() {
        let result = coerce_concept_value(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_concept_value_empty_string() {
        let result = coerce_concept_value(&json!(""), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_concept_value_invalid_uuid() {
        let result = coerce_concept_value(&json!("not-a-uuid"), None);
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_concept_value_static_concept_rejected() {
        // StaticConcept objects (with prefLabels) should be rejected
        let result = coerce_concept_value(&json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "prefLabels": {"en": {"id": "...", "value": "Label"}}
        }), None);
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_concept_value_object_missing_id() {
        let result = coerce_concept_value(&json!({
            "value": "Some Label"
        }), None);
        assert!(result.is_error());
    }

    // Phase 4: ConceptList tests
    #[test]
    fn test_coerce_concept_list_empty() {
        let result = coerce_concept_list(&json!([]), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!([]));
    }

    #[test]
    fn test_coerce_concept_list_uuids() {
        let result = coerce_concept_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001"
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], json!("550e8400-e29b-41d4-a716-446655440000"));
        assert_eq!(arr[1], json!("550e8400-e29b-41d4-a716-446655440001"));
    }

    #[test]
    fn test_coerce_concept_list_mixed() {
        // Mix of UUIDs and StaticValue objects
        let result = coerce_concept_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            {"id": "550e8400-e29b-41d4-a716-446655440001", "value": "Label"}
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_coerce_concept_list_null() {
        let result = coerce_concept_list(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_concept_list_invalid_item() {
        let result = coerce_concept_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "not-a-uuid"
        ]), None);
        assert!(result.is_error());
    }

    // Dispatcher tests for Phase 4
    #[test]
    fn test_coerce_value_concept() {
        let result = coerce_value("concept", &json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_value_concept_list() {
        let result = coerce_value("concept-list", &json!(["550e8400-e29b-41d4-a716-446655440000"]), None);
        assert!(!result.is_error());
    }

    // Phase 5: ResourceInstance tests
    #[test]
    fn test_coerce_resource_instance_uuid() {
        let result = coerce_resource_instance(&json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
        // tile_data should be [{resourceId: uuid}]
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["resourceId"], json!("550e8400-e29b-41d4-a716-446655440000"));
        // display_value should be the UUID
        assert_eq!(result.display_value, json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_resource_instance_object() {
        let result = coerce_resource_instance(&json!({
            "resourceId": "550e8400-e29b-41d4-a716-446655440000"
        }), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr[0]["resourceId"], json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_resource_instance_array_single() {
        let result = coerce_resource_instance(&json!([
            "550e8400-e29b-41d4-a716-446655440000"
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr[0]["resourceId"], json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_resource_instance_null() {
        let result = coerce_resource_instance(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_resource_instance_empty_string() {
        let result = coerce_resource_instance(&json!(""), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_resource_instance_empty_array() {
        let result = coerce_resource_instance(&json!([]), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_resource_instance_invalid_uuid() {
        let result = coerce_resource_instance(&json!("not-a-uuid"), None);
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_resource_instance_array_multiple() {
        let result = coerce_resource_instance(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001"
        ]), None);
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_resource_instance_object_missing_resourceid() {
        let result = coerce_resource_instance(&json!({"id": "test"}), None);
        assert!(result.is_error());
    }

    // Phase 5: ResourceInstanceList tests
    #[test]
    fn test_coerce_resource_instance_list_empty() {
        let result = coerce_resource_instance_list(&json!([]), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!([]));
    }

    #[test]
    fn test_coerce_resource_instance_list_uuids() {
        let result = coerce_resource_instance_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001"
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["resourceId"], json!("550e8400-e29b-41d4-a716-446655440000"));
        assert_eq!(arr[1]["resourceId"], json!("550e8400-e29b-41d4-a716-446655440001"));
    }

    #[test]
    fn test_coerce_resource_instance_list_objects() {
        let result = coerce_resource_instance_list(&json!([
            {"resourceId": "550e8400-e29b-41d4-a716-446655440000"},
            {"resourceId": "550e8400-e29b-41d4-a716-446655440001"}
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_coerce_resource_instance_list_mixed() {
        let result = coerce_resource_instance_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            {"resourceId": "550e8400-e29b-41d4-a716-446655440001"}
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_coerce_resource_instance_list_null() {
        let result = coerce_resource_instance_list(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_resource_instance_list_skips_nulls() {
        let result = coerce_resource_instance_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            null,
            "550e8400-e29b-41d4-a716-446655440001"
        ]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_coerce_resource_instance_list_invalid_uuid() {
        let result = coerce_resource_instance_list(&json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "not-a-uuid"
        ]), None);
        assert!(result.is_error());
    }

    // Dispatcher tests for Phase 5
    #[test]
    fn test_coerce_value_resource_instance() {
        let result = coerce_value("resource-instance", &json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr[0]["resourceId"], json!("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_coerce_value_resource_instance_list() {
        let result = coerce_value("resource-instance-list", &json!(["550e8400-e29b-41d4-a716-446655440000"]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
    }
}
