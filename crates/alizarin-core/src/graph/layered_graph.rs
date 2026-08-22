//! Layered graph composition.
//!
//! A [`LayeredGraph`] presents multiple [`StaticGraph`] layers as a single
//! [`GraphLookup`]. The base layer is the original resource model; each
//! overlay can add nodegroups, nodes, edges, and cards.
//!
//! Lookups walk layers **top-down** (latest overlay wins). Merged indices
//! are built lazily on first access, matching `StaticGraph`'s own pattern.
//!
//! When only a base graph exists, callers use `StaticGraph` directly —
//! `LayeredGraph` is only constructed when overlays are present, so the
//! single-layer path has zero overhead.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use super::card_index::CardIndex;
use super::cards::{StaticCard, StaticCardsXNodesXWidgets};
use super::graph_lookup::GraphLookup;
use super::nodes::{StaticEdge, StaticNode, StaticNodegroup};
use super::static_graph::StaticGraph;
use super::translatable::StaticTranslatableString;

/// Lookup indices built lazily — cheap HashMap entries, no struct cloning.
#[derive(Debug, Clone)]
struct LookupIndices {
    node_by_id: HashMap<String, LayerRef>,
    node_by_alias: HashMap<String, LayerRef>,
    nodes_by_alias_arc: HashMap<String, Arc<StaticNode>>,
    nodegroup_by_id: HashMap<String, LayerRef>,
    edges_map: HashMap<String, Vec<String>>,
    nodes_by_nodegroup: HashMap<String, Vec<LayerRef>>,
}

/// Flat merged vecs built lazily on first slice access — clones every
/// node/nodegroup/edge/card from all layers. Deferred so callers that
/// only use indexed lookups never pay this cost.
#[derive(Debug, Clone)]
struct MergedSlices {
    all_nodes: Vec<StaticNode>,
    all_nodegroups: Vec<StaticNodegroup>,
    all_edges: Vec<StaticEdge>,
    all_cards: Vec<StaticCard>,
    all_cxnxw: Vec<StaticCardsXNodesXWidgets>,
    nodes_by_nodegroup_flat: HashMap<String, Vec<usize>>,
}

/// Points to a specific item in a specific layer.
#[derive(Debug, Clone)]
struct LayerRef {
    layer_idx: usize,
    item_idx: usize,
}

/// A stack of graph layers presented as a single read-only graph.
///
/// Layer 0 is the base graph. Higher indices are overlays, topmost wins.
/// The `LayeredGraph` borrows the underlying `StaticGraph`s via `Arc`.
#[derive(Clone)]
pub struct LayeredGraph {
    /// Layer 0 = base, layer N = topmost overlay
    layers: Vec<Arc<StaticGraph>>,
    indices: OnceLock<LookupIndices>,
    slices: OnceLock<MergedSlices>,
}

impl LayeredGraph {
    /// Create a layered graph from a base and one or more overlays.
    ///
    /// Panics if `layers` is empty — use `StaticGraph` directly for
    /// the single-layer case.
    pub fn new(base: Arc<StaticGraph>, overlays: Vec<Arc<StaticGraph>>) -> Self {
        assert!(
            !overlays.is_empty(),
            "Use StaticGraph directly when there are no overlays"
        );
        let mut layers = Vec::with_capacity(1 + overlays.len());
        layers.push(base);
        layers.extend(overlays);
        Self {
            layers,
            indices: OnceLock::new(),
            slices: OnceLock::new(),
        }
    }

    /// The base graph (layer 0).
    pub fn base(&self) -> &StaticGraph {
        &self.layers[0]
    }

    /// Number of layers (including the base).
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Return a new `LayeredGraph` with an additional overlay on top.
    ///
    /// The existing instance (and any already-loaded resources referencing
    /// it) remains valid with its original layer set. Callers should
    /// replace their reference to pick up the new layer.
    pub fn with_overlay(&self, overlay: Arc<StaticGraph>) -> Self {
        let mut layers = self.layers.clone();
        layers.push(overlay);
        Self {
            layers,
            indices: OnceLock::new(),
            slices: OnceLock::new(),
        }
    }

