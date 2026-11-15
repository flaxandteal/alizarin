//! Derive providers across a binding boundary: one implementation of both
//! sides, as [`extension_abi`](crate::extension_abi) is for datatype handlers.
//!
//! - **Host side** ([`register_derive_provider_from_ptr`]): an extension's
//!   [`DeriveProviderInfo`] (from a NAPI pointer or a Python capsule) becomes a
//!   [`DeriveProvider`] in the global functions registry. Each call passes the
//!   resource's tiles and the declaration (its `nodes` already resolved) by
//!   pointer, plus [`HostGraph`]: lookups into the host's own graph, so no copy
//!   of the graph ever crosses.
//! - **Extension side** ([`derive_with`], [`CallbackGraph`]): what an
//!   extension's exported `derive_fn` calls, so its provider is written once
//!   against [`DeriveProvider`] and runs unchanged in-process or across the
//!   boundary. [`CallbackGraph`] is generic over the lookup, so a WASM
//!   extension can serve it from JS callbacks.
//!
//! [`CallbackGraph`] answers what a provider asks of a graph: its root, nodes
//! by alias or id, and nodegroups by id. Listings (edges, cards, every node)
//! come back empty: a provider that needs them must run in-process.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use alizarin_extension_api::{
    abi_fingerprint, DeriveFn, DeriveProviderInfo, DeriveResult, HostGraph,
};
/// The lookup kinds [`CallbackGraph`] and a host's lookup use, the same across
/// the C ABI and from JS (so a WASM binding needn't depend on the ABI crate).
pub use alizarin_extension_api::{
    LOOKUP_NODEGROUP_BY_ID, LOOKUP_NODE_BY_ALIAS, LOOKUP_NODE_BY_ID, LOOKUP_ROOT,
};
use serde_json::Value;

use crate::graph::card_index::CardIndex;
use crate::graph::{
    ComputeTilesConfig, DeriveProvider, FunctionsRegistry, GraphLookup, RegisteredFunction,
    StaticCard, StaticCardsXNodesXWidgets, StaticEdge, StaticNode, StaticNodegroup, StaticTile,
    StaticTranslatableString,
};

// =============================================================================
// Host side
// =============================================================================

/// What [`host_lookup`] answers from, and the values it has handed out (kept
/// alive until the derive call returns).
struct HostCtx<'a> {
    graph: &'a dyn GraphLookup,
    // Boxed: the extension holds pointers to these values, which must not
    // move when the Vec grows.
    #[allow(clippy::vec_box)]
    handed_out: RefCell<Vec<Box<Value>>>,
}

/// The host's [`HostGraph::lookup`]: one lookup in the graph, as a
/// `*const serde_json::Value` owned by the call's [`HostCtx`], or null.
///
/// # Safety
/// `ctx` is the call's `HostCtx`, and `key_ptr` is `key_len` bytes of UTF-8 (or
/// null with `key_len` 0).
unsafe extern "C" fn host_lookup(
    ctx: *const c_void,
    kind: u32,
    key_ptr: *const u8,
    key_len: usize,
) -> *const c_void {
    let ctx = &*(ctx as *const HostCtx<'_>);
    let key = if key_ptr.is_null() {
        ""
    } else {
        match std::str::from_utf8(std::slice::from_raw_parts(key_ptr, key_len)) {
            Ok(key) => key,
            Err(_) => return std::ptr::null(),
        }
    };
    let value = match kind {
        LOOKUP_ROOT => serde_json::to_value(ctx.graph.get_root()).ok(),
        LOOKUP_NODE_BY_ALIAS => ctx
            .graph
            .get_node_by_alias(key)
            .and_then(|n| serde_json::to_value(n).ok()),
        LOOKUP_NODE_BY_ID => ctx
            .graph
            .get_node_by_id(key)
            .and_then(|n| serde_json::to_value(n).ok()),
        LOOKUP_NODEGROUP_BY_ID => ctx
            .graph
            .get_nodegroup_by_id(key)
            .and_then(|ng| serde_json::to_value(ng).ok()),
        _ => None,
    };
    match value {
        Some(value) => {
            let value = Box::new(value);
            let ptr = &*value as *const Value as *const c_void;
            ctx.handed_out.borrow_mut().push(value);
            ptr
        }
        None => std::ptr::null(),
    }
}

/// A [`DeriveProvider`] in an extension, called through its exported functions.
struct AbiDeriveProvider {
    derive_fn: DeriveFn,
    free_fn: alizarin_extension_api::FreeDeriveFn,
}

impl DeriveProvider for AbiDeriveProvider {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let tiles = serde_json::to_value(tiles).map_err(|e| format!("serialize tiles: {e}"))?;
        let config = serde_json::to_value(config).map_err(|e| format!("serialize config: {e}"))?;
        let ctx = HostCtx {
            graph,
            handed_out: RefCell::new(Vec::new()),
        };
        let host = HostGraph {
            ctx: &ctx as *const HostCtx<'_> as *const c_void,
            lookup: host_lookup,
        };
        // SAFETY: the fn pointers come from an extension whose ABI fingerprint
        // was checked at registration; every pointer passed lives for the call.
        unsafe {
            let result = (self.derive_fn)(
                resource_id.as_ptr(),
                resource_id.len(),
                &tiles as *const Value as *const c_void,
                &config as *const Value as *const c_void,
                &host,
            );
            let outcome = read_result(&result);
            (self.free_fn)(result);
            outcome
        }
    }
}

