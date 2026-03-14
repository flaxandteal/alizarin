//! StaticGraph and IndexedGraph types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::cards::{StaticCard, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs};
use super::descriptors::{DescriptorConfig, StaticResourceDescriptors, DESCRIPTOR_FUNCTION_ID};
use super::nodes::{StaticEdge, StaticNode, StaticNodegroup};
use super::tile::StaticTile;
use super::translatable::StaticTranslatableString;

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

    // Arches-HER 2.0+ fields (backwards-compatible with older formats)
    /// Source identifier for import/export tracking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_identifier_id: Option<String>,
    /// Whether graph is active
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Whether graph has unpublished changes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_unpublished_changes: Option<bool>,
    /// Whether copy is immutable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_copy_immutable: Option<bool>,
    /// Resource instance lifecycle configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_instance_lifecycle: Option<serde_json::Value>,
    /// Spatial views configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_views: Option<serde_json::Value>,
    /// Group permissions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_permissions: Option<serde_json::Value>,
    /// User permissions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_permissions: Option<serde_json::Value>,

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
        let nodes_by_alias_arc: HashMap<String, Arc<StaticNode>> = self
            .nodes
            .iter()
            .filter_map(|n| {
                n.alias
                    .as_ref()
                    .filter(|a| !a.is_empty())
                    .map(|a| (a.clone(), Arc::new(n.clone())))
            })
            .collect();

        self.node_by_id = Some(node_by_id);
        self.node_by_alias = Some(node_by_alias);
        self.edges_map = Some(edges_map);
        self.nodes_by_nodegroup = Some(nodes_by_nodegroup);
        self.nodegroup_by_id = Some(nodegroup_by_id);
        self.nodes_by_alias_arc = Some(nodes_by_alias_arc);
    }

    /// Invalidate all internal lookup indices.
    ///
    /// This must be called after mutations that modify the nodes vector,
    /// especially operations like `retain()` that shift element positions.
    pub fn invalidate_indices(&mut self) {
        self.node_by_id = None;
        self.node_by_alias = None;
        self.edges_map = None;
        self.nodes_by_nodegroup = None;
        self.nodegroup_by_id = None;
        self.nodes_by_alias_arc = None;
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

    /// Get the model class name for this graph.
    ///
    /// This returns the graph's display name, which is used as the
    /// ResourceInstanceCacheEntry "type" field in TypeScript.
    pub fn get_model_class_name(&self) -> Option<String> {
        let name = self.name.to_string_default();
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
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
        self.nodes_by_alias_arc.as_ref()?.get(alias).cloned()
    }

    // =========================================================================
    // Mutation Methods (for GraphMutator)
    // =========================================================================

    /// Create a deep clone of the graph with fresh indices
    pub fn deep_clone(&self) -> Self {
        let mut cloned = self.clone();
        cloned.build_indices();
        cloned
    }

    /// Push a new node to the graph
    ///
    /// Note: You must call `build_indices()` after all mutations to rebuild lookup tables.
    pub fn push_node(&mut self, node: StaticNode) {
        self.nodes.push(node);
        self.invalidate_indices();
    }

    /// Push a new edge to the graph
    pub fn push_edge(&mut self, edge: StaticEdge) {
        self.edges.push(edge);
        self.edges_map = None;
    }

    /// Push a new nodegroup to the graph
    pub fn push_nodegroup(&mut self, nodegroup: StaticNodegroup) {
        self.nodegroups.push(nodegroup);
        self.nodegroup_by_id = None;
    }

    /// Push a new card to the graph
    pub fn push_card(&mut self, card: StaticCard) {
        if self.cards.is_none() {
            self.cards = Some(Vec::new());
        }
        if let Some(ref mut cards) = self.cards {
            cards.push(card);
        }
    }

    /// Push a new cards_x_nodes_x_widgets entry
    pub fn push_card_x_node_x_widget(&mut self, cxnxw: StaticCardsXNodesXWidgets) {
        if self.cards_x_nodes_x_widgets.is_none() {
            self.cards_x_nodes_x_widgets = Some(Vec::new());
        }
        if let Some(ref mut cxnxw_list) = self.cards_x_nodes_x_widgets {
            cxnxw_list.push(cxnxw);
        }
    }

    /// Get cards slice (for mutation checks)
    pub fn cards_slice(&self) -> &[StaticCard] {
        self.cards.as_deref().unwrap_or(&[])
    }

    /// Get cards_x_nodes_x_widgets slice
    pub fn cards_x_nodes_x_widgets_slice(&self) -> &[StaticCardsXNodesXWidgets] {
        self.cards_x_nodes_x_widgets.as_deref().unwrap_or(&[])
    }

    /// Find a card by nodegroup_id
    pub fn find_card_by_nodegroup(&self, nodegroup_id: &str) -> Option<&StaticCard> {
        self.cards
            .as_ref()?
            .iter()
            .find(|c| c.nodegroup_id == nodegroup_id)
    }

    /// Find a node by alias (without requiring indices to be built)
    pub fn find_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        // Try indexed lookup first
        if let Some(node) = self.get_node_by_alias(alias) {
            return Some(node);
        }
        // Fall back to linear search
        self.nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some(alias))
    }

    /// Get a simplified schema view of the graph showing node aliases and structure.
    ///
    /// Returns a nested structure representing the tree with:
    /// - Keys are node aliases (or nodeid if no alias)
    /// - Values contain 'datatype', 'nodeid', optionally 'required', and 'children'
    ///
    /// Useful for understanding what keys are available in tree output.
    pub fn get_schema(&self) -> serde_json::Value {
        // Build a map from nodeid to node for quick lookup
        let node_map: HashMap<&str, &StaticNode> =
            self.nodes.iter().map(|n| (n.nodeid.as_str(), n)).collect();

        // Build parent -> children map based on edges
        let mut children_map: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            children_map
                .entry(edge.domainnode_id.as_str())
                .or_default()
                .push(&edge.rangenode_id);
        }

        // Recursive function to build node schema
        fn build_node_schema(
            nodeid: &str,
            node_map: &HashMap<&str, &StaticNode>,
            children_map: &HashMap<&str, Vec<&str>>,
        ) -> serde_json::Value {
            let node = match node_map.get(nodeid) {
                Some(n) => n,
                None => return serde_json::json!({}),
            };

            let mut schema = serde_json::json!({
                "datatype": node.datatype,
                "nodeid": node.nodeid,
            });

            if node.isrequired {
                schema["required"] = serde_json::json!(true);
            }

            // Add children recursively
            if let Some(child_ids) = children_map.get(nodeid) {
                let mut children = serde_json::Map::new();
                for child_id in child_ids {
                    if let Some(child_node) = node_map.get(child_id) {
                        let key = child_node.alias.as_deref().unwrap_or(&child_node.nodeid);
                        children.insert(
                            key.to_string(),
                            build_node_schema(child_id, node_map, children_map),
                        );
                    }
                }
                if !children.is_empty() {
                    schema["children"] = serde_json::Value::Object(children);
                }
            }

            schema
        }

        // Start from root node and build tree
        let root_id = &self.root.nodeid;
        let mut root_schema = serde_json::Map::new();

        if let Some(child_ids) = children_map.get(root_id.as_str()) {
            for child_id in child_ids {
                if let Some(child_node) = node_map.get(child_id) {
                    let key = child_node.alias.as_deref().unwrap_or(&child_node.nodeid);
                    root_schema.insert(
                        key.to_string(),
                        build_node_schema(child_id, &node_map, &children_map),
                    );
                }
            }
        }

        serde_json::Value::Object(root_schema)
    }

    /// Set a descriptor template for a given descriptor type (e.g. "slug", "name").
    ///
    /// The `nodegroup_id` is inferred from the `<Node Name>` placeholders in the
    /// template by looking up nodes in the graph. All placeholder nodes must belong
    /// to exactly one nodegroup — returns an error otherwise.
    ///
    /// Creates or updates the descriptor function entry in `functions_x_graphs`.
    pub fn set_descriptor_template(
        &mut self,
        descriptor_type: &str,
        string_template: &str,
    ) -> Result<(), String> {
        use crate::graph::descriptors::DESCRIPTOR_FUNCTION_ID;
        use crate::graph::StaticFunctionsXGraphs;
        use std::collections::HashSet;

        // Extract placeholders and resolve nodegroup_id
        let placeholders = IndexedGraph::extract_placeholders(string_template);
        if placeholders.is_empty() {
            return Err(format!(
                "Template '{}' has no <Node Name> placeholders",
                string_template
            ));
        }

        let mut nodegroup_ids = HashSet::new();
        for placeholder in &placeholders {
            let node_name = placeholder.trim_start_matches('<').trim_end_matches('>');
            let node = self
                .nodes
                .iter()
                .find(|n| n.name == node_name)
                .ok_or_else(|| format!("Node '{}' from template not found in graph", node_name))?;
            let ng_id = node
                .nodegroup_id
                .as_ref()
                .ok_or_else(|| format!("Node '{}' has no nodegroup_id", node_name))?;
            nodegroup_ids.insert(ng_id.clone());
        }

        if nodegroup_ids.len() != 1 {
            return Err(format!(
                "Template placeholders span {} nodegroups ({:?}), expected exactly 1",
                nodegroup_ids.len(),
                nodegroup_ids
            ));
        }

        let nodegroup_id = nodegroup_ids.into_iter().next().unwrap();

        let fxg = self.functions_x_graphs.get_or_insert_with(Vec::new);

        // Find existing descriptor function entry
        let existing = fxg
            .iter_mut()
            .find(|f| f.function_id == DESCRIPTOR_FUNCTION_ID);

        let entry = serde_json::json!({
            "nodegroup_id": nodegroup_id,
            "string_template": string_template,
        });

        if let Some(func) = existing {
            if !func.config.is_object() {
                func.config = serde_json::json!({"descriptor_types": {}});
            }
            let config = func.config.as_object_mut().unwrap();
            let dt = config
                .entry("descriptor_types")
                .or_insert_with(|| serde_json::json!({}));
            if let Some(dt_obj) = dt.as_object_mut() {
                dt_obj.insert(descriptor_type.to_string(), entry);
            }
        } else {
            let mut dt_map = serde_json::Map::new();
            dt_map.insert(descriptor_type.to_string(), entry);
            fxg.push(StaticFunctionsXGraphs {
                config: serde_json::json!({ "descriptor_types": dt_map }),
                function_id: DESCRIPTOR_FUNCTION_ID.to_string(),
                graph_id: self.graphid.clone(),
                id: uuid::Uuid::new_v4().to_string(),
            });
        }

        Ok(())
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
        self.build_descriptors_with_diagnostics(tiles, &mut Vec::new())
    }

    /// Build descriptors with diagnostic warnings for silent failure cases
    pub fn build_descriptors_with_diagnostics(
        &self,
        tiles: &[StaticTile],
        warnings: &mut Vec<String>,
    ) -> StaticResourceDescriptors {
        // Get descriptor config from graph with diagnostics
        let config = match self.get_descriptor_config_with_diagnostics(warnings) {
            Some(c) => c,
            None => {
                // Warning already added by get_descriptor_config_with_diagnostics
                return StaticResourceDescriptors::default();
            }
        };

        let mut descriptors = StaticResourceDescriptors::default();

        // Process each descriptor type (name, description, map_popup)
        for (descriptor_type, type_config) in &config.descriptor_types {
            let mut template = type_config.string_template.clone();

            // Extract placeholders from template (e.g., <Node Name>)
            let placeholders = Self::extract_placeholders(&template);
            if placeholders.is_empty() {
                warnings.push(format!(
                    "Descriptor '{}': No placeholders found in template '{}'",
                    descriptor_type, template
                ));
                continue;
            }

            // Find tiles for this nodegroup
            let relevant_tiles: Vec<&StaticTile> = tiles
                .iter()
                .filter(|t| t.nodegroup_id == type_config.nodegroup_id)
                .collect();

            if relevant_tiles.is_empty() {
                let tile_nodegroups: Vec<_> = tiles.iter().map(|t| &t.nodegroup_id).collect();
                warnings.push(format!(
                    "Descriptor '{}': No tiles match nodegroup_id '{}'. Available tile nodegroups: {:?}",
                    descriptor_type, type_config.nodegroup_id, tile_nodegroups
                ));
                continue;
            }

            // Replace each placeholder with actual value from tiles
            for placeholder in &placeholders {
                // Remove < > from placeholder to get node name
                let node_name = placeholder.trim_start_matches('<').trim_end_matches('>');

                // Find node by name in this nodegroup
                if let Some(node) =
                    self.find_node_by_name_in_nodegroup(node_name, &type_config.nodegroup_id)
                {
                    // Extract value from tiles using the type serialization system
                    if let Some(value) = Self::extract_display_value_from_tiles(
                        &relevant_tiles,
                        &node.nodeid,
                        &node.datatype,
                    ) {
                        template = template.replace(placeholder, &value);
                    } else {
                        let available_keys: Vec<_> =
                            relevant_tiles.iter().flat_map(|t| t.data.keys()).collect();
                        warnings.push(format!(
                            "Descriptor '{}': No value found for node '{}' (nodeid '{}') in tiles. Available data keys: {:?}",
                            descriptor_type, node_name, node.nodeid, available_keys
                        ));
                    }
                } else {
                    let nodes_in_nodegroup: Vec<_> = self
                        .nodes_by_id
                        .values()
                        .filter(|n| {
                            n.nodegroup_id
                                .as_ref()
                                .map(|id| id == &type_config.nodegroup_id)
                                .unwrap_or(false)
                        })
                        .map(|n| &n.name)
                        .collect();
                    warnings.push(format!(
                        "Descriptor '{}': Node '{}' not found in nodegroup '{}'. Available nodes: {:?}",
                        descriptor_type, node_name, type_config.nodegroup_id, nodes_in_nodegroup
                    ));
                }
            }

            // Assign to appropriate descriptor field
            match descriptor_type.as_str() {
                "name" => descriptors.name = Some(template),
                "description" => descriptors.description = Some(template),
                "map_popup" => descriptors.map_popup = Some(template),
                "slug" => {
                    descriptors.slug =
                        Some(crate::graph_mutator::slugify(&template).replace('_', "-"))
                }
                _ => {} // Unknown descriptor type, ignore
            }
        }

        descriptors
    }

    /// Extract descriptor config with diagnostic warnings
    fn get_descriptor_config_with_diagnostics(
        &self,
        warnings: &mut Vec<String>,
    ) -> Option<DescriptorConfig> {
        let functions_x_graphs = match self.graph.functions_x_graphs.as_ref() {
            Some(fxg) => fxg,
            None => {
                warnings.push("Graph has no functions_x_graphs array".to_string());
                return None;
            }
        };

        for func in functions_x_graphs {
            if func.function_id == DESCRIPTOR_FUNCTION_ID {
                // Parse config as DescriptorConfig
                match serde_json::from_value::<DescriptorConfig>(func.config.clone()) {
                    Ok(config) => return Some(config),
                    Err(e) => {
                        warnings.push(format!(
                            "Failed to parse descriptor config: {}. Raw config: {}",
                            e,
                            serde_json::to_string(&func.config).unwrap_or_default()
                        ));
                        return None;
                    }
                }
            }
        }

        warnings.push(format!(
            "No descriptor function found in functions_x_graphs (looking for function_id {}). Available function_ids: {:?}",
            DESCRIPTOR_FUNCTION_ID,
            functions_x_graphs.iter().map(|f| &f.function_id).collect::<Vec<_>>()
        ));
        None
    }

    /// Extract placeholders from template using regex pattern
    /// Finds patterns like <Node Name> in the template string
    pub fn extract_placeholders(template: &str) -> Vec<String> {
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
    fn find_node_by_name_in_nodegroup(
        &self,
        name: &str,
        nodegroup_id: &str,
    ) -> Option<&StaticNode> {
        self.nodes_by_id.values().find(|node| {
            node.name == name
                && node
                    .nodegroup_id
                    .as_ref()
                    .map(|id| id == nodegroup_id)
                    .unwrap_or(false)
        })
    }

    /// Extract a display string from tiles for a given node, using:
    /// 1. Extension render_display (for extension datatypes like "reference")
    /// 2. Built-in serialize_display (for string, number, date, concept, etc.)
    /// 3. Fallback to extract_string_from_json (language maps, Arches format)
    fn extract_display_value_from_tiles(
        tiles: &[&StaticTile],
        node_id: &str,
        datatype: &str,
    ) -> Option<String> {
        for tile in tiles {
            if let Some(value) = tile.data.get(node_id) {
                // 1. Try extension render_display (optional — not all extensions have it)
                if let Ok(Some(display)) =
                    crate::registry::render_extension_display(datatype, value, "en")
                {
                    return Some(display);
                }

                // 2. Try built-in type serialization
                let result = crate::type_serialization::serialize_display(datatype, value, "en");
                if !result.is_error() {
                    match &result.value {
                        serde_json::Value::String(s) if !s.is_empty() => return Some(s.clone()),
                        serde_json::Value::Number(n) => return Some(n.to_string()),
                        serde_json::Value::Bool(b) => return Some(b.to_string()),
                        _ => {}
                    }
                }

                // 3. Fallback to raw JSON extraction (language maps, Arches format)
                if let Some(extracted) = Self::extract_string_from_json(value) {
                    return Some(extracted);
                }
            }
        }
        None
    }

    /// Extract string value from JSON, handling language-nested objects
    /// Handles:
    /// - Simple strings: "value"
    /// - Language objects: {"en": "value"}
    /// - Arches localized strings: {"en": {"direction": "ltr", "value": "actual value"}}
    fn extract_string_from_json(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            serde_json::Value::Object(map) => {
                // Try to get language-specific value
                Self::extract_lang_value(map, "en").or_else(|| {
                    // Fallback to first available language
                    map.values().find_map(Self::extract_single_lang_value)
                })
            }
            _ => None,
        }
    }

    /// Extract value for a specific language key
    fn extract_lang_value(
        map: &serde_json::Map<String, serde_json::Value>,
        lang: &str,
    ) -> Option<String> {
        map.get(lang).and_then(Self::extract_single_lang_value)
    }

    /// Extract string from a single language value, handling both formats:
    /// - Direct string: "value"
    /// - Arches format: {"direction": "ltr", "value": "actual value"}
    fn extract_single_lang_value(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Object(obj) => {
                // Arches localized string format: {"direction": "...", "value": "..."}
                obj.get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }
            _ => None,
        }
    }
}
