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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::computed_tiles::{compute_tiles_functions, ComputeTilesConfig, DeriveMode};
use super::descriptors::{AGGREGATE_FUNCTION_ID, COMPUTE_TILES_FUNCTION_ID};
use super::graph_lookup::GraphLookup;
use super::tile::{tile_namespace, StaticTile};
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

    /// Expand this Aggregate's node values with the `added` members only, without
    /// revisiting earlier ones, by merging them into the totals already stored in
    /// `tiles` (the calling resource's current tiles). Returns the node values to
    /// write, as [`aggregate`](Self::aggregate) does.
    ///
    /// `None` (the default) means this provider can't, and the caller must run
    /// [`aggregate`](Self::aggregate) over the whole slice instead.
    fn add(
        &self,
        _target_id: &str,
        _graph: &dyn GraphLookup,
        _added: &dyn SliceSource,
        _tiles: &[StaticTile],
        _config: &AggregateConfig,
        _context: &AggregateContext<'_>,
    ) -> Option<Result<Vec<NodeValueWrite>, String>> {
        None
    }
}

/// One node value an Aggregate writes: `data[node_id] = value` in the calling
/// resource's tile `tileid`. Serialises as `{tileid, nodeid, value}`, Arches'
/// own names, for the bindings that return the writes that changed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeValueWrite {
    pub tileid: String,
    #[serde(rename = "nodeid")]
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
    /// Optional: a `number` node on the calling resource counting members added
    /// since its node values were last brought up to date, so a consumer can
    /// tell they're stale, and by how much, without tracking it elsewhere.
    /// [`mark_aggregate_pending`] adds to it; every run and addition resets it
    /// to 0. Several Aggregates may share one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<PendingNode>,
}

/// Where an Aggregate's pending count lives: node `alias` in tile `tileid` of
/// the calling resource (created, if missing, as Aggregate writes are).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingNode {
    pub tileid: String,
    pub alias: String,
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

/// Run the graph's on-load **Derive** functions against `tiles`, resolving each
/// provider from `registry` by its `provider` UUID and merging the results in.
/// Stored declarations are skipped: their tiles are already in `tiles`
/// ([`apply_stored_derive_functions`]).
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
        if func.mode != DeriveMode::OnLoad || !is_member(&func.member_of) {
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

/// The stored-mode Derive declarations a graph makes, in declaration order.
/// Strict, unlike the on-load reader: a stored function runs on writes, and
/// silently skipping a malformed one would leave its stored tiles stale. A
/// declaration is stored-mode if its config says `"mode": "stored"`; each
/// must name its `sources`.
pub fn stored_derive_functions(graph: &dyn GraphLookup) -> Result<Vec<ComputeTilesConfig>, String> {
    let mut configs = Vec::new();
    for f in graph
        .functions_x_graphs()
        .into_iter()
        .filter(|f| f.function_id == COMPUTE_TILES_FUNCTION_ID)
    {
        if f.config.get("mode").and_then(|m| m.as_str()) != Some("stored") {
            continue;
        }
        let config: ComputeTilesConfig = serde_json::from_value(f.config.clone())
            .map_err(|e| format!("derive function {}: invalid config: {e}", f.id))?;
        if config.sources.is_empty() {
            return Err(format!(
                "derive function {}: stored mode needs `sources`, the nodegroups whose \
                 changes rerun it",
                f.id
            ));
        }
        configs.push(config);
    }
    Ok(configs)
}

/// The id of the `k`th tile a stored Derive function (`provider`) writes into
/// `nodegroup_id`, under `parent_tileid` or, without one, at `resource_id`'s
/// root: `uuid5(TILE_NS, "{parent|resource}/{nodegroup}/derive/{provider}/{k}")`.
///
/// Predictable, so a rerun finds and replaces the function's own earlier tiles,
/// and a layer can address them. Deliberately not
/// [`canonical_tile_id`](super::canonical_tile_id), even for cardinality 1: tiles written by anyone
/// else carry that id, and the function must tell its own tiles from theirs.
pub fn derived_tile_id(
    resource_id: &str,
    nodegroup_id: &str,
    parent_tileid: Option<&str>,
    provider: &str,
    k: usize,
) -> String {
    let scope = parent_tileid.unwrap_or(resource_id);
    let name = format!("{scope}/{nodegroup_id}/derive/{provider}/{k}");
    Uuid::new_v5(&tile_namespace(), name.as_bytes()).to_string()
}

/// What [`apply_stored_derive_functions`] changed in a resource's tiles.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StoredDeriveChanges {
    /// Derived tiles that are new or changed, as they now stand.
    pub upserted: Vec<StaticTile>,
    /// Ids of tiles removed: a function's earlier tiles it no longer produces,
    /// and tiles it overwrote.
    pub removed: Vec<String>,
}

