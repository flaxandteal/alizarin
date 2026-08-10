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
    /// Ontology IDs used by this graph. Accepts a single string or an array
    /// of strings on the wire; a single-element list is serialised as a plain
    /// string for round-trip compatibility with upstream Arches.
    #[serde(default, with = "super::serde_helpers::optional_string_or_vec")]
    pub ontology_id: Option<Vec<String>>,
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

    /// Internal lookup tables. Built lazily on first use, never serialized.
    #[serde(skip)]
    indices: std::sync::OnceLock<Indices>,
}

#[derive(Clone, Debug, Default)]
struct Indices {
    node_by_id: HashMap<String, usize>,
    node_by_alias: HashMap<String, usize>,
    edges_map: HashMap<String, Vec<String>>,
    nodes_by_nodegroup: HashMap<String, Vec<usize>>,
    nodegroup_by_id: HashMap<String, usize>,
    nodes_by_alias_arc: HashMap<String, Arc<StaticNode>>,
    card_index: Option<super::card_index::CardIndex>,
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

    fn indices(&self) -> &Indices {
        self.indices.get_or_init(|| self.compute_indices())
    }

    pub fn build_indices(&mut self) {
        self.invalidate_indices();
    }

    fn compute_indices(&self) -> Indices {
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

        let mut edges_map: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            edges_map
                .entry(edge.domainnode_id.clone())
                .or_default()
                .push(edge.rangenode_id.clone());
        }

        let mut nodegroup_by_id = HashMap::new();
        for (idx, ng) in self.nodegroups.iter().enumerate() {
            nodegroup_by_id.insert(ng.nodegroupid.clone(), idx);
        }

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

        let card_index = self.cards.is_some().then(|| {
            super::card_index::build_card_index(
                self.cards_slice(),
                self.cards_x_nodes_x_widgets_slice(),
                &self.nodegroups,
                &self.nodes,
                crate::graph_mutator::get_widget_name_by_id,
            )
        });

