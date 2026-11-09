/// Node configuration types and manager for type coercion.
///
/// This module provides platform-agnostic types for:
/// - StaticDomainValue: Domain value option from graph config
/// - NodeConfig: Enum of the core datatypes' config types (Domain, Boolean, Concept)
/// - NodeConfigManager: Cache and lookup for node configs
///
/// These types can be used from:
/// - TypeScript via WASM bindings
/// - Python via PyO3 bindings
/// - Native Rust applications
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::graph::StaticGraph;

// =============================================================================
// Domain Value Types
// =============================================================================

/// A domain value option from node config.
///
/// Represents a selectable option with id, selected state, and i18n labels.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StaticDomainValue {
    /// Unique identifier for this value
    pub id: String,
    /// Whether this is the selected/default value
    #[serde(default)]
    pub selected: bool,
    /// Translatable text labels by language code
    #[serde(default, serialize_with = "crate::sorted_serde::sorted_map")]
    pub text: HashMap<String, String>,
}

impl StaticDomainValue {
    /// Create a new domain value
    pub fn new(id: String, selected: bool, text: HashMap<String, String>) -> Self {
        Self { id, selected, text }
    }

    /// Get text for a specific language, with fallback
    pub fn lang(&self, language: &str) -> Option<&str> {
        self.text
            .get(language)
            .or_else(|| self.text.get("en"))
            .or_else(|| crate::sorted_serde::lowest_key_value(&self.text))
            .map(|s| s.as_str())
    }

    /// Get display string (defaults to English)
    pub fn display(&self) -> &str {
        self.lang("en").unwrap_or_default()
    }
}

// =============================================================================
// Node Config Types
// =============================================================================

/// Boolean node configuration with true/false labels.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NodeConfigBoolean {
    /// Language code -> label for true value
    #[serde(
        default,
        rename = "trueLabel",
        serialize_with = "crate::sorted_serde::sorted_map"
    )]
    pub true_label: HashMap<String, String>,
    /// Language code -> label for false value
    #[serde(
        default,
        rename = "falseLabel",
        serialize_with = "crate::sorted_serde::sorted_map"
    )]
    pub false_label: HashMap<String, String>,
    /// i18n property names
    #[serde(default)]
    pub i18n_properties: Vec<String>,
}

impl NodeConfigBoolean {
    /// Create a new boolean config
    pub fn new(true_label: HashMap<String, String>, false_label: HashMap<String, String>) -> Self {
        Self {
            true_label,
            false_label,
            i18n_properties: vec![],
        }
    }

    /// Get label for a boolean value in a specific language
    pub fn get_label(&self, value: bool, language: &str) -> Option<&str> {
        let labels = if value {
            &self.true_label
        } else {
            &self.false_label
        };
        labels
            .get(language)
            .or_else(|| labels.get("en"))
            .or_else(|| crate::sorted_serde::lowest_key_value(labels))
            .map(|s| s.as_str())
    }
}

/// Concept node configuration (RDM collection reference).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NodeConfigConcept {
    /// RDM collection ID
    #[serde(default, rename = "rdmCollection")]
    pub rdm_collection: String,
}

impl NodeConfigConcept {
    /// Create a new concept config
    pub fn new(rdm_collection: String) -> Self {
        Self { rdm_collection }
    }
}

/// Domain value node configuration with options list.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NodeConfigDomain {
    /// Available domain value options
    #[serde(default)]
    pub options: Vec<StaticDomainValue>,
    /// i18n configuration
    #[serde(default)]
    pub i18n_config: HashMap<String, String>,
}

impl NodeConfigDomain {
    /// Create a new domain config
    pub fn new(options: Vec<StaticDomainValue>) -> Self {
        Self {
            options,
            i18n_config: HashMap::new(),
        }
    }

    /// Get the selected option (if any)
    pub fn get_selected(&self) -> Option<&StaticDomainValue> {
        self.options.iter().find(|opt| opt.selected)
    }

    /// Find option by ID
    pub fn value_from_id(&self, id: &str) -> Option<&StaticDomainValue> {
        self.options.iter().find(|opt| opt.id == id)
    }

    /// Get all option IDs
    pub fn get_option_ids(&self) -> Vec<&str> {
        self.options.iter().map(|opt| opt.id.as_str()).collect()
    }
}