impl StoredDeriveChanges {
    pub fn is_empty(&self) -> bool {
        self.upserted.is_empty() && self.removed.is_empty()
    }
}

/// Run the graph's stored-mode **Derive** functions whose `sources` include a
/// nodegroup in `changed` (`None`: all of them, e.g. for a backfill), and
/// store their tiles in `tiles`.
///
/// Called by write paths after a resource's tiles change. Each function:
/// - Gets the current tiles, including earlier functions' output in this pass,
///   so derivations chain: a function's target counts as changed for the
///   functions after it.
/// - Has its output given [`derived_tile_id`]s, replacing every tile it wrote
///   before (so a function that now produces fewer tiles leaves none stale).
/// - In a cardinality-1 target, at most one tile per parent: another tile
///   already there is replaced if `overwrite` (the default), or else kept, and
///   the function's tile dropped. A cardinality-n target's other tiles are left
///   alone.
///
/// All-or-nothing: on `Err` (a malformed declaration, an unregistered provider,
/// a provider error, an unknown target nodegroup, a tile outside it, or two
/// tiles for one cardinality-1 slot) `tiles` is untouched.
///
/// Its own earlier tiles are recognised by id; tiles in the target are checked
/// for slots below the number of tiles there, so a derived tile someone removed
/// by hand may leave a higher-numbered sibling behind.
pub fn apply_stored_derive_functions(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    resource_id: &str,
    changed: Option<&[String]>,
    registry: &FunctionsRegistry,
) -> Result<StoredDeriveChanges, String> {
    let configs = stored_derive_functions(graph)?;
    let mut changed: Option<HashSet<String>> = changed.map(|c| c.iter().cloned().collect());
    let mut working = tiles.clone();
    let mut written: HashSet<String> = HashSet::new();
    let mut ran = false;
    for config in &configs {
        if let Some(changed) = &changed {
            if !config.sources.iter().any(|s| changed.contains(s)) {
                continue;
            }
        }
        let provider = registry.derive(&config.provider).ok_or_else(|| {
            format!(
                "stored derive function into {}: no derive provider registered as {}",
                config.nodegroup, config.provider
            )
        })?;
        let produced = provider
            .derive(resource_id, graph, &working, config)
            .map_err(|e| {
                format!(
                    "stored derive function {} into {}: {e}",
                    config.provider, config.nodegroup
                )
            })?;
        replace_derived(
            &mut working,
            graph,
            resource_id,
            config,
            produced,
            &mut written,
        )?;
        if let Some(changed) = changed.as_mut() {
            changed.insert(config.nodegroup.clone());
        }
        ran = true;
    }
    if !ran {
        return Ok(StoredDeriveChanges::default());
    }

    let before: HashMap<&str, &StaticTile> = tiles
        .iter()
        .filter_map(|t| t.tileid.as_deref().map(|id| (id, t)))
        .collect();
    let after: HashSet<&str> = working.iter().filter_map(|t| t.tileid.as_deref()).collect();
    let changes = StoredDeriveChanges {
        upserted: working
            .iter()
            .filter(|t| {
                t.tileid.as_deref().is_some_and(|id| {
                    written.contains(id) && before.get(id).is_none_or(|old| !same_tile(old, t))
                })
            })
            .cloned()
            .collect(),
        removed: tiles
            .iter()
            .filter_map(|t| t.tileid.as_deref())
            .filter(|id| !after.contains(id))
            .map(str::to_string)
            .collect(),
    };
    *tiles = working;
    Ok(changes)
}

