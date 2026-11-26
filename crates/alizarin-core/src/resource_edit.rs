//! Editing a loaded resource: the core under every binding's ORM assignment
//! sugar (`place.name = "Baile"`, `place.names.push({...})`, `$commit()`), so
//! WASM, NAPI and Python edit the same way.
//!
//! A [`ResourceEdit`] holds pending changes against a copy of the wrapper's
//! stored tiles. Each change is checked against the graph when it is made
//! (unknown node, derived or semantic node, permissions, an unloaded
//! nodegroup), and coerced as tree-to-tiles coerces. [`ResourceEdit::commit`]
//! then reruns stored Derive functions for the nodegroups touched, applies the
//! result to the wrapper, and returns the tile delta for the caller to persist:
//! committing does no I/O.

use std::collections::HashSet;
use std::sync::Arc;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::graph::FunctionsRegistry;
use crate::graph_model_access::GraphModelAccess;
use crate::instance_wrapper_core::{ModelAccess, ResourceInstanceWrapperCore, TileStore};
use crate::{StaticNode, StaticTile};

/// What a commit changed in the resource's stored tiles: the tiles to write
/// (new or changed, as they now stand) and the ids of tiles to delete.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TileDelta {
    pub upserted: Vec<StaticTile>,
    pub removed: Vec<String>,
}

/// Pending changes to one resource's stored tiles. See the module docs.
pub struct ResourceEdit {
    resource_id: String,
    /// The stored tiles when the edit began.
    original: TileStore,
    /// The tiles as they will be.
    tiles: TileStore,
    /// Nodegroups changed, in the order first changed.
    touched: Vec<String>,
    /// Lazy wrappers: the nodegroups loaded, which alone may be edited.
    loaded: Option<HashSet<String>>,
}

impl ResourceEdit {
    /// An edit of `wrapper`'s stored tiles (on-load derived tiles aren't
    /// stored, so can't be edited).
    pub fn new(wrapper: &ResourceInstanceWrapperCore) -> Result<Self, String> {
        let resource_id = wrapper
            .resource_instance
            .as_ref()
            .map(|r| r.resourceinstanceid.clone())
            .ok_or("the resource has no id to edit")?;
        let original: TileStore = wrapper
            .stored_tiles()
            .into_iter()
            .filter_map(|t| t.tileid.clone().map(|id| (id, Arc::new(t.clone()))))
            .collect();
        let loaded = wrapper.is_lazy().then(|| {
            wrapper
                .loaded_nodegroups
                .read()
                .map(|l| {
                    l.iter()
                        .filter(|(_, s)| **s == crate::instance_wrapper_core::LoadState::Loaded)
                        .map(|(id, _)| id.clone())
                        .collect()
                })
                .unwrap_or_default()
        });
        Ok(ResourceEdit {
            resource_id,
            tiles: original.clone(),
            original,
            touched: Vec::new(),
            loaded,
        })
    }

    /// Whether anything has changed.
    pub fn is_empty(&self) -> bool {
        self.touched.is_empty()
    }

    /// Set node `alias`'s value, coerced as tree-to-tiles coerces it, and
    /// return the id of the tile it is in.
    ///
    /// - `tile_id` names the tile. Without one, the node's nodegroup must be
    ///   cardinality-1: its tile under `parent_tile_id` (the resource's root
    ///   when `None`) is used, or created with its canonical id.
    /// - A translatable `string` given as a plain string sets only the current
    ///   language, keeping the others; an object sets them all; `null` clears
    ///   the node.
    /// - An outer node (a value with children) given its tree form,
    ///   `{"_": value, child: …}`, takes `_` as its own value and sets the
    ///   children too, as tree-to-tiles reads it (see `set_outer_tree`).
    pub fn set_value(
        &mut self,
        tile_id: Option<&str>,
        parent_tile_id: Option<&str>,
        alias: &str,
        value: &Value,
        model: &GraphModelAccess,
        registry: Option<&ExtensionTypeRegistry>,
    ) -> Result<String, String> {
        let node = writable_node(alias, model)?;
        if let Some(tree) = outer_tree(&node, value, model) {
            return self.set_outer_tree(tile_id, parent_tile_id, &node, tree, model, registry);
        }
        let nodegroup_id = node.nodegroup_id.clone().unwrap_or_default();
        self.check_loaded(&nodegroup_id)?;
        let id = self.value_slot(tile_id, parent_tile_id, alias, &nodegroup_id, model)?;

        let mut tile = StaticTile::clone(self.tiles.get(&id).expect("slot found or made"));
        let current = tile.data.get(&node.nodeid).cloned();
        tile.data.insert(
            node.nodeid.clone(),
            coerce(&node, value, current, registry)?,
        );
        if !model.is_tile_permitted(&tile) {
            return Err(format!("{alias}: this model doesn't permit editing it"));
        }
        self.tiles.insert(id.clone(), Arc::new(tile));
        self.touch(&nodegroup_id);
        Ok(id)
    }

