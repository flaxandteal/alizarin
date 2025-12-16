//! Tile type for storing data instances.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
