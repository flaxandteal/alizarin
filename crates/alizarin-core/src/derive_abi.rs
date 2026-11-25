//! Derive providers across a binding boundary: the host side, as
//! [`extension_abi`](crate::extension_abi) is for datatype handlers.
//!
//! An extension's provider implements the extension API's
//! [`ExtDeriveProvider`], and its exported `derive_fn` calls the API's
//! `derive_with`: it links no core, whose globals it would get its own empty
//! copy of. Here:
//!
//! - [`register_derive_provider_from_ptr`]: an extension's
//!   [`DeriveProviderInfo`] (from a NAPI pointer or a Python capsule) becomes a
//!   [`DeriveProvider`] in the global functions registry. Each call passes the
//!   resource's tiles and the declaration (its `nodes` already resolved) by
//!   pointer, plus [`HostGraph`]: lookups into the host's own graph, so no copy
//!   of the graph ever crosses.
//! - [`InProcess`]: an [`ExtDeriveProvider`] registered in-process, e.g. by a
//!   Rust program linking the extension's core crate.
//! - [`lookup_json`]: the one answer to a lookup, for the C ABI, in-process,
//!   and WASM's JS providers alike.

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Arc;

use alizarin_extension_api::{
    abi_fingerprint, DeriveFn, DeriveLookup, DeriveProviderInfo, DeriveResult, ExtDeriveProvider,
    HostGraph,
};
/// The lookup kinds a provider's lookups use, the same across the C ABI and
/// from JS (so a WASM binding needn't depend on the ABI crate).
pub use alizarin_extension_api::{
    LOOKUP_NODEGROUP_BY_ID, LOOKUP_NODE_BY_ALIAS, LOOKUP_NODE_BY_ID, LOOKUP_ROOT,
};
use serde_json::Value;

use crate::graph::{
    DeriveConfig, DeriveProvider, FunctionsRegistry, GraphLookup, RegisteredFunction, StaticTile,
};

/// One lookup into `graph`, by kind (`LOOKUP_*`) and key, as the item's JSON.
pub fn lookup_json(graph: &dyn GraphLookup, kind: u32, key: &str) -> Option<Value> {
    match kind {
        LOOKUP_ROOT => serde_json::to_value(graph.get_root()).ok(),
        LOOKUP_NODE_BY_ALIAS => graph
            .get_node_by_alias(key)
            .and_then(|n| serde_json::to_value(n).ok()),
        LOOKUP_NODE_BY_ID => graph
            .get_node_by_id(key)
            .and_then(|n| serde_json::to_value(n).ok()),
        LOOKUP_NODEGROUP_BY_ID => graph
            .get_nodegroup_by_id(key)
            .and_then(|ng| serde_json::to_value(ng).ok()),
        _ => None,
    }
}

/// A host graph as an [`ExtDeriveProvider`] sees it, in-process.
pub struct GraphLookupView<'a>(pub &'a dyn GraphLookup);

impl DeriveLookup for GraphLookupView<'_> {
    fn lookup(&self, kind: u32, key: &str) -> Option<Value> {
        lookup_json(self.0, kind, key)
    }
}

/// An [`ExtDeriveProvider`] as a [`DeriveProvider`], to register in-process:
/// `RegisteredFunction::Derive(Arc::new(InProcess(provider)))`.
pub struct InProcess<P>(pub P);

impl<P: ExtDeriveProvider> DeriveProvider for InProcess<P> {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &DeriveConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let mut derived = self
            .0
            .derive(resource_id, &GraphLookupView(graph), tiles, config)?;
        coerce_derived_tiles(graph, &mut derived)?;
        Ok(derived)
    }
}

/// Coerce each value a provider outside this host derived (across the C
/// ABI, from JS, or an [`ExtDeriveProvider`] in-process) for its node's
/// datatype, with this host's language and extension handlers: such a
/// provider returns raw values, as it has neither. A value already in its
/// datatype's shape comes back unchanged, so a provider that coerces its
/// own output is not harmed. A key that names no node is left alone.
pub fn coerce_derived_tiles(
    graph: &dyn GraphLookup,
    tiles: &mut [StaticTile],
) -> Result<(), String> {
    let registry = crate::get_global_extension_registry();
    for tile in tiles.iter_mut() {
        for (nodeid, value) in tile.data.iter_mut() {
            let Some(node) = graph.get_node_by_id(nodeid) else {
                continue;
            };
            let config = serde_json::to_value(&node.config).ok();
            let coerced = crate::type_coercion::coerce_value_with_registry(
                &node.datatype,
                value,
                config.as_ref(),
                registry.as_ref(),
            );
            if let Some(error) = coerced.error {
                return Err(format!(
                    "derived value for node '{}' ({}): {error}",
                    node.alias.as_deref().unwrap_or(nodeid),
                    node.datatype
                ));
            }
            *value = coerced.tile_data;
        }
    }
    Ok(())
}

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
    match lookup_json(ctx.graph, kind, key) {
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
        config: &DeriveConfig,
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
            let mut derived = outcome?;
            coerce_derived_tiles(graph, &mut derived)?;
            Ok(derived)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    use alizarin_extension_api::{derive_provider_info, derive_with};
    use serde_json::json;

    const PROVIDER: &str = "derive-abi-test-label";

    /// `label` = the node named by role "source" upper-cased, suffixed with
    /// the alias the provider looks up itself (from its own config).
    struct Label;
    impl ExtDeriveProvider for Label {
        fn derive(
            &self,
            resource_id: &str,
            graph: &dyn DeriveLookup,
            tiles: &[StaticTile],
            config: &DeriveConfig,
        ) -> Result<Vec<StaticTile>, String> {
            let source = config.node("source")?;
            let value = tiles
                .iter()
                .find_map(|t| t.data.get(&source.nodeid).and_then(Value::as_str))
                .ok_or("no source value")?;
            // A lookup through the handoff, not the declaration.
            let output = graph
                .node_by_alias("label")
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

    /// What `coerce_derived_tiles` relies on: a provider that already coerced
    /// its output (as JS providers do) gets it back unchanged.
    #[test]
    fn coercing_a_coerced_value_changes_nothing() {
        use crate::type_coercion::coerce_value_with_registry as coerce;
        let irish = json!({"language": "ga"});
        let cases = [
            ("number", json!(30), None),
            ("number", json!(699320.12), None),
            ("non-localized-string", json!(30), None),
            ("non-localized-string", json!("N"), None),
            ("string", json!("N"), Some(&irish)),
            ("edtf", json!("2020-01"), None),
            ("date", json!("2020-01-02"), None),
            ("url", json!("https://example.org"), None),
            (
                "geojson-feature-collection",
                json!({"type": "FeatureCollection", "features": [{"type": "Feature",
                    "properties": {}, "geometry": {"type": "Point", "coordinates": [1, 2]}}]}),
                None,
            ),
        ];
        for (datatype, raw, config) in cases {
            let once = coerce(datatype, &raw, config, None);
            assert!(once.error.is_none(), "{datatype}: {:?}", once.error);
            let twice = coerce(datatype, &once.tile_data, config, None);
            assert!(twice.error.is_none(), "{datatype}: {:?}", twice.error);
            assert_eq!(twice.tile_data, once.tile_data, "{datatype}");
        }
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