    /// The tile node `alias` of `nodegroup_id` takes its value in: `tile_id`'s
    /// (which must be of that nodegroup), else its cardinality-1 slot under
    /// `parent_tile_id`, made if missing.
    fn value_slot(
        &mut self,
        tile_id: Option<&str>,
        parent_tile_id: Option<&str>,
        alias: &str,
        nodegroup_id: &str,
        model: &GraphModelAccess,
    ) -> Result<String, String> {
        match tile_id {
            Some(id) => {
                let tile = self
                    .tiles
                    .get(id)
                    .ok_or_else(|| format!("no tile {id} to set {alias} in"))?;
                if tile.nodegroup_id != nodegroup_id {
                    return Err(format!(
                        "{alias} is in nodegroup {nodegroup_id}, not tile {id}'s ({})",
                        tile.nodegroup_id
                    ));
                }
                Ok(id.to_string())
            }
            None => self.single_slot(nodegroup_id, parent_tile_id, alias, model),
        }
    }

    /// Set outer node `node` from its tree form, as tree-to-tiles reads one:
    /// `tree["_"]` is its own value, and every other key one of its children,
    /// set in its tile (a child in its nodegroup) or in the cardinality-1
    /// tile of the child's own nodegroup under it. A semantic or multi-valued
    /// child is refused: add or set it on its own. All or nothing.
    fn set_outer_tree(
        &mut self,
        tile_id: Option<&str>,
        parent_tile_id: Option<&str>,
        node: &StaticNode,
        tree: &Map<String, Value>,
        model: &GraphModelAccess,
        registry: Option<&ExtensionTypeRegistry>,
    ) -> Result<String, String> {
        let saved = (self.tiles.clone(), self.touched.clone());
        let result = (|| {
            let alias = node.alias.as_deref().unwrap_or(&node.nodeid);
            let nodegroup_id = node.nodegroup_id.clone().unwrap_or_default();
            self.check_loaded(&nodegroup_id)?;
            let id = self.value_slot(tile_id, parent_tile_id, alias, &nodegroup_id, model)?;
            if let Some(own) = tree.get("_") {
                self.set_value(Some(&id), None, alias, own, model, registry)?;
            }
            let children = model
                .get_edges_internal()
                .and_then(|edges| edges.get(&node.nodeid))
                .cloned()
                .unwrap_or_default();
            for (key, value) in tree.iter().filter(|(key, _)| key.as_str() != "_") {
                let child = model
                    .get_node_by_alias(key)
                    .filter(|child| children.contains(&child.nodeid))
                    .ok_or_else(|| format!("{key} isn't a child of {alias}"))?;
                let child_nodegroup = child.nodegroup_id.as_deref().unwrap_or_default();
                if child_nodegroup == nodegroup_id {
                    self.set_value(Some(&id), None, key, value, model, registry)?;
                } else if child.datatype != "semantic" && is_single(child_nodegroup, model) {
                    self.set_value(None, Some(&id), key, value, model, registry)?;
                } else {
                    return Err(format!(
                        "{key} is under {alias} in its own nodegroup: add or set it on its own"
                    ));
                }
            }
            Ok(id)
        })();
        if result.is_err() {
            (self.tiles, self.touched) = saved;
        }
        result
    }

