//! RDM (Reference Data Manager) serialization: concepts.
//!
//! In display mode, these require RDM cache for label lookup.

use serde_json::Value;
use super::options::{SerializationOptions, SerializationResult};

/// Callback type for resolving concept UUIDs to labels
pub type ConceptResolver = dyn Fn(&str, &str) -> Option<String>;

/// Serialize a concept value (UUID).
///
/// In TileData mode: returns UUID string
/// In Display mode: calls resolver to get label, returns UUID if not found
pub fn serialize_concept(
    tile_data: &Value,
    options: &SerializationOptions,
    resolver: Option<&ConceptResolver>,
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
            SerializationResult::success(Value::String(uuid.clone()))
        }
        // Handle object format (concept may be stored as StaticValue object)
        Value::Object(obj) => {
            // Try to extract id for lookup
            if let Some(Value::String(uuid)) = obj.get("id") {
                if options.is_display() {
                    if let Some(resolve) = resolver {
                        if let Some(label) = resolve(uuid, &options.language) {
                            return SerializationResult::success(Value::String(label));
                        }
                    }
                    // If no resolver or not found, try to use value field
                    if let Some(value) = obj.get("value") {
                        return SerializationResult::success(value.clone());
                    }
                }
                return SerializationResult::success(Value::String(uuid.clone()));
            }
            // Return as-is if we can't extract id
            SerializationResult::success(tile_data.clone())
        }
        _ => SerializationResult::error(format!(
            "Expected concept UUID or object, got {:?}",
            tile_data
        )),
    }
}

/// Serialize a concept list (array of UUIDs).
///
/// In TileData mode: returns array of UUIDs
/// In Display mode: resolves each UUID to label
pub fn serialize_concept_list(
    tile_data: &Value,
    options: &SerializationOptions,
    resolver: Option<&ConceptResolver>,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::Array(arr) => {
            if options.is_display() {
                if let Some(resolve) = resolver {
                    let resolved: Vec<Value> = arr
                        .iter()
                        .map(|v| match v {
                            Value::String(uuid) => {
                                if let Some(label) = resolve(uuid, &options.language) {
                                    Value::String(label)
                                } else {
                                    v.clone()
                                }
                            }
                            Value::Object(obj) => {
                                if let Some(Value::String(uuid)) = obj.get("id") {
                                    if let Some(label) = resolve(uuid, &options.language) {
                                        return Value::String(label);
                                    }
                                }
                                if let Some(value) = obj.get("value") {
                                    return value.clone();
                                }
                                v.clone()
                            }
                            _ => v.clone(),
                        })
                        .collect();
                    return SerializationResult::success(Value::Array(resolved));
                }
            }
            SerializationResult::success(tile_data.clone())
        }
        // Single value - return as single element
        Value::String(_) | Value::Object(_) => {
            let result = serialize_concept(tile_data, options, resolver);
            if result.is_error() {
                return result;
            }
            // For consistency, concept-list should return array
            SerializationResult::success(Value::Array(vec![result.value]))
        }
        _ => SerializationResult::error(format!(
            "Expected concept list, got {:?}",
            tile_data
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_concept_tile_data() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let tile_data = json!(uuid);
        let options = SerializationOptions::tile_data();

        let result = serialize_concept(&tile_data, &options, None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(uuid));
    }

    #[test]
    fn test_serialize_concept_display_with_resolver() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let tile_data = json!(uuid);
        let options = SerializationOptions::display("en");

        let resolver = |_uuid: &str, _lang: &str| Some("Concept Label".to_string());

        let result = serialize_concept(&tile_data, &options, Some(&resolver));
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Concept Label"));
    }

    #[test]
    fn test_serialize_concept_object_format() {
        let tile_data = json!({"id": "uuid-123", "value": "Some Value"});
        let options = SerializationOptions::display("en");

        let result = serialize_concept(&tile_data, &options, None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Some Value"));
    }

    #[test]
    fn test_serialize_concept_list() {
        let tile_data = json!(["uuid1", "uuid2"]);
        let options = SerializationOptions::tile_data();

        let result = serialize_concept_list(&tile_data, &options, None);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(["uuid1", "uuid2"]));
    }

    #[test]
    fn test_serialize_concept_list_display() {
        let tile_data = json!(["uuid1", "uuid2"]);
        let options = SerializationOptions::display("en");

        let resolver = |uuid: &str, _lang: &str| {
            match uuid {
                "uuid1" => Some("Label 1".to_string()),
                "uuid2" => Some("Label 2".to_string()),
                _ => None,
            }
        };

        let result = serialize_concept_list(&tile_data, &options, Some(&resolver));
        assert!(!result.is_error());
        assert_eq!(result.value, json!(["Label 1", "Label 2"]));
    }
}
