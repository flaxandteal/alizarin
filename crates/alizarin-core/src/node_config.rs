/// Node configuration types and manager for type coercion.
///
/// This module provides platform-agnostic types for:
/// - StaticDomainValue: Domain value option from graph config
/// - NodeConfig: Enum of config types (Domain, Boolean, Concept, Reference)
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
    #[serde(default)]
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
            .or_else(|| self.text.values().next())
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
    #[serde(default, rename = "trueLabel")]
    pub true_label: HashMap<String, String>,
    /// Language code -> label for false value
    #[serde(default, rename = "falseLabel")]
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
            .or_else(|| labels.values().next())
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

/// Reference node configuration (CLM controlled list).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NodeConfigReference {
    /// Controlled list ID
    #[serde(default, rename = "controlledList")]
    pub controlled_list: String,
    /// RDM collection ID (alternative)
    #[serde(default, rename = "rdmCollection")]
    pub rdm_collection: String,
    /// Whether multiple values are allowed
    #[serde(default, rename = "multiValue")]
    pub multi_value: bool,
}

impl NodeConfigReference {
    /// Create a new reference config
    pub fn new(controlled_list: String, rdm_collection: String, multi_value: bool) -> Self {
        Self {
            controlled_list,
            rdm_collection,
            multi_value,
        }
    }

    /// Get the collection ID (controlledList takes precedence over rdmCollection)
    pub fn get_collection_id(&self) -> Option<&str> {
        if !self.controlled_list.is_empty() {
            Some(&self.controlled_list)
        } else if !self.rdm_collection.is_empty() {
            Some(&self.rdm_collection)
        } else {
            None
        }
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

/// Enum of all node config types
#[derive(Clone, Debug)]
pub enum NodeConfig {
    Boolean(NodeConfigBoolean),
    Concept(NodeConfigConcept),
    Reference(NodeConfigReference),
    Domain(NodeConfigDomain),
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

    /// Get as reference config
    pub fn as_reference(&self) -> Option<&NodeConfigReference> {
        match self {
            NodeConfig::Reference(c) => Some(c),
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

    /// Get the config type name
    pub fn type_name(&self) -> &'static str {
        match self {
            NodeConfig::Boolean(_) => "boolean",
            NodeConfig::Concept(_) => "concept",
            NodeConfig::Reference(_) => "reference",
            NodeConfig::Domain(_) => "domain-value",
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
    pub fn build_from_graph(&mut self, graph: &StaticGraph) {
        for node in graph.nodes.iter() {
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

    /// Get reference config for a node
    pub fn get_reference(&self, nodeid: &str) -> Option<&NodeConfigReference> {
        self.configs.get(nodeid).and_then(|c| c.as_reference())
    }

    /// Get domain config for a node
    pub fn get_domain(&self, nodeid: &str) -> Option<&NodeConfigDomain> {
        self.configs.get(nodeid).and_then(|c| c.as_domain())
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
            "reference" => {
                let cfg = self.parse_reference_config(config);
                Some(NodeConfig::Reference(cfg))
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

    /// Parse reference config from node.config HashMap
    fn parse_reference_config(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> NodeConfigReference {
        let controlled_list = config
            .get("controlledList")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let rdm_collection = config
            .get("rdmCollection")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let multi_value = config
            .get("multiValue")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        NodeConfigReference {
            controlled_list,
            rdm_collection,
            multi_value,
        }
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
    fn test_node_config_reference() {
        let config1 = NodeConfigReference::new("clm-id".to_string(), "".to_string(), false);
        assert_eq!(config1.get_collection_id(), Some("clm-id"));

        let config2 = NodeConfigReference::new("".to_string(), "rdm-id".to_string(), true);
        assert_eq!(config2.get_collection_id(), Some("rdm-id"));
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
}
