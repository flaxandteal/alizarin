//! Domain serialization: booleans, domain values.
//!
//! In display/search modes, these read from `SerializationContext.node_config`
//! for label lookup.

use super::options::{SerializationOptions, SerializationResult};
use super::SerializationContext;
use serde_json::Value;

/// Serialize a boolean value.
///
/// In TileData mode: returns true/false
/// In Display/Search mode: returns label from node_config, or "true"/"false" string
pub fn serialize_boolean(
    tile_data: &Value,
    options: &SerializationOptions,
    ctx: &SerializationContext,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::Bool(b) => {
            if options.is_display_like() {
                // Try to get label from node config
                if let Some(bool_config) = ctx.node_config.and_then(|nc| nc.as_boolean()) {
                    if let Some(label) = bool_config.get_label(*b, &options.language) {
                        return SerializationResult::success(Value::String(label.to_string()));
                    }
                }
                // Display fallback: return string representation
                return SerializationResult::success(Value::String(
                    if *b { "true" } else { "false" }.to_string(),
                ));
            }
            // TileData: return boolean as-is
            SerializationResult::success(Value::Bool(*b))
        }
        _ => SerializationResult::error(format!("Expected boolean, got {:?}", tile_data)),
    }
}

/// Serialize a domain value (UUID).
///
/// In TileData mode: returns UUID string
/// In Display/Search mode: looks up label from node_config domain options
pub fn serialize_domain_value(
    tile_data: &Value,
    options: &SerializationOptions,
    ctx: &SerializationContext,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::String(uuid) => {
            if options.is_display_like() {
                if let Some(label) = resolve_domain_label(uuid, &options.language, ctx) {
                    return SerializationResult::success(Value::String(label));
                }
            }
            SerializationResult::success(Value::String(uuid.clone()))
        }
        // Handle array format (sometimes domain values are stored as arrays)
        Value::Array(arr) => {
            if arr.len() == 1 {
                if let Some(Value::String(uuid)) = arr.first() {
                    if options.is_display_like() {
                        if let Some(label) = resolve_domain_label(uuid, &options.language, ctx) {
                            return SerializationResult::success(Value::String(label));
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
/// In Display/Search mode: resolves each UUID to label
pub fn serialize_domain_value_list(
    tile_data: &Value,
    options: &SerializationOptions,
    ctx: &SerializationContext,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::Array(arr) => {
            if options.is_display_like() {
                let resolved: Vec<Value> = arr
                    .iter()
                    .map(|v| {
                        if let Value::String(uuid) = v {
                            if let Some(label) = resolve_domain_label(uuid, &options.language, ctx)
                            {
                                return Value::String(label);
                            }
                        }
                        v.clone()
                    })
                    .collect();
                return SerializationResult::success(Value::Array(resolved));
            }
            SerializationResult::success(tile_data.clone())
        }
        // Single value - wrap in array
        Value::String(_) => {
            let result = serialize_domain_value(tile_data, options, ctx);
            if result.is_error() {
                return result;
            }
            SerializationResult::success(Value::Array(vec![result.value]))
        }
        _ => SerializationResult::error(format!("Expected domain value list, got {:?}", tile_data)),
    }
}

/// Resolve a domain value UUID to its display label using node config.
fn resolve_domain_label(uuid: &str, language: &str, ctx: &SerializationContext) -> Option<String> {
    let domain_config = ctx.node_config.and_then(|nc| nc.as_domain())?;
    let domain_value = domain_config.options.iter().find(|dv| dv.id == uuid)?;
    domain_value
        .lang(language)
        .map(|s| s.to_string())
        .or_else(|| Some(domain_value.display().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_config::{NodeConfig, NodeConfigBoolean, NodeConfigDomain, StaticDomainValue};
    use serde_json::json;

    #[test]
    fn test_serialize_boolean_true() {
        let tile_data = json!(true);
        let options = SerializationOptions::tile_data();

        let result = serialize_boolean(&tile_data, &options, &SerializationContext::empty());
        assert!(!result.is_error());
        assert_eq!(result.value, json!(true));
    }

    #[test]
    fn test_serialize_boolean_with_label() {
        let tile_data = json!(true);
        let options = SerializationOptions::display("en");
        let config = NodeConfig::Boolean(NodeConfigBoolean::new(
            [("en".into(), "Yes".into())].into(),
            [("en".into(), "No".into())].into(),
        ));
        let ctx = SerializationContext {
            node_config: Some(&config),
            ..Default::default()
        };

        let result = serialize_boolean(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Yes"));
    }

    #[test]
    fn test_serialize_boolean_display_no_label() {
        let tile_data = json!(false);
        let options = SerializationOptions::display("en");

        let result = serialize_boolean(&tile_data, &options, &SerializationContext::empty());
        assert!(!result.is_error());
        assert_eq!(result.value, json!("false")); // String fallback, not Value::Bool
    }

    #[test]
    fn test_serialize_domain_value_tile_data() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let tile_data = json!(uuid);
        let options = SerializationOptions::tile_data();

        let result = serialize_domain_value(&tile_data, &options, &SerializationContext::empty());
        assert!(!result.is_error());
        assert_eq!(result.value, json!(uuid));
    }

    #[test]
    fn test_serialize_domain_value_display_with_config() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let tile_data = json!(uuid);
        let options = SerializationOptions::display("en");

        let config = NodeConfig::Domain(NodeConfigDomain {
            options: vec![StaticDomainValue::new(
                uuid.to_string(),
                false,
                [("en".into(), "Resolved Label".into())].into(),
            )],
            i18n_config: Default::default(),
        });
        let ctx = SerializationContext {
            node_config: Some(&config),
            ..Default::default()
        };

        let result = serialize_domain_value(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Label"));
    }

    #[test]
    fn test_serialize_domain_value_list() {
        let tile_data = json!(["uuid1", "uuid2"]);
        let options = SerializationOptions::tile_data();

        let result =
            serialize_domain_value_list(&tile_data, &options, &SerializationContext::empty());
        assert!(!result.is_error());
        assert_eq!(result.value, json!(["uuid1", "uuid2"]));
    }
}
