//! Core graph data structures for Arches/Alizarin
//!
//! These are platform-agnostic types that can be used from:
//! - JavaScript/TypeScript via WASM bindings (alizarin main crate)
//! - Native Rust applications (alizarin-explorer)
//! - Python via PyO3 bindings (alizarin-python)

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

// ============================================================================
// StaticTranslatableString
// ============================================================================

/// A string with translations for multiple languages
#[derive(Clone, Debug, Default)]
pub struct StaticTranslatableString {
    pub translations: HashMap<String, String>,
    pub lang: String,
}

impl StaticTranslatableString {
    /// Create a new translatable string from a map of translations
    pub fn from_translations(translations: HashMap<String, String>, default_lang: Option<String>) -> Self {
        let lang = default_lang.unwrap_or_else(|| "en".to_string());
        let actual_lang = if translations.contains_key(&lang) {
            lang
        } else {
            translations.keys().next().cloned().unwrap_or_else(|| "en".to_string())
        };
        StaticTranslatableString {
            translations,
            lang: actual_lang,
        }
    }

    /// Create from a simple string (assumes English)
    pub fn from_string(s: String) -> Self {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), s);
        StaticTranslatableString {
            translations,
            lang: "en".to_string(),
        }
    }

    /// Get the string for a specific language, falling back to any available
    pub fn get(&self, lang: &str) -> String {
        self.translations
            .get(lang)
            .or_else(|| self.translations.get("en"))
            .or_else(|| self.translations.values().next())
            .cloned()
            .unwrap_or_default()
    }

    /// Get the string using the default language
    pub fn to_string_default(&self) -> String {
        self.get(&self.lang)
    }

    /// Copy/clone the translatable string
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.translations).unwrap_or(serde_json::Value::Null)
    }
}

impl std::fmt::Display for StaticTranslatableString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_default())
    }
}

impl Serialize for StaticTranslatableString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.translations.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StaticTranslatableString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde_json::Value;
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Object(map) => {
                let translations: HashMap<String, String> = map
                    .into_iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                    .collect();

                let lang = translations
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "en".to_string());

                Ok(StaticTranslatableString { translations, lang })
            }
            Value::String(s) => {
                let mut translations = HashMap::new();
                translations.insert("en".to_string(), s);
                Ok(StaticTranslatableString {
                    translations,
                    lang: "en".to_string(),
                })
            }
            _ => {
                let mut translations = HashMap::new();
                translations.insert("en".to_string(), String::new());
                Ok(StaticTranslatableString {
                    translations,
                    lang: "en".to_string(),
                })
            }
        }
    }
}

// ============================================================================
// StaticNode
// ============================================================================

/// A node in the graph representing a data field or structural element
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticNode {
    pub nodeid: String,
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    pub datatype: String,
    #[serde(default)]
    pub nodegroup_id: Option<String>,
    pub graph_id: String,
    #[serde(default)]
    pub is_collector: bool,
    #[serde(default)]
    pub isrequired: bool,
    #[serde(default)]
    pub exportable: bool,
    #[serde(default)]
    pub sortorder: Option<i32>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub parentproperty: Option<String>,
    #[serde(default)]
    pub ontologyclass: Option<String>,
    #[serde(default)]
    pub description: Option<StaticTranslatableString>,
    #[serde(default)]
    pub fieldname: Option<String>,
    #[serde(default)]
    pub hascustomalias: bool,
    #[serde(default)]
    pub issearchable: bool,
    #[serde(default)]
    pub istopnode: bool,
    #[serde(default)]
    pub sourcebranchpublication_id: Option<String>,
}

impl StaticNode {
    /// Check if this node is the root node (no nodegroup_id)
    pub fn is_root(&self) -> bool {
        self.nodegroup_id.is_none()
            || self.nodegroup_id
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
    }

