//! String serialization: localized strings, URLs, GeoJSON.

use serde_json::Value;
use super::options::{SerializationOptions, SerializationResult};

/// Serialize a localized string value.
///
/// Tile data format: `{"en": "Hello", "es": "Hola", ...}`
/// or `{"en": {"value": "Hello", "direction": "ltr"}, ...}`
///
/// In TileData mode: returns the full language map
/// In Display mode: extracts the string for the requested language
pub fn serialize_string(tile_data: &Value, options: &SerializationOptions) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),

        Value::Object(lang_map) => {
            if options.is_display() {
                // Display mode - extract single language value
                let lang = &options.language;

                // Try exact language match first
                if let Some(val) = lang_map.get(lang) {
                    return SerializationResult::success(extract_string_value(val));
                }

                // Try base language (e.g., "en" for "en-US")
                let base_lang = lang.split('-').next().unwrap_or(lang);
                if base_lang != lang {
                    if let Some(val) = lang_map.get(base_lang) {
                        return SerializationResult::success(extract_string_value(val));
                    }
                }

                // Fall back to first available language
                if let Some((_, val)) = lang_map.iter().next() {
                    return SerializationResult::success(extract_string_value(val));
                }

                SerializationResult::success(Value::String(String::new()))
            } else {
                // TileData mode - return full language map
                if options.include_all_languages {
                    SerializationResult::success(tile_data.clone())
                } else {
                    // Extract just the requested language
                    let lang = &options.language;
                    if let Some(val) = lang_map.get(lang) {
                        let mut result = serde_json::Map::new();
                        result.insert(lang.clone(), val.clone());
                        SerializationResult::success(Value::Object(result))
                    } else {
                        SerializationResult::success(tile_data.clone())
                    }
                }
            }
        }

        // Plain string (shouldn't happen for well-formed tile_data, but handle gracefully)
        Value::String(s) => {
            if options.is_display() {
                SerializationResult::success(Value::String(s.clone()))
            } else {
                // Wrap in language map for consistency
                let mut obj = serde_json::Map::new();
                obj.insert(options.language.clone(), Value::String(s.clone()));
                SerializationResult::success(Value::Object(obj))
            }
        }

        _ => SerializationResult::error(format!(
            "Expected string language map, got {:?}",
            tile_data
        )),
    }
}

/// Extract the string value from a language entry.
/// Handles both plain strings and StringTranslatedLanguage objects.
fn extract_string_value(val: &Value) -> Value {
    match val {
        Value::String(s) => Value::String(s.clone()),
        Value::Object(obj) => {
            // StringTranslatedLanguage format: {"value": "...", "direction": "..."}
            if let Some(v) = obj.get("value") {
                if let Value::String(s) = v {
                    return Value::String(s.clone());
                }
            }
            Value::String(String::new())
        }
        Value::Null => Value::String(String::new()),
        _ => Value::String(String::new()),
    }
}

/// Serialize a URL value.
///
/// Tile data format: `{"url": "https://...", "url_label": "Link text"}`
///
/// In Display mode: returns the url_label (or url if no label)
/// In TileData mode: returns the full object
pub fn serialize_url(tile_data: &Value, options: &SerializationOptions) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),

        Value::Object(obj) => {
            if options.is_display() {
                // Display mode - return label or url
                if let Some(label) = obj.get("url_label") {
                    if let Value::String(s) = label {
                        if !s.is_empty() {
                            return SerializationResult::success(Value::String(s.clone()));
                        }
                    }
                }
                if let Some(url) = obj.get("url") {
                    return SerializationResult::success(url.clone());
                }
                SerializationResult::success(Value::Null)
            } else {
                // TileData mode - return as-is
                SerializationResult::success(tile_data.clone())
            }
        }

        // Plain string URL
        Value::String(s) => {
            if options.is_display() {
                SerializationResult::success(Value::String(s.clone()))
            } else {
                // Wrap in object for consistency
                let mut obj = serde_json::Map::new();
                obj.insert("url".to_string(), Value::String(s.clone()));
                obj.insert("url_label".to_string(), Value::String(s.clone()));
                SerializationResult::success(Value::Object(obj))
            }
        }

        _ => SerializationResult::error(format!(
            "Expected URL object or string, got {:?}",
            tile_data
        )),
    }
}

/// Serialize a GeoJSON value.
///
/// GeoJSON is always returned as-is (no display transformation needed).
pub fn serialize_geojson(tile_data: &Value, _options: &SerializationOptions) -> SerializationResult {
    // GeoJSON is returned as-is regardless of mode
    SerializationResult::success(tile_data.clone())
}

/// Serialize a non-localized string (pass-through).
pub fn serialize_non_localized_string(tile_data: &Value, _options: &SerializationOptions) -> SerializationResult {
    SerializationResult::success(tile_data.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_string_display_mode() {
        let tile_data = json!({"en": "Hello", "es": "Hola"});
        let options = SerializationOptions::display("en");

        let result = serialize_string(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Hello"));
    }

    #[test]
    fn test_serialize_string_display_mode_spanish() {
        let tile_data = json!({"en": "Hello", "es": "Hola"});
        let options = SerializationOptions::display("es");

        let result = serialize_string(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Hola"));
    }

    #[test]
    fn test_serialize_string_display_mode_fallback() {
        let tile_data = json!({"en": "Hello"});
        let options = SerializationOptions::display("de"); // German not available

        let result = serialize_string(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Hello")); // Falls back to first available
    }

    #[test]
    fn test_serialize_string_tile_data_mode() {
        let tile_data = json!({"en": "Hello", "es": "Hola"});
        let options = SerializationOptions::tile_data();

        let result = serialize_string(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!({"en": "Hello", "es": "Hola"}));
    }

    #[test]
    fn test_serialize_string_translated_language_object() {
        let tile_data = json!({"en": {"value": "Hello", "direction": "ltr"}});
        let options = SerializationOptions::display("en");

        let result = serialize_string(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Hello"));
    }

    #[test]
    fn test_serialize_string_null() {
        let tile_data = json!(null);
        let options = SerializationOptions::display("en");

        let result = serialize_string(&tile_data, &options);
        assert!(!result.is_error());
        assert!(result.value.is_null());
    }

    #[test]
    fn test_serialize_url_display_mode() {
        let tile_data = json!({"url": "https://example.com", "url_label": "Example"});
        let options = SerializationOptions::display("en");

        let result = serialize_url(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Example"));
    }

    #[test]
    fn test_serialize_url_display_mode_no_label() {
        let tile_data = json!({"url": "https://example.com"});
        let options = SerializationOptions::display("en");

        let result = serialize_url(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("https://example.com"));
    }

    #[test]
    fn test_serialize_url_tile_data_mode() {
        let tile_data = json!({"url": "https://example.com", "url_label": "Example"});
        let options = SerializationOptions::tile_data();

        let result = serialize_url(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, tile_data);
    }

    #[test]
    fn test_serialize_geojson() {
        let tile_data = json!({"type": "Point", "coordinates": [0.0, 0.0]});
        let options = SerializationOptions::display("en");

        let result = serialize_geojson(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, tile_data);
    }
}