    /// Add a tile to the nodegroup of node `alias` (its grouping node or any
    /// node in it), under `parent_tile_id`, with `values` by alias (nodes of
    /// that nodegroup only); return its id. A cardinality-n tile gets the next
    /// free multi-tile id and goes last; a cardinality-1 slot must be empty.
    pub fn add_tile(
        &mut self,
        alias: &str,
        parent_tile_id: Option<&str>,
        values: &Map<String, Value>,
        model: &GraphModelAccess,
        registry: Option<&ExtensionTypeRegistry>,
    ) -> Result<String, String> {
        let node = model
            .get_node_by_alias(alias)
            .ok_or_else(|| format!("no node {alias}"))?;
        let nodegroup_id = node
            .nodegroup_id
            .clone()
            .ok_or_else(|| format!("{alias} has no nodegroup"))?;
        check_not_derived(alias, &nodegroup_id, model)?;
        self.check_loaded(&nodegroup_id)?;
        let parent = self.check_parent(&nodegroup_id, parent_tile_id, model)?;

        let id = if is_single(&nodegroup_id, model) {
            if self.slot_tile(&nodegroup_id, parent.as_deref()).is_some() {
                return Err(format!(
                    "{alias} is single-valued and already has a tile here: set its values"
                ));
            }
            crate::canonical_tile_id(&self.resource_id, &nodegroup_id, parent.as_deref())
        } else {
            let siblings = self
                .tiles
                .values()
                .filter(|t| t.nodegroup_id == nodegroup_id)
                .count() as u32;
            crate::allocate_multi_tile_id(None, &self.resource_id, &nodegroup_id, siblings, |id| {
                self.tiles.contains_key(id) || self.original.contains_key(id)
            })
            .0
        };
        let sortorder = self
            .tiles
            .values()
            .filter(|t| t.nodegroup_id == nodegroup_id && t.parenttile_id == parent)
            .filter_map(|t| t.sortorder)
            .max()
            .map_or(0, |s| s + 1);
        let mut tile = StaticTile::new_empty(nodegroup_id.clone());
        tile.tileid = Some(id.clone());
        tile.resourceinstance_id = self.resource_id.clone();
        tile.parenttile_id = parent;
        tile.sortorder = Some(sortorder);
        if !model.is_tile_permitted(&tile) {
            return Err(format!("{alias}: this model doesn't permit adding to it"));
        }

        // All or nothing: values go in on a copy.
        let saved = (self.tiles.clone(), self.touched.clone());
        self.tiles.insert(id.clone(), Arc::new(tile));
        self.touch(&nodegroup_id);
        for (child, value) in values {
            let child_node = writable_node(child, model)?;
            let result = if child_node.nodegroup_id.as_deref() != Some(nodegroup_id.as_str()) {
                Err(format!(
                    "{child} isn't in {alias}'s nodegroup: add its own tile under this one"
                ))
            } else {
                self.set_value(Some(&id), None, child, value, model, registry)
                    .map(|_| ())
            };
            if let Err(e) = result {
                (self.tiles, self.touched) = saved;
                return Err(e);
            }
        }
        Ok(id)
    }

    /// Remove tile `tile_id` and every tile under it.
    pub fn remove_tile(&mut self, tile_id: &str, model: &GraphModelAccess) -> Result<(), String> {
        let tile = self
            .tiles
            .get(tile_id)
            .ok_or_else(|| format!("no tile {tile_id} to remove"))?;
        let nodegroup_id = tile.nodegroup_id.clone();
        check_not_derived(&nodegroup_id, &nodegroup_id, model)?;
        self.check_loaded(&nodegroup_id)?;
        if !model.is_tile_permitted(tile) {
            return Err(format!(
                "tile {tile_id}: this model doesn't permit removing it"
            ));
        }
        let mut doomed = vec![tile_id.to_string()];
        let mut i = 0;
        while i < doomed.len() {
            let parent = doomed[i].clone();
            doomed.extend(
                self.tiles
                    .values()
                    .filter(|t| t.parenttile_id.as_deref() == Some(parent.as_str()))
                    .filter_map(|t| t.tileid.clone()),
            );
            i += 1;
        }
        for id in doomed {
            if let Some(tile) = self.tiles.shift_remove(&id) {
                self.touch(&tile.nodegroup_id);
            }
        }
        Ok(())
    }