    /// Get the display name
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Get the alias or empty string
    pub fn display_alias(&self) -> &str {
        self.alias.as_deref().unwrap_or("")
    }

    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

// ============================================================================
// StaticNodegroup
// ============================================================================

/// A nodegroup defining cardinality and grouping of nodes
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticNodegroup {
    pub nodegroupid: String,
    #[serde(default)]
    pub cardinality: Option<String>,
    #[serde(default)]
    pub parentnodegroup_id: Option<String>,
    #[serde(default)]
    pub legacygroupid: Option<String>,
}

impl StaticNodegroup {
    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

// ============================================================================
// StaticEdge
// ============================================================================

/// An edge connecting two nodes (domain -> range)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticEdge {
    pub domainnode_id: String,
    pub rangenode_id: String,
    #[serde(default)]
    pub edgeid: String,
    #[serde(default)]
    pub graph_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ontologyproperty: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl StaticEdge {
    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

// ============================================================================
// StaticGraph
// ============================================================================

/// The main graph structure containing nodes, edges, and metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticGraph {
    pub graphid: String,
    pub name: StaticTranslatableString,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub subtitle: Option<StaticTranslatableString>,
    #[serde(default)]
    pub description: Option<StaticTranslatableString>,
    pub nodes: Vec<StaticNode>,
    #[serde(default)]
    pub nodegroups: Vec<StaticNodegroup>,
    #[serde(default)]
    pub edges: Vec<StaticEdge>,
    pub root: StaticNode,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub iconclass: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub isresource: Option<bool>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub is_editable: Option<bool>,
    #[serde(default)]
    pub ontology_id: Option<String>,
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub deploymentdate: Option<String>,
    #[serde(default)]
    pub deploymentfile: Option<String>,
    #[serde(default)]
    pub jsonldcontext: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub relatable_resource_model_ids: Vec<String>,
    #[serde(default)]
    pub publication: Option<serde_json::Value>,
    #[serde(default)]
    pub resource_2_resource_constraints: Option<Vec<serde_json::Value>>,
    // UI-specific fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<StaticCard>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cards_x_nodes_x_widgets: Option<Vec<StaticCardsXNodesXWidgets>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub functions_x_graphs: Option<Vec<StaticFunctionsXGraphs>>,

    // Internal lookup tables (not serialized)
    #[serde(skip)]
    node_by_id: Option<HashMap<String, usize>>,
    #[serde(skip)]
    node_by_alias: Option<HashMap<String, usize>>,
}

/// Wrapper for loading from JSON files with {graph: [...]} structure
#[derive(Debug, Deserialize)]
pub struct GraphWrapper {
    pub graph: Vec<StaticGraph>,
}

impl StaticGraph {
    /// Load a graph from a JSON string
    /// Handles both direct graph objects and wrapped format: {"graph": [...]}
    pub fn from_json_string(json_str: &str) -> Result<StaticGraph, String> {
        // Try parsing as a direct StaticGraph first
        if let Ok(mut graph) = serde_json::from_str::<StaticGraph>(json_str) {
            graph.build_indices();
            return Ok(graph);
        }

        // Fall back to wrapped format {graph: [...]}
        let wrapper: GraphWrapper =
            serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        let mut graph = wrapper
            .graph
            .into_iter()
            .next()
            .ok_or_else(|| "No graphs found in JSON".to_string())?;
        graph.build_indices();
        Ok(graph)
    }

    /// Build the internal lookup indices
    pub fn build_indices(&mut self) {
        let mut node_by_id = HashMap::new();
        let mut node_by_alias = HashMap::new();

        for (idx, node) in self.nodes.iter().enumerate() {
            node_by_id.insert(node.nodeid.clone(), idx);
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    node_by_alias.insert(alias.clone(), idx);
                }
            }
        }

        self.node_by_id = Some(node_by_id);
        self.node_by_alias = Some(node_by_alias);
    }

    /// Get the root node
    pub fn get_root(&self) -> &StaticNode {
        &self.root
    }

