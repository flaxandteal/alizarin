//! Value-validation capability: the `validate` handler method, the registry
//! dispatcher, and the `validate_business_data` pass run during tree→tiles.
//!
//! Covers a range of happy AND unhappy paths so future refactors stay honest:
//! valid/invalid values, warnings, config-driven rules, `can_validate` gating,
//! unregistered datatypes, handler errors, and the Off/Collect/FailFast modes
//! with diagnostic location.

use std::sync::Arc;

use alizarin_core::extension_type_registry::{
    ExtensionError, ExtensionTypeHandler, ExtensionTypeRegistry, HandlerCapabilities,
    ValidationResult,
};
use alizarin_core::{validate_business_data, BusinessDataWrapper, StaticGraph, ValidationMode};
use serde_json::{json, Value};

/// Requires a positive integer. `config.max` caps it; even values get a warning.
/// Coerce is left as the default pass-through.
struct PositiveIntHandler;
impl ExtensionTypeHandler for PositiveIntHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        HandlerCapabilities {
            can_validate: true,
            ..Default::default()
        }
    }
    fn validate(
        &self,
        value: &Value,
        config: Option<&Value>,
    ) -> Result<ValidationResult, ExtensionError> {
        let n = match value.as_i64() {
            Some(n) => n,
            None => return Ok(ValidationResult::invalid("not an integer")),
        };
        let mut errors = Vec::new();
        if n <= 0 {
            errors.push(format!("must be positive, got {n}"));
        }
        if let Some(max) = config.and_then(|c| c.get("max")).and_then(|v| v.as_i64()) {
            if n > max {
                errors.push(format!("exceeds max {max}"));
            }
        }
        let mut result = ValidationResult::from_errors(errors);
        if n % 2 == 0 {
            result = result.with_warning("even value");
        }
        Ok(result)
    }
}

/// Declares `can_validate = false` yet has a validate impl that would fail —
/// proves the dispatcher gates on the capability and never calls it.
struct GatedOffHandler;
impl ExtensionTypeHandler for GatedOffHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        HandlerCapabilities::default() // can_validate = false
    }
    fn validate(&self, _v: &Value, _c: Option<&Value>) -> Result<ValidationResult, ExtensionError> {
        Ok(ValidationResult::invalid("should never be seen"))
    }
}

/// Validating handler that cannot run at all.
struct BrokenHandler;
impl ExtensionTypeHandler for BrokenHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        HandlerCapabilities {
            can_validate: true,
            ..Default::default()
        }
    }
    fn validate(&self, _v: &Value, _c: Option<&Value>) -> Result<ValidationResult, ExtensionError> {
        Err(ExtensionError::new("validator exploded"))
    }
}

/// `can_validate` but no `validate` override → default accepts everything.
struct DefaultValidateHandler;
impl ExtensionTypeHandler for DefaultValidateHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        HandlerCapabilities {
            can_validate: true,
            ..Default::default()
        }
    }
}

fn registry() -> ExtensionTypeRegistry {
    let mut r = ExtensionTypeRegistry::new();
    r.register("posint".to_string(), Arc::new(PositiveIntHandler));
    r.register("gated".to_string(), Arc::new(GatedOffHandler));
    r.register("broken".to_string(), Arc::new(BrokenHandler));
    r.register("defaulted".to_string(), Arc::new(DefaultValidateHandler));
    r
}

// ── Registry-level (happy) ──────────────────────────────────────────────

#[test]
fn valid_value_passes_clean() {
    let r = registry();
    let res = r.validate("posint", &json!(5), None).unwrap().unwrap();
    assert!(res.valid);
    assert!(res.errors.is_empty());
    assert!(res.warnings.is_empty());
}

#[test]
fn valid_but_warned() {
    let r = registry();
    let res = r.validate("posint", &json!(4), None).unwrap().unwrap();
    assert!(res.valid, "even is valid, just warned");
    assert!(res.errors.is_empty());
    assert_eq!(res.warnings, vec!["even value".to_string()]);
}

#[test]
fn default_validate_handler_accepts_everything() {
    let r = registry();
    let res = r
        .validate("defaulted", &json!("anything"), None)
        .unwrap()
        .unwrap();
    assert!(res.valid && res.errors.is_empty());
}

// ── Registry-level (unhappy) ────────────────────────────────────────────

#[test]
fn negative_value_is_invalid() {
    let r = registry();
    let res = r.validate("posint", &json!(-3), None).unwrap().unwrap();
    assert!(!res.valid);
    assert_eq!(res.errors, vec!["must be positive, got -3".to_string()]);
}

#[test]
fn wrong_type_is_invalid() {
    let r = registry();
    let res = r.validate("posint", &json!("x"), None).unwrap().unwrap();
    assert!(!res.valid);
    assert_eq!(res.errors, vec!["not an integer".to_string()]);
}

#[test]
fn config_driven_max_is_enforced() {
    let r = registry();
    let cfg = json!({ "max": 5 });
    let ok = r
        .validate("posint", &json!(3), Some(&cfg))
        .unwrap()
        .unwrap();
    assert!(ok.valid);
    let bad = r
        .validate("posint", &json!(10), Some(&cfg))
        .unwrap()
        .unwrap();
    assert!(!bad.valid);
    assert_eq!(bad.errors, vec!["exceeds max 5".to_string()]);
}

