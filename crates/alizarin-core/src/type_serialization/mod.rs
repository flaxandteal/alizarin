//! Type serialization handlers for Alizarin datatypes.
//!
//! This module provides platform-agnostic serialization logic for converting
//! tile_data values to output JSON. It mirrors type_coercion which handles
//! input validation.
//!
//! ## Output Modes
//!
//! - `TileData`: Output suitable for storage (UUIDs, language maps)
//! - `Display`: Human-readable output (resolved labels, extracted strings)
//! - `SearchData`: Search-indexable output (resolved JSON values, falls back to Display)
//!
//! ## Extension Support
//!
//! Custom datatypes register handlers via `ExtensionTypeRegistry`. The
//! `serialize_value` dispatcher checks the registry for extension types.
//!
//! ## Submodules
//!
//! - `options` - SerializationOptions and SerializationResult types
//! - `scalars` - Simple pass-through types (Date, EDTF, Number)
//! - `strings` - Localized strings, URLs, GeoJSON
//! - `domain` - Booleans, domain values (require config lookup)
//! - `rdm` - Concepts (require RDM cache lookup)
//! - `resources` - Resource instances, references

mod domain;
mod options;
mod rdm;
mod resources;
mod scalars;
mod strings;

use serde_json::Value;

use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::node_config::NodeConfig;

// Re-export core types
pub use options::{SerializationMode, SerializationOptions, SerializationResult};

// Re-export serialization functions
pub use domain::{serialize_boolean, serialize_domain_value, serialize_domain_value_list};
pub use rdm::{serialize_concept, serialize_concept_list};
pub use resources::{serialize_resource_instance, serialize_resource_instance_list};
pub use scalars::{serialize_date, serialize_edtf, serialize_number};
pub use strings::{
    serialize_geojson, serialize_non_localized_string, serialize_string, serialize_url,
};

// =============================================================================
// External Resolver Traits
// =============================================================================

/// Trait for resolving external data during serialization.
///
/// Currently used for concept label lookups from RDM cache.
/// Platform bindings implement this — e.g., `impl ExternalResolver for RdmCache`.
pub trait ExternalResolver: Send + Sync {
    /// Resolve a concept UUID to its display label.
    ///
    /// # Arguments
    /// * `collection_id` - The RDM collection containing the concept
    /// * `concept_id` - The concept UUID
    /// * `language` - Language code for the label
    fn resolve_concept(
        &self,
        collection_id: &str,
        concept_id: &str,
        language: &str,
    ) -> Option<String>;
}

/// Trait for resolving resource-instance UUIDs to display names.
///
/// Implemented by `StaticResourceRegistry` in `graph::resources`.
/// Used during display/search serialization of resource-instance and
/// resource-instance-list nodes.
pub trait ResourceDisplayResolver: Send + Sync {
    /// Resolve a resource instance UUID to its display name.
    ///
    /// # Arguments
    /// * `resource_id` - The resource instance UUID
    /// * `language` - Language code for the display name
    fn resolve_resource_display(&self, resource_id: &str, language: &str) -> Option<String>;
}

// =============================================================================
// Serialization Context
// =============================================================================

/// Context for serialization operations that require lookups.
///
/// At the graph/tree level, `node_config` is None — it's set per-node at leaf
/// serialization time by looking up `NodeConfigManager::get(&node_id)`.
/// `external_resolver` and `extension_registry` are shared across all nodes.
pub struct SerializationContext<'a> {
    /// Per-node config (domain options, boolean labels, concept collection).
    /// Set at leaf serialization, not at graph level.
    pub node_config: Option<&'a NodeConfig>,
    /// External data lookup (e.g., RDM cache for concept labels).
    pub external_resolver: Option<&'a dyn ExternalResolver>,
    /// Resource display resolver (e.g., StaticResourceRegistry for resource-instance names).
    pub resource_resolver: Option<&'a dyn ResourceDisplayResolver>,
    /// Unified extension registry for custom datatypes.
    pub extension_registry: Option<&'a ExtensionTypeRegistry>,
}

impl<'a> SerializationContext<'a> {
    /// Create an empty context (no config, no resolvers)
    pub fn empty() -> Self {
        SerializationContext {
            node_config: None,
            external_resolver: None,
            resource_resolver: None,
            extension_registry: None,
        }
    }

    /// Create a child context sharing external_resolver and extension_registry
    /// but with a different node_config.
    pub fn with_node_config(&self, node_config: Option<&'a NodeConfig>) -> Self {
        SerializationContext {
            node_config,
            external_resolver: self.external_resolver,
            resource_resolver: self.resource_resolver,
            extension_registry: self.extension_registry,
        }
    }
}

