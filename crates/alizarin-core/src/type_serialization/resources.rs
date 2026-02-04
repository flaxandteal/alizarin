//! Resource and reference serialization.

use serde_json::Value;
use super::options::{SerializationOptions, SerializationResult};

/// Serialize a resource instance value.
///
/// Tile data format: `[{"resourceId": "uuid", ...}]`
///
/// In TileData mode: returns as-is
/// In Display mode: extracts display name if available
pub fn serialize_resource_instance(
    tile_data: &Value,
    options: &SerializationOptions,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::Array(arr) => {
            if options.is_display() {
                // Extract display names from resource references
                let resolved: Vec<Value> = arr
                    .iter()
                    .map(|v| extract_resource_display(v, &options.language))
                    .collect();

                // If single value, return unwrapped
                if resolved.len() == 1 {
                    // SAFETY: len() == 1 guarantees exactly one element
                    return SerializationResult::success(
                        resolved.into_iter().next().expect("len() == 1 guarantees one element")
                    );
                }
                return SerializationResult::success(Value::Array(resolved));
            }
            SerializationResult::success(tile_data.clone())
        }
        // Single resource object
        Value::Object(_) => {
            if options.is_display() {
                return SerializationResult::success(extract_resource_display(tile_data, &options.language));
            }
            // Wrap in array for consistency
            SerializationResult::success(Value::Array(vec![tile_data.clone()]))
        }
        // UUID string
        Value::String(uuid) => {
            if options.is_display() {
                return SerializationResult::success(Value::String(uuid.clone()));
            }
            // Wrap in standard format
            let mut obj = serde_json::Map::new();
            obj.insert("resourceId".to_string(), Value::String(uuid.clone()));
            SerializationResult::success(Value::Array(vec![Value::Object(obj)]))
        }
        _ => SerializationResult::error(format!(
            "Expected resource instance, got {:?}",
            tile_data
        )),
    }
}

/// Serialize a resource instance list.
pub fn serialize_resource_instance_list(
    tile_data: &Value,
    options: &SerializationOptions,
) -> SerializationResult {
    // Resource instance list uses same format as single resource instance
    serialize_resource_instance(tile_data, options)
}

/// Extract display value from a resource reference object
fn extract_resource_display(value: &Value, language: &str) -> Value {
    if let Value::Object(obj) = value {
        // Try displayName (multilingual)
        if let Some(display_name) = obj.get("displayName") {
            if let Value::Object(names) = display_name {
                if let Some(Value::String(name)) = names.get(language) {
                    return Value::String(name.clone());
                }
                // Fallback to first available
                if let Some((_, Value::String(name))) = names.iter().next() {
                    return Value::String(name.clone());
                }
            } else if let Value::String(name) = display_name {
                return Value::String(name.clone());
            }
        }

        // Try descriptors.name
        if let Some(Value::Object(descriptors)) = obj.get("descriptors") {
            if let Some(name) = descriptors.get("name") {
                if let Value::Object(names) = name {
                    if let Some(Value::String(s)) = names.get(language) {
                        return Value::String(s.clone());
                    }
                    if let Some((_, Value::String(s))) = names.iter().next() {
                        return Value::String(s.clone());
                    }
                } else if let Value::String(s) = name {
                    return Value::String(s.clone());
                }
            }
        }

        // Fall back to resourceId
        if let Some(Value::String(id)) = obj.get("resourceId") {
            return Value::String(id.clone());
        }
    }

    value.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_resource_instance_tile_data() {
        let tile_data = json!([{"resourceId": "uuid-123"}]);
        let options = SerializationOptions::tile_data();

        let result = serialize_resource_instance(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, tile_data);
    }

    #[test]
    fn test_serialize_resource_instance_display() {
        let tile_data = json!([{
            "resourceId": "uuid-123",
            "displayName": {"en": "Test Resource"}
        }]);
        let options = SerializationOptions::display("en");

        let result = serialize_resource_instance(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Test Resource"));
    }

    #[test]
    fn test_serialize_resource_instance_from_uuid() {
        let tile_data = json!("uuid-123");
        let options = SerializationOptions::tile_data();

        let result = serialize_resource_instance(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!([{"resourceId": "uuid-123"}]));
    }
}
