//! Tile type for storing data instances.
//!
//! Also home to [`canonical_tile_id`]: the deterministic (uuid5) tile-id
//! derivation that lets independently built layers address the same
//! cardinality-1 tile without either having seen the other's data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Stable namespace URI for tile-id derivation. Hashed into a namespace UUID
/// exactly as `rdm_namespace` does for RDM ids (uuid5 under `NAMESPACE_URL`),
/// so the whole codebase derives ids the one way.
///
/// CHANGING THIS STRING RE-IDENTIFIES EVERY CANONICAL TILE. It is part of the
/// cross-layer contract: two builds that disagree on it cannot address each
/// other's tiles.
pub const TILE_NAMESPACE_URI: &str = "https://alizarin.flaxandteal.co.uk/ns/tile";

/// The TILE namespace UUID: `uuid5(NAMESPACE_URL, TILE_NAMESPACE_URI)`.
pub fn tile_namespace() -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, TILE_NAMESPACE_URI.as_bytes())
}

/// Derive the canonical (deterministic) id of a **cardinality-1** tile.
///
/// # Why
///
/// Layers are built independently: a standalone or on-device layer may never
/// have seen the base it will be composed over. For such a layer to OVERRIDE a
/// cardinality-1 tile, or to hang children off a base tile it does not
/// possess, it must be able to COMPUTE that tile's id rather than look it up.
/// Random (v4) tile ids make that impossible. This function makes it possible.
///
/// # The key
///
/// The invariant is: **one tile per cardinality-1 nodegroup, PER PARENT
/// INSTANCE**. So the key is `(scope, nodegroup)`, where the scope is the
/// parent tile instance if there is one and the resource otherwise:
///
/// * **Cardinality-1 tile WITH a parent**:
///   `uuid5(TILE_NS, "{parent_tileid}/{nodegroup_id}")`
///
/// * **Cardinality-1 tile at ROOT (no parent)**:
///   `uuid5(TILE_NS, "{resource_id}/{nodegroup_id}")`, computable with NO
///   access to the base at all, which is what lets a standalone layer address
///   a root tile it has never seen.
///
/// * **Cardinality-n tiles do not appear here.** They keep whatever id they
///   were minted with (Arches, or v4). Uniqueness is all they need. The
///   consequence is deliberate: a standalone layer *cannot guess* an existing
///   cardinality-n tile id, so it can only ADD to a multi-valued nodegroup,
///   never silently replace a member of one.
///
/// # DO NOT "simplify" either half of the key, both halves are load-bearing
///
/// **Why the scope may not be the resource when there is a parent.** A
/// cardinality-1 nodegroup nested under a cardinality-n parent has MANY
/// instances per resource, one per parent instance. A `(resource, nodegroup)`
/// key would collapse every one of them onto a single id.
///
/// **Why the nodegroup may not be dropped when there is a parent** (i.e. why
/// the key is not the bare `parent_tileid`). A parent nodegroup may have
/// SEVERAL cardinality-1 CHILD NODEGROUPS, whose tiles all hang off the same
/// parent instance. A bare-parent key hands those sibling tiles the same id,
/// two distinct tiles, one id, which does not merely duplicate an id, it
/// FUSES them: whichever is written second wins, and everything parented
/// beneath the loser is silently mis-parented. Appending `/{nodegroup_id}` is
/// free: a layer computing the id necessarily knows which nodegroup it is
/// addressing.
///
/// Together those two constraints make `(parent instance | resource, nodegroup)`
/// the unique total, collision-free key. See the sibling-collision unit test.
///
/// # Where ids are actually assigned
///
/// Canonical ids require GRAPH CONTEXT: you cannot know a tile is
/// cardinality-1 without the nodegroup. So they are assigned on the
/// graph-aware creation path only: `json_conversion::tree_to_tiles`, which has
/// the [`StaticGraph`](crate::graph::StaticGraph) and therefore knows both
/// cardinality and parenttile_id.
///
/// [`StaticTile::ensure_id`] has no graph and keeps minting v4. That is not an
/// oversight: a tile in isolation cannot be classified.
///
/// # Accepted limitation: Arches-imported tiles
///
/// Tiles imported from an ARCHES EXPORT carry ids Arches minted at random. They
/// are NOT canonical and NOT composable, and nothing rewrites them. A layer
/// cannot compute the id of an Arches-originated cardinality-1 tile, so it
/// cannot override it. Only alizarin-GENERATED tiles have derivable
/// cardinality-1 ids.
///
/// # Examples
///
/// ```
/// use alizarin_core::graph::canonical_tile_id;
///
/// // Root cardinality-1 tile: derivable from resource + nodegroup alone.
/// let root = canonical_tile_id("res-1", "ng-a", None);
/// assert_eq!(root, canonical_tile_id("res-1", "ng-a", None)); // deterministic
///
/// // Nested cardinality-1 tile: keyed on the PARENT INSTANCE, so two parent
/// // instances of the same nodegroup give two distinct child ids.
/// let a = canonical_tile_id("res-1", "ng-child", Some("parent-tile-1"));
/// let b = canonical_tile_id("res-1", "ng-child", Some("parent-tile-2"));
/// assert_ne!(a, b);
/// ```
pub fn canonical_tile_id(
    resource_id: &str,
    nodegroup_id: &str,
    parent_tileid: Option<&str>,
) -> String {
    let name = match parent_tileid {
        // Parented: the parent instance is the scope. The resource does not
        // appear: it is implied by the parent, and including it would make
        // the id un-derivable for a layer that knows only the parent tile.
        Some(parent) => format!("{}/{}", parent, nodegroup_id),
        // Root: one per (resource, nodegroup).
        None => format!("{}/{}", resource_id, nodegroup_id),
    };
    Uuid::new_v5(&tile_namespace(), name.as_bytes()).to_string()
}

