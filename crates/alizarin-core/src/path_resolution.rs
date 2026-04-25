/// Path resolution for navigating dot-separated paths through a graph model.
///
/// Walks the graph's edge structure (parent→child) matching node aliases at each
/// segment, then resolves the target node's nodegroup for tile lookup. This avoids
/// full tree materialization — instead it goes straight from path to nodegroup to tiles.
use std::collections::HashMap;
use std::sync::Arc;

use crate::{is_node_single_cardinality, StaticNode, StaticNodegroup};

// =============================================================================
// Error type
// =============================================================================

/// Errors that can occur during path resolution
#[derive(Debug, Clone)]
pub enum PathError {
    /// The path string was empty or contained only separators
    EmptyPath,
    /// No child of the current node matched the given alias segment
    AliasNotFound {
        segment: String,
        parent_alias: Option<String>,
    },
    /// `_` was used on a node that is not a collector (no inner/outer split)
    UnderscoreOnNonCollector { node_alias: String },
    /// `*` was used on a single-cardinality node
    StarOnSingleCardinality { node_alias: String },
    /// The target node has no nodegroup_id
    NoNodegroup { node_alias: String },
    /// Model data (nodes, edges, etc.) was not available
    ModelNotInitialized(String),
    /// Tiles storage was not initialized on the wrapper
    TilesNotInitialized,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::EmptyPath => write!(f, "Path is empty"),
            PathError::AliasNotFound {
                segment,
                parent_alias,
            } => {
                if let Some(parent) = parent_alias {
                    write!(
                        f,
                        "No child with alias '{}' found under '{}'",
                        segment, parent
                    )
                } else {
                    write!(f, "No child with alias '{}' found under root node", segment)
                }
            }
            PathError::UnderscoreOnNonCollector { node_alias } => {
                write!(
                    f,
                    "'_' used on node '{}' which is not a collector (no inner/outer split)",
                    node_alias
                )
            }
            PathError::StarOnSingleCardinality { node_alias } => {
                write!(
                    f,
                    "'*' used on node '{}' which is single-cardinality",
                    node_alias
                )
            }
            PathError::NoNodegroup { node_alias } => {
                write!(f, "Node '{}' has no nodegroup_id", node_alias)
            }
            PathError::ModelNotInitialized(msg) => {
                write!(f, "Model not initialized: {}", msg)
            }
            PathError::TilesNotInitialized => write!(f, "Tiles not initialized"),
        }
    }
}

impl std::error::Error for PathError {}

impl From<String> for PathError {
    fn from(s: String) -> Self {
        PathError::ModelNotInitialized(s)
    }
}

// =============================================================================
// Resolution result
// =============================================================================

/// The result of resolving a dot-separated path through the graph model.
///
/// Contains everything needed to build a PseudoList from the tile store:
/// the target node, its nodegroup, the child node IDs (for PseudoValueCore),
/// and cardinality.
#[derive(Debug, Clone)]
pub struct PathResolutionInfo {
    /// The node at the end of the path
    pub target_node: Arc<StaticNode>,
    /// The nodegroup the target node belongs to
    pub nodegroup_id: String,
    /// The nodegroup of the parent (penultimate) node in the path, if any
    pub parent_nodegroup_id: Option<String>,
    /// Child node IDs of the target (from edges)
    pub child_node_ids: Vec<String>,
    /// Whether this node is single-cardinality
    pub is_single: bool,
}

// =============================================================================
// Path resolution algorithm
// =============================================================================