    /// Return a new `LayeredGraph` without the overlay whose `graphid`
    /// matches `overlay_id`. Returns `None` if removal would leave only
    /// the base (use `StaticGraph` directly in that case).
    pub fn without_overlay(&self, overlay_id: &str) -> Option<Self> {
        let layers: Vec<_> = self
            .layers
            .iter()
            .filter(|g| g.graphid != overlay_id || std::ptr::eq(g.as_ref(), self.base()))
            .cloned()
            .collect();
        if layers.len() <= 1 {
            return None;
        }
        Some(Self {
            layers,
            indices: OnceLock::new(),
            slices: OnceLock::new(),
        })
    }

    fn indices(&self) -> &LookupIndices {
        self.indices.get_or_init(|| self.build_lookup_indices())
    }

    fn merged_slices(&self) -> &MergedSlices {
        self.slices.get_or_init(|| self.build_merged_slices())
    }

    fn build_lookup_indices(&self) -> LookupIndices {
        let mut node_by_id: HashMap<String, LayerRef> = HashMap::new();
        let mut node_by_alias: HashMap<String, LayerRef> = HashMap::new();
        let mut nodes_by_alias_arc: HashMap<String, Arc<StaticNode>> = HashMap::new();
        let mut nodegroup_by_id: HashMap<String, LayerRef> = HashMap::new();
        let mut edges_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut nodes_by_nodegroup: HashMap<String, Vec<LayerRef>> = HashMap::new();

        // Iterate top-down so topmost wins (or_insert keeps first = topmost).
        // A node/edge already contributed by a higher layer is SKIPPED, so an
        // overlay that re-describes the same model does not duplicate entries
        // (which would double every cardinality-n nodegroup at tree-build time).
        let mut seen_node_ids = std::collections::HashSet::new();
        let mut seen_edges = std::collections::HashSet::new();
        for (layer_idx, graph) in self.layers.iter().enumerate().rev() {
            for (item_idx, node) in graph.nodes.iter().enumerate() {
                let lr = LayerRef {
                    layer_idx,
                    item_idx,
                };

                // First (topmost) occurrence of this node id only — otherwise a
                // re-describing overlay double-fills nodes_by_nodegroup.
                if !seen_node_ids.insert(node.nodeid.clone()) {
                    continue;
                }
                node_by_id.entry(node.nodeid.clone()).or_insert(lr.clone());

                if let Some(ref alias) = node.alias {
                    if !alias.is_empty() {
                        node_by_alias.entry(alias.clone()).or_insert(lr.clone());
                        nodes_by_alias_arc
                            .entry(alias.clone())
                            .or_insert_with(|| Arc::new(node.clone()));
                    }
                }

                if let Some(ref ng_id) = node.nodegroup_id {
                    if !ng_id.is_empty() {
                        nodes_by_nodegroup
                            .entry(ng_id.clone())
                            .or_default()
                            .push(lr.clone());
                    }
                }
            }

            for (item_idx, ng) in graph.nodegroups.iter().enumerate() {
                nodegroup_by_id
                    .entry(ng.nodegroupid.clone())
                    .or_insert(LayerRef {
                        layer_idx,
                        item_idx,
                    });
            }

            for edge in &graph.edges {
                if seen_edges.insert((edge.domainnode_id.clone(), edge.rangenode_id.clone())) {
                    edges_map
                        .entry(edge.domainnode_id.clone())
                        .or_default()
                        .push(edge.rangenode_id.clone());
                }
            }
        }

        LookupIndices {
            node_by_id,
            node_by_alias,
            nodes_by_alias_arc,
            nodegroup_by_id,
            edges_map,
            nodes_by_nodegroup,
        }
    }

