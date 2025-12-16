//! StaticGraph and IndexedGraph types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::translatable::StaticTranslatableString;
use super::nodes::{StaticNode, StaticNodegroup, StaticEdge};
use super::tile::StaticTile;
use super::cards::{StaticCard, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs};
use super::descriptors::{StaticResourceDescriptors, DescriptorConfig, DESCRIPTOR_FUNCTION_ID};

/// Wrapper for loading from JSON files with {graph: [...]} structure
#[derive(Debug, Deserialize)]
pub struct GraphWrapper {
    pub graph: Vec<StaticGraph>,
}

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
    #[serde(skip)]
    edges_map: Option<HashMap<String, Vec<String>>>,
    #[serde(skip)]
    nodes_by_nodegroup: Option<HashMap<String, Vec<usize>>>,
    #[serde(skip)]
    nodegroup_by_id: Option<HashMap<String, usize>>,
    // Arc-wrapped node caches for pseudo_value infrastructure (avoids cloning on every conversion)
    #[serde(skip)]
    nodes_by_alias_arc: Option<HashMap<String, Arc<StaticNode>>>,
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
        let mut nodes_by_nodegroup: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, node) in self.nodes.iter().enumerate() {
            node_by_id.insert(node.nodeid.clone(), idx);
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    node_by_alias.insert(alias.clone(), idx);
                }
            }
            if let Some(ref ng_id) = node.nodegroup_id {
                if !ng_id.is_empty() {
                    nodes_by_nodegroup
                        .entry(ng_id.clone())
                        .or_default()
                        .push(idx);
                }
            }
        }

        // Build edges map (parent_nodeid -> child_nodeids)
        let mut edges_map: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            edges_map
                .entry(edge.domainnode_id.clone())
                .or_default()
                .push(edge.rangenode_id.clone());
        }

        // Build nodegroup index
        let mut nodegroup_by_id = HashMap::new();
        for (idx, ng) in self.nodegroups.iter().enumerate() {
            nodegroup_by_id.insert(ng.nodegroupid.clone(), idx);
        }

        // Build Arc-wrapped nodes_by_alias for pseudo_value infrastructure
        let nodes_by_alias_arc: HashMap<String, Arc<StaticNode>> = self.nodes
            .iter()
            .filter_map(|n| n.alias.as_ref().filter(|a| !a.is_empty()).map(|a| (a.clone(), Arc::new(n.clone()))))
            .collect();

        self.node_by_id = Some(node_by_id);
        self.node_by_alias = Some(node_by_alias);
        self.edges_map = Some(edges_map);
        self.nodes_by_nodegroup = Some(nodes_by_nodegroup);
        self.nodegroup_by_id = Some(nodegroup_by_id);
        self.nodes_by_alias_arc = Some(nodes_by_alias_arc);
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

    /// Get edges map (parent_nodeid -> child_nodeids)
    /// Returns None if indices haven't been built
    pub fn edges_map(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.edges_map.as_ref()
    }

    /// Get child node IDs for a given node
    pub fn get_child_ids(&self, node_id: &str) -> Option<&Vec<String>> {
        self.edges_map.as_ref()?.get(node_id)
    }

    /// Get nodes by nodegroup (nodegroup_id -> node indices)
    /// Returns None if indices haven't been built
    pub fn nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<usize>>> {
        self.nodes_by_nodegroup.as_ref()
    }

    /// Get nodes in a specific nodegroup
    pub fn get_nodes_in_nodegroup(&self, nodegroup_id: &str) -> Vec<&StaticNode> {
        self.nodes_by_nodegroup
            .as_ref()
            .and_then(|map| map.get(nodegroup_id))
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.nodes.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get nodegroup by ID
    pub fn get_nodegroup_by_id(&self, nodegroup_id: &str) -> Option<&StaticNodegroup> {
        self.nodegroup_by_id
            .as_ref()?
            .get(nodegroup_id)
            .and_then(|&idx| self.nodegroups.get(idx))
    }

    /// Get Arc-wrapped nodes by alias map (for pseudo_value infrastructure)
    /// Returns None if indices haven't been built
    pub fn nodes_by_alias_arc(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        self.nodes_by_alias_arc.as_ref()
    }

    /// Get Arc-wrapped node by alias
    pub fn get_node_arc_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>> {
        self.nodes_by_alias_arc
            .as_ref()?
            .get(alias)
            .cloned()
    }
}

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
