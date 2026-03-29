//! Type coercion handlers for Alizarin datatypes.
//!
//! This module provides platform-agnostic type coercion/validation logic
//! that can be used from TypeScript (via WASM), Python (via PyO3), or native Rust.
//!
//! ## Submodules
//!
//! - `config` - Language configuration
//! - `result` - CoercionResult type
//! - `scalars` - Phase 1: Simple scalars (Date, EDTF, NonLocalizedString, Number)
//! - `strings` - Phase 2: Dict types (String, Url, GeoJSON)
//! - `domain` - Phase 3: Config-dependent types (Boolean, DomainValue, DomainValueList)
//! - `rdm` - Phase 4: RDM-dependent types (ConceptValue, ConceptList)
//! - `resources` - Phase 5: Format normalization (ResourceInstance, ResourceInstanceList)
//! - `helpers` - Helper functions

mod config;
mod domain;
mod helpers;
mod rdm;
mod resources;
mod result;
mod scalars;
mod strings;

use serde_json::Value;

// Re-export core types
pub use config::{get_current_language, set_current_language, DEFAULT_LANGUAGE};
pub use helpers::value_type_name;
pub use result::CoercionResult;

// Re-export coercion functions
pub use domain::{coerce_boolean, coerce_domain_value, coerce_domain_value_list};
pub use rdm::{coerce_concept_list, coerce_concept_value};
pub use resources::{coerce_resource_instance, coerce_resource_instance_list};
pub use scalars::{coerce_date, coerce_edtf, coerce_non_localized_string, coerce_number};
pub use strings::{coerce_geojson, coerce_string, coerce_url};

/// Coerce a value based on datatype.
///
/// This is the main entry point for type coercion.
///
/// Config parameter usage by type:
/// - string: `{"language": "en"}` - language for wrapping plain strings
/// - boolean: `{"trueLabel": {...}, "falseLabel": {...}}` - labels for display
/// - domain-value/domain-value-list: `{"options": [...]}` - domain options for lookup
pub fn coerce_value(datatype: &str, value: &Value, config: Option<&Value>) -> CoercionResult {
    match datatype {
        // Phase 1: Simple scalars
        "number" => coerce_number(value),
        "non-localized-string" => coerce_non_localized_string(value),
        "edtf" => coerce_edtf(value),
        "date" => coerce_date(value),
        // Phase 2: Dict types
        "string" => {
            let language = config
                .and_then(|c| c.get("language"))
                .and_then(|l| l.as_str());
            coerce_string(value, language)
        }
        "url" => coerce_url(value),
        "geojson-feature-collection" => coerce_geojson(value),
        // Phase 3: Config-dependent types
        "boolean" => coerce_boolean(value, config),
        "domain-value" => coerce_domain_value(value, config),
        "domain-value-list" => coerce_domain_value_list(value, config),
        // Phase 4: RDM-dependent types
        "concept" => coerce_concept_value(value, config),
        "concept-list" => coerce_concept_list(value, config),
        // Phase 5: Format normalization
        "resource-instance" => coerce_resource_instance(value, config),
        "resource-instance-list" => coerce_resource_instance_list(value, config),
        _ => {
            // Unknown type - pass through unchanged but flag as passthrough
            CoercionResult::success_passthrough(value.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Dispatcher tests
    #[test]
    fn test_coerce_value_dispatcher() {
        assert!(!coerce_value("number", &json!(42), None).is_error());
        assert!(!coerce_value("non-localized-string", &json!("test"), None).is_error());
        assert!(!coerce_value("edtf", &json!("2023"), None).is_error());
        assert!(!coerce_value("date", &json!("2023-05-15"), None).is_error());
    }

    #[test]
    fn test_coerce_value_unknown_type() {
        // Unknown types pass through unchanged but flagged as passthrough
        let result = coerce_value("unknown-type", &json!({"foo": "bar"}), None);
        assert!(!result.is_error());
        assert!(result.passthrough);
        assert_eq!(result.tile_data, json!({"foo": "bar"}));
    }

    #[test]
    fn test_coerce_value_known_type_not_passthrough() {
        let result = coerce_value("string", &json!("test"), None);
        assert!(!result.passthrough);
    }

    #[test]
    fn test_coerce_value_string() {
        let result = coerce_value("string", &json!("test"), Some(&json!({"language": "de"})));
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!({"de": {"value": "test", "direction": "ltr"}})
        );
    }

    #[test]
    fn test_coerce_value_url() {
        let result = coerce_value("url", &json!("https://test.com"), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data["url"], json!("https://test.com"));
    }

    #[test]
    fn test_coerce_value_geojson() {
        let result = coerce_value(
            "geojson-feature-collection",
            &json!({"type": "Point", "coordinates": [1, 2]}),
            None,
        );
        assert!(!result.is_error());
    }

    #[test]
    fn test_coerce_value_boolean() {
        let result = coerce_value("boolean", &json!(true), None);
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(true));
    }

    #[test]
    fn test_coerce_value_domain_value() {
        let result = coerce_value(
            "domain-value",
            &json!("550e8400-e29b-41d4-a716-446655440000"),
            None,
        );
        assert!(!result.is_error());
    }

    #[test]
    fn test_coerce_value_domain_value_list() {
        let result = coerce_value(
            "domain-value-list",
            &json!(["550e8400-e29b-41d4-a716-446655440000"]),
            None,
        );
        assert!(!result.is_error());
    }

    #[test]
    fn test_coerce_value_concept() {
        let result = coerce_value(
            "concept",
            &json!("550e8400-e29b-41d4-a716-446655440000"),
            None,
        );
        assert!(!result.is_error());
        assert_eq!(
            result.tile_data,
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn test_coerce_value_concept_list() {
        let result = coerce_value(
            "concept-list",
            &json!(["550e8400-e29b-41d4-a716-446655440000"]),
            None,
        );
        assert!(!result.is_error());
    }

    #[test]
    fn test_coerce_value_resource_instance() {
        let result = coerce_value(
            "resource-instance",
            &json!("550e8400-e29b-41d4-a716-446655440000"),
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
    fn test_coerce_value_resource_instance_list() {
        let result = coerce_value(
            "resource-instance-list",
            &json!(["550e8400-e29b-41d4-a716-446655440000"]),
            None,
        );
        assert!(!result.is_error());
        let arr = result.tile_data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
    }

    // Round-trip tests
    #[test]
    fn test_domain_value_round_trip() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let result1 = coerce_domain_value(&json!(uuid), None);
        assert!(!result1.is_error());
        let result2 = coerce_domain_value(&result1.tile_data, None);
        assert!(!result2.is_error());
        assert_eq!(result2.tile_data, result1.tile_data);
    }

    #[test]
    fn test_domain_value_list_round_trip() {
        let uuid1 = "550e8400-e29b-41d4-a716-446655440000";
        let uuid2 = "660e8400-e29b-41d4-a716-446655440001";
        let result1 = coerce_domain_value_list(&json!([uuid1, uuid2]), None);
        assert!(!result1.is_error());
        let result2 = coerce_domain_value_list(&result1.tile_data, None);
        assert!(!result2.is_error());
        assert_eq!(result2.tile_data, result1.tile_data);
    }
}
