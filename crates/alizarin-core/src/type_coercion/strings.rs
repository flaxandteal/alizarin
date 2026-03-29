//! Dict-based type coercion: String, URL, GeoJSON.

use super::config::get_current_language;
use super::helpers::value_type_name;
use super::result::CoercionResult;
use serde_json::Value;

/// Create a StringTranslatedLanguage entry: `{"value": "...", "direction": "ltr"}`
fn make_string_entry(s: &str) -> Value {
    let mut entry = serde_json::Map::new();
    entry.insert("value".to_string(), Value::String(s.to_string()));
    entry.insert("direction".to_string(), Value::String("ltr".to_string()));
    Value::Object(entry)
}

/// Coerce a value to a localized string (dict of language -> StringTranslatedLanguage).
///
/// Tile data format: `{"en": {"value": "Hello", "direction": "ltr"}, ...}`
///
/// If input is a plain string, wraps it in the current language key.
/// If input is already a dict, normalizes entries to StringTranslatedLanguage format.
pub fn coerce_string(value: &Value, language: Option<&str>) -> CoercionResult {
    let lang: String = language
        .map(|s| s.to_string())
        .unwrap_or_else(get_current_language);

    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) => {
            // Plain string -> wrap in language key with direction
            let mut obj = serde_json::Map::new();
            obj.insert(lang, make_string_entry(s));
            CoercionResult::success_same(Value::Object(obj))
        }
        Value::Number(n) => {
            // Number -> convert to string and wrap in language key
            let mut obj = serde_json::Map::new();
            obj.insert(lang, make_string_entry(&n.to_string()));
            CoercionResult::success_same(Value::Object(obj))
        }
        Value::Object(obj) => {
            // Already a dict - normalize all entries to StringTranslatedLanguage format
            let mut result = serde_json::Map::new();
            for (k, v) in obj {
                match v {
                    // Already in new format: {"value": "...", "direction": "..."}
                    Value::Object(inner) if inner.contains_key("value") => {
                        result.insert(k.clone(), v.clone());
                    }
                    // Old format: plain string value -> normalize
                    Value::String(s) => {
                        result.insert(k.clone(), make_string_entry(s));
                    }
                    Value::Null => {
                        result.insert(k.clone(), make_string_entry(""));
                    }
                    _ => {
                        return CoercionResult::error(format!(
                            "String dict value for '{}' must be a string or {{value: string, direction: string}}, got {:?}",
                            k,
                            value_type_name(v)
                        ));
                    }
                }
            }
            CoercionResult::success_same(Value::Object(result))
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
        Value::String(s) if s.is_empty() => CoercionResult::success_same(Value::Null),
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
                "Point",
                "MultiPoint",
                "LineString",
                "MultiLineString",
                "Polygon",
                "MultiPolygon",
                "GeometryCollection",
                "Feature",
                "FeatureCollection",
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

#[cfg(test)]
mod tests {
    use super::super::config::set_current_language;
    use super::*;
    use serde_json::json;

    // String tests
    #[test]
    fn test_coerce_string_plain() {
        let result = coerce_string(&json!("Hello"), Some("en"));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({"en": {"value": "Hello", "direction": "ltr"}})
        );
    }

    #[test]
    fn test_coerce_string_with_language() {
        let result = coerce_string(&json!("Hola"), Some("es"));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({"es": {"value": "Hola", "direction": "ltr"}})
        );
    }

    #[test]
    fn test_coerce_string_dict_old_format_normalized() {
        let result = coerce_string(&json!({"en": "Hello", "es": "Hola"}), Some("en"));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({
                "en": {"value": "Hello", "direction": "ltr"},
                "es": {"value": "Hola", "direction": "ltr"}
            })
        );
    }

    #[test]
    fn test_coerce_string_dict_new_format_passthrough() {
        let input = json!({
            "en": {"value": "Hello", "direction": "ltr"},
            "ar": {"value": "مرحبا", "direction": "rtl"}
        });
        let result = coerce_string(&input, Some("en"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, input);
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
        assert_eq!(
            result.tile_data,
            json!({"fr": {"value": "Bonjour", "direction": "ltr"}})
        );
        // Reset
        set_current_language("en");
    }

    #[test]
    fn test_coerce_string_from_integer() {
        let result = coerce_string(&json!(650284), Some("en"));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({"en": {"value": "650284", "direction": "ltr"}})
        );
    }

    #[test]
    fn test_coerce_string_from_float() {
        let result = coerce_string(&json!(4.2), Some("en"));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({"en": {"value": "4.2", "direction": "ltr"}})
        );
    }

    #[test]
    fn test_coerce_string_empty() {
        let result = coerce_string(&json!(""), Some("en"));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({"en": {"value": "", "direction": "ltr"}})
        );
    }

    // URL tests
    #[test]
    fn test_coerce_url_plain_string() {
        let result = coerce_url(&json!("https://example.com"));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({
                "url": "https://example.com",
                "url_label": "https://example.com"
            })
        );
    }

    #[test]
    fn test_coerce_url_object() {
        let result = coerce_url(&json!({"url": "https://example.com", "url_label": "Example"}));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({"url": "https://example.com", "url_label": "Example"})
        );
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

    // GeoJSON tests
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
}
