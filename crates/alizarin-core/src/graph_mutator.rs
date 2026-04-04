//! # Graph Mutator
//!
//! Builder pattern for constructing and modifying Arches graphs using a combination
//! of the Builder and Command patterns.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use alizarin_core::graph_mutator::{GraphMutator, Cardinality};
//!
//! let graph = GraphMutator::new(base_graph)
//!     .add_semantic_node("parent", "child", "Child", Cardinality::N, "", "")?
//!     .add_string_node("child", "name", "Name", Cardinality::One, "", "")?
//!     .build()?;
//! ```
//!
//! ## Mutations Reference
//!
//! All mutations follow the Command pattern and can be serialized/deserialized for
//! cross-platform use (Rust, Python, WASM). Each mutation has a conformance level
//! indicating whether it's valid for branches, models, or both.
//!
//! ### Conformance Levels
//!
//! | Level | Description |
//! |-------|-------------|
//! | `AlwaysConformant` | Valid for both branches and resource models |
//! | `BranchConformant` | Valid only for branches (isresource=false) |
//! | `ModelConformant` | Valid only for resource models (isresource=true) |
//!
//! ---
//!
//! ## Structure Mutations (BranchConformant)
//!
//! ### AddNode
//!
//! Adds a new node to the graph as a child of an existing node.
//!
//! **Parameters:**
//! - `parent_alias` (Option<String>): Alias of parent node (None for root children)
//! - `alias` (String): Unique alias for the new node
//! - `name` (String): Display name
//! - `cardinality` (Cardinality): `One` or `N` - determines if nodegroup is created
//! - `datatype` (String): Node datatype (e.g., "string", "number", "concept", "semantic")
//! - `ontology_class` (String): Ontology class URI
//! - `parent_property` (String): Ontology property for edge to parent
//! - `description` (Option<String>): Node description
//! - `config` (Option<Value>): Node configuration JSON
//! - `options` (NodeOptions): Additional options (exportable, isrequired, etc.)
//!
//! **Behavior:**
//! - Creates nodegroup if cardinality=N or parent is root
//! - Auto-creates card if `MutatorOptions.autocreate_card` is true
//! - Auto-creates widget if `MutatorOptions.autocreate_widget` is true and datatype is not "semantic"
//! - Creates edge from parent to new node
//!
//! **Instruction:** `add_node` (subject: parent_alias, object: new_alias)
//!
//! ---
//!
//! ### AddNodegroup
//!
//! Adds a nodegroup to organize nodes with shared cardinality.
//!
//! **Parameters:**
//! - `nodegroup_id` (String): Unique ID for the nodegroup
//! - `cardinality` (Cardinality): `One` or `N`
//! - `parent_alias` (Option<String>): Alias of node that owns this nodegroup
//!
//! **Instruction:** `add_nodegroup` (subject: parent_alias)
//!
//! ---
//!
//! ### AddEdge
//!
//! Creates an edge connecting two existing nodes.
//!
//! **Parameters:**
//! - `from_node_id` (String): Source node ID or alias
//! - `to_node_id` (String): Target node ID or alias
//! - `ontology_property` (String): Ontology property URI
//! - `name` (Option<String>): Edge name
//! - `description` (Option<String>): Edge description
//!
//! **Instruction:** `add_edge` (subject: from_node, object: to_node)
//!
//! ---
//!
//! ### AddCard
//!
//! Adds a card (UI configuration) for a nodegroup.
//!
//! **Parameters:**
//! - `nodegroup_id` (String): Target nodegroup ID
//! - `name` (StaticTranslatableString): Card display name
//! - `component_id` (Option<String>): UI component ID
//! - `options` (CardOptions): Card options (active, visible, help text, etc.)
//! - `config` (Option<Value>): Card configuration JSON
//!
//! **Instruction:** `add_card` (subject: nodegroup_id, object: card_name)
//!
//! ---
//!
//! ### AddWidgetToCard
//!
//! Adds a widget mapping for a node within a card.
//!
//! **Parameters:**
//! - `node_id` (String): Target node ID or alias
//! - `widget_id` (String): Widget type ID
//! - `label` (String): Widget label
//! - `config` (Value): Widget configuration
//! - `sortorder` (Option<i32>): Display order
//! - `visible` (Option<bool>): Widget visibility
//!
//! **Instruction:** `add_widget` (subject: node_id)
//!
//! ---
//!
//! ### UpdateNode (BranchConformant)
//!
//! Updates node properties without changing structural IDs or datatype.
//!
//! **Parameters:**
//! - `node_id` (String): Node ID or alias to update
//! - `name` (Option<String>): New display name
//! - `ontology_class` (Option<String>): New ontology class
//! - `parent_property` (Option<String>): New parent property
//! - `description` (Option<String>): New description
//! - `config` (Option<Value>): Config to merge into existing
//! - `options` (UpdateNodeOptions): exportable, fieldname, isrequired, issearchable, sortorder
//!
//! **Preserved:** nodeid, nodegroup_id, graph_id, datatype, alias
//!
//! **Instruction:** `update_node` (subject: node_id)
//!
//! ---
//!
//! ### ChangeNodeType (BranchConformant)
//!
//! Changes a node's datatype. Requires that no widgets exist for the node.
//!
//! **Parameters:**
//! - `node_id` (String): Node ID or alias
//! - `datatype` (String): New datatype
//! - Plus all optional fields from UpdateNode
//!
//! **Error:** `NodeHasDependentWidgets` if node has widget mappings
//!
//! **Instruction:** `change_node_type` (subject: node_id, object: new_datatype)
//!
//! ---
//!
//! ### ChangeCardinality (BranchConformant)
//!
//! Changes a nodegroup's cardinality from 1 (single) to n (multiple) or vice versa.
//! The node must be the grouping node of its nodegroup.
//!
//! **Parameters:**
//! - `node_id` (String): Node ID or alias (must be the grouping node)
//! - `cardinality` (Cardinality): `One` or `N`
//!
//! **Error:** If node is not the grouping node for its nodegroup
//!
//! **Instruction:** `change_cardinality` (subject: node_id, object: "1" or "n")
//!
//! ---
//!
//! ## Subgraph Mutations (ModelConformant)
//!
//! ### AddSubgraph
//!
//! Appends an entire branch to a model with automatic ID remapping.
//!
//! **Parameters:**
//! - `subgraph` (StaticGraph): The branch to add
//! - `target_node_id` (String): Node to attach branch children to
//! - `ontology_property` (String): Property for connecting edges
//! - `alias_suffix` (Option<String>): Suffix for clashing aliases
//!
//! **Behavior:**
//! - Skips branch root; connects children directly to target
//! - Regenerates: nodeid, nodegroupid, edgeid, cardid, constraint IDs
//! - Preserves: widget_id, component_id, function_id, ontology URIs
//! - Sets `sourcebranchpublication_id` on added nodes
//!
//! **Instruction:** `add_subgraph` (subject: target_node_id, params.subgraph: JSON)
//!
//! ---
//!
//! ### UpdateSubgraph
//!
//! Updates an existing branch within a model.
//!
//! **Parameters:**
//! - `subgraph` (StaticGraph): Updated branch
//! - `target_node_id` (String): Root node of existing branch
//! - `ontology_property` (String): Property for new connecting edges
//! - `alias_suffix` (Option<String>): Suffix for aliases
//! - `remove_orphaned` (bool): If true, removes nodes no longer in branch
//!
//! **Behavior:**
//! - Updates existing nodes in place (preserves IDs)
//! - Adds new nodes from updated branch
//! - Optionally removes orphaned nodes
//!
//! **Instruction:** `update_subgraph` (subject: target_node_id, params.subgraph: JSON)
//!
//! ---
//!
//! ## Modification Mutations (AlwaysConformant)
//!
//! ### ConceptChangeCollection
//!
//! Changes the RDM collection for a concept or concept-list node.
//!
//! **Parameters:**
//! - `node_id` (String): Node ID or alias
//! - `collection_id` (String): New collection UUID
//!
//! **Error:** `InvalidDatatype` if node is not concept or concept-list type
//!
//! **Instruction:** `concept_change_collection` (subject: node_id, object: collection_id)
//!
//! ---
//!
//! ### RenameNode (AlwaysConformant)
//!
//! Changes text metadata for a node (alias, name, description).
//!
//! **Parameters:**
//! - `node_id` (String): Node ID or alias to rename
//! - `alias` (Option<String>): New alias
//! - `name` (Option<String>): New display name
//! - `description` (Option<String>): New description
//!
//! **Error:** `AliasAlreadyExists` if new alias is already used
//!
//! **Instruction:** `rename_node` (subject: node_id, object: new_alias)
//!
//! ---
//!
//! ## Deletion Mutations (AlwaysConformant)
//!
//! ### DeleteCard
//!
//! Removes a card and its widget mappings.
//!
//! **Parameters:**
//! - `card_id` (String): Card ID to delete
//!
//! **Cascades:** Removes all `cards_x_nodes_x_widgets` entries for this card
//!
//! **Instruction:** `delete_card` (subject: card_id)
//!
//! ---
//!
//! ### DeleteWidget
//!
//! Removes a widget mapping (cards_x_nodes_x_widgets entry).
//!
//! **Parameters:**
//! - `widget_mapping_id` (String): Widget mapping ID
//!
//! **Instruction:** `delete_widget` (subject: widget_mapping_id)
//!
//! ---
//!
//! ### DeleteFunction
//!
//! Removes a function mapping (functions_x_graphs entry).
//!
//! **Parameters:**
//! - `function_mapping_id` (String): Function mapping ID
//!
//! **Instruction:** `delete_function` (subject: function_mapping_id)
//!
//! ---
//!
//! ### DeleteNode
//!
//! Removes a node and its related entities.
//!
//! **Parameters:**
//! - `node_id` (String): Node ID or alias
//!
//! **Cascades:**
//! - Removes widget mappings for this node
//! - Removes edges where node is domain or range
//!
//! **Error:** `CannotDeleteRootNode` if node has `istopnode=true`
//!
//! **Instruction:** `delete_node` (subject: node_id)
//!
//! ---
//!
//! ### DeleteNodegroup
//!
//! Removes a nodegroup and all descendant entities.
//!
//! **Parameters:**
//! - `nodegroup_id` (String): Nodegroup ID
//!
//! **Cascades (recursively):**
//! - Child nodegroups (via parentnodegroup_id)
//! - All nodes in deleted nodegroups
//! - All edges referencing deleted nodes
//! - All cards for deleted nodegroups
//! - All widget mappings for deleted nodes
//!
//! **Error:** `CannotDeleteRootNode` if any affected node is root
//!
//! **Instruction:** `delete_nodegroup` (subject: nodegroup_id)
//!
//! ---
//!
//! ## Error Types
//!
//! | Error | Description |
//! |-------|-------------|
//! | `ParentNotFound` | Parent node alias not found |
//! | `NodeNotFound` | Node ID or alias not found |
//! | `NodegroupNotFound` | Nodegroup ID not found |
//! | `CardNotFound` | Card ID not found |
//! | `CardAlreadyExists` | Nodegroup already has a card |
//! | `WidgetNotFound` | Widget mapping ID not found |
//! | `FunctionNotFound` | Function mapping ID not found |
//! | `NoWidgetForDatatype` | No default widget for datatype |
//! | `AliasClash` | Alias conflicts with existing node |
//! | `AliasAlreadyExists` | New alias already in use |
//! | `BranchHasNoRoot` | Subgraph has no root node |
//! | `InvalidSubgraph` | Subgraph structure invalid |
//! | `InvalidDatatype` | Node datatype doesn't match operation |
//! | `NodeHasDependentWidgets` | Cannot change type with widgets |
//! | `CannotDeleteRootNode` | Cannot delete root node |
//! | `InconsistentBranchPublication` | Branch publication ID mismatch |
//! | `NoBranchNodesFound` | No branch nodes at target |

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::graph::{
    StaticCard, StaticCardsXNodesXWidgets, StaticEdge, StaticGraph, StaticNode, StaticNodegroup,
    StaticTranslatableString,
};

// =============================================================================
// UUID Generation
// =============================================================================

/// Base namespace UUID for Alizarin graph mutations
/// This ensures deterministic UUID generation across all platforms
const ALIZARIN_NAMESPACE: &str = "1a79f1c8-9505-4bea-a18e-28a053f725ca";

/// Generate a deterministic UUID v5 from a group and key
///
/// This matches the JS `generateUuidv5` function for cross-platform compatibility.
///
/// # Arguments
/// * `group` - A tuple of (type, optional_id) that forms the namespace
/// * `key` - The key to hash within the namespace
pub fn generate_uuid_v5(group: (&str, Option<&str>), key: &str) -> String {
    // Build namespace from group
    let namespace_str = match group.1 {
        Some(id) => format!("{}/{}", group.0, id),
        None => group.0.to_string(),
    };

    // Create namespace UUID from base namespace + group
    let base_namespace = Uuid::parse_str(ALIZARIN_NAMESPACE).expect("Invalid base namespace");
    let namespace = Uuid::new_v5(&base_namespace, namespace_str.as_bytes());

    // Generate final UUID from namespace + key
    Uuid::new_v5(&namespace, key.as_bytes()).to_string()
}

/// Convert a display name to a slug (lowercase, spaces to underscores)
///
/// # Example
/// ```
/// use alizarin_core::graph_mutator::slugify;
/// assert_eq!(slugify("Heritage Item"), "heritage_item");
/// assert_eq!(slugify("My Test Graph"), "my_test_graph");
/// ```
pub fn slugify(name: &str) -> String {
    name.to_lowercase().replace(' ', "_")
}

// =============================================================================
// Widget and CardComponent Types
// =============================================================================

/// A widget type for UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub name: String,
    pub datatype: String,
    pub default_config: serde_json::Value,
}

impl Widget {
    pub fn new(id: &str, name: &str, datatype: &str, default_config_json: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            datatype: datatype.to_string(),
            default_config: serde_json::from_str(default_config_json)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        }
    }

    /// Get a fresh copy of the default config
    pub fn get_default_config(&self) -> serde_json::Value {
        self.default_config.clone()
    }
}

/// Convert a RegisteredWidget (from dynamic registry) to Widget
impl From<crate::registry::RegisteredWidget> for Widget {
    fn from(registered: crate::registry::RegisteredWidget) -> Self {
        Self {
            id: registered.id,
            name: registered.name,
            datatype: registered.datatype,
            default_config: registered.default_config,
        }
    }
}

/// A card component type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardComponent {
    pub id: String,
    pub name: String,
}

impl CardComponent {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

/// The default card component from Arches
pub const DEFAULT_CARD_COMPONENT_ID: &str = "f05e4d3a-53c1-11e8-b0ea-784f435179ea";
pub const DEFAULT_CARD_COMPONENT_NAME: &str = "Default Card";

/// Get the default card component
pub fn default_card_component() -> CardComponent {
    CardComponent::new(DEFAULT_CARD_COMPONENT_ID, DEFAULT_CARD_COMPONENT_NAME)
}

// =============================================================================
// Default Widget Registry
// =============================================================================

lazy_static::lazy_static! {
    /// Default widgets by name (from Arches)
    pub static ref WIDGETS: HashMap<String, Widget> = {
        let mut m = HashMap::new();
        m.insert("text-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000001",
            "text-widget",
            "string",
            r#"{ "placeholder": "Enter text", "width": "100%", "maxLength": null}"#
        ));
        m.insert("concept-select-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000002",
            "concept-select-widget",
            "concept",
            r#"{ "placeholder": "Select an option", "options": [] }"#
        ));
        m.insert("resource-instance-multiselect-widget".to_string(), Widget::new(
            "ff3c400a-76ec-11e7-a793-784f435179ea",
            "resource-instance-multiselect-widget",
            "resource-instance-list",
            r#"{ "placeholder": "Select an option", "options": [] }"#
        ));
        m.insert("concept-multiselect-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000012",
            "concept-multiselect-widget",
            "concept-list",
            r#"{ "placeholder": "Select an option", "options": [] }"#
        ));
        m.insert("domain-select-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000015",
            "domain-select-widget",
            "domain-value",
            r#"{ "placeholder": "Select an option" }"#
        ));
        m.insert("domain-multiselect-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000016",
            "domain-multiselect-widget",
            "domain-value-list",
            r#"{ "placeholder": "Select an option" }"#
        ));
        m.insert("switch-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000003",
            "switch-widget",
            "boolean",
            r#"{ "subtitle": "Click to switch"}"#
        ));
        m.insert("datepicker-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000004",
            "datepicker-widget",
            "date",
            r#"{
                "placeholder": "Enter date",
                "viewMode": "days",
                "dateFormat": "YYYY-MM-DD",
                "minDate": false,
                "maxDate": false
            }"#
        ));
        m.insert("rich-text-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000005",
            "rich-text-widget",
            "string",
            r#"{}"#
        ));
        m.insert("radio-boolean-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000006",
            "radio-boolean-widget",
            "boolean",
            r#"{"trueLabel": "Yes", "falseLabel": "No"}"#
        ));
        m.insert("map-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000007",
            "map-widget",
            "geojson-feature-collection",
            r#"{
                "basemap": "streets",
                "geometryTypes": [{"text":"Point", "id":"Point"}, {"text":"Line", "id":"Line"}, {"text":"Polygon", "id":"Polygon"}],
                "overlayConfigs": [],
                "overlayOpacity": 0.0,
                "geocodeProvider": "MapzenGeocoder",
                "zoom": 0,
                "maxZoom": 20,
                "minZoom": 0,
                "centerX": 0,
                "centerY": 0,
                "pitch": 0.0,
                "bearing": 0.0,
                "geocodePlaceholder": "Search",
                "geocoderVisible": true,
                "featureColor": null,
                "featureLineWidth": null,
                "featurePointSize": null
            }"#
        ));
        m.insert("number-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000008",
            "number-widget",
            "number",
            r#"{ "placeholder": "Enter number", "width": "100%", "min":"", "max":""}"#
        ));
        m.insert("concept-radio-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000009",
            "concept-radio-widget",
            "concept",
            r#"{ "options": [] }"#
        ));
        m.insert("concept-checkbox-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000013",
            "concept-checkbox-widget",
            "concept-list",
            r#"{ "options": [] }"#
        ));
        m.insert("domain-radio-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000017",
            "domain-radio-widget",
            "domain-value",
            r#"{}"#
        ));
        m.insert("domain-checkbox-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000018",
            "domain-checkbox-widget",
            "domain-value-list",
            r#"{}"#
        ));
        m.insert("file-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000019",
            "file-widget",
            "file-list",
            r#"{"acceptedFiles": "", "maxFilesize": "200"}"#
        ));
        m.insert("urldatatype-widget".to_string(), Widget::new(
            "ca0c43ff-af73-4349-bafd-53ff9f22eebd",
            "urldatatype-widget",
            "url",
            r#"{ "placeholder": "Enter URL", "url_placeholder": "Enter URL", "url_label_placeholder": "Enter URL label" }"#
        ));
        m.insert("resource-instance-select-widget".to_string(), Widget::new(
            "31f3728c-7613-11e7-a139-784f435179ea",
            "resource-instance-select-widget",
            "resource-instance",
            r#"{ "placeholder": "Select a resource" }"#
        ));
        m.insert("edtf-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000010",
            "edtf-widget",
            "edtf",
            r#"{ "placeholder": "Enter EDTF date" }"#
        ));
        m.insert("non-localized-text-widget".to_string(), Widget::new(
            "10000000-0000-0000-0000-000000000011",
            "non-localized-text-widget",
            "non-localized-string",
            r#"{ "placeholder": "Enter text", "width": "100%" }"#
        ));
        m
    };
}

lazy_static::lazy_static! {
    /// Reverse lookup: widget ID -> widget name
    pub static ref WIDGET_BY_ID: HashMap<String, String> = {
        WIDGETS.iter().map(|(name, w)| (w.id.clone(), name.clone())).collect()
    };
}

/// Look up a widget name by its UUID.
///
/// Checks both the static Arches widget registry and the dynamic extension registry.
pub fn get_widget_name_by_id(widget_id: &str) -> Option<String> {
    // Check static registry first
    if let Some(name) = WIDGET_BY_ID.get(widget_id) {
        return Some(name.clone());
    }
    // Check dynamic (extension) registry
    for name in crate::registry::registered_widgets() {
        if let Some(widget) = crate::registry::get_registered_widget(&name) {
            if widget.id == widget_id {
                return Some(name);
            }
        }
    }
    None
}

/// Get the default widget for a datatype
///
/// Checks in order:
/// 1. Extension widget mapping registry (datatype -> widget name)
/// 2. Dynamic widget registry (for extension-registered widgets)
/// 3. Core static widget mappings
pub fn get_default_widget_for_datatype(datatype: &str) -> Result<Widget, MutationError> {
    // First check the extension registry for custom datatype mappings
    if let Some(widget_name) = crate::registry::get_widget_for_datatype(datatype) {
        // Check dynamic widget registry first
        if let Some(registered) = crate::registry::get_registered_widget(&widget_name) {
            return Ok(Widget::from(registered));
        }
        // Fall back to static widgets
        return WIDGETS
            .get(&widget_name)
            .cloned()
            .ok_or(MutationError::WidgetNotFound(widget_name));
    }

    // Fall back to core datatype mappings
    let widget_name = match datatype {
        "number" => "number-widget",
        "string" => "text-widget",
        "concept" => "concept-select-widget",
        "concept-list" => "concept-multiselect-widget",
        "resource-instance-list" => "resource-instance-multiselect-widget",
        "domain-value" => "domain-select-widget",
        "domain-value-list" => "domain-multiselect-widget",
        "geojson-feature-collection" => "map-widget",
        "boolean" => "switch-widget",
        "date" => "datepicker-widget",
        "url" => "urldatatype-widget",
        "resource-instance" => "resource-instance-select-widget",
        "edtf" => "edtf-widget",
        "non-localized-string" => "non-localized-text-widget",
        "file-list" => "file-widget",
        "semantic" => return Err(MutationError::NoWidgetForDatatype(datatype.to_string())),
        other => return Err(MutationError::NoWidgetForDatatype(other.to_string())),
    };

    WIDGETS
        .get(widget_name)
        .cloned()
        .ok_or_else(|| MutationError::WidgetNotFound(widget_name.to_string()))
}

// =============================================================================
// Cardinality
// =============================================================================

/// Node cardinality (one or many instances allowed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String")]
pub enum Cardinality {
    /// Single instance only
    One,
    /// Multiple instances allowed
    N,
}

impl Cardinality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Cardinality::One => "1",
            Cardinality::N => "n",
        }
    }
}

impl From<&str> for Cardinality {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "1" | "one" => Cardinality::One,
            _ => Cardinality::N,
        }
    }
}

impl From<String> for Cardinality {
    fn from(s: String) -> Self {
        Cardinality::from(s.as_str())
    }
}

// =============================================================================
// Mutation Error
// =============================================================================

/// Errors that can occur during graph mutation
#[derive(Debug, Clone)]
pub enum MutationError {
    /// Parent node not found
    ParentNotFound(String),
    /// Node not found
    NodeNotFound(String),
    /// Nodegroup not found
    NodegroupNotFound(String),
    /// Card not found for nodegroup
    CardNotFound(String),
    /// Nodegroup already has a card
    CardAlreadyExists(String),
    /// No widget for this datatype
    NoWidgetForDatatype(String),
    /// Widget not found
    WidgetNotFound(String),
    /// JSON serialization error
    JsonError(String),
    /// Alias already exists in target graph
    AliasClash(String),
    /// Branch has no root node
    BranchHasNoRoot,
    /// Invalid subgraph structure
    InvalidSubgraph(String),
    /// Inconsistent branch publication ID during traversal
    InconsistentBranchPublication {
        expected: String,
        found: Option<String>,
        node_id: String,
    },
    /// No nodes found for the branch at target
    NoBranchNodesFound(String),
    /// Invalid datatype for operation
    InvalidDatatype {
        expected: String,
        found: String,
        node_id: String,
    },
    /// Function mapping not found
    FunctionNotFound(String),
    /// Cannot delete root node
    CannotDeleteRootNode(String),
    /// Node has dependent widgets (cannot change type)
    NodeHasDependentWidgets(String),
    /// Alias already exists
    AliasAlreadyExists(String),
    /// Invalid node config
    InvalidConfig { alias: String, error: String },
    /// Extension mutation not found in registry
    ExtensionNotFound(String),
    /// Extension mutation used but no registry provided
    NoExtensionRegistry(String),
    /// Ontology validation failure
    OntologyValidation(crate::ontology::OntologyValidationDetail),
    /// Generic error
    Other(String),
}

impl std::fmt::Display for MutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationError::ParentNotFound(alias) => write!(f, "Parent node not found: {}", alias),
            MutationError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            MutationError::NodegroupNotFound(id) => write!(f, "Nodegroup not found: {}", id),
            MutationError::CardNotFound(ng) => write!(f, "Card not found for nodegroup: {}", ng),
            MutationError::CardAlreadyExists(ng) => {
                write!(f, "Nodegroup already has a card: {}", ng)
            }
            MutationError::NoWidgetForDatatype(dt) => {
                write!(f, "No default widget for datatype: {}", dt)
            }
            MutationError::WidgetNotFound(name) => write!(f, "Widget not found: {}", name),
            MutationError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            MutationError::AliasClash(alias) => {
                write!(f, "Alias already exists in target graph: {}", alias)
            }
            MutationError::BranchHasNoRoot => write!(f, "Branch has no root node"),
            MutationError::InvalidSubgraph(msg) => write!(f, "Invalid subgraph: {}", msg),
            MutationError::InconsistentBranchPublication {
                expected,
                found,
                node_id,
            } => {
                write!(
                    f,
                    "Inconsistent branch publication ID at node {}: expected {}, found {:?}",
                    node_id, expected, found
                )
            }
            MutationError::NoBranchNodesFound(target_id) => {
                write!(f, "No branch nodes found at target: {}", target_id)
            }
            MutationError::InvalidDatatype {
                expected,
                found,
                node_id,
            } => {
                write!(
                    f,
                    "Invalid datatype for node {}: expected {}, found {}",
                    node_id, expected, found
                )
            }
            MutationError::FunctionNotFound(id) => write!(f, "Function mapping not found: {}", id),
            MutationError::CannotDeleteRootNode(id) => write!(f, "Cannot delete root node: {}", id),
            MutationError::NodeHasDependentWidgets(id) => {
                write!(f, "Node has dependent widgets, cannot change type: {}", id)
            }
            MutationError::AliasAlreadyExists(alias) => {
                write!(f, "Alias already exists: {}", alias)
            }
            MutationError::InvalidConfig { alias, error } => {
                write!(f, "Invalid config for node '{}': {}", alias, error)
            }
            MutationError::ExtensionNotFound(name) => {
                write!(f, "Extension mutation not found: {}", name)
            }
            MutationError::NoExtensionRegistry(name) => write!(
                f,
                "Extension mutation '{}' used but no registry provided",
                name
            ),
            MutationError::OntologyValidation(detail) => {
                write!(f, "Ontology validation error: {}", detail)
            }
            MutationError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MutationError {}

// =============================================================================
// Mutation Types (Command Pattern)
// =============================================================================

/// Parameters for adding a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNodeParams {
    pub parent_alias: Option<String>,
    pub alias: String,
    pub name: String,
    pub cardinality: Cardinality,
    pub datatype: String,
    pub ontology_class: String,
    pub parent_property: String,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub options: NodeOptions,
}

/// Options for node creation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeOptions {
    pub exportable: Option<bool>,
    pub fieldname: Option<String>,
    pub hascustomalias: Option<bool>,
    pub is_collector: Option<bool>,
    pub isrequired: Option<bool>,
    pub issearchable: Option<bool>,
    pub istopnode: Option<bool>,
    pub sortorder: Option<i32>,
}

/// Parameters for adding a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCardParams {
    pub nodegroup_id: String,
    pub name: StaticTranslatableString,
    pub component_id: Option<String>,
    pub options: CardOptions,
    pub config: Option<serde_json::Value>,
}

