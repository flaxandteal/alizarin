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
use super::cards::{StaticCard, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs};
use super::graph_lookup::GraphLookup;
use super::nodes::{StaticEdge, StaticNode, StaticNodegroup};
use super::static_graph::StaticGraph;
use super::translatable::StaticTranslatableString;

/// Lookup indices built lazily — cheap HashMap entries, no struct cloning.
#[derive(Debug, Clone)]
struct LookupIndices {
    // Node CONTENT lookups live on MergedSlices (so they carry the ontology-class
    // union); these are the structural LayerRef indices that don't need it.
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
    all_fxg: Vec<StaticFunctionsXGraphs>,
    nodes_by_nodegroup_flat: HashMap<String, Vec<usize>>,
    // Node CONTENT lookups resolve into `all_nodes` (not per-layer graphs) so that
    // callers see the cross-layer ONTOLOGY-CLASS UNION baked into the merged node,
    // not just the topmost layer's classes.
    node_pos_by_id: HashMap<String, usize>,
    node_pos_by_alias: HashMap<String, usize>,
    nodes_by_alias_arc: HashMap<String, Arc<StaticNode>>,
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
    /// Create a layered graph from a base and zero or more overlays.
    ///
    /// A single-layer graph (no overlays) is permitted: it merges to the base
    /// alone. This lets callers that always hand a `LayeredGraph` to the
    /// hydrate/query path (e.g. ros-madair-duck `hydrate_layers`) work uniformly
    /// for 1..N layers without a separate `StaticGraph` code path.
    pub fn new(base: Arc<StaticGraph>, overlays: Vec<Arc<StaticGraph>>) -> Self {
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
        let mut nodegroup_by_id: HashMap<String, LayerRef> = HashMap::new();
        let mut edges_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut nodes_by_nodegroup: HashMap<String, Vec<LayerRef>> = HashMap::new();

        // Iterate top-down so topmost wins (or_insert keeps first = topmost).
        // A node/edge already contributed by a higher layer is SKIPPED, so an
        // overlay that re-describes the same model does not duplicate entries
        // (which would double every cardinality-n nodegroup at tree-build time).
        // Node CONTENT (id/alias → node, carrying the ontology-class union) is
        // served from MergedSlices, not from here.
        let mut seen_node_ids = std::collections::HashSet::new();
        let mut seen_edges = std::collections::HashSet::new();
        for (layer_idx, graph) in self.layers.iter().enumerate().rev() {
            for (item_idx, node) in graph.nodes.iter().enumerate() {
                // First (topmost) occurrence of this node id only — otherwise a
                // re-describing overlay double-fills nodes_by_nodegroup.
                if !seen_node_ids.insert(node.nodeid.clone()) {
                    continue;
                }
                if let Some(ref ng_id) = node.nodegroup_id {
                    if !ng_id.is_empty() {
                        nodes_by_nodegroup
                            .entry(ng_id.clone())
                            .or_default()
                            .push(LayerRef {
                                layer_idx,
                                item_idx,
                            });
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
        let mut all_fxg: Vec<StaticFunctionsXGraphs> = Vec::new();

        // Every structural collection is deduped by its natural key as layers are
        // folded top-down (topmost wins), so an overlay that re-describes the same
        // model does not duplicate entries. Without this, a duplicated edge/card
        // makes the tile->tree builder emit a cardinality-n nodegroup TWICE, and a
        // duplicated fxg makes compute_tiles run the same function twice.
        // Nodes keep the topmost description, but their ONTOLOGY CLASSES are unioned
        // across layers (see the node loop) — so `node_idx_by_id` tracks where each
        // node landed rather than just whether it was seen.
        let mut node_idx_by_id: HashMap<String, usize> = HashMap::new();
        let mut seen_ng_ids = std::collections::HashSet::new();
        let mut seen_card_ngs = std::collections::HashSet::new();
        let mut seen_edges = std::collections::HashSet::new();
        let mut seen_cxnxw_ids = std::collections::HashSet::new();
        let mut seen_fxg_ids = std::collections::HashSet::new();

        for graph in self.layers.iter().rev() {
            for node in &graph.nodes {
                match node_idx_by_id.get(&node.nodeid) {
                    None => {
                        node_idx_by_id.insert(node.nodeid.clone(), all_nodes.len());
                        all_nodes.push(node.clone());
                    }
                    Some(&idx) => {
                        // The same node, re-described by a LOWER layer. Keep the
                        // topmost description, but UNION its ontology classes: a model
                        // may be typed under a different ontology in each layer, and
                        // the merged view should expose the union (`ontologyclass` is a
                        // vector). Topmost classes stay first; lower layers append any
                        // new ones, deduped. Order-preserving, so the primary (topmost)
                        // ontology leads.
                        //
                        // NOTE: edge `ontologyproperty` is single-valued, so an edge
                        // cannot yet carry the union of properties across ontologies —
                        // only classes compose. See the edge loop below (topmost wins).
                        if let Some(extra) = node.ontologyclass.as_ref() {
                            let target = all_nodes[idx].ontologyclass.get_or_insert_with(Vec::new);
                            for class in extra {
                                if !target.contains(class) {
                                    target.push(class.clone());
                                }
                            }
                        }
                    }
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
                for cw in cxnxw {
                    if seen_cxnxw_ids.insert(cw.id.clone()) {
                        all_cxnxw.push(cw.clone());
                    }
                }
            }
            if let Some(ref fxgs) = graph.functions_x_graphs {
                for fxg in fxgs {
                    if seen_fxg_ids.insert(fxg.id.clone()) {
                        all_fxg.push(fxg.clone());
                    }
                }
            }
        }

        let mut nodes_by_nodegroup_flat: HashMap<String, Vec<usize>> = HashMap::new();
        let mut node_pos_by_alias: HashMap<String, usize> = HashMap::new();
        let mut nodes_by_alias_arc: HashMap<String, Arc<StaticNode>> = HashMap::new();
        for (idx, node) in all_nodes.iter().enumerate() {
            if let Some(ref ng_id) = node.nodegroup_id {
                if !ng_id.is_empty() {
                    nodes_by_nodegroup_flat
                        .entry(ng_id.clone())
                        .or_default()
                        .push(idx);
                }
            }
            if let Some(ref alias) = node.alias {
                if !alias.is_empty() {
                    // `all_nodes` already deduped by id (topmost kept, classes unioned),
                    // so the first node carrying an alias is the authoritative one.
                    node_pos_by_alias.entry(alias.clone()).or_insert(idx);
                    nodes_by_alias_arc
                        .entry(alias.clone())
                        .or_insert_with(|| Arc::new(node.clone()));
                }
            }
        }

        MergedSlices {
            node_pos_by_id: node_idx_by_id,
            node_pos_by_alias,
            nodes_by_alias_arc,
            all_nodes,
            all_nodegroups,
            all_edges,
            all_cards,
            all_cxnxw,
            all_fxg,
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
        // Resolve into the MERGED nodes so the returned node carries the
        // cross-layer ontology-class union, not just the topmost layer's classes.
        let slices = self.merged_slices();
        slices.node_pos_by_id.get(id).map(|&i| &slices.all_nodes[i])
    }

    fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        let slices = self.merged_slices();
        slices
            .node_pos_by_alias
            .get(alias)
            .map(|&i| &slices.all_nodes[i])
    }

    fn find_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        // Indexed lookup covers all layers
        self.get_node_by_alias(alias)
    }

    fn get_node_arc_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>> {
        self.merged_slices().nodes_by_alias_arc.get(alias).cloned()
    }

    fn nodes_by_alias_arc(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        Some(&self.merged_slices().nodes_by_alias_arc)
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

    fn functions_x_graphs(&self) -> Vec<&super::cards::StaticFunctionsXGraphs> {
        // Deduped in the same unified merge as nodes/edges/cards (by id, topmost
        // wins), so a re-describing overlay does not double a declaration. Returned
        // from the memoized slice rather than recomputed per call.
        self.merged_slices().all_fxg.iter().collect()
    }

    fn build_descriptors(
        &self,
        tiles: &[super::tile::StaticTile],
    ) -> super::descriptors::StaticResourceDescriptors {
        // Descriptor templates are a base-model concern; the base layer carries
        // them and the nodes they reference. Layer 0 is always the base.
        self.layers[0].build_descriptors(tiles)
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

    fn graph_with_fxg(id: &str, fxg_id: &str) -> StaticGraph {
        let mut g: StaticGraph = serde_json::from_value(json!({
            "graphid": id,
            "name": {"en": id},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": id},
            "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": id}],
            "nodegroups": [], "edges": [],
            "functions_x_graphs": [{
                "id": fxg_id, "function_id": "fn-x", "graph_id": id, "config": {}
            }]
        }))
        .unwrap();
        g.build_indices();
        g
    }

    #[test]
    fn functions_x_graphs_union_across_layers() {
        // An overlay contributes a function declaration the base does not carry;
        // both surface through the union. This lets a computed layer bring its own
        // compute-tiles fxg without editing the base graph. Order is not
        // significant (consumers filter by function_id), so assert the set.
        let base = graph_with_fxg("base", "fxg-base");
        let overlay = graph_with_fxg("overlay", "fxg-overlay");
        let lg = LayeredGraph::new(Arc::new(base), vec![Arc::new(overlay)]);
        let mut ids: Vec<&str> = lg
            .functions_x_graphs()
            .iter()
            .map(|f| f.id.as_str())
            .collect();
        ids.sort();
        assert_eq!(
            ids,
            vec!["fxg-base", "fxg-overlay"],
            "both declarations surface"
        );
    }

    #[test]
    fn re_describing_overlay_does_not_duplicate_fxg() {
        // An overlay that re-declares the SAME fxg (same id) as the base must not
        // double it — otherwise compute_tiles would run the function twice. This
        // is the fxg analogue of the edge/node dedup.
        let base = graph_with_fxg("base", "fxg-shared");
        let overlay = graph_with_fxg("overlay", "fxg-shared");
        let lg = LayeredGraph::new(Arc::new(base), vec![Arc::new(overlay)]);
        let fxgs = lg.functions_x_graphs();
        assert_eq!(
            fxgs.len(),
            1,
            "shared fxg id deduped; got {:?}",
            fxgs.iter().map(|f| &f.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn re_describing_overlay_does_not_duplicate_cxnxw() {
        // cards_x_nodes_x_widgets deduped by id like everything else, so an
        // overlay carrying the same widget row does not double it.
        let mk = || {
            let mut g: StaticGraph = serde_json::from_value(json!({
                "graphid": "base", "name": {"en": "base"},
                "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"},
                "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"}],
                "nodegroups": [], "edges": [],
                "cards_x_nodes_x_widgets": [{
                    "id": "cw1", "card_id": "c1", "node_id": "root", "widget_id": "w1",
                    "label": {"en": "W"}, "visible": true
                }]
            }))
            .unwrap();
            g.build_indices();
            g
        };
        let lg = LayeredGraph::new(Arc::new(mk()), vec![Arc::new(mk())]);
        assert_eq!(
            lg.cards_x_nodes_x_widgets_slice().len(),
            1,
            "shared cxnxw row deduped by id"
        );
    }

    #[test]
    fn functions_x_graphs_surfaces_overlay_only_declaration() {
        // Base has none; the overlay's fxg is still visible through the lookup.
        let mut base: StaticGraph = serde_json::from_value(json!({
            "graphid": "base", "name": {"en": "base"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"},
            "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "base"}],
            "nodegroups": [], "edges": []
        }))
        .unwrap();
        base.build_indices();
        let overlay = graph_with_fxg("overlay", "fxg-overlay");
        let lg = LayeredGraph::new(Arc::new(base), vec![Arc::new(overlay)]);
        let fxgs = lg.functions_x_graphs();
        assert_eq!(fxgs.len(), 1);
        assert_eq!(fxgs[0].id, "fxg-overlay");
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

    // --- Ontology-class composition across layers --------------------------
    //
    // `ontologyclass` is a vector: the SAME node may be typed under a different
    // ontology in each layer, and the composed `LayeredGraph` must expose the UNION
    // of those classes — through every node-content accessor, not just one.
    // (`ontologyproperty` on edges is single-valued, so edges cannot yet carry a
    // cross-ontology union — noted, not tested here.)

    fn graph_typing_node(graphid: &str, n1_classes: serde_json::Value) -> StaticGraph {
        let mut g: StaticGraph = serde_json::from_value(json!({
            "graphid": graphid,
            "name": {"en": graphid},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": graphid},
            "nodes": [
                {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": graphid},
                {"nodeid": "n1", "name": "Name", "datatype": "string", "graph_id": graphid,
                 "alias": "name", "nodegroup_id": "ng1", "ontologyclass": n1_classes}
            ],
            "nodegroups": [{"nodegroupid": "ng1", "cardinality": "1"}],
            "edges": [{"edgeid": "e1", "domainnode_id": "root", "rangenode_id": "n1", "graph_id": graphid}]
        }))
        .unwrap();
        g.build_indices();
        g
    }

    #[test]
    fn layered_graph_unions_ontology_classes_across_every_accessor() {
        // Same node n1, typed under CIDOC-CRM in the base and Dublin Core in the overlay.
        let base = graph_typing_node(
            "base",
            json!(["http://www.cidoc-crm.org/cidoc-crm/E33_Linguistic_Object"]),
        );
        let overlay = graph_typing_node("overlay", json!(["http://purl.org/dc/terms/title"]));
        let lg = LayeredGraph::new(Arc::new(base), vec![Arc::new(overlay)]);

        let assert_union = |classes: &Option<Vec<String>>, via: &str| {
            let c = classes
                .as_ref()
                .unwrap_or_else(|| panic!("{via}: n1 should carry ontology classes"));
            assert!(
                c.iter().any(|s| s.contains("E33_Linguistic_Object")),
                "{via}: base ontology missing from union {c:?}"
            );
            assert!(
                c.iter().any(|s| s.contains("dc/terms/title")),
                "{via}: overlay ontology missing from union {c:?}"
            );
            assert_eq!(c.len(), 2, "{via}: exactly the two distinct classes {c:?}");
        };

        // The union must be visible no matter how the node is fetched.
        assert_union(
            &lg.get_node_by_id("n1").unwrap().ontologyclass,
            "get_node_by_id",
        );
        assert_union(
            &lg.find_node_by_alias("name").unwrap().ontologyclass,
            "find_node_by_alias",
        );
        assert_union(
            &lg.get_node_arc_by_alias("name").unwrap().ontologyclass,
            "get_node_arc_by_alias",
        );
        let in_slice = lg.nodes_slice().iter().find(|n| n.nodeid == "n1").unwrap();
        assert_union(&in_slice.ontologyclass, "nodes_slice");

        // The node's OTHER content still follows topmost-wins — only classes union.
        assert_eq!(lg.get_node_by_id("n1").unwrap().graph_id, "overlay");
    }

    #[test]
    fn ontology_union_dedupes_preserves_topmost_first_and_tolerates_missing() {
        // base n1 = [A, B]; overlay (topmost) n1 = [B, C]  -> union, B not duplicated.
        let mut base = graph_typing_node("base", json!(["urn:A", "urn:B"]));
        let mut overlay = graph_typing_node("ov", json!(["urn:B", "urn:C"]));
        // Add a second node: typed only in the overlay, absent class in the base.
        for (g, cls) in [
            (&mut base, serde_json::Value::Null),
            (&mut overlay, json!(["urn:Z"])),
        ] {
            let mut n2: StaticNode = serde_json::from_value(json!({
                "nodeid": "n2", "name": "Other", "datatype": "string",
                "graph_id": g.graphid, "alias": "other", "nodegroup_id": "ng1",
                "ontologyclass": cls
            }))
            .unwrap();
            n2.ontologyclass = n2.ontologyclass.filter(|v| !v.is_empty());
            g.nodes.push(n2);
            g.build_indices();
        }
        let lg = LayeredGraph::new(Arc::new(base), vec![Arc::new(overlay)]);

        let n1 = lg
            .get_node_by_id("n1")
            .unwrap()
            .ontologyclass
            .clone()
            .unwrap();
        assert_eq!(n1.len(), 3, "deduped union (B once): {n1:?}");
        let mut sorted = n1.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["urn:A", "urn:B", "urn:C"]);
        assert_eq!(n1[0], "urn:B", "topmost (overlay) classes lead: {n1:?}");

        // n2: no class in the base, [Z] in the overlay -> just [Z] (no loss, no panic).
        assert_eq!(
            lg.get_node_by_id("n2")
                .unwrap()
                .ontologyclass
                .clone()
                .unwrap(),
            vec!["urn:Z"]
        );
    }

    #[test]
    fn ontology_class_union_is_consistent_with_validate_edge_multi() {
        use crate::ontology::OntologyValidator;
        // `has_name` is only valid with domain `Place`. The base types the node as the
        // generic `Thing`; only the overlay adds `Place`. So the edge validates *only*
        // because the composed view unions both classes.
        const ONT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xml:base="urn:o/">
  <rdfs:Class rdf:about="Thing"/>
  <rdfs:Class rdf:about="Place"/>
  <rdfs:Class rdf:about="Name"/>
  <rdf:Property rdf:about="has_name">
    <rdfs:domain rdf:resource="Place"/>
    <rdfs:range rdf:resource="Name"/>
  </rdf:Property>
</rdf:RDF>"#;
        let validator = OntologyValidator::from_rdfs_xml(&[ONT]).unwrap();

        let base = graph_typing_node("base", json!(["urn:o/Thing"]));
        let overlay = graph_typing_node("ov", json!(["urn:o/Place"]));
        let lg = LayeredGraph::new(Arc::new(base), vec![Arc::new(overlay)]);

        let domain_union = lg
            .get_node_by_id("n1")
            .unwrap()
            .ontologyclass
            .clone()
            .unwrap();
        let range = vec!["urn:o/Name".to_string()];

        // The composed union validates (Place is in it)...
        assert!(
            validator
                .validate_edge_multi(&domain_union, "urn:o/has_name", &range)
                .is_ok(),
            "edge should validate against the unioned classes {domain_union:?}"
        );
        // ...whereas the base layer's classes ALONE would not — proving the union is
        // what the validator consumes, consistently.
        assert!(
            validator
                .validate_edge_multi(&["urn:o/Thing".to_string()], "urn:o/has_name", &range)
                .is_err(),
            "the base ontology alone must NOT validate — the union is load-bearing"
        );
    }
}
