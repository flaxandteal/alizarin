//! Read-only graph lookup trait.
//!
//! [`GraphLookup`] abstracts the read-only query surface of a graph model so
//! that callers (json_conversion, card_traversal, resource merging, …) can
//! operate on either a single [`StaticGraph`] or a [`LayeredGraph`] stack
//! without caring which.
//!
//! Design constraint: when the concrete type is `StaticGraph`, generic
//! dispatch (`impl GraphLookup`) monomorphises to the same code as a direct
//! `&StaticGraph` call — zero vtable, zero indirection, zero extra allocation.

use std::sync::Arc;

use super::card_index::CardIndex;
use super::cards::{StaticCard, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs};
use super::descriptors::StaticResourceDescriptors;
use super::nodes::{StaticEdge, StaticNode, StaticNodegroup};
use super::tile::StaticTile;
use super::translatable::StaticTranslatableString;

use std::collections::HashMap;

/// Read-only access to a graph's nodes, nodegroups, edges, and cards.
///
/// Every method here has a default-less signature matching the existing
/// [`StaticGraph`] method of the same name, so the impl for `StaticGraph`
/// is pure delegation and the compiler can inline it away.
pub trait GraphLookup {
    // ── Identity ────────────────────────────────────────────────────────

    fn graph_id(&self) -> &str;

    fn display_name(&self) -> String;

    fn get_model_class_name(&self) -> Option<String>;

    fn name(&self) -> &StaticTranslatableString;

    // ── Root ────────────────────────────────────────────────────────────

    fn get_root(&self) -> &StaticNode;

    fn root_node(&self) -> &StaticNode {
        self.get_root()
    }

    // ── Node lookups ────────────────────────────────────────────────────

    fn get_node_by_id(&self, id: &str) -> Option<&StaticNode>;

    fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode>;

    fn find_node_by_alias(&self, alias: &str) -> Option<&StaticNode>;

    fn get_node_arc_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>>;

    fn nodes_by_alias_arc(&self) -> Option<&HashMap<String, Arc<StaticNode>>>;

    // ── Nodegroup lookups ───────────────────────────────────────────────

    fn get_nodegroup_by_id(&self, nodegroup_id: &str) -> Option<&StaticNodegroup>;

    fn get_nodes_in_nodegroup(&self, nodegroup_id: &str) -> Vec<&StaticNode>;

    // ── Edge / tree traversal ───────────────────────────────────────────

    fn edges_map(&self) -> Option<&HashMap<String, Vec<String>>>;

    fn get_child_ids(&self, node_id: &str) -> Option<&Vec<String>>;

    fn get_children(&self, node_id: &str) -> Vec<&StaticNode> {
        self.get_child_ids(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.get_node_by_id(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn has_children(&self, node_id: &str) -> bool {
        self.get_child_ids(node_id)
            .is_some_and(|ids| !ids.is_empty())
    }

    // ── Card lookups ────────────────────────────────────────────────────

    fn find_card_by_nodegroup(&self, nodegroup_id: &str) -> Option<&StaticCard>;

    fn cards_slice(&self) -> &[StaticCard];

    fn card_index(&self) -> Option<&CardIndex>;

    fn cards_x_nodes_x_widgets_slice(&self) -> &[StaticCardsXNodesXWidgets];

    // ── Slice access (for iteration / export) ───────────────────────────

    fn nodes_slice(&self) -> &[StaticNode];

    fn nodegroups_slice(&self) -> &[StaticNodegroup];

    fn edges_slice(&self) -> &[StaticEdge];

    // ── Index-level access ──────────────────────────────────────────────

    fn nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<usize>>>;

    // ── Functions (compute-tiles / descriptors) ─────────────────────────

    /// `functions_x_graphs` declarations attached to this graph (e.g. the
    /// compute-tiles function that materialises a nodegroup's tiles on demand).
    ///
    /// Default: none. Implementors that carry declarations override this — the
    /// default keeps the trait backwards-compatible for existing implementors.
    fn functions_x_graphs(&self) -> Vec<&StaticFunctionsXGraphs> {
        Vec::new()
    }

    /// Build resource descriptors for `tiles` from this graph's descriptor
    /// configuration.
    ///
    /// Default: empty. Implementors carrying descriptor templates override this
    /// (the default keeps the trait backwards-compatible).
    fn build_descriptors(&self, _tiles: &[StaticTile]) -> StaticResourceDescriptors {
        StaticResourceDescriptors::default()
    }
}
