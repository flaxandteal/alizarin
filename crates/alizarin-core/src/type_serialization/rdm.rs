//! RDM (Reference Data Manager) serialization: concepts.
//!
//! In display/search modes, these read collection_id from `SerializationContext.node_config`
//! and resolve labels via `SerializationContext.external_resolver`.

use super::options::{SerializationOptions, SerializationResult};
use super::SerializationContext;
use serde_json::Value;

/// Resolve a concept UUID to its label using node config + external resolver.
fn resolve_concept_label(uuid: &str, language: &str, ctx: &SerializationContext) -> Option<String> {
    let concept_config = ctx.node_config.and_then(|nc| nc.as_concept())?;
    let resolver = ctx.external_resolver?;
    resolver.resolve_concept(&concept_config.rdm_collection, uuid, language)
}

/// Serialize a concept value (UUID or object).
///
/// In TileData mode: returns UUID string or object as-is
/// In Display/Search mode: resolves to label via RDM cache
pub fn serialize_concept(
    tile_data: &Value,
    options: &SerializationOptions,
    ctx: &SerializationContext,
) -> SerializationResult {
    match tile_data {
        Value::Null => SerializationResult::success(Value::Null),
        Value::String(uuid) => {
            if options.is_display_like() {
                if let Some(label) = resolve_concept_label(uuid, &options.language, ctx) {
                    return SerializationResult::success(Value::String(label));
                }
            }
            SerializationResult::success(Value::String(uuid.clone()))
        }
        // Handle object format: {"id": "uuid", "value": "..."}
        Value::Object(obj) => {
            if let Some(Value::String(uuid)) = obj.get("id") {
                if options.is_display_like() {
                    if let Some(label) = resolve_concept_label(uuid, &options.language, ctx) {
                        return SerializationResult::success(Value::String(label));
                    }
                    // Fall back to value field if resolver fails
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

/// Serialize a concept list (array of UUIDs or objects).
///
/// In TileData mode: returns array as-is
/// In Display/Search mode: resolves each UUID to label
pub fn serialize_concept_list(
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
                    .map(|v| match v {
                        Value::String(uuid) => {
                            if let Some(label) = resolve_concept_label(uuid, &options.language, ctx)
                            {
                                Value::String(label)
                            } else {
                                v.clone()
                            }
                        }
                        Value::Object(obj) => {
                            if let Some(Value::String(uuid)) = obj.get("id") {
                                if let Some(label) =
                                    resolve_concept_label(uuid, &options.language, ctx)
                                {
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
            SerializationResult::success(tile_data.clone())
        }
        // Single value - return as single element
        Value::String(_) | Value::Object(_) => {
            let result = serialize_concept(tile_data, options, ctx);
            if result.is_error() {
                return result;
            }
            SerializationResult::success(Value::Array(vec![result.value]))
        }
        _ => SerializationResult::error(format!("Expected concept list, got {:?}", tile_data)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_config::{NodeConfig, NodeConfigConcept};
    use crate::type_serialization::ExternalResolver;
    use serde_json::json;

    struct MockResolver;
    impl ExternalResolver for MockResolver {
        fn resolve_concept(
            &self,
            _collection_id: &str,
            uuid: &str,
            _language: &str,
        ) -> Option<String> {
            match uuid {
                "uuid-123" => Some("Resolved Concept".to_string()),
                "uuid1" => Some("Label 1".to_string()),
                "uuid2" => Some("Label 2".to_string()),
                _ => None,
            }
        }
    }

    fn concept_ctx<'a>(
        config: &'a NodeConfig,
        resolver: &'a dyn ExternalResolver,
    ) -> SerializationContext<'a> {
        SerializationContext {
            node_config: Some(config),
            external_resolver: Some(resolver),
            extension_registry: None,
        }
    }

    #[test]
    fn test_serialize_concept_tile_data() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let tile_data = json!(uuid);
        let options = SerializationOptions::tile_data();

        let result = serialize_concept(&tile_data, &options, &SerializationContext::empty());
        assert!(!result.is_error());
        assert_eq!(result.value, json!(uuid));
    }

    #[test]
    fn test_serialize_concept_display_with_resolver() {
        let tile_data = json!("uuid-123");
        let options = SerializationOptions::display("en");
        let resolver = MockResolver;
        let config = NodeConfig::Concept(NodeConfigConcept {
            rdm_collection: "coll-1".to_string(),
        });
        let ctx = concept_ctx(&config, &resolver);

        let result = serialize_concept(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Concept"));
    }

    #[test]
    fn test_serialize_concept_object_format() {
        let tile_data = json!({"id": "uuid-123", "value": "Some Value"});
        let options = SerializationOptions::display("en");
        let resolver = MockResolver;
        let config = NodeConfig::Concept(NodeConfigConcept {
            rdm_collection: "coll-1".to_string(),
        });
        let ctx = concept_ctx(&config, &resolver);

        let result = serialize_concept(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Concept"));
    }

    #[test]
    fn test_serialize_concept_object_fallback_to_value() {
        let tile_data = json!({"id": "unknown-uuid", "value": "Fallback Value"});
        let options = SerializationOptions::display("en");
        let resolver = MockResolver;
        let config = NodeConfig::Concept(NodeConfigConcept {
            rdm_collection: "coll-1".to_string(),
        });
        let ctx = concept_ctx(&config, &resolver);

        let result = serialize_concept(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Fallback Value"));
    }

    #[test]
    fn test_serialize_concept_list() {
        let tile_data = json!(["uuid1", "uuid2"]);
        let options = SerializationOptions::tile_data();

        let result = serialize_concept_list(&tile_data, &options, &SerializationContext::empty());
        assert!(!result.is_error());
        assert_eq!(result.value, json!(["uuid1", "uuid2"]));
    }

    #[test]
    fn test_serialize_concept_list_display() {
        let tile_data = json!(["uuid1", "uuid2"]);
        let options = SerializationOptions::display("en");
        let resolver = MockResolver;
        let config = NodeConfig::Concept(NodeConfigConcept {
            rdm_collection: "coll-1".to_string(),
        });
        let ctx = concept_ctx(&config, &resolver);

        let result = serialize_concept_list(&tile_data, &options, &ctx);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(["Label 1", "Label 2"]));
    }
}