    /// Rerun the stored Derive functions whose sources were touched, apply
    /// the result to `wrapper` (rerunning on-load ones), and return what
    /// changed. All or nothing: on `Err`, `wrapper` is untouched.
    pub fn commit(
        self,
        wrapper: &mut ResourceInstanceWrapperCore,
        model: &GraphModelAccess,
        functions: &FunctionsRegistry,
    ) -> Result<TileDelta, String> {
        if self.touched.is_empty() {
            return Ok(TileDelta::default());
        }
        let mut tiles: Vec<StaticTile> =
            self.tiles.into_values().map(Arc::unwrap_or_clone).collect();
        crate::apply_stored_derive_functions(
            &mut tiles,
            model.get_graph(),
            &self.resource_id,
            Some(&self.touched),
            functions,
        )?;

        let kept: HashSet<&str> = tiles.iter().filter_map(|t| t.tileid.as_deref()).collect();
        let delta = TileDelta {
            upserted: tiles
                .iter()
                .filter(|t| {
                    t.tileid
                        .as_deref()
                        .and_then(|id| self.original.get(id))
                        .is_none_or(|old| {
                            old.data != t.data
                                || old.parenttile_id != t.parenttile_id
                                || old.sortorder != t.sortorder
                        })
                })
                .cloned()
                .collect(),
            removed: self
                .original
                .keys()
                .filter(|id| !kept.contains(id.as_str()))
                .cloned()
                .collect(),
        };
        wrapper.apply_tile_delta(&delta, model);
        Ok(delta)
    }

    fn touch(&mut self, nodegroup_id: &str) {
        if !self.touched.iter().any(|t| t == nodegroup_id) {
            self.touched.push(nodegroup_id.to_string());
        }
    }

    fn check_loaded(&self, nodegroup_id: &str) -> Result<(), String> {
        match &self.loaded {
            Some(loaded) if !loaded.contains(nodegroup_id) => {
                Err(format!("TILES_NOT_LOADED:{nodegroup_id}"))
            }
            _ => Ok(()),
        }
    }

    /// The parent `parent_tile_id` for a tile in `nodegroup_id`, checked: a
    /// nested nodegroup needs one, in its parent nodegroup; a top-level one
    /// none.
    fn check_parent(
        &self,
        nodegroup_id: &str,
        parent_tile_id: Option<&str>,
        model: &GraphModelAccess,
    ) -> Result<Option<String>, String> {
        let parent_nodegroup = model
            .get_nodegroups()
            .and_then(|ngs| ngs.get(nodegroup_id).cloned())
            .and_then(|ng| ng.parentnodegroup_id.clone());
        match (parent_nodegroup, parent_tile_id) {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(format!(
                "nodegroup {nodegroup_id} is top-level: it takes no parent tile"
            )),
            (Some(_), None) => Err(format!(
                "nodegroup {nodegroup_id} is nested: name the parent tile"
            )),
            (Some(expected), Some(parent)) => match self.tiles.get(parent) {
                Some(tile) if tile.nodegroup_id == expected => Ok(Some(parent.to_string())),
                Some(tile) => Err(format!(
                    "tile {parent} is in nodegroup {}, not {nodegroup_id}'s parent {expected}",
                    tile.nodegroup_id
                )),
                None => Err(format!("no parent tile {parent}")),
            },
        }
    }

    fn slot_tile(&self, nodegroup_id: &str, parent: Option<&str>) -> Option<String> {
        self.tiles
            .values()
            .find(|t| t.nodegroup_id == nodegroup_id && t.parenttile_id.as_deref() == parent)
            .and_then(|t| t.tileid.clone())
    }

    /// The cardinality-1 tile of `nodegroup_id` under `parent_tile_id`,
    /// created if missing.
    fn single_slot(
        &mut self,
        nodegroup_id: &str,
        parent_tile_id: Option<&str>,
        alias: &str,
        model: &GraphModelAccess,
    ) -> Result<String, String> {
        if !is_single(nodegroup_id, model) {
            return Err(format!(
                "{alias} is multi-valued: name the tile to set it in, or add one"
            ));
        }
        let parent = self.check_parent(nodegroup_id, parent_tile_id, model)?;
        if let Some(id) = self.slot_tile(nodegroup_id, parent.as_deref()) {
            return Ok(id);
        }
        let id = crate::canonical_tile_id(&self.resource_id, nodegroup_id, parent.as_deref());
        let mut tile = StaticTile::new_empty(nodegroup_id.to_string());
        tile.tileid = Some(id.clone());
        tile.resourceinstance_id = self.resource_id.clone();
        tile.parenttile_id = parent;
        self.tiles.insert(id.clone(), Arc::new(tile));
        Ok(id)
    }
}

