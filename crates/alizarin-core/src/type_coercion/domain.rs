//! Config-dependent type coercion: Boolean, DomainValue, DomainValueList.

use serde_json::Value;
use super::result::CoercionResult;
use super::helpers::{value_type_name, is_valid_uuid};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Boolean tests
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

    // DomainValue tests
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
        assert_eq!(result.display_value["text"]["en"], json!("Option A"));
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
    fn test_coerce_domain_value_invalid_uuid() {
        let result = coerce_domain_value(&json!("not-a-uuid"), None);
        assert!(result.is_error());
    }

    // DomainValueList tests
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
    fn test_coerce_domain_value_list_null() {
        let result = coerce_domain_value_list(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }
}
