use wasm_bindgen::prelude::*;
use std::collections::{HashMap, HashSet};
use serde_json::Value as JsonValue;
use crate::graph::{StaticTile, StaticNode};
use js_sys::{Array, Map as JsMap};
use wasm_bindgen::JsCast;

/// Recipe for creating a PseudoValue in JavaScript
/// Contains all information needed to instantiate a Pseudo without holding full tile data
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PseudoRecipe {
    node_alias: String,
    tile_id: Option<String>,
    node_id: String,
    is_collector: bool,
    sentinel_undefined: bool,
}

#[wasm_bindgen]
impl PseudoRecipe {
    #[wasm_bindgen(getter = nodeAlias)]
    pub fn node_alias(&self) -> String {
        self.node_alias.clone()
    }

    #[wasm_bindgen(getter = tileId)]
    pub fn tile_id(&self) -> Option<String> {
        self.tile_id.clone()
    }

    #[wasm_bindgen(getter = nodeId)]
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }

    #[wasm_bindgen(getter = isCollector)]
    pub fn is_collector(&self) -> bool {
        self.is_collector
    }

    #[wasm_bindgen(getter = sentinelUndefined)]
    pub fn sentinel_undefined(&self) -> bool {
        self.sentinel_undefined
    }
}

/// Result from values_from_resource_nodegroup
#[wasm_bindgen]
#[derive(Clone)]
pub struct ValuesFromNodegroupResult {
    recipes: Vec<PseudoRecipe>,
    implied_nodegroups: Vec<String>,
}

#[wasm_bindgen]
impl ValuesFromNodegroupResult {
    #[wasm_bindgen(getter = recipes)]
    pub fn recipes(&self) -> Vec<PseudoRecipe> {
        self.recipes.clone()
    }

    #[wasm_bindgen(getter = impliedNodegroups)]
    pub fn implied_nodegroups(&self) -> Vec<String> {
        self.implied_nodegroups.clone()
    }
}

/// Rust-side resource instance wrapper that owns tile data
/// Manages tile storage, indexing, and provides query methods
#[wasm_bindgen]
#[derive(Clone)]
pub struct WASMResourceInstanceWrapper {
    // Core tile storage
    tiles: HashMap<String, StaticTile>,

    // Index: nodegroup_id -> list of tile_ids
    nodegroup_index: HashMap<String, Vec<String>>,

    // Track which nodegroups have been loaded
    loaded_nodegroups: HashMap<String, bool>,
}

#[wasm_bindgen]
impl WASMResourceInstanceWrapper {
    /// Create a new empty resource instance wrapper
    #[wasm_bindgen(constructor)]
    pub fn new() -> WASMResourceInstanceWrapper {
        WASMResourceInstanceWrapper {
            tiles: HashMap::new(),
            nodegroup_index: HashMap::new(),
            loaded_nodegroups: HashMap::new(),
        }
    }

    /// Load tiles from JavaScript into Rust storage
    /// This is called during ResourceInstanceWrapper construction
    #[wasm_bindgen(js_name = loadTiles)]
    pub fn load_tiles(&mut self, tiles_js: JsValue) -> Result<(), JsValue> {
        // Deserialize JS array of tiles
        let tiles: Vec<StaticTile> = serde_wasm_bindgen::from_value(tiles_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize tiles: {:?}", e)))?;

        // Clear existing data
        self.tiles.clear();
        self.nodegroup_index.clear();

        // Store tiles and build index
        for mut tile in tiles {
            // Ensure tile has an ID
            let tile_id = tile.ensure_id();
            let nodegroup_id = tile.nodegroup_id.clone();

            // Add to nodegroup index
            self.nodegroup_index
                .entry(nodegroup_id)
                .or_insert_with(Vec::new)
                .push(tile_id.clone());

            // Store tile
            self.tiles.insert(tile_id, tile);
        }

        Ok(())
    }

    /// Get tile IDs for a specific nodegroup
    /// Returns array of tile ID strings
    #[wasm_bindgen(js_name = getTileIdsByNodegroup)]
    pub fn get_tile_ids_by_nodegroup(&self, nodegroup_id: &str) -> Vec<String> {
        self.nodegroup_index
            .get(nodegroup_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get full tile data by tile ID
    /// Returns StaticTile WASM object or error if not found
    #[wasm_bindgen(js_name = getTile)]
    pub fn get_tile(&self, tile_id: &str) -> Result<StaticTile, JsValue> {
        self.tiles
            .get(tile_id)
            .cloned()
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))
    }

    /// Get specific node data from a tile
    /// Returns the data value for the given node_id within the tile
    #[wasm_bindgen(js_name = getTileData)]
    pub fn get_tile_data(&self, tile_id: &str, node_id: &str) -> Result<JsValue, JsValue> {
        let tile = self.tiles
            .get(tile_id)
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))?;

