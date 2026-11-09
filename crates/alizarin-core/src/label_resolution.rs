//! Label Resolution Module
//!
//! Provides core label-to-UUID resolution logic shared between WASM (JS) and PyO3 (Python).
//!
//! This module handles:
//! - Scanning JSON trees to identify which collections are needed
//! - Building alias -> collection ID mappings from graph definitions
//! - Resolving label strings to UUIDs using collection lookups
//! - UUID validation and passthrough

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Datatypes that support label resolution by default.
/// Extension types (e.g. "reference") should register themselves
/// via the extension handler mechanism rather than being listed here.
pub const DEFAULT_RESOLVABLE_DATATYPES: &[&str] = &[
    "concept",
    "concept-list",
    "domain-value",
    "domain-value-list",
];

/// Config keys that hold collection IDs for core datatypes (in order of
/// preference). An extension datatype's collection comes from its handler
/// ([`collection_id`](crate::extension_type_registry::ExtensionTypeHandler::collection_id)).
pub const DEFAULT_CONFIG_KEYS: &[&str] = &["rdmCollection"];

/// Error type for label resolution
#[derive(Debug, Clone)]
pub struct LabelResolutionError {
    pub message: String,
    pub errors: Vec<String>,
}

impl std::fmt::Display for LabelResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LabelResolutionError {}

/// Trait for looking up concepts by label in a collection.
///
/// This trait abstracts over the actual collection storage (RdmCache in Python,
/// StaticCollection in JS) so the resolution logic can be shared.
pub trait ConceptLookup: Send + Sync {
    /// Look up a concept ID by its label in a specific collection.
    /// Returns None if not found or ambiguous.
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<String>;
}

/// Check if a string is a valid UUID
#[inline]
pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

/// Configuration for label resolution
#[derive(Clone, Debug)]
pub struct LabelResolutionConfig {
    /// Datatypes that should have their labels resolved
    pub resolvable_datatypes: Vec<String>,
    /// Config keys to check for collection IDs
    pub config_keys: Vec<String>,
    /// If true, return errors for unresolved labels. If false, pass through.
    pub strict: bool,
}

impl Default for LabelResolutionConfig {
    fn default() -> Self {
        Self {
            resolvable_datatypes: DEFAULT_RESOLVABLE_DATATYPES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            config_keys: DEFAULT_CONFIG_KEYS.iter().map(|s| s.to_string()).collect(),
            strict: false,
        }
    }
}

impl LabelResolutionConfig {
    /// Create a new config with custom datatypes and keys
    pub fn new(resolvable_datatypes: Vec<String>, config_keys: Vec<String>, strict: bool) -> Self {
        Self {
            resolvable_datatypes,
            config_keys,
            strict,
        }
    }

    /// Add additional resolvable datatypes
    pub fn with_additional_datatypes(mut self, datatypes: &[&str]) -> Self {
        for dt in datatypes {
            if !self.resolvable_datatypes.contains(&dt.to_string()) {
                self.resolvable_datatypes.push(dt.to_string());
            }
        }
        self
    }

