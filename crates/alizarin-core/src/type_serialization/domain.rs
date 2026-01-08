//! Domain serialization: booleans, domain values.
//!
//! In display mode, these require node config for label lookup.
//! The resolver callback is passed to handle the actual lookup.

use serde_json::Value;
use super::options::{SerializationOptions, SerializationResult};

/// Callback type for resolving domain value UUIDs to labels
pub type DomainValueResolver = dyn Fn(&str, &str) -> Option<String>;

/// Serialize a boolean value.
///
/// In TileData mode: returns true/false
/// In Display mode with resolver: returns trueLabel/falseLabel if available
pub fn serialize_boolean(
    tile_data: &Value,
    options: &SerializationOptions,
    true_label: Option<&Value>,
    false_label: Option<&Value>,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::Bool(b) => {
            if options.is_display() {
                // Try to get label
                let label_map = if *b { true_label } else { false_label };
                if let Some(label_obj) = label_map {
                    if let Some(label) = extract_label(label_obj, &options.language) {
                        return SerializationResult::success(Value::String(label));
                    }
                }
            }
            // Return boolean as-is
            SerializationResult::success(Value::Bool(*b))
        }
        _ => SerializationResult::error(format!("Expected boolean, got {:?}", tile_data)),
    }
}

/// Serialize a domain value (UUID).
///
/// In TileData mode: returns UUID string
/// In Display mode: calls resolver to get label, returns UUID if not found
pub fn serialize_domain_value(
    tile_data: &Value,
    options: &SerializationOptions,
    resolver: Option<&DomainValueResolver>,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::String(uuid) => {
            if options.is_display() {
                if let Some(resolve) = resolver {
                    if let Some(label) = resolve(uuid, &options.language) {
                        return SerializationResult::success(Value::String(label));
                    }
                }
            }
            // Return UUID as-is
            SerializationResult::success(Value::String(uuid.clone()))
        }
        // Handle array format (sometimes domain values are stored as arrays)
        Value::Array(arr) => {
            if arr.len() == 1 {
                if let Some(Value::String(uuid)) = arr.first() {
                    if options.is_display() {
                        if let Some(resolve) = resolver {
                            if let Some(label) = resolve(uuid, &options.language) {
                                return SerializationResult::success(Value::String(label));
                            }
                        }
                    }
                    return SerializationResult::success(Value::String(uuid.clone()));
                }
            }
            SerializationResult::success(tile_data.clone())
        }
        _ => SerializationResult::error(format!("Expected domain value UUID, got {:?}", tile_data)),
    }
}

/// Serialize a domain value list (array of UUIDs).
///
/// In TileData mode: returns array of UUIDs
/// In Display mode: resolves each UUID to label
pub fn serialize_domain_value_list(
    tile_data: &Value,
    options: &SerializationOptions,
    resolver: Option<&DomainValueResolver>,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::Array(arr) => {
            if options.is_display() {
                if let Some(resolve) = resolver {
                    let resolved: Vec<Value> = arr
                        .iter()
                        .map(|v| {
                            if let Value::String(uuid) = v {
                                if let Some(label) = resolve(uuid, &options.language) {
                                    return Value::String(label);
                                }
                            }
                            v.clone()
                        })
                        .collect();
                    return SerializationResult::success(Value::Array(resolved));
                }
            }
            SerializationResult::success(tile_data.clone())
        }
        // Single value - wrap in array
        Value::String(_) => {
            let result = serialize_domain_value(tile_data, options, resolver);
            if result.is_error() {
                return result;
            }
            SerializationResult::success(Value::Array(vec![result.value]))
        }
        _ => SerializationResult::error(format!(
            "Expected domain value list, got {:?}",
            tile_data
        )),
    }
}

/// Extract label from a translatable label object
fn extract_label(label_obj: &Value, language: &str) -> Option<String> {
    match label_obj {
        Value::String(s) => Some(s.clone()),
        Value::Object(obj) => {
            // Try exact language
            if let Some(Value::String(s)) = obj.get(language) {
                return Some(s.clone());
            }
            // Try base language
            let base_lang = language.split('-').next().unwrap_or(language);
            if base_lang != language {
                if let Some(Value::String(s)) = obj.get(base_lang) {
                    return Some(s.clone());
                }
            }
            // Fall back to first available
            for (_, v) in obj {
                if let Value::String(s) = v {
                    return Some(s.clone());
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_boolean_true() {
        let tile_data = json!(true);
        let options = SerializationOptions::tile_data();

        let result = serialize_boolean(&tile_data, &options, None, None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(true));
    }

    #[test]
    fn test_serialize_boolean_with_label() {
        let tile_data = json!(true);
        let options = SerializationOptions::display("en");
        let true_label = json!({"en": "Yes", "es": "Sí"});

        let result = serialize_boolean(&tile_data, &options, Some(&true_label), None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Yes"));
    }

    #[test]
    fn test_serialize_boolean_display_no_label() {
        let tile_data = json!(false);
        let options = SerializationOptions::display("en");

        let result = serialize_boolean(&tile_data, &options, None, None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(false)); // No label available, returns bool
    }

    #[test]
    fn test_serialize_domain_value_tile_data() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let tile_data = json!(uuid);
        let options = SerializationOptions::tile_data();

        let result = serialize_domain_value(&tile_data, &options, None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(uuid));
    }

    #[test]
    fn test_serialize_domain_value_display_with_resolver() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let tile_data = json!(uuid);
        let options = SerializationOptions::display("en");

        let resolver = |_uuid: &str, _lang: &str| Some("Resolved Label".to_string());

        let result = serialize_domain_value(&tile_data, &options, Some(&resolver));
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Label"));
    }

    #[test]
    fn test_serialize_domain_value_list() {
        let tile_data = json!(["uuid1", "uuid2"]);
        let options = SerializationOptions::tile_data();

        let result = serialize_domain_value_list(&tile_data, &options, None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(["uuid1", "uuid2"]));
    }
}