/// Options for card creation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardOptions {
    pub active: Option<bool>,
    pub cssclass: Option<String>,
    pub helpenabled: Option<bool>,
    pub helptext: Option<StaticTranslatableString>,
    pub helptitle: Option<StaticTranslatableString>,
    pub instructions: Option<StaticTranslatableString>,
    pub is_editable: Option<bool>,
    pub description: Option<StaticTranslatableString>,
    pub sortorder: Option<i32>,
    pub visible: Option<bool>,
}

/// Parameters for adding a widget to a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddWidgetParams {
    pub node_id: String,
    pub widget_id: String,
    pub label: String,
    pub config: serde_json::Value,
    pub sortorder: Option<i32>,
    pub visible: Option<bool>,
}

/// Parameters for adding a nodegroup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNodegroupParams {
    pub parent_alias: Option<String>,
    pub nodegroup_id: String,
    pub cardinality: Cardinality,
}

/// Parameters for adding an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEdgeParams {
    pub from_node_id: String,
    pub to_node_id: String,
    pub ontology_property: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Parameters for adding an entire subgraph/branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSubgraphParams {
    /// The subgraph to add (branch)
    pub subgraph: StaticGraph,
    /// Target node ID to attach the branch's children to
    pub target_node_id: String,
    /// Ontology property for the connecting edges
    pub ontology_property: String,
    /// Optional suffix to add to all aliases from the subgraph
    #[serde(default)]
    pub alias_suffix: Option<String>,
    /// Optional prefix for aliases (e.g. "monument" → alias "name" becomes "monument_name")
    #[serde(default)]
    pub alias_prefix: Option<String>,
    /// Optional prefix for display names (e.g. "Monument" → name "Name" becomes "Monument Name")
    #[serde(default)]
    pub name_prefix: Option<String>,
}

/// Parameters for updating an existing subgraph/branch in a graph
///
/// This mutation finds nodes previously added from a branch (by alias matching)
/// and updates them to match the current branch version:
/// - **Update**: Nodes that exist in both branch and graph (preserves IDs)
/// - **Add**: Nodes new in branch that don't exist in graph
/// - **Remove**: Nodes in graph from old branch no longer in new branch (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubgraphParams {
    /// The new version of the subgraph/branch
    pub subgraph: StaticGraph,
    /// Target node ID where the branch is attached
    pub target_node_id: String,
    /// Ontology property for connecting edges (used for new nodes)
    pub ontology_property: String,
    /// The suffix used when the branch was originally added (if any)
    #[serde(default)]
    pub alias_suffix: Option<String>,
    /// Whether to remove nodes that are no longer in the branch (default: false)
    /// WARNING: Setting this to true may orphan resource instance data
    #[serde(default)]
    pub remove_orphaned: bool,
    /// Optional prefix for aliases — used when matching branch aliases to existing
    /// prefixed aliases (e.g. "monument" matches branch "name" to existing "monument_name")
    /// and when adding new nodes from the branch.
    #[serde(default)]
    pub alias_prefix: Option<String>,
    /// Optional prefix for display names — applied to new nodes added from the branch
    /// (e.g. "Monument" → "Name Type" becomes "Monument Name Type")
    #[serde(default)]
    pub name_prefix: Option<String>,
}

/// Parameters for changing the collection of a concept/concept-list node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptChangeCollectionParams {
    /// Node alias or ID to update
    pub node_id: String,
    /// New collection ID (UUID of the RDM collection)
    pub collection_id: String,
}

/// Parameters for deleting a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCardParams {
    /// Card ID to delete
    pub card_id: String,
}

/// Parameters for deleting a widget mapping (cards_x_nodes_x_widgets entry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWidgetParams {
    /// Widget mapping ID (the id field of cards_x_nodes_x_widgets)
    pub widget_mapping_id: String,
}

/// Parameters for deleting a function mapping (functions_x_graphs entry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFunctionParams {
    /// Function mapping ID (the id field of functions_x_graphs)
    pub function_mapping_id: String,
}

/// Parameters for deleting a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNodeParams {
    /// Node ID or alias to delete
    pub node_id: String,
}

/// Parameters for deleting a nodegroup (cascades to children)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNodegroupParams {
    /// Nodegroup ID to delete
    pub nodegroup_id: String,
}

/// Parameters for updating a node (preserves structural IDs, cannot change datatype)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodeParams {
    /// Node ID or alias to update
    pub node_id: String,
    /// New name (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New ontology class (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ontology_class: Option<String>,
    /// New parent property (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_property: Option<String>,
    /// New description (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New config (if provided, replaces existing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Update options
    #[serde(default)]
    pub options: UpdateNodeOptions,
}

/// Options for updating a node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateNodeOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exportable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fieldname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isrequired: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issearchable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sortorder: Option<i32>,
}

/// Parameters for changing a node's datatype (requires no dependent widgets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeNodeTypeParams {
    /// Node ID or alias to update
    pub node_id: String,
    /// New datatype
    pub datatype: String,
    /// New name (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New ontology class (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ontology_class: Option<String>,
    /// New parent property (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_property: Option<String>,
    /// New description (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New config (if provided, replaces existing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Update options
    #[serde(default)]
    pub options: UpdateNodeOptions,
}

/// Parameters for renaming a node (text metadata only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameNodeParams {
    /// Node ID or alias to rename
    pub node_id: String,
    /// New alias (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// New name (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Parameters for changing a nodegroup's cardinality (1 <-> n)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeCardinalityParams {
    /// Node ID or alias - the grouping node of the nodegroup to change
    pub node_id: String,
    /// New cardinality
    pub cardinality: Cardinality,
}

/// Parameters for creating a new graph from scratch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGraphParams {
    /// Name for the graph
    pub name: String,
    /// Whether this is a resource model (true) or branch (false)
    pub is_resource: bool,
    /// Alias for the root node
    pub root_alias: String,
    /// Ontology class URI for the root node
    pub root_ontology_class: String,
    /// Optional custom graph ID (otherwise generated deterministically)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<String>,
    /// Optional author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Parameters for renaming a graph (updating name, description, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameGraphParams {
    /// New name for the graph (language -> value map)
    /// If provided, replaces the graph's name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<std::collections::HashMap<String, String>>,
    /// New description for the graph (language -> value map)
    /// If provided, replaces the graph's description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<std::collections::HashMap<String, String>>,
    /// New subtitle for the graph (language -> value map)
    /// If provided, replaces the graph's subtitle
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<std::collections::HashMap<String, String>>,
    /// New author for the graph
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

// =============================================================================
// Extension Mutations
// =============================================================================

/// Parameters for an extension mutation
///
/// Extension mutations allow external crates (like CLM) to define custom
/// graph operations that integrate with the mutation system.
///
/// ## Example
///
/// ```ignore
/// // CLM defines a mutation to change a reference node's controlled list
/// let mutation = GraphMutation::Extension(ExtensionMutationParams {
///     name: "clm.reference_change_collection".to_string(),
///     params: serde_json::json!({
///         "node_id": "my_reference_node",
///         "collection_id": "new-collection-uuid"
///     }),
///     conformance: MutationConformance::AlwaysConformant,
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMutationParams {
    /// The registered mutation name (e.g., "clm.reference_change_collection")
    ///
    /// Convention: use "extension_name.mutation_name" format
    pub name: String,

    /// The mutation parameters as JSON
    ///
    /// The handler registered for this mutation name interprets these params.
    pub params: serde_json::Value,

    /// Conformance level for this mutation
    ///
    /// Set by the extension when defining the mutation.
    #[serde(default = "default_extension_conformance")]
    pub conformance: MutationConformance,
}

fn default_extension_conformance() -> MutationConformance {
    MutationConformance::AlwaysConformant
}

/// Handler trait for extension mutations
///
/// Extensions implement this trait to define custom mutations.
/// Handlers are registered with `ExtensionMutationRegistry`.
///
/// ## Example
///
/// ```ignore
/// struct ReferenceChangeCollectionHandler;
///
/// impl ExtensionMutationHandler for ReferenceChangeCollectionHandler {
///     fn apply(
///         &self,
///         graph: &mut StaticGraph,
///         params: &serde_json::Value,
///         _options: &MutatorOptions,
///     ) -> Result<(), MutationError> {
///         let node_id = params.get("node_id")
///             .and_then(|v| v.as_str())
///             .ok_or_else(|| MutationError::Other("missing node_id".into()))?;
///         // ... apply the mutation
///         Ok(())
///     }
///
///     fn conformance(&self) -> MutationConformance {
///         MutationConformance::AlwaysConformant
///     }
/// }
/// ```
pub trait ExtensionMutationHandler: Send + Sync {
    /// Apply the mutation to the graph
    fn apply(
        &self,
        graph: &mut StaticGraph,
        params: &serde_json::Value,
        options: &MutatorOptions,
    ) -> Result<(), MutationError>;

    /// Get the default conformance level for this mutation type
    ///
    /// This can be overridden per-invocation via `ExtensionMutationParams.conformance`.
    fn conformance(&self) -> MutationConformance;

    /// Get a description of this mutation for documentation
    fn description(&self) -> &str {
        "Extension mutation"
    }
}

/// Registry for extension mutation handlers
///
/// Extensions register their mutation handlers here. The registry is passed
/// to `apply_mutations` functions to enable extension mutations.
///
/// ## Thread Safety
///
/// The registry is thread-safe for concurrent reads. Registration should
/// happen at startup before mutations are applied.
///
/// ## Example
///
/// ```ignore
/// use std::sync::Arc;
///
/// let mut registry = ExtensionMutationRegistry::new();
/// registry.register(
///     "clm.reference_change_collection",
///     Arc::new(ReferenceChangeCollectionHandler),
/// );
///
/// // Pass registry when applying mutations
/// apply_mutations_with_extensions(graph, mutations, options, Some(&registry))?;
/// ```
pub struct ExtensionMutationRegistry {
    handlers: std::collections::HashMap<String, std::sync::Arc<dyn ExtensionMutationHandler>>,
}

impl ExtensionMutationRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Register a mutation handler
    ///
    /// # Arguments
    /// * `name` - The mutation name (e.g., "clm.reference_change_collection")
    /// * `handler` - The handler implementation
    pub fn register(
        &mut self,
        name: impl Into<String>,
        handler: std::sync::Arc<dyn ExtensionMutationHandler>,
    ) {
        self.handlers.insert(name.into(), handler);
    }

    /// Get a handler by name
    pub fn get(&self, name: &str) -> Option<&std::sync::Arc<dyn ExtensionMutationHandler>> {
        self.handlers.get(name)
    }

    /// Check if a handler is registered
    pub fn has(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// List all registered mutation names
    pub fn list(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ExtensionMutationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ExtensionMutationRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionMutationRegistry")
            .field("handlers", &self.handlers.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Conformance level for graph mutations
///
/// Indicates what type of graph a mutation is valid for:
/// - `AlwaysConformant`: Valid for both branches and models
/// - `BranchConformant`: Valid only when building/modifying branches
/// - `ModelConformant`: Valid only when building/modifying models
/// - `NonConformant`: Not valid for standard use (deprecated or special-purpose)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationConformance {
    /// Valid for both branches and resource models
    AlwaysConformant,
    /// Valid only for branches (isresource=false)
    BranchConformant,
    /// Valid only for resource models (isresource=true)
    ModelConformant,
    /// Not conformant with standard workflows
    NonConformant,
}

/// A graph mutation operation (Command pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphMutation {
    AddNode(AddNodeParams),
    AddNodegroup(AddNodegroupParams),
    AddEdge(AddEdgeParams),
    AddCard(AddCardParams),
    AddWidgetToCard(AddWidgetParams),
    AddSubgraph(AddSubgraphParams),
    UpdateSubgraph(UpdateSubgraphParams),
    ConceptChangeCollection(ConceptChangeCollectionParams),
    DeleteCard(DeleteCardParams),
    DeleteWidget(DeleteWidgetParams),
    DeleteFunction(DeleteFunctionParams),
    DeleteNode(DeleteNodeParams),
    DeleteNodegroup(DeleteNodegroupParams),
    UpdateNode(UpdateNodeParams),
    ChangeNodeType(ChangeNodeTypeParams),
    ChangeCardinality(ChangeCardinalityParams),
    RenameNode(RenameNodeParams),
    /// Rename/update graph metadata (name, description, subtitle, author)
    RenameGraph(RenameGraphParams),
    /// Create a new graph from scratch (only valid as first mutation in apply_mutations_create)
    CreateGraph(CreateGraphParams),
    /// Extension mutation - delegated to a registered handler
    Extension(ExtensionMutationParams),
}

impl GraphMutation {
    /// Get the conformance level for this mutation
    pub fn conformance(&self) -> MutationConformance {
        match self {
            // Basic structure operations - valid for branches
            GraphMutation::AddNode(_) => MutationConformance::BranchConformant,
            GraphMutation::AddNodegroup(_) => MutationConformance::BranchConformant,
            GraphMutation::AddEdge(_) => MutationConformance::BranchConformant,
            GraphMutation::AddCard(_) => MutationConformance::BranchConformant,
            GraphMutation::AddWidgetToCard(_) => MutationConformance::BranchConformant,
            // Subgraph operations - only valid for models (adding branches to models)
            GraphMutation::AddSubgraph(_) => MutationConformance::ModelConformant,
            GraphMutation::UpdateSubgraph(_) => MutationConformance::ModelConformant,
            // Collection changes - valid for both
            GraphMutation::ConceptChangeCollection(_) => MutationConformance::AlwaysConformant,
            // Deletion operations - valid for both branches and models
            GraphMutation::DeleteCard(_) => MutationConformance::AlwaysConformant,
            GraphMutation::DeleteWidget(_) => MutationConformance::AlwaysConformant,
            GraphMutation::DeleteFunction(_) => MutationConformance::AlwaysConformant,
            GraphMutation::DeleteNode(_) => MutationConformance::AlwaysConformant,
            GraphMutation::DeleteNodegroup(_) => MutationConformance::AlwaysConformant,
            // Node update operations
            GraphMutation::UpdateNode(_) => MutationConformance::BranchConformant,
            GraphMutation::ChangeNodeType(_) => MutationConformance::BranchConformant,
            GraphMutation::ChangeCardinality(_) => MutationConformance::BranchConformant,
            GraphMutation::RenameNode(_) => MutationConformance::AlwaysConformant,
            // Graph metadata update - valid for both
            GraphMutation::RenameGraph(_) => MutationConformance::AlwaysConformant,
            // CreateGraph - not a standard mutation, only valid in apply_mutations_create
            GraphMutation::CreateGraph(_) => MutationConformance::NonConformant,
            // Extension mutations - conformance is specified in params
            GraphMutation::Extension(params) => params.conformance,
        }
    }
}

// =============================================================================
// Graph Mutator Options
// =============================================================================

/// Options for the GraphMutator
#[derive(Debug, Clone)]
pub struct MutatorOptions {
    /// Automatically create cards for new nodegroups
    pub autocreate_card: bool,
    /// Automatically add default widgets to cards
    pub autocreate_widget: bool,
    /// Optional ontology validator for class/property validation
    pub ontology_validator: Option<crate::ontology::OntologyValidator>,
}

impl Default for MutatorOptions {
    fn default() -> Self {
        Self {
            autocreate_card: true,
            autocreate_widget: true,
            ontology_validator: None,
        }
    }
}

// =============================================================================
// Graph Mutator (Builder Pattern)
// =============================================================================

/// Builder for constructing and modifying graphs
///
/// Uses a combination of Builder and Command patterns:
/// - Builder pattern for ergonomic, chainable API
/// - Command pattern for serializable, replayable mutations
pub struct GraphMutator {
    base_graph: StaticGraph,
    mutations: Vec<GraphMutation>,
    options: MutatorOptions,
}

impl GraphMutator {
    /// Create a new GraphMutator from a base graph
    pub fn new(base_graph: StaticGraph) -> Self {
        Self {
            base_graph,
            mutations: Vec::new(),
            options: MutatorOptions::default(),
        }
    }

    /// Create a new GraphMutator with custom options
    pub fn with_options(base_graph: StaticGraph, options: MutatorOptions) -> Self {
        Self {
            base_graph,
            mutations: Vec::new(),
            options,
        }
    }

    /// Get the list of pending mutations (for debugging/serialization)
    pub fn mutations(&self) -> &[GraphMutation] {
        &self.mutations
    }

    // =========================================================================
    // Node Addition Methods
    // =========================================================================

    /// Add a semantic node (structural grouping node)
    #[allow(clippy::too_many_arguments)]
    pub fn add_semantic_node(
        mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        cardinality: Cardinality,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        self.add_generic_node_mut(
            parent_alias,
            alias,
            name,
            cardinality,
            "semantic",
            ontology_class,
            parent_property,
            description,
            options,
            config,
        );
        self
    }

    /// Add a string node
    #[allow(clippy::too_many_arguments)]
    pub fn add_string_node(
        mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        cardinality: Cardinality,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        self.add_generic_node_mut(
            parent_alias,
            alias,
            name,
            cardinality,
            "string",
            ontology_class,
            parent_property,
            description,
            options,
            config,
        );
        self
    }

    /// Add a concept node (RDM collection reference)
    #[allow(clippy::too_many_arguments)]
    pub fn add_concept_node(
        mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        collection_id: Option<&str>,
        is_list: bool,
        cardinality: Cardinality,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        let mut node_config = config.unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let Some(coll_id) = collection_id {
            if let serde_json::Value::Object(ref mut map) = node_config {
                map.insert(
                    "rdmCollection".to_string(),
                    serde_json::Value::String(coll_id.to_string()),
                );
            }
        }
        let datatype = if is_list { "concept-list" } else { "concept" };
        self.add_generic_node_mut(
            parent_alias,
            alias,
            name,
            cardinality,
            datatype,
            ontology_class,
            parent_property,
            description,
            options,
            Some(node_config),
        );
        self
    }

    /// Add a number node
    #[allow(clippy::too_many_arguments)]
    pub fn add_number_node(
        mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        cardinality: Cardinality,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        self.add_generic_node_mut(
            parent_alias,
            alias,
            name,
            cardinality,
            "number",
            ontology_class,
            parent_property,
            description,
            options,
            config,
        );
        self
    }

    /// Add a date node
    #[allow(clippy::too_many_arguments)]
    pub fn add_date_node(
        mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        cardinality: Cardinality,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        self.add_generic_node_mut(
            parent_alias,
            alias,
            name,
            cardinality,
            "date",
            ontology_class,
            parent_property,
            description,
            options,
            config,
        );
        self
    }

    /// Add a boolean node
    #[allow(clippy::too_many_arguments)]
    pub fn add_boolean_node(
        mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        cardinality: Cardinality,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        self.add_generic_node_mut(
            parent_alias,
            alias,
            name,
            cardinality,
            "boolean",
            ontology_class,
            parent_property,
            description,
            options,
            config,
        );
        self
    }

    /// Add a generic node with any datatype (consuming builder pattern)
    #[allow(clippy::too_many_arguments)]
    pub fn add_generic_node(
        mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        cardinality: Cardinality,
        datatype: &str,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        self.add_generic_node_mut(
            parent_alias,
            alias,
            name,
            cardinality,
            datatype,
            ontology_class,
            parent_property,
            description,
            options,
            config,
        );
        self
    }

    /// Internal: add a generic node (mutating)
    #[allow(clippy::too_many_arguments)]
    fn add_generic_node_mut(
        &mut self,
        parent_alias: Option<&str>,
        alias: &str,
        name: &str,
        cardinality: Cardinality,
        datatype: &str,
        ontology_class: &str,
        parent_property: &str,
        description: Option<&str>,
        options: NodeOptions,
        config: Option<serde_json::Value>,
    ) {
        self.mutations.push(GraphMutation::AddNode(AddNodeParams {
            parent_alias: parent_alias.map(String::from),
            alias: alias.to_string(),
            name: name.to_string(),
            cardinality,
            datatype: datatype.to_string(),
            ontology_class: ontology_class.to_string(),
            parent_property: parent_property.to_string(),
            description: description.map(String::from),
            config,
            options,
        }));
    }

    // =========================================================================
    // Card Methods
    // =========================================================================

    /// Add a card for a nodegroup
    pub fn add_card(
        mut self,
        nodegroup_id: &str,
        name: &str,
        options: CardOptions,
        config: Option<serde_json::Value>,
    ) -> Self {
        self.mutations.push(GraphMutation::AddCard(AddCardParams {
            nodegroup_id: nodegroup_id.to_string(),
            name: StaticTranslatableString::from_string(name),
            component_id: Some(DEFAULT_CARD_COMPONENT_ID.to_string()),
            options,
            config,
        }));
        self
    }

    /// Add a widget to a card
    pub fn add_widget_to_card(
        mut self,
        node_id: &str,
        widget: &Widget,
        label: &str,
        config: serde_json::Value,
        sortorder: Option<i32>,
        visible: Option<bool>,
    ) -> Self {
        self.mutations
            .push(GraphMutation::AddWidgetToCard(AddWidgetParams {
                node_id: node_id.to_string(),
                widget_id: widget.id.clone(),
                label: label.to_string(),
                config,
                sortorder,
                visible,
            }));
        self
    }

    // =========================================================================
    // Build Methods
    // =========================================================================

    /// Apply all mutations and return the resulting graph
    pub fn build(self) -> Result<StaticGraph, MutationError> {
        let mut graph = self.base_graph.deep_clone();

        for mutation in self.mutations {
            apply_mutation(&mut graph, mutation, &self.options)?;
        }

        // Rebuild indices after all mutations
        graph.build_indices();

        Ok(graph)
    }
}

// =============================================================================
// Mutation Application
// =============================================================================

/// Apply a single mutation to a graph
///
/// For extension mutations, pass an `ExtensionMutationRegistry` via `registry`.
/// If an extension mutation is encountered without a registry, an error is returned.
fn apply_mutation(
    graph: &mut StaticGraph,
    mutation: GraphMutation,
    options: &MutatorOptions,
) -> Result<(), MutationError> {
    apply_mutation_with_extensions(graph, mutation, options, None)
}

/// Apply a single mutation to a graph with extension support
///
/// # Arguments
/// * `graph` - The graph to mutate
/// * `mutation` - The mutation to apply
/// * `options` - Mutator options
/// * `registry` - Optional extension mutation registry
fn apply_mutation_with_extensions(
    graph: &mut StaticGraph,
    mutation: GraphMutation,
    options: &MutatorOptions,
    registry: Option<&ExtensionMutationRegistry>,
) -> Result<(), MutationError> {
    match mutation {
        GraphMutation::AddNode(params) => apply_add_node(graph, params, options),
        GraphMutation::AddNodegroup(params) => apply_add_nodegroup(graph, params, options),
        GraphMutation::AddEdge(params) => apply_add_edge(graph, params),
        GraphMutation::AddCard(params) => apply_add_card(graph, params),
        GraphMutation::AddWidgetToCard(params) => apply_add_widget(graph, params),
        GraphMutation::AddSubgraph(params) => apply_add_subgraph(graph, params),
        GraphMutation::UpdateSubgraph(params) => apply_update_subgraph(graph, params),
        GraphMutation::ConceptChangeCollection(params) => apply_concept_change_collection(graph, params),
        GraphMutation::DeleteCard(params) => apply_delete_card(graph, params),
        GraphMutation::DeleteWidget(params) => apply_delete_widget(graph, params),
        GraphMutation::DeleteFunction(params) => apply_delete_function(graph, params),
        GraphMutation::DeleteNode(params) => apply_delete_node(graph, params),
        GraphMutation::DeleteNodegroup(params) => apply_delete_nodegroup(graph, params),
        GraphMutation::UpdateNode(params) => apply_update_node(graph, params, options),
        GraphMutation::ChangeNodeType(params) => apply_change_node_type(graph, params),
        GraphMutation::ChangeCardinality(params) => apply_change_cardinality(graph, params),
        GraphMutation::RenameNode(params) => apply_rename_node(graph, params),
        GraphMutation::RenameGraph(params) => apply_rename_graph(graph, params),
        GraphMutation::CreateGraph(_) => {
            Err(MutationError::Other(
                "CreateGraph cannot be used as a regular mutation. Use apply_mutations_create_from_json instead.".to_string()
            ))
        }
        GraphMutation::Extension(params) => {
            match registry {
                Some(reg) => {
                    let handler = reg.get(&params.name)
                        .ok_or_else(|| MutationError::ExtensionNotFound(params.name.clone()))?;
                    handler.apply(graph, &params.params, options)
                }
                None => Err(MutationError::NoExtensionRegistry(params.name)),
            }
        }
    }
}

fn apply_add_node(
    graph: &mut StaticGraph,
    params: AddNodeParams,
    options: &MutatorOptions,
) -> Result<(), MutationError> {
    // Check for duplicate alias
    if graph.find_node_by_alias(&params.alias).is_some() {
        return Err(MutationError::AliasAlreadyExists(params.alias.clone()));
    }

    // Find parent node
    let parent = if let Some(ref parent_alias) = params.parent_alias {
        graph
            .find_node_by_alias(parent_alias)
            .ok_or_else(|| MutationError::ParentNotFound(parent_alias.clone()))?
    } else {
        graph.get_root()
    };
    let parent_nodeid = parent.nodeid.clone();
    let parent_nodegroup_id = parent.nodegroup_id.clone();
    let parent_ontologyclass = parent.ontologyclass.clone();

    // Validate ontology class and property if validator is present
    if let Some(ref validator) = options.ontology_validator {
        if !params.ontology_class.is_empty() {
            validator
                .validate_edge(
                    parent_ontologyclass.as_deref().unwrap_or(""),
                    &params.parent_property,
                    &params.ontology_class,
                )
                .map_err(MutationError::OntologyValidation)?;
        }
    }

    // Generate node ID
    let node_id = generate_uuid_v5(
        ("graph", Some(&graph.graphid)),
        &format!("node-{}", params.alias),
    );

    // Determine nodegroup
    // All nodes except root must have a nodegroup. Create a new one if:
    // - Cardinality is N (multiple instances allowed), OR
    // - Parent is root (direct children of root always get their own nodegroup)
    let (nodegroup_id, created_new_nodegroup) =
        if params.cardinality == Cardinality::N || parent.is_root() {
            // Create new nodegroup for this node
            let ng_id = node_id.clone();

            // Add nodegroup
            let nodegroup = StaticNodegroup {
                nodegroupid: ng_id.clone(),
                cardinality: Some(params.cardinality.as_str().to_string()),
                parentnodegroup_id: parent_nodegroup_id.clone(),
                legacygroupid: None,
                grouping_node_id: None,
            };
            graph.push_nodegroup(nodegroup);

            // Auto-create card if enabled
            if options.autocreate_card {
                let card_id = generate_uuid_v5(
                    ("graph", Some(&graph.graphid)),
                    &format!("card-ng-{}", ng_id),
                );
                let card = StaticCard {
                    active: true,
                    cardid: card_id,
                    component_id: DEFAULT_CARD_COMPONENT_ID.to_string(),
                    config: None,
                    constraints: vec![],
                    cssclass: None,
                    description: None,
                    graph_id: graph.graphid.clone(),
                    helpenabled: false,
                    helptext: StaticTranslatableString::empty(),
                    helptitle: StaticTranslatableString::empty(),
                    instructions: StaticTranslatableString::empty(),
                    is_editable: Some(true),
                    name: StaticTranslatableString::from_string(&params.name),
                    nodegroup_id: ng_id.clone(),
                    sortorder: Some(0),
                    visible: true,
                    source_identifier_id: None,
                };
                graph.push_card(card);
            }

            (Some(ng_id), true)
        } else {
            (parent_nodegroup_id, false)
        };

    // Build config - error if provided but invalid
    let config: HashMap<String, serde_json::Value> = match params.config {
        Some(v) => serde_json::from_value(v).map_err(|e| MutationError::InvalidConfig {
            alias: params.alias.clone(),
            error: e.to_string(),
        })?,
        None => HashMap::new(),
    };

    // Create node
    let node = StaticNode {
        nodeid: node_id.clone(),
        name: params.name.clone(),
        alias: Some(params.alias.clone()),
        datatype: params.datatype.clone(),
        nodegroup_id: nodegroup_id.clone(),
        graph_id: graph.graphid.clone(),
        is_collector: params.options.is_collector.unwrap_or(false),
        isrequired: params.options.isrequired.unwrap_or(false),
        exportable: params.options.exportable.unwrap_or(false),
        sortorder: Some(params.options.sortorder.unwrap_or(0)),
        config,
        parentproperty: Some(params.parent_property.clone()),
        ontologyclass: Some(params.ontology_class),
        description: params
            .description
            .map(|d| StaticTranslatableString::from_string(&d)),
        fieldname: params.options.fieldname,
        hascustomalias: params.options.hascustomalias.unwrap_or(false),
        issearchable: params.options.issearchable.unwrap_or(true),
        istopnode: params.options.istopnode.unwrap_or(false),
        sourcebranchpublication_id: None,
        source_identifier_id: None,
        is_immutable: None,
    };
    graph.push_node(node);

    // Create edge from parent to new node
    let edge_id = generate_uuid_v5(
        ("graph", Some(&graph.graphid)),
        &format!("edge-{}-{}", parent_nodeid, node_id),
    );
    let edge = StaticEdge {
        domainnode_id: parent_nodeid,
        rangenode_id: node_id.clone(),
        edgeid: edge_id,
        graph_id: graph.graphid.clone(),
        name: None,
        ontologyproperty: Some(params.parent_property),
        description: None,
        source_identifier_id: None,
    };
    graph.push_edge(edge);

    // Auto-create widget if enabled and not semantic
    if options.autocreate_widget && params.datatype != "semantic" {
        let widget = get_default_widget_for_datatype(&params.datatype)
            .map_err(|_| MutationError::NoWidgetForDatatype(params.datatype.clone()))?;

        let ng_id = nodegroup_id.as_ref().ok_or_else(|| {
            MutationError::Other(format!(
                "Cannot create widget for node '{}': no nodegroup",
                params.alias
            ))
        })?;

        // If we created a new nodegroup, a card must exist (created above if autocreate_card).
        // If inheriting an existing nodegroup that has no card, silently skip widget creation.
        if let Some(card) = graph.find_card_by_nodegroup(ng_id) {
            let mut widget_config = widget.get_default_config();
            if let serde_json::Value::Object(ref mut map) = widget_config {
                map.insert(
                    "label".to_string(),
                    serde_json::Value::String(params.name.clone()),
                );
            }

            let cxnxw_id = generate_uuid_v5(
                ("graph", Some(&graph.graphid)),
                &format!("cxnxw-{}-{}", node_id, widget.id),
            );

            let cxnxw = StaticCardsXNodesXWidgets {
                card_id: card.cardid.clone(),
                config: widget_config,
                id: cxnxw_id,
                label: StaticTranslatableString::from_string(&params.name),
                node_id: node_id.clone(),
                sortorder: Some(params.options.sortorder.unwrap_or(0)),
                visible: true,
                widget_id: widget.id.clone(),
                source_identifier_id: None,
            };
            graph.push_card_x_node_x_widget(cxnxw);
        } else if created_new_nodegroup {
            // We created a new nodegroup but it has no card - this shouldn't happen
            // if autocreate_card is true, so this is an error
            return Err(MutationError::CardNotFound(ng_id.clone()));
        }
        // else: inherited nodegroup with no card, silently skip widget creation
    }

    Ok(())
}

fn apply_add_nodegroup(
    graph: &mut StaticGraph,
    params: AddNodegroupParams,
    options: &MutatorOptions,
) -> Result<(), MutationError> {
    // Find parent
    let parent_nodegroup_id = if let Some(ref parent_alias) = params.parent_alias {
        let parent = graph
            .find_node_by_alias(parent_alias)
            .ok_or_else(|| MutationError::ParentNotFound(parent_alias.clone()))?;
        parent.nodegroup_id.clone()
    } else {
        graph.get_root().nodegroup_id.clone()
    };

    let nodegroup = StaticNodegroup {
        nodegroupid: params.nodegroup_id.clone(),
        cardinality: Some(params.cardinality.as_str().to_string()),
        parentnodegroup_id: parent_nodegroup_id,
        legacygroupid: None,
        grouping_node_id: None,
    };
    graph.push_nodegroup(nodegroup);

    // Auto-create card if enabled
    if options.autocreate_card {
        let card_id = generate_uuid_v5(
            ("graph", Some(&graph.graphid)),
            &format!("card-ng-{}", params.nodegroup_id),
        );
        let card = StaticCard {
            active: true,
            cardid: card_id,
            component_id: DEFAULT_CARD_COMPONENT_ID.to_string(),
            config: None,
            constraints: vec![],
            cssclass: None,
            description: None,
            graph_id: graph.graphid.clone(),
            helpenabled: false,
            helptext: StaticTranslatableString::empty(),
            helptitle: StaticTranslatableString::empty(),
            instructions: StaticTranslatableString::empty(),
            is_editable: Some(true),
            name: StaticTranslatableString::from_string("(unnamed)"),
            nodegroup_id: params.nodegroup_id,
            sortorder: Some(0),
            visible: true,
            source_identifier_id: None,
        };
        graph.push_card(card);
    }

    Ok(())
}

fn apply_add_edge(graph: &mut StaticGraph, params: AddEdgeParams) -> Result<(), MutationError> {
    let edge_id = generate_uuid_v5(
        ("graph", Some(&graph.graphid)),
        &format!("edge-{}-{}", params.from_node_id, params.to_node_id),
    );

    let edge = StaticEdge {
        domainnode_id: params.from_node_id,
        rangenode_id: params.to_node_id,
        edgeid: edge_id,
        graph_id: graph.graphid.clone(),
        name: params.name,
        ontologyproperty: Some(params.ontology_property),
        description: params.description,
        source_identifier_id: None,
    };
    graph.push_edge(edge);

    Ok(())
}

fn apply_add_card(graph: &mut StaticGraph, params: AddCardParams) -> Result<(), MutationError> {
    // Check if card already exists for this nodegroup
    if graph.find_card_by_nodegroup(&params.nodegroup_id).is_some() {
        return Err(MutationError::CardAlreadyExists(params.nodegroup_id));
    }

    let card_id = generate_uuid_v5(
        ("graph", Some(&graph.graphid)),
        &format!("card-ng-{}", params.nodegroup_id),
    );

    let card = StaticCard {
        active: params.options.active.unwrap_or(true),
        cardid: card_id,
        component_id: params
            .component_id
            .unwrap_or_else(|| DEFAULT_CARD_COMPONENT_ID.to_string()),
        config: params.config,
        constraints: vec![],
        cssclass: params.options.cssclass,
        description: params.options.description,
        graph_id: graph.graphid.clone(),
        helpenabled: params.options.helpenabled.unwrap_or(false),
        helptext: params
            .options
            .helptext
            .unwrap_or_else(StaticTranslatableString::empty),
        helptitle: params
            .options
            .helptitle
            .unwrap_or_else(StaticTranslatableString::empty),
        instructions: params
            .options
            .instructions
            .unwrap_or_else(StaticTranslatableString::empty),
        is_editable: params.options.is_editable,
        name: params.name,
        nodegroup_id: params.nodegroup_id,
        sortorder: Some(params.options.sortorder.unwrap_or(0)),
        visible: params.options.visible.unwrap_or(true),
        source_identifier_id: None,
    };
    graph.push_card(card);

    Ok(())
}

fn apply_add_widget(graph: &mut StaticGraph, params: AddWidgetParams) -> Result<(), MutationError> {
    // Find the node to get its nodegroup
    let node = graph
        .nodes
        .iter()
        .find(|n| n.nodeid == params.node_id)
        .ok_or_else(|| MutationError::NodeNotFound(params.node_id.clone()))?;
    let nodegroup_id = node
        .nodegroup_id
        .clone()
        .ok_or_else(|| MutationError::NodegroupNotFound(params.node_id.clone()))?;

    // Find card for this nodegroup
    let card = graph
        .find_card_by_nodegroup(&nodegroup_id)
        .ok_or_else(|| MutationError::CardNotFound(nodegroup_id.clone()))?;
    let card_id = card.cardid.clone();

    let cxnxw_id = generate_uuid_v5(
        ("graph", Some(&graph.graphid)),
        &format!("cxnxw-{}-{}", params.node_id, params.widget_id),
    );

    let cxnxw = StaticCardsXNodesXWidgets {
        card_id,
        config: params.config,
        id: cxnxw_id,
        label: StaticTranslatableString::from_string(&params.label),
        node_id: params.node_id,
        sortorder: Some(params.sortorder.unwrap_or(0)),
        visible: params.visible.unwrap_or(true),
        widget_id: params.widget_id,
        source_identifier_id: None,
    };
    graph.push_card_x_node_x_widget(cxnxw);

    Ok(())
}

/// Valid datatypes for ConceptChangeCollection operation
const CONCEPT_DATATYPES: &[&str] = &["concept", "concept-list"];

fn apply_concept_change_collection(
    graph: &mut StaticGraph,
    params: ConceptChangeCollectionParams,
) -> Result<(), MutationError> {
    // Find node by alias first, then by ID
    let node = graph
        .find_node_by_alias(&params.node_id)
        .or_else(|| graph.nodes.iter().find(|n| n.nodeid == params.node_id))
        .ok_or_else(|| MutationError::NodeNotFound(params.node_id.clone()))?;

    // Validate datatype is concept or concept-list
    if !CONCEPT_DATATYPES.contains(&node.datatype.as_str()) {
        return Err(MutationError::InvalidDatatype {
            expected: "concept or concept-list".to_string(),
            found: node.datatype.clone(),
            node_id: params.node_id.clone(),
        });
    }

    let node_id = node.nodeid.clone();

    // Find the mutable node and update its config
    let node_mut = graph
        .nodes
        .iter_mut()
        .find(|n| n.nodeid == node_id)
        .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

    // Update rdmCollection in config
    node_mut.config.insert(
        "rdmCollection".to_string(),
        serde_json::Value::String(params.collection_id),
    );

    Ok(())
}

// =============================================================================
// Deletion Operations
// =============================================================================

fn apply_delete_card(
    graph: &mut StaticGraph,
    params: DeleteCardParams,
) -> Result<(), MutationError> {
    // Verify card exists
    let card_exists = graph
        .cards
        .as_ref()
        .map(|cards| cards.iter().any(|c| c.cardid == params.card_id))
        .unwrap_or(false);

    if !card_exists {
        return Err(MutationError::CardNotFound(params.card_id));
    }

    // Remove associated cards_x_nodes_x_widgets entries
    if let Some(ref mut cxnxws) = graph.cards_x_nodes_x_widgets {
        cxnxws.retain(|c| c.card_id != params.card_id);
    }

    // Remove the card
    if let Some(ref mut cards) = graph.cards {
        cards.retain(|c| c.cardid != params.card_id);
    }

    Ok(())
}

fn apply_delete_widget(
    graph: &mut StaticGraph,
    params: DeleteWidgetParams,
) -> Result<(), MutationError> {
    // Verify widget mapping exists
    let widget_exists = graph
        .cards_x_nodes_x_widgets
        .as_ref()
        .map(|cxnxws| cxnxws.iter().any(|c| c.id == params.widget_mapping_id))
        .unwrap_or(false);

    if !widget_exists {
        return Err(MutationError::WidgetNotFound(params.widget_mapping_id));
    }

    // Remove the widget mapping
    if let Some(ref mut cxnxws) = graph.cards_x_nodes_x_widgets {
        cxnxws.retain(|c| c.id != params.widget_mapping_id);
    }

    Ok(())
}

fn apply_delete_function(
    graph: &mut StaticGraph,
    params: DeleteFunctionParams,
) -> Result<(), MutationError> {
    // Verify function mapping exists
    let function_exists = graph
        .functions_x_graphs
        .as_ref()
        .map(|fxgs| fxgs.iter().any(|f| f.id == params.function_mapping_id))
        .unwrap_or(false);

    if !function_exists {
        return Err(MutationError::FunctionNotFound(params.function_mapping_id));
    }

    // Remove the function mapping
    if let Some(ref mut fxgs) = graph.functions_x_graphs {
        fxgs.retain(|f| f.id != params.function_mapping_id);
    }

    Ok(())
}

fn apply_delete_node(
    graph: &mut StaticGraph,
    params: DeleteNodeParams,
) -> Result<(), MutationError> {
    // Find node by alias first, then by ID
    let node = graph
        .find_node_by_alias(&params.node_id)
        .or_else(|| graph.nodes.iter().find(|n| n.nodeid == params.node_id))
        .ok_or_else(|| MutationError::NodeNotFound(params.node_id.clone()))?;

    // Check if this is a root node (cannot delete root)
    if node.istopnode {
        return Err(MutationError::CannotDeleteRootNode(params.node_id.clone()));
    }

    let root_id = node.nodeid.clone();

    // Cascade: collect all descendant nodes by traversing edges
    let mut nodes_to_delete = vec![root_id.clone()];
    let mut i = 0;
    while i < nodes_to_delete.len() {
        let current = &nodes_to_delete[i].clone();
        for edge in &graph.edges {
            if edge.domainnode_id == *current && !nodes_to_delete.contains(&edge.rangenode_id) {
                nodes_to_delete.push(edge.rangenode_id.clone());
            }
        }
        i += 1;
    }

    // Check none of the cascaded descendants is a root node
    for nid in &nodes_to_delete {
        if let Some(n) = graph.nodes.iter().find(|n| n.nodeid == *nid) {
            if n.istopnode {
                return Err(MutationError::CannotDeleteRootNode(nid.clone()));
            }
        }
    }

    // Collect nodegroups to delete: where a deleted node IS the collector
    let mut nodegroups_to_delete: Vec<String> = Vec::new();
    for nid in &nodes_to_delete {
        if let Some(n) = graph.nodes.iter().find(|n| n.nodeid == *nid) {
            if let Some(ref ng_id) = n.nodegroup_id {
                if *ng_id == n.nodeid && !nodegroups_to_delete.contains(ng_id) {
                    nodegroups_to_delete.push(ng_id.clone());
                }
            }
        }
    }

    // Remove associated cards_x_nodes_x_widgets entries
    if let Some(ref mut cxnxws) = graph.cards_x_nodes_x_widgets {
        cxnxws.retain(|c| !nodes_to_delete.contains(&c.node_id));
    }

    // Remove edges where any deleted node is domain or range
    graph.edges.retain(|e| {
        !nodes_to_delete.contains(&e.domainnode_id) && !nodes_to_delete.contains(&e.rangenode_id)
    });

    // Remove cards for the deleted nodegroups
    if let Some(ref mut cards) = graph.cards {
        cards.retain(|c| !nodegroups_to_delete.contains(&c.nodegroup_id));
    }

    // Remove the nodegroups
    graph
        .nodegroups
        .retain(|ng| !nodegroups_to_delete.contains(&ng.nodegroupid));

    // Remove the nodes
    graph.nodes.retain(|n| !nodes_to_delete.contains(&n.nodeid));

    // Invalidate cached indices after retain operations
    graph.invalidate_indices();

    Ok(())
}

fn apply_delete_nodegroup(
    graph: &mut StaticGraph,
    params: DeleteNodegroupParams,
) -> Result<(), MutationError> {
    // Verify nodegroup exists
    if !graph
        .nodegroups
        .iter()
        .any(|ng| ng.nodegroupid == params.nodegroup_id)
    {
        return Err(MutationError::NodegroupNotFound(params.nodegroup_id));
    }

    // Collect all nodegroups to delete (this nodegroup and all descendants)
    let mut nodegroups_to_delete: Vec<String> = vec![params.nodegroup_id.clone()];
    let mut i = 0;
    while i < nodegroups_to_delete.len() {
        let current_ng = nodegroups_to_delete[i].clone();
        // Find child nodegroups
        for ng in &graph.nodegroups {
            if ng.parentnodegroup_id.as_ref() == Some(&current_ng)
                && !nodegroups_to_delete.contains(&ng.nodegroupid)
            {
                nodegroups_to_delete.push(ng.nodegroupid.clone());
            }
        }
        i += 1;
    }

    // Collect all nodes in these nodegroups
    let nodes_to_delete: Vec<String> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.nodegroup_id
                .as_ref()
                .map(|ng| nodegroups_to_delete.contains(ng))
                .unwrap_or(false)
        })
        .map(|n| n.nodeid.clone())
        .collect();

    // Check if any node is a root node
    for node_id in &nodes_to_delete {
        if let Some(node) = graph.nodes.iter().find(|n| n.nodeid == *node_id) {
            if node.istopnode {
                return Err(MutationError::CannotDeleteRootNode(node_id.clone()));
            }
        }
    }

    // Remove cards_x_nodes_x_widgets for all nodes being deleted
    if let Some(ref mut cxnxws) = graph.cards_x_nodes_x_widgets {
        cxnxws.retain(|c| !nodes_to_delete.contains(&c.node_id));
    }

    // Remove edges referencing any deleted nodes
    graph.edges.retain(|e| {
        !nodes_to_delete.contains(&e.domainnode_id) && !nodes_to_delete.contains(&e.rangenode_id)
    });

    // Remove cards for the deleted nodegroups
    if let Some(ref mut cards) = graph.cards {
        cards.retain(|c| !nodegroups_to_delete.contains(&c.nodegroup_id));
    }

    // Remove the nodegroups
    graph
        .nodegroups
        .retain(|ng| !nodegroups_to_delete.contains(&ng.nodegroupid));

    // Remove the nodes
    graph.nodes.retain(|n| !nodes_to_delete.contains(&n.nodeid));

    // Invalidate cached indices after retain operations
    graph.invalidate_indices();

    Ok(())
}