// =============================================================================
// Node Config Enum
// =============================================================================

/// Node config types. Core models only its own datatypes; an extension
/// datatype's (e.g. CLM's `reference`) is kept raw, and the extension's handler
/// interprets it ([`NodeConfigManager::get_extension_config`]).
#[derive(Clone, Debug)]
pub enum NodeConfig {
    Boolean(NodeConfigBoolean),
    Concept(NodeConfigConcept),
    Domain(NodeConfigDomain),
    /// An extension datatype's raw node config, untouched by core.
    Extension {
        datatype: String,
        config: serde_json::Value,
        /// The handler's [`display_as_list`](crate::extension_type_registry::ExtensionTypeHandler::display_as_list)
        /// answer, asked once per node (see [`NodeConfig::extension_display_as_list`]).
        display_as_list: std::sync::OnceLock<bool>,
    },
}

impl NodeConfig {
    /// Get as boolean config
    pub fn as_boolean(&self) -> Option<&NodeConfigBoolean> {
        match self {
            NodeConfig::Boolean(c) => Some(c),
            _ => None,
        }
    }

    /// Get as concept config
    pub fn as_concept(&self) -> Option<&NodeConfigConcept> {
        match self {
            NodeConfig::Concept(c) => Some(c),
            _ => None,
        }
    }

    /// Get as domain config
    pub fn as_domain(&self) -> Option<&NodeConfigDomain> {
        match self {
            NodeConfig::Domain(c) => Some(c),
            _ => None,
        }
    }

    /// Get an extension datatype's raw config
    pub fn as_extension(&self) -> Option<&serde_json::Value> {
        match self {
            NodeConfig::Extension { config, .. } => Some(config),
            _ => None,
        }
    }

    /// Whether an extension node's values display as a list, per `handler`.
    /// `None` if this isn't an extension config.
    ///
    /// Asked once per node and cached. The answer depends only on the node's
    /// config, and on wasm each ask crosses into JS (and often a second wasm
    /// module), so asking per value would put boundary crossings on the
    /// serialization hot path.
    pub fn extension_display_as_list(
        &self,
        handler: &dyn crate::extension_type_registry::ExtensionTypeHandler,
    ) -> Option<bool> {
        match self {
            NodeConfig::Extension {
                config,
                display_as_list,
                ..
            } => Some(*display_as_list.get_or_init(|| handler.display_as_list(Some(config)))),
            _ => None,
        }
    }

    /// Get the config type name
    pub fn type_name(&self) -> &'static str {
        match self {
            NodeConfig::Boolean(_) => "boolean",
            NodeConfig::Concept(_) => "concept",
            NodeConfig::Domain(_) => "domain-value",
            NodeConfig::Extension { .. } => "extension",
        }
    }
}

// =============================================================================
// Node Config Manager
// =============================================================================

/// Manager for caching and retrieving node configurations.
///
/// Builds configs from graph node data and caches them by node ID.
#[derive(Debug, Default)]
pub struct NodeConfigManager {
    /// Cache of configs by node ID
    configs: HashMap<String, NodeConfig>,
}

impl NodeConfigManager {
    /// Create a new empty manager
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Build configs from a graph
    pub fn build_from_graph(&mut self, graph: &impl crate::GraphLookup) {
        for node in graph.nodes_slice().iter() {
            if let Some(config) = self.build_config_for_node(&node.datatype, &node.config) {
                self.configs.insert(node.nodeid.clone(), config);
            }
        }
    }

    /// Build configs from a JSON string
    pub fn from_graph_json(&mut self, graph_json: &str) -> Result<(), String> {
        let graph: StaticGraph = serde_json::from_str(graph_json)
            .map_err(|e| format!("Failed to parse graph: {}", e))?;

        self.build_from_graph(&graph);
        Ok(())
    }

    /// Get boolean config for a node
    pub fn get_boolean(&self, nodeid: &str) -> Option<&NodeConfigBoolean> {
        self.configs.get(nodeid).and_then(|c| c.as_boolean())
    }

    /// Get concept config for a node
    pub fn get_concept(&self, nodeid: &str) -> Option<&NodeConfigConcept> {
        self.configs.get(nodeid).and_then(|c| c.as_concept())
    }