/// A tile named in an [`EditOp`]: an existing tile's id, or the tile an
/// earlier `add` in the same batch made (`{"ref": index}`).
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum TileRef {
    Id(String),
    Ref { r#ref: usize },
}

/// One change in a batch for [`ResourceInstanceWrapperCore::apply_edits`]:
/// the form every binding's ORM records assignments in.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum EditOp {
    /// [`ResourceEdit::set_value`].
    Set {
        #[serde(default)]
        tile: Option<TileRef>,
        #[serde(default)]
        parent: Option<TileRef>,
        alias: String,
        value: Value,
    },
    /// [`ResourceEdit::add_tile`].
    Add {
        alias: String,
        #[serde(default)]
        parent: Option<TileRef>,
        #[serde(default)]
        values: Map<String, Value>,
    },
    /// [`ResourceEdit::remove_tile`].
    Remove { tile: TileRef },
}

impl ResourceInstanceWrapperCore {
    /// An error if node `alias` can't be given a value here: unknown, semantic,
    /// derived, or (lazy) in a nodegroup not loaded. What an ORM checks when a
    /// value is assigned, before recording it.
    pub fn check_editable(&self, alias: &str, model: &GraphModelAccess) -> Result<(), String> {
        let node = writable_node(alias, model)?;
        let nodegroup_id = node.nodegroup_id.as_deref().unwrap_or_default();
        if self.is_lazy() && !self.is_nodegroup_loaded(nodegroup_id) {
            return Err(format!("TILES_NOT_LOADED:{nodegroup_id}"));
        }
        Ok(())
    }