// =============================================================================
// Node Update Operations
// =============================================================================

fn apply_update_node(
    graph: &mut StaticGraph,
    params: UpdateNodeParams,
    options: &MutatorOptions,
) -> Result<(), MutationError> {
    // Find node by alias first, then by ID
    let node = graph
        .find_node_by_alias(&params.node_id)
        .or_else(|| graph.nodes.iter().find(|n| n.nodeid == params.node_id))
        .ok_or_else(|| MutationError::NodeNotFound(params.node_id.clone()))?;

    let node_id = node.nodeid.clone();

    // Validate ontology class if validator is present and class is being changed
    if let Some(ref validator) = options.ontology_validator {
        if let Some(ref new_class) = params.ontology_class {
            if !new_class.is_empty() && !validator.is_valid_class(new_class) {
                return Err(MutationError::OntologyValidation(
                    crate::ontology::OntologyValidationDetail::UnknownClass(new_class.clone()),
                ));
            }
        }
    }

    // Find the mutable node and update its fields
    let node_mut = graph
        .nodes
        .iter_mut()
        .find(|n| n.nodeid == node_id)
        .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

    // Update provided fields
    if let Some(name) = params.name {
        node_mut.name = name;
    }
    if let Some(ontology_class) = params.ontology_class {
        node_mut.ontologyclass = if ontology_class.is_empty() {
            None
        } else {
            Some(ontology_class)
        };
    }
    if let Some(parent_property) = params.parent_property {
        node_mut.parentproperty = if parent_property.is_empty() {
            None
        } else {
            Some(parent_property)
        };
    }
    if let Some(description) = params.description {
        node_mut.description = Some(StaticTranslatableString::from_string(&description));
    }
    if let Some(serde_json::Value::Object(map)) = params.config {
        // Merge config into existing
        for (k, v) in map {
            node_mut.config.insert(k, v);
        }
    }

    // Update options
    if let Some(exportable) = params.options.exportable {
        node_mut.exportable = exportable;
    }
    if let Some(fieldname) = params.options.fieldname {
        node_mut.fieldname = if fieldname.is_empty() {
            None
        } else {
            Some(fieldname)
        };
    }
    if let Some(isrequired) = params.options.isrequired {
        node_mut.isrequired = isrequired;
    }
    if let Some(issearchable) = params.options.issearchable {
        node_mut.issearchable = issearchable;
    }
    if let Some(sortorder) = params.options.sortorder {
        node_mut.sortorder = Some(sortorder);
    }

    Ok(())
}

fn apply_change_node_type(
    graph: &mut StaticGraph,
    params: ChangeNodeTypeParams,
) -> Result<(), MutationError> {
    // Find node by alias first, then by ID
    let node = graph
        .find_node_by_alias(&params.node_id)
        .or_else(|| graph.nodes.iter().find(|n| n.nodeid == params.node_id))
        .ok_or_else(|| MutationError::NodeNotFound(params.node_id.clone()))?;

    let node_id = node.nodeid.clone();

    // Check for dependent widgets
    let has_widgets = graph
        .cards_x_nodes_x_widgets
        .as_ref()
        .map(|cxnxws| cxnxws.iter().any(|c| c.node_id == node_id))
        .unwrap_or(false);

    if has_widgets {
        return Err(MutationError::NodeHasDependentWidgets(params.node_id));
    }

    // Find the mutable node and update its fields
    let node_mut = graph
        .nodes
        .iter_mut()
        .find(|n| n.nodeid == node_id)
        .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

    // Update datatype
    node_mut.datatype = params.datatype;

    // Update other provided fields (same as update_node)
    if let Some(name) = params.name {
        node_mut.name = name;
    }
    if let Some(ontology_class) = params.ontology_class {
        node_mut.ontologyclass = if ontology_class.is_empty() {
            None
        } else {
            Some(ontology_class)
        };
    }
    if let Some(parent_property) = params.parent_property {
        node_mut.parentproperty = if parent_property.is_empty() {
            None
        } else {
            Some(parent_property)
        };
    }
    if let Some(description) = params.description {
        node_mut.description = Some(StaticTranslatableString::from_string(&description));
    }
    if let Some(serde_json::Value::Object(map)) = params.config {
        // Merge config into existing
        for (k, v) in map {
            node_mut.config.insert(k, v);
        }
    }

    // Update options
    if let Some(exportable) = params.options.exportable {
        node_mut.exportable = exportable;
    }
    if let Some(fieldname) = params.options.fieldname {
        node_mut.fieldname = if fieldname.is_empty() {
            None
        } else {
            Some(fieldname)
        };
    }
    if let Some(isrequired) = params.options.isrequired {
        node_mut.isrequired = isrequired;
    }
    if let Some(issearchable) = params.options.issearchable {
        node_mut.issearchable = issearchable;
    }
    if let Some(sortorder) = params.options.sortorder {
        node_mut.sortorder = Some(sortorder);
    }

    Ok(())
}

fn apply_change_cardinality(
    graph: &mut StaticGraph,
    params: ChangeCardinalityParams,
) -> Result<(), MutationError> {
    // Find node by alias first, then by ID - extract what we need
    let (node_id, nodegroup_id) = {
        let node = graph
            .find_node_by_alias(&params.node_id)
            .or_else(|| graph.nodes.iter().find(|n| n.nodeid == params.node_id))
            .ok_or_else(|| MutationError::NodeNotFound(params.node_id.clone()))?;

        let nodegroup_id = node.nodegroup_id.clone().ok_or_else(|| {
            MutationError::Other(format!(
                "Node '{}' has no nodegroup_id - cannot change cardinality",
                params.node_id
            ))
        })?;

        (node.nodeid.clone(), nodegroup_id)
    };

    // Verify the node is the grouping node for the nodegroup
    // The grouping node is defined as:
    // 1. If nodegroup.grouping_node_id is set, the node must match it
    // 2. If not set, the node's nodegroup_id must equal the nodegroup's nodegroupid
    //    AND the nodegroupid must equal the node's nodeid (semantic node pattern)
    let nodegroup = graph
        .nodegroups
        .iter()
        .find(|ng| ng.nodegroupid == nodegroup_id)
        .ok_or_else(|| MutationError::NodegroupNotFound(nodegroup_id.clone()))?;

    let is_grouping_node = match &nodegroup.grouping_node_id {
        Some(grouping_id) => grouping_id == &node_id,
        None => {
            // If grouping_node_id not set, check if this is the semantic/grouping node pattern:
            // The grouping node typically has nodegroup_id == nodegroupid == nodeid
            nodegroup_id == node_id
        }
    };

    if !is_grouping_node {
        return Err(MutationError::Other(format!(
            "Node '{}' is not the grouping node for nodegroup '{}'. Only the grouping node can change cardinality.",
            params.node_id, nodegroup_id
        )));
    }

    // Find the nodegroup mutably and update its cardinality
    let nodegroup_mut = graph
        .nodegroups
        .iter_mut()
        .find(|ng| ng.nodegroupid == nodegroup_id)
        .ok_or_else(|| MutationError::NodegroupNotFound(nodegroup_id.clone()))?;

    nodegroup_mut.cardinality = Some(params.cardinality.as_str().to_string());

    Ok(())
}

fn apply_rename_node(
    graph: &mut StaticGraph,
    params: RenameNodeParams,
) -> Result<(), MutationError> {
    // Find node by alias first, then by ID
    let node = graph
        .find_node_by_alias(&params.node_id)
        .or_else(|| graph.nodes.iter().find(|n| n.nodeid == params.node_id))
        .ok_or_else(|| MutationError::NodeNotFound(params.node_id.clone()))?;

    let node_id = node.nodeid.clone();

    // Check if new alias already exists (if changing alias)
    if let Some(ref new_alias) = params.alias {
        // Check if another node already has this alias
        let alias_exists = graph
            .nodes
            .iter()
            .any(|n| n.nodeid != node_id && n.alias.as_ref() == Some(new_alias));
        if alias_exists {
            return Err(MutationError::AliasAlreadyExists(new_alias.clone()));
        }
    }

    // Find the mutable node and update text fields
    let node_mut = graph
        .nodes
        .iter_mut()
        .find(|n| n.nodeid == node_id)
        .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

    if let Some(alias) = params.alias {
        node_mut.alias = if alias.is_empty() { None } else { Some(alias) };
    }
    if let Some(name) = params.name {
        node_mut.name = name;
    }
    if let Some(description) = params.description {
        node_mut.description = Some(StaticTranslatableString::from_string(&description));
    }

    Ok(())
}

fn apply_rename_graph(
    graph: &mut StaticGraph,
    params: RenameGraphParams,
) -> Result<(), MutationError> {
    // Update name if provided
    if let Some(name_map) = params.name {
        let new_name = StaticTranslatableString::from_translations(name_map, None);

        // Update graph name
        graph.name = new_name.clone();

        // Also update root node name to match (root node name should equal graph name)
        let root_display_name = new_name.to_string_default();
        graph.root.name = root_display_name.clone();

        // Generate slug from name and update graph slug and root alias
        let new_slug = slugify(&root_display_name);
        graph.slug = Some(new_slug.clone());
        graph.root.alias = Some(new_slug.clone());

        // Update the root node in the nodes array as well
        if let Some(root_node) = graph.nodes.iter_mut().find(|n| n.istopnode) {
            root_node.name = root_display_name;
            root_node.alias = Some(new_slug);
        }
    }

    // Update description if provided
    if let Some(desc_map) = params.description {
        graph.description = Some(StaticTranslatableString::from_translations(desc_map, None));
    }

    // Update subtitle if provided
    if let Some(subtitle_map) = params.subtitle {
        graph.subtitle = Some(StaticTranslatableString::from_translations(
            subtitle_map,
            None,
        ));
    }

    // Update author if provided
    if let Some(author) = params.author {
        graph.author = if author.is_empty() {
            None
        } else {
            Some(author)
        };
    }

    Ok(())
}

// =============================================================================
// Subgraph Addition
// =============================================================================

/// Tracks ID remapping during subgraph addition
struct IdRemapper {
    /// Target graph ID for UUID generation
    graph_id: String,
    /// Suffix for UUID generation (used in deterministic ID creation)
    suffix: String,
    /// Branch publication ID to set on added nodes
    branch_publication_id: Option<String>,
    /// Maps old node ID -> new node ID
    node_map: HashMap<String, String>,
    /// Maps old nodegroup ID -> new nodegroup ID
    nodegroup_map: HashMap<String, String>,
    /// Maps old edge ID -> new edge ID
    edge_map: HashMap<String, String>,
    /// Maps old card ID -> new card ID
    card_map: HashMap<String, String>,
    /// Maps old cxnxw ID -> new cxnxw ID
    cxnxw_map: HashMap<String, String>,
    /// Maps old constraint ID -> new constraint ID
    constraint_map: HashMap<String, String>,
    /// Maps old alias -> new alias (only for clashing aliases that need suffixing)
    alias_map: HashMap<String, String>,
}