/// The derived tiles in `result`, or its error.
///
/// # Safety
/// `result` came from a [`DeriveFn`] and hasn't been freed.
unsafe fn read_result(result: &DeriveResult) -> Result<Vec<StaticTile>, String> {
    if !result.error_ptr.is_null() {
        let bytes = std::slice::from_raw_parts(result.error_ptr, result.error_len);
        return Err(String::from_utf8_lossy(bytes).into_owned());
    }
    if result.json_ptr.is_null() {
        return Ok(Vec::new());
    }
    let bytes = std::slice::from_raw_parts(result.json_ptr, result.json_len);
    serde_json::from_slice(bytes).map_err(|e| format!("derived tiles: {e}"))
}

/// Register an extension's derive provider from its [`DeriveProviderInfo`]
/// pointer into the global functions registry, under its provider id, which
/// is returned. What `registerDeriveProvider` does on NAPI and Python.
///
/// # Safety
/// `ptr` is null or points to a `DeriveProviderInfo` (and its id bytes and fn
/// pointers) that outlives the process's use of it, as an extension's static
/// info does. The ABI fingerprint is checked before anything else is read.
pub unsafe fn register_derive_provider_from_ptr(
    ptr: *const DeriveProviderInfo,
) -> Result<String, String> {
    if ptr.is_null() {
        return Err("null DeriveProviderInfo pointer".to_string());
    }
    let info = &*ptr;
    if info.abi != abi_fingerprint() {
        return Err(format!(
            "extension ABI mismatch: built against {:?}, host is {:?}: rebuild the extension \
             against this alizarin version",
            info.abi,
            abi_fingerprint()
        ));
    }
    let provider_id = std::str::from_utf8(std::slice::from_raw_parts(
        info.provider_id_ptr,
        info.provider_id_len,
    ))
    .map_err(|e| format!("provider id is not valid UTF-8: {e}"))?
    .to_string();
    let provider = AbiDeriveProvider {
        derive_fn: info.derive_fn,
        free_fn: info.free_fn,
    };
    crate::with_global_functions_registry_mut(|registry: &mut FunctionsRegistry| {
        registry.register(
            provider_id.clone(),
            RegisteredFunction::Derive(Arc::new(provider)),
        )
    })
    .ok_or("the global functions registry is unavailable")?;
    Ok(provider_id)
}

// =============================================================================
// Extension side
// =============================================================================

// Every Item is boxed (see `CallbackGraph::items`), so the size gap costs nothing.
#[allow(clippy::large_enum_variant)]
enum Item {
    Node(StaticNode),
    Nodegroup(StaticNodegroup),
}

/// A [`GraphLookup`] over a lookup function (`lookup(kind, key)`, kinds as
/// [`HostGraph`]'s, returning the item's JSON): the host's graph as an
/// extension's provider sees it. Each item is looked up once and kept, so the
/// references it hands out live as long as it does.
///
/// Answers root, nodes by alias or id, and nodegroups by id; listings (edges,
/// cards, every node) are empty.
pub struct CallbackGraph<L: Fn(u32, &str) -> Option<Value>> {
    lookup: L,
    root: StaticNode,
    graph_id: String,
    name: StaticTranslatableString,
    // Boxed: references handed out point into these items, which must not
    // move when the Vec grows.
    #[allow(clippy::vec_box)]
    items: RefCell<Vec<Box<Item>>>,
    found: RefCell<HashMap<(u32, String), Option<usize>>>,
}

