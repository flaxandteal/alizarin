//! Derive providers in an extension: the declaration a provider is given,
//! the graph lookups it can make, and the extension side of the C ABI
//! ([`derive_with`], [`derive_provider_info`]).
//!
//! A provider here implements [`ExtDeriveProvider`] and needs no
//! alizarin-core: a copy of core in the extension would have its own, empty,
//! globals (language, registries). The host owns all of that, so it resolves
//! the declaration's nodes before the call, answers lookups into its own
//! graph, and **coerces the derived values** to each node's datatype after
//! it: a provider returns raw values (a number, a plain string), never
//! datatype-shaped ones.
//!
//! alizarin-core re-exports [`DeriveConfig`], [`DeriveMode`] and
//! [`ResolvedNode`] from `graph`, and runs an [`ExtDeriveProvider`] in-process
//! through its `derive_abi::InProcess` adaptor.

use std::collections::BTreeMap;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::graph::{StaticNode, StaticNodegroup, StaticTile};
use crate::{
    abi_fingerprint, DeriveFn, DeriveProviderInfo, DeriveResult, HostGraph, LOOKUP_NODEGROUP_BY_ID,
    LOOKUP_NODE_BY_ALIAS, LOOKUP_NODE_BY_ID, LOOKUP_ROOT,
};

/// When a Derive declaration runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeriveMode {
    /// Computed when the resource is loaded, and never stored: readers need the
    /// provider (e.g. Gramadán's forms in a computed layer).
    #[default]
    OnLoad,
    /// Computed when one of its `sources` nodegroups changes, and stored with
    /// the resource's data, so readers need no provider and a static index can
    /// search it (e.g. UTM). Run by alizarin-core's
    /// `apply_stored_derive_functions`.
    Stored,
}

fn default_overwrite() -> bool {
    true
}

/// Configuration for a compute-tiles function, deserialized from the
/// `config` field of a `functions_x_graphs` entry whose `function_id`
/// is alizarin-core's `COMPUTE_TILES_FUNCTION_ID`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeriveConfig {
    /// Provider dispatch key (e.g. "gramadan"). A JS-registered callback
    /// keyed by this name generates the tiles.
    pub provider: String,
    /// Which nodegroup this function fills.
    pub nodegroup: String,
    /// On-load only: layer ID for the fast first test: is the resource present
    /// in this layer? Usually the computed layer's own ID ("am I a member
    /// here?"). Stored declarations ignore it.
    #[serde(default)]
    pub member_of: String,
    /// When it runs; must be given as `"stored"` for stored mode.
    #[serde(default)]
    pub mode: DeriveMode,
    /// Stored only, and required there: the nodegroups whose changes rerun it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    /// Stored only: in a cardinality-1 target, whether its tile replaces a tile
    /// someone else put there (`true`, the default) or gives way to it. Its own
    /// earlier tiles are always replaced, and in a cardinality-n target other
    /// tiles are always left alone.
    #[serde(default = "default_overwrite")]
    pub overwrite: bool,
    #[serde(default)]
    pub cache: bool,
    /// The nodes the function reads and writes, by role (e.g. "geometry",
    /// "easting"), each named by alias or node id, as an Arches function's
    /// config names its input and output nodes. Resolved against the graph
    /// before the provider runs, into [`resolved`](Self::resolved).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub nodes: BTreeMap<String, String>,
    /// [`nodes`](Self::nodes), resolved by the host: what a provider reads
    /// (see [`node`](Self::node)), in-process or across a binding boundary,
    /// so it needs no graph lookups of its own. Never declared.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub resolved: BTreeMap<String, ResolvedNode>,
}

/// A node a Derive function declared by role, resolved against the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedNode {
    pub nodeid: String,
    pub nodegroup_id: Option<String>,
    pub datatype: String,
    pub alias: Option<String>,
}

impl DeriveConfig {
    /// The node declared for `role`, resolved: an error naming the role if the
    /// declaration doesn't give one.
    pub fn node(&self, role: &str) -> Result<&ResolvedNode, String> {
        self.resolved.get(role).ok_or_else(|| {
            format!(
                "derive function {}: no node declared for role '{role}' (config \"nodes\")",
                self.provider
            )
        })
    }
}

/// The host's graph as a provider sees it: one lookup, by kind
/// (`LOOKUP_*`) and key, answered as the item's JSON. The typed helpers
/// deserialise it. A provider needing more of the graph than these kinds is
/// a reason to add a kind, not to give extensions the graph.
pub trait DeriveLookup {
    /// The item of `kind` under `key` (empty for the root), or `None`.
    fn lookup(&self, kind: u32, key: &str) -> Option<Value>;

    /// The graph's root node.
    fn root(&self) -> Option<StaticNode> {
        typed(self.lookup(LOOKUP_ROOT, ""))
    }

    /// The node with id `id`.
    fn node(&self, id: &str) -> Option<StaticNode> {
        typed(self.lookup(LOOKUP_NODE_BY_ID, id))
    }

    /// The node with alias `alias`.
    fn node_by_alias(&self, alias: &str) -> Option<StaticNode> {
        typed(self.lookup(LOOKUP_NODE_BY_ALIAS, alias))
    }

    /// The nodegroup with id `id`.
    fn nodegroup(&self, id: &str) -> Option<StaticNodegroup> {
        typed(self.lookup(LOOKUP_NODEGROUP_BY_ID, id))
    }
}

fn typed<T: serde::de::DeserializeOwned>(value: Option<Value>) -> Option<T> {
    serde_json::from_value(value?).ok()
}

/// Any `lookup(kind, key)` function is a [`DeriveLookup`], e.g. one over a
/// JS callback in a WASM extension.
impl<F: Fn(u32, &str) -> Option<Value>> DeriveLookup for F {
    fn lookup(&self, kind: u32, key: &str) -> Option<Value> {
        self(kind, key)
    }
}

/// A derive provider written against the extension API: runs across the C
/// ABI through [`derive_with`], from JS in a WASM extension, or in-process
/// through alizarin-core's `derive_abi::InProcess`.
///
/// Returns the derived tiles with **raw** values (see the module docs): the
/// host coerces each to its node's datatype, with its own language and
/// extension handlers.
pub trait ExtDeriveProvider: Send + Sync {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn DeriveLookup,
        tiles: &[StaticTile],
        config: &DeriveConfig,
    ) -> Result<Vec<StaticTile>, String>;
}

/// Run `provider` for one [`DeriveFn`] call: what an extension's exported
/// `derive_fn` does with its arguments. A provider error or panic comes back
/// as the result's error.
///
/// # Safety
/// The arguments are a [`DeriveFn`]'s, from the host: live for the call.
pub unsafe fn derive_with<P: ExtDeriveProvider + ?Sized>(
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
        let config: DeriveConfig = serde_json::from_value((*(config_ptr as *const Value)).clone())
            .map_err(|e| format!("config: {e}"))?;
        let host = &*graph;
        let lookup = |kind: u32, key: &str| {
            // SAFETY: the host's lookup, with its own context, for this call.
            let ptr = (host.lookup)(host.ctx, kind, key.as_ptr(), key.len());
            (!ptr.is_null()).then(|| (*(ptr as *const Value)).clone())
        };
        let derived = provider.derive(resource_id, &lookup, &tiles, &config)?;
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