    /// Get node by index
    pub fn get_node_by_index(&self, idx: usize) -> Option<&StaticNode> {
        self.nodes.get(idx)
    }

    /// Get node by ID
    pub fn get_node_by_id(&self, id: &str) -> Option<&StaticNode> {
        self.node_by_id
            .as_ref()?
            .get(id)
            .and_then(|&idx| self.nodes.get(idx))
    }

    /// Get node by alias
    pub fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        self.node_by_alias
            .as_ref()?
            .get(alias)
            .and_then(|&idx| self.nodes.get(idx))
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        self.name.to_string_default()
    }

    /// Get subtitle
    pub fn display_subtitle(&self) -> String {
        self.subtitle
            .as_ref()
            .map(|s| s.to_string_default())
            .unwrap_or_default()
    }

    /// Get author
    pub fn display_author(&self) -> String {
        self.author.clone().unwrap_or_default()
    }

    /// Get nodes slice
    pub fn nodes_slice(&self) -> &[StaticNode] {
        &self.nodes
    }

    /// Get nodegroups slice
    pub fn nodegroups_slice(&self) -> &[StaticNodegroup] {
        &self.nodegroups
    }

    /// Get edges slice
    pub fn edges_slice(&self) -> &[StaticEdge] {
        &self.edges
    }

    /// Get root node
    pub fn root_node(&self) -> &StaticNode {
        &self.root
    }

    /// Get graph ID
    pub fn graph_id(&self) -> &str {
        &self.graphid
    }
}

// ============================================================================
// IndexedGraph - Graph with precomputed indices for fast traversal
// ============================================================================

/// Graph with precomputed indices for efficient tree traversal
pub struct IndexedGraph {
    pub graph: StaticGraph,
    /// node_id -> StaticNode
    pub nodes_by_id: HashMap<String, StaticNode>,
    /// node_id -> [child_node_ids]
    pub children_by_node: HashMap<String, Vec<String>>,
    /// alias -> StaticNode
    pub nodes_by_alias: HashMap<String, StaticNode>,
    /// nodegroup_id -> StaticNodegroup
    pub nodegroups_by_id: HashMap<String, StaticNodegroup>,
}