impl<L: Fn(u32, &str) -> Option<Value>> CallbackGraph<L> {
    /// The graph `lookup` answers for: an error if it has no root.
    pub fn new(lookup: L) -> Result<Self, String> {
        let root: StaticNode = lookup(LOOKUP_ROOT, "")
            .ok_or("the host graph has no root")
            .and_then(|v| {
                serde_json::from_value(v).map_err(|_| "the host graph's root is invalid")
            })?;
        Ok(CallbackGraph {
            graph_id: root.graph_id.clone(),
            root,
            lookup,
            name: StaticTranslatableString::default(),
            items: RefCell::new(Vec::new()),
            found: RefCell::new(HashMap::new()),
        })
    }

    fn item(&self, kind: u32, key: &str) -> Option<&Item> {
        let cached = self.found.borrow().get(&(kind, key.to_string())).copied();
        let index = match cached {
            Some(index) => index,
            None => {
                let item = (self.lookup)(kind, key).and_then(|value| match kind {
                    LOOKUP_NODEGROUP_BY_ID => {
                        serde_json::from_value(value).ok().map(Item::Nodegroup)
                    }
                    _ => serde_json::from_value(value).ok().map(Item::Node),
                });
                let index = item.map(|item| {
                    let mut items = self.items.borrow_mut();
                    items.push(Box::new(item));
                    items.len() - 1
                });
                self.found
                    .borrow_mut()
                    .insert((kind, key.to_string()), index);
                index
            }
        }?;
        let item: *const Item = &*self.items.borrow()[index];
        // SAFETY: items are boxed and never removed while `self` lives, so the
        // box's contents stay put however the Vec grows.
        Some(unsafe { &*item })
    }

    fn node(&self, kind: u32, key: &str) -> Option<&StaticNode> {
        match self.item(kind, key)? {
            Item::Node(node) => Some(node),
            Item::Nodegroup(_) => None,
        }
    }
}

impl<L: Fn(u32, &str) -> Option<Value>> GraphLookup for CallbackGraph<L> {
    fn graph_id(&self) -> &str {
        &self.graph_id
    }
    fn display_name(&self) -> String {
        String::new()
    }
    fn get_model_class_name(&self) -> Option<String> {
        None
    }
    fn name(&self) -> &StaticTranslatableString {
        &self.name
    }
    fn get_root(&self) -> &StaticNode {
        &self.root
    }
    fn get_node_by_id(&self, id: &str) -> Option<&StaticNode> {
        self.node(LOOKUP_NODE_BY_ID, id)
    }
    fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        self.node(LOOKUP_NODE_BY_ALIAS, alias)
    }
    fn find_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        self.get_node_by_alias(alias)
    }
    fn get_node_arc_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>> {
        self.get_node_by_alias(alias).cloned().map(Arc::new)
    }
    fn nodes_by_alias_arc(&self) -> Option<&HashMap<String, Arc<StaticNode>>> {
        None
    }
    fn get_nodegroup_by_id(&self, nodegroup_id: &str) -> Option<&StaticNodegroup> {
        match self.item(LOOKUP_NODEGROUP_BY_ID, nodegroup_id)? {
            Item::Nodegroup(nodegroup) => Some(nodegroup),
            Item::Node(_) => None,
        }
    }
    fn get_nodes_in_nodegroup(&self, _nodegroup_id: &str) -> Vec<&StaticNode> {
        Vec::new()
    }
    fn edges_map(&self) -> Option<&HashMap<String, Vec<String>>> {
        None
    }
    fn get_child_ids(&self, _node_id: &str) -> Option<&Vec<String>> {
        None
    }
    fn find_card_by_nodegroup(&self, _nodegroup_id: &str) -> Option<&StaticCard> {
        None
    }
    fn cards_slice(&self) -> &[StaticCard] {
        &[]
    }
    fn card_index(&self) -> Option<&CardIndex> {
        None
    }
    fn cards_x_nodes_x_widgets_slice(&self) -> &[StaticCardsXNodesXWidgets] {
        &[]
    }
    fn nodes_slice(&self) -> &[StaticNode] {
        &[]
    }
    fn nodegroups_slice(&self) -> &[StaticNodegroup] {
        &[]
    }
    fn edges_slice(&self) -> &[StaticEdge] {
        &[]
    }
    fn nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<usize>>> {
        None
    }
}