        Indices {
            node_by_id,
            node_by_alias,
            edges_map,
            nodes_by_nodegroup,
            nodegroup_by_id,
            nodes_by_alias_arc,
            card_index,
        }
    }

    pub fn invalidate_indices(&mut self) {
        self.indices = std::sync::OnceLock::new();
    }

    /// Get the root node
    pub fn get_root(&self) -> &StaticNode {
        &self.root
    }

    /// Get node by index
    pub fn get_node_by_index(&self, idx: usize) -> Option<&StaticNode> {
        self.nodes.get(idx)
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<&StaticNode> {
        self.indices()
            .node_by_id
            .get(id)
            .and_then(|&idx| self.nodes.get(idx))
    }

    pub fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        self.indices()
            .node_by_alias
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

    pub fn edges_map(&self) -> Option<&HashMap<String, Vec<String>>> {
        Some(&self.indices().edges_map)
    }

    pub fn get_child_ids(&self, node_id: &str) -> Option<&Vec<String>> {
        self.indices().edges_map.get(node_id)
    }

    pub fn nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<usize>>> {
        Some(&self.indices().nodes_by_nodegroup)
    }

    pub fn get_nodes_in_nodegroup(&self, nodegroup_id: &str) -> Vec<&StaticNode> {
        self.indices()
            .nodes_by_nodegroup
            .get(nodegroup_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.nodes.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_nodegroup_by_id(&self, nodegroup_id: &str) -> Option<&StaticNodegroup> {
        self.indices()
            .nodegroup_by_id
            .get(nodegroup_id)
            .and_then(|&idx| self.nodegroups.get(idx))
    }

    pub fn nodes_by_alias_arc(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        Some(&self.indices().nodes_by_alias_arc)
    }

    pub fn get_node_arc_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>> {
        self.indices().nodes_by_alias_arc.get(alias).cloned()
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
        self.invalidate_indices();
    }

    /// Push a new nodegroup to the graph
    pub fn push_nodegroup(&mut self, nodegroup: StaticNodegroup) {
        self.nodegroups.push(nodegroup);
        self.invalidate_indices();
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

    pub fn card_index(&self) -> Option<&super::card_index::CardIndex> {
        self.indices().card_index.as_ref()
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

        let fxg = self.functions_x_graphs.get_or_insert_with(Vec::new);

        // Find existing descriptor function entry:
        // 1. Non-default function with empty/null config (just added via add_function,
        //    intended to replace the default descriptor — e.g. multicard descriptor)
        // 2. Non-default function that already has descriptor_types config
        // 3. Exact match on the default descriptor function ID
        let idx = fxg
            .iter()
            .position(|f| {
                f.function_id != DESCRIPTOR_FUNCTION_ID
                    && (f.config.is_null() || f.config == serde_json::json!({}))
            })
            .or_else(|| {
                fxg.iter().position(|f| {
                    f.function_id != DESCRIPTOR_FUNCTION_ID
                        && f.config
                            .as_object()
                            .is_some_and(|c| c.contains_key("descriptor_types"))
                })
            })
            .or_else(|| {
                fxg.iter()
                    .position(|f| f.function_id == DESCRIPTOR_FUNCTION_ID)
            });

        let is_default_function = idx
            .map(|i| fxg[i].function_id == DESCRIPTOR_FUNCTION_ID)
            .unwrap_or(true); // will create default if no match

        // Non-default functions (e.g. multicard descriptor) resolve templates
        // at runtime using node aliases — alizarin stores the template verbatim
        // with empty nodegroup_id. Default function requires name-based resolution
        // to a single nodegroup at build time.
        let entry = if is_default_function {
            let placeholders = Self::extract_placeholders(string_template);
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
                    .ok_or_else(|| {
                        format!("Node '{}' from template not found in graph", node_name)
                    })?;
                let ng_id = node
                    .nodegroup_id
                    .as_ref()
                    .ok_or_else(|| format!("Node '{}' has no nodegroup_id", node_name))?;
                nodegroup_ids.insert(ng_id.clone());
            }

            if nodegroup_ids.len() != 1 && descriptor_type != "slug" {
                return Err(format!(
                    "Template placeholders span {} nodegroups ({:?}), expected exactly 1",
                    nodegroup_ids.len(),
                    nodegroup_ids
                ));
            }

            let nodegroup_id = nodegroup_ids.into_iter().next().unwrap();
            serde_json::json!({
                "nodegroup_id": nodegroup_id,
                "string_template": string_template,
            })
        } else {
            // Non-default: store template as-is with empty nodegroup_id.
            // The function resolves aliases at runtime.
            serde_json::json!({
                "nodegroup_id": "",
                "string_template": string_template,
            })
        };

        let existing = idx.map(|i| &mut fxg[i]);

        if let Some(func) = existing {
            let needs_seed = !func.config.is_object()
                || !func
                    .config
                    .as_object()
                    .is_some_and(|c| c.contains_key("descriptor_types"));
            if needs_seed && !is_default_function {
                // Seed all descriptor types with empty defaults for non-default functions
                func.config = serde_json::json!({
                    "descriptor_types": {
                        "name": {"nodegroup_id": "", "string_template": ""},
                        "description": {"nodegroup_id": "", "string_template": ""},
                        "map_popup": {"nodegroup_id": "", "string_template": ""},
                    }
                });
            } else if !func.config.is_object() {
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
                id: crate::graph_mutator::generate_uuid_v5(
                    ("function", Some(&self.graphid)),
                    DESCRIPTOR_FUNCTION_ID,
                ),
            });
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Descriptor building on StaticGraph, so it needs no graph clone.
//
// A resource's descriptors (name/description/map_popup/slug) are the descriptor
// template rendered against its tiles. This needs only a name->node lookup (a
// linear scan) and the graph's descriptor config, NEITHER of which needs a
// precomputed index. Running it on `&StaticGraph` directly allocates nothing;
// `IndexedGraph` keeps thin delegators (below) for its own callers.
// ---------------------------------------------------------------------------
impl StaticGraph {
    /// Build resource descriptors (name/description/map_popup/slug) from tiles.
    pub fn build_descriptors(&self, tiles: &[StaticTile]) -> StaticResourceDescriptors {
        self.build_descriptors_with_context(tiles, &mut Vec::new(), None, None)
    }

    /// Build descriptors, collecting diagnostic warnings for silent-failure cases.
    pub fn build_descriptors_with_diagnostics(
        &self,
        tiles: &[StaticTile],
        warnings: &mut Vec<String>,
        cache: Option<&super::resources::ResourceCache>,
    ) -> StaticResourceDescriptors {
        self.build_descriptors_with_context(tiles, warnings, cache, None)
    }

    /// Build descriptors with full context (diagnostics, cache, extension registry).
    pub fn build_descriptors_with_context(
        &self,
        tiles: &[StaticTile],
        warnings: &mut Vec<String>,
        cache: Option<&super::resources::ResourceCache>,
        extension_registry: Option<&crate::extension_type_registry::ExtensionTypeRegistry>,
    ) -> StaticResourceDescriptors {
        let config = match self.get_descriptor_config_with_diagnostics(warnings) {
            Some(c) => c,
            None => {
                return StaticResourceDescriptors::default();
            }
        };

        let mut descriptors = StaticResourceDescriptors::default();

        for (descriptor_type, type_config) in &config.descriptor_types {
            let mut template = type_config.string_template.clone();

            let placeholders = Self::extract_placeholders(&template);
            if placeholders.is_empty() {
                warnings.push(format!(
                    "Descriptor '{}': No placeholders found in template '{}'",
                    descriptor_type, template
                ));
                continue;
            }

            for placeholder in &placeholders {
                let node_name = placeholder.trim_start_matches('<').trim_end_matches('>');

                if let Some(node) = self.find_node_by_name(node_name) {
                    let node_ng = match node.nodegroup_id.as_ref() {
                        Some(ng) => ng,
                        None => {
                            warnings.push(format!(
                                "Descriptor '{}': Node '{}' has no nodegroup_id",
                                descriptor_type, node_name
                            ));
                            continue;
                        }
                    };
                    let node_tiles: Vec<&StaticTile> = tiles
                        .iter()
                        .filter(|t| &t.nodegroup_id == node_ng)
                        .collect();
                    if let Some(value) = Self::extract_display_value_from_tiles(
                        &node_tiles,
                        &node.nodeid,
                        &node.datatype,
                        cache,
                        extension_registry,
                    ) {
                        template = template.replace(placeholder, &value);
                    } else {
                        let available_keys: Vec<_> =
                            node_tiles.iter().flat_map(|t| t.data.keys()).collect();
                        warnings.push(format!(
                            "Descriptor '{}': No value found for node '{}' (nodeid '{}') in tiles. Available data keys: {:?}",
                            descriptor_type, node_name, node.nodeid, available_keys
                        ));
                    }
                } else {
                    warnings.push(format!(
                        "Descriptor '{}': Node '{}' not found in graph",
                        descriptor_type, node_name
                    ));
                }
            }

            match descriptor_type.as_str() {
                "name" => descriptors.name = Some(template),
                "description" => descriptors.description = Some(template),
                "map_popup" => descriptors.map_popup = Some(template),
                "slug" => {
                    descriptors.slug = Some(if template.contains('<') {
                        template.clone()
                    } else {
                        crate::graph_mutator::slugify(&template).replace('_', "-")
                    })
                }
                _ => {}
            }
        }

        descriptors
    }

    fn get_descriptor_config_with_diagnostics(
        &self,
        warnings: &mut Vec<String>,
    ) -> Option<DescriptorConfig> {
        let functions_x_graphs = match self.functions_x_graphs.as_ref() {
            Some(fxg) => fxg,
            None => {
                warnings.push("Graph has no functions_x_graphs array".to_string());
                return None;
            }
        };

        let descriptor_func = functions_x_graphs
            .iter()
            .find(|f| f.function_id == DESCRIPTOR_FUNCTION_ID)
            .or_else(|| {
                functions_x_graphs.iter().find(|f| {
                    f.config
                        .as_object()
                        .is_some_and(|c| c.contains_key("descriptor_types"))
                })
            });

        if let Some(func) = descriptor_func {
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

        warnings.push(format!(
            "No descriptor function found in functions_x_graphs (looking for function_id {} or descriptor_types config). Available function_ids: {:?}",
            DESCRIPTOR_FUNCTION_ID,
            functions_x_graphs.iter().map(|f| &f.function_id).collect::<Vec<_>>()
        ));
        None
    }

    /// Extract placeholders like `<Node Name>` from a descriptor template.
    pub fn extract_placeholders(template: &str) -> Vec<String> {
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

    /// Find a node by NAME (linear scan over all nodes, since placeholders
    /// match on node name and there is no name index).
    fn find_node_by_name(&self, name: &str) -> Option<&StaticNode> {
        self.nodes.iter().find(|node| node.name == name)
    }

    /// Extract a display string from tiles for a given node, using cache lookup,
    /// extension render_display, built-in serialize_display, then a raw-JSON fallback.
    fn extract_display_value_from_tiles(
        tiles: &[&StaticTile],
        node_id: &str,
        datatype: &str,
        cache: Option<&super::resources::ResourceCache>,
        extension_registry: Option<&crate::extension_type_registry::ExtensionTypeRegistry>,
    ) -> Option<String> {
        if let Some(cache) = cache {
            if datatype == "resource-instance" || datatype == "resource-instance-list" {
                for tile in tiles {
                    if let Some(tile_id) = &tile.tileid {
                        if let Some(node_entries) = cache.get(tile_id) {
                            if let Some(entry) = node_entries.get(node_id) {
                                return Self::display_from_cache_entry(entry);
                            }
                        }
                    }
                }
            }
        }

        for tile in tiles {
            if let Some(value) = tile.data.get(node_id) {
                if let Some(registry) = extension_registry {
                    if let Ok(Some(display)) = registry.render_display(datatype, value, "en") {
                        return Some(display);
                    }
                }

                let result = crate::type_serialization::serialize_display(datatype, value, "en");
                if !result.is_error() {
                    match &result.value {
                        serde_json::Value::String(s) if !s.is_empty() => return Some(s.clone()),
                        serde_json::Value::Number(n) => return Some(n.to_string()),
                        serde_json::Value::Bool(b) => return Some(b.to_string()),
                        _ => {}
                    }
                }

                if let Some(extracted) = Self::extract_string_from_json(value) {
                    return Some(extracted);
                }
            }
        }
        None
    }

    fn display_from_cache_entry(entry: &super::resources::CacheEntry) -> Option<String> {
        match entry {
            super::resources::CacheEntry::Single(r) => r.title.clone(),
            super::resources::CacheEntry::List(list) => {
                let titles: Vec<&str> = list
                    .entries
                    .iter()
                    .filter_map(|e| e.title.as_deref())
                    .collect();
                if titles.is_empty() {
                    None
                } else {
                    Some(titles.join(", "))
                }
            }
        }
    }

    /// Extract string value from JSON, handling language-nested objects.
    fn extract_string_from_json(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            serde_json::Value::Object(map) => Self::extract_lang_value(map, "en")
                .or_else(|| map.values().find_map(Self::extract_single_lang_value)),
            _ => None,
        }
    }

    fn extract_lang_value(
        map: &serde_json::Map<String, serde_json::Value>,
        lang: &str,
    ) -> Option<String> {
        map.get(lang).and_then(Self::extract_single_lang_value)
    }

    /// Extract string from a single language value (direct string or Arches
    /// `{"direction":..,"value":..}` format).
    fn extract_single_lang_value(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Object(obj) => obj
                .get("value")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        }
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

        for node in &graph.nodes {
            nodes_by_id.insert(node.nodeid.clone(), node.clone());
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    nodes_by_alias.insert(alias.clone(), node.clone());
                }
            }
        }

        for edge in &graph.edges {
            children_by_node
                .entry(edge.domainnode_id.clone())
                .or_default()
                .push(edge.rangenode_id.clone());
        }

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

    /// Delegates to [`StaticGraph::build_descriptors`].
    pub fn build_descriptors(&self, tiles: &[StaticTile]) -> StaticResourceDescriptors {
        self.graph.build_descriptors(tiles)
    }

    /// Delegates to [`StaticGraph::build_descriptors_with_diagnostics`].
    pub fn build_descriptors_with_diagnostics(
        &self,
        tiles: &[StaticTile],
        warnings: &mut Vec<String>,
        cache: Option<&super::resources::ResourceCache>,
    ) -> StaticResourceDescriptors {
        self.graph
            .build_descriptors_with_diagnostics(tiles, warnings, cache)
    }

    /// Delegates to [`StaticGraph::build_descriptors_with_context`].
    pub fn build_descriptors_with_context(
        &self,
        tiles: &[StaticTile],
        warnings: &mut Vec<String>,
        cache: Option<&super::resources::ResourceCache>,
        extension_registry: Option<&crate::extension_type_registry::ExtensionTypeRegistry>,
    ) -> StaticResourceDescriptors {
        self.graph
            .build_descriptors_with_context(tiles, warnings, cache, extension_registry)
    }
}

#[cfg(test)]
mod lazy_index_tests {
    use super::*;

    fn deserialized_graph() -> StaticGraph {
        serde_json::from_value(serde_json::json!({
            "graphid": "g",
            "name": {"en": "G"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
            "nodes": [
                {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
                {"nodeid": "n1", "name": "N1", "alias": "n1", "datatype": "string",
                 "nodegroup_id": "ng1", "graph_id": "g"}
            ],
            "nodegroups": [{"nodegroupid": "ng1", "cardinality": "1"}],
            "edges": [{"edgeid": "e1", "domainnode_id": "root", "rangenode_id": "n1",
                       "graph_id": "g"}]
        }))
        .expect("graph")
    }

    #[test]
    fn a_graph_that_was_never_explicitly_indexed_answers_every_lookup() {
        let graph = deserialized_graph();

        assert!(graph.get_node_by_id("n1").is_some());
        assert!(graph.get_node_by_alias("n1").is_some());
        assert!(graph.get_nodegroup_by_id("ng1").is_some());
        assert!(graph.get_node_arc_by_alias("n1").is_some());
        assert_eq!(graph.get_child_ids("root"), Some(&vec!["n1".to_string()]));
        assert_eq!(graph.get_nodes_in_nodegroup("ng1").len(), 1);
        assert!(graph.edges_map().is_some());
        assert!(graph.nodes_by_nodegroup().is_some());
    }

    #[test]
    fn static_graph_remains_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StaticGraph>();
    }

    #[test]
    fn concurrent_first_lookups_agree() {
        let graph = std::sync::Arc::new(deserialized_graph());
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let g = std::sync::Arc::clone(&graph);
                std::thread::spawn(move || g.get_nodegroup_by_id("ng1").is_some())
            })
            .collect();
        for h in handles {
            assert!(
                h.join().expect("thread"),
                "every racing reader must see the index"
            );
        }
    }

    #[test]
    fn a_lookup_that_misses_is_a_real_absence() {
        let graph = deserialized_graph();
        assert!(graph.get_node_by_id("nope").is_none());
        assert!(graph.get_nodegroup_by_id("nope").is_none());
    }

    #[test]
    fn a_mutation_is_visible_through_the_index() {
        let mut graph = deserialized_graph();
        assert!(graph.get_nodegroup_by_id("ng1").is_some());
        assert!(graph.get_nodegroup_by_id("ng2").is_none());

        graph.push_nodegroup(
            serde_json::from_value(serde_json::json!({
                "nodegroupid": "ng2", "cardinality": "n"
            }))
            .expect("nodegroup"),
        );

        assert!(
            graph.get_nodegroup_by_id("ng2").is_some(),
            "the pushed nodegroup must be visible: push_nodegroup invalidates"
        );
        assert!(
            graph.get_nodegroup_by_id("ng1").is_some(),
            "and the old one survives"
        );
    }

    #[test]
    fn removing_a_node_does_not_leave_the_index_pointing_at_the_wrong_one() {
        let mut graph = deserialized_graph();
        assert_eq!(
            graph.get_node_by_id("n1").map(|n| n.nodeid.as_str()),
            Some("n1")
        );

        graph.nodes.retain(|n| n.nodeid != "root");
        graph.invalidate_indices();

        assert_eq!(
            graph.get_node_by_id("n1").map(|n| n.nodeid.as_str()),
            Some("n1"),
            "n1 must still resolve to n1 after the vector shifted under the index"
        );
        assert!(graph.get_node_by_id("root").is_none());
    }
}
