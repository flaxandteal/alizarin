//! Format normalization: ResourceInstance, ResourceInstanceList.

use super::helpers::{is_valid_uuid, value_type_name};
use super::result::CoercionResult;
use serde_json::Value;

/// Coerce a value to a resource instance.
///
/// Tile data format: `[{resourceId: "uuid"}]` (array with single object)
///
/// Input: UUID string, object with `resourceId`, or array with single element
/// Config: Not used
///
/// Returns: tile_data is `[{resourceId: uuid}]`, display_value is the UUID string
#[allow(clippy::only_used_in_recursion)]
pub fn coerce_resource_instance(value: &Value, config: Option<&Value>) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => CoercionResult::success_same(Value::Null),
        Value::String(uuid) => {
            // Validate UUID format
            if !is_valid_uuid(uuid) {
                return CoercionResult::error(format!(
                    "Resource instance must be a UUID, got '{}'",
                    uuid
                ));
            }
            // Normalize to [{resourceId: uuid}]
            let tile_data = Value::Array(vec![serde_json::json!({"resourceId": uuid})]);
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
                let tile_data = Value::Array(vec![serde_json::json!({"resourceId": resource_id})]);
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
            coerce_resource_instance(&arr[0], config)
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
        Value::Array(arr) if arr.is_empty() => CoercionResult::success_same(Value::Array(vec![])),
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
                                errors.push(format!(
                                    "[{}]: invalid resourceId UUID '{}'",
                                    i, resource_id
                                ));
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
                        errors.push(format!(
                            "[{}]: expected UUID or object, got {:?}",
                            i,
                            value_type_name(item)
                        ));
                    }
                }
            }

            if !errors.is_empty() {
                return CoercionResult::error(format!(
                    "Resource instance list errors: {}",
                    errors.join(", ")
                ));
            }

            CoercionResult::success(Value::Array(tile_data), Value::Array(display_values))
        }
        _ => CoercionResult::error(format!(
            "Expected resource instance list (array), got {:?}",
            value_type_name(value)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ResourceInstance tests
    #[test]
    fn test_coerce_resource_instance_uuid() {
        let result = coerce_resource_instance(&json!("550e8400-e29b-41d4-a716-446655440000"), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(
            arr[0]["resourceId"],
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
        assert_eq!(
            result.display_value,
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn test_coerce_resource_instance_object() {
        let result = coerce_resource_instance(
            &json!({
                "resourceId": "550e8400-e29b-41d4-a716-446655440000"
            }),
            None,
        );
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(
            arr[0]["resourceId"],
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn test_coerce_resource_instance_array_single() {
        let result =
            coerce_resource_instance(&json!(["550e8400-e29b-41d4-a716-446655440000"]), None);
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(
            arr[0]["resourceId"],
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
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
    fn test_coerce_resource_instance_invalid_uuid() {
        let result = coerce_resource_instance(&json!("not-a-uuid"), None);
        assert!(result.is_error());
    }

    #[test]
    fn test_coerce_resource_instance_array_multiple() {
        let result = coerce_resource_instance(
            &json!([
                "550e8400-e29b-41d4-a716-446655440000",
                "550e8400-e29b-41d4-a716-446655440001"
            ]),
            None,
        );
        assert!(result.is_error());
    }

    // ResourceInstanceList tests
    #[test]
    fn test_coerce_resource_instance_list_empty() {
        let result = coerce_resource_instance_list(&json!([]), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!([]));
    }

    #[test]
    fn test_coerce_resource_instance_list_uuids() {
        let result = coerce_resource_instance_list(
            &json!([
                "550e8400-e29b-41d4-a716-446655440000",
                "550e8400-e29b-41d4-a716-446655440001"
            ]),
            None,
        );
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(
            arr[0]["resourceId"],
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
        assert_eq!(
            arr[1]["resourceId"],
            json!("550e8400-e29b-41d4-a716-446655440001")
        );
    }

    #[test]
    fn test_coerce_resource_instance_list_null() {
        let result = coerce_resource_instance_list(&json!(null), None);
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_resource_instance_list_skips_nulls() {
        let result = coerce_resource_instance_list(
            &json!([
                "550e8400-e29b-41d4-a716-446655440000",
                null,
                "550e8400-e29b-41d4-a716-446655440001"
            ]),
            None,
        );
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }
}