    /// Apply a batch of edits (as an ORM recorded them) and commit them with
    /// the process-wide extension and functions registries: all or nothing.
    /// An error names the failing edit by its index.
    pub fn apply_edits(
        &mut self,
        ops: &[EditOp],
        model: &GraphModelAccess,
    ) -> Result<TileDelta, String> {
        let registry = crate::get_global_extension_registry();
        let mut edit = ResourceEdit::new(self)?;
        let mut added: Vec<Option<String>> = Vec::with_capacity(ops.len());
        let resolve = |tile: &Option<TileRef>, added: &[Option<String>]| match tile {
            None => Ok(None),
            Some(TileRef::Id(id)) => Ok(Some(id.clone())),
            Some(TileRef::Ref { r#ref }) => added
                .get(*r#ref)
                .cloned()
                .flatten()
                .map(Some)
                .ok_or_else(|| format!("ref {ref} names no earlier add", r#ref = r#ref)),
        };
        for (index, op) in ops.iter().enumerate() {
            let result = match op {
                EditOp::Set {
                    tile,
                    parent,
                    alias,
                    value,
                } => {
                    let tile = resolve(tile, &added);
                    let parent = resolve(parent, &added);
                    tile.and_then(|tile| parent.map(|parent| (tile, parent)))
                        .and_then(|(tile, parent)| {
                            edit.set_value(
                                tile.as_deref(),
                                parent.as_deref(),
                                alias,
                                value,
                                model,
                                registry.as_ref(),
                            )
                        })
                        .map(|_| None)
                }
                EditOp::Add {
                    alias,
                    parent,
                    values,
                } => resolve(parent, &added).and_then(|parent| {
                    edit.add_tile(alias, parent.as_deref(), values, model, registry.as_ref())
                        .map(Some)
                }),
                EditOp::Remove { tile } => resolve(&Some(tile.clone()), &added)
                    .and_then(|tile| edit.remove_tile(&tile.unwrap_or_default(), model))
                    .map(|_| None),
            };
            added.push(result.map_err(|e| format!("edit {index}: {e}"))?);
        }
        edit.commit(self, model, &crate::get_global_functions_registry())
    }

    /// Apply a committed [`TileDelta`]: remove and write tiles, drop every
    /// cached value (the ORMs rebuild theirs from these tiles), and rerun the
    /// on-load Derive functions.
    pub fn apply_tile_delta(&mut self, delta: &TileDelta, model: &GraphModelAccess) {
        for id in &delta.removed {
            if let Some(tile) = self.tiles.as_mut().and_then(|t| t.shift_remove(id)) {
                if let Some(ids) = self.nodegroup_index.get_mut(&tile.nodegroup_id) {
                    ids.retain(|i| i != id);
                }
            }
        }
        // Also drops the child tile index, removals or not.
        self.merge_tiles(delta.upserted.clone());
        if let Ok(mut cache) = self.pseudo_cache.write() {
            cache.clear();
        }
        self.derive_on_load_with(model);
    }
}

/// Node `alias`, if it can be given a value: it exists, holds a value (isn't
/// semantic) and isn't filled by a Derive function.
fn writable_node(
    alias: &str,
    model: &GraphModelAccess,
) -> Result<std::sync::Arc<StaticNode>, String> {
    let node = model
        .get_node_by_alias(alias)
        .ok_or_else(|| format!("no node {alias}"))?;
    if node.datatype == "semantic" {
        return Err(format!(
            "{alias} is a semantic node: set the nodes under it"
        ));
    }
    let nodegroup_id = node
        .nodegroup_id
        .clone()
        .ok_or_else(|| format!("{alias} has no nodegroup"))?;
    check_not_derived(alias, &nodegroup_id, model)?;
    Ok(node)
}

/// An error if a Derive function (stored or on-load) fills `nodegroup_id`.
fn check_not_derived(
    alias: &str,
    nodegroup_id: &str,
    model: &GraphModelAccess,
) -> Result<(), String> {
    let stored = crate::stored_derive_functions(model.get_graph()).unwrap_or_default();
    let on_load = model.on_load_functions();
    match stored
        .iter()
        .chain(on_load.iter())
        .find(|f| f.nodegroup == nodegroup_id)
    {
        Some(f) => Err(format!(
            "{alias} is derived by {}: edit its sources instead",
            f.provider
        )),
        None => Ok(()),
    }
}

/// `value` as `node`'s tree form, if it is one: an object with an `_` key,
/// given to an outer node (one that holds a value and has children). Without
/// `_`, an object is the node's own value (a string's languages, say).
fn outer_tree<'v>(
    node: &StaticNode,
    value: &'v Value,
    model: &GraphModelAccess,
) -> Option<&'v Map<String, Value>> {
    let tree = value.as_object().filter(|tree| tree.contains_key("_"))?;
    let has_children = model
        .get_edges_internal()
        .and_then(|edges| edges.get(&node.nodeid))
        .is_some_and(|children| !children.is_empty());
    (node.datatype != "semantic" && has_children).then_some(tree)
}

fn is_single(nodegroup_id: &str, model: &GraphModelAccess) -> bool {
    model
        .get_nodegroups()
        .and_then(|ngs| ngs.get(nodegroup_id).cloned())
        .and_then(|ng| ng.cardinality.clone())
        .is_none_or(|c| c != "n")
}

/// `value` coerced for `node`, given its `current` tile data: a plain string
/// for a translatable string merges into the current language.
fn coerce(
    node: &StaticNode,
    value: &Value,
    current: Option<Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<Value, String> {
    let alias = node.alias.as_deref().unwrap_or(&node.nodeid);
    if value.is_null() {
        return Ok(Value::Null);
    }
    let config = (!node.config.is_empty()).then(|| {
        Value::Object(
            node.config
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    });
    let coerced = crate::type_coercion::coerce_value_with_registry(
        &node.datatype,
        value,
        config.as_ref(),
        registry,
    );
    if let Some(error) = coerced.error {
        return Err(format!("{alias} ({}): {error}", node.datatype));
    }
    if node.datatype == "string" && value.is_string() {
        if let (Some(Value::Object(mut merged)), Value::Object(new)) =
            (current, coerced.tile_data.clone())
        {
            merged.extend(new);
            return Ok(Value::Object(merged));
        }
    }
    Ok(coerced.tile_data)
}
