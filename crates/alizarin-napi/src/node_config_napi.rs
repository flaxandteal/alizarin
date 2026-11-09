//! NAPI bindings for node configuration types, mirroring the WASM ones
//! (`alizarin-wasm/src/node_config_wasm.rs`) method for method, so the shared
//! TypeScript `NodeConfigManager` works on either backend. All logic is core's.
//! Core datatypes only: extension datatypes (e.g. CLM's `reference`) read their
//! own node config.

use napi_derive::napi;
use serde_json::Value;

use alizarin_core::node_config::{
    NodeConfigBoolean, NodeConfigConcept, NodeConfigDomain, StaticDomainValue,
};

/// A label map as a JS object, keys sorted (matches serialised configs).
fn labels(map: &std::collections::HashMap<String, String>) -> Value {
    serde_json::to_value(map.iter().collect::<std::collections::BTreeMap<_, _>>())
        .unwrap_or(Value::Null)
}

/// A domain value option from node config.
#[napi]
pub struct NapiStaticDomainValue {
    inner: StaticDomainValue,
}

#[napi]
impl NapiStaticDomainValue {
    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[napi(getter)]
    pub fn selected(&self) -> bool {
        self.inner.selected
    }

    /// Text for a language, falling back to `en`, then the lowest language key.
    #[napi]
    pub fn lang(&self, language: String) -> Option<String> {
        self.inner.lang(&language).map(str::to_string)
    }

    /// Display string (defaults to English).
    #[napi]
    pub fn display(&self) -> String {
        self.inner.display().to_string()
    }

    /// All text labels, keyed by language.
    #[napi(getter)]
    pub fn text(&self) -> Value {
        labels(&self.inner.text)
    }
}

impl From<&StaticDomainValue> for NapiStaticDomainValue {
    fn from(inner: &StaticDomainValue) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

/// Boolean node configuration with true/false labels.
#[napi]
pub struct NapiNodeConfigBoolean {
    inner: NodeConfigBoolean,
}

#[napi]
impl NapiNodeConfigBoolean {
    #[napi(js_name = "getLabel")]
    pub fn get_label(&self, value: bool, language: String) -> Option<String> {
        self.inner.get_label(value, &language).map(str::to_string)
    }

    #[napi(getter, js_name = "trueLabel")]
    pub fn true_label(&self) -> Value {
        labels(&self.inner.true_label)
    }

    #[napi(getter, js_name = "falseLabel")]
    pub fn false_label(&self) -> Value {
        labels(&self.inner.false_label)
    }
}

impl From<&NodeConfigBoolean> for NapiNodeConfigBoolean {
    fn from(inner: &NodeConfigBoolean) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

/// Concept node configuration (RDM collection reference).
#[napi]
pub struct NapiNodeConfigConcept {
    inner: NodeConfigConcept,
}

#[napi]
impl NapiNodeConfigConcept {
    #[napi(getter, js_name = "rdmCollection")]
    pub fn rdm_collection(&self) -> String {
        self.inner.rdm_collection.clone()
    }
}

impl From<&NodeConfigConcept> for NapiNodeConfigConcept {
    fn from(inner: &NodeConfigConcept) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

/// Domain value node configuration with an options list.
#[napi]
pub struct NapiNodeConfigDomain {
    inner: NodeConfigDomain,
}

#[napi]
impl NapiNodeConfigDomain {
    #[napi(getter)]
    pub fn options(&self) -> Vec<NapiStaticDomainValue> {
        self.inner.options.iter().map(Into::into).collect()
    }

    #[napi(js_name = "getSelected")]
    pub fn get_selected(&self) -> Option<NapiStaticDomainValue> {
        self.inner.get_selected().map(Into::into)
    }

    #[napi(js_name = "valueFromId")]
    pub fn value_from_id(&self, id: String) -> Option<NapiStaticDomainValue> {
        self.inner.value_from_id(&id).map(Into::into)
    }

    #[napi(js_name = "getOptionIds")]
    pub fn get_option_ids(&self) -> Vec<String> {
        self.inner
            .get_option_ids()
            .iter()
            .map(|id| id.to_string())
            .collect()
    }
}

impl From<&NodeConfigDomain> for NapiNodeConfigDomain {
    fn from(inner: &NodeConfigDomain) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}