impl IdRemapper {
    fn new(graph_id: &str, suffix: Option<&str>, branch_publication_id: Option<String>) -> Self {
        Self {
            graph_id: graph_id.to_string(),
            suffix: suffix.unwrap_or("").to_string(),
            branch_publication_id,
            node_map: HashMap::new(),
            nodegroup_map: HashMap::new(),
            edge_map: HashMap::new(),
            card_map: HashMap::new(),
            cxnxw_map: HashMap::new(),
            constraint_map: HashMap::new(),
            alias_map: HashMap::new(),
        }
    }

    /// Generate a new node ID and store the mapping
    fn remap_node(&mut self, old_id: &str) -> String {
        let new_id = generate_uuid_v5(
            ("graph", Some(&self.graph_id)),
            &format!("subgraph-node-{}-{}", old_id, self.suffix),
        );
        self.node_map.insert(old_id.to_string(), new_id.clone());
        new_id
    }

    /// Generate a new nodegroup ID and store the mapping
    fn remap_nodegroup(&mut self, old_id: &str) -> String {
        let new_id = generate_uuid_v5(
            ("graph", Some(&self.graph_id)),
            &format!("subgraph-ng-{}-{}", old_id, self.suffix),
        );
        self.nodegroup_map
            .insert(old_id.to_string(), new_id.clone());
        new_id
    }

    /// Generate a new edge ID and store the mapping
    fn remap_edge(&mut self, old_id: &str) -> String {
        let new_id = generate_uuid_v5(
            ("graph", Some(&self.graph_id)),
            &format!("subgraph-edge-{}-{}", old_id, self.suffix),
        );
        self.edge_map.insert(old_id.to_string(), new_id.clone());
        new_id
    }

    /// Generate a new card ID and store the mapping
    fn remap_card(&mut self, old_id: &str) -> String {
        let new_id = generate_uuid_v5(
            ("graph", Some(&self.graph_id)),
            &format!("subgraph-card-{}-{}", old_id, self.suffix),
        );
        self.card_map.insert(old_id.to_string(), new_id.clone());
        new_id
    }

    /// Generate a new cxnxw ID and store the mapping
    fn remap_cxnxw(&mut self, old_id: &str) -> String {
        let new_id = generate_uuid_v5(
            ("graph", Some(&self.graph_id)),
            &format!("subgraph-cxnxw-{}-{}", old_id, self.suffix),
        );
        self.cxnxw_map.insert(old_id.to_string(), new_id.clone());
        new_id
    }

    /// Generate a new constraint ID and store the mapping
    fn remap_constraint(&mut self, old_id: &str) -> String {
        let new_id = generate_uuid_v5(
            ("graph", Some(&self.graph_id)),
            &format!("subgraph-constraint-{}-{}", old_id, self.suffix),
        );
        self.constraint_map
            .insert(old_id.to_string(), new_id.clone());
        new_id
    }

    /// Get the remapped node ID, or return the original if not mapped
    fn get_node(&self, old_id: &str) -> Option<&String> {
        self.node_map.get(old_id)
    }

    /// Get the remapped nodegroup ID, or return the original if not mapped
    fn get_nodegroup(&self, old_id: &str) -> Option<&String> {
        self.nodegroup_map.get(old_id)
    }

    /// Get the remapped card ID
    fn get_card(&self, old_id: &str) -> Option<&String> {
        self.card_map.get(old_id)
    }

    /// Register an alias mapping (for clashing aliases that need unique suffixes)
    fn register_alias(&mut self, old_alias: &str, new_alias: String) {
        self.alias_map.insert(old_alias.to_string(), new_alias);
    }

    /// Get the remapped alias, or return original if no clash was registered
    fn get_alias(&self, alias: Option<&str>) -> Option<String> {
        alias.map(|a| {
            self.alias_map
                .get(a)
                .cloned()
                .unwrap_or_else(|| a.to_string())
        })
    }
}

/// Make a name unique by appending _n1, _n2, etc. (matching Arches behavior)
fn make_name_unique(name: &str, existing: &HashSet<String>) -> String {
    if !existing.contains(name) {
        return name.to_string();
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}_n{}", name, counter);
        if !existing.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// Apply an AddSubgraph mutation to a graph
fn apply_add_subgraph(
    graph: &mut StaticGraph,
    params: AddSubgraphParams,
) -> Result<(), MutationError> {
    let subgraph = params.subgraph;
    let target_node_id = params.target_node_id;
    let ontology_property = params.ontology_property;
    let alias_suffix = params.alias_suffix;

    // Get the branch's publicationid for sourcebranchpublication_id tracking.
    // In Arches, sourcebranchpublication_id is an FK to graphs_x_published_graphs,
    // so it must be the publication's publicationid, not the branch's graphid.
    let branch_publication_id = subgraph
        .publication
        .as_ref()
        .and_then(|p| p.get("publicationid"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| MutationError::InvalidSubgraph(format!(
            "Subgraph '{}' has no publication.publicationid — the branch must be published before it can be added as a subgraph",
            subgraph.graphid
        )))?;

    // 1. VALIDATE: Target node exists (find by alias first, then by ID)
    let target_node = graph
        .find_node_by_alias(&target_node_id)
        .or_else(|| graph.nodes.iter().find(|n| n.nodeid == target_node_id))
        .ok_or_else(|| MutationError::NodeNotFound(target_node_id.clone()))?;
    let target_node_id = target_node.nodeid.clone();
    let target_nodegroup_id = target_node.nodegroup_id.clone();

    // 2. IDENTIFY ROOT: Find the root node of the subgraph
    let root_node = subgraph
        .nodes
        .iter()
        .find(|n| n.istopnode)
        .or(Some(&subgraph.root))
        .ok_or(MutationError::BranchHasNoRoot)?;
    let root_node_id = root_node.nodeid.clone();
    let root_nodegroup_id = root_node
        .nodegroup_id
        .clone()
        .unwrap_or_else(|| root_node_id.clone());

    // 3. BUILD ALIAS MAPPINGS (Arches-style: only suffix clashing aliases)
    // Collect all existing aliases in the target graph
    let mut existing_aliases: HashSet<String> =
        graph.nodes.iter().filter_map(|n| n.alias.clone()).collect();

    // Build the remapper with branch publication ID for tracking
    let suffix_ref = alias_suffix.as_deref();
    let mut remapper = IdRemapper::new(&graph.graphid, suffix_ref, Some(branch_publication_id));

    // Process aliases from subgraph nodes (excluding root)
    // Apply alias_prefix if set, then dedup with _n1, _n2 if clashing.
    for node in &subgraph.nodes {
        if node.nodeid == root_node_id {
            continue; // Skip root node
        }
        if let Some(ref alias) = node.alias {
            let prefixed_alias = if let Some(ref prefix) = params.alias_prefix {
                format!("{}_{}", prefix, alias)
            } else {
                alias.clone()
            };
            let new_alias = make_name_unique(&prefixed_alias, &existing_aliases);
            if new_alias != *alias {
                // Alias was changed (prefix and/or clash) - record the mapping
                remapper.register_alias(alias, new_alias.clone());
            }
            // Add to existing set so subsequent branch aliases don't clash with each other
            existing_aliases.insert(new_alias);
        }
    }

    // 4. BUILD ID MAPPINGS
    // Pre-generate all node mappings (excluding root)
    for node in &subgraph.nodes {
        if node.nodeid != root_node_id {
            remapper.remap_node(&node.nodeid);
        }
    }

    // Pre-generate all nodegroup mappings (excluding root's nodegroup)
    // In Arches, nodegroupid == groupingnodeid (the grouping node's nodeid).
    // If a nodegroup's ID matches a node's ID, use the node's remapped ID
    // so the constraint is preserved after remapping.
    for nodegroup in &subgraph.nodegroups {
        if nodegroup.nodegroupid != root_nodegroup_id {
            if let Some(node_id) = remapper.get_node(&nodegroup.nodegroupid) {
                // This nodegroup's ID matches a node ID - reuse the node's remapped ID
                let node_id = node_id.clone();
                remapper
                    .nodegroup_map
                    .insert(nodegroup.nodegroupid.clone(), node_id);
            } else {
                remapper.remap_nodegroup(&nodegroup.nodegroupid);
            }
        }
    }

    // Pre-generate edge mappings for edges NOT from root
    for edge in &subgraph.edges {
        if edge.domainnode_id != root_node_id {
            remapper.remap_edge(&edge.edgeid);
        }
    }

    // Pre-generate card mappings (including root-nodegroup cards, since
    // non-root CXNXWs may reference them)
    if let Some(ref cards) = subgraph.cards {
        for card in cards {
            remapper.remap_card(&card.cardid);
        }
    }

    // Pre-generate cxnxw mappings (excluding root's node)
    if let Some(ref cxnxws) = subgraph.cards_x_nodes_x_widgets {
        for cxnxw in cxnxws {
            if cxnxw.node_id != root_node_id {
                remapper.remap_cxnxw(&cxnxw.id);
            }
        }
    }

    // 5. ADD NODES (excluding root)
    for node in subgraph.nodes {
        if node.nodeid == root_node_id {
            continue;
        }

        let new_node_id = remapper
            .get_node(&node.nodeid)
            .ok_or_else(|| {
                MutationError::InvalidSubgraph(format!("Node {} not mapped", node.nodeid))
            })?
            .clone();

        // Remap nodegroup_id
        let new_nodegroup_id = node.nodegroup_id.as_ref().and_then(|ng_id| {
            if *ng_id == root_nodegroup_id {
                // Child of root's nodegroup -> use target's nodegroup
                target_nodegroup_id.clone()
            } else {
                remapper.get_nodegroup(ng_id).cloned()
            }
        });

        let prefixed_name = if let Some(ref prefix) = params.name_prefix {
            format!("{} {}", prefix, node.name)
        } else {
            node.name
        };

        let new_node = StaticNode {
            nodeid: new_node_id,
            name: prefixed_name,
            alias: remapper.get_alias(node.alias.as_deref()),
            datatype: node.datatype,
            nodegroup_id: new_nodegroup_id,
            graph_id: graph.graphid.clone(),
            is_collector: node.is_collector,
            isrequired: node.isrequired,
            exportable: node.exportable,
            sortorder: node.sortorder,
            config: node.config,
            parentproperty: node.parentproperty,
            ontologyclass: node.ontologyclass,
            description: node.description,
            fieldname: node.fieldname,
            hascustomalias: node.hascustomalias,
            issearchable: node.issearchable,
            istopnode: false, // Not a top node in the target graph
            // Set sourcebranchpublication_id to track which branch this node came from
            sourcebranchpublication_id: remapper.branch_publication_id.clone(),
            source_identifier_id: node.source_identifier_id,
            is_immutable: node.is_immutable,
        };
        graph.push_node(new_node);
    }

    // 6. ADD NODEGROUPS (excluding root's nodegroup)
    for nodegroup in subgraph.nodegroups {
        if nodegroup.nodegroupid == root_nodegroup_id {
            continue;
        }

        let new_ng_id = remapper
            .get_nodegroup(&nodegroup.nodegroupid)
            .ok_or_else(|| {
                MutationError::InvalidSubgraph(format!(
                    "Nodegroup {} not mapped",
                    nodegroup.nodegroupid
                ))
            })?
            .clone();

        // Remap parentnodegroup_id
        let new_parent_ng_id = nodegroup.parentnodegroup_id.as_ref().and_then(|parent_id| {
            if *parent_id == root_nodegroup_id {
                // Parent was root's nodegroup -> use target's nodegroup
                target_nodegroup_id.clone()
            } else {
                remapper.get_nodegroup(parent_id).cloned()
            }
        });

        // Remap grouping_node_id
        let new_grouping_node_id = nodegroup
            .grouping_node_id
            .as_ref()
            .and_then(|gn_id| remapper.get_node(gn_id).cloned());

        let new_nodegroup = StaticNodegroup {
            nodegroupid: new_ng_id,
            cardinality: nodegroup.cardinality,
            parentnodegroup_id: new_parent_ng_id,
            legacygroupid: nodegroup.legacygroupid,
            grouping_node_id: new_grouping_node_id,
        };
        graph.push_nodegroup(new_nodegroup);
    }

    // 7. ADD EDGES (excluding edges from root) and CREATE CONNECTING EDGES
    for edge in subgraph.edges {
        if edge.domainnode_id == root_node_id {
            // This is an edge from root to a child - create a new edge from target to child
            let child_node_id = remapper
                .get_node(&edge.rangenode_id)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!(
                        "Child node {} not mapped",
                        edge.rangenode_id
                    ))
                })?
                .clone();

            let new_edge_id = generate_uuid_v5(
                ("graph", Some(&graph.graphid)),
                &format!(
                    "subgraph-connect-{}-{}-{}",
                    target_node_id, child_node_id, remapper.suffix
                ),
            );

            let new_edge = StaticEdge {
                edgeid: new_edge_id,
                domainnode_id: target_node_id.clone(),
                rangenode_id: child_node_id,
                graph_id: graph.graphid.clone(),
                name: edge.name,
                ontologyproperty: if ontology_property.is_empty() {
                    edge.ontologyproperty
                } else {
                    Some(ontology_property.clone())
                },
                description: edge.description,
                source_identifier_id: None,
            };
            graph.push_edge(new_edge);
        } else {
            // Regular edge within the subgraph
            let new_edge_id = remapper
                .edge_map
                .get(&edge.edgeid)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!("Edge {} not mapped", edge.edgeid))
                })?
                .clone();

            let new_domain = remapper
                .get_node(&edge.domainnode_id)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!(
                        "Domain node {} not mapped",
                        edge.domainnode_id
                    ))
                })?
                .clone();

            let new_range = remapper
                .get_node(&edge.rangenode_id)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!(
                        "Range node {} not mapped",
                        edge.rangenode_id
                    ))
                })?
                .clone();

            let new_edge = StaticEdge {
                edgeid: new_edge_id,
                domainnode_id: new_domain,
                rangenode_id: new_range,
                graph_id: graph.graphid.clone(),
                name: edge.name,
                ontologyproperty: edge.ontologyproperty,
                description: edge.description,
                source_identifier_id: None,
            };
            graph.push_edge(new_edge);
        }
    }

    // 8. ADD CARDS (root-nodegroup cards get reassigned to the target's nodegroup)
    if let Some(cards) = subgraph.cards {
        for card in cards {
            let new_card_id = remapper
                .get_card(&card.cardid)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!("Card {} not mapped", card.cardid))
                })?
                .clone();

            let new_ng_id = if card.nodegroup_id == root_nodegroup_id {
                // Root-nodegroup card gets reassigned to the target node's nodegroup
                target_nodegroup_id
                    .clone()
                    .unwrap_or_else(|| target_node_id.clone())
            } else {
                remapper
                    .get_nodegroup(&card.nodegroup_id)
                    .ok_or_else(|| {
                        MutationError::InvalidSubgraph(format!(
                            "Card nodegroup {} not mapped",
                            card.nodegroup_id
                        ))
                    })?
                    .clone()
            };

            // Remap constraints
            let new_constraints: Vec<_> = card
                .constraints
                .into_iter()
                .map(|c| {
                    let new_constraint_id = remapper.remap_constraint(&c.constraintid);
                    let new_nodes: Vec<_> = c
                        .nodes
                        .into_iter()
                        .filter_map(|n| remapper.get_node(&n).cloned())
                        .collect();
                    crate::graph::StaticConstraint {
                        card_id: new_card_id.clone(),
                        constraintid: new_constraint_id,
                        nodes: new_nodes,
                        uniquetoallinstances: c.uniquetoallinstances,
                    }
                })
                .collect();

            let new_card = StaticCard {
                active: card.active,
                cardid: new_card_id,
                component_id: card.component_id, // Preserve external ID
                config: card.config,
                constraints: new_constraints,
                cssclass: card.cssclass,
                description: card.description,
                graph_id: graph.graphid.clone(),
                helpenabled: card.helpenabled,
                helptext: card.helptext,
                helptitle: card.helptitle,
                instructions: card.instructions,
                is_editable: card.is_editable,
                name: card.name,
                nodegroup_id: new_ng_id,
                sortorder: Some(card.sortorder.unwrap_or(0)),
                visible: card.visible,
                source_identifier_id: None,
            };
            graph.push_card(new_card);
        }
    }

    // 9. ADD CARDS_X_NODES_X_WIDGETS (excluding root's)
    if let Some(cxnxws) = subgraph.cards_x_nodes_x_widgets {
        for cxnxw in cxnxws {
            if cxnxw.node_id == root_node_id {
                continue;
            }

            let new_id = remapper
                .cxnxw_map
                .get(&cxnxw.id)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!("CXNXW {} not mapped", cxnxw.id))
                })?
                .clone();

            let new_card_id = remapper
                .get_card(&cxnxw.card_id)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!(
                        "CXNXW card {} not mapped",
                        cxnxw.card_id
                    ))
                })?
                .clone();

            let new_node_id = remapper
                .get_node(&cxnxw.node_id)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!(
                        "CXNXW node {} not mapped",
                        cxnxw.node_id
                    ))
                })?
                .clone();

            let new_cxnxw = StaticCardsXNodesXWidgets {
                id: new_id,
                card_id: new_card_id,
                node_id: new_node_id,
                widget_id: cxnxw.widget_id, // Preserve external ID
                config: cxnxw.config,
                label: cxnxw.label,
                sortorder: Some(cxnxw.sortorder.unwrap_or(0)),
                visible: cxnxw.visible,
                source_identifier_id: None,
            };
            graph.push_card_x_node_x_widget(new_cxnxw);
        }
    }

    Ok(())
}

/// Apply an UpdateSubgraph mutation to a graph
///
/// This finds nodes previously added from a branch by traversing edges from
/// the target node and using sourcebranchpublication_id for validation.
fn apply_update_subgraph(
    graph: &mut StaticGraph,
    params: UpdateSubgraphParams,
) -> Result<(), MutationError> {
    let subgraph = params.subgraph;
    let target_node_id = params.target_node_id.clone();
    let ontology_property = params.ontology_property;
    let remove_orphaned = params.remove_orphaned;

    // Get the branch's publicationid for sourcebranchpublication_id tracking
    let branch_publication_id = subgraph
        .publication
        .as_ref()
        .and_then(|p| p.get("publicationid"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| MutationError::InvalidSubgraph(format!(
            "Subgraph '{}' has no publication.publicationid — the branch must be published before it can be added as a subgraph",
            subgraph.graphid
        )))?;

    // 1. VALIDATE: Target node exists (find by alias first, then by ID)
    let target_node_ref = graph
        .find_node_by_alias(&target_node_id)
        .or_else(|| graph.nodes.iter().find(|n| n.nodeid == target_node_id))
        .ok_or_else(|| MutationError::NodeNotFound(target_node_id.clone()))?;
    let target_node_id = target_node_ref.nodeid.clone();

    // 2. IDENTIFY ROOT: Find the root node of the new subgraph
    let root_node = subgraph
        .nodes
        .iter()
        .find(|n| n.istopnode)
        .or(Some(&subgraph.root))
        .ok_or(MutationError::BranchHasNoRoot)?;
    let root_node_id = root_node.nodeid.clone();

    // 3. TRAVERSE: Find existing branch nodes by following edges from target
    // and validating sourcebranchpublication_id consistency
    let existing_branch_nodes =
        find_branch_nodes_by_traversal(graph, &target_node_id, &branch_publication_id)?;

    // If no nodes found, this might be a first-time add (fallback to AddSubgraph)
    if existing_branch_nodes.is_empty() {
        // No existing branch nodes - treat as new AddSubgraph
        return apply_add_subgraph(
            graph,
            AddSubgraphParams {
                subgraph,
                target_node_id: params.target_node_id,
                ontology_property,
                alias_suffix: params.alias_suffix,
                alias_prefix: params.alias_prefix,
                name_prefix: params.name_prefix,
            },
        );
    }

    // 4. BUILD MAPPING: Map branch aliases to existing node IDs
    // existing_branch_nodes: HashMap<nodeid, (alias, node)>
    let existing_by_alias: HashMap<String, String> = existing_branch_nodes
        .iter()
        .filter_map(|(node_id, alias)| alias.clone().map(|a| (a, node_id.clone())))
        .collect();

    // Collect aliases from new branch (excluding root)
    let new_branch_aliases: HashSet<String> = subgraph
        .nodes
        .iter()
        .filter(|n| n.nodeid != root_node_id)
        .filter_map(|n| n.alias.clone())
        .collect();

    // 5. CATEGORIZE NODES
    // - Update: alias exists in both existing and new branch
    // - Add: alias in new branch but not in existing (new nodes)
    // - Remove: alias in existing but not in new branch (orphaned nodes)

    let mut nodes_to_update: Vec<(&StaticNode, String)> = Vec::new(); // (new_node, existing_node_id)
    let mut nodes_to_add: Vec<&StaticNode> = Vec::new();

    for node in &subgraph.nodes {
        if node.nodeid == root_node_id {
            continue; // Skip root
        }
        if let Some(ref alias) = node.alias {
            // When alias_prefix is set, look for prefixed version in existing nodes
            // e.g. branch "name" matches existing "monument_name" with prefix "monument"
            let lookup_alias = if let Some(ref prefix) = params.alias_prefix {
                format!("{}_{}", prefix, alias)
            } else {
                alias.clone()
            };
            if let Some(existing_node_id) = existing_by_alias.get(&lookup_alias) {
                nodes_to_update.push((node, existing_node_id.clone()));
            } else {
                nodes_to_add.push(node);
            }
        } else {
            // Node without alias - treat as new
            nodes_to_add.push(node);
        }
    }

    // Build the set of expected existing aliases (with prefix applied) for orphan detection
    let expected_existing_aliases: HashSet<String> = if let Some(ref prefix) = params.alias_prefix {
        new_branch_aliases
            .iter()
            .map(|a| format!("{}_{}", prefix, a))
            .collect()
    } else {
        new_branch_aliases.clone()
    };

    let orphaned_node_ids: HashSet<String> = if remove_orphaned {
        existing_branch_nodes
            .iter()
            .filter(|(_, alias)| {
                alias
                    .as_ref()
                    .map(|a| !expected_existing_aliases.contains(a))
                    .unwrap_or(true)
            })
            .map(|(node_id, _)| node_id.clone())
            .collect()
    } else {
        HashSet::new()
    };

    // 6. UPDATE EXISTING NODES
    for (new_node, existing_node_id) in nodes_to_update {
        // Find and update the existing node
        if let Some(existing) = graph
            .nodes
            .iter_mut()
            .find(|n| n.nodeid == existing_node_id)
        {
            // Update mutable fields (preserve IDs and structural references)
            existing.name = if let Some(ref prefix) = params.name_prefix {
                format!("{} {}", prefix, new_node.name)
            } else {
                new_node.name.clone()
            };
            existing.datatype = new_node.datatype.clone();
            existing.ontologyclass = new_node.ontologyclass.clone();
            existing.config = new_node.config.clone();
            existing.description = new_node.description.clone();
            existing.isrequired = new_node.isrequired;
            existing.issearchable = new_node.issearchable;
            existing.exportable = new_node.exportable;
            existing.sortorder = new_node.sortorder;
            existing.is_collector = new_node.is_collector;
            existing.is_immutable = new_node.is_immutable;
            // Keep: nodeid, alias, nodegroup_id, graph_id, istopnode, sourcebranchpublication_id
        }
    }

    // 7. ADD NEW NODES (similar to AddSubgraph logic)
    if !nodes_to_add.is_empty() {
        // Need to get target nodegroup for new nodes
        let target_node = graph
            .nodes
            .iter()
            .find(|n| n.nodeid == target_node_id)
            .ok_or_else(|| MutationError::NodeNotFound(target_node_id.clone()))?;
        let target_nodegroup_id = target_node.nodegroup_id.clone();

        // Get root nodegroup from subgraph
        let root_nodegroup_id = root_node
            .nodegroup_id
            .clone()
            .unwrap_or_else(|| root_node_id.clone());

        // Build alias uniqueness set
        let mut existing_aliases: HashSet<String> =
            graph.nodes.iter().filter_map(|n| n.alias.clone()).collect();

        let suffix_ref = params.alias_suffix.as_deref();
        let mut remapper = IdRemapper::new(
            &graph.graphid,
            suffix_ref,
            Some(branch_publication_id.clone()),
        );

        // Register aliases for new nodes — apply prefix if set, then dedup if clashing
        for node in &nodes_to_add {
            if let Some(ref alias) = node.alias {
                let prefixed_alias = if let Some(ref prefix) = params.alias_prefix {
                    format!("{}_{}", prefix, alias)
                } else {
                    alias.clone()
                };
                let new_alias = make_name_unique(&prefixed_alias, &existing_aliases);
                if new_alias != *alias {
                    remapper.register_alias(alias, new_alias.clone());
                }
                existing_aliases.insert(new_alias);
            }
        }

        // Generate IDs for new nodes
        for node in &nodes_to_add {
            remapper.remap_node(&node.nodeid);
        }

        // Generate nodegroup IDs for nodegroups of new nodes
        let new_node_nodegroups: HashSet<String> = nodes_to_add
            .iter()
            .filter_map(|n| n.nodegroup_id.clone())
            .filter(|ng_id| *ng_id != root_nodegroup_id)
            .collect();

        for nodegroup in &subgraph.nodegroups {
            if new_node_nodegroups.contains(&nodegroup.nodegroupid) {
                remapper.remap_nodegroup(&nodegroup.nodegroupid);
            }
        }

        // Add the new nodes
        for node in nodes_to_add {
            let new_node_id = remapper
                .get_node(&node.nodeid)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!("Node {} not mapped", node.nodeid))
                })?
                .clone();

            let new_nodegroup_id = node.nodegroup_id.as_ref().and_then(|ng_id| {
                if *ng_id == root_nodegroup_id {
                    target_nodegroup_id.clone()
                } else {
                    remapper.get_nodegroup(ng_id).cloned()
                }
            });

            let prefixed_name = if let Some(ref prefix) = params.name_prefix {
                format!("{} {}", prefix, node.name)
            } else {
                node.name.clone()
            };

            let new_node = StaticNode {
                nodeid: new_node_id.clone(),
                name: prefixed_name,
                alias: remapper.get_alias(node.alias.as_deref()),
                datatype: node.datatype.clone(),
                nodegroup_id: new_nodegroup_id,
                graph_id: graph.graphid.clone(),
                is_collector: node.is_collector,
                isrequired: node.isrequired,
                exportable: node.exportable,
                sortorder: node.sortorder,
                config: node.config.clone(),
                parentproperty: node.parentproperty.clone(),
                ontologyclass: node.ontologyclass.clone(),
                description: node.description.clone(),
                fieldname: node.fieldname.clone(),
                hascustomalias: node.hascustomalias,
                issearchable: node.issearchable,
                istopnode: false,
                sourcebranchpublication_id: Some(branch_publication_id.clone()),
                source_identifier_id: node.source_identifier_id.clone(),
                is_immutable: node.is_immutable,
            };
            graph.push_node(new_node);

            // Create edge from target to new node
            let original_edge = subgraph
                .edges
                .iter()
                .find(|e| e.domainnode_id == root_node_id && e.rangenode_id == node.nodeid);
            let new_edge_id = generate_uuid_v5(
                ("graph", Some(&graph.graphid)),
                &format!("update-subgraph-edge-{}-{}", target_node_id, new_node_id),
            );
            let new_edge = StaticEdge {
                edgeid: new_edge_id,
                domainnode_id: target_node_id.clone(),
                rangenode_id: new_node_id,
                graph_id: graph.graphid.clone(),
                name: original_edge.and_then(|e| e.name.clone()),
                ontologyproperty: if ontology_property.is_empty() {
                    original_edge.and_then(|e| e.ontologyproperty.clone())
                } else {
                    Some(ontology_property.clone())
                },
                description: original_edge.and_then(|e| e.description.clone()),
                source_identifier_id: None,
            };
            graph.push_edge(new_edge);
        }

        // Add nodegroups for new nodes
        for nodegroup in subgraph.nodegroups {
            if !new_node_nodegroups.contains(&nodegroup.nodegroupid) {
                continue;
            }

            let new_ng_id = remapper
                .get_nodegroup(&nodegroup.nodegroupid)
                .ok_or_else(|| {
                    MutationError::InvalidSubgraph(format!(
                        "Nodegroup {} not mapped",
                        nodegroup.nodegroupid
                    ))
                })?
                .clone();

            let new_parent_ng_id = nodegroup.parentnodegroup_id.as_ref().and_then(|parent_id| {
                if *parent_id == root_nodegroup_id {
                    target_nodegroup_id.clone()
                } else {
                    remapper.get_nodegroup(parent_id).cloned()
                }
            });

            let new_grouping_node_id = nodegroup
                .grouping_node_id
                .as_ref()
                .and_then(|gn_id| remapper.get_node(gn_id).cloned());

            let new_nodegroup = StaticNodegroup {
                nodegroupid: new_ng_id,
                cardinality: nodegroup.cardinality,
                parentnodegroup_id: new_parent_ng_id,
                legacygroupid: nodegroup.legacygroupid,
                grouping_node_id: new_grouping_node_id,
            };
            graph.push_nodegroup(new_nodegroup);
        }
    }

    // 8. REMOVE ORPHANED NODES (if requested)
    if remove_orphaned && !orphaned_node_ids.is_empty() {
        // Remove nodes
        graph
            .nodes
            .retain(|n| !orphaned_node_ids.contains(&n.nodeid));

        // Remove edges referencing orphaned nodes
        graph.edges.retain(|e| {
            !orphaned_node_ids.contains(&e.domainnode_id)
                && !orphaned_node_ids.contains(&e.rangenode_id)
        });

        // Remove cards_x_nodes_x_widgets for orphaned nodes
        if let Some(ref mut cxnxws) = graph.cards_x_nodes_x_widgets {
            cxnxws.retain(|c| !orphaned_node_ids.contains(&c.node_id));
        }
    }

    Ok(())
}

