//! RDM-dependent type coercion: ConceptValue, ConceptList.

use super::helpers::{is_valid_uuid, value_type_name};
use super::result::CoercionResult;
use serde_json::Value;

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
        Value::String(s) if s.is_empty() => CoercionResult::success_same(Value::Null),
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
                        "Cannot coerce StaticConcept directly - use its value ID instead",
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
                CoercionResult::success(Value::String(id.to_string()), value.clone())
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
        Value::Array(arr) if arr.is_empty() => CoercionResult::success_same(Value::Array(vec![])),
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

            CoercionResult::success(Value::Array(tile_data), Value::Array(display_values))
        }
        _ => CoercionResult::error(format!(
            "Expected concept list (array), got {:?}",
            value_type_name(value)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ConceptValue tests
    #[test]
    fn test_coerce_concept_value_uuid() {
        let result = coerce_concept_value(&json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn test_coerce_concept_value_static_value_object() {
        let result = coerce_concept_value(
            &json!({
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "value": "Some Label",
                "__conceptId": "660e8400-e29b-41d4-a716-446655440000"
            }),
            None,
        );
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
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
        let result = coerce_concept_value(
            &json!({
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "prefLabels": {"en": {"id": "...", "value": "Label"}}
            }),
            None,
        );
        assert!(result.is_error());
    }

    // ConceptList tests
    #[test]
    fn test_coerce_concept_list_empty() {
        let result = coerce_concept_list(&json!([]), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!([]));
    }

    #[test]
    fn test_coerce_concept_list_uuids() {
        let result = coerce_concept_list(
            &json!([
                "550e8400-e29b-41d4-a716-446655440000",
                "550e8400-e29b-41d4-a716-446655440001"
            ]),
            None,
        );
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
}