/// Replace `config`'s earlier tiles in `working` with `produced`, as
/// [`apply_stored_derive_functions`] describes, recording the ids written.
fn replace_derived(
    working: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    resource_id: &str,
    config: &ComputeTilesConfig,
    produced: Vec<StaticTile>,
    written: &mut HashSet<String>,
) -> Result<(), String> {
    let ng = config.nodegroup.as_str();
    let nodegroup = graph.get_nodegroup_by_id(ng).ok_or_else(|| {
        format!(
            "stored derive function {}: no nodegroup {ng} on the graph",
            config.provider
        )
    })?;
    let single = nodegroup.cardinality.as_deref().is_none_or(|c| c != "n");
    if let Some(stray) = produced.iter().find(|t| t.nodegroup_id != ng) {
        return Err(format!(
            "stored derive function {} into {ng} produced a tile in nodegroup {}",
            config.provider, stray.nodegroup_id
        ));
    }

    // Remove the function's own earlier tiles.
    let slots = working.iter().filter(|t| t.nodegroup_id == ng).count();
    working.retain(|t| {
        let own = t.nodegroup_id == ng
            && t.tileid.as_deref().is_some_and(|id| {
                (0..slots).any(|k| {
                    derived_tile_id(
                        resource_id,
                        ng,
                        t.parenttile_id.as_deref(),
                        &config.provider,
                        k,
                    ) == id
                })
            });
        !own
    });

    // Number the new tiles per parent, and place them.
    let mut next: HashMap<Option<String>, usize> = HashMap::new();
    for mut tile in produced {
        let k = next.entry(tile.parenttile_id.clone()).or_insert(0);
        if single && *k > 0 {
            return Err(format!(
                "stored derive function {} produced two tiles for cardinality-1 nodegroup \
                 {ng} under one parent",
                config.provider
            ));
        }
        let id = derived_tile_id(
            resource_id,
            ng,
            tile.parenttile_id.as_deref(),
            &config.provider,
            *k,
        );
        *k += 1;
        if single {
            let same_slot =
                |w: &StaticTile| w.nodegroup_id == ng && w.parenttile_id == tile.parenttile_id;
            if working.iter().any(same_slot) {
                if !config.overwrite {
                    continue;
                }
                working.retain(|w| !same_slot(w));
            }
        }
        tile.tileid = Some(id.clone());
        tile.resourceinstance_id = resource_id.to_string();
        written.insert(id);
        working.push(tile);
    }
    Ok(())
}