/// Find branch nodes by traversing edges from target and validating sourcebranchpublication_id
fn find_branch_nodes_by_traversal(
    graph: &StaticGraph,
    target_node_id: &str,
    expected_branch_id: &str,
) -> Result<HashMap<String, Option<String>>, MutationError> {
    let mut branch_nodes: HashMap<String, Option<String>> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = Vec::new();

    // Find immediate children of target node
    for edge in &graph.edges {
        if edge.domainnode_id == target_node_id {
            queue.push(edge.rangenode_id.clone());
        }
    }

    while let Some(node_id) = queue.pop() {
        if visited.contains(&node_id) {
            continue;
        }
        visited.insert(node_id.clone());

        // Find the node
        let node = match graph.nodes.iter().find(|n| n.nodeid == node_id) {
            Some(n) => n,
            None => continue, // Edge points to non-existent node (shouldn't happen)
        };

        // Check sourcebranchpublication_id
        match &node.sourcebranchpublication_id {
            Some(pub_id) if pub_id == expected_branch_id => {
                // This node belongs to the branch - add it and explore its children
                branch_nodes.insert(node_id.clone(), node.alias.clone());

                // Add children to queue
                for edge in &graph.edges {
                    if edge.domainnode_id == node_id && !visited.contains(&edge.rangenode_id) {
                        queue.push(edge.rangenode_id.clone());
                    }
                }
            }
            Some(pub_id) => {
                // Different branch publication ID - error
                return Err(MutationError::InconsistentBranchPublication {
                    expected: expected_branch_id.to_string(),
                    found: Some(pub_id.clone()),
                    node_id: node_id.clone(),
                });
            }
            None => {
                // No sourcebranchpublication_id - this node wasn't added via branch
                // Stop traversing this path (it's part of original graph or different branch)
                continue;
            }
        }
    }

    Ok(branch_nodes)
}

// =============================================================================
// JSON-based Mutation API
// =============================================================================

/// Request structure for applying mutations via JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRequest {
    /// List of mutations to apply in order
    pub mutations: Vec<GraphMutation>,
    /// Options for mutation application
    #[serde(default)]
    pub options: MutationRequestOptions,
}

/// Options for JSON mutation requests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MutationRequestOptions {
    /// Automatically create cards for new nodegroups
    #[serde(default = "default_true")]
    pub autocreate_card: bool,
    /// Automatically add default widgets to cards
    #[serde(default = "default_true")]
    pub autocreate_widget: bool,
}

fn default_true() -> bool {
    true
}

impl From<MutationRequestOptions> for MutatorOptions {
    fn from(opts: MutationRequestOptions) -> Self {
        MutatorOptions {
            autocreate_card: opts.autocreate_card,
            autocreate_widget: opts.autocreate_widget,
            ontology_validator: None,
        }
    }
}

/// Apply mutations to a graph from a JSON string
///
/// # Arguments
/// * `graph` - The graph to mutate (will be cloned, original unchanged)
/// * `mutations_json` - JSON string containing a MutationRequest
///
/// # Returns
/// * `Ok(StaticGraph)` - The mutated graph
/// * `Err(String)` - Error message if parsing or mutation failed
///
/// # Example JSON
/// ```json
/// {
///   "mutations": [
///     {
///       "AddNode": {
///         "parent_alias": "root",
///         "alias": "child",
///         "name": "Child Node",
///         "cardinality": "N",
///         "datatype": "string",
///         "ontology_class": "E41_Appellation",
///         "parent_property": "P1_is_identified_by",
///         "description": null,
///         "config": null,
///         "options": {}
///       }
///     }
///   ],
///   "options": {
///     "autocreate_card": true,
///     "autocreate_widget": true
///   }
/// }
/// ```
pub fn apply_mutations_from_json(
    graph: &StaticGraph,
    mutations_json: &str,
) -> Result<StaticGraph, String> {
    apply_mutations_from_json_with_extensions(graph, mutations_json, None)
}

/// Apply mutations from JSON with extension support
///
/// Same as `apply_mutations_from_json` but allows extension mutations.
///
/// # Arguments
/// * `graph` - The graph to mutate
/// * `mutations_json` - JSON string containing the mutation request
/// * `registry` - Optional extension mutation registry
pub fn apply_mutations_from_json_with_extensions(
    graph: &StaticGraph,
    mutations_json: &str,
    registry: Option<&ExtensionMutationRegistry>,
) -> Result<StaticGraph, String> {
    // Parse the JSON request
    let request: MutationRequest = serde_json::from_str(mutations_json)
        .map_err(|e| format!("Failed to parse mutations JSON: {}", e))?;

    apply_mutations_with_extensions(graph, request.mutations, request.options.into(), registry)
}

/// Apply mutations that may start with a CreateGraph mutation
///
/// If `graph` is `None`, the first mutation must be `CreateGraph` which creates
/// a new graph from scratch. Remaining mutations are applied to the created graph.
///
/// If `graph` is `Some`, the first mutation must NOT be `CreateGraph`, and all
/// mutations are applied to the existing graph (delegating to the normal path).
///
/// # Arguments
/// * `mutations_json` - JSON string containing a MutationRequest
/// * `graph` - Optional existing graph. If None, first mutation must be CreateGraph.
///
/// # Returns
/// * `Ok(StaticGraph)` - The resulting graph
/// * `Err(String)` - Error message if mutation failed
pub fn apply_mutations_create_from_json(
    mutations_json: &str,
    graph: Option<&StaticGraph>,
) -> Result<StaticGraph, String> {
    let request: MutationRequest = serde_json::from_str(mutations_json)
        .map_err(|e| format!("Failed to parse mutations JSON: {}", e))?;

    let mut mutations = request.mutations;
    let options: MutatorOptions = request.options.into();

    match graph {
        None => {
            // No graph provided - first mutation must be CreateGraph
            if mutations.is_empty() {
                return Err("No graph provided and no mutations to apply".to_string());
            }

            let first = mutations.remove(0);
            match first {
                GraphMutation::CreateGraph(params) => {
                    // Create skeleton graph
                    let mut new_graph = create_skeleton_graph(
                        &params.name,
                        &params.root_alias,
                        params.is_resource,
                        Some(&params.root_ontology_class),
                    );

                    // Override graph ID if provided
                    if let Some(ref custom_id) = params.graph_id {
                        // Update graphid and all node graph_id references
                        let old_id = new_graph.graphid.clone();
                        new_graph.graphid = custom_id.clone();
                        for node in &mut new_graph.nodes {
                            if node.graph_id == old_id {
                                node.graph_id = custom_id.clone();
                            }
                        }
                        if new_graph.root.graph_id == old_id {
                            new_graph.root.graph_id = custom_id.clone();
                        }
                    }

                    // Set author if provided
                    if let Some(author) = params.author {
                        new_graph.author = Some(author);
                    }

                    // Set description if provided
                    if let Some(desc) = params.description {
                        new_graph.description = Some(StaticTranslatableString::from_string(&desc));
                    }

                    // Apply remaining mutations to the new graph
                    if mutations.is_empty() {
                        stamp_publication(&mut new_graph);
                        new_graph.build_indices();
                        Ok(new_graph)
                    } else {
                        apply_mutations_with_extensions(&new_graph, mutations, options, None)
                    }
                }
                _ => Err("No graph provided and first mutation is not CreateGraph".to_string()),
            }
        }
        Some(existing_graph) => {
            // Graph provided - first mutation must NOT be CreateGraph
            if let Some(GraphMutation::CreateGraph(_)) = mutations.first() {
                return Err("CreateGraph cannot be used when a graph already exists".to_string());
            }

            apply_mutations_with_extensions(existing_graph, mutations, options, None)
        }
    }
}

/// Apply a list of mutations to a graph
///
/// # Arguments
/// * `graph` - The graph to mutate (will be cloned, original unchanged)
/// * `mutations` - List of mutations to apply
/// * `options` - Options for mutation application
///
/// # Returns
/// * `Ok(StaticGraph)` - The mutated graph
/// * `Err(String)` - Error message if mutation failed
///
/// # Note
/// This function does not support extension mutations. For extension support,
/// use `apply_mutations_with_extensions` instead.
pub fn apply_mutations(
    graph: &StaticGraph,
    mutations: Vec<GraphMutation>,
    options: MutatorOptions,
) -> Result<StaticGraph, String> {
    apply_mutations_with_extensions(graph, mutations, options, None)
}

/// Apply a list of mutations to a graph with extension support
///
/// # Arguments
/// * `graph` - The graph to mutate (will be cloned, original unchanged)
/// * `mutations` - List of mutations to apply
/// * `options` - Options for mutation application
/// * `registry` - Optional extension mutation registry
///
/// # Returns
/// * `Ok(StaticGraph)` - The mutated graph
/// * `Err(String)` - Error message if mutation failed
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use alizarin_core::graph_mutator::{
///     apply_mutations_with_extensions, ExtensionMutationRegistry,
///     GraphMutation, ExtensionMutationParams, MutatorOptions,
/// };
///
/// let mut registry = ExtensionMutationRegistry::new();
/// registry.register("my_ext.custom_mutation", Arc::new(MyHandler));
///
/// let mutations = vec![
///     GraphMutation::Extension(ExtensionMutationParams {
///         name: "my_ext.custom_mutation".to_string(),
///         params: serde_json::json!({"key": "value"}),
///         conformance: MutationConformance::AlwaysConformant,
///     }),
/// ];
///
/// let result = apply_mutations_with_extensions(
///     &graph,
///     mutations,
///     MutatorOptions::default(),
///     Some(&registry),
/// )?;
/// ```
pub fn apply_mutations_with_extensions(
    graph: &StaticGraph,
    mutations: Vec<GraphMutation>,
    options: MutatorOptions,
    registry: Option<&ExtensionMutationRegistry>,
) -> Result<StaticGraph, String> {
    let mut result = graph.deep_clone();

    for mutation in mutations {
        apply_mutation_with_extensions(&mut result, mutation, &options, registry)
            .map_err(|e| e.to_string())?;
    }

    // Stamp publication after applying mutations
    stamp_publication(&mut result);

    result.build_indices();
    Ok(result)
}

/// Stamp a graph with a new publication entry.
///
/// Generates a deterministic `publicationid` (UUID5 from graphid + timestamp)
/// and sets `published_time` to the current UTC time. This ensures the graph
/// can be used as a subgraph source (add_subgraph requires a publicationid).
fn stamp_publication(graph: &mut StaticGraph) {
    let now = chrono::Utc::now();
    let timestamp = now.timestamp_millis().to_string();

    let publication_id = generate_uuid_v5(("publication", Some(&graph.graphid)), &timestamp);
    let published_time = now.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

    graph.publication = Some(serde_json::json!({
        "publicationid": publication_id,
        "graph_id": graph.graphid,
        "published_time": published_time,
        "notes": null
    }));
}

/// Serialize mutations to JSON
///
/// Useful for debugging or persisting mutation sequences
pub fn mutations_to_json(mutations: &[GraphMutation]) -> Result<String, String> {
    serde_json::to_string_pretty(mutations)
        .map_err(|e| format!("Failed to serialize mutations: {}", e))
}

// =============================================================================
// Skeleton Graph Creation
// =============================================================================

/// Create a minimal valid skeleton graph with just a root node
///
/// This creates a graph that can be built up from scratch using mutations.
/// The root node is created with:
/// - `istopnode: true`
/// - `nodegroup_id: None` (root has no nodegroup)
/// - `datatype: "semantic"`
///
/// # Arguments
/// * `name` - The name of the graph (used for display and UUID generation)
/// * `root_alias` - Alias for the root node (used to reference it in mutations)
/// * `is_resource` - Whether this is a resource model (true) or branch (false)
/// * `ontology_class` - Optional ontology class URI for the root node
///
/// # Example
/// ```rust,ignore
/// use alizarin_core::graph_mutator::{create_skeleton_graph, apply_mutations, Cardinality};
///
/// let graph = create_skeleton_graph("Person", "person", true, Some("http://example.org/Person"));
/// // Now add nodes using apply_mutations...
/// ```
pub fn create_skeleton_graph(
    name: &str,
    root_alias: &str,
    is_resource: bool,
    ontology_class: Option<&str>,
) -> StaticGraph {
    // Generate deterministic graph ID from name
    let graphid = generate_uuid_v5(("skeleton", None), name);

    // Generate root node ID
    let root_nodeid = generate_uuid_v5(("graph", Some(&graphid)), &format!("root-{}", root_alias));

    // For branches, the root must be a collector (Arches validation requirement).
    // In Arches, is_collector is a computed property: nodeid == nodegroup_id && nodegroup_id != null.
    // So for branches, the root's nodegroup_id must equal its nodeid.
    let root_nodegroup_id: serde_json::Value = if is_resource {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(root_nodeid.clone())
    };
    let root_is_collector = !is_resource;

    // Build graph as JSON to handle private fields correctly
    let graph_json = serde_json::json!({
        "graphid": graphid,
        "name": { "en": name },
        "isresource": is_resource,
        "is_editable": true,
        "config": {},
        "template_id": "50000000-0000-0000-0000-000000000001",
        "version": "1",
        "nodes": [{
            "nodeid": root_nodeid,
            "name": name,
            "alias": root_alias,
            "datatype": "semantic",
            "nodegroup_id": root_nodegroup_id,
            "graph_id": graphid,
            "is_collector": root_is_collector,
            "isrequired": false,
            "exportable": true,
            "sortorder": 0,
            "istopnode": true,
            "issearchable": true,
            "ontologyclass": ontology_class
        }],
        "root": {
            "nodeid": root_nodeid,
            "name": name,
            "alias": root_alias,
            "datatype": "semantic",
            "nodegroup_id": root_nodegroup_id,
            "graph_id": graphid,
            "is_collector": root_is_collector,
            "isrequired": false,
            "exportable": true,
            "sortorder": 0,
            "istopnode": true,
            "issearchable": true,
            "ontologyclass": ontology_class
        },
        "nodegroups": [],
        "edges": [],
        "cards": [],
        "cards_x_nodes_x_widgets": [],
        "functions_x_graphs": []
    });

    let mut graph: StaticGraph =
        serde_json::from_value(graph_json).expect("Failed to create skeleton graph");
    graph.build_indices();
    graph
}

// =============================================================================
// Instruction-based Graph Building (Triple-like DSL)
// =============================================================================

/// A row-based instruction for graph building
///
/// This provides a triple-like DSL for building graphs:
/// - `action`: The operation to perform (add_node, add_edge, etc.)
/// - `subject`: The source/parent entity (alias or ID)
/// - `object`: The target/new entity (alias, name, or ID)
/// - `params`: Additional parameters as key-value pairs
///
/// # Actions and their semantics
///
/// | Action | Subject | Object | Key Params |
/// |--------|---------|--------|------------|
/// | `create_model` | root_alias | graphid (optional) | `name`, `ontology_class`, `slug` |
/// | `create_branch` | root_alias | graphid (optional) | `name`, `ontology_class`, `slug` |
/// | `add_node` | parent_alias | new_alias | `datatype`, `name`, `ontology_class`, `cardinality`, `parent_property` |
/// | `add_edge` | domain_alias | range_alias | `ontology_property` |
/// | `add_nodegroup` | node_alias | (unused) | `cardinality` |
/// | `add_card` | nodegroup_id | card_name | `component_id` |
/// | `add_widget` | node_alias | (unused) | `widget_id` |
/// | `add_subgraph` | target_alias | (unused) | `subgraph` (JSON), `ontology_property`, `alias_suffix` |
/// | `update_subgraph` | target_alias | (unused) | `subgraph` (JSON), `ontology_property`, `alias_suffix`, `remove_orphaned` |
/// | `concept_change_collection` | node_alias | collection_id | (none) |
///
/// Note: `create_model` and `create_branch` must be the first instruction when using
/// `build_graph_from_instructions()`. They create a new graph rather than mutating one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInstruction {
    /// The action to perform
    pub action: String,
    /// The subject (parent/source entity alias)
    pub subject: String,
    /// The object (child/target entity alias or name)
    #[serde(default)]
    pub object: String,
    /// Additional parameters
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

impl GraphInstruction {
    /// Create a new instruction
    pub fn new(action: &str, subject: &str, object: &str) -> Self {
        Self {
            action: action.to_string(),
            subject: subject.to_string(),
            object: object.to_string(),
            params: HashMap::new(),
        }
    }

    /// Add a parameter
    pub fn with_param(mut self, key: &str, value: serde_json::Value) -> Self {
        self.params.insert(key.to_string(), value);
        self
    }

    /// Add a string parameter
    pub fn with_str(self, key: &str, value: &str) -> Self {
        self.with_param(key, serde_json::Value::String(value.to_string()))
    }

    /// Helper to get a string param
    fn get_str(&self, key: &str) -> Option<String> {
        self.params
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Helper to get a string param with default
    fn get_str_or(&self, key: &str, default: &str) -> String {
        self.get_str(key).unwrap_or_else(|| default.to_string())
    }

    /// Resolve a subgraph from either `params.subgraph` (inline JSON) or
    /// `object` (graph ID looked up from the global registry).
    fn resolve_subgraph(&self) -> Result<StaticGraph, MutationError> {
        if let Some(subgraph_value) = self.params.get("subgraph") {
            serde_json::from_value(subgraph_value.clone()).map_err(|e| {
                MutationError::InvalidSubgraph(format!("Failed to parse subgraph: {}", e))
            })
        } else if !self.object.is_empty() {
            let graph = crate::registry::get_graph(&self.object).ok_or_else(|| {
                MutationError::InvalidSubgraph(format!(
                    "Branch '{}' not found in graph registry",
                    self.object
                ))
            })?;
            Ok((*graph).clone())
        } else {
            Err(MutationError::InvalidSubgraph(
                "add_subgraph/update_subgraph requires either 'subgraph' param or a branch graph ID as object".to_string(),
            ))
        }
    }

    /// Helper to get a translatable map (language -> value) from params
    fn get_translatable_map(&self, key: &str) -> Option<HashMap<String, String>> {
        self.params.get(key).and_then(|v| {
            if let Some(obj) = v.as_object() {
                let map: HashMap<String, String> = obj
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect();
                if map.is_empty() {
                    None
                } else {
                    Some(map)
                }
            } else {
                None
            }
        })
    }

    /// Convert this instruction to a GraphMutation
    pub fn to_mutation(&self) -> Result<GraphMutation, MutationError> {
        match self.action.as_str() {
            "add_node" => {
                let cardinality_str = self.get_str_or("cardinality", "1");
                let cardinality = match cardinality_str.as_str() {
                    "1" | "one" | "One" => Cardinality::One,
                    "n" | "N" | "many" => Cardinality::N,
                    _ => {
                        return Err(MutationError::InvalidSubgraph(format!(
                            "Invalid cardinality: {}",
                            cardinality_str
                        )))
                    }
                };

                Ok(GraphMutation::AddNode(AddNodeParams {
                    parent_alias: if self.subject.is_empty() {
                        None
                    } else {
                        Some(self.subject.clone())
                    },
                    alias: self.object.clone(),
                    name: self.get_str_or("name", &self.object),
                    cardinality,
                    datatype: self.get_str_or("datatype", "semantic"),
                    ontology_class: self.get_str_or("ontology_class", ""),
                    parent_property: self.get_str_or("parent_property", ""),
                    description: self.get_str("description"),
                    config: self.params.get("config").cloned(),
                    options: NodeOptions {
                        is_collector: self
                            .params
                            .get("is_collector")
                            .and_then(|v| v.as_bool())
                            .or_else(|| {
                                // Default: semantic nodes with cardinality N are collectors
                                let dt = self.get_str_or("datatype", "semantic");
                                if dt == "semantic" && cardinality == Cardinality::N {
                                    Some(true)
                                } else {
                                    None
                                }
                            }),
                        exportable: self.params.get("exportable").and_then(|v| v.as_bool()),
                        isrequired: self.params.get("isrequired").and_then(|v| v.as_bool()),
                        issearchable: self.params.get("issearchable").and_then(|v| v.as_bool()),
                        sortorder: self
                            .params
                            .get("sortorder")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32),
                        ..NodeOptions::default()
                    },
                }))
            }
            "add_edge" => Ok(GraphMutation::AddEdge(AddEdgeParams {
                from_node_id: self.subject.clone(),
                to_node_id: self.object.clone(),
                ontology_property: self.get_str_or("ontology_property", ""),
                name: self.get_str("name"),
                description: self.get_str("description"),
            })),
            "add_nodegroup" => {
                let cardinality_str = self.get_str_or("cardinality", "n");
                let cardinality = match cardinality_str.as_str() {
                    "1" | "one" | "One" => Cardinality::One,
                    "n" | "N" | "many" => Cardinality::N,
                    _ => Cardinality::N,
                };

                // Generate nodegroup ID from subject alias
                let nodegroup_id = self
                    .get_str("nodegroup_id")
                    .unwrap_or_else(|| format!("ng-{}", self.subject));

                Ok(GraphMutation::AddNodegroup(AddNodegroupParams {
                    nodegroup_id,
                    cardinality,
                    parent_alias: if self.subject.is_empty() {
                        None
                    } else {
                        Some(self.subject.clone())
                    },
                }))
            }
            "add_card" => {
                let name = if self.object.is_empty() {
                    StaticTranslatableString::from_string("Card")
                } else {
                    StaticTranslatableString::from_string(&self.object)
                };
                Ok(GraphMutation::AddCard(AddCardParams {
                    nodegroup_id: self.subject.clone(),
                    name,
                    component_id: self.get_str("component_id"),
                    options: CardOptions {
                        description: self
                            .get_str("description")
                            .map(|s| StaticTranslatableString::from_string(&s)),
                        ..CardOptions::default()
                    },
                    config: self.params.get("config").cloned(),
                }))
            }
            "add_widget" => Ok(GraphMutation::AddWidgetToCard(AddWidgetParams {
                node_id: self.subject.clone(),
                widget_id: self.get_str_or("widget_id", "10000000-0000-0000-0000-000000000001"),
                label: self.get_str_or("label", ""),
                config: self
                    .params
                    .get("config")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                sortorder: self
                    .params
                    .get("sortorder")
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i32),
                visible: self.params.get("visible").and_then(|v| v.as_bool()),
            })),
            "add_subgraph" => {
                let subgraph = self.resolve_subgraph()?;

                Ok(GraphMutation::AddSubgraph(AddSubgraphParams {
                    subgraph,
                    target_node_id: self.subject.clone(),
                    ontology_property: self.get_str_or("ontology_property", ""),
                    alias_suffix: self.get_str("alias_suffix"),
                    alias_prefix: self.get_str("alias_prefix"),
                    name_prefix: self.get_str("name_prefix"),
                }))
            }
            "update_subgraph" => {
                let subgraph = self.resolve_subgraph()?;

                let remove_orphaned = self
                    .params
                    .get("remove_orphaned")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                Ok(GraphMutation::UpdateSubgraph(UpdateSubgraphParams {
                    subgraph,
                    target_node_id: self.subject.clone(),
                    ontology_property: self.get_str_or("ontology_property", ""),
                    alias_suffix: self.get_str("alias_suffix"),
                    remove_orphaned,
                    alias_prefix: self.get_str("alias_prefix"),
                    name_prefix: self.get_str("name_prefix"),
                }))
            }
            "concept_change_collection" => Ok(GraphMutation::ConceptChangeCollection(
                ConceptChangeCollectionParams {
                    node_id: self.subject.clone(),
                    collection_id: self.object.clone(),
                },
            )),
            "delete_card" => Ok(GraphMutation::DeleteCard(DeleteCardParams {
                card_id: self.subject.clone(),
            })),
            "delete_widget" => Ok(GraphMutation::DeleteWidget(DeleteWidgetParams {
                widget_mapping_id: self.subject.clone(),
            })),
            "delete_function" => Ok(GraphMutation::DeleteFunction(DeleteFunctionParams {
                function_mapping_id: self.subject.clone(),
            })),
            "delete_node" => Ok(GraphMutation::DeleteNode(DeleteNodeParams {
                node_id: self.subject.clone(),
            })),
            "delete_nodegroup" => Ok(GraphMutation::DeleteNodegroup(DeleteNodegroupParams {
                nodegroup_id: self.subject.clone(),
            })),
            "update_node" => Ok(GraphMutation::UpdateNode(UpdateNodeParams {
                node_id: self.subject.clone(),
                name: self.get_str("name"),
                ontology_class: self.get_str("ontology_class"),
                parent_property: self.get_str("parent_property"),
                description: self.get_str("description"),
                config: self.params.get("config").cloned(),
                options: UpdateNodeOptions {
                    exportable: self.params.get("exportable").and_then(|v| v.as_bool()),
                    fieldname: self.get_str("fieldname"),
                    isrequired: self.params.get("isrequired").and_then(|v| v.as_bool()),
                    issearchable: self.params.get("issearchable").and_then(|v| v.as_bool()),
                    sortorder: self
                        .params
                        .get("sortorder")
                        .and_then(|v| v.as_i64())
                        .map(|i| i as i32),
                },
            })),
            "change_node_type" => {
                let datatype = self
                    .get_str("datatype")
                    .or_else(|| {
                        if self.object.is_empty() {
                            None
                        } else {
                            Some(self.object.clone())
                        }
                    })
                    .ok_or_else(|| {
                        MutationError::InvalidSubgraph(
                            "change_node_type requires 'datatype' param or object".to_string(),
                        )
                    })?;

                Ok(GraphMutation::ChangeNodeType(ChangeNodeTypeParams {
                    node_id: self.subject.clone(),
                    datatype,
                    name: self.get_str("name"),
                    ontology_class: self.get_str("ontology_class"),
                    parent_property: self.get_str("parent_property"),
                    description: self.get_str("description"),
                    config: self.params.get("config").cloned(),
                    options: UpdateNodeOptions {
                        exportable: self.params.get("exportable").and_then(|v| v.as_bool()),
                        fieldname: self.get_str("fieldname"),
                        isrequired: self.params.get("isrequired").and_then(|v| v.as_bool()),
                        issearchable: self.params.get("issearchable").and_then(|v| v.as_bool()),
                        sortorder: self
                            .params
                            .get("sortorder")
                            .and_then(|v| v.as_i64())
                            .map(|i| i as i32),
                    },
                }))
            }
            "change_cardinality" => {
                let cardinality_str = self
                    .get_str("cardinality")
                    .or_else(|| {
                        if self.object.is_empty() {
                            None
                        } else {
                            Some(self.object.clone())
                        }
                    })
                    .ok_or_else(|| {
                        MutationError::InvalidSubgraph(
                            "change_cardinality requires 'cardinality' param or object (1 or n)"
                                .to_string(),
                        )
                    })?;

                let cardinality = match cardinality_str.to_lowercase().as_str() {
                    "1" | "one" => Cardinality::One,
                    "n" | "many" => Cardinality::N,
                    _ => {
                        return Err(MutationError::InvalidSubgraph(format!(
                            "Invalid cardinality '{}', expected '1', 'one', 'n', or 'many'",
                            cardinality_str
                        )))
                    }
                };

                Ok(GraphMutation::ChangeCardinality(ChangeCardinalityParams {
                    node_id: self.subject.clone(),
                    cardinality,
                }))
            }
            "rename_node" => Ok(GraphMutation::RenameNode(RenameNodeParams {
                node_id: self.subject.clone(),
                alias: self.get_str("alias").or_else(|| {
                    if self.object.is_empty() {
                        None
                    } else {
                        Some(self.object.clone())
                    }
                }),
                name: self.get_str("name"),
                description: self.get_str("description"),
            })),
            "rename_graph" => {
                // Parse name: either from params.name (as map) or object (as simple en string)
                let name = self.get_translatable_map("name").or_else(|| {
                    if self.object.is_empty() {
                        None
                    } else {
                        let mut map = HashMap::new();
                        map.insert("en".to_string(), self.object.clone());
                        Some(map)
                    }
                });
                Ok(GraphMutation::RenameGraph(RenameGraphParams {
                    name,
                    description: self.get_translatable_map("description"),
                    subtitle: self.get_translatable_map("subtitle"),
                    author: self.get_str("author"),
                }))
            }
            // create_model and create_branch are handled separately via to_skeleton_graph()
            "create_model" | "create_branch" => Err(MutationError::InvalidSubgraph(format!(
                "'{}' creates a new graph, use build_graph_from_instructions() instead",
                self.action
            ))),
            // Unrecognized actions are treated as extension mutations
            other => Ok(GraphMutation::Extension(ExtensionMutationParams {
                name: other.to_string(),
                params: {
                    let mut map = serde_json::Map::new();
                    // Pass subject and object as params for the extension handler
                    if !self.subject.is_empty() {
                        map.insert(
                            "subject".to_string(),
                            serde_json::Value::String(self.subject.clone()),
                        );
                    }
                    if !self.object.is_empty() {
                        map.insert(
                            "object".to_string(),
                            serde_json::Value::String(self.object.clone()),
                        );
                    }
                    // Merge all instruction params
                    for (k, v) in &self.params {
                        map.insert(k.clone(), v.clone());
                    }
                    serde_json::Value::Object(map)
                },
                conformance: MutationConformance::AlwaysConformant,
            })),
        }
    }

    /// Check if this instruction creates or loads a graph
    pub fn is_create_action(&self) -> bool {
        matches!(
            self.action.as_str(),
            "create_model" | "create_branch" | "load_graph"
        )
    }

    /// Get the conformance level for this instruction action
    pub fn conformance(&self) -> MutationConformance {
        match self.action.as_str() {
            // Basic structure operations - valid for branches
            "add_node" | "add_edge" | "add_nodegroup" | "add_card" | "add_widget" => {
                MutationConformance::BranchConformant
            }
            // Subgraph operations - only valid for models
            "add_subgraph" | "update_subgraph" => MutationConformance::ModelConformant,
            // Collection changes - valid for both
            "concept_change_collection" => MutationConformance::AlwaysConformant,
            // Deletion operations - valid for both branches and models
            "delete_card" | "delete_widget" | "delete_function" | "delete_node"
            | "delete_nodegroup" => MutationConformance::AlwaysConformant,
            // Node update operations
            "update_node" | "change_node_type" | "change_cardinality" => {
                MutationConformance::BranchConformant
            }
            "rename_node" | "rename_graph" => MutationConformance::AlwaysConformant,
            // Create operations
            "create_model" => MutationConformance::ModelConformant,
            "create_branch" => MutationConformance::BranchConformant,
            // Unknown actions
            _ => MutationConformance::NonConformant,
        }
    }

    /// Convert a create_model or create_branch instruction to a skeleton graph
    ///
    /// # Arguments
    /// - `subject`: root alias for the graph
    /// - `object`: graphid (if empty, generated from name)
    /// - `params.name`: graph name (defaults to subject if not provided)
    /// - `params.ontology_class`: optional ontology class for root node
    ///
    /// # Returns
    /// A new skeleton StaticGraph, or error if not a create action
    pub fn to_skeleton_graph(&self) -> Result<StaticGraph, MutationError> {
        let is_resource = match self.action.as_str() {
            "create_model" => true,
            "create_branch" => false,
            _ => {
                return Err(MutationError::InvalidSubgraph(format!(
                    "'{}' is not a create action, use to_mutation() instead",
                    self.action
                )))
            }
        };

        let root_alias = &self.subject;
        let name = self.get_str_or("name", root_alias);
        let ontology_class = self.get_str("ontology_class");

        // If object is provided, use it as the graphid override
        let mut graph =
            create_skeleton_graph(&name, root_alias, is_resource, ontology_class.as_deref());

        // Override graphid if object is provided and non-empty
        if !self.object.is_empty() {
            let new_graphid = self.object.clone();
            // Update graphid in graph and all nodes
            graph.graphid = new_graphid.clone();
            graph.root.graph_id = new_graphid.clone();
            for node in &mut graph.nodes {
                node.graph_id = new_graphid.clone();
            }
        }

        // Apply slug (both models and branches can have slugs)
        graph.slug = self
            .get_str("slug")
            .or_else(|| Some(root_alias.to_lowercase()));

        Ok(graph)
    }
}

