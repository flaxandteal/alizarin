//! WASM bindings for label resolution functions.
//!
//! These wrappers expose the platform-agnostic label resolution logic
//! from alizarin-core to TypeScript/JavaScript via WASM.
//!
//! The approach is designed to work with JS-managed collections:
//! 1. JS calls `buildAliasToCollectionMap` to get alias -> collectionId mapping
//! 2. JS calls `findNeededCollections` to identify which collections to load
//! 3. JS loads collections and builds a lookup table (collectionId -> label -> conceptId)
//! 4. JS calls `resolveLabelsWithLookup` with the lookup table

use alizarin_core::label_resolution::{
    self, build_alias_to_collection_map, find_needed_collections, ConceptLookup,
    LabelResolutionConfig,
};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Build a mapping from node alias to collection ID.
///
/// @param graphJson - JSON string of the graph definition
/// @param resolvableDatatypes - Array of datatypes to resolve (default: concept, concept-list)
/// @param configKeys - Array of config keys to check (default: rdmCollection, controlledList)
/// @returns Map<string, string> of alias -> collectionId
#[wasm_bindgen(js_name = buildAliasToCollectionMap)]
pub fn wasm_build_alias_to_collection_map(
    graph_json: &str,
    resolvable_datatypes: Option<Vec<String>>,
    config_keys: Option<Vec<String>>,
) -> Result<JsValue, JsError> {
    let config = LabelResolutionConfig {
        resolvable_datatypes: resolvable_datatypes.unwrap_or_else(|| {
            label_resolution::DEFAULT_RESOLVABLE_DATATYPES
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        config_keys: config_keys.unwrap_or_else(|| {
            label_resolution::DEFAULT_CONFIG_KEYS
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        strict: false,
    };

    let graph: serde_json::Value = serde_json::from_str(graph_json)
        .map_err(|e| JsError::new(&format!("Invalid graph JSON: {}", e)))?;

    let alias_map = build_alias_to_collection_map(&graph, &config);

    serde_wasm_bindgen::to_value(&alias_map)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Find which collections are needed to resolve labels in a tree.
///
/// @param treeJson - JSON string of the tree to scan
/// @param aliasToCollection - Map<string, string> from buildAliasToCollectionMap
/// @returns Array<string> of collection IDs that need to be loaded
#[wasm_bindgen(js_name = findNeededCollections)]
pub fn wasm_find_needed_collections(
    tree_json: &str,
    alias_to_collection: JsValue,
) -> Result<Vec<String>, JsError> {
    let tree: serde_json::Value = serde_json::from_str(tree_json)
        .map_err(|e| JsError::new(&format!("Invalid tree JSON: {}", e)))?;

    let alias_map: HashMap<String, String> = serde_wasm_bindgen::from_value(alias_to_collection)
        .map_err(|e| JsError::new(&format!("Invalid alias map: {}", e)))?;

    let needed = find_needed_collections(&tree, &alias_map);

    Ok(needed.into_iter().collect())
}

/// Lookup implementation that uses a pre-built table.
/// The table structure is: Map<collectionId, Map<lowercase_label, conceptId>>
struct TableLookup {
    table: HashMap<String, HashMap<String, String>>,
}

impl ConceptLookup for TableLookup {
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<String> {
        self.table
            .get(collection_id)?
            .get(&label.to_lowercase())
            .cloned()
    }
}

/// Resolve labels to UUIDs using a pre-built lookup table.
///
/// @param treeJson - JSON string of the tree to resolve
/// @param aliasToCollection - Map<string, string> from buildAliasToCollectionMap
/// @param lookupTable - Map<collectionId, Map<label, conceptId>> built from collections
/// @param strict - If true, return error for unresolved labels
/// @returns JSON string with labels resolved to UUIDs
#[wasm_bindgen(js_name = resolveLabelsWithLookup)]
pub fn wasm_resolve_labels_with_lookup(
    tree_json: &str,
    alias_to_collection: JsValue,
    lookup_table: JsValue,
    strict: bool,
) -> Result<String, JsError> {
    let tree: serde_json::Value = serde_json::from_str(tree_json)
        .map_err(|e| JsError::new(&format!("Invalid tree JSON: {}", e)))?;

    let alias_map: HashMap<String, String> = serde_wasm_bindgen::from_value(alias_to_collection)
        .map_err(|e| JsError::new(&format!("Invalid alias map: {}", e)))?;

    let table: HashMap<String, HashMap<String, String>> =
        serde_wasm_bindgen::from_value(lookup_table)
            .map_err(|e| JsError::new(&format!("Invalid lookup table: {}", e)))?;

    let lookup = TableLookup { table };

    let resolved = label_resolution::resolve_labels(tree, &alias_map, &lookup, strict)
        .map_err(|e| JsError::new(&e.message))?;

    serde_json::to_string(&resolved)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Check if a string is a valid UUID.
///
/// @param s - String to check
/// @returns true if valid UUID
#[wasm_bindgen(js_name = isValidUuid)]
pub fn wasm_is_valid_uuid(s: &str) -> bool {
    label_resolution::is_valid_uuid(s)
}

/// Get the default resolvable datatypes.
///
/// @returns Array of datatype strings: ["concept", "concept-list"]
#[wasm_bindgen(js_name = getDefaultResolvableDatatypes)]
pub fn wasm_get_default_resolvable_datatypes() -> Vec<String> {
    label_resolution::DEFAULT_RESOLVABLE_DATATYPES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Get the default config keys for collection ID lookup.
///
/// @returns Array of config key strings: ["rdmCollection", "controlledList"]
#[wasm_bindgen(js_name = getDefaultConfigKeys)]
pub fn wasm_get_default_config_keys() -> Vec<String> {
    label_resolution::DEFAULT_CONFIG_KEYS
        .iter()
        .map(|s| s.to_string())
        .collect()
}