/// Walk the graph model following a dot-separated path of node aliases.
///
/// Starting from the root node, each segment of the path is matched against
/// the aliases of children (via edges). Returns information about the target
/// node sufficient for tile lookup and PseudoList construction.
///
/// # Arguments
/// * `path` — dot-separated path, e.g. `"building.name"` or `".building.name"`
/// * `root_node` — the graph's root (top) node
/// * `nodes` — all nodes indexed by node ID
/// * `edges` — parent_node_id → child_node_ids
/// * `nodegroups` — nodegroup lookup (for cardinality checks)
///
/// # Errors
/// Returns `PathError` if the path is empty, a segment doesn't match any child alias,
/// or the target node has no nodegroup.
pub fn resolve_path_segments(
    path: &str,
    root_node: &Arc<StaticNode>,
    nodes: &HashMap<String, Arc<StaticNode>>,
    edges: &HashMap<String, Vec<String>>,
    nodegroups: Option<&HashMap<String, Arc<StaticNodegroup>>>,
) -> Result<PathResolutionInfo, PathError> {
    // Split on '.', filter out empty segments (handles leading '.')
    let segments: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();

    if segments.is_empty() {
        return Err(PathError::EmptyPath);
    }

    let mut current_node = Arc::clone(root_node);
    let mut parent_nodegroup_id: Option<String> = None;

    for segment in &segments {
        // "_" navigates into the inner part of a collector's inner/outer split.
        // At graph level this is a no-op, but validate the node is actually a collector.
        if *segment == "_" {
            if !current_node.is_collector {
                return Err(PathError::UnderscoreOnNonCollector {
                    node_alias: current_node.alias.clone().unwrap_or_default(),
                });
            }
            continue;
        }
        // "*" asserts cardinality-N. Validate the node is not single-cardinality.
        if *segment == "*" {
            if is_node_single_cardinality(&current_node, nodegroups) {
                return Err(PathError::StarOnSingleCardinality {
                    node_alias: current_node.alias.clone().unwrap_or_default(),
                });
            }
            continue;
        }

        // Get children of the current node
        let child_ids = edges.get(&current_node.nodeid).cloned().unwrap_or_default();

        // Find the child whose alias matches this segment
        let matched = child_ids.iter().find_map(|child_id| {
            nodes.get(child_id).and_then(|child_node| {
                if child_node.alias.as_deref() == Some(segment) {
                    Some(Arc::clone(child_node))
                } else {
                    None
                }
            })
        });

        parent_nodegroup_id = current_node.nodegroup_id.clone();
        current_node = matched.ok_or_else(|| PathError::AliasNotFound {
            segment: segment.to_string(),
            parent_alias: current_node.alias.clone(),
        })?;
    }

    // Resolve nodegroup
    let nodegroup_id = current_node
        .nodegroup_id
        .clone()
        .ok_or_else(|| PathError::NoNodegroup {
            node_alias: current_node.alias.clone().unwrap_or_default(),
        })?;

    // Get child node IDs for PseudoValue construction
    let child_node_ids = edges.get(&current_node.nodeid).cloned().unwrap_or_default();

    // Determine cardinality
    let is_single = is_node_single_cardinality(&current_node, nodegroups);

    Ok(PathResolutionInfo {
        target_node: current_node,
        nodegroup_id,
        parent_nodegroup_id,
        child_node_ids,
        is_single,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a StaticNode with the minimum required fields
    fn make_node(
        nodeid: &str,
        alias: &str,
        nodegroup_id: Option<&str>,
        is_collector: bool,
        istopnode: bool,
    ) -> Arc<StaticNode> {
        Arc::new(StaticNode {
            nodeid: nodeid.to_string(),
            name: alias.to_string(),
            alias: Some(alias.to_string()),
            datatype: "string".to_string(),
            is_collector,
            nodegroup_id: nodegroup_id.map(|s| s.to_string()),
            graph_id: "test-graph".to_string(),
            isrequired: false,
            exportable: true,
            sortorder: None,
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: None,
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: false,
            istopnode,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        })
    }

    /// Build a simple graph:
    ///   root
    ///   ├── building (ng: ng-building)
    ///   │   ├── name (ng: ng-building, same nodegroup)
    ///   │   └── address (ng: ng-address, different nodegroup)
    ///   │       └── city (ng: ng-address, same nodegroup as address)
    ///   └── status (ng: ng-status)
    fn setup_graph() -> (
        Arc<StaticNode>,
        HashMap<String, Arc<StaticNode>>,
        HashMap<String, Vec<String>>,
        HashMap<String, Arc<StaticNodegroup>>,
    ) {
        let root = make_node("root-id", "root", Some("root-id"), false, true);
        let building = make_node("building-id", "building", Some("ng-building"), false, false);
        let name = make_node("name-id", "name", Some("ng-building"), false, false);
        let address = make_node("address-id", "address", Some("ng-address"), true, false);
        let city = make_node("city-id", "city", Some("ng-address"), false, false);
        let status = make_node("status-id", "status", Some("ng-status"), false, false);

        let mut nodes = HashMap::new();
        for n in [&root, &building, &name, &address, &city, &status] {
            nodes.insert(n.nodeid.clone(), Arc::clone(n));
        }

        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        edges.insert(
            "root-id".into(),
            vec!["building-id".into(), "status-id".into()],
        );
        edges.insert(
            "building-id".into(),
            vec!["name-id".into(), "address-id".into()],
        );
        edges.insert("address-id".into(), vec!["city-id".into()]);

        let mut nodegroups = HashMap::new();
        let make_ng = |id: &str, cardinality: Option<&str>| {
            Arc::new(StaticNodegroup {
                nodegroupid: id.to_string(),
                cardinality: cardinality.map(|s| s.to_string()),
                legacygroupid: None,
                parentnodegroup_id: None,
                grouping_node_id: None,
            })
        };
        nodegroups.insert("ng-building".into(), make_ng("ng-building", Some("1")));
        nodegroups.insert("ng-address".into(), make_ng("ng-address", Some("n")));
        nodegroups.insert("ng-status".into(), make_ng("ng-status", Some("1")));

        (root, nodes, edges, nodegroups)
    }

    #[test]
    fn test_single_segment_path() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let result =
            resolve_path_segments("building", &root, &nodes, &edges, Some(&nodegroups)).unwrap();

        assert_eq!(result.target_node.alias.as_deref(), Some("building"));
        assert_eq!(result.nodegroup_id, "ng-building");
        // building has children: name, address
        assert_eq!(result.child_node_ids.len(), 2);
    }

    #[test]
    fn test_multi_segment_path() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let result =
            resolve_path_segments("building.name", &root, &nodes, &edges, Some(&nodegroups))
                .unwrap();

        assert_eq!(result.target_node.alias.as_deref(), Some("name"));
        assert_eq!(result.nodegroup_id, "ng-building");
        assert!(result.child_node_ids.is_empty());
        assert!(result.is_single); // non-collector, nodegroup cardinality "1"
    }

    #[test]
    fn test_cross_nodegroup_path() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let result = resolve_path_segments(
            "building.address.city",
            &root,
            &nodes,
            &edges,
            Some(&nodegroups),
        )
        .unwrap();

        assert_eq!(result.target_node.alias.as_deref(), Some("city"));
        assert_eq!(result.nodegroup_id, "ng-address");
    }

    #[test]
    fn test_leading_dot_stripped() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let result =
            resolve_path_segments(".building.name", &root, &nodes, &edges, Some(&nodegroups))
                .unwrap();

        assert_eq!(result.target_node.alias.as_deref(), Some("name"));
    }

    #[test]
    fn test_collector_is_not_single() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let result =
            resolve_path_segments("building.address", &root, &nodes, &edges, Some(&nodegroups))
                .unwrap();

        assert_eq!(result.target_node.alias.as_deref(), Some("address"));
        // address is a collector node → not single
        assert!(!result.is_single);
    }

    #[test]
    fn test_empty_path_error() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let err = resolve_path_segments("", &root, &nodes, &edges, Some(&nodegroups)).unwrap_err();
        assert!(matches!(err, PathError::EmptyPath));
    }

    #[test]
    fn test_only_dots_error() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let err =
            resolve_path_segments("...", &root, &nodes, &edges, Some(&nodegroups)).unwrap_err();
        assert!(matches!(err, PathError::EmptyPath));
    }

    #[test]
    fn test_alias_not_found() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let err = resolve_path_segments(
            "building.nonexistent",
            &root,
            &nodes,
            &edges,
            Some(&nodegroups),
        )
        .unwrap_err();
        match err {
            PathError::AliasNotFound {
                segment,
                parent_alias,
            } => {
                assert_eq!(segment, "nonexistent");
                assert_eq!(parent_alias.as_deref(), Some("building"));
            }
            _ => panic!("Expected AliasNotFound, got {:?}", err),
        }
    }

    #[test]
    fn test_first_segment_not_found() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        let err =
            resolve_path_segments("unknown", &root, &nodes, &edges, Some(&nodegroups)).unwrap_err();
        match err {
            PathError::AliasNotFound {
                segment,
                parent_alias,
            } => {
                assert_eq!(segment, "unknown");
                assert_eq!(parent_alias.as_deref(), Some("root"));
            }
            _ => panic!("Expected AliasNotFound, got {:?}", err),
        }
    }

    #[test]
    fn test_path_beyond_leaf_fails() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        // "name" is a leaf — no children
        let err = resolve_path_segments(
            "building.name.extra",
            &root,
            &nodes,
            &edges,
            Some(&nodegroups),
        )
        .unwrap_err();
        assert!(matches!(err, PathError::AliasNotFound { .. }));
    }

    #[test]
    fn test_no_nodegroup_error() {
        // Create a node without a nodegroup
        let root = make_node("root-id", "root", Some("root-id"), false, true);
        let child = Arc::new(StaticNode {
            nodeid: "child-id".to_string(),
            name: "child".to_string(),
            alias: Some("child".to_string()),
            datatype: "string".to_string(),
            is_collector: false,
            nodegroup_id: None, // No nodegroup!
            graph_id: "test-graph".to_string(),
            isrequired: false,
            exportable: true,
            sortorder: None,
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: None,
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: false,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        });

        let mut nodes = HashMap::new();
        nodes.insert("root-id".into(), Arc::clone(&root));
        nodes.insert("child-id".into(), Arc::clone(&child));

        let mut edges = HashMap::new();
        edges.insert("root-id".into(), vec!["child-id".into()]);

        let err = resolve_path_segments("child", &root, &nodes, &edges, None).unwrap_err();
        assert!(matches!(err, PathError::NoNodegroup { .. }));
    }

    // =========================================================================
    // Tests for _ and * segment handling
    // =========================================================================

    #[test]
    fn test_underscore_skipped_on_collector() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        // address is a collector — "building.address._.city" should resolve to city
        let result = resolve_path_segments(
            "building.address._.city",
            &root,
            &nodes,
            &edges,
            Some(&nodegroups),
        )
        .unwrap();
        assert_eq!(result.target_node.alias.as_deref(), Some("city"));
        assert_eq!(result.nodegroup_id, "ng-address");
    }

    #[test]
    fn test_underscore_errors_on_non_collector() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        // building is NOT a collector — "building._.name" should error
        let err =
            resolve_path_segments("building._.name", &root, &nodes, &edges, Some(&nodegroups))
                .unwrap_err();
        assert!(matches!(err, PathError::UnderscoreOnNonCollector { .. }));
    }

    #[test]
    fn test_star_skipped_on_multi_cardinality() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        // address is a collector (multi-cardinality) — "building.address.*.city" should work
        let result = resolve_path_segments(
            "building.address.*.city",
            &root,
            &nodes,
            &edges,
            Some(&nodegroups),
        )
        .unwrap();
        assert_eq!(result.target_node.alias.as_deref(), Some("city"));
    }

    #[test]
    fn test_star_errors_on_single_cardinality() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        // building has nodegroup cardinality "1" — "building.*.name" should error
        let err =
            resolve_path_segments("building.*.name", &root, &nodes, &edges, Some(&nodegroups))
                .unwrap_err();
        assert!(matches!(err, PathError::StarOnSingleCardinality { .. }));
    }

    #[test]
    fn test_star_and_underscore_combined() {
        let (root, nodes, edges, nodegroups) = setup_graph();
        // address is collector + multi — both * and _ should be valid
        let result = resolve_path_segments(
            "building.address.*._.city",
            &root,
            &nodes,
            &edges,
            Some(&nodegroups),
        )
        .unwrap();
        assert_eq!(result.target_node.alias.as_deref(), Some("city"));
    }
}
