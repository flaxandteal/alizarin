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
//!
//! ## Extension Support
//!
//! Custom datatypes can implement `ExtensionDisplaySerializer` and register
//! with `DisplaySerializerRegistry`. The `serialize_value` dispatcher checks
//! the registry for unknown types before falling back to pass-through.
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
use std::collections::HashMap;
use std::sync::Arc;

// Re-export core types
pub use options::{SerializationMode, SerializationOptions, SerializationResult};

// Re-export serialization functions
pub use domain::{
    serialize_boolean, serialize_domain_value, serialize_domain_value_list, DomainValueResolver,
};
pub use rdm::{serialize_concept, serialize_concept_list, ConceptResolver};
pub use resources::{serialize_resource_instance, serialize_resource_instance_list};
pub use scalars::{serialize_date, serialize_edtf, serialize_number};
pub use strings::{
    serialize_geojson, serialize_non_localized_string, serialize_string, serialize_url,
};

// =============================================================================
// Extension Display Serializer API
// =============================================================================

/// Trait for extension display serializers.
///
/// Extensions can implement this trait to provide custom display serialization
/// for their datatypes. This is used by `toDisplayJson()` to convert resolved
/// values to human-readable strings.
///
/// # Example
///
/// ```ignore
/// struct ReferenceDisplaySerializer;
///
/// impl ExtensionDisplaySerializer for ReferenceDisplaySerializer {
///     fn serialize_display(
///         &self,
///         tile_data: &Value,
///         options: &SerializationOptions,
///     ) -> SerializationResult {
///         // Extract label from StaticReference format
///         if let Some(labels) = tile_data.get("labels").and_then(|v| v.as_array()) {
///             for label in labels {
///                 if label.get("language_id").and_then(|v| v.as_str()) == Some(&options.language) {
///                     if let Some(value) = label.get("value").and_then(|v| v.as_str()) {
///                         return SerializationResult::success(json!(value));
///                     }
///                 }
///             }
///         }
///         SerializationResult::success(tile_data.clone())
///     }
/// }
/// ```
pub trait ExtensionDisplaySerializer: Send + Sync {
    /// Serialize a tile_data value to display format.
    ///
    /// # Arguments
    /// * `tile_data` - The resolved tile_data value to serialize
    /// * `options` - Serialization options (mode, language)
    ///
    /// # Returns
    /// SerializationResult with the display value or error
    fn serialize_display(
        &self,
        tile_data: &Value,
        options: &SerializationOptions,
    ) -> SerializationResult;

    /// Get a description of this serializer for documentation
    fn description(&self) -> &str {
        "Extension display serializer"
    }
}

/// Registry for extension display serializers.
///
/// Extensions register their display serializers by datatype name.
/// The `serialize_value` dispatcher checks this registry for custom
/// datatypes before falling back to pass-through.
///
/// # Example
///
/// ```ignore
/// let mut registry = DisplaySerializerRegistry::new();
/// registry.register("reference", Arc::new(ReferenceDisplaySerializer));
///
/// let ctx = SerializationContext {
///     display_registry: Some(&registry),
///     ..Default::default()
/// };
///
/// let result = serialize_value("reference", &tile_data, &options, Some(&ctx));
/// ```
pub struct DisplaySerializerRegistry {
    serializers: HashMap<String, Arc<dyn ExtensionDisplaySerializer>>,
}

impl DisplaySerializerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            serializers: HashMap::new(),
        }
    }

    /// Register a display serializer for a datatype.
    ///
    /// # Arguments
    /// * `datatype` - The datatype name (e.g., "reference", "my-custom-type")
    /// * `serializer` - The serializer implementation
    pub fn register(
        &mut self,
        datatype: impl Into<String>,
        serializer: Arc<dyn ExtensionDisplaySerializer>,
    ) {
        self.serializers.insert(datatype.into(), serializer);
    }

    /// Get the serializer for a datatype, if registered.
    pub fn get(&self, datatype: &str) -> Option<&Arc<dyn ExtensionDisplaySerializer>> {
        self.serializers.get(datatype)
    }

    /// Check if a serializer is registered for a datatype.
    pub fn has(&self, datatype: &str) -> bool {
        self.serializers.contains_key(datatype)
    }

    /// Get all registered datatype names.
    pub fn datatypes(&self) -> impl Iterator<Item = &String> {
        self.serializers.keys()
    }
}