impl IndexedGraph {
    /// Create an indexed graph from a StaticGraph
    pub fn new(graph: StaticGraph) -> Self {
        let mut nodes_by_id = HashMap::new();
        let mut nodes_by_alias = HashMap::new();
        let mut children_by_node: HashMap<String, Vec<String>> = HashMap::new();
        let mut nodegroups_by_id = HashMap::new();

        // Index nodes by ID and alias
        for node in &graph.nodes {
            nodes_by_id.insert(node.nodeid.clone(), node.clone());
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    nodes_by_alias.insert(alias.clone(), node.clone());
                }
            }
        }

        // Index edges (domainnode_id -> rangenode_id)
        for edge in &graph.edges {
            children_by_node
                .entry(edge.domainnode_id.clone())
                .or_default()
                .push(edge.rangenode_id.clone());
        }

        // Index nodegroups
        for ng in &graph.nodegroups {
            nodegroups_by_id.insert(ng.nodegroupid.clone(), ng.clone());
        }

        IndexedGraph {
            graph,
            nodes_by_id,
            nodes_by_alias,
            children_by_node,
            nodegroups_by_id,
        }
    }

    /// Get root node
    pub fn get_root(&self) -> &StaticNode {
        self.graph.get_root()
    }

    /// Get node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&StaticNode> {
        self.nodes_by_id.get(node_id)
    }

    /// Get node by alias
    pub fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        self.nodes_by_alias.get(alias)
    }

    /// Get child nodes for a given node ID
    pub fn get_children(&self, node_id: &str) -> Vec<&StaticNode> {
        self.children_by_node
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes_by_id.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get child node IDs for a given node
    pub fn get_child_ids(&self, node_id: &str) -> Vec<&String> {
        self.children_by_node
            .get(node_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Check if a node has children
    pub fn has_children(&self, node_id: &str) -> bool {
        self.children_by_node
            .get(node_id)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    /// Get the nodegroup for a node
    pub fn get_nodegroup(&self, node: &StaticNode) -> Option<&StaticNodegroup> {
        node.nodegroup_id
            .as_ref()
            .and_then(|id| self.nodegroups_by_id.get(id))
    }

    /// Build resource descriptors from tiles using graph configuration
    ///
    /// This replaces the TypeScript buildResourceDescriptors function, making
    /// descriptor computation completely platform-independent.
    ///
    /// # Arguments
    /// * `tiles` - The resource tiles containing data values
    ///
    /// # Returns
    /// Populated StaticResourceDescriptors with name, description, map_popup fields
    pub fn build_descriptors(&self, tiles: &[StaticTile]) -> StaticResourceDescriptors {
        // Get descriptor config from graph
        let config = match self.get_descriptor_config() {
            Some(c) => c,
            None => return StaticResourceDescriptors::default(),
        };

        let mut descriptors = StaticResourceDescriptors::default();

        // Process each descriptor type (name, description, map_popup)
        for (descriptor_type, type_config) in &config.descriptor_types {
            let mut template = type_config.string_template.clone();

            // Extract placeholders from template (e.g., <Node Name>)
            let placeholders = Self::extract_placeholders(&template);
            if placeholders.is_empty() {
                continue;
            }

            // Find tiles for this nodegroup
            let relevant_tiles: Vec<&StaticTile> = tiles
                .iter()
                .filter(|t| t.nodegroup_id == type_config.nodegroup_id)
                .collect();

            if relevant_tiles.is_empty() {
                continue;
            }

            // Replace each placeholder with actual value from tiles
            for placeholder in &placeholders {
                // Remove < > from placeholder to get node name
                let node_name = placeholder.trim_start_matches('<').trim_end_matches('>');

                // Find node by name in this nodegroup
                if let Some(node) = self.find_node_by_name_in_nodegroup(node_name, &type_config.nodegroup_id) {
                    // Extract value from tiles
                    if let Some(value) = Self::extract_value_from_tiles(&relevant_tiles, &node.nodeid) {
                        template = template.replace(placeholder, &value);
                    }
                }
            }

            // Assign to appropriate descriptor field
            match descriptor_type.as_str() {
                "name" => descriptors.name = Some(template),
                "description" => descriptors.description = Some(template),
                "map_popup" => descriptors.map_popup = Some(template),
                _ => {} // Unknown descriptor type, ignore
            }
        }

        descriptors
    }

    /// Extract descriptor config from graph.functions_x_graphs
    fn get_descriptor_config(&self) -> Option<DescriptorConfig> {
        let functions_x_graphs = self.graph.functions_x_graphs.as_ref()?;

        for func in functions_x_graphs {
            if func.function_id == DESCRIPTOR_FUNCTION_ID {
                // Parse config as DescriptorConfig
                return serde_json::from_value(func.config.clone()).ok();
            }
        }
        None
    }

    /// Extract placeholders from template using regex pattern
    /// Finds patterns like <Node Name> in the template string
    fn extract_placeholders(template: &str) -> Vec<String> {
        // Pattern matches: <[A-Za-z _-]+>
        // For now, use a simple manual parser to avoid regex dependency
        let mut placeholders = Vec::new();
        let mut in_placeholder = false;
        let mut current = String::new();

        for ch in template.chars() {
            if ch == '<' {
                in_placeholder = true;
                current.clear();
                current.push(ch);
            } else if ch == '>' && in_placeholder {
                current.push(ch);
                placeholders.push(current.clone());
                in_placeholder = false;
                current.clear();
            } else if in_placeholder {
                current.push(ch);
            }
        }

        placeholders
    }

    /// Find node by name within a specific nodegroup
    fn find_node_by_name_in_nodegroup(&self, name: &str, nodegroup_id: &str) -> Option<&StaticNode> {
        self.nodes_by_id
            .values()
            .find(|node| {
                node.name == name &&
                node.nodegroup_id.as_ref().map(|id| id == nodegroup_id).unwrap_or(false)
            })
    }

    /// Extract value from tiles for a given node ID
    fn extract_value_from_tiles(tiles: &[&StaticTile], node_id: &str) -> Option<String> {
        for tile in tiles {
            if let Some(value) = tile.data.get(node_id) {
                if let Some(extracted) = Self::extract_string_from_json(value) {
                    return Some(extracted);
                }
            }
        }
        None
    }

    /// Extract string value from JSON, handling language-nested objects
    /// Handles both simple strings and language objects like {"en": "value"}
    fn extract_string_from_json(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Object(map) => {
                // Try to get language-specific value (e.g., {"en": "value"})
                map.get("en")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        // Fallback to first available language
                        map.values()
                            .find_map(|v| v.as_str())
                    })
                    .map(|s| s.to_string())
            }
            _ => None,
        }
    }
}