    /// Add additional config keys
    pub fn with_additional_config_keys(mut self, keys: &[&str]) -> Self {
        for key in keys {
            if !self.config_keys.contains(&key.to_string()) {
                self.config_keys.push(key.to_string());
            }
        }
        self
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

/// Build a mapping from node alias to collection ID based on graph definition.
///
/// Returns a HashMap where keys are node aliases and values are collection IDs
/// for nodes with resolvable datatypes. A node whose datatype has a handler in
/// `registry` takes its collection from the handler; otherwise from the first
/// of `config.config_keys` present in its config.
pub fn build_alias_to_collection_map(
    graph: &Value,
    config: &LabelResolutionConfig,
    registry: Option<&crate::extension_type_registry::ExtensionTypeRegistry>,
) -> HashMap<String, String> {
    let mut alias_to_collection: HashMap<String, String> = HashMap::new();

    // Handle wrapped graph format: {graph: [graphDef]} or direct graphDef
    let graph_def = if let Some(graphs) = graph.get("graph").and_then(|g| g.as_array()) {
        graphs.first().cloned().unwrap_or(graph.clone())
    } else {
        graph.clone()
    };

    // Get nodes array
    let nodes = match graph_def.get("nodes").and_then(|n| n.as_array()) {
        Some(n) => n,
        None => return alias_to_collection,
    };

    let resolvable_set: HashSet<&str> = config
        .resolvable_datatypes
        .iter()
        .map(|s| s.as_str())
        .collect();

    for node in nodes {
        let alias = match node.get("alias").and_then(|a| a.as_str()) {
            Some(a) => a,
            None => continue,
        };

        let datatype = match node.get("datatype").and_then(|d| d.as_str()) {
            Some(d) => d,
            None => continue,
        };

        if !resolvable_set.contains(datatype) {
            continue;
        }

        let node_config = match node.get("config") {
            Some(c) => c,
            None => continue,
        };

        if let Some(handler) = registry.and_then(|r| r.get(datatype)) {
            if let Some(collection_id) = handler.collection_id(node_config) {
                alias_to_collection.insert(alias.to_string(), collection_id);
                continue;
            }
        }

        // Check config keys in order of preference
        for key in &config.config_keys {
            if let Some(collection_id) = node_config.get(key).and_then(|v| v.as_str()) {
                alias_to_collection.insert(alias.to_string(), collection_id.to_string());
                break;
            }
        }
    }

    alias_to_collection
}

/// [`build_alias_to_collection_map`] over a parsed graph, with the default
/// config keys. Every datatype with a handler in `registry` is resolvable,
/// alongside [`DEFAULT_RESOLVABLE_DATATYPES`].
pub fn alias_to_collection_for_graph(
    graph: &crate::graph::StaticGraph,
    registry: Option<&crate::extension_type_registry::ExtensionTypeRegistry>,
) -> HashMap<String, String> {
    let mut alias_to_collection = HashMap::new();
    for node in &graph.nodes {
        let Some(alias) = &node.alias else { continue };
        let handler = registry.and_then(|r| r.get(&node.datatype));
        if handler.is_none() && !DEFAULT_RESOLVABLE_DATATYPES.contains(&node.datatype.as_str()) {
            continue;
        }
        let collection_id = match handler {
            Some(handler) => serde_json::to_value(&node.config)
                .ok()
                .and_then(|config| handler.collection_id(&config)),
            None => None,
        }
        .or_else(|| {
            DEFAULT_CONFIG_KEYS
                .iter()
                .find_map(|key| node.config.get(*key).and_then(|v| v.as_str()))
                .map(str::to_string)
        });
        if let Some(collection_id) = collection_id {
            alias_to_collection.insert(alias.clone(), collection_id);
        }
    }
    alias_to_collection
}

/// Scan a JSON tree to find which collections are needed for resolution.
///
/// Returns a set of collection IDs that appear in the tree for resolvable fields.
pub fn find_needed_collections(
    tree: &Value,
    alias_to_collection: &HashMap<String, String>,
) -> HashSet<String> {
    let mut needed: HashSet<String> = HashSet::new();

    fn scan(
        value: &Value,
        alias: Option<&str>,
        alias_map: &HashMap<String, String>,
        needed: &mut HashSet<String>,
    ) {
        match value {
            Value::Object(obj) => {
                // Check for _value wrapper
                if let Some(inner) = obj.get("_value") {
                    scan(inner, alias, alias_map, needed);
                    return;
                }
                // Process each field
                for (key, v) in obj {
                    scan(v, Some(key.as_str()), alias_map, needed);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    scan(item, alias, alias_map, needed);
                }
            }
            Value::String(_) => {
                if let Some(a) = alias {
                    if let Some(collection_id) = alias_map.get(a) {
                        needed.insert(collection_id.clone());
                    }
                }
            }
            _ => {}
        }
    }

    scan(tree, None, alias_to_collection, &mut needed);
    needed
}

/// Resolve labels to UUIDs in a JSON tree.
///
/// This is the core resolution function. It takes:
/// - The tree to resolve
/// - The alias -> collection mapping
/// - A lookup implementation for finding concepts
/// - Configuration options
///
/// Returns the resolved tree and any errors encountered.
pub fn resolve_labels<L: ConceptLookup>(
    tree: Value,
    alias_to_collection: &HashMap<String, String>,
    lookup: &L,
    strict: bool,
) -> Result<Value, LabelResolutionError> {
    let mut errors: Vec<String> = Vec::new();

    fn resolve(
        value: Value,
        alias: Option<&str>,
        alias_map: &HashMap<String, String>,
        lookup: &impl ConceptLookup,
        errors: &mut Vec<String>,
        strict: bool,
    ) -> Value {
        match value {
            Value::Object(mut obj) => {
                // Check for _value wrapper
                if obj.contains_key("_value") {
                    if let Some(inner) = obj.remove("_value") {
                        let resolved = resolve(inner, alias, alias_map, lookup, errors, strict);
                        obj.insert("_value".to_string(), resolved);
                    }
                    return Value::Object(obj);
                }
                // Process each field
                let resolved_obj: serde_json::Map<String, Value> = obj
                    .into_iter()
                    .map(|(key, v)| {
                        let resolved =
                            resolve(v, Some(key.as_str()), alias_map, lookup, errors, strict);
                        (key, resolved)
                    })
                    .collect();
                Value::Object(resolved_obj)
            }
            Value::Array(arr) => {
                let resolved_arr: Vec<Value> = arr
                    .into_iter()
                    .map(|item| resolve(item, alias, alias_map, lookup, errors, strict))
                    .collect();
                Value::Array(resolved_arr)
            }
            Value::String(s) => {
                if let Some(a) = alias {
                    if let Some(collection_id) = alias_map.get(a) {
                        // Skip if already a UUID
                        if is_valid_uuid(&s) {
                            return Value::String(s);
                        }

                        // Try to resolve the label
                        if let Some(concept_id) = lookup.lookup_by_label(collection_id, &s) {
                            return Value::String(concept_id);
                        } else if strict {
                            errors.push(format!(
                                "Label '{}' not found in collection '{}' for field '{}'",
                                s, collection_id, a
                            ));
                        }
                    }
                }
                Value::String(s)
            }
            other => other,
        }
    }

    let resolved = resolve(tree, None, alias_to_collection, lookup, &mut errors, strict);

    if !errors.is_empty() {
        return Err(LabelResolutionError {
            message: format!("Failed to resolve labels:\n  {}", errors.join("\n  ")),
            errors,
        });
    }

    Ok(resolved)
}

/// High-level function to resolve labels in a JSON tree.
///
/// This combines all steps:
/// 1. Parse graph and build alias mapping
/// 2. Scan tree for needed collections (returned for lazy loading)
/// 3. Resolve labels using the provided lookup
///
/// Returns (resolved_tree, needed_collection_ids)
pub fn resolve_labels_full<L: ConceptLookup>(
    tree_json: &str,
    graph_json: &str,
    lookup: &L,
    config: &LabelResolutionConfig,
    registry: Option<&crate::extension_type_registry::ExtensionTypeRegistry>,
) -> Result<(String, HashSet<String>), LabelResolutionError> {
    // Parse inputs
    let tree: Value = serde_json::from_str(tree_json).map_err(|e| LabelResolutionError {
        message: format!("Failed to parse tree JSON: {}", e),
        errors: vec![],
    })?;

    let graph: Value = serde_json::from_str(graph_json).map_err(|e| LabelResolutionError {
        message: format!("Failed to parse graph JSON: {}", e),
        errors: vec![],
    })?;

    // Build alias -> collection mapping
    let alias_to_collection = build_alias_to_collection_map(&graph, config, registry);

    if alias_to_collection.is_empty() {
        // No resolvable nodes, return tree unchanged
        return Ok((tree_json.to_string(), HashSet::new()));
    }

    // Find which collections are needed
    let needed_collections = find_needed_collections(&tree, &alias_to_collection);

    // Resolve labels
    let resolved = resolve_labels(tree, &alias_to_collection, lookup, config.strict)?;

    // Serialize result
    let resolved_json = serde_json::to_string(&resolved).map_err(|e| LabelResolutionError {
        message: format!("Failed to serialize resolved tree: {}", e),
        errors: vec![],
    })?;

    Ok((resolved_json, needed_collections))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLookup {
        collections: HashMap<String, HashMap<String, String>>,
    }

    impl ConceptLookup for MockLookup {
        fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<String> {
            self.collections
                .get(collection_id)?
                .get(&label.to_lowercase())
                .cloned()
        }
    }

    #[test]
    fn test_is_valid_uuid() {
        assert!(is_valid_uuid("f8dbf847-aa2b-5a56-bf9e-b4648e8bda8b"));
        assert!(is_valid_uuid("F8DBF847-AA2B-5A56-BF9E-B4648E8BDA8B"));
        assert!(!is_valid_uuid("not-a-uuid"));
        assert!(!is_valid_uuid("Category A"));
    }

    #[test]
    fn test_build_alias_to_collection_map() {
        let graph = serde_json::json!({
            "nodes": [
                {
                    "alias": "category",
                    "datatype": "concept",
                    "config": {"rdmCollection": "collection-1"}
                },
                {
                    "alias": "tags",
                    "datatype": "concept-list",
                    "config": {"rdmCollection": "collection-2"}
                },
                {
                    "alias": "status",
                    "datatype": "reference",
                    "config": {"controlledList": "collection-3"}
                },
                {
                    "alias": "name",
                    "datatype": "string",
                    "config": {}
                }
            ]
        });

        // Default config only includes core types (concept, concept-list)
        let config = LabelResolutionConfig::default();
        let map = build_alias_to_collection_map(&graph, &config, None);

        assert_eq!(map.get("category"), Some(&"collection-1".to_string()));
        assert_eq!(map.get("tags"), Some(&"collection-2".to_string()));
        // "reference" is an extension type — not in default resolvable set
        assert_eq!(map.get("status"), None);
        assert_eq!(map.get("name"), None);
    }

    #[test]
    fn test_build_alias_to_collection_map_with_additional_datatypes() {
        let graph = serde_json::json!({
            "nodes": [
                {
                    "alias": "category",
                    "datatype": "concept",
                    "config": {"rdmCollection": "collection-1"}
                },
                {
                    "alias": "status",
                    "datatype": "reference",
                    "config": {"controlledList": "collection-2"}
                }
            ]
        });

        // Extensions add their datatypes via with_additional_datatypes; core
        // knows nothing of the extension's config key, so without a handler
        // there is no collection.
        let config = LabelResolutionConfig::default().with_additional_datatypes(&["reference"]);
        let map = build_alias_to_collection_map(&graph, &config, None);
        assert_eq!(map.get("category"), Some(&"collection-1".to_string()));
        assert_eq!(map.get("status"), None);

        // The handler names the collection.
        let registry = list_handler_registry();
        let map = build_alias_to_collection_map(&graph, &config, Some(&registry));
        assert_eq!(map.get("category"), Some(&"collection-1".to_string()));
        assert_eq!(map.get("status"), Some(&"collection-2".to_string()));
    }

    fn list_handler_registry() -> crate::extension_type_registry::ExtensionTypeRegistry {
        use crate::extension_type_registry::{
            ExtensionTypeHandler, ExtensionTypeRegistry, HandlerCapabilities,
        };
        struct ListHandler;
        impl ExtensionTypeHandler for ListHandler {
            fn capabilities(&self) -> HandlerCapabilities {
                HandlerCapabilities::default()
            }
            fn node_config(&self, config: &Value) -> Option<Value> {
                Some(serde_json::json!({ "collectionId": config.get("controlledList")? }))
            }
        }
        let mut registry = ExtensionTypeRegistry::new();
        registry.register("reference", std::sync::Arc::new(ListHandler));
        registry
    }

    #[test]
    fn alias_to_collection_for_graph_asks_handlers() {
        let root = serde_json::json!(
            {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"}
        );
        let graph = crate::graph::StaticGraph::from_json_string(
            &serde_json::json!({
                "graphid": "g",
                "name": {"en": "G"},
                "nodes": [
                    root,
                    {"alias": "category", "datatype": "concept", "nodeid": "n1",
                     "name": "category", "graph_id": "g",
                     "config": {"rdmCollection": "collection-1"}},
                    {"alias": "status", "datatype": "reference", "nodeid": "n2",
                     "name": "status", "graph_id": "g",
                     "config": {"controlledList": "collection-2"}}
                ],
                "root": root
            })
            .to_string(),
        )
        .unwrap();

        let map = alias_to_collection_for_graph(&graph, None);
        assert_eq!(map.get("category"), Some(&"collection-1".to_string()));
        assert_eq!(map.get("status"), None);

        let registry = list_handler_registry();
        let map = alias_to_collection_for_graph(&graph, Some(&registry));
        assert_eq!(map.get("status"), Some(&"collection-2".to_string()));
    }

    #[test]
    fn test_find_needed_collections() {
        let tree = serde_json::json!({
            "category": ["Cat A", "Cat B"],
            "name": ["John"],
            "status": ["Active"]
        });

        let mut alias_map = HashMap::new();
        alias_map.insert("category".to_string(), "coll-1".to_string());
        alias_map.insert("status".to_string(), "coll-2".to_string());

        let needed = find_needed_collections(&tree, &alias_map);

        assert!(needed.contains("coll-1"));
        assert!(needed.contains("coll-2"));
        assert_eq!(needed.len(), 2);
    }

    #[test]
    fn test_resolve_labels() {
        let tree = serde_json::json!({
            "category": ["Category A", "Category B"],
            "name": ["John"]
        });

        let mut alias_map = HashMap::new();
        alias_map.insert("category".to_string(), "test-collection".to_string());

        let mut concepts = HashMap::new();
        concepts.insert("category a".to_string(), "uuid-a".to_string());
        concepts.insert("category b".to_string(), "uuid-b".to_string());

        let mut collections = HashMap::new();
        collections.insert("test-collection".to_string(), concepts);

        let lookup = MockLookup { collections };

        let resolved = resolve_labels(tree, &alias_map, &lookup, false).unwrap();

        assert_eq!(resolved["category"][0], "uuid-a");
        assert_eq!(resolved["category"][1], "uuid-b");
        assert_eq!(resolved["name"][0], "John");
    }

    #[test]
    fn test_resolve_labels_uuid_passthrough() {
        let tree = serde_json::json!({
            "category": ["f8dbf847-aa2b-5a56-bf9e-b4648e8bda8b"]
        });

        let mut alias_map = HashMap::new();
        alias_map.insert("category".to_string(), "test-collection".to_string());

        let lookup = MockLookup {
            collections: HashMap::new(),
        };

        let resolved = resolve_labels(tree, &alias_map, &lookup, false).unwrap();

        assert_eq!(
            resolved["category"][0],
            "f8dbf847-aa2b-5a56-bf9e-b4648e8bda8b"
        );
    }

    #[test]
    fn test_resolve_labels_strict_mode() {
        let tree = serde_json::json!({
            "category": ["Unknown Label"]
        });

        let mut alias_map = HashMap::new();
        alias_map.insert("category".to_string(), "test-collection".to_string());

        let lookup = MockLookup {
            collections: HashMap::new(),
        };

        let result = resolve_labels(tree, &alias_map, &lookup, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unknown Label"));
    }

    #[test]
    fn test_resolve_labels_value_wrapper() {
        let tree = serde_json::json!({
            "category": [{"_value": "Category A"}]
        });

        let mut alias_map = HashMap::new();
        alias_map.insert("category".to_string(), "test-collection".to_string());

        let mut concepts = HashMap::new();
        concepts.insert("category a".to_string(), "uuid-a".to_string());

        let mut collections = HashMap::new();
        collections.insert("test-collection".to_string(), concepts);

        let lookup = MockLookup { collections };

        let resolved = resolve_labels(tree, &alias_map, &lookup, false).unwrap();

        assert_eq!(resolved["category"][0]["_value"], "uuid-a");
    }
}