/// A tile containing data for a nodegroup instance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticTile {
    #[serde(default)]
    pub data: HashMap<String, serde_json::Value>,
    pub nodegroup_id: String,
    pub resourceinstance_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tileid: Option<String>,
    #[serde(default)]
    pub parenttile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisionaledits: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

    /// Ensure this tile has an ID, generating one if needed.
    ///
    /// Mints a RANDOM (v4) id, and deliberately so: this method has no
    /// [`StaticGraph`](crate::graph::StaticGraph), so it cannot know whether
    /// the tile's nodegroup is cardinality-1, and a canonical id is only
    /// meaningful (and only safe) for cardinality-1 tiles. COMPOSABLE IDS
    /// REQUIRE THE GRAPH-AWARE PATH (`json_conversion::tree_to_tiles`) which
    /// routes cardinality-1 tiles through [`canonical_tile_id`].
    ///
    /// Do not "fix" this by guessing a cardinality here.
    pub fn ensure_id(&mut self) -> String {
        if self.tileid.is_none() {
            self.tileid = Some(uuid::Uuid::new_v4().to_string());
        }
        self.tileid.clone().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_id_is_deterministic() {
        let a = canonical_tile_id("res-1", "ng-a", None);
        let b = canonical_tile_id("res-1", "ng-a", None);
        assert_eq!(a, b);
        assert_eq!(a.len(), 36);
        assert_eq!(Uuid::parse_str(&a).unwrap().get_version_num(), 5);
    }

    #[test]
    fn root_branch_keys_on_resource_and_nodegroup() {
        let base = canonical_tile_id("res-1", "ng-a", None);
        assert_ne!(base, canonical_tile_id("res-2", "ng-a", None));
        assert_ne!(base, canonical_tile_id("res-1", "ng-b", None));
    }

    #[test]
    fn parent_branch_differs_from_root_branch() {
        let root = canonical_tile_id("res-1", "ng-a", None);
        let parented = canonical_tile_id("res-1", "ng-a", Some("parent-1"));
        assert_ne!(root, parented);
    }

    #[test]
    fn parent_branch_ignores_resource_id() {
        let a = canonical_tile_id("res-1", "ng-a", Some("parent-1"));
        let b = canonical_tile_id("res-DIFFERENT", "ng-a", Some("parent-1"));
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_parent_instances_give_distinct_ids() {
        let a = canonical_tile_id("res-1", "ng-child", Some("parent-tile-1"));
        let b = canonical_tile_id("res-1", "ng-child", Some("parent-tile-2"));
        assert_ne!(a, b, "one tile per parent instance, ids must not collide");
    }

    #[test]
    fn sibling_cardinality_one_nodegroups_under_one_parent_do_not_collide() {
        let child_a = canonical_tile_id("res-1", "ng-child-a", Some("parent-tile-1"));
        let child_b = canonical_tile_id("res-1", "ng-child-b", Some("parent-tile-1"));
        assert_ne!(
            child_a, child_b,
            "sibling cardinality-1 nodegroups share a parent instance but are \
             distinct tiles, the nodegroup must be part of the key"
        );
    }

    #[test]
    fn namespace_is_stable() {
        assert_eq!(
            tile_namespace(),
            Uuid::new_v5(&Uuid::NAMESPACE_URL, TILE_NAMESPACE_URI.as_bytes())
        );
    }
}
