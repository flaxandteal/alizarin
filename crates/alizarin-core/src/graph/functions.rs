//! Graph-attached function registry — a UUID-keyed, cross-FFI lookup for the
//! functions declared in `functions_x_graphs`.
//!
//! Two kinds today: **Descriptor** (tiles → display string) and **Derive**
//! (existing node values → derived nodegroup tiles). It mirrors the
//! [`ExtensionTypeRegistry`](crate::extension_type_registry) pattern used for
//! datatypes: one Rust registry embedded by every binding, a serializable
//! declaration (the `functions_x_graphs` entry) that travels with the graph, and
//! providers that self-register by UUID. So a provider crate (e.g.
//! `alizarin-gramadan`) is written once and reached from native / napi / wasm /
//! python without duplication.
//!
//! # Purity is the contract
//!
//! Derive/descriptor functions MUST be pure and deterministic — node values +
//! config in, tiles/string out, no side effects. That is what keeps derived
//! tiles cacheable, trusted-by-code, and reproducibly attestable. The
//! side-effecting Arches hooks (`post_save`, external I/O) are deliberately NOT
//! this mechanism.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::computed_tiles::{compute_tiles_functions, ComputeTilesConfig};
use super::descriptors::AGGREGATE_FUNCTION_ID;
use super::graph_lookup::GraphLookup;
use super::tile::StaticTile;
use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::slice_source::{GraphSource, SliceMember, SliceSource};

/// A **Derive** function: read the resource's EXISTING node values (`tiles`) and
/// produce derived tiles for a target nodegroup.
///
/// The tiles-in input is the key generalisation over the older
/// [`ComputeTilesProvider`](super::computed_tiles::ComputeTilesProvider): it lets
/// a *transform* read the value it converts (latlng → UTM reads the latlng tile),
/// while a *generator* reads its inputs the same way (Gramadán reads the headword
/// tile). MUST be pure and deterministic.
pub trait DeriveProvider: Send + Sync {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String>;
}

/// A **Descriptor** function: read the resource's tiles and produce a display
/// string (name/description). The registry carries this kind so the existing
/// descriptor path can migrate onto it; `config` is opaque JSON for now.
pub trait DescriptorProvider: Send + Sync {
    fn describe(
        &self,
        resource_id: &str,
        tiles: &[StaticTile],
        config: &serde_json::Value,
    ) -> Result<String, String>;
}

/// An **Aggregate** function: read a slice of OTHER resources and compute node
/// values on the calling resource (e.g. a Layer resource's span values, from the
/// layer's members).
///
/// Where Derive reads the resource's own tiles, an Aggregate folds across many,
/// typically through the datatype handlers' fold capability
/// (`crate::datatype_fold`). The same purity contract applies: the output depends
/// only on the slice's contents and the config, never on visit order.
///
/// It returns **node values**, not tiles: each names a tile (by an id the config
/// supplies) and a node on the calling resource's graph. Everything else in those
/// tiles is ordinary resource data, written by whoever owns the resource;
/// [`apply_aggregate_functions`] only sets the named node values.
pub trait AggregateProvider: Send + Sync {
    fn aggregate(
        &self,
        target_id: &str,
        graph: &dyn GraphLookup,
        slice: &dyn SliceSource,
        config: &AggregateConfig,
        context: &AggregateContext<'_>,
    ) -> Result<Vec<NodeValueWrite>, String>;
}

/// One node value an Aggregate writes: `data[node_id] = value` in the calling
/// resource's tile `tileid`.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeValueWrite {
    pub tileid: String,
    pub node_id: String,
    pub value: serde_json::Value,
}

/// Shared services an Aggregate may use. Built with [`AggregateContext::new`];
/// non-exhaustive so services can be added without breaking providers.
#[non_exhaustive]
pub struct AggregateContext<'a> {
    /// Datatype handlers, for fold dispatch (`crate::datatype_fold`).
    pub handlers: &'a ExtensionTypeRegistry,
    /// The graphs of the slice's members, to resolve paths per graph.
    pub graphs: &'a dyn GraphSource,
}