impl Default for SerializationContext<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

/// Serialize a value based on datatype.
///
/// This is the main entry point for type serialization.
/// It mirrors `coerce_value` from type_coercion.
///
/// # Arguments
///
/// * `datatype` - The Arches datatype (e.g., "string", "concept", "domain-value")
/// * `tile_data` - The tile_data value to serialize
/// * `options` - Serialization options (mode, language)
/// * `ctx` - Optional context for UUID resolution (needed for display/search modes)
///
/// # Returns
///
/// SerializationResult containing the serialized value or error
pub fn serialize_value(
    datatype: &str,
    tile_data: &Value,
    options: &SerializationOptions,
    ctx: Option<&SerializationContext>,
) -> SerializationResult {
    let empty_ctx = SerializationContext::empty();
    let ctx = ctx.unwrap_or(&empty_ctx);

    // Check extension registry for custom datatypes.
    // Try context-provided registry first, then fall back to the global registry.
    let try_extension = |registry: &ExtensionTypeRegistry| -> Option<SerializationResult> {
        let handler = registry.get(datatype)?;
        let caps = handler.capabilities();

        match options.mode {
            SerializationMode::SearchData => {
                if caps.can_render_search {
                    if let Ok(Some(val)) = handler.render_search(tile_data, &options.language) {
                        return Some(SerializationResult::success(val));
                    }
                }
                if caps.can_render_display {
                    if let Ok(Some(label)) = handler.render_display(tile_data, &options.language) {
                        return Some(SerializationResult::success(Value::String(label)));
                    }
                }
            }
            SerializationMode::Display => {
                if caps.can_render_display {
                    if let Ok(Some(label)) = handler.render_display(tile_data, &options.language) {
                        return Some(SerializationResult::success(Value::String(label)));
                    }
                }
            }
            _ => {}
        }
        None
    };

    if let Some(registry) = ctx.extension_registry {
        if let Some(result) = try_extension(registry) {
            return result;
        }
    }

    match datatype {
        // Pass-through types
        "number" => serialize_number(tile_data, options),
        "date" => serialize_date(tile_data, options),
        "edtf" => serialize_edtf(tile_data, options),
        "non-localized-string" => serialize_non_localized_string(tile_data, options),

        // String types
        "string" => serialize_string(tile_data, options),
        "url" => serialize_url(tile_data, options),
        "geojson-feature-collection" => serialize_geojson(tile_data, options),

        // Domain types (need node_config for display/search)
        "boolean" => serialize_boolean(tile_data, options, ctx),
        "domain-value" => serialize_domain_value(tile_data, options, ctx),
        "domain-value-list" => serialize_domain_value_list(tile_data, options, ctx),

        // RDM types (need node_config + external_resolver for display/search)
        "concept" | "concept-value" => serialize_concept(tile_data, options, ctx),
        "concept-list" => serialize_concept_list(tile_data, options, ctx),

        // Resource types (need resource_resolver for display/search)
        "resource-instance" => serialize_resource_instance(tile_data, options, ctx),
        "resource-instance-list" => serialize_resource_instance_list(tile_data, options, ctx),

        // Semantic nodes don't have leaf values
        "semantic" => SerializationResult::success(Value::Null),

        // Unknown types - pass through
        _ => SerializationResult::success(tile_data.clone()),
    }
}

/// Simplified serialization for tile_data mode (no context needed).
///
/// Use this when you just need to serialize without resolving UUIDs.
pub fn serialize_tile_data(datatype: &str, tile_data: &Value) -> SerializationResult {
    serialize_value(
        datatype,
        tile_data,
        &SerializationOptions::tile_data(),
        None,
    )
}