// ============================================================================
// StaticGraphMeta - Lightweight graph metadata without full node/edge data
// ============================================================================

/// Lightweight metadata about a graph, without the full nodes/edges arrays.
/// Used for listing graphs without loading all their data.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticGraphMeta {
    pub graphid: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub cards: Option<u32>,
    #[serde(default)]
    pub cards_x_nodes_x_widgets: Option<u32>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<StaticTranslatableString>,
    #[serde(default)]
    pub edges: Option<u32>,
    #[serde(default)]
    pub iconclass: Option<String>,
    #[serde(default)]
    pub is_editable: Option<bool>,
    #[serde(default)]
    pub isresource: Option<bool>,
    /// JSON-LD context - can be a string (URL) or an object (inline context)
    #[serde(default)]
    pub jsonldcontext: Option<serde_json::Value>,
    #[serde(default)]
    pub name: Option<StaticTranslatableString>,
    #[serde(default)]
    pub nodegroups: Option<u32>,
    #[serde(default)]
    pub nodes: Option<u32>,
    #[serde(default)]
    pub ontology_id: Option<String>,
    #[serde(default)]
    pub publication: Option<HashMap<String, Option<String>>>,
    #[serde(default)]
    pub relatable_resource_model_ids: Vec<String>,
    #[serde(default)]
    pub resource_2_resource_constraints: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub root: Option<Box<StaticNode>>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub subtitle: Option<StaticTranslatableString>,
    #[serde(default)]
    pub version: Option<String>,
    /// Extra fields not explicitly defined
    #[serde(default, flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

impl StaticGraphMeta {
    /// Get the display name of the graph
    pub fn display_name(&self) -> String {
        self.name
            .as_ref()
            .map(|n| n.to_string_default())
            .unwrap_or_default()
    }

    /// Get the display subtitle
    pub fn display_subtitle(&self) -> String {
        self.subtitle
            .as_ref()
            .map(|s| s.to_string_default())
            .unwrap_or_default()
    }

    /// Get the display description
    pub fn display_description(&self) -> String {
        self.description
            .as_ref()
            .map(|d| d.to_string_default())
            .unwrap_or_default()
    }

    /// Get the author
    pub fn display_author(&self) -> String {
        self.author.clone().unwrap_or_default()
    }
}

// ============================================================================
// StaticConstraint
// ============================================================================

/// A constraint on a card limiting how nodes can be used
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticConstraint {
    pub card_id: String,
    pub constraintid: String,
    pub nodes: Vec<String>,
    pub uniquetoallinstances: bool,
}

// ============================================================================
// StaticPublication
// ============================================================================

/// Publication information for a graph
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticPublication {
    pub graph_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub publicationid: String,
    pub published_time: String,
}

// ============================================================================
// StaticCardsXNodesXWidgets
// ============================================================================