impl<'a> AggregateContext<'a> {
    pub fn new(handlers: &'a ExtensionTypeRegistry, graphs: &'a dyn GraphSource) -> Self {
        Self { handlers, graphs }
    }
}

/// An Aggregate function's declaration: the `config` of a `functions_x_graphs`
/// entry whose `function_id` is [`AGGREGATE_FUNCTION_ID`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregateConfig {
    /// The provider's UUID in the [`FunctionsRegistry`].
    pub provider: String,
    /// Provider-specific settings, opaque to core.
    #[serde(default)]
    pub config: serde_json::Value,
}

/// A registered graph-attached function, keyed in the registry by provider UUID.
///
/// A **closed enum**: each kind keeps its own trait and signature; the registry
/// unifies storage, population, and cross-FFI reach — not the call shapes. A new
/// kind is a new variant plus a new hook point, like the datatype list is a
/// closed set.
#[derive(Clone)]
pub enum RegisteredFunction {
    Descriptor(Arc<dyn DescriptorProvider>),
    Derive(Arc<dyn DeriveProvider>),
    Aggregate(Arc<dyn AggregateProvider>),
}

impl RegisteredFunction {
    pub fn kind(&self) -> &'static str {
        match self {
            RegisteredFunction::Descriptor(_) => "descriptor",
            RegisteredFunction::Derive(_) => "derive",
            RegisteredFunction::Aggregate(_) => "aggregate",
        }
    }
}

/// `provider_uuid → registered function`. The direct analogue of
/// [`ExtensionTypeRegistry`](crate::extension_type_registry) (`datatype →
/// handler`), for functions.
#[derive(Default, Clone)]
pub struct FunctionsRegistry {
    funcs: HashMap<String, RegisteredFunction>,
}

impl FunctionsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a function under its provider UUID. A provider crate calls this
    /// with the UUID(s) it supplies.
    pub fn register(&mut self, provider_uuid: impl Into<String>, func: RegisteredFunction) {
        self.funcs.insert(provider_uuid.into(), func);
    }

    pub fn unregister(&mut self, provider_uuid: &str) -> Option<RegisteredFunction> {
        self.funcs.remove(provider_uuid)
    }

    pub fn get(&self, provider_uuid: &str) -> Option<&RegisteredFunction> {
        self.funcs.get(provider_uuid)
    }

    /// The Derive provider registered under `provider_uuid`, if the entry exists
    /// AND is a Derive (not a Descriptor). Wrong-kind lookups return `None`.
    pub fn derive(&self, provider_uuid: &str) -> Option<&Arc<dyn DeriveProvider>> {
        match self.funcs.get(provider_uuid) {
            Some(RegisteredFunction::Derive(p)) => Some(p),
            _ => None,
        }
    }

    /// The Descriptor provider registered under `provider_uuid`, if present and a
    /// Descriptor.
    pub fn descriptor(&self, provider_uuid: &str) -> Option<&Arc<dyn DescriptorProvider>> {
        match self.funcs.get(provider_uuid) {
            Some(RegisteredFunction::Descriptor(p)) => Some(p),
            _ => None,
        }
    }

    /// The Aggregate provider registered under `provider_uuid`, if present and an
    /// Aggregate.
    pub fn aggregate(&self, provider_uuid: &str) -> Option<&Arc<dyn AggregateProvider>> {
        match self.funcs.get(provider_uuid) {
            Some(RegisteredFunction::Aggregate(p)) => Some(p),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.funcs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.funcs.is_empty()
    }
}