/// Run `provider` for one [`DeriveFn`] call: what an extension's exported
/// `derive_fn` does with its arguments. A provider error or panic comes back
/// as the result's error.
///
/// # Safety
/// The arguments are a [`DeriveFn`]'s, from the host: live for the call.
pub unsafe fn derive_with<P: DeriveProvider + ?Sized>(
    provider: &P,
    resource_id_ptr: *const u8,
    resource_id_len: usize,
    tiles_ptr: *const c_void,
    config_ptr: *const c_void,
    graph: *const HostGraph,
) -> DeriveResult {
    let run = || -> Result<Vec<u8>, String> {
        let resource_id =
            std::str::from_utf8(std::slice::from_raw_parts(resource_id_ptr, resource_id_len))
                .map_err(|e| format!("resource id: {e}"))?;
        let tiles: Vec<StaticTile> = serde_json::from_value((*(tiles_ptr as *const Value)).clone())
            .map_err(|e| format!("tiles: {e}"))?;
        let config: ComputeTilesConfig =
            serde_json::from_value((*(config_ptr as *const Value)).clone())
                .map_err(|e| format!("config: {e}"))?;
        let host = &*graph;
        let lookup = |kind: u32, key: &str| {
            // SAFETY: the host's lookup, with its own context, for this call.
            let ptr = (host.lookup)(host.ctx, kind, key.as_ptr(), key.len());
            (!ptr.is_null()).then(|| (*(ptr as *const Value)).clone())
        };
        let graph = CallbackGraph::new(lookup)?;
        let derived = provider.derive(resource_id, &graph, &tiles, &config)?;
        serde_json::to_vec(&derived).map_err(|e| format!("serialize tiles: {e}"))
    };
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(Ok(json)) => DeriveResult::success(json),
        Ok(Err(error)) => DeriveResult::error(error),
        Err(_) => DeriveResult::error("the derive provider panicked".to_string()),
    }
}

/// Free a [`DeriveResult`] made by [`derive_with`]: an extension's `free_fn`.
///
/// # Safety
/// `result` came from [`derive_with`] in this same build, and isn't used again.
pub unsafe extern "C" fn free_derive_result(result: DeriveResult) {
    if !result.json_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.json_ptr,
            result.json_len,
        ));
    }
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