#[test]
fn multiple_errors_accumulate() {
    let r = registry();
    let cfg = json!({ "max": 5 });
    // -2: negative AND (>max is false), but even → warning; negative → error
    let res = r
        .validate("posint", &json!(-2), Some(&cfg))
        .unwrap()
        .unwrap();
    assert!(!res.valid);
    assert_eq!(res.errors, vec!["must be positive, got -2".to_string()]);
    assert_eq!(res.warnings, vec!["even value".to_string()]);
}

// ── Registry-level (gating / dispatch) ──────────────────────────────────

#[test]
fn unregistered_datatype_is_skipped() {
    let r = registry();
    assert!(r
        .validate("no-such-datatype", &json!(1), None)
        .unwrap()
        .is_none());
}

#[test]
fn handler_without_can_validate_is_never_called() {
    let r = registry();
    // GatedOffHandler.validate would fail, but can_validate=false → skipped.
    assert!(r.validate("gated", &json!(1), None).unwrap().is_none());
}

#[test]
fn handler_error_surfaces_as_err() {
    let r = registry();
    let err = r.validate("broken", &json!(1), None);
    assert!(
        err.is_err(),
        "handler-cannot-run is Err, not an invalid value"
    );
}

// ── Pass-level: validate_business_data over built tiles ─────────────────

fn graph_with_posint(node_config: Value) -> StaticGraph {
    let mut g: StaticGraph = serde_json::from_value(json!({
        "graphid": "g1",
        "name": {"en": "G"},
        "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g1"},
        "nodes": [
            {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g1"},
            {"nodeid": "n1", "name": "Count", "datatype": "posint", "graph_id": "g1",
             "alias": "count", "nodegroup_id": "ng1", "config": node_config}
        ],
        "nodegroups": [{"nodegroupid": "ng1", "cardinality": "n"}],
        "edges": []
    }))
    .unwrap();
    g.build_indices();
    g
}

fn wrapper_with_count(value: Value) -> BusinessDataWrapper {
    serde_json::from_value(json!({
        "business_data": {
            "resources": [{
                "resourceinstance": {"resourceinstanceid": "r1", "graph_id": "g1", "name": "R1", "descriptors": {}},
                "tiles": [{
                    "nodegroup_id": "ng1",
                    "resourceinstance_id": "r1",
                    "data": {"n1": value}
                }]
            }]
        }
    }))
    .unwrap()
}

#[test]
fn pass_off_never_validates() {
    let g = graph_with_posint(json!({}));
    let r = registry();
    let data = wrapper_with_count(json!(-99)); // would be invalid
    let diags = validate_business_data(&data, &g, Some(&r), ValidationMode::Off).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn pass_collect_valid_data_is_clean() {
    let g = graph_with_posint(json!({}));
    let r = registry();
    let data = wrapper_with_count(json!(7));
    let diags = validate_business_data(&data, &g, Some(&r), ValidationMode::Collect).unwrap();
    assert!(diags.is_empty(), "got {diags:?}");
}

#[test]
fn pass_collect_reports_error_with_location() {
    let g = graph_with_posint(json!({}));
    let r = registry();
    let data = wrapper_with_count(json!(-1));
    let diags = validate_business_data(&data, &g, Some(&r), ValidationMode::Collect).unwrap();
    let errors: Vec<_> = diags.iter().filter(|d| !d.warning).collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].node_id, "n1");
    assert_eq!(errors[0].nodegroup_id, "ng1");
    assert_eq!(errors[0].node_alias.as_deref(), Some("count"));
    assert!(errors[0].message.contains("must be positive"));
}

#[test]
fn pass_collect_surfaces_warnings_separately() {
    let g = graph_with_posint(json!({}));
    let r = registry();
    let data = wrapper_with_count(json!(4)); // valid but even → warning
    let diags = validate_business_data(&data, &g, Some(&r), ValidationMode::Collect).unwrap();
    assert_eq!(diags.len(), 1);
    assert!(diags[0].warning);
    assert_eq!(diags[0].message, "even value");
}

#[test]
fn pass_collect_uses_node_config() {
    let g = graph_with_posint(json!({ "max": 5 }));
    let r = registry();
    let data = wrapper_with_count(json!(10)); // >max
    let diags = validate_business_data(&data, &g, Some(&r), ValidationMode::Collect).unwrap();
    assert!(diags
        .iter()
        .any(|d| !d.warning && d.message.contains("exceeds max 5")));
}

#[test]
fn pass_fail_fast_aborts_on_first_error() {
    let g = graph_with_posint(json!({}));
    let r = registry();
    let data = wrapper_with_count(json!(-1));
    let err = validate_business_data(&data, &g, Some(&r), ValidationMode::FailFast);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("must be positive"));
}

#[test]
fn pass_no_registry_is_a_noop() {
    let g = graph_with_posint(json!({}));
    let data = wrapper_with_count(json!(-1));
    let diags = validate_business_data(&data, &g, None, ValidationMode::Collect).unwrap();
    assert!(diags.is_empty());
}