/// The default registry for a context: core's built-in providers, currently the
/// fold Aggregate ([`crate::fold_aggregate`], under
/// [`FOLD_AGGREGATE_PROVIDER_ID`](crate::fold_aggregate::FOLD_AGGREGATE_PROVIDER_ID)).
/// Provider crates (e.g. `alizarin-gramadan`) register their UUIDs on top,
/// exactly as `ros-madair-handlers::default_registry` seeds the datatype
/// registry. Each FFI binding builds one once (a `static`/`OnceLock`) and layers
/// providers in.
pub fn default_functions_registry() -> FunctionsRegistry {
    let mut registry = FunctionsRegistry::new();
    registry.register(
        crate::fold_aggregate::FOLD_AGGREGATE_PROVIDER_ID,
        RegisteredFunction::Aggregate(Arc::new(crate::fold_aggregate::FoldAggregate)),
    );
    registry
}

/// Run the graph's **Derive** functions against `tiles`, resolving each provider
/// from `registry` by its `provider` UUID and merging the results in.
///
/// - `is_member(layer_id)` is the fast first test (resource present in the
///   configured layer); each substrate provides an O(1) implementation.
/// - A nodegroup already present in `tiles` is NOT overwritten — attested data
///   wins over derived.
/// - Providers receive the CURRENT `tiles` (including any produced by earlier
///   derive functions in this pass), so derivations can chain.
/// - A `provider` UUID with no entry in this context's registry is skipped (not
///   an error) — a computed layer whose provider crate is absent simply yields
///   nothing.
pub fn apply_derive_functions(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    resource_id: &str,
    is_member: &dyn Fn(&str) -> bool,
    registry: &FunctionsRegistry,
) {
    for func in compute_tiles_functions(graph) {
        if !is_member(&func.member_of) {
            continue;
        }
        if tiles.iter().any(|t| t.nodegroup_id == func.nodegroup) {
            continue;
        }
        let provider = match registry.derive(&func.provider) {
            Some(p) => p.clone(),
            None => continue,
        };
        // The immutable borrow of `tiles` ends when `derive` returns (owned Vec),
        // so the subsequent `extend` does not overlap it.
        if let Ok(computed) = provider.derive(resource_id, graph, tiles, &func) {
            tiles.extend(computed);
        }
    }
}

/// The Aggregate functions a graph declares, in declaration order. Unlike the
/// compute-tiles reader, a malformed declaration is an error: Aggregates run on
/// an explicit call, and silently skipping one would leave its output stale.
pub fn aggregate_functions(graph: &dyn GraphLookup) -> Result<Vec<AggregateConfig>, String> {
    graph
        .functions_x_graphs()
        .into_iter()
        .filter(|f| f.function_id == AGGREGATE_FUNCTION_ID)
        .map(|f| {
            serde_json::from_value(f.config.clone())
                .map_err(|e| format!("aggregate function {}: invalid config: {e}", f.id))
        })
        .collect()
}

/// A slice with the calling resource left out: a Layer resource is not a member
/// of its own layer.
struct ExcludeTarget<'a> {
    inner: &'a dyn SliceSource,
    target_id: &'a str,
}

impl SliceSource for ExcludeTarget<'_> {
    fn visit(
        &self,
        f: &mut dyn FnMut(SliceMember<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        self.inner.visit(&mut |member| {
            if member.resource_id == self.target_id {
                Ok(())
            } else {
                f(member)
            }
        })
    }
}