    /// Get domain config for a node
    pub fn get_domain(&self, nodeid: &str) -> Option<&NodeConfigDomain> {
        self.configs.get(nodeid).and_then(|c| c.as_domain())
    }

    /// An extension datatype's node config, as its registered handler
    /// normalises it ([`node_config`](crate::extension_type_registry::ExtensionTypeHandler::node_config)).
    ///
    /// `None` if the node has no extension config, or no handler for its
    /// datatype is in `registry`. Consulted at lookup, so it doesn't matter
    /// whether the extension registered before or after the graph loaded.
    pub fn get_extension_config(
        &self,
        nodeid: &str,
        registry: Option<&crate::extension_type_registry::ExtensionTypeRegistry>,
    ) -> Option<serde_json::Value> {
        match self.configs.get(nodeid)? {
            NodeConfig::Extension {
                datatype, config, ..
            } => registry?.get(datatype)?.node_config(config),
            _ => None,
        }
    }

    /// Look up domain value by ID
    pub fn lookup_domain_value(&self, nodeid: &str, value_id: &str) -> Option<&StaticDomainValue> {
        self.configs
            .get(nodeid)
            .and_then(|c| c.as_domain())
            .and_then(|d| d.value_from_id(value_id))
    }

    /// Check if a node has config
    pub fn has_config(&self, nodeid: &str) -> bool {
        self.configs.contains_key(nodeid)
    }