/// Build a graph from scratch using instructions
///
/// The first instruction must be `create_model` or `create_branch` to create the skeleton.
/// Subsequent instructions are applied as mutations.
///
/// # Arguments
/// * `instructions` - List of instructions, first must be a create action
/// * `options` - Options for mutation application
///
/// # Returns
/// * `Ok(StaticGraph)` - The built graph
/// * `Err(String)` - Error message if build failed
///
/// # Example
/// ```rust,ignore
/// let instructions = vec![
///     GraphInstruction::new("create_model", "person", "")
///         .with_str("name", "Person"),
///     GraphInstruction::new("add_node", "person", "name")
///         .with_str("datatype", "string")
///         .with_str("cardinality", "n"),
/// ];
/// let graph = build_graph_from_instructions(instructions, MutatorOptions::default())?;
/// ```
pub fn build_graph_from_instructions(
    instructions: Vec<GraphInstruction>,
    options: MutatorOptions,
) -> Result<StaticGraph, String> {
    build_graph_from_instructions_with_extensions(instructions, options, None)
}

/// Build a graph from instructions with extension mutation support.
///
/// Same as `build_graph_from_instructions` but accepts an optional extension
/// mutation registry for handling custom/extension actions in the CSV.
pub fn build_graph_from_instructions_with_extensions(
    instructions: Vec<GraphInstruction>,
    options: MutatorOptions,
    registry: Option<&ExtensionMutationRegistry>,
) -> Result<StaticGraph, String> {
    if instructions.is_empty() {
        return Err("No instructions provided".to_string());
    }

    let mut iter = instructions.into_iter();
    let first = iter.next().unwrap();

    // First instruction must create or load the graph
    if !first.is_create_action() {
        return Err(format!(
            "First instruction must be 'create_model', 'create_branch', or 'load_graph', got '{}'",
            first.action
        ));
    }

    let graph = if first.action == "load_graph" {
        let graph_id = &first.subject;
        let arc = crate::registry::get_graph(graph_id).ok_or_else(|| {
            format!(
                "Graph '{}' not found in registry. Call register_graph() first.",
                graph_id
            )
        })?;
        (*arc).clone()
    } else {
        first.to_skeleton_graph().map_err(|e| e.to_string())?
    };

    // Apply remaining instructions as mutations
    let remaining: Vec<GraphInstruction> = iter.collect();
    if remaining.is_empty() {
        let mut graph = graph;
        stamp_publication(&mut graph);
        return Ok(graph);
    }

    apply_instructions(&graph, remaining, options, registry)
}

/// Build a graph from scratch using JSON instructions
///
/// # JSON Format
/// ```json
/// {
///   "instructions": [
///     { "action": "create_model", "subject": "person", "object": "", "params": { "name": "Person" } },
///     { "action": "add_node", "subject": "person", "object": "name", "params": { "datatype": "string" } }
///   ],
///   "options": { "autocreate_card": true, "autocreate_widget": true }
/// }
/// ```
pub fn build_graph_from_instructions_json(json: &str) -> Result<StaticGraph, String> {
    #[derive(Deserialize)]
    struct BuildRequest {
        instructions: Vec<GraphInstruction>,
        #[serde(default)]
        options: MutationRequestOptions,
    }

    let request: BuildRequest = serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse build request JSON: {}", e))?;

    build_graph_from_instructions(request.instructions, request.options.into())
}

/// Parse CSV text into a list of GraphInstructions.
///
/// Expected columns: `action`, `subject`, `object`, plus any `params.*` columns.
/// The first row is the header. Empty `params.*` values are ignored.
///
/// # Example CSV
/// ```csv
/// action,subject,object,params.name,params.datatype,params.ontology_class,params.parent_property
/// create_model,registry,,Registry,,,E78_Collection,
/// add_node,registry,names,Names,semantic,E41_Appellation,P1_is_identified_by
/// ```
pub fn parse_instructions_from_csv(csv_text: &str) -> Result<Vec<GraphInstruction>, String> {
    // Pre-filter comment lines (starting with #) before CSV parsing,
    // since they may have mismatched column counts
    let filtered: String = csv_text
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#')
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut reader = csv::Reader::from_reader(filtered.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| format!("Failed to parse CSV headers: {}", e))?
        .clone();

    let param_indices: Vec<(usize, String)> = headers
        .iter()
        .enumerate()
        .filter_map(|(i, h)| h.strip_prefix("params.").map(|p| (i, p.to_string())))
        .collect();

    let action_idx = headers
        .iter()
        .position(|h| h == "action")
        .ok_or("CSV missing 'action' column")?;
    let subject_idx = headers
        .iter()
        .position(|h| h == "subject")
        .ok_or("CSV missing 'subject' column")?;
    let object_idx = headers
        .iter()
        .position(|h| h == "object")
        .ok_or("CSV missing 'object' column")?;

    let mut instructions = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("Failed to parse CSV row: {}", e))?;

        let action = record.get(action_idx).unwrap_or("").to_string();
        if action.is_empty() || action.starts_with('#') {
            continue;
        }

        let mut params = serde_json::Map::new();
        for (idx, param_name) in &param_indices {
            if let Some(value) = record.get(*idx) {
                if !value.is_empty() {
                    // Try to parse as JSON first (for config objects), fall back to string
                    let json_value = serde_json::from_str(value)
                        .unwrap_or(serde_json::Value::String(value.to_string()));
                    params.insert(param_name.clone(), json_value);
                }
            }
        }

        instructions.push(GraphInstruction {
            action,
            subject: record.get(subject_idx).unwrap_or("").to_string(),
            object: record.get(object_idx).unwrap_or("").to_string(),
            params: serde_json::Value::Object(params)
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        });
    }

    Ok(instructions)
}

/// Build or mutate a graph from CSV instructions.
///
/// Parses CSV text into instructions, then builds or mutates a graph.
/// The first row must be `create_model`, `create_branch`, or `load_graph`.
///
/// Use `load_graph` with the graph ID as `subject` to load an existing graph
/// from the registry and apply subsequent instructions to it.
///
/// # Arguments
/// * `csv_text` - CSV string with header row
/// * `options` - Options for mutation application
pub fn build_graph_from_instructions_csv(
    csv_text: &str,
    options: MutatorOptions,
) -> Result<StaticGraph, String> {
    let instructions = parse_instructions_from_csv(csv_text)?;
    build_graph_from_instructions(instructions, options)
}

/// Apply a list of instructions to a graph
///
/// Instructions are converted to mutations and applied in order.
///
/// # Arguments
/// * `graph` - The graph to mutate (will be cloned, original unchanged)
/// * `instructions` - List of instructions to apply
/// * `options` - Options for mutation application
///
/// # Returns
/// * `Ok(StaticGraph)` - The mutated graph
/// * `Err(String)` - Error message if any instruction failed
pub fn apply_instructions(
    graph: &StaticGraph,
    instructions: Vec<GraphInstruction>,
    options: MutatorOptions,
    registry: Option<&ExtensionMutationRegistry>,
) -> Result<StaticGraph, String> {
    let mutations: Vec<GraphMutation> = instructions
        .into_iter()
        .map(|i| i.to_mutation())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    apply_mutations_with_extensions(graph, mutations, options, registry)
}

/// Apply instructions from JSON
///
/// # JSON Format
/// ```json
/// {
///   "instructions": [
///     { "action": "add_node", "subject": "root", "object": "name", "params": { "datatype": "string" } }
///   ],
///   "options": { "autocreate_card": true, "autocreate_widget": true }
/// }
/// ```
pub fn apply_instructions_from_json(
    graph: &StaticGraph,
    json: &str,
) -> Result<StaticGraph, String> {
    #[derive(Deserialize)]
    struct InstructionRequest {
        instructions: Vec<GraphInstruction>,
        #[serde(default)]
        options: MutationRequestOptions,
    }

    let request: InstructionRequest = serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse instructions JSON: {}", e))?;

    apply_instructions(graph, request.instructions, request.options.into(), None)
}

