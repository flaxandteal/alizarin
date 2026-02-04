//! Card-related types for UI configuration.

use super::translatable::StaticTranslatableString;
use serde::{Deserialize, Serialize};

/// A constraint on a card limiting how nodes can be used
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticConstraint {
    pub card_id: String,
    pub constraintid: String,
    pub nodes: Vec<String>,
    pub uniquetoallinstances: bool,
}

/// Publication information for a graph
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticPublication {
    pub graph_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub publicationid: String,
    pub published_time: String,
}

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
    // Arches-HER 2.0+ fields
    /// Source identifier for import/export tracking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_identifier_id: Option<String>,
}

/// Mapping between functions and graphs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticFunctionsXGraphs {
    #[serde(default)]
    pub config: serde_json::Value,
    pub function_id: String,
    pub graph_id: String,
    pub id: String,
}

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
    #[serde(default)]
    pub is_editable: Option<bool>,
    pub name: StaticTranslatableString,
    pub nodegroup_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortorder: Option<i32>,
    pub visible: bool,
    // Arches-HER 2.0+ fields
    /// Source identifier for import/export tracking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_identifier_id: Option<String>,
}