    /// Get the config type for a node
    pub fn get_config_type(&self, nodeid: &str) -> Option<&'static str> {
        self.configs.get(nodeid).map(|c| c.type_name())
    }

    /// Get config for a node
    pub fn get(&self, nodeid: &str) -> Option<&NodeConfig> {
        self.configs.get(nodeid)
    }

    /// Clear all cached configs
    pub fn clear(&mut self) {
        self.configs.clear();
    }

    /// Get number of cached configs
    pub fn len(&self) -> usize {
        self.configs.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }

    /// Build config for a single node based on its datatype
    fn build_config_for_node(
        &self,
        datatype: &str,
        config: &HashMap<String, serde_json::Value>,
    ) -> Option<NodeConfig> {
        match datatype {
            "boolean" => {
                let cfg = self.parse_boolean_config(config);
                Some(NodeConfig::Boolean(cfg))
            }
            "domain-value" | "domain-value-list" => {
                let cfg = self.parse_domain_config(config);
                Some(NodeConfig::Domain(cfg))
            }
            "concept" | "concept-list" => {
                let cfg = self.parse_concept_config(config);
                Some(NodeConfig::Concept(cfg))
            }
            dt if crate::type_coercion::is_extension_datatype(dt) && !config.is_empty() => {
                Some(NodeConfig::Extension {
                    datatype: dt.to_string(),
                    config: serde_json::Value::Object(
                        config.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    ),
                    display_as_list: std::sync::OnceLock::new(),
                })
            }
            _ => None,
        }
    }

    /// Parse boolean config from node.config HashMap
    fn parse_boolean_config(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> NodeConfigBoolean {
        let true_label = config
            .get("trueLabel")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let false_label = config
            .get("falseLabel")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let i18n_properties = config
            .get("i18n_properties")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        NodeConfigBoolean {
            true_label,
            false_label,
            i18n_properties,
        }
    }

    /// Parse domain config from node.config HashMap
    fn parse_domain_config(&self, config: &HashMap<String, serde_json::Value>) -> NodeConfigDomain {
        let options: Vec<StaticDomainValue> = config
            .get("options")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let i18n_config = config
            .get("i18n_config")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        NodeConfigDomain {
            options,
            i18n_config,
        }
    }

    /// Parse concept config from node.config HashMap
    fn parse_concept_config(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> NodeConfigConcept {
        let rdm_collection = config
            .get("rdmCollection")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        NodeConfigConcept { rdm_collection }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_domain_value() {
        let mut text = HashMap::new();
        text.insert("en".to_string(), "English".to_string());
        text.insert("es".to_string(), "Español".to_string());

        let dv = StaticDomainValue::new("test-id".to_string(), true, text);

        assert_eq!(dv.id, "test-id");
        assert!(dv.selected);
        assert_eq!(dv.lang("en"), Some("English"));
        assert_eq!(dv.lang("es"), Some("Español"));
        assert_eq!(dv.lang("de"), Some("English")); // Falls back to en
    }

    #[test]
    fn test_node_config_boolean() {
        let mut true_label = HashMap::new();
        true_label.insert("en".to_string(), "Yes".to_string());

        let mut false_label = HashMap::new();
        false_label.insert("en".to_string(), "No".to_string());

        let config = NodeConfigBoolean::new(true_label, false_label);

        assert_eq!(config.get_label(true, "en"), Some("Yes"));
        assert_eq!(config.get_label(false, "en"), Some("No"));
    }

    #[test]
    fn test_node_config_domain() {
        let opt1 = StaticDomainValue::new("opt-1".to_string(), false, HashMap::new());
        let opt2 = StaticDomainValue::new("opt-2".to_string(), true, HashMap::new());

        let config = NodeConfigDomain::new(vec![opt1, opt2]);

        assert_eq!(config.options.len(), 2);
        assert_eq!(config.get_selected().map(|o| o.id.as_str()), Some("opt-2"));
        assert!(config.value_from_id("opt-1").is_some());
    }

    #[test]
    fn extension_configs_are_kept_raw_and_read_through_the_handler() {
        use crate::extension_type_registry::{
            ExtensionTypeHandler, ExtensionTypeRegistry, HandlerCapabilities,
        };
        use serde_json::{json, Value};

        struct Upper;
        impl ExtensionTypeHandler for Upper {
            fn capabilities(&self) -> HandlerCapabilities {
                HandlerCapabilities::default()
            }
            fn node_config(&self, config: &Value) -> Option<Value> {
                Some(json!({ "list": config["list"].as_str()?.to_uppercase() }))
            }
        }

        let mut manager = NodeConfigManager::new();
        let raw: HashMap<String, Value> = [("list".to_string(), json!("abc"))].into();
        let built = manager.build_config_for_node("x-list", &raw).unwrap();
        assert_eq!(built.as_extension(), Some(&json!({ "list": "abc" })));
        assert_eq!(built.type_name(), "extension");
        manager.configs.insert("n1".to_string(), built);

        assert_eq!(
            manager.get_extension_config("n1", None),
            None,
            "no registry"
        );
        let mut registry = ExtensionTypeRegistry::new();
        assert_eq!(
            manager.get_extension_config("n1", Some(&registry)),
            None,
            "no handler for the datatype"
        );
        registry.register("x-list".to_string(), std::sync::Arc::new(Upper));
        assert_eq!(
            manager.get_extension_config("n1", Some(&registry)),
            Some(json!({ "list": "ABC" }))
        );

        assert!(
            manager.build_config_for_node("string", &raw).is_none(),
            "core datatypes are not stored raw"
        );
        assert!(
            manager
                .build_config_for_node("x-list", &HashMap::new())
                .is_none(),
            "an empty config is no config"
        );
    }

    #[test]
    fn display_as_list_is_asked_once_per_node() {
        use crate::extension_type_registry::{ExtensionTypeHandler, HandlerCapabilities};
        use serde_json::{json, Value};
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Counting(AtomicUsize);
        impl ExtensionTypeHandler for Counting {
            fn capabilities(&self) -> HandlerCapabilities {
                HandlerCapabilities::default()
            }
            fn display_as_list(&self, config: Option<&Value>) -> bool {
                self.0.fetch_add(1, Ordering::SeqCst);
                config.and_then(|c| c["multi"].as_bool()).unwrap_or(false)
            }
        }

        let manager = NodeConfigManager::new();
        let raw: HashMap<String, Value> = [("multi".to_string(), json!(true))].into();
        let built = manager.build_config_for_node("x-list", &raw).unwrap();
        let handler = Counting(AtomicUsize::new(0));
        for _ in 0..3 {
            assert_eq!(built.extension_display_as_list(&handler), Some(true));
        }
        assert_eq!(handler.0.load(Ordering::SeqCst), 1);

        let boolean = manager
            .build_config_for_node("boolean", &HashMap::new())
            .unwrap();
        assert_eq!(boolean.extension_display_as_list(&handler), None);
    }
}