/// Mapping between cards, nodes, and widgets
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticCardsXNodesXWidgets {
    pub card_id: String,
    #[serde(default)]
    pub config: serde_json::Value,
    pub id: String,
    pub label: StaticTranslatableString,
    pub node_id: String,
    #[serde(default)]
    pub sortorder: Option<i32>,
    pub visible: bool,
    pub widget_id: String,
}

// ============================================================================
// StaticFunctionsXGraphs
// ============================================================================

/// Mapping between functions and graphs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticFunctionsXGraphs {
    #[serde(default)]
    pub config: serde_json::Value,
    pub function_id: String,
    pub graph_id: String,
    pub id: String,
}

// ============================================================================
// StaticCard
// ============================================================================

/// A card defining UI and validation for a nodegroup
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticCard {
    pub active: bool,
    pub cardid: String,
    pub component_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub constraints: Vec<StaticConstraint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cssclass: Option<String>,
    #[serde(default)]
    pub description: Option<StaticTranslatableString>,
    pub graph_id: String,
    pub helpenabled: bool,
    pub helptext: StaticTranslatableString,
    pub helptitle: StaticTranslatableString,
    pub instructions: StaticTranslatableString,
    pub is_editable: bool,
    pub name: StaticTranslatableString,
    pub nodegroup_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortorder: Option<i32>,
    pub visible: bool,
}

// ============================================================================
// StaticTile
// ============================================================================

/// A tile containing data for a nodegroup instance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticTile {
    #[serde(default)]
    pub data: HashMap<String, serde_json::Value>,
    pub nodegroup_id: String,
    pub resourceinstance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tileid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parenttile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisionaledits: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortorder: Option<i32>,
}

impl StaticTile {
    /// Create a new empty tile for a nodegroup
    pub fn new_empty(nodegroup_id: String) -> Self {
        StaticTile {
            tileid: None,
            nodegroup_id,
            parenttile_id: None,
            resourceinstance_id: String::new(),
            sortorder: None,
            provisionaledits: None,
            data: HashMap::new(),
        }
    }

    /// Ensure this tile has an ID, generating one if needed
    pub fn ensure_id(&mut self) -> String {
        if self.tileid.is_none() {
            self.tileid = Some(uuid::Uuid::new_v4().to_string());
        }
        self.tileid.clone().unwrap()
    }
}

// ============================================================================
// StaticResourceDescriptors
// ============================================================================

/// Descriptors for resource display
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StaticResourceDescriptors {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_popup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl StaticResourceDescriptors {
    /// Check if all descriptors are empty
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.map_popup.is_none() && self.description.is_none()
    }

    /// Create empty descriptors
    pub fn empty() -> Self {
        Self::default()
    }
}

// ============================================================================
// Descriptor Configuration
// ============================================================================

/// Descriptor function UUID (from Arches)
pub const DESCRIPTOR_FUNCTION_ID: &str = "60000000-0000-0000-0000-000000000001";

/// Configuration for a single descriptor type (name, description, map_popup)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DescriptorTypeConfig {
    pub nodegroup_id: String,
    pub string_template: String,
}

/// Complete descriptor configuration from functions_x_graphs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DescriptorConfig {
    pub descriptor_types: HashMap<String, DescriptorTypeConfig>,
}

// ============================================================================
// StaticResourceMetadata
// ============================================================================

/// Metadata about a resource instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceMetadata {
    pub descriptors: StaticResourceDescriptors,
    pub graph_id: String,
    pub name: String,
    pub resourceinstanceid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmodified: Option<String>,
}

// ============================================================================
// StaticResourceSummary
// ============================================================================

/// Summary info for a resource (used for lazy loading)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceSummary {
    pub resourceinstanceid: String,
    pub graph_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptors: Option<StaticResourceDescriptors>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmodified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_publication_id: Option<String>,
}