/// Whether two tiles hold the same content (StaticTile has no `PartialEq`).
fn same_tile(a: &StaticTile, b: &StaticTile) -> bool {
    a.nodegroup_id == b.nodegroup_id
        && a.parenttile_id == b.parenttile_id
        && a.sortorder == b.sortorder
        && a.data == b.data
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
///
/// Each declared pending count ([`AggregateConfig::pending`]) is reset to 0 in
/// the same write: the node values are now up to date.
///
/// Returns the writes that changed a node value (all of a new tile's do), in
/// declaration order, so `[]` when a rerun finds nothing new. A consumer that
/// caches these node values can refresh exactly those.
pub fn apply_aggregate_functions(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    target_id: &str,
    slice: &dyn SliceSource,
    functions: &FunctionsRegistry,
    context: &AggregateContext<'_>,
) -> Result<Vec<NodeValueWrite>, String> {
    let slice = ExcludeTarget {
        inner: slice,
        target_id,
    };
    run_aggregates(tiles, graph, target_id, functions, |provider, config, _| {
        provider.aggregate(target_id, graph, &slice, config, context)
    })
}

/// Expand the graph's **Aggregate** functions' node values in `tiles` with the
/// `added` members only, without revisiting earlier ones: each provider merges
/// the additions into the totals already stored in `tiles`
/// ([`AggregateProvider::add`]).
///
/// An event response, like [`apply_aggregate_functions`]: call it when the node
/// values are about to be consumed, not on every insertion. Additions compose,
/// so the members added since the last call can be passed in one batch, with
/// the same result as adding them one at a time.
///
/// - Each member must be added once: a provider doesn't track membership, so
///   adding one again counts it again (extents are unaffected; counts aren't).
/// - Errors (and leaves `tiles` untouched) if any of the graph's Aggregate
///   providers can't add incrementally; run [`apply_aggregate_functions`] then.
/// - The calling resource is left out of `added`; otherwise as
///   [`apply_aggregate_functions`] (same checks, same writes, all-or-nothing).
pub fn apply_aggregate_additions(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    target_id: &str,
    added: &dyn SliceSource,
    functions: &FunctionsRegistry,
    context: &AggregateContext<'_>,
) -> Result<Vec<NodeValueWrite>, String> {
    let added = ExcludeTarget {
        inner: added,
        target_id,
    };
    run_aggregates(
        tiles,
        graph,
        target_id,
        functions,
        |provider, config, current| {
            provider
                .add(target_id, graph, &added, current, config, context)
                .unwrap_or_else(|| {
                    Err(
                        "can't add incrementally; run apply_aggregate_functions instead"
                            .to_string(),
                    )
                })
        },
    )
}

/// Collect every Aggregate function's node values (from `produce`, given the
/// provider, its config and the current tiles), check them, then write them.
fn run_aggregates(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    target_id: &str,
    functions: &FunctionsRegistry,
    mut produce: impl FnMut(
        &dyn AggregateProvider,
        &AggregateConfig,
        &[StaticTile],
    ) -> Result<Vec<NodeValueWrite>, String>,
) -> Result<Vec<NodeValueWrite>, String> {
    let configs = aggregate_functions(graph)?;
    let mut writes: Vec<NodeValueWrite> = Vec::new();
    for (index, config) in configs.iter().enumerate() {
        let provider = functions.aggregate(&config.provider).ok_or_else(|| {
            format!(
                "aggregate function {index}: no aggregate provider registered as {}",
                config.provider
            )
        })?;
        let produced = produce(provider.as_ref(), config, tiles)
            .map_err(|e| format!("aggregate function {index} ({}): {e}", config.provider))?;
        writes.extend(produced);
    }
    // Every declared pending count resets: the node values are now up to date.
    for (tileid, node_id) in pending_nodes(graph, &configs)? {
        writes.push(NodeValueWrite {
            tileid,
            node_id,
            value: serde_json::json!(0),
        });
    }
    write_checked(tiles, graph, target_id, &writes)
}

/// Check every write (as [`apply_aggregate_functions`] lists), then make them
/// all, returning those that changed a node value. On `Err`, `tiles` is
/// untouched.
fn write_checked(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    target_id: &str,
    writes: &[NodeValueWrite],
) -> Result<Vec<NodeValueWrite>, String> {
    // Check every write, and work out each one's nodegroup, before changing tiles.
    let mut written = std::collections::HashSet::new(); // (tileid, node_id)
    let mut new_tiles: HashMap<&str, &str> = HashMap::new(); // tileid -> nodegroup
    let mut planned: Vec<(&NodeValueWrite, &str)> = Vec::with_capacity(writes.len());
    for write in writes {
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

    // Write, keeping the writes that changed a node value.
    let mut changed = Vec::new();
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
        let data = &mut tiles[index].data;
        if data.get(&write.node_id) != Some(&write.value) {
            data.insert(write.node_id.clone(), write.value.clone());
            changed.push(write.clone());
        }
    }
    Ok(changed)
}

/// The pending-count nodes the Aggregates in `configs` declare, as
/// `(tileid, node id)`, each once however many share it.
fn pending_nodes(
    graph: &dyn GraphLookup,
    configs: &[AggregateConfig],
) -> Result<Vec<(String, String)>, String> {
    let mut nodes: Vec<(String, String)> = Vec::new();
    for pending in configs.iter().filter_map(|c| c.pending.as_ref()) {
        let node = graph.get_node_by_alias(&pending.alias).ok_or_else(|| {
            format!(
                "pending: no node with alias {} on the calling graph",
                pending.alias
            )
        })?;
        if node.datatype != "number" {
            return Err(format!(
                "pending: node {} is {}, not number",
                pending.alias, node.datatype
            ));
        }
        let key = (pending.tileid.clone(), node.nodeid.clone());
        if !nodes.contains(&key) {
            nodes.push(key);
        }
    }
    Ok(nodes)
}

/// Add `count` to each pending count the graph's Aggregates declare
/// ([`AggregateConfig::pending`]) in `tiles`: call it as members are added
/// without applying them, so a consumer can tell the node values are stale,
/// and by how much, before an event brings them up to date (which resets the
/// counts). A missing or null count starts from 0.
///
/// Errors if no Aggregate declares a pending node. Returns the writes that
/// changed, all-or-nothing, as [`apply_aggregate_functions`] does.
pub fn mark_aggregate_pending(
    tiles: &mut Vec<StaticTile>,
    graph: &dyn GraphLookup,
    target_id: &str,
    count: u64,
) -> Result<Vec<NodeValueWrite>, String> {
    let nodes = pending_nodes(graph, &aggregate_functions(graph)?)?;
    if nodes.is_empty() {
        return Err("no Aggregate function on this graph declares a pending node".to_string());
    }
    let writes: Vec<NodeValueWrite> = nodes
        .into_iter()
        .map(|(tileid, node_id)| {
            let current = tiles
                .iter()
                .find(|t| t.tileid.as_deref() == Some(tileid.as_str()))
                .and_then(|t| t.data.get(&node_id))
                .and_then(serde_json::Value::as_f64)
                .map_or(0, |n| n as u64);
            NodeValueWrite {
                tileid,
                node_id,
                value: serde_json::json!(current.saturating_add(count)),
            }
        })
        .collect();
    write_checked(tiles, graph, target_id, &writes)
}

/// Run `resource`'s Aggregate functions over `slice` with the process-wide
/// registries: its graph and the members' graphs from the registered graphs, and
/// providers from the global functions registry (core's built-ins plus any a
/// provider crate registered). The entry point the bindings use. Returns the
/// writes that changed, as [`apply_aggregate_functions`] does.
pub fn apply_aggregate_functions_to_resource(
    resource: &mut super::StaticResource,
    slice: &dyn SliceSource,
    handlers: &ExtensionTypeRegistry,
) -> Result<Vec<NodeValueWrite>, String> {
    with_process_registries(
        resource,
        handlers,
        |tiles, graph, target_id, functions, context| {
            apply_aggregate_functions(tiles, graph, target_id, slice, functions, context)
        },
    )
}

/// [`apply_aggregate_additions`] for `resource` with the process-wide registries,
/// as [`apply_aggregate_functions_to_resource`] does for a full run.
pub fn apply_aggregate_additions_to_resource(
    resource: &mut super::StaticResource,
    added: &dyn SliceSource,
    handlers: &ExtensionTypeRegistry,
) -> Result<Vec<NodeValueWrite>, String> {
    with_process_registries(
        resource,
        handlers,
        |tiles, graph, target_id, functions, context| {
            apply_aggregate_additions(tiles, graph, target_id, added, functions, context)
        },
    )
}

/// Run `f` over `resource`'s tiles with its registered graph, the global
/// functions registry and the registered member graphs; store the tiles back
/// only if `f` succeeds.
fn with_process_registries(
    resource: &mut super::StaticResource,
    handlers: &ExtensionTypeRegistry,
    f: impl FnOnce(
        &mut Vec<StaticTile>,
        &dyn GraphLookup,
        &str,
        &FunctionsRegistry,
        &AggregateContext<'_>,
    ) -> Result<Vec<NodeValueWrite>, String>,
) -> Result<Vec<NodeValueWrite>, String> {
    let graph_id = &resource.resourceinstance.graph_id;
    let graph =
        crate::get_graph(graph_id).ok_or_else(|| format!("graph {graph_id} is not registered"))?;
    let target_id = resource.resourceinstance.resourceinstanceid.clone();
    let functions = crate::get_global_functions_registry();
    let graphs = crate::slice_source::RegisteredGraphs;
    let context = AggregateContext::new(handlers, &graphs);
    let mut tiles = resource.tiles.clone().unwrap_or_default();
    let changed = f(&mut tiles, graph.as_ref(), &target_id, &functions, &context)?;
    resource.tiles = Some(tiles);
    Ok(changed)
}

/// The members of a registry named by id, as a slice. Every id must be loaded
/// in full: a missing one is an error, not silently skipped.
struct RegistryMembers<'a> {
    registry: &'a super::StaticResourceRegistry,
    ids: &'a [String],
}