/// The [`DeriveProviderInfo`] an extension exports for `derive_fn` (which
/// calls [`derive_with`]) under `provider_id`.
pub fn derive_provider_info(provider_id: &'static str, derive_fn: DeriveFn) -> DeriveProviderInfo {
    DeriveProviderInfo {
        abi: abi_fingerprint(),
        provider_id_ptr: provider_id.as_ptr(),
        provider_id_len: provider_id.len(),
        derive_fn,
        free_fn: free_derive_result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    use serde_json::json;

    const PROVIDER: &str = "derive-abi-test-label";

    /// `label` = the node named by role "source" upper-cased, suffixed with
    /// the alias the provider looks up itself (from its own config).
    struct Label;
    impl DeriveProvider for Label {
        fn derive(
            &self,
            resource_id: &str,
            graph: &dyn GraphLookup,
            tiles: &[StaticTile],
            config: &ComputeTilesConfig,
        ) -> Result<Vec<StaticTile>, String> {
            let source = config.node("source")?;
            let value = tiles
                .iter()
                .find_map(|t| t.data.get(&source.nodeid).and_then(Value::as_str))
                .ok_or("no source value")?;
            // A lookup through the handoff, not the declaration.
            let output = graph
                .get_node_by_alias("label")
                .ok_or("no label node in the host graph")?;
            let mut tile = StaticTile::new_empty(config.nodegroup.clone());
            tile.resourceinstance_id = resource_id.to_string();
            tile.data
                .insert(output.nodeid.clone(), json!(value.to_uppercase()));
            Ok(vec![tile])
        }
    }

    unsafe extern "C" fn label_derive(
        resource_id_ptr: *const u8,
        resource_id_len: usize,
        tiles_ptr: *const c_void,
        config_ptr: *const c_void,
        graph: *const HostGraph,
    ) -> DeriveResult {
        derive_with(
            &Label,
            resource_id_ptr,
            resource_id_len,
            tiles_ptr,
            config_ptr,
            graph,
        )
    }

    fn info() -> &'static DeriveProviderInfo {
        static INFO: OnceLock<DeriveProviderInfo> = OnceLock::new();
        INFO.get_or_init(|| derive_provider_info(PROVIDER, label_derive))
    }

    fn graph() -> crate::StaticGraph {
        let c = "http://www.cidoc-crm.org/cidoc-crm/";
        let header = "parent_alias,alias,name,datatype,cardinality,ontology_class,\
                      parent_property,description,collection_name,required,searchable,\
                      exportable,sortorder";
        let rows = [
            ",name,non-localized-string,1",
            ",label,non-localized-string,1",
        ]
        .iter()
        .map(|r| {
            let f: Vec<&str> = r.split(',').collect();
            format!(
                "{},{},{},{},{},{c}E1,{c}P2,,,,,,",
                f[0], f[1], f[1], f[2], f[3]
            )
        })
        .collect::<Vec<_>>();
        let (mut graph, _) = crate::build_graph_from_model_csvs(
            &format!("name,ontology_class,author,description,is_resource\nThing,{c}E1,,,true"),
            &format!("{header}\n{}", rows.join("\n")),
            None,
            "https://example.org/test",
            Default::default(),
        )
        .expect("graph");
        let label_ng = graph
            .get_node_by_alias("label")
            .and_then(|n| n.nodegroup_id.clone())
            .unwrap();
        let name_ng = graph
            .get_node_by_alias("name")
            .and_then(|n| n.nodegroup_id.clone())
            .unwrap();
        graph.functions_x_graphs = Some(
            serde_json::from_value(json!([{
                "id": "fxg-label",
                "function_id": crate::COMPUTE_TILES_FUNCTION_ID,
                "graph_id": graph.graphid,
                "config": {
                    "provider": PROVIDER,
                    "nodegroup": label_ng,
                    "mode": "stored",
                    "sources": [name_ng],
                    "nodes": {"source": "name"},
                }
            }]))
            .unwrap(),
        );
        graph
    }

    #[test]
    fn a_provider_registered_through_the_abi_derives_with_resolved_nodes_and_lookups() {
        let id = unsafe { register_derive_provider_from_ptr(info()) }.expect("register");
        assert_eq!(id, PROVIDER);

        let graph = graph();
        let name = graph.get_node_by_alias("name").unwrap();
        let mut tile = StaticTile::new_empty(name.nodegroup_id.clone().unwrap());
        tile.tileid = Some("t1".to_string());
        tile.resourceinstance_id = "r1".to_string();
        tile.data.insert(name.nodeid.clone(), json!("abc"));
        let mut tiles = vec![tile];

        let changes = crate::apply_stored_derive_functions(
            &mut tiles,
            &graph,
            "r1",
            None,
            &crate::get_global_functions_registry(),
        )
        .expect("derive");
        assert_eq!(changes.upserted.len(), 1);
        let label = graph.get_node_by_alias("label").unwrap();
        assert_eq!(changes.upserted[0].data[&label.nodeid], json!("ABC"));
    }

    #[test]
    fn a_provider_error_crosses_back_as_an_error() {
        unsafe { register_derive_provider_from_ptr(info()) }.expect("register");
        let graph = graph();
        // No source value: the provider errors, and the stored pass with it.
        let mut tiles = Vec::new();
        let err = crate::apply_stored_derive_functions(
            &mut tiles,
            &graph,
            "r1",
            None,
            &crate::get_global_functions_registry(),
        )
        .unwrap_err();
        assert!(err.contains("no source value"), "{err}");
    }

    #[test]
    fn an_abi_mismatch_is_refused() {
        let mut bad = derive_provider_info(PROVIDER, label_derive);
        bad.abi.version += 1;
        let err = unsafe { register_derive_provider_from_ptr(&bad) }.unwrap_err();
        assert!(err.contains("ABI mismatch"), "{err}");
    }

    #[test]
    fn a_declared_node_the_graph_lacks_is_an_error_in_stored_mode() {
        let mut graph = graph();
        let mut fxgs = graph.functions_x_graphs.clone().unwrap();
        fxgs[0].config["nodes"] = json!({"source": "missing"});
        graph.functions_x_graphs = Some(fxgs);
        let err = crate::stored_derive_functions(&graph).unwrap_err();
        assert!(err.contains("'missing'"), "{err}");
    }
}