impl Default for DisplaySerializerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for serialization operations that require lookups.
///
/// This allows the serialization dispatcher to resolve UUIDs to labels
/// without directly depending on RdmCache or NodeConfigManager.
pub struct SerializationContext<'a> {
    /// Resolver for domain value UUIDs -> labels
    pub domain_resolver: Option<&'a DomainValueResolver>,
    /// Resolver for concept UUIDs -> labels
    pub concept_resolver: Option<&'a ConceptResolver>,
    /// True/false labels for boolean nodes (from node config)
    pub boolean_true_label: Option<&'a Value>,
    pub boolean_false_label: Option<&'a Value>,
    /// Registry of extension display serializers for custom datatypes
    pub display_registry: Option<&'a DisplaySerializerRegistry>,
}

impl<'a> SerializationContext<'a> {
    /// Create an empty context (no resolvers)
    pub fn empty() -> Self {
        SerializationContext {
            domain_resolver: None,
            concept_resolver: None,
            boolean_true_label: None,
            boolean_false_label: None,
            display_registry: None,
        }
    }

    /// Create a context with just a display registry
    pub fn with_registry(registry: &'a DisplaySerializerRegistry) -> Self {
        SerializationContext {
            domain_resolver: None,
            concept_resolver: None,
            boolean_true_label: None,
            boolean_false_label: None,
            display_registry: Some(registry),
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
/// * `ctx` - Optional context for UUID resolution (needed for display mode)
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

    // Check extension registry first (allows overriding built-in types)
    if options.is_display() {
        if let Some(registry) = ctx.display_registry {
            if let Some(serializer) = registry.get(datatype) {
                return serializer.serialize_display(tile_data, options);
            }
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

        // Domain types (need config for display)
        "boolean" => serialize_boolean(
            tile_data,
            options,
            ctx.boolean_true_label,
            ctx.boolean_false_label,
        ),
        "domain-value" => serialize_domain_value(tile_data, options, ctx.domain_resolver),
        "domain-value-list" => serialize_domain_value_list(tile_data, options, ctx.domain_resolver),

        // RDM types (need cache for display)
        "concept" | "concept-value" => serialize_concept(tile_data, options, ctx.concept_resolver),
        "concept-list" => serialize_concept_list(tile_data, options, ctx.concept_resolver),

        // Resource types
        "resource-instance" => serialize_resource_instance(tile_data, options),
        "resource-instance-list" => serialize_resource_instance_list(tile_data, options),

        // Semantic nodes don't have leaf values
        "semantic" => SerializationResult::success(Value::Null),

        // Unknown types - check extension registry (non-display mode), else pass through
        _ => {
            if let Some(registry) = ctx.display_registry {
                if let Some(serializer) = registry.get(datatype) {
                    return serializer.serialize_display(tile_data, options);
                }
            }
            SerializationResult::success(tile_data.clone())
        }
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
    fn test_serialize_value_dispatcher_concept_with_resolver() {
        let tile_data = json!("uuid-123");
        let resolver = |uuid: &str, _lang: &str| {
            if uuid == "uuid-123" {
                Some("Resolved Concept".to_string())
            } else {
                None
            }
        };
        let ctx = SerializationContext {
            concept_resolver: Some(&resolver),
            ..Default::default()
        };

        let result = serialize_value(
            "concept",
            &tile_data,
            &SerializationOptions::display("en"),
            Some(&ctx),
        );
        assert!(!result.is_error());
        assert_eq!(result.value, json!("Resolved Concept"));
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
}