        // Get data for specific node from tile.data HashMap
        match tile.data.get(node_id) {
            Some(value) => serde_wasm_bindgen::to_value(value)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize data: {:?}", e))),
            None => Ok(JsValue::NULL),
        }
    }

    /// Mark a nodegroup as loaded to prevent re-loading
    #[wasm_bindgen(js_name = markNodegroupLoaded)]
    pub fn mark_nodegroup_loaded(&mut self, nodegroup_id: String) {
        self.loaded_nodegroups.insert(nodegroup_id, true);
    }

    /// Check if a nodegroup has been loaded
    #[wasm_bindgen(js_name = isNodegroupLoaded)]
    pub fn is_nodegroup_loaded(&self, nodegroup_id: &str) -> bool {
        self.loaded_nodegroups.get(nodegroup_id).copied().unwrap_or(false)
    }

    /// Get count of tiles stored
    #[wasm_bindgen(js_name = getTileCount)]
    pub fn get_tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Get count of nodegroups indexed
    #[wasm_bindgen(js_name = getNodegroupCount)]
    pub fn get_nodegroup_count(&self) -> usize {
        self.nodegroup_index.len()
    }

    // ========================================================================
    // Phase 2: Graph Traversal Helpers
    // ========================================================================

    /// PORT: graphManager.ts lines 505-643
    /// Returns recipes for creating PseudoValues and discovered implied nodegroups
    #[wasm_bindgen(js_name = valuesFromResourceNodegroup)]
    pub fn values_from_resource_nodegroup(
        &self,
        existing_values_js: JsValue,  // Map<string, any>
        nodegroup_tile_ids: Vec<String>,  // Tile IDs for this nodegroup (nulls become None)
        nodegroup_id: &str,
        node_objs_js: JsValue,  // Map<string, StaticNode>
        edges_js: JsValue,  // Map<string, string[]>
    ) -> Result<ValuesFromNodegroupResult, JsValue> {
        // Deserialize JS Maps to Rust HashMaps
        let node_objs = self.deserialize_node_map(node_objs_js)?;
        let edges = self.deserialize_edges_map(edges_js)?;
        let existing_values = self.deserialize_existing_values(existing_values_js)?;

        // Get tiles for this nodegroup
        let mut nodegroup_tiles: Vec<Option<&StaticTile>> = Vec::new();
        for tile_id in &nodegroup_tile_ids {
            if tile_id.is_empty() {
                nodegroup_tiles.push(None); // null tile
            } else {
                let tile = self.tiles.get(tile_id);
                nodegroup_tiles.push(tile);
            }
        }

        // PORT: Line 514
        let mut implied_nodegroups: HashSet<String> = HashSet::new();
        // PORT: Line 515
        let mut implied_nodes: HashMap<String, (StaticNode, String)> = HashMap::new(); // key = nodeid+tileid, value = (node, tileid)

        // PORT: Lines 517-521 - Track unseen nodes
        let mut nodes_unseen: HashSet<String> = node_objs
            .values()
            .filter(|node| node.nodegroup_id.as_deref() == Some(nodegroup_id))
            .filter_map(|node| node.alias.clone())
            .collect();

        // PORT: Line 522 - Track seen tile-node pairs
        let mut tile_nodes_seen: HashSet<(String, String)> = HashSet::new(); // (nodeid, tileid)

        let mut recipes: Vec<PseudoRecipe> = Vec::new();

        // Helper to add a pseudo recipe
        // PORT: Lines 523-582
        let mut add_pseudo = |node: &StaticNode, tile: Option<&StaticTile>| {
            let key = node.alias.clone().unwrap_or_default();

            // PORT: Line 525
            nodes_unseen.remove(&key);

            // PORT: Lines 526-528
            let tileid: Option<String> = tile.and_then(|t| t.tileid.clone());
            if let Some(ref tid) = tileid {
                tile_nodes_seen.insert((node.nodeid.clone(), tid.clone()));
            }

            // PORT: Lines 530-537 - Check if already exists
            if let Some(existing) = existing_values.get(&key) {
                if !existing.is_false && !existing.is_undefined {
                    // Already loaded, skip
                    return;
                }
            }

            // PORT: Lines 546-563 - Discover implied nodegroups and nodes
            for (domain, ranges) in &edges {
                if ranges.contains(&node.nodeid) {
                    if let Some(domain_node) = node_objs.get(domain) {
                        // PORT: Lines 552-557 - Implied nodegroup
                        if let Some(ref ng_id) = domain_node.nodegroup_id {
                            if !ng_id.is_empty() && ng_id != nodegroup_id {
                                implied_nodegroups.insert(ng_id.clone());
                            }
                        }

                        // PORT: Lines 558-560 - Implied node (shares tile)
                        if let (Some(ref domain_ng), Some(tile), Some(ref tid)) =
                            (&domain_node.nodegroup_id, tile, &tileid) {
                            if domain_ng == &tile.nodegroup_id
                                && domain_ng != &domain_node.nodeid
                                && !implied_nodes.contains_key(&format!("{}{}", domain_node.nodeid, tid)) {
                                implied_nodes.insert(
                                    format!("{}{}", domain_node.nodeid, tid),
                                    (domain_node.clone(), tid.clone())
                                );
                            }
                        }
                    }
                    break;
                }
            }

            // PORT: Lines 542, 581 - Add the recipe
            recipes.push(PseudoRecipe {
                node_alias: key,
                tile_id: tileid,
                node_id: node.nodeid.clone(),
                is_collector: node.is_collector,
                sentinel_undefined: false,
            });
        };

        // PORT: Lines 584-621 - Main loop over tiles
        for tile_opt in nodegroup_tiles {
            // PORT: Lines 585-591 - Add parent node
            if let Some(parent_node) = node_objs.get(nodegroup_id) {
                if parent_node.nodegroup_id.as_deref().unwrap_or("") == ""
                    || parent_node.nodegroup_id.as_deref() == Some(nodegroup_id) {
                    add_pseudo(parent_node, tile_opt);
                }
            }

            // PORT: Lines 593-620 - Process tile data
            if let Some(tile) = tile_opt {
                // PORT: Lines 594-597 - Get tile nodes
                let mut tile_nodes: HashMap<String, JsonValue> = HashMap::new();
                for (key, value) in &tile.data {
                    tile_nodes.insert(key.clone(), value.clone());
                }

                // PORT: Lines 599-602 - Add semantic nodes without data
                for node in node_objs.values() {
                    if node.nodegroup_id.as_deref() == Some(nodegroup_id)
                        && !tile_nodes.contains_key(&node.nodeid)
                        && node.datatype == "semantic" {
                        tile_nodes.insert(node.nodeid.clone(), JsonValue::Object(serde_json::Map::new()));
                    }
                }

                // PORT: Lines 604-606 - Ensure nodegroup_id is in tile_nodes
                if !tile_nodes.contains_key(&tile.nodegroup_id) {
                    tile_nodes.insert(tile.nodegroup_id.clone(), JsonValue::Object(serde_json::Map::new()));
                }

                // PORT: Lines 607-620 - Add pseudo for each tile node
                for (nodeid, node_value) in tile_nodes {
                    // PORT: Lines 608-611 - Skip nodegroup node
                    if nodeid == nodegroup_id {
                        continue;
                    }

                    if let Some(node) = node_objs.get(&nodeid) {
                        // PORT: Line 616 - Only add if value is not null
                        if !node_value.is_null() {
                            add_pseudo(node, Some(tile));
                        }
                    }
                }
            }
        }

        // PORT: Lines 622-632 - Process implied nodes
        // Note: Can't use add_pseudo closure due to borrow checker, inline the logic
        while !implied_nodes.is_empty() {
            let key = implied_nodes.keys().next().unwrap().clone();
            let (node, tileid) = implied_nodes.remove(&key).unwrap();

            // PORT: Line 627 - Only add if not seen
            if !tile_nodes_seen.contains(&(node.nodeid.clone(), tileid.clone())) {
                let tile = self.tiles.get(&tileid);

                // Inline add_pseudo logic
                let key = node.alias.clone().unwrap_or_default();
                nodes_unseen.remove(&key);

                // tileid is already a String here (from the tuple), not Option
                tile_nodes_seen.insert((node.nodeid.clone(), tileid.clone()));

                // Check if already exists
                if let Some(existing) = existing_values.get(&key) {
                    if existing.is_false || existing.is_undefined {
                        // Can add
                        recipes.push(PseudoRecipe {
                            node_alias: key.clone(),
                            tile_id: Some(tileid.clone()),
                            node_id: node.nodeid.clone(),
                            is_collector: node.is_collector,
                            sentinel_undefined: false,
                        });
                    }
                } else {
                    // Not exists, add
                    recipes.push(PseudoRecipe {
                        node_alias: key,
                        tile_id: Some(tileid),
                        node_id: node.nodeid.clone(),
                        is_collector: node.is_collector,
                        sentinel_undefined: false,
                    });
                }
            }
        }

        // PORT: Lines 635-641 - Mark unseen nodes as undefined (sentinel)
        for node_unseen in nodes_unseen {
            if !node_unseen.is_empty() {
                recipes.push(PseudoRecipe {
                    node_alias: node_unseen,
                    tile_id: None,
                    node_id: String::new(),
                    is_collector: false,
                    sentinel_undefined: true,  // Special flag for undefined sentinel
                });
            }
        }

        Ok(ValuesFromNodegroupResult {
            recipes,
            implied_nodegroups: implied_nodegroups.into_iter().collect(),
        })
    }
}

