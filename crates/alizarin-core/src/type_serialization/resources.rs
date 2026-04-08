//! Resource and reference serialization.

use super::options::{SerializationOptions, SerializationResult};
use super::SerializationContext;
use serde_json::Value;

/// Serialize a resource instance value.
///
/// Tile data format: `[{"resourceId": "uuid", ...}]`
///
/// In TileData mode: returns as-is
/// In Display mode: tries resource_resolver first, then tile_data fields, then UUID fallback
pub fn serialize_resource_instance(
    tile_data: &Value,
    options: &SerializationOptions,
    ctx: &SerializationContext,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::Array(arr) => {
            if options.is_display_like() {
                // Extract display names from resource references
                let resolved: Vec<Value> = arr
                    .iter()
                    .map(|v| resolve_or_extract_display(v, &options.language, ctx))
                    .collect();

                // If single value, return unwrapped
                if resolved.len() == 1 {
                    // SAFETY: len() == 1 guarantees exactly one element
                    return SerializationResult::success(
                        resolved
                            .into_iter()
                            .next()
                            .expect("len() == 1 guarantees one element"),
                    );
                }
                return SerializationResult::success(Value::Array(resolved));
            }
            SerializationResult::success(tile_data.clone())
        }
        // Single resource object
        Value::Object(_) => {
            if options.is_display_like() {
                return SerializationResult::success(resolve_or_extract_display(
                    tile_data,
                    &options.language,
                    ctx,
                ));
            }
            // Wrap in array for consistency
            SerializationResult::success(Value::Array(vec![tile_data.clone()]))
        }
        // UUID string
        Value::String(uuid) => {
            if options.is_display_like() {
                // Try resolver for bare UUID
                if let Some(display) = resolve_resource_id(uuid, &options.language, ctx) {
                    return SerializationResult::success(Value::String(display));
                }
                return SerializationResult::success(Value::String(uuid.clone()));
            }
            // Wrap in standard format
            let mut obj = serde_json::Map::new();
            obj.insert("resourceId".to_string(), Value::String(uuid.clone()));
            SerializationResult::success(Value::Array(vec![Value::Object(obj)]))
        }
        _ => SerializationResult::error(format!("Expected resource instance, got {:?}", tile_data)),
    }
}

/// Serialize a resource instance list.
pub fn serialize_resource_instance_list(
    tile_data: &Value,
    options: &SerializationOptions,
    ctx: &SerializationContext,
) -> SerializationResult {
    // Resource instance list uses same format as single resource instance
    serialize_resource_instance(tile_data, options, ctx)
}

/// Try the resource_resolver first, then fall back to tile_data extraction.
fn resolve_or_extract_display(value: &Value, language: &str, ctx: &SerializationContext) -> Value {
    // Try resolver if we can extract a resourceId
    if let Some(uuid) = extract_resource_id(value) {
        if let Some(display) = resolve_resource_id(&uuid, language, ctx) {
            return Value::String(display);
        }
    }

    // Fall back to tile_data field extraction (displayName, descriptors.name, resourceId)
    extract_resource_display(value, language)
}

/// Extract resourceId from a resource reference value.
fn extract_resource_id(value: &Value) -> Option<String> {
    if let Value::Object(obj) = value {
        if let Some(Value::String(id)) = obj.get("resourceId") {
            if !id.is_empty() {
                return Some(id.clone());
            }
        }
    }
    None
}

/// Look up a resource UUID via the resource_resolver on the context.
fn resolve_resource_id(uuid: &str, language: &str, ctx: &SerializationContext) -> Option<String> {
    ctx.resource_resolver?
        .resolve_resource_display(uuid, language)
}

