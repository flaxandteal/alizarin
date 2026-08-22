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

use super::computed_tiles::{compute_tiles_functions, ComputeTilesConfig};
use super::graph_lookup::GraphLookup;
use super::tile::StaticTile;

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
}

impl RegisteredFunction {
    pub fn kind(&self) -> &'static str {
        match self {
            RegisteredFunction::Descriptor(_) => "descriptor",
            RegisteredFunction::Derive(_) => "derive",
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

    pub fn len(&self) -> usize {
        self.funcs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.funcs.is_empty()
    }
}

/// The default registry for a context. alizarin-core ships it EMPTY — provider
/// crates (e.g. `alizarin-gramadan`) register their UUIDs on top, exactly as
/// `ros-madair-handlers::default_registry` seeds the datatype registry. Each FFI
/// binding builds one once (a `static`/`OnceLock`) and layers providers in.
pub fn default_functions_registry() -> FunctionsRegistry {
    FunctionsRegistry::new()
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