// ============================================================================
// Helper implementations
// ============================================================================

/// Helper struct to represent existing values from JS
#[derive(Debug)]
struct ExistingValue {
    is_false: bool,
    is_undefined: bool,
}

impl WASMResourceInstanceWrapper {
    /// Deserialize JS Map<string, StaticNode> to Rust HashMap
    fn deserialize_node_map(&self, js_map: JsValue) -> Result<HashMap<String, StaticNode>, JsValue> {
        let map = JsMap::from(js_map);
        let mut result = HashMap::new();

        map.for_each(&mut |value, key| {
            if let Some(key_str) = key.as_string() {
                if let Ok(node) = serde_wasm_bindgen::from_value::<StaticNode>(value) {
                    result.insert(key_str, node);
                }
            }
        });

        Ok(result)
    }

    /// Deserialize JS Map<string, string[]> to Rust HashMap
    fn deserialize_edges_map(&self, js_map: JsValue) -> Result<HashMap<String, Vec<String>>, JsValue> {
        let map = JsMap::from(js_map);
        let mut result = HashMap::new();

        map.for_each(&mut |value, key| {
            if let Some(key_str) = key.as_string() {
                if let Ok(arr) = value.dyn_into::<Array>() {
                    let vec: Vec<String> = (0..arr.length())
                        .filter_map(|i| arr.get(i).as_string())
                        .collect();
                    result.insert(key_str, vec);
                }
            }
        });

        Ok(result)
    }

    /// Deserialize existing values map (JS Map<string, any>)
    fn deserialize_existing_values(&self, js_map: JsValue) -> Result<HashMap<String, ExistingValue>, JsValue> {
        let map = JsMap::from(js_map);
        let mut result = HashMap::new();

        map.for_each(&mut |value, key| {
            if let Some(key_str) = key.as_string() {
                let existing = ExistingValue {
                    is_false: value.is_falsy() && !value.is_undefined(),
                    is_undefined: value.is_undefined(),
                };
                result.insert(key_str, existing);
            }
        });

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_storage() {
        let mut wrapper = WASMResourceInstanceWrapper::new();
        assert_eq!(wrapper.get_tile_count(), 0);

        // Test will be expanded when we have tile data
    }
}