impl SliceSource for RegistryMembers<'_> {
    fn visit(
        &self,
        f: &mut dyn FnMut(SliceMember<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        for id in self.ids {
            let resource = self
                .registry
                .get_full(id)
                .ok_or_else(|| format!("resource {id} is not loaded in full"))?;
            f(SliceMember {
                graph_id: &resource.resourceinstance.graph_id,
                resource_id: &resource.resourceinstance.resourceinstanceid,
                tiles: resource.tiles.as_deref().unwrap_or(&[]),
            })?;
        }
        Ok(())
    }
}

/// [`apply_aggregate_additions`] for a resource held in full in `registry`: the
/// members `added_ids` (already merged into the registry) are folded into its
/// stored totals, and the registry entry is updated in place. The event-response
/// form for a loop that adds members over time: pass the ids added since the
/// last call.
pub fn apply_aggregate_additions_in_registry(
    registry: &mut super::StaticResourceRegistry,
    target_id: &str,
    added_ids: &[String],
    handlers: &ExtensionTypeRegistry,
) -> Result<Vec<NodeValueWrite>, String> {
    let mut resource = registry
        .get_full(target_id)
        .cloned()
        .ok_or_else(|| format!("resource {target_id} is not loaded in full"))?;
    let added = RegistryMembers {
        registry,
        ids: added_ids,
    };
    let changed = apply_aggregate_additions_to_resource(&mut resource, &added, handlers)?;
    if !changed.is_empty() {
        registry.insert_full(resource);
    }
    Ok(changed)
}

