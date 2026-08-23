// SPDX-License-Identifier: AGPL-3.0-or-later
//! Datatype → index-spec derivation.
//!
//! Single entry point [`datatype_index_spec`] that decides how a datatype's
//! tile value is head-indexed. It is **registry-first**: a registered
//! [`ExtensionTypeHandler`] with the `can_index` capability owns its own
//! datatype's indexing (including which collection its keys belong to). Only
//! when no handler claims the datatype does the built-in fallback run, and
//! that fallback knows **only** the core-owned datatypes — concepts,
//! domain values, and resource-instance links. Extension datatypes are
//! deliberately invisible here; they reach indexing exclusively through their
//! registered handler.
//!
//! This is the seam that lets the emitter delete its hardcoded per-datatype
//! extraction: it calls this and routes on the returned [`IndexClass`].

use serde_json::Value;

use crate::extension_type_registry::{ExtensionTypeRegistry, IndexClass, IndexSpec};

/// Concept-like keys: a bare id string, or an array of id strings.
/// Anything else yields no keys.
fn concept_keys(value: &Value) -> Vec<String> {
    match value {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

/// Ordered-scalar keys (A8.1): the RAW value string(s) — a bare string or an
/// array of strings. Identical shape to [`concept_keys`]; kept separate because
/// the emitter quantizes these (they are not dict-interned ids). An empty/absent
/// value yields no keys, so the emitter writes nothing.
fn scalar_keys(value: &Value) -> Vec<String> {
    match value {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

/// Spatial keys (A8.2): the geometry as a serialized GeoJSON string, for the
/// emitter to parse and extract a bounding box. A geojson tile value is a JSON
/// object (a FeatureCollection), so unlike a concept/date it is serialized whole,
/// not read as a bare string. Null/absent → no key.
fn geojson_keys(value: &Value) -> Vec<String> {
    if value.is_null() {
        Vec::new()
    } else {
        vec![value.to_string()]
    }
}

/// Link keys: a bare target string, an object carrying `resourceId`, or an
/// array of either. Anything else yields no keys.
fn link_keys(value: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => {
            for item in items {
                match item {
                    Value::String(s) => out.push(s.clone()),
                    Value::Object(m) => {
                        if let Some(rid) = m.get("resourceId").and_then(|v| v.as_str()) {
                            out.push(rid.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        Value::Object(m) => {
            if let Some(rid) = m.get("resourceId").and_then(|v| v.as_str()) {
                out.push(rid.to_string());
            }
        }
        _ => {}
    }
    out
}

/// Resolve the collection/controlled-list id from a concept node's config.
/// `rdmCollection` takes precedence over `controlledList`.
fn builtin_collection(config: Option<&Value>) -> Option<String> {
    let config = config?;
    config
        .get("rdmCollection")
        .or_else(|| config.get("controlledList"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Derive the [`IndexSpec`] for a datatype's tile value.
///
/// Registry-first: if a handler is registered for `datatype` and is capable
/// of indexing, its spec wins outright. Otherwise the built-in fallback runs
/// over the core-owned datatypes only; every unknown datatype is
/// [`IndexClass::DetailOnly`].
pub fn datatype_index_spec(
    datatype: &str,
    value: &Value,
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> IndexSpec {
    // Registry first: a capable handler owns its datatype entirely.
    if let Some(reg) = registry {
        if let Some(spec) = reg.index_spec(datatype, value, config) {
            return spec;
        }
    }

    // Built-in fallback: core knows only its own datatypes.
    match datatype {
        "concept" | "concept-value" | "concept-list" | "domain-value" | "domain-value-list" => {
            IndexSpec {
                class: IndexClass::ConceptHierarchical {
                    collection: builtin_collection(config),
                },
                keys: concept_keys(value),
            }
        }
        "resource-instance" | "resource-instance-list" => IndexSpec {
            class: IndexClass::Link,
            keys: link_keys(value),
        },
        // Ordered scalars (A8.1): date/edtf are head-indexed as a sortable key
        // so the head can answer range queries. The keys are the RAW value
        // strings; the emitter quantizes them (per `quantize`).
        "date" | "edtf" => IndexSpec {
            class: IndexClass::Ordered,
            keys: scalar_keys(value),
        },
        // Spatial (A8.2): geometry is head-indexed as its bounding box for coarse
        // overlap queries. The key is the RAW GeoJSON; the emitter extracts the
        // bbox (it owns the geojson dependency — this stays WASM-clean).
        "geojson-feature-collection" => IndexSpec {
            class: IndexClass::SpatialBbox,
            keys: geojson_keys(value),
        },
        _ => IndexSpec {
            class: IndexClass::DetailOnly,
            keys: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension_type_registry::{
        ExtensionError, ExtensionTypeHandler, HandlerCapabilities,
    };
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn concept_string_yields_one_key() {
        let spec = datatype_index_spec("concept", &json!("abc-123"), None, None);
        assert_eq!(
            spec.class,
            IndexClass::ConceptHierarchical { collection: None }
        );
        assert_eq!(spec.keys, vec!["abc-123".to_string()]);
    }

    #[test]
    fn concept_array_yields_all_keys_and_collection() {
        let config = json!({ "rdmCollection": "coll-xyz" });
        let spec =
            datatype_index_spec("concept-list", &json!(["a", "b", "c"]), Some(&config), None);
        assert_eq!(
            spec.class,
            IndexClass::ConceptHierarchical {
                collection: Some("coll-xyz".to_string())
            }
        );
        assert_eq!(spec.keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn resource_instance_object_yields_resource_id() {
        let value = json!([{ "resourceId": "rid-1", "ontologyProperty": "x" }]);
        let spec = datatype_index_spec("resource-instance", &value, None, None);
        assert_eq!(spec.class, IndexClass::Link);
        assert_eq!(spec.keys, vec!["rid-1".to_string()]);
    }

    #[test]
    fn unknown_datatype_is_detail_only() {
        let spec = datatype_index_spec("string", &json!("some text"), None, None);
        assert_eq!(spec.class, IndexClass::DetailOnly);
        assert!(spec.keys.is_empty());
    }

    struct StubHandler;
    impl ExtensionTypeHandler for StubHandler {
        fn capabilities(&self) -> HandlerCapabilities {
            HandlerCapabilities {
                can_index: true,
                ..Default::default()
            }
        }
        fn index_spec(
            &self,
            _tile_data: &Value,
            _config: Option<&Value>,
        ) -> Result<Option<IndexSpec>, ExtensionError> {
            Ok(Some(IndexSpec {
                class: IndexClass::ConceptHierarchical {
                    collection: Some("stub-coll".to_string()),
                },
                keys: vec!["stub-key".to_string()],
            }))
        }
    }

    #[test]
    fn registry_path_wins_over_builtin() {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register("xtest", Arc::new(StubHandler));
        // "xtest" is unknown to the built-in match (would be DetailOnly),
        // but the registered handler claims it.
        let spec = datatype_index_spec("xtest", &json!("ignored"), None, Some(&registry));
        assert_eq!(
            spec.class,
            IndexClass::ConceptHierarchical {
                collection: Some("stub-coll".to_string())
            }
        );
        assert_eq!(spec.keys, vec!["stub-key".to_string()]);
    }
}