/// Simplified serialization for display mode without resolvers.
///
/// Use this when you want display format but don't have resolvers available.
/// UUIDs will be passed through unchanged.
pub fn serialize_display(datatype: &str, tile_data: &Value, language: &str) -> SerializationResult {
    serialize_value(
        datatype,
        tile_data,
        &SerializationOptions::display(language),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_value_dispatcher_number() {
        let result = serialize_value(
            "number",
            &json!(42),
            &SerializationOptions::tile_data(),
            None,
        );
        assert!(!result.is_error());
        assert_eq!(result.value, json!(42));
    }

    #[test]
    fn test_serialize_value_dispatcher_string_tile_data() {
        let tile_data = json!({"en": "Hello", "es": "Hola"});
        let result = serialize_value(
            "string",
            &tile_data,
            &SerializationOptions::tile_data(),
            None,
        );
        assert!(!result.is_error());
        assert_eq!(result.value, tile_data);
    }

    #[test]
    fn test_serialize_value_dispatcher_string_display() {
        let tile_data = json!({"en": "Hello", "es": "Hola"});
        let result = serialize_value(
            "string",
            &tile_data,
            &SerializationOptions::display("en"),
            None,
        );
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Hello"));
    }

    #[test]
    fn test_serialize_value_unknown_type() {
        let tile_data = json!({"custom": "data"});
        let result = serialize_value(
            "unknown-type",
            &tile_data,
            &SerializationOptions::tile_data(),
            None,
        );
        assert!(!result.is_error());
        assert_eq!(result.value, tile_data);
    }

    #[test]
    fn test_serialize_tile_data_helper() {
        let result = serialize_tile_data("string", &json!({"en": "Test"}));
        assert!(!result.is_error());
        assert_eq!(result.value, json!({"en": "Test"}));
    }

    #[test]
    fn test_serialize_display_helper() {
        let result = serialize_display("string", &json!({"en": "Test", "es": "Prueba"}), "es");
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Prueba"));
    }

    #[test]
    fn test_search_data_resolves_string_like_display() {
        let tile_data = json!({"en": "Hello", "es": "Hola"});
        let result = serialize_value(
            "string",
            &tile_data,
            &SerializationOptions::search_data("en"),
            None,
        );
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Hello"));
    }

    #[test]
    fn test_search_data_resolves_boolean_like_display() {
        let result = serialize_value(
            "boolean",
            &json!(true),
            &SerializationOptions::search_data("en"),
            None,
        );
        assert!(!result.is_error());
        // Without node config, falls back to "true"/"false" string
        assert_eq!(result.value, json!("true"));
    }

    #[test]
    fn test_extension_dispatch_display_mode() {
        use crate::extension_type_registry::ExtensionTypeRegistry;
        use crate::{CoercionResult, ExtensionError, ExtensionTypeHandler, HandlerCapabilities};

        struct TestHandler;
        impl ExtensionTypeHandler for TestHandler {
            fn capabilities(&self) -> HandlerCapabilities {
                HandlerCapabilities {
                    can_coerce: false,
                    can_render_display: true,
                    can_render_search: false,
                    can_resolve_markers: false,
                }
            }
            fn coerce(
                &self,
                _: &Value,
                _: Option<&Value>,
            ) -> Result<CoercionResult, ExtensionError> {
                unimplemented!()
            }
            fn render_display(
                &self,
                _: &Value,
                lang: &str,
            ) -> Result<Option<String>, ExtensionError> {
                Ok(Some(format!("displayed:{}", lang)))
            }
            fn render_search(&self, _: &Value, _: &str) -> Result<Option<Value>, ExtensionError> {
                unimplemented!()
            }
            fn resolve_markers(&self, td: &Value, _: &str) -> Result<Value, ExtensionError> {
                Ok(td.clone())
            }
            fn description(&self) -> &str {
                "test"
            }
        }

        let mut registry = ExtensionTypeRegistry::new();
        registry.register("test-ext", std::sync::Arc::new(TestHandler));

        let ctx = SerializationContext {
            node_config: None,
            external_resolver: None,
            resource_resolver: None,
            extension_registry: Some(&registry),
        };

        let result = serialize_value(
            "test-ext",
            &json!({"some": "data"}),
            &SerializationOptions::display("en"),
            Some(&ctx),
        );
        assert_eq!(result.value, json!("displayed:en"));
    }

    #[test]
    fn test_extension_dispatch_search_data_falls_back_to_display() {
        use crate::extension_type_registry::ExtensionTypeRegistry;
        use crate::{CoercionResult, ExtensionError, ExtensionTypeHandler, HandlerCapabilities};

        struct DisplayOnlyHandler;
        impl ExtensionTypeHandler for DisplayOnlyHandler {
            fn capabilities(&self) -> HandlerCapabilities {
                HandlerCapabilities {
                    can_coerce: false,
                    can_render_display: true,
                    can_render_search: false,
                    can_resolve_markers: false,
                }
            }
            fn coerce(
                &self,
                _: &Value,
                _: Option<&Value>,
            ) -> Result<CoercionResult, ExtensionError> {
                unimplemented!()
            }
            fn render_display(&self, _: &Value, _: &str) -> Result<Option<String>, ExtensionError> {
                Ok(Some("display-fallback".to_string()))
            }
            fn render_search(&self, _: &Value, _: &str) -> Result<Option<Value>, ExtensionError> {
                unimplemented!()
            }
            fn resolve_markers(&self, td: &Value, _: &str) -> Result<Value, ExtensionError> {
                Ok(td.clone())
            }
            fn description(&self) -> &str {
                "test"
            }
        }

        let mut registry = ExtensionTypeRegistry::new();
        registry.register("test-ext", std::sync::Arc::new(DisplayOnlyHandler));

        let ctx = SerializationContext {
            node_config: None,
            external_resolver: None,
            resource_resolver: None,
            extension_registry: Some(&registry),
        };

        // SearchData mode with handler that only has render_display should fall back
        let result = serialize_value(
            "test-ext",
            &json!({"data": 1}),
            &SerializationOptions::search_data("en"),
            Some(&ctx),
        );
        assert_eq!(result.value, json!("display-fallback"));
    }

    #[test]
    fn test_extension_dispatch_search_data_prefers_render_search() {
        use crate::extension_type_registry::ExtensionTypeRegistry;
        use crate::{CoercionResult, ExtensionError, ExtensionTypeHandler, HandlerCapabilities};

        struct SearchHandler;
        impl ExtensionTypeHandler for SearchHandler {
            fn capabilities(&self) -> HandlerCapabilities {
                HandlerCapabilities {
                    can_coerce: false,
                    can_render_display: true,
                    can_render_search: true,
                    can_resolve_markers: false,
                }
            }
            fn coerce(
                &self,
                _: &Value,
                _: Option<&Value>,
            ) -> Result<CoercionResult, ExtensionError> {
                unimplemented!()
            }
            fn render_display(&self, _: &Value, _: &str) -> Result<Option<String>, ExtensionError> {
                Ok(Some("should-not-use".to_string()))
            }
            fn render_search(&self, _: &Value, _: &str) -> Result<Option<Value>, ExtensionError> {
                Ok(Some(json!({"search": "result"})))
            }
            fn resolve_markers(&self, td: &Value, _: &str) -> Result<Value, ExtensionError> {
                Ok(td.clone())
            }
            fn description(&self) -> &str {
                "test"
            }
        }

        let mut registry = ExtensionTypeRegistry::new();
        registry.register("test-ext", std::sync::Arc::new(SearchHandler));

        let ctx = SerializationContext {
            node_config: None,
            external_resolver: None,
            resource_resolver: None,
            extension_registry: Some(&registry),
        };

        let result = serialize_value(
            "test-ext",
            &json!({"data": 1}),
            &SerializationOptions::search_data("en"),
            Some(&ctx),
        );
        // Should use render_search, not render_display
        assert_eq!(result.value, json!({"search": "result"}));
    }

    #[test]
    fn test_extension_dispatch_tile_data_skips_handlers() {
        use crate::extension_type_registry::ExtensionTypeRegistry;
        use crate::{CoercionResult, ExtensionError, ExtensionTypeHandler, HandlerCapabilities};

        struct TestHandler;
        impl ExtensionTypeHandler for TestHandler {
            fn capabilities(&self) -> HandlerCapabilities {
                HandlerCapabilities {
                    can_coerce: false,
                    can_render_display: true,
                    can_render_search: true,
                    can_resolve_markers: false,
                }
            }
            fn coerce(
                &self,
                _: &Value,
                _: Option<&Value>,
            ) -> Result<CoercionResult, ExtensionError> {
                unimplemented!()
            }
            fn render_display(&self, _: &Value, _: &str) -> Result<Option<String>, ExtensionError> {
                Ok(Some("should-not-use".to_string()))
            }
            fn render_search(&self, _: &Value, _: &str) -> Result<Option<Value>, ExtensionError> {
                Ok(Some(json!("should-not-use")))
            }
            fn resolve_markers(&self, td: &Value, _: &str) -> Result<Value, ExtensionError> {
                Ok(td.clone())
            }
            fn description(&self) -> &str {
                "test"
            }
        }

        let mut registry = ExtensionTypeRegistry::new();
        registry.register("test-ext", std::sync::Arc::new(TestHandler));

        let ctx = SerializationContext {
            node_config: None,
            external_resolver: None,
            resource_resolver: None,
            extension_registry: Some(&registry),
        };

        let data = json!({"raw": "data"});
        let result = serialize_value(
            "test-ext",
            &data,
            &SerializationOptions::tile_data(),
            Some(&ctx),
        );
        // TileData mode should NOT call render_display or render_search
        // Falls through to unknown type passthrough
        assert_eq!(result.value, data);
    }
}