/// Extract display value from a resource reference object's tile_data fields.
///
/// Tries: displayName → descriptors.name → resourceId
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
    use crate::type_serialization::ResourceDisplayResolver;
    use serde_json::json;

    struct MockResolver;
    impl ResourceDisplayResolver for MockResolver {
        fn resolve_resource_display(&self, resource_id: &str, _language: &str) -> Option<String> {
            match resource_id {
                "uuid-123" => Some("Resolved Name".to_string()),
                _ => None,
            }
        }
    }

    fn empty_ctx() -> SerializationContext<'static> {
        SerializationContext::empty()
    }

    #[test]
    fn test_serialize_resource_instance_tile_data() {
        let tile_data = json!([{"resourceId": "uuid-123"}]);
        let options = SerializationOptions::tile_data();

        let result = serialize_resource_instance(&tile_data, &options, &empty_ctx());
        assert!(!result.is_error());
        assert_eq!(result.value, tile_data);
    }

    #[test]
    fn test_serialize_resource_instance_display_from_tile_data() {
        let tile_data = json!([{
            "resourceId": "uuid-123",
            "displayName": {"en": "Test Resource"}
        }]);
        let options = SerializationOptions::display("en");

        let result = serialize_resource_instance(&tile_data, &options, &empty_ctx());
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Test Resource"));
    }

    #[test]
    fn test_serialize_resource_instance_from_uuid() {
        let tile_data = json!("uuid-123");
        let options = SerializationOptions::tile_data();

        let result = serialize_resource_instance(&tile_data, &options, &empty_ctx());
        assert!(!result.is_error());
        assert_eq!(result.value, json!([{"resourceId": "uuid-123"}]));
    }

    #[test]
    fn test_resolver_used_in_display_mode() {
        let resolver = MockResolver;
        let ctx = SerializationContext {
            resource_resolver: Some(&resolver),
            ..Default::default()
        };
        let tile_data = json!([{"resourceId": "uuid-123"}]);
        let options = SerializationOptions::display("en");

        let result = serialize_resource_instance(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Name"));
    }

    #[test]
    fn test_resolver_miss_falls_back_to_tile_data() {
        let resolver = MockResolver;
        let ctx = SerializationContext {
            resource_resolver: Some(&resolver),
            ..Default::default()
        };
        // UUID not in mock resolver, but tile_data has displayName
        let tile_data = json!([{
            "resourceId": "unknown-uuid",
            "displayName": {"en": "Fallback Name"}
        }]);
        let options = SerializationOptions::display("en");

        let result = serialize_resource_instance(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Fallback Name"));
    }

    #[test]
    fn test_resolver_miss_falls_back_to_uuid() {
        let resolver = MockResolver;
        let ctx = SerializationContext {
            resource_resolver: Some(&resolver),
            ..Default::default()
        };
        // UUID not in mock resolver, no displayName — falls back to resourceId
        let tile_data = json!([{"resourceId": "unknown-uuid"}]);
        let options = SerializationOptions::display("en");

        let result = serialize_resource_instance(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("unknown-uuid"));
    }

    #[test]
    fn test_resolver_on_bare_uuid_string() {
        let resolver = MockResolver;
        let ctx = SerializationContext {
            resource_resolver: Some(&resolver),
            ..Default::default()
        };
        let tile_data = json!("uuid-123");
        let options = SerializationOptions::display("en");

        let result = serialize_resource_instance(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Name"));
    }

    #[test]
    fn test_resolver_not_called_in_tile_data_mode() {
        let resolver = MockResolver;
        let ctx = SerializationContext {
            resource_resolver: Some(&resolver),
            ..Default::default()
        };
        let tile_data = json!([{"resourceId": "uuid-123"}]);
        let options = SerializationOptions::tile_data();

        let result = serialize_resource_instance(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        // Should return raw tile_data, not resolved name
        assert_eq!(result.value, tile_data);
    }

    #[test]
    fn test_null_input() {
        let result = serialize_resource_instance(
            &Value::Null,
            &SerializationOptions::display("en"),
            &empty_ctx(),
        );
        assert!(!result.is_error());
        assert_eq!(result.value, Value::Null);
    }

    #[test]
    fn test_resource_instance_list_delegates() {
        let resolver = MockResolver;
        let ctx = SerializationContext {
            resource_resolver: Some(&resolver),
            ..Default::default()
        };
        let tile_data = json!([{"resourceId": "uuid-123"}]);
        let options = SerializationOptions::display("en");

        let result = serialize_resource_instance_list(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Name"));
    }
}