/// Run the graph's **Aggregate** functions for `target_id` over `slice`, and
/// write their node values into `tiles` (the calling resource's tiles).
///
/// Called explicitly (e.g. when a layer is built), never on load: an Aggregate
/// is a pass over the whole slice.
///
/// Each write sets one node in one tile, and touches nothing else in it:
/// - **The tile exists:** its `node_id` value is set. The tile must be in the
///   node's nodegroup and belong to `target_id`.
/// - **The tile doesn't exist:** it is created with that id, the node's
///   nodegroup, and just that value. Only for a top-level nodegroup: a tile in
///   a nested nodegroup needs a parent tile, which only its owner can supply, so
///   it must be created first.
///
/// Also:
/// - The calling resource is left out of the slice.
/// - Re-running overwrites the same node values, so it is idempotent.
/// - All-or-nothing: every function runs and every write is checked before
///   `tiles` changes, so on `Err` (a malformed declaration, an unregistered
///   provider, a provider error, an unknown node, a tile in the wrong nodegroup
///   or resource, or two writes to one node of one tile) `tiles` is untouched.
pub fn apply_aggregate_functions(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    target_id: &str,
    slice: &dyn SliceSource,
    functions: &FunctionsRegistry,
    context: &AggregateContext<'_>,
) -> Result<(), String> {
    let configs = aggregate_functions(graph)?;
    let slice = ExcludeTarget {
        inner: slice,
        target_id,
    };

    let mut writes: Vec<NodeValueWrite> = Vec::new();
    for (index, config) in configs.iter().enumerate() {
        let provider = functions.aggregate(&config.provider).ok_or_else(|| {
            format!(
                "aggregate function {index}: no aggregate provider registered as {}",
                config.provider
            )
        })?;
        let produced = provider
            .aggregate(target_id, graph, &slice, config, context)
            .map_err(|e| format!("aggregate function {index} ({}): {e}", config.provider))?;
        writes.extend(produced);
    }

    // Check every write, and work out each one's nodegroup, before changing tiles.
    let mut written = std::collections::HashSet::new(); // (tileid, node_id)
    let mut new_tiles: HashMap<&str, &str> = HashMap::new(); // tileid -> nodegroup
    let mut planned: Vec<(&NodeValueWrite, &str)> = Vec::with_capacity(writes.len());
    for write in &writes {
        if !written.insert((write.tileid.as_str(), write.node_id.as_str())) {
            return Err(format!(
                "two aggregate writes to node {} of tile {}",
                write.node_id, write.tileid
            ));
        }
        let node = graph
            .get_node_by_id(&write.node_id)
            .ok_or_else(|| format!("aggregate write to unknown node {}", write.node_id))?;
        let nodegroup = node.nodegroup_id.as_deref().ok_or_else(|| {
            format!(
                "aggregate write to node {}, which has no nodegroup",
                node.nodeid
            )
        })?;
        match tiles
            .iter()
            .find(|t| t.tileid.as_deref() == Some(write.tileid.as_str()))
        {
            Some(tile) => {
                if tile.resourceinstance_id != target_id || tile.nodegroup_id != nodegroup {
                    return Err(format!(
                        "aggregate write to node {} of tile {}: the tile is in nodegroup {} on {}, \
                         the node in nodegroup {nodegroup} on {target_id}",
                        node.nodeid, write.tileid, tile.nodegroup_id, tile.resourceinstance_id
                    ));
                }
            }
            None => {
                if let Some(other) = new_tiles.insert(&write.tileid, nodegroup) {
                    if other != nodegroup {
                        return Err(format!(
                            "aggregate writes to new tile {} name nodes in nodegroups {other} and \
                             {nodegroup}",
                            write.tileid
                        ));
                    }
                }
                let nested = graph
                    .get_nodegroup_by_id(nodegroup)
                    .is_some_and(|ng| ng.parentnodegroup_id.is_some());
                if nested {
                    return Err(format!(
                        "aggregate write to node {} of tile {}: no such tile, and nodegroup \
                         {nodegroup} is nested, so the tile (with its parent) must be created first",
                        node.nodeid, write.tileid
                    ));
                }
            }
        }
        planned.push((write, nodegroup));
    }

    for (write, nodegroup) in planned {
        let index = match tiles
            .iter()
            .position(|t| t.tileid.as_deref() == Some(write.tileid.as_str()))
        {
            Some(index) => index,
            None => {
                tiles.push(StaticTile {
                    data: Default::default(),
                    nodegroup_id: nodegroup.to_string(),
                    resourceinstance_id: target_id.to_string(),
                    tileid: Some(write.tileid.clone()),
                    parenttile_id: None,
                    provisionaledits: None,
                    sortorder: None,
                });
                tiles.len() - 1
            }
        };
        tiles[index]
            .data
            .insert(write.node_id.clone(), write.value.clone());
    }
    Ok(())
}