    fn build_merged_slices(&self) -> MergedSlices {
        let mut all_nodes: Vec<StaticNode> = Vec::new();
        let mut all_nodegroups: Vec<StaticNodegroup> = Vec::new();
        let mut all_edges: Vec<StaticEdge> = Vec::new();
        let mut all_cards: Vec<StaticCard> = Vec::new();
        let mut all_cxnxw: Vec<StaticCardsXNodesXWidgets> = Vec::new();

        let mut seen_node_ids = std::collections::HashSet::new();
        let mut seen_ng_ids = std::collections::HashSet::new();
        let mut seen_card_ngs = std::collections::HashSet::new();
        // Dedup edges by their (domain -> range) endpoints, like nodes/nodegroups.
        // Overlays that re-describe the SAME model (e.g. a computed layer carrying
        // the whole graph) would otherwise duplicate every edge, and the
        // tile->tree builder traverses via edges - so a cardinality-n nodegroup
        // reachable through a duplicated edge gets emitted TWICE.
        let mut seen_edges = std::collections::HashSet::new();

        for graph in self.layers.iter().rev() {
            for node in &graph.nodes {
                if seen_node_ids.insert(node.nodeid.clone()) {
                    all_nodes.push(node.clone());
                }
            }
            for ng in &graph.nodegroups {
                if seen_ng_ids.insert(ng.nodegroupid.clone()) {
                    all_nodegroups.push(ng.clone());
                }
            }
            for edge in &graph.edges {
                if seen_edges.insert((edge.domainnode_id.clone(), edge.rangenode_id.clone())) {
                    all_edges.push(edge.clone());
                }
            }
            if let Some(ref cards) = graph.cards {
                for card in cards {
                    if seen_card_ngs.insert(card.nodegroup_id.clone()) {
                        all_cards.push(card.clone());
                    }
                }
            }
            if let Some(ref cxnxw) = graph.cards_x_nodes_x_widgets {
                all_cxnxw.extend(cxnxw.iter().cloned());
            }
        }

        let mut nodes_by_nodegroup_flat: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, node) in all_nodes.iter().enumerate() {
            if let Some(ref ng_id) = node.nodegroup_id {
                if !ng_id.is_empty() {
                    nodes_by_nodegroup_flat
                        .entry(ng_id.clone())
                        .or_default()
                        .push(idx);
                }
            }
        }

        MergedSlices {
            all_nodes,
            all_nodegroups,
            all_edges,
            all_cards,
            all_cxnxw,
            nodes_by_nodegroup_flat,
        }
    }

    /// Resolve a LayerRef to the node it points to.
    fn resolve_node(&self, lr: &LayerRef) -> Option<&StaticNode> {
        self.layers.get(lr.layer_idx)?.nodes.get(lr.item_idx)
    }

    /// Resolve a LayerRef to the nodegroup it points to.
    fn resolve_nodegroup(&self, lr: &LayerRef) -> Option<&StaticNodegroup> {
        self.layers.get(lr.layer_idx)?.nodegroups.get(lr.item_idx)
    }
}

impl GraphLookup for LayeredGraph {
    fn graph_id(&self) -> &str {
        self.base().graph_id()
    }

    fn display_name(&self) -> String {
        self.base().display_name()
    }

    fn get_model_class_name(&self) -> Option<String> {
        self.base().get_model_class_name()
    }

    fn name(&self) -> &StaticTranslatableString {
        &self.base().name
    }

    fn get_root(&self) -> &StaticNode {
        self.base().get_root()
    }

    fn get_node_by_id(&self, id: &str) -> Option<&StaticNode> {
        self.indices()
            .node_by_id
            .get(id)
            .and_then(|lr| self.resolve_node(lr))
    }

    fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        self.indices()
            .node_by_alias
            .get(alias)
            .and_then(|lr| self.resolve_node(lr))
    }

    fn find_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        // Indexed lookup covers all layers
        self.get_node_by_alias(alias)
    }

    fn get_node_arc_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>> {
        self.indices().nodes_by_alias_arc.get(alias).cloned()
    }

    fn nodes_by_alias_arc(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        Some(&self.indices().nodes_by_alias_arc)
    }

    fn get_nodegroup_by_id(&self, nodegroup_id: &str) -> Option<&StaticNodegroup> {
        self.indices()
            .nodegroup_by_id
            .get(nodegroup_id)
            .and_then(|lr| self.resolve_nodegroup(lr))
    }

    fn get_nodes_in_nodegroup(&self, nodegroup_id: &str) -> Vec<&StaticNode> {
        self.indices()
            .nodes_by_nodegroup
            .get(nodegroup_id)
            .map(|lrs| lrs.iter().filter_map(|lr| self.resolve_node(lr)).collect())
            .unwrap_or_default()
    }

    fn edges_map(&self) -> Option<&HashMap<String, Vec<String>>> {
        Some(&self.indices().edges_map)
    }

    fn get_child_ids(&self, node_id: &str) -> Option<&Vec<String>> {
        self.indices().edges_map.get(node_id)
    }

    fn find_card_by_nodegroup(&self, nodegroup_id: &str) -> Option<&StaticCard> {
        // Topmost layer wins — scan top-down without triggering merged slices.
        for graph in self.layers.iter().rev() {
            if let Some(ref cards) = graph.cards {
                if let Some(card) = cards.iter().find(|c| c.nodegroup_id == nodegroup_id) {
                    return Some(card);
                }
            }
        }
        None
    }

    fn cards_slice(&self) -> &[StaticCard] {
        &self.merged_slices().all_cards
    }

    fn card_index(&self) -> Option<&CardIndex> {
        self.base().card_index()
    }

    fn cards_x_nodes_x_widgets_slice(&self) -> &[StaticCardsXNodesXWidgets] {
        &self.merged_slices().all_cxnxw
    }

    fn nodes_slice(&self) -> &[StaticNode] {
        &self.merged_slices().all_nodes
    }

    fn nodegroups_slice(&self) -> &[StaticNodegroup] {
        &self.merged_slices().all_nodegroups
    }

    fn edges_slice(&self) -> &[StaticEdge] {
        &self.merged_slices().all_edges
    }

    fn nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<usize>>> {
        Some(&self.merged_slices().nodes_by_nodegroup_flat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base_graph() -> StaticGraph {
        let mut g: StaticGraph = serde_json::from_value(json!({
            "graphid": "base",
            "name": {"en": "Base Model"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"},
            "nodes": [
                {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"},
                {"nodeid": "n1", "name": "Name", "datatype": "string", "graph_id": "base",
                 "alias": "name", "nodegroup_id": "ng1"}
            ],
            "nodegroups": [
                {"nodegroupid": "ng1", "cardinality": "1"}
            ],
            "edges": [
                {"edgeid": "e1", "domainnode_id": "root", "rangenode_id": "n1", "graph_id": "base"}
            ]
        }))
        .unwrap();
        g.build_indices();
        g
    }

    fn overlay_graph() -> StaticGraph {
        let mut g: StaticGraph = serde_json::from_value(json!({
            "graphid": "overlay1",
            "name": {"en": "Overlay"},
            "root": {"nodeid": "oroot", "name": "ORoot", "datatype": "semantic", "graph_id": "overlay1"},
            "nodes": [
                {"nodeid": "oroot", "name": "ORoot", "datatype": "semantic", "graph_id": "overlay1"},
                {"nodeid": "n2", "name": "Description", "datatype": "string", "graph_id": "overlay1",
                 "alias": "description", "nodegroup_id": "ng2"}
            ],
            "nodegroups": [
                {"nodegroupid": "ng2", "cardinality": "1"}
            ],
            "edges": [
                {"edgeid": "e2", "domainnode_id": "root", "rangenode_id": "n2", "graph_id": "overlay1"}
            ]
        }))
        .unwrap();
        g.build_indices();
        g
    }

    #[test]
    fn layered_graph_merges_nodes() {
        let lg = LayeredGraph::new(Arc::new(base_graph()), vec![Arc::new(overlay_graph())]);
        assert!(lg.get_node_by_id("n1").is_some(), "base node visible");
        assert!(lg.get_node_by_id("n2").is_some(), "overlay node visible");
        assert!(lg.find_node_by_alias("name").is_some());
        assert!(lg.find_node_by_alias("description").is_some());
    }

    #[test]
    fn layered_graph_merges_nodegroups() {
        let lg = LayeredGraph::new(Arc::new(base_graph()), vec![Arc::new(overlay_graph())]);
        assert!(lg.get_nodegroup_by_id("ng1").is_some(), "base ng");
        assert!(lg.get_nodegroup_by_id("ng2").is_some(), "overlay ng");
    }

    #[test]
    fn layered_graph_merges_edges() {
        let lg = LayeredGraph::new(Arc::new(base_graph()), vec![Arc::new(overlay_graph())]);
        let children = lg.get_child_ids("root");
        assert!(children.is_some());
        let ids = children.unwrap();
        assert!(ids.contains(&"n1".to_string()), "base edge");
        assert!(ids.contains(&"n2".to_string()), "overlay edge");
    }

    #[test]
    fn overlay_redescribing_the_base_does_not_duplicate_edges_or_children() {
        // A computed-layer overlay may carry the WHOLE base model (same edges +
        // nodes). Its shared edges/children must not duplicate, or the tile->tree
        // builder walks a cardinality-n nodegroup twice and doubles every row.
        let lg = LayeredGraph::new(Arc::new(base_graph()), vec![Arc::new(base_graph())]);
        let children = lg.get_child_ids("root").expect("root has children");
        assert_eq!(
            children.iter().filter(|c| c.as_str() == "n1").count(),
            1,
            "shared edge deduped in lookup; got {children:?}"
        );
        assert_eq!(
            lg.edges_slice()
                .iter()
                .filter(|e| e.domainnode_id == "root" && e.rangenode_id == "n1")
                .count(),
            1,
            "shared edge deduped in slices"
        );
        assert_eq!(
            lg.get_nodes_in_nodegroup("ng1").len(),
            1,
            "shared nodegroup node not double-counted"
        );
    }

    #[test]
    fn edge_dedup_keys_on_domain_and_range_not_domain_alone() {
        // Two edges share a domain (root) but have distinct ranges (n1, n2).
        // Stacking the graph over itself must keep BOTH (dedup by the full
        // (domain,range) tuple), not collapse root's children to one.
        let mut g: StaticGraph = serde_json::from_value(json!({
            "graphid": "base", "name": {"en": "Base"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"},
            "nodes": [
                {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"},
                {"nodeid": "n1", "name": "A", "datatype": "string", "graph_id": "base"},
                {"nodeid": "n2", "name": "B", "datatype": "string", "graph_id": "base"}
            ],
            "nodegroups": [],
            "edges": [
                {"edgeid": "e1", "domainnode_id": "root", "rangenode_id": "n1", "graph_id": "base"},
                {"edgeid": "e2", "domainnode_id": "root", "rangenode_id": "n2", "graph_id": "base"}
            ]
        }))
        .unwrap();
        g.build_indices();
        let lg = LayeredGraph::new(Arc::new(g.clone()), vec![Arc::new(g)]);
        let mut children = lg.get_child_ids("root").expect("root has children").clone();
        children.sort();
        assert_eq!(
            children,
            vec!["n1".to_string(), "n2".to_string()],
            "both distinct-range edges kept"
        );
    }

    #[test]
    fn identity_is_from_base() {
        let lg = LayeredGraph::new(Arc::new(base_graph()), vec![Arc::new(overlay_graph())]);
        assert_eq!(lg.graph_id(), "base");
        assert_eq!(lg.display_name(), "Base Model");
        assert_eq!(lg.get_root().nodeid, "root");
    }

    #[test]
    fn topmost_overlay_wins_on_conflict() {
        let mut override_graph = overlay_graph();
        // Add a node with same ID as base but different name
        let override_node: StaticNode = serde_json::from_value(json!({
            "nodeid": "n1", "name": "Overridden Name", "datatype": "string",
            "graph_id": "overlay1", "alias": "name", "nodegroup_id": "ng1"
        }))
        .unwrap();
        override_graph.nodes.push(override_node);
        override_graph.build_indices();

        let lg = LayeredGraph::new(Arc::new(base_graph()), vec![Arc::new(override_graph)]);
        let node = lg.get_node_by_id("n1").unwrap();
        assert_eq!(node.name, "Overridden Name", "topmost layer wins");
    }

    #[test]
    fn nodes_slice_contains_all() {
        let lg = LayeredGraph::new(Arc::new(base_graph()), vec![Arc::new(overlay_graph())]);
        let nodes = lg.nodes_slice();
        let ids: Vec<&str> = nodes.iter().map(|n| n.nodeid.as_str()).collect();
        assert!(ids.contains(&"n1"));
        assert!(ids.contains(&"n2"));
        assert!(ids.contains(&"root"));
    }
}
