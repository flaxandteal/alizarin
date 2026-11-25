//! Tile type for storing data instances. Tile-id derivation, which needs
//! uuid, stays in alizarin-core (`graph::canonical_tile_id` and friends).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A tile containing data for a nodegroup instance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticTile {
    /// Node id -> value. A `BTreeMap` so every serialisation, on every
    /// backend, writes keys in the same (sorted) order; for the handful of
    /// keys a tile holds it is also as fast as a `HashMap` to look up.
    #[serde(default)]
    pub data: BTreeMap<String, serde_json::Value>,
    pub nodegroup_id: String,
    pub resourceinstance_id: String,
    // Arches' shape, on every backend: the keys Arches' business-data importer
    // indexes directly (`tileid`, `parenttile_id`, `sortorder`) are always
    // written, `null` when empty; `provisionaledits`, which it doesn't read,
    // is left out when empty. See notes/backend-support-matrix.md.
    #[serde(default)]
    pub tileid: Option<String>,
    #[serde(default)]
    pub parenttile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisionaledits: Option<Vec<serde_json::Value>>,
    #[serde(default)]
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
            data: BTreeMap::new(),
        }
    }
}