/// Get the JSON schema for mutation types (as documentation)
///
/// Returns a descriptive JSON object showing the structure of each mutation type
pub fn get_mutation_schema() -> serde_json::Value {
    serde_json::json!({
        "MutationRequest": {
            "description": "Container for a list of mutations to apply",
            "properties": {
                "mutations": {
                    "type": "array",
                    "items": { "$ref": "#/GraphMutation" }
                },
                "options": { "$ref": "#/MutationRequestOptions" }
            }
        },
        "MutationRequestOptions": {
            "properties": {
                "autocreate_card": { "type": "boolean", "default": true },
                "autocreate_widget": { "type": "boolean", "default": true }
            }
        },
        "GraphMutation": {
            "oneOf": [
                { "AddNode": { "$ref": "#/AddNodeParams" } },
                { "AddNodegroup": { "$ref": "#/AddNodegroupParams" } },
                { "AddEdge": { "$ref": "#/AddEdgeParams" } },
                { "AddCard": { "$ref": "#/AddCardParams" } },
                { "AddWidgetToCard": { "$ref": "#/AddWidgetParams" } },
                { "CreateGraph": { "$ref": "#/CreateGraphParams" } }
            ]
        },
        "CreateGraphParams": {
            "required": ["name", "is_resource", "root_alias", "root_ontology_class"],
            "properties": {
                "name": { "type": "string", "description": "Name for the graph" },
                "is_resource": { "type": "boolean", "description": "Whether this is a resource model (true) or branch (false)" },
                "root_alias": { "type": "string", "description": "Alias for the root node" },
                "root_ontology_class": { "type": "string", "description": "Ontology class URI for the root node" },
                "graph_id": { "type": "string", "nullable": true, "description": "Optional custom graph ID" },
                "author": { "type": "string", "nullable": true, "description": "Optional author" },
                "description": { "type": "string", "nullable": true, "description": "Optional description" }
            }
        },
        "AddNodeParams": {
            "required": ["alias", "name", "cardinality", "datatype", "ontology_class", "parent_property"],
            "properties": {
                "parent_alias": { "type": "string", "nullable": true },
                "alias": { "type": "string" },
                "name": { "type": "string" },
                "cardinality": { "enum": ["One", "N"] },
                "datatype": { "type": "string", "examples": ["semantic", "string", "number", "date", "boolean", "concept", "concept-list"] },
                "ontology_class": { "type": "string" },
                "parent_property": { "type": "string" },
                "description": { "type": "string", "nullable": true },
                "config": { "type": "object", "nullable": true },
                "options": { "$ref": "#/NodeOptions" }
            }
        },
        "NodeOptions": {
            "properties": {
                "exportable": { "type": "boolean" },
                "fieldname": { "type": "string" },
                "hascustomalias": { "type": "boolean" },
                "is_collector": { "type": "boolean" },
                "isrequired": { "type": "boolean" },
                "issearchable": { "type": "boolean" },
                "istopnode": { "type": "boolean" },
                "sortorder": { "type": "integer" }
            }
        },
        "Cardinality": {
            "enum": ["One", "N"],
            "description": "One = single instance only, N = multiple instances allowed"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> StaticGraph {
        let graph_json = r#"{
            "graphid": "test-graph-id",
            "name": {"en": "Test Graph"},
            "isresource": true,
            "is_editable": true,
            "nodes": [{
                "nodeid": "root-node-id",
                "name": "Root",
                "alias": "root",
                "datatype": "semantic",
                "nodegroup_id": "root-nodegroup",
                "graph_id": "test-graph-id",
                "is_collector": false,
                "isrequired": false,
                "exportable": false,
                "ontologyclass": "E1_CRM_Entity",
                "hascustomalias": false,
                "issearchable": false,
                "istopnode": true
            }],
            "nodegroups": [{
                "nodegroupid": "root-nodegroup",
                "cardinality": "1"
            }],
            "edges": [],
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "root": {
                "nodeid": "root-node-id",
                "name": "Root",
                "alias": "root",
                "datatype": "semantic",
                "nodegroup_id": "root-nodegroup",
                "graph_id": "test-graph-id",
                "is_collector": false,
                "isrequired": false,
                "exportable": false,
                "ontologyclass": "E1_CRM_Entity",
                "hascustomalias": false,
                "issearchable": false,
                "istopnode": true
            }
        }"#;

        let mut graph: StaticGraph =
            serde_json::from_str(graph_json).expect("Failed to parse test graph JSON");
        graph.build_indices();
        graph
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = generate_uuid_v5(("graph", Some("test-id")), "node-1");
        let uuid2 = generate_uuid_v5(("graph", Some("test-id")), "node-1");
        let uuid3 = generate_uuid_v5(("graph", Some("test-id")), "node-2");

        // Same inputs should produce same output
        assert_eq!(uuid1, uuid2);
        // Different inputs should produce different output
        assert_ne!(uuid1, uuid3);
        // Should be valid UUID format
        assert!(Uuid::parse_str(&uuid1).is_ok());
    }

    #[test]
    fn test_add_semantic_node() {
        let graph = create_test_graph();

        let result = GraphMutator::new(graph)
            .add_semantic_node(
                Some("root"),
                "child",
                "Child Node",
                Cardinality::N,
                "E1_CRM_Entity",
                "P1_is_identified_by",
                Some("A child node"),
                NodeOptions::default(),
                None,
            )
            .build();

        assert!(result.is_ok());
        let built_graph = result.unwrap();

        // Should have 2 nodes now (root + child)
        assert_eq!(built_graph.nodes.len(), 2);

        // Should have 2 nodegroups
        assert_eq!(built_graph.nodegroups.len(), 2);

        // Should have 1 edge
        assert_eq!(built_graph.edges.len(), 1);

        // Should have a card auto-created
        assert_eq!(built_graph.cards_slice().len(), 1);
    }

    #[test]
    fn test_add_string_node() {
        let graph = create_test_graph();

        let result = GraphMutator::new(graph)
            .add_string_node(
                Some("root"),
                "name",
                "Name",
                Cardinality::One,
                "E41_Appellation",
                "P1_is_identified_by",
                None,
                NodeOptions::default(),
                None,
            )
            .build();

        assert!(result.is_ok());
        let built_graph = result.unwrap();

        // String node with cardinality One should inherit parent's nodegroup
        // So we should still have 1 nodegroup (root's)
        assert_eq!(built_graph.nodegroups.len(), 1);

        // Should have a widget auto-created (string -> text-widget)
        // But only if there's a card for the nodegroup
        // Since cardinality is One, no new card is created, so no widget
    }

    #[test]
    fn test_add_node_duplicate_alias_error() {
        let graph = create_test_graph();

        // Try to add a node with alias "root" which already exists
        let result = GraphMutator::new(graph)
            .add_string_node(
                Some("root"),
                "root", // This alias already exists!
                "Duplicate",
                Cardinality::One,
                "E41_Appellation",
                "P1_is_identified_by",
                None,
                NodeOptions::default(),
                None,
            )
            .build();

        assert!(result.is_err());
        assert!(matches!(result, Err(MutationError::AliasAlreadyExists(_))));
    }

    #[test]
    fn test_add_node_invalid_config_error() {
        let graph = create_test_graph();

        // Provide invalid config (not an object)
        let result = GraphMutator::new(graph)
            .add_generic_node(
                Some("root"),
                "child",
                "Child",
                Cardinality::N,
                "string",
                "E41_Appellation",
                "P1_is_identified_by",
                None,
                NodeOptions::default(),
                Some(serde_json::json!("not an object")), // Invalid: should be object
            )
            .build();

        assert!(result.is_err());
        assert!(matches!(result, Err(MutationError::InvalidConfig { .. })));
    }

    #[test]
    fn test_get_default_widget() {
        assert!(get_default_widget_for_datatype("string").is_ok());
        assert!(get_default_widget_for_datatype("number").is_ok());
        assert!(get_default_widget_for_datatype("concept").is_ok());
        assert!(get_default_widget_for_datatype("semantic").is_err());
        assert!(get_default_widget_for_datatype("unknown").is_err());
    }

    #[test]
    fn test_json_mutation_api() {
        let graph = create_test_graph();

        let mutations_json = r#"{
            "mutations": [
                {
                    "AddNode": {
                        "parent_alias": "root",
                        "alias": "child",
                        "name": "Child Node",
                        "cardinality": "N",
                        "datatype": "string",
                        "ontology_class": "E41_Appellation",
                        "parent_property": "P1_is_identified_by",
                        "description": "A test child node",
                        "config": null,
                        "options": {
                            "isrequired": true
                        }
                    }
                }
            ],
            "options": {
                "autocreate_card": true,
                "autocreate_widget": true
            }
        }"#;

        let result = apply_mutations_from_json(&graph, mutations_json);
        assert!(result.is_ok(), "JSON mutation failed: {:?}", result.err());

        let mutated = result.unwrap();
        // Should have 2 nodes (root + child)
        assert_eq!(mutated.nodes.len(), 2);
        // Should have 2 nodegroups (root + child with cardinality N)
        assert_eq!(mutated.nodegroups.len(), 2);
        // Should have 1 edge (root -> child)
        assert_eq!(mutated.edges.len(), 1);
    }

    #[test]
    fn test_mutations_serialization() {
        let mutations = vec![GraphMutation::AddNode(AddNodeParams {
            parent_alias: Some("root".to_string()),
            alias: "test".to_string(),
            name: "Test".to_string(),
            cardinality: Cardinality::One,
            datatype: "string".to_string(),
            ontology_class: "E41".to_string(),
            parent_property: "P1".to_string(),
            description: None,
            config: None,
            options: NodeOptions::default(),
        })];

        let json = mutations_to_json(&mutations);
        assert!(json.is_ok());

        // Verify it's valid JSON
        let parsed: Result<Vec<GraphMutation>, _> = serde_json::from_str(&json.unwrap());
        assert!(parsed.is_ok());
    }

    fn create_test_subgraph() -> StaticGraph {
        // Create a simple subgraph/branch to add
        let subgraph_json = r#"{
            "graphid": "subgraph-id",
            "name": {"en": "Test Subgraph"},
            "isresource": false,
            "publication": {
                "publicationid": "test-publication-id",
                "graph_id": "subgraph-id",
                "published_time": "2024-01-01T00:00:00.000"
            },
            "nodes": [
                {
                    "nodeid": "sub-root-id",
                    "name": "Subgraph Root",
                    "alias": "sub_root",
                    "datatype": "semantic",
                    "nodegroup_id": "sub-root-ng",
                    "graph_id": "subgraph-id",
                    "is_collector": true,
                    "isrequired": false,
                    "exportable": false,
                    "ontologyclass": "E41_Appellation",
                    "hascustomalias": false,
                    "issearchable": false,
                    "istopnode": true
                },
                {
                    "nodeid": "sub-child1-id",
                    "name": "Child 1",
                    "alias": "child1",
                    "datatype": "string",
                    "nodegroup_id": "sub-child1-ng",
                    "graph_id": "subgraph-id",
                    "is_collector": false,
                    "isrequired": false,
                    "exportable": true,
                    "ontologyclass": "E41_Appellation",
                    "hascustomalias": false,
                    "issearchable": true,
                    "istopnode": false
                },
                {
                    "nodeid": "sub-child2-id",
                    "name": "Child 2",
                    "alias": "child2",
                    "datatype": "concept",
                    "nodegroup_id": "sub-child1-ng",
                    "graph_id": "subgraph-id",
                    "is_collector": false,
                    "isrequired": false,
                    "exportable": true,
                    "ontologyclass": "E55_Type",
                    "hascustomalias": false,
                    "issearchable": true,
                    "istopnode": false
                }
            ],
            "nodegroups": [
                {
                    "nodegroupid": "sub-root-ng",
                    "cardinality": "n",
                    "parentnodegroup_id": null
                },
                {
                    "nodegroupid": "sub-child1-ng",
                    "cardinality": "1",
                    "parentnodegroup_id": "sub-root-ng"
                }
            ],
            "edges": [
                {
                    "edgeid": "sub-edge1-id",
                    "domainnode_id": "sub-root-id",
                    "rangenode_id": "sub-child1-id",
                    "graph_id": "subgraph-id",
                    "ontologyproperty": "P3_has_note"
                },
                {
                    "edgeid": "sub-edge2-id",
                    "domainnode_id": "sub-child1-id",
                    "rangenode_id": "sub-child2-id",
                    "graph_id": "subgraph-id",
                    "ontologyproperty": "P2_has_type"
                }
            ],
            "cards": [
                {
                    "cardid": "sub-card1-id",
                    "nodegroup_id": "sub-child1-ng",
                    "graph_id": "subgraph-id",
                    "name": {"en": "Child Card"},
                    "active": true,
                    "visible": true,
                    "component_id": "f05e4d3a-53c1-11e8-b0ea-784f435179ea",
                    "helpenabled": false,
                    "helptext": {"en": ""},
                    "helptitle": {"en": ""},
                    "instructions": {"en": ""},
                    "constraints": []
                }
            ],
            "cards_x_nodes_x_widgets": [
                {
                    "id": "sub-cxnxw1-id",
                    "card_id": "sub-card1-id",
                    "node_id": "sub-child1-id",
                    "widget_id": "10000000-0000-0000-0000-000000000001",
                    "config": {},
                    "label": {"en": "Child 1 Label"},
                    "sortorder": 1,
                    "visible": true
                },
                {
                    "id": "sub-cxnxw2-id",
                    "card_id": "sub-card1-id",
                    "node_id": "sub-child2-id",
                    "widget_id": "10000000-0000-0000-0000-000000000002",
                    "config": {},
                    "label": {"en": "Child 2 Label"},
                    "sortorder": 2,
                    "visible": true
                }
            ],
            "root": {
                "nodeid": "sub-root-id",
                "name": "Subgraph Root",
                "alias": "sub_root",
                "datatype": "semantic",
                "nodegroup_id": "sub-root-ng",
                "graph_id": "subgraph-id",
                "is_collector": true,
                "isrequired": false,
                "exportable": false,
                "ontologyclass": "E41_Appellation",
                "hascustomalias": false,
                "issearchable": false,
                "istopnode": true
            }
        }"#;

        let mut graph: StaticGraph =
            serde_json::from_str(subgraph_json).expect("Failed to parse test subgraph JSON");
        graph.build_indices();
        graph
    }

    #[test]
    fn test_add_subgraph_basic() {
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        let params = AddSubgraphParams {
            subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };

        let mut graph_clone = graph.deep_clone();
        let result = apply_add_subgraph(&mut graph_clone, params);

        assert!(result.is_ok(), "AddSubgraph failed: {:?}", result.err());

        // Original graph had 1 node, subgraph has 3 (1 root + 2 children)
        // We skip the root, so we add 2 nodes
        assert_eq!(graph_clone.nodes.len(), 3); // 1 original + 2 from subgraph

        // Original had 1 nodegroup, subgraph has 2 (1 root + 1 child)
        // We skip root's nodegroup, so we add 1
        assert_eq!(graph_clone.nodegroups.len(), 2); // 1 original + 1 from subgraph

        // Original had 0 edges, subgraph has 2 (1 from root, 1 internal)
        // We skip edge from root, but create 1 connecting edge + 1 internal
        assert_eq!(graph_clone.edges.len(), 2); // 1 connecting + 1 internal

        // Original had 0 cards, subgraph has 1 (not for root's nodegroup)
        assert_eq!(graph_clone.cards_slice().len(), 1);

        // Subgraph had 2 cxnxw (both for non-root nodes)
        assert_eq!(graph_clone.cards_x_nodes_x_widgets_slice().len(), 2);
    }

    #[test]
    fn test_add_subgraph_with_alias_suffix() {
        // With Arches-compatible behavior, alias_suffix is used for UUID generation,
        // not for alias suffixing. Aliases are only suffixed when they clash.
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        let params = AddSubgraphParams {
            subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: Some("v2".to_string()),
            alias_prefix: None,
            name_prefix: None,
        };

        let mut graph_clone = graph.deep_clone();
        let result = apply_add_subgraph(&mut graph_clone, params);

        assert!(
            result.is_ok(),
            "AddSubgraph with suffix failed: {:?}",
            result.err()
        );

        // Aliases should be preserved (no clash in base graph)
        let child1 = graph_clone
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child1"));
        assert!(
            child1.is_some(),
            "Node with alias 'child1' not found (aliases should be preserved when no clash)"
        );

        let child2 = graph_clone
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child2"));
        assert!(
            child2.is_some(),
            "Node with alias 'child2' not found (aliases should be preserved when no clash)"
        );

        // Verify sourcebranchpublication_id is set
        let child1_node = child1.unwrap();
        assert!(
            child1_node.sourcebranchpublication_id.is_some(),
            "sourcebranchpublication_id should be set on branch nodes"
        );
    }

    #[test]
    fn test_add_subgraph_alias_clash() {
        // With Arches-compatible behavior, clashing aliases are auto-suffixed
        // with _n1, _n2, etc. instead of throwing an error.
        let graph = create_test_graph();

        // First, add a node with alias "child1" to the base graph
        let mut graph_with_child = GraphMutator::new(graph)
            .add_string_node(
                Some("root"),
                "child1",
                "Existing Child",
                Cardinality::N,
                "E41_Appellation",
                "P1_is_identified_by",
                None,
                NodeOptions::default(),
                None,
            )
            .build()
            .expect("Failed to create graph with child");

        // Now try to add subgraph which also has "child1" alias
        let subgraph = create_test_subgraph();

        let params = AddSubgraphParams {
            subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };

        let result = apply_add_subgraph(&mut graph_with_child, params);

        // Should succeed - clashing aliases get auto-suffixed
        assert!(
            result.is_ok(),
            "AddSubgraph should auto-suffix clashing aliases: {:?}",
            result.err()
        );

        // Original child1 should still exist
        let original_child1 = graph_with_child
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child1") && n.name == "Existing Child");
        assert!(
            original_child1.is_some(),
            "Original 'child1' node should still exist"
        );

        // New child1 from branch should be renamed to child1_n1
        let new_child1 = graph_with_child
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child1_n1"));
        assert!(
            new_child1.is_some(),
            "Clashing alias should be renamed to 'child1_n1'"
        );

        // child2 should be unchanged (no clash)
        let child2 = graph_with_child
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child2"));
        assert!(
            child2.is_some(),
            "Non-clashing alias 'child2' should be preserved"
        );
    }

    #[test]
    fn test_add_subgraph_id_remapping() {
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        let params = AddSubgraphParams {
            subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };

        let mut graph_clone = graph.deep_clone();
        let result = apply_add_subgraph(&mut graph_clone, params);
        assert!(result.is_ok());

        // Verify that the new nodes have different IDs from the original subgraph
        let original_ids = ["sub-root-id", "sub-child1-id", "sub-child2-id"];
        for node in &graph_clone.nodes {
            assert!(
                !original_ids.contains(&node.nodeid.as_str()),
                "Node ID {} was not remapped",
                node.nodeid
            );
        }

        // Verify that edges have been remapped
        let original_edge_ids = ["sub-edge1-id", "sub-edge2-id"];
        for edge in &graph_clone.edges {
            assert!(
                !original_edge_ids.contains(&edge.edgeid.as_str()),
                "Edge ID {} was not remapped",
                edge.edgeid
            );
        }

        // Verify graph_id is remapped to target graph
        for node in &graph_clone.nodes {
            assert_eq!(
                node.graph_id, "test-graph-id",
                "Node graph_id not remapped to target graph"
            );
        }
    }

    #[test]
    fn test_add_subgraph_preserves_external_ids() {
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        let params = AddSubgraphParams {
            subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };

        let mut graph_clone = graph.deep_clone();
        let result = apply_add_subgraph(&mut graph_clone, params);
        assert!(result.is_ok());

        // Verify widget_ids are preserved (not remapped)
        let cxnxws = graph_clone.cards_x_nodes_x_widgets_slice();
        assert!(
            cxnxws
                .iter()
                .any(|c| c.widget_id == "10000000-0000-0000-0000-000000000001"),
            "Widget ID for text-widget should be preserved"
        );
        assert!(
            cxnxws
                .iter()
                .any(|c| c.widget_id == "10000000-0000-0000-0000-000000000002"),
            "Widget ID for concept-select-widget should be preserved"
        );

        // Verify component_id is preserved
        let cards = graph_clone.cards_slice();
        assert!(
            cards
                .iter()
                .any(|c| c.component_id == "f05e4d3a-53c1-11e8-b0ea-784f435179ea"),
            "Component ID should be preserved"
        );
    }

    #[test]
    fn test_add_subgraph_target_not_found() {
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        let params = AddSubgraphParams {
            subgraph,
            target_node_id: "nonexistent-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };

        let mut graph_clone = graph.deep_clone();
        let result = apply_add_subgraph(&mut graph_clone, params);

        assert!(result.is_err(), "Expected NodeNotFound error");
        match result {
            Err(MutationError::NodeNotFound(id)) => {
                assert_eq!(id, "nonexistent-node-id");
            }
            Err(e) => panic!("Expected NodeNotFound error, got: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }

    #[test]
    fn test_add_subgraph_via_json_api() {
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        // Serialize the subgraph to include in the mutation
        let subgraph_json = serde_json::to_string(&subgraph).expect("Failed to serialize subgraph");

        let mutations_json = format!(
            r#"{{
            "mutations": [
                {{
                    "AddSubgraph": {{
                        "subgraph": {},
                        "target_node_id": "root-node-id",
                        "ontology_property": "P106_is_composed_of",
                        "alias_suffix": "json"
                    }}
                }}
            ],
            "options": {{
                "autocreate_card": true,
                "autocreate_widget": true
            }}
        }}"#,
            subgraph_json
        );

        let result = apply_mutations_from_json(&graph, &mutations_json);
        assert!(
            result.is_ok(),
            "JSON AddSubgraph mutation failed: {:?}",
            result.err()
        );

        let mutated = result.unwrap();
        // Should have 3 nodes (1 original + 2 from subgraph)
        assert_eq!(mutated.nodes.len(), 3);

        // With Arches-compatible behavior, aliases are preserved (no clash)
        // alias_suffix is only used for UUID generation
        assert!(
            mutated
                .nodes
                .iter()
                .any(|n| n.alias.as_deref() == Some("child1")),
            "Alias 'child1' should be preserved (no clash)"
        );

        // Verify sourcebranchpublication_id is set
        let branch_node = mutated
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child1"))
            .unwrap();
        assert!(
            branch_node.sourcebranchpublication_id.is_some(),
            "sourcebranchpublication_id should be set on branch nodes"
        );
    }

    // =========================================================================
    // UpdateSubgraph Tests
    // =========================================================================

    #[test]
    fn test_update_subgraph_first_time_acts_like_add() {
        // When there are no existing branch nodes, UpdateSubgraph falls back to AddSubgraph
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        let params = UpdateSubgraphParams {
            subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            remove_orphaned: false,
            alias_prefix: None,
            name_prefix: None,
        };

        let mut graph_clone = graph.deep_clone();
        let result = apply_update_subgraph(&mut graph_clone, params);

        assert!(
            result.is_ok(),
            "UpdateSubgraph should succeed: {:?}",
            result.err()
        );

        // Should have added the branch nodes
        assert_eq!(
            graph_clone.nodes.len(),
            3,
            "Should have 3 nodes: root + 2 from branch"
        );

        // Verify sourcebranchpublication_id is set
        let child1 = graph_clone
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child1"))
            .unwrap();
        assert!(child1.sourcebranchpublication_id.is_some());
    }

    #[test]
    fn test_update_subgraph_updates_existing_nodes() {
        // First add a subgraph, then update it
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        // Add subgraph first
        let add_params = AddSubgraphParams {
            subgraph: subgraph.clone(),
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };
        let mut graph_with_branch = graph.deep_clone();
        apply_add_subgraph(&mut graph_with_branch, add_params).expect("Add should succeed");

        // Now update with a modified subgraph
        let mut updated_subgraph = subgraph.deep_clone();
        // Modify a node's name
        for node in &mut updated_subgraph.nodes {
            if node.alias.as_deref() == Some("child1") {
                node.name = "Updated Child 1".to_string();
            }
        }

        let update_params = UpdateSubgraphParams {
            subgraph: updated_subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            remove_orphaned: false,
            alias_prefix: None,
            name_prefix: None,
        };
        let result = apply_update_subgraph(&mut graph_with_branch, update_params);

        assert!(
            result.is_ok(),
            "UpdateSubgraph should succeed: {:?}",
            result.err()
        );

        // Still should have 3 nodes
        assert_eq!(graph_with_branch.nodes.len(), 3);

        // Check that the name was updated
        let child1 = graph_with_branch
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child1"))
            .unwrap();
        assert_eq!(
            child1.name, "Updated Child 1",
            "Node name should be updated"
        );
    }

    #[test]
    fn test_update_subgraph_adds_new_nodes() {
        // First add a subgraph, then update with additional nodes
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        // Add subgraph first
        let add_params = AddSubgraphParams {
            subgraph: subgraph.clone(),
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };
        let mut graph_with_branch = graph.deep_clone();
        apply_add_subgraph(&mut graph_with_branch, add_params).expect("Add should succeed");
        assert_eq!(graph_with_branch.nodes.len(), 3);

        // Now update with an additional node
        let mut updated_subgraph = subgraph.deep_clone();
        // Add a new node (child3)
        let new_node = StaticNode {
            nodeid: "sub-child3-id".to_string(),
            name: "Child 3".to_string(),
            alias: Some("child3".to_string()),
            datatype: "string".to_string(),
            nodegroup_id: Some("sub-child-ng-id".to_string()),
            graph_id: "sub-graph-id".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: Some(3),
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: Some("E41_Appellation".to_string()),
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: true,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        };
        updated_subgraph.nodes.push(new_node);

        let update_params = UpdateSubgraphParams {
            subgraph: updated_subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            remove_orphaned: false,
            alias_prefix: None,
            name_prefix: None,
        };
        let result = apply_update_subgraph(&mut graph_with_branch, update_params);

        assert!(
            result.is_ok(),
            "UpdateSubgraph should succeed: {:?}",
            result.err()
        );

        // Should now have 4 nodes (root + 3 from branch)
        assert_eq!(
            graph_with_branch.nodes.len(),
            4,
            "Should have added new node"
        );

        // Check that child3 was added
        let child3 = graph_with_branch
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child3"));
        assert!(child3.is_some(), "New node child3 should be added");
    }

    #[test]
    fn test_update_subgraph_removes_orphaned() {
        // First add a subgraph, then update with fewer nodes and remove_orphaned=true
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        // Add subgraph first
        let add_params = AddSubgraphParams {
            subgraph: subgraph.clone(),
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            alias_prefix: None,
            name_prefix: None,
        };
        let mut graph_with_branch = graph.deep_clone();
        apply_add_subgraph(&mut graph_with_branch, add_params).expect("Add should succeed");
        assert_eq!(graph_with_branch.nodes.len(), 3);

        // Now update with only child1 (remove child2)
        let mut updated_subgraph = subgraph.deep_clone();
        updated_subgraph
            .nodes
            .retain(|n| n.alias.as_deref() != Some("child2"));

        let update_params = UpdateSubgraphParams {
            subgraph: updated_subgraph,
            target_node_id: "root-node-id".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            remove_orphaned: true, // Enable orphan removal
            alias_prefix: None,
            name_prefix: None,
        };
        let result = apply_update_subgraph(&mut graph_with_branch, update_params);

        assert!(
            result.is_ok(),
            "UpdateSubgraph should succeed: {:?}",
            result.err()
        );

        // Should now have 2 nodes (root + child1 only)
        assert_eq!(
            graph_with_branch.nodes.len(),
            2,
            "Orphaned child2 should be removed"
        );

        // child2 should be gone
        let child2 = graph_with_branch
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child2"));
        assert!(child2.is_none(), "child2 should be removed");
    }

    #[test]
    fn test_update_subgraph_target_not_found() {
        let graph = create_test_graph();
        let subgraph = create_test_subgraph();

        let params = UpdateSubgraphParams {
            subgraph,
            target_node_id: "non-existent-node".to_string(),
            ontology_property: "P106_is_composed_of".to_string(),
            alias_suffix: None,
            remove_orphaned: false,
            alias_prefix: None,
            name_prefix: None,
        };

        let mut graph_clone = graph.deep_clone();
        let result = apply_update_subgraph(&mut graph_clone, params);

        assert!(result.is_err(), "Should fail when target not found");
        match result {
            Err(MutationError::NodeNotFound(id)) => {
                assert_eq!(id, "non-existent-node");
            }
            Err(e) => panic!("Expected NodeNotFound error, got: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }

    // =========================================================================
    // ConceptChangeCollection Tests
    // =========================================================================

    #[test]
    fn test_concept_change_collection_concept_node() {
        // Create a graph with a concept node
        let mut graph = create_test_graph();

        // Add a concept node
        let concept_node = StaticNode {
            nodeid: "concept-node-id".to_string(),
            name: "Test Concept".to_string(),
            alias: Some("test_concept".to_string()),
            datatype: "concept".to_string(),
            nodegroup_id: Some("root-node-id".to_string()),
            graph_id: "test-graph-id".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: Some(1),
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: Some("E55_Type".to_string()),
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: true,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        };
        graph.push_node(concept_node);

        let params = ConceptChangeCollectionParams {
            node_id: "test_concept".to_string(),
            collection_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        };

        let result = apply_concept_change_collection(&mut graph, params);
        assert!(
            result.is_ok(),
            "ConceptChangeCollection should succeed: {:?}",
            result.err()
        );

        // Verify config was updated
        let node = graph.find_node_by_alias("test_concept").unwrap();
        let rdm_collection = node.config.get("rdmCollection").and_then(|v| v.as_str());
        assert_eq!(rdm_collection, Some("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_concept_change_collection_concept_list_node() {
        let mut graph = create_test_graph();

        // Add a concept-list node
        let concept_list_node = StaticNode {
            nodeid: "concept-list-node-id".to_string(),
            name: "Test Concept List".to_string(),
            alias: Some("test_concept_list".to_string()),
            datatype: "concept-list".to_string(),
            nodegroup_id: Some("root-node-id".to_string()),
            graph_id: "test-graph-id".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: Some(1),
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: Some("E55_Type".to_string()),
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: true,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        };
        graph.push_node(concept_list_node);

        let params = ConceptChangeCollectionParams {
            node_id: "test_concept_list".to_string(),
            collection_id: "my-new-collection-id".to_string(),
        };

        let result = apply_concept_change_collection(&mut graph, params);
        assert!(
            result.is_ok(),
            "ConceptChangeCollection should succeed for concept-list: {:?}",
            result.err()
        );

        let node = graph.find_node_by_alias("test_concept_list").unwrap();
        assert_eq!(
            node.config.get("rdmCollection").and_then(|v| v.as_str()),
            Some("my-new-collection-id")
        );
    }

    #[test]
    fn test_concept_change_collection_invalid_datatype() {
        let mut graph = create_test_graph();

        // Add a string node (not concept)
        let string_node = StaticNode {
            nodeid: "string-node-id".to_string(),
            name: "Test String".to_string(),
            alias: Some("test_string".to_string()),
            datatype: "string".to_string(),
            nodegroup_id: Some("root-node-id".to_string()),
            graph_id: "test-graph-id".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: Some(1),
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: Some("E41_Appellation".to_string()),
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: true,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        };
        graph.push_node(string_node);

        let params = ConceptChangeCollectionParams {
            node_id: "test_string".to_string(),
            collection_id: "some-collection".to_string(),
        };

        let result = apply_concept_change_collection(&mut graph, params);
        assert!(result.is_err(), "Should fail for non-concept datatype");

        match result {
            Err(MutationError::InvalidDatatype {
                expected,
                found,
                node_id,
            }) => {
                assert!(expected.contains("concept"));
                assert_eq!(found, "string");
                assert_eq!(node_id, "test_string");
            }
            Err(e) => panic!("Expected InvalidDatatype error, got: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }

    #[test]
    fn test_concept_change_collection_node_not_found() {
        let mut graph = create_test_graph();

        let params = ConceptChangeCollectionParams {
            node_id: "nonexistent_node".to_string(),
            collection_id: "some-collection".to_string(),
        };

        let result = apply_concept_change_collection(&mut graph, params);
        assert!(result.is_err(), "Should fail when node not found");

        match result {
            Err(MutationError::NodeNotFound(id)) => {
                assert_eq!(id, "nonexistent_node");
            }
            Err(e) => panic!("Expected NodeNotFound error, got: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }

    #[test]
    fn test_concept_change_collection_by_node_id() {
        // Test that we can also find nodes by ID, not just alias
        let mut graph = create_test_graph();

        let concept_node = StaticNode {
            nodeid: "concept-node-uuid".to_string(),
            name: "Test Concept".to_string(),
            alias: None, // No alias
            datatype: "concept".to_string(),
            nodegroup_id: Some("root-node-id".to_string()),
            graph_id: "test-graph-id".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: Some(1),
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: Some("E55_Type".to_string()),
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: true,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        };
        graph.push_node(concept_node);

        let params = ConceptChangeCollectionParams {
            node_id: "concept-node-uuid".to_string(), // Use node ID
            collection_id: "new-collection".to_string(),
        };

        let result = apply_concept_change_collection(&mut graph, params);
        assert!(result.is_ok(), "Should find node by ID: {:?}", result.err());

        let node = graph
            .nodes
            .iter()
            .find(|n| n.nodeid == "concept-node-uuid")
            .unwrap();
        assert_eq!(
            node.config.get("rdmCollection").and_then(|v| v.as_str()),
            Some("new-collection")
        );
    }

    #[test]
    fn test_concept_change_collection_via_instruction() {
        let mut graph = create_test_graph();

        // Add a concept node
        let concept_node = StaticNode {
            nodeid: "concept-node-id".to_string(),
            name: "Test Concept".to_string(),
            alias: Some("my_concept".to_string()),
            datatype: "concept".to_string(),
            nodegroup_id: Some("root-node-id".to_string()),
            graph_id: "test-graph-id".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: Some(1),
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: Some("E55_Type".to_string()),
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: true,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        };
        graph.push_node(concept_node);

        // Create instruction
        let instruction = GraphInstruction {
            action: "concept_change_collection".to_string(),
            subject: "my_concept".to_string(),
            object: "new-collection-uuid".to_string(),
            params: HashMap::new(),
        };

        let mutation = instruction.to_mutation().expect("Should create mutation");
        let options = MutatorOptions::default();
        let result = apply_mutation(&mut graph, mutation, &options);
        assert!(
            result.is_ok(),
            "Instruction should apply: {:?}",
            result.err()
        );

        let node = graph.find_node_by_alias("my_concept").unwrap();
        assert_eq!(
            node.config.get("rdmCollection").and_then(|v| v.as_str()),
            Some("new-collection-uuid")
        );
    }

    // =========================================================================
    // Skeleton Graph Tests
    // =========================================================================

    #[test]
    fn test_create_skeleton_graph() {
        let graph =
            create_skeleton_graph("Person", "person", true, Some("http://example.org/Person"));

        // Check basic structure
        assert!(!graph.graphid.is_empty());
        assert_eq!(graph.name.to_string_default(), "Person".to_string());
        assert_eq!(graph.isresource, Some(true));

        // Check root node
        assert_eq!(graph.root.alias, Some("person".to_string()));
        assert_eq!(graph.root.datatype, "semantic");
        assert!(graph.root.istopnode);
        assert!(
            graph.root.nodegroup_id.is_none(),
            "Root should have no nodegroup"
        );
        assert_eq!(
            graph.root.ontologyclass,
            Some("http://example.org/Person".to_string())
        );

        // Check nodes vector includes root
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].nodeid, graph.root.nodeid);

        // Check empty collections
        assert!(graph.nodegroups.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_skeleton_graph_deterministic_ids() {
        let graph1 = create_skeleton_graph("Person", "person", true, None);
        let graph2 = create_skeleton_graph("Person", "person", true, None);

        // Same input should produce same IDs
        assert_eq!(graph1.graphid, graph2.graphid);
        assert_eq!(graph1.root.nodeid, graph2.root.nodeid);

        // Different input should produce different IDs
        let graph3 = create_skeleton_graph("Monument", "monument", true, None);
        assert_ne!(graph1.graphid, graph3.graphid);
    }

    #[test]
    fn test_skeleton_graph_branch_vs_resource() {
        let resource = create_skeleton_graph("Person", "person", true, None);
        let branch = create_skeleton_graph("Addresses", "addresses", false, None);

        assert_eq!(resource.isresource, Some(true));
        assert_eq!(branch.isresource, Some(false));
    }

    // =========================================================================
    // Instruction DSL Tests
    // =========================================================================

    #[test]
    fn test_instruction_add_node() {
        let graph = create_skeleton_graph("Person", "person", true, None);

        let instructions = vec![GraphInstruction::new("add_node", "person", "name")
            .with_str("datatype", "string")
            .with_str("name", "Full Name")
            .with_str("cardinality", "n")
            .with_str("ontology_class", "http://example.org/Name")
            .with_str("parent_property", "http://example.org/hasName")];

        let result = apply_instructions(&graph, instructions, MutatorOptions::default(), None);
        assert!(result.is_ok(), "Instruction failed: {:?}", result.err());

        let mutated = result.unwrap();
        assert_eq!(mutated.nodes.len(), 2);

        let name_node = mutated
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("name"));
        assert!(name_node.is_some(), "Should find 'name' node");

        let name_node = name_node.unwrap();
        assert_eq!(name_node.datatype, "string");
        assert_eq!(name_node.name, "Full Name");
        assert!(
            name_node.nodegroup_id.is_some(),
            "Non-root node should have nodegroup"
        );
    }

    #[test]
    fn test_instruction_multiple_nodes() {
        let graph = create_skeleton_graph("Person", "person", true, None);

        let instructions = vec![
            GraphInstruction::new("add_node", "person", "names")
                .with_str("datatype", "semantic")
                .with_str("cardinality", "n"),
            GraphInstruction::new("add_node", "names", "full_name")
                .with_str("datatype", "string")
                .with_str("cardinality", "1"),
            GraphInstruction::new("add_node", "names", "alias_name")
                .with_str("datatype", "string")
                .with_str("cardinality", "1"),
        ];

        let result = apply_instructions(&graph, instructions, MutatorOptions::default(), None);
        assert!(result.is_ok(), "Instructions failed: {:?}", result.err());

        let mutated = result.unwrap();
        assert_eq!(mutated.nodes.len(), 4); // person, names, full_name, alias_name

        // Check 'names' semantic node has its own nodegroup
        let names_node = mutated
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("names"))
            .unwrap();
        assert!(names_node.nodegroup_id.is_some());
        let names_ng = names_node.nodegroup_id.clone().unwrap();

        // Check full_name and alias_name share names' nodegroup (cardinality 1)
        let full_name = mutated
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("full_name"))
            .unwrap();
        let alias_name = mutated
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("alias_name"))
            .unwrap();
        assert_eq!(full_name.nodegroup_id, Some(names_ng.clone()));
        assert_eq!(alias_name.nodegroup_id, Some(names_ng.clone()));
    }

    #[test]
    fn test_instruction_from_json() {
        let graph = create_skeleton_graph("Person", "person", true, None);

        let json = r#"{
            "instructions": [
                {
                    "action": "add_node",
                    "subject": "person",
                    "object": "name",
                    "params": {
                        "datatype": "string",
                        "cardinality": "n"
                    }
                }
            ],
            "options": {
                "autocreate_card": true,
                "autocreate_widget": true
            }
        }"#;

        let result = apply_instructions_from_json(&graph, json);
        assert!(
            result.is_ok(),
            "JSON instructions failed: {:?}",
            result.err()
        );

        let mutated = result.unwrap();
        assert_eq!(mutated.nodes.len(), 2);
    }

    #[test]
    fn test_instruction_unknown_action_becomes_extension() {
        let graph = create_skeleton_graph("Test", "test", true, None);

        let instructions = vec![GraphInstruction::new("invalid_action", "test", "foo")];

        // Without a registry, unknown actions are treated as extension mutations
        // and fail because no handler is registered
        let result = apply_instructions(&graph, instructions, MutatorOptions::default(), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Extension mutation"));
    }

    #[test]
    fn test_root_children_always_get_nodegroup() {
        // This tests the fix for apply_add_node where direct children of root
        // should always get their own nodegroup, regardless of cardinality
        let graph = create_skeleton_graph("Test", "test", true, None);

        let instructions = vec![
            // Even with cardinality 1, direct children of root should get their own nodegroup
            GraphInstruction::new("add_node", "test", "child")
                .with_str("datatype", "string")
                .with_str("cardinality", "1"),
        ];

        let result = apply_instructions(&graph, instructions, MutatorOptions::default(), None);
        assert!(result.is_ok());

        let mutated = result.unwrap();
        let child = mutated
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("child"))
            .unwrap();

        // Child of root should have its own nodegroup (not inherit None from root)
        assert!(
            child.nodegroup_id.is_some(),
            "Direct children of root must have their own nodegroup"
        );
    }

    // =========================================================================
    // Create Model/Branch Instruction Tests
    // =========================================================================

    #[test]
    fn test_create_model_instruction() {
        let instructions = vec![GraphInstruction::new("create_model", "person", "")
            .with_str("name", "Person")
            .with_str("ontology_class", "http://example.org/Person")
            .with_str("slug", "person")];

        let result = build_graph_from_instructions(instructions, MutatorOptions::default());
        assert!(result.is_ok(), "create_model failed: {:?}", result.err());

        let graph = result.unwrap();
        assert_eq!(graph.isresource, Some(true));
        assert_eq!(graph.name.to_string_default(), "Person");
        assert_eq!(graph.root.alias, Some("person".to_string()));
        assert_eq!(
            graph.root.ontologyclass,
            Some("http://example.org/Person".to_string())
        );
        assert_eq!(graph.slug, Some("person".to_string()));
        assert!(
            graph.root.nodegroup_id.is_none(),
            "Root should have no nodegroup"
        );
    }

    #[test]
    fn test_create_model_default_slug() {
        let instructions =
            vec![GraphInstruction::new("create_model", "Person", "").with_str("name", "Person")];

        let result = build_graph_from_instructions(instructions, MutatorOptions::default());
        assert!(result.is_ok());

        let graph = result.unwrap();
        // Slug should default to lowercase root_alias for models too
        assert_eq!(graph.slug, Some("person".to_string()));
    }

    #[test]
    fn test_create_branch_instruction() {
        let instructions = vec![GraphInstruction::new("create_branch", "addresses", "")
            .with_str("name", "Addresses")
            .with_str("slug", "addresses-branch")];

        let result = build_graph_from_instructions(instructions, MutatorOptions::default());
        assert!(result.is_ok(), "create_branch failed: {:?}", result.err());

        let graph = result.unwrap();
        assert_eq!(graph.isresource, Some(false));
        assert_eq!(graph.name.to_string_default(), "Addresses");
        assert_eq!(graph.root.alias, Some("addresses".to_string()));
        assert_eq!(graph.slug, Some("addresses-branch".to_string()));
    }

    #[test]
    fn test_create_branch_default_slug() {
        let instructions = vec![GraphInstruction::new("create_branch", "MyAddresses", "")
            .with_str("name", "My Addresses")];

        let result = build_graph_from_instructions(instructions, MutatorOptions::default());
        assert!(result.is_ok());

        let graph = result.unwrap();
        // Slug should default to lowercase root_alias
        assert_eq!(graph.slug, Some("myaddresses".to_string()));
    }

    #[test]
    fn test_create_with_explicit_graphid() {
        let custom_graphid = "12345678-1234-1234-1234-123456789abc";
        let instructions = vec![
            GraphInstruction::new("create_model", "person", custom_graphid)
                .with_str("name", "Person"),
        ];

        let result = build_graph_from_instructions(instructions, MutatorOptions::default());
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.graphid, custom_graphid);
        assert_eq!(graph.root.graph_id, custom_graphid);
    }

    #[test]
    fn test_build_graph_with_nodes() {
        let instructions = vec![
            GraphInstruction::new("create_model", "person", "").with_str("name", "Person"),
            GraphInstruction::new("add_node", "person", "names")
                .with_str("datatype", "semantic")
                .with_str("cardinality", "n"),
            GraphInstruction::new("add_node", "names", "full_name")
                .with_str("datatype", "string")
                .with_str("cardinality", "1"),
        ];

        let result = build_graph_from_instructions(instructions, MutatorOptions::default());
        assert!(result.is_ok(), "Build failed: {:?}", result.err());

        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 3); // person, names, full_name

        // Verify structure
        let names = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("names"))
            .unwrap();
        let full_name = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("full_name"))
            .unwrap();

        assert!(names.nodegroup_id.is_some());
        assert_eq!(full_name.nodegroup_id, names.nodegroup_id);
    }

    #[test]
    fn test_build_graph_from_json() {
        let json = r#"{
            "instructions": [
                {
                    "action": "create_model",
                    "subject": "monument",
                    "object": "",
                    "params": { "name": "Monument" }
                },
                {
                    "action": "add_node",
                    "subject": "monument",
                    "object": "name",
                    "params": { "datatype": "string", "cardinality": "n" }
                }
            ],
            "options": {
                "autocreate_card": true,
                "autocreate_widget": true
            }
        }"#;

        let result = build_graph_from_instructions_json(json);
        assert!(result.is_ok(), "JSON build failed: {:?}", result.err());

        let graph = result.unwrap();
        assert_eq!(graph.isresource, Some(true));
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_build_graph_requires_create_first() {
        let instructions = vec![
            // Missing create_model/create_branch as first instruction
            GraphInstruction::new("add_node", "person", "name").with_str("datatype", "string"),
        ];

        let result = build_graph_from_instructions(instructions, MutatorOptions::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("First instruction must be"));
    }

    #[test]
    fn test_build_graph_empty_instructions() {
        let result = build_graph_from_instructions(vec![], MutatorOptions::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No instructions provided"));
    }

    #[test]
    fn test_create_action_in_apply_instructions_errors() {
        // create_model/create_branch should error when used with apply_instructions
        let graph = create_skeleton_graph("Test", "test", true, None);

        let instructions = vec![GraphInstruction::new("create_model", "other", "")];

        let result = apply_instructions(&graph, instructions, MutatorOptions::default(), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("creates a new graph"));
    }

    // =============================================================================
    // Deletion Mutation Tests
    // =============================================================================

    #[test]
    fn test_delete_card() {
        // Build a graph with a card
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node (this creates a nodegroup and card automatically)
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field1".to_string(),
                name: "Field 1".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Get the card ID
        let card_id = graph.cards.as_ref().unwrap()[0].cardid.clone();
        assert!(!card_id.is_empty());

        // Should have widgets
        assert!(graph
            .cards_x_nodes_x_widgets
            .as_ref()
            .map(|c| !c.is_empty())
            .unwrap_or(false));

        // Delete the card
        apply_mutation(
            &mut graph,
            GraphMutation::DeleteCard(DeleteCardParams {
                card_id: card_id.clone(),
            }),
            &options,
        )
        .unwrap();

        // Card should be gone
        assert!(graph
            .cards
            .as_ref()
            .map(|c| c.iter().all(|card| card.cardid != card_id))
            .unwrap_or(true));

        // Widgets for that card should be gone
        assert!(graph
            .cards_x_nodes_x_widgets
            .as_ref()
            .map(|c| c.iter().all(|w| w.card_id != card_id))
            .unwrap_or(true));
    }

    #[test]
    fn test_delete_card_not_found() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        let result = apply_mutation(
            &mut graph,
            GraphMutation::DeleteCard(DeleteCardParams {
                card_id: "nonexistent".to_string(),
            }),
            &options,
        );

        assert!(matches!(result, Err(MutationError::CardNotFound(_))));
    }

    #[test]
    fn test_delete_widget() {
        // Build a graph with a widget
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node (this creates a widget automatically)
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field1".to_string(),
                name: "Field 1".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Get the widget mapping ID
        let widget_id = graph.cards_x_nodes_x_widgets.as_ref().unwrap()[0]
            .id
            .clone();
        let initial_count = graph.cards_x_nodes_x_widgets.as_ref().unwrap().len();

        // Delete the widget
        apply_mutation(
            &mut graph,
            GraphMutation::DeleteWidget(DeleteWidgetParams {
                widget_mapping_id: widget_id.clone(),
            }),
            &options,
        )
        .unwrap();

        // Widget should be gone
        let final_count = graph
            .cards_x_nodes_x_widgets
            .as_ref()
            .map(|c| c.len())
            .unwrap_or(0);
        assert_eq!(final_count, initial_count - 1);
    }

    #[test]
    fn test_delete_widget_not_found() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        let result = apply_mutation(
            &mut graph,
            GraphMutation::DeleteWidget(DeleteWidgetParams {
                widget_mapping_id: "nonexistent".to_string(),
            }),
            &options,
        );

        assert!(matches!(result, Err(MutationError::WidgetNotFound(_))));
    }

    #[test]
    fn test_delete_function() {
        use crate::graph::StaticFunctionsXGraphs;

        // Build a graph with a function mapping
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a function mapping
        graph.functions_x_graphs = Some(vec![StaticFunctionsXGraphs {
            id: "func-mapping-1".to_string(),
            function_id: "60000000-0000-0000-0000-000000000001".to_string(),
            graph_id: graph.graphid.clone(),
            config: serde_json::Value::Object(serde_json::Map::new()),
        }]);

        // Delete the function
        apply_mutation(
            &mut graph,
            GraphMutation::DeleteFunction(DeleteFunctionParams {
                function_mapping_id: "func-mapping-1".to_string(),
            }),
            &options,
        )
        .unwrap();

        // Function mapping should be gone
        assert!(graph
            .functions_x_graphs
            .as_ref()
            .map(|f| f.is_empty())
            .unwrap_or(true));
    }

    #[test]
    fn test_delete_function_not_found() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        let result = apply_mutation(
            &mut graph,
            GraphMutation::DeleteFunction(DeleteFunctionParams {
                function_mapping_id: "nonexistent".to_string(),
            }),
            &options,
        );

        assert!(matches!(result, Err(MutationError::FunctionNotFound(_))));
    }

    #[test]
    fn test_delete_node() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field1".to_string(),
                name: "Field 1".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        let initial_node_count = graph.nodes.len();
        let initial_edge_count = graph.edges.len();

        // Delete the node by alias
        apply_mutation(
            &mut graph,
            GraphMutation::DeleteNode(DeleteNodeParams {
                node_id: "field1".to_string(),
            }),
            &options,
        )
        .unwrap();

        // Node should be gone
        assert_eq!(graph.nodes.len(), initial_node_count - 1);
        assert!(graph.find_node_by_alias("field1").is_none());

        // Edge should be gone
        assert!(graph.edges.len() < initial_edge_count);

        // Widget should be gone
        let has_widget_for_node = graph
            .cards_x_nodes_x_widgets
            .as_ref()
            .map(|c| c.iter().any(|w| w.node_id.contains("field1")))
            .unwrap_or(false);
        assert!(!has_widget_for_node);
    }

    #[test]
    fn test_delete_node_not_found() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        let result = apply_mutation(
            &mut graph,
            GraphMutation::DeleteNode(DeleteNodeParams {
                node_id: "nonexistent".to_string(),
            }),
            &options,
        );

        assert!(matches!(result, Err(MutationError::NodeNotFound(_))));
    }

    #[test]
    fn test_delete_node_cannot_delete_root() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Try to delete the root node
        let result = apply_mutation(
            &mut graph,
            GraphMutation::DeleteNode(DeleteNodeParams {
                node_id: "test".to_string(),
            }),
            &options,
        );

        assert!(matches!(
            result,
            Err(MutationError::CannotDeleteRootNode(_))
        ));
    }

    #[test]
    fn test_delete_nodegroup_cascade() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a parent node (creates nodegroup)
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "parent_field".to_string(),
                name: "Parent Field".to_string(),
                cardinality: Cardinality::N,
                datatype: "semantic".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Add a child node under it
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("parent_field".to_string()),
                alias: "child_field".to_string(),
                name: "Child Field".to_string(),
                cardinality: Cardinality::One,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Find the nodegroup for parent_field
        let parent_node = graph.find_node_by_alias("parent_field").unwrap();
        let nodegroup_id = parent_node.nodegroup_id.clone().unwrap();

        let initial_node_count = graph.nodes.len();
        let initial_nodegroup_count = graph.nodegroups.len();

        // Delete the nodegroup
        apply_mutation(
            &mut graph,
            GraphMutation::DeleteNodegroup(DeleteNodegroupParams {
                nodegroup_id: nodegroup_id.clone(),
            }),
            &options,
        )
        .unwrap();

        // Nodegroup should be gone
        assert!(graph
            .nodegroups
            .iter()
            .all(|ng| ng.nodegroupid != nodegroup_id));

        // Parent node should be gone
        assert!(graph.find_node_by_alias("parent_field").is_none());

        // Child node should also be gone (cascade)
        assert!(graph.find_node_by_alias("child_field").is_none());

        // Counts should be reduced
        assert!(graph.nodes.len() < initial_node_count);
        assert!(graph.nodegroups.len() < initial_nodegroup_count);
    }

    #[test]
    fn test_delete_nodegroup_not_found() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        let result = apply_mutation(
            &mut graph,
            GraphMutation::DeleteNodegroup(DeleteNodegroupParams {
                nodegroup_id: "nonexistent".to_string(),
            }),
            &options,
        );

        assert!(matches!(result, Err(MutationError::NodegroupNotFound(_))));
    }

    #[test]
    fn test_delete_node_via_instruction() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "my_field".to_string(),
                name: "My Field".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Delete via instruction
        let instruction = GraphInstruction::new("delete_node", "my_field", "");
        let mutation = instruction.to_mutation().unwrap();

        apply_mutation(&mut graph, mutation, &options).unwrap();

        // Node should be gone
        assert!(graph.find_node_by_alias("my_field").is_none());
    }

    // =============================================================================
    // Node Update Mutation Tests
    // =============================================================================

    #[test]
    fn test_update_node() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field1".to_string(),
                name: "Original Name".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Update the node
        apply_mutation(
            &mut graph,
            GraphMutation::UpdateNode(UpdateNodeParams {
                node_id: "field1".to_string(),
                name: Some("Updated Name".to_string()),
                ontology_class: Some("http://example.org/Class".to_string()),
                parent_property: None,
                description: Some("A description".to_string()),
                config: None,
                options: UpdateNodeOptions {
                    isrequired: Some(true),
                    ..UpdateNodeOptions::default()
                },
            }),
            &options,
        )
        .unwrap();

        // Verify updates
        let node = graph.find_node_by_alias("field1").unwrap();
        assert_eq!(node.name, "Updated Name");
        assert_eq!(
            node.ontologyclass,
            Some("http://example.org/Class".to_string())
        );
        assert!(node.description.is_some());
        assert!(node.isrequired);
        // Datatype should be unchanged
        assert_eq!(node.datatype, "string");
    }

    #[test]
    fn test_update_node_not_found() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        let result = apply_mutation(
            &mut graph,
            GraphMutation::UpdateNode(UpdateNodeParams {
                node_id: "nonexistent".to_string(),
                name: Some("New Name".to_string()),
                ontology_class: None,
                parent_property: None,
                description: None,
                config: None,
                options: UpdateNodeOptions::default(),
            }),
            &options,
        );

        assert!(matches!(result, Err(MutationError::NodeNotFound(_))));
    }

    #[test]
    fn test_change_node_type() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        // Disable autocreate_widget so we can change type
        let options = MutatorOptions {
            autocreate_card: true,
            autocreate_widget: false,
            ontology_validator: None,
        };

        // Add a semantic node (no widget)
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field1".to_string(),
                name: "Field 1".to_string(),
                cardinality: Cardinality::N,
                datatype: "semantic".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Change to string type
        apply_mutation(
            &mut graph,
            GraphMutation::ChangeNodeType(ChangeNodeTypeParams {
                node_id: "field1".to_string(),
                datatype: "string".to_string(),
                name: Some("Field 1 String".to_string()),
                ontology_class: None,
                parent_property: None,
                description: None,
                config: None,
                options: UpdateNodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Verify type changed
        let node = graph.find_node_by_alias("field1").unwrap();
        assert_eq!(node.datatype, "string");
        assert_eq!(node.name, "Field 1 String");
    }

    #[test]
    fn test_change_node_type_with_widgets_error() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default(); // Creates widgets

        // Add a node (creates widget automatically)
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field1".to_string(),
                name: "Field 1".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Verify widget exists
        let node = graph.find_node_by_alias("field1").unwrap();
        let has_widget = graph
            .cards_x_nodes_x_widgets
            .as_ref()
            .map(|cxnxws| cxnxws.iter().any(|c| c.node_id == node.nodeid))
            .unwrap_or(false);
        assert!(has_widget, "Widget should exist");

        // Try to change type - should fail
        let result = apply_mutation(
            &mut graph,
            GraphMutation::ChangeNodeType(ChangeNodeTypeParams {
                node_id: "field1".to_string(),
                datatype: "number".to_string(),
                name: None,
                ontology_class: None,
                parent_property: None,
                description: None,
                config: None,
                options: UpdateNodeOptions::default(),
            }),
            &options,
        );

        assert!(matches!(
            result,
            Err(MutationError::NodeHasDependentWidgets(_))
        ));
    }

    #[test]
    fn test_rename_node() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "old_alias".to_string(),
                name: "Old Name".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Rename the node
        apply_mutation(
            &mut graph,
            GraphMutation::RenameNode(RenameNodeParams {
                node_id: "old_alias".to_string(),
                alias: Some("new_alias".to_string()),
                name: Some("New Name".to_string()),
                description: Some("New description".to_string()),
            }),
            &options,
        )
        .unwrap();

        // Old alias should not find it
        assert!(graph.find_node_by_alias("old_alias").is_none());

        // New alias should find it
        let node = graph.find_node_by_alias("new_alias").unwrap();
        assert_eq!(node.name, "New Name");
        assert!(node.description.is_some());
    }

    #[test]
    fn test_rename_node_alias_conflict() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add two nodes
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field1".to_string(),
                name: "Field 1".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "field2".to_string(),
                name: "Field 2".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Try to rename field1 to field2 - should fail
        let result = apply_mutation(
            &mut graph,
            GraphMutation::RenameNode(RenameNodeParams {
                node_id: "field1".to_string(),
                alias: Some("field2".to_string()),
                name: None,
                description: None,
            }),
            &options,
        );

        assert!(matches!(result, Err(MutationError::AliasAlreadyExists(_))));
    }

    #[test]
    fn test_update_node_via_instruction() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "my_field".to_string(),
                name: "My Field".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Update via instruction
        let instruction = GraphInstruction::new("update_node", "my_field", "")
            .with_str("name", "Updated Field Name")
            .with_param("isrequired", serde_json::Value::Bool(true));
        let mutation = instruction.to_mutation().unwrap();

        apply_mutation(&mut graph, mutation, &options).unwrap();

        // Verify update
        let node = graph.find_node_by_alias("my_field").unwrap();
        assert_eq!(node.name, "Updated Field Name");
        assert!(node.isrequired);
    }

    #[test]
    fn test_rename_node_via_instruction() {
        let mut graph = create_skeleton_graph("Test", "test", false, None);
        let options = MutatorOptions::default();

        // Add a node
        apply_mutation(
            &mut graph,
            GraphMutation::AddNode(AddNodeParams {
                parent_alias: Some("test".to_string()),
                alias: "old_name".to_string(),
                name: "Old Name".to_string(),
                cardinality: Cardinality::N,
                datatype: "string".to_string(),
                ontology_class: String::new(),
                parent_property: String::new(),
                description: None,
                config: None,
                options: NodeOptions::default(),
            }),
            &options,
        )
        .unwrap();

        // Rename via instruction (object becomes new alias)
        let instruction = GraphInstruction::new("rename_node", "old_name", "new_name")
            .with_str("name", "New Display Name");
        let mutation = instruction.to_mutation().unwrap();

        apply_mutation(&mut graph, mutation, &options).unwrap();

        // Old alias gone, new alias works
        assert!(graph.find_node_by_alias("old_name").is_none());
        let node = graph.find_node_by_alias("new_name").unwrap();
        assert_eq!(node.name, "New Display Name");
    }

    // =========================================================================
    // Extension Mutation Tests
    // =========================================================================

    /// Test extension handler that adds a prefix to the root node's name
    struct TestPrefixHandler {
        prefix: String,
    }

    impl ExtensionMutationHandler for TestPrefixHandler {
        fn apply(
            &self,
            graph: &mut StaticGraph,
            params: &serde_json::Value,
            _options: &MutatorOptions,
        ) -> Result<(), MutationError> {
            let suffix = params.get("suffix").and_then(|v| v.as_str()).unwrap_or("");

            // Find the root node and modify it
            let root_id = graph.get_root().nodeid.clone();
            let root_name = graph.get_root().name.clone();
            if let Some(node) = graph.nodes.iter_mut().find(|n| n.nodeid == root_id) {
                node.name = format!("{}{}{}", self.prefix, root_name, suffix);
            }
            Ok(())
        }

        fn conformance(&self) -> MutationConformance {
            MutationConformance::AlwaysConformant
        }

        fn description(&self) -> &str {
            "Test handler that adds prefix/suffix to root name"
        }
    }

    #[test]
    fn test_extension_mutation_with_registry() {
        let graph = create_test_graph();
        let options = MutatorOptions::default();

        // Create registry with test handler
        let mut registry = ExtensionMutationRegistry::new();
        registry.register(
            "test.prefix_name",
            std::sync::Arc::new(TestPrefixHandler {
                prefix: "[PREFIX] ".to_string(),
            }),
        );

        // Create extension mutation
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: "test.prefix_name".to_string(),
            params: serde_json::json!({"suffix": " [SUFFIX]"}),
            conformance: MutationConformance::AlwaysConformant,
        });

        // Apply with registry
        let result =
            apply_mutations_with_extensions(&graph, vec![mutation], options, Some(&registry));

        assert!(result.is_ok());
        let mutated = result.unwrap();
        // Check via nodes array since root field is a cached copy
        let root_node = mutated.nodes.iter().find(|n| n.istopnode).unwrap();
        assert_eq!(root_node.name, "[PREFIX] Root [SUFFIX]");
    }

    #[test]
    fn test_extension_mutation_without_registry() {
        let graph = create_test_graph();
        let options = MutatorOptions::default();

        // Create extension mutation
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: "test.some_mutation".to_string(),
            params: serde_json::json!({}),
            conformance: MutationConformance::AlwaysConformant,
        });

        // Apply without registry
        let result = apply_mutations(&graph, vec![mutation], options);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no registry provided"));
    }

    #[test]
    fn test_extension_mutation_not_found() {
        let graph = create_test_graph();
        let options = MutatorOptions::default();

        // Create empty registry
        let registry = ExtensionMutationRegistry::new();

        // Create extension mutation for non-existent handler
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: "test.nonexistent".to_string(),
            params: serde_json::json!({}),
            conformance: MutationConformance::AlwaysConformant,
        });

        // Apply with empty registry
        let result =
            apply_mutations_with_extensions(&graph, vec![mutation], options, Some(&registry));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_extension_registry_operations() {
        let mut registry = ExtensionMutationRegistry::new();

        assert!(!registry.has("test.handler"));
        assert!(registry.list().is_empty());

        registry.register(
            "test.handler",
            std::sync::Arc::new(TestPrefixHandler {
                prefix: "x".to_string(),
            }),
        );

        assert!(registry.has("test.handler"));
        assert!(!registry.has("test.other"));
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("test.handler").is_some());
    }

    #[test]
    fn test_extension_mutation_conformance() {
        // Test that conformance is read from params
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: "test.mutation".to_string(),
            params: serde_json::json!({}),
            conformance: MutationConformance::BranchConformant,
        });

        assert_eq!(
            mutation.conformance(),
            MutationConformance::BranchConformant
        );

        let mutation2 = GraphMutation::Extension(ExtensionMutationParams {
            name: "test.mutation".to_string(),
            params: serde_json::json!({}),
            conformance: MutationConformance::ModelConformant,
        });

        assert_eq!(
            mutation2.conformance(),
            MutationConformance::ModelConformant
        );
    }

    #[test]
    fn test_extension_mutation_serialization() {
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: "clm.reference_change_collection".to_string(),
            params: serde_json::json!({
                "node_id": "my_node",
                "collection_id": "new-collection"
            }),
            conformance: MutationConformance::AlwaysConformant,
        });

        // Serialize
        let json = serde_json::to_string(&mutation).unwrap();
        assert!(json.contains("clm.reference_change_collection"));
        assert!(json.contains("my_node"));

        // Deserialize
        let parsed: GraphMutation = serde_json::from_str(&json).unwrap();
        if let GraphMutation::Extension(params) = parsed {
            assert_eq!(params.name, "clm.reference_change_collection");
            assert_eq!(params.params["node_id"], "my_node");
        } else {
            panic!("Expected Extension mutation");
        }
    }

    #[test]
    fn test_extension_mutation_from_json() {
        let graph = create_test_graph();

        let mut registry = ExtensionMutationRegistry::new();
        registry.register(
            "test.prefix_name",
            std::sync::Arc::new(TestPrefixHandler {
                prefix: "[TEST] ".to_string(),
            }),
        );

        let mutations_json = r#"{
            "mutations": [{
                "Extension": {
                    "name": "test.prefix_name",
                    "params": {"suffix": "!"},
                    "conformance": "AlwaysConformant"
                }
            }],
            "options": {}
        }"#;

        let result =
            apply_mutations_from_json_with_extensions(&graph, mutations_json, Some(&registry));

        assert!(result.is_ok());
        let mutated = result.unwrap();
        // Check via nodes array since root field is a cached copy
        let root_node = mutated.nodes.iter().find(|n| n.istopnode).unwrap();
        assert_eq!(root_node.name, "[TEST] Root!");
    }

    // =========================================================================
    // RenameGraph Tests
    // =========================================================================

    #[test]
    fn test_rename_graph() {
        let mut graph = create_skeleton_graph("Test Graph", "test", false, None);
        let options = MutatorOptions::default();

        // Verify initial state
        assert_eq!(graph.name.get("en"), "Test Graph");
        assert!(graph.description.is_none());
        assert!(graph.subtitle.is_none());
        assert!(graph.author.is_none());

        // Rename the graph with all fields
        let mut name_map = HashMap::new();
        name_map.insert("en".to_string(), "New Name".to_string());
        name_map.insert("es".to_string(), "Nuevo Nombre".to_string());

        let mut desc_map = HashMap::new();
        desc_map.insert("en".to_string(), "A description".to_string());

        let mut subtitle_map = HashMap::new();
        subtitle_map.insert("en".to_string(), "A subtitle".to_string());

        apply_mutation(
            &mut graph,
            GraphMutation::RenameGraph(RenameGraphParams {
                name: Some(name_map),
                description: Some(desc_map),
                subtitle: Some(subtitle_map),
                author: Some("Test Author".to_string()),
            }),
            &options,
        )
        .unwrap();

        // Verify all fields were updated
        assert_eq!(graph.name.get("en"), "New Name");
        assert_eq!(graph.name.translations.get("es").unwrap(), "Nuevo Nombre");
        assert!(graph.description.is_some());
        assert_eq!(
            graph.description.as_ref().unwrap().get("en"),
            "A description"
        );
        assert!(graph.subtitle.is_some());
        assert_eq!(graph.subtitle.as_ref().unwrap().get("en"), "A subtitle");
        assert_eq!(graph.author, Some("Test Author".to_string()));

        // Verify root node name matches graph name
        assert_eq!(graph.root.name, "New Name");
        let root_in_nodes = graph.nodes.iter().find(|n| n.istopnode).unwrap();
        assert_eq!(root_in_nodes.name, "New Name");

        // Verify slug and alias are updated
        assert_eq!(graph.slug, Some("new_name".to_string()));
        assert_eq!(graph.root.alias, Some("new_name".to_string()));
        assert_eq!(root_in_nodes.alias, Some("new_name".to_string()));
    }

    #[test]
    fn test_rename_graph_partial() {
        let mut graph = create_skeleton_graph("Original Name", "test", false, None);
        let options = MutatorOptions::default();

        // Only update description, leave name unchanged
        let mut desc_map = HashMap::new();
        desc_map.insert("en".to_string(), "New description".to_string());

        apply_mutation(
            &mut graph,
            GraphMutation::RenameGraph(RenameGraphParams {
                name: None,
                description: Some(desc_map),
                subtitle: None,
                author: None,
            }),
            &options,
        )
        .unwrap();

        // Name should be unchanged
        assert_eq!(graph.name.get("en"), "Original Name");
        // Description should be updated
        assert!(graph.description.is_some());
        assert_eq!(
            graph.description.as_ref().unwrap().get("en"),
            "New description"
        );
    }

    #[test]
    fn test_rename_graph_via_instruction() {
        let mut graph = create_skeleton_graph("Test Graph", "test", false, None);
        let options = MutatorOptions::default();

        // Use instruction pattern: action=rename_graph, subject=graphid, object=new name (simple string)
        let instruction = GraphInstruction::new("rename_graph", "test", "New Graph Name")
            .with_str("author", "Instruction Author");

        // Verify conformance
        assert_eq!(
            instruction.conformance(),
            MutationConformance::AlwaysConformant
        );

        let mutation = instruction.to_mutation().unwrap();
        apply_mutation(&mut graph, mutation, &options).unwrap();

        // Name should be updated (object becomes English name)
        assert_eq!(graph.name.get("en"), "New Graph Name");
        assert_eq!(graph.author, Some("Instruction Author".to_string()));
    }

    #[test]
    fn test_rename_graph_via_instruction_multilingual() {
        let mut graph = create_skeleton_graph("Test Graph", "test", false, None);
        let options = MutatorOptions::default();

        // Use instruction with translatable map in params
        let mut name_obj = serde_json::Map::new();
        name_obj.insert(
            "en".to_string(),
            serde_json::Value::String("English Name".to_string()),
        );
        name_obj.insert(
            "de".to_string(),
            serde_json::Value::String("Deutscher Name".to_string()),
        );

        let mut desc_obj = serde_json::Map::new();
        desc_obj.insert(
            "en".to_string(),
            serde_json::Value::String("English description".to_string()),
        );

        let instruction = GraphInstruction::new("rename_graph", "test", "")
            .with_param("name", serde_json::Value::Object(name_obj))
            .with_param("description", serde_json::Value::Object(desc_obj));

        let mutation = instruction.to_mutation().unwrap();
        apply_mutation(&mut graph, mutation, &options).unwrap();

        // Verify multilingual name
        assert_eq!(graph.name.get("en"), "English Name");
        assert_eq!(graph.name.translations.get("de").unwrap(), "Deutscher Name");
        // Verify description
        assert!(graph.description.is_some());
        assert_eq!(
            graph.description.as_ref().unwrap().get("en"),
            "English description"
        );
    }
}