/// [`apply_aggregate_functions_to_resource`] for a resource held in full in
/// `registry`, with the registry as the slice (the resource itself is left out).
/// The registry entry is updated in place.
pub fn apply_aggregate_functions_in_registry(
    registry: &mut super::StaticResourceRegistry,
    target_id: &str,
    handlers: &ExtensionTypeRegistry,
) -> Result<Vec<NodeValueWrite>, String> {
    let mut resource = registry
        .get_full(target_id)
        .cloned()
        .ok_or_else(|| format!("resource {target_id} is not loaded in full"))?;
    let changed = apply_aggregate_functions_to_resource(&mut resource, &*registry, handlers)?;
    if !changed.is_empty() {
        registry.insert_full(resource);
    }
    Ok(changed)
}

/// [`mark_aggregate_pending`] for a resource held in full in `registry`, with
/// its registered graph; the registry entry is updated in place. For a loop
/// that adds members over time: mark as it adds, apply on the consumer event.
pub fn mark_aggregate_pending_in_registry(
    registry: &mut super::StaticResourceRegistry,
    target_id: &str,
    count: u64,
) -> Result<Vec<NodeValueWrite>, String> {
    let mut resource = registry
        .get_full(target_id)
        .cloned()
        .ok_or_else(|| format!("resource {target_id} is not loaded in full"))?;
    let graph_id = resource.resourceinstance.graph_id.clone();
    let graph =
        crate::get_graph(&graph_id).ok_or_else(|| format!("graph {graph_id} is not registered"))?;
    let mut tiles = resource.tiles.take().unwrap_or_default();
    let changed = mark_aggregate_pending(&mut tiles, graph.as_ref(), target_id, count)?;
    if !changed.is_empty() {
        resource.tiles = Some(tiles);
        registry.insert_full(resource);
    }
    Ok(changed)
}
