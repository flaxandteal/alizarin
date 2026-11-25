//! The plain graph data an extension reads: nodes, nodegroups, edges,
//! translatable strings and tiles. alizarin-core re-exports these from
//! `graph`, so host and extension name the same types. The graph itself (and
//! `GraphLookup`, cards, descriptors) stays in core: only the host has one.

mod node_compare;
mod nodes;
mod tile;
mod translatable;

pub use node_compare::{compare_nodes, compare_nodes_from_json, NodeComparison};
pub use nodes::{StaticEdge, StaticNode, StaticNodegroup};
pub use tile::StaticTile;
pub use translatable::StaticTranslatableString;