impl StaticResourceSummary {
    /// Convert summary to metadata
    pub fn to_metadata(&self) -> StaticResourceMetadata {
        StaticResourceMetadata {
            descriptors: self.descriptors.clone().unwrap_or_default(),
            graph_id: self.graph_id.clone(),
            name: self.name.clone(),
            resourceinstanceid: self.resourceinstanceid.clone(),
            publication_id: self.publication_id.clone(),
            principaluser_id: self.principaluser_id,
            legacyid: self.legacyid.clone(),
            graph_publication_id: self.graph_publication_id.clone(),
            createdtime: self.createdtime.clone(),
            lastmodified: self.lastmodified.clone(),
        }
    }
}

// ============================================================================
// StaticResourceReference
// ============================================================================

/// Reference to another resource instance (for resource-instance datatype)
///
/// Used in ResourceInstanceViewModel to represent relationships between resources.
/// Can include the full resource tree if cascade is enabled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceReference {
    /// Resource instance ID
    pub id: String,
    /// Graph ID for the resource model
    #[serde(rename = "graphId")]
    pub graph_id: String,
    /// Resource model type/name (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub resource_type: Option<String>,
    /// Display title for the resource (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Full resource tree data (when cascaded, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<serde_json::Value>,
    /// Additional metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl StaticResourceReference {
    /// Create a minimal resource reference with just ID and graph ID
    pub fn new(id: String, graph_id: String) -> Self {
        StaticResourceReference {
            id,
            graph_id,
            resource_type: None,
            title: None,
            root: None,
            meta: None,
        }
    }

    /// Create a reference with type information
    pub fn with_type(id: String, graph_id: String, resource_type: String) -> Self {
        StaticResourceReference {
            id,
            graph_id,
            resource_type: Some(resource_type),
            title: None,
            root: None,
            meta: None,
        }
    }

    /// Add title to reference (builder pattern)
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Add metadata to reference (builder pattern)
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Add root data to reference for cascaded loading (builder pattern)
    pub fn with_root(mut self, root: serde_json::Value) -> Self {
        self.root = Some(root);
        self
    }
}

// ============================================================================
// StaticResource
// ============================================================================

/// Complete resource data with tiles
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResource {
    pub resourceinstance: StaticResourceMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<StaticTile>>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    // Optional cache and scopes - stored as JSON for platform independence
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "__cache")]
    pub cache: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "__scopes")]
    pub scopes: Option<serde_json::Value>,

    // Tracking flag for lazy loading
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tiles_loaded: Option<bool>,
}

// ============================================================================
// Constants
// ============================================================================

/// Datatypes that represent iterable/list values
pub const ITERABLE_DATATYPES: &[&str] = &["concept-list", "resource-instance-list", "domain-value-list"];

/// Check if a datatype is iterable
pub fn is_iterable_datatype(datatype: &str) -> bool {
    ITERABLE_DATATYPES.contains(&datatype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translatable_string_from_map() {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), "Hello".to_string());
        translations.insert("de".to_string(), "Hallo".to_string());

        let ts = StaticTranslatableString::from_translations(translations, None);
        assert_eq!(ts.get("en"), "Hello");
        assert_eq!(ts.get("de"), "Hallo");
    }

    #[test]
    fn test_translatable_string_fallback() {
        let mut translations = HashMap::new();
        translations.insert("de".to_string(), "Hallo".to_string());

        let ts = StaticTranslatableString::from_translations(translations, None);
        // Should fall back to any available language
        assert_eq!(ts.get("en"), "Hallo");
    }

    #[test]
    fn test_node_is_root() {
        let node = StaticNode {
            nodeid: "123".to_string(),
            name: "Root".to_string(),
            alias: None,
            datatype: "semantic".to_string(),
            nodegroup_id: None,
            graph_id: "graph1".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: None,
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: None,
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: false,
            istopnode: true,
            sourcebranchpublication_id: None,
        };
        assert!(node.is_root());
    }
}
