//! Derive providers from JavaScript: what an extension's JS package registers
//! on the WASM backend, as NAPI and Python register an extension's
//! `DeriveProviderInfo` (core's [`derive_abi`](alizarin_core::derive_abi)).
//!
//! `registerDeriveProvider(providerId, { derive })` puts a provider in core's
//! global functions registry under `providerId`. Core then calls
//! `derive(resourceId, tiles, config, lookup)`, synchronously, with the
//! resource's tiles and its declaration (its `nodes` resolved), and expects the
//! derived tiles back. `lookup(kind, key)` answers from this module's graph,
//! as `HostGraph` does across the C ABI (kinds: 0 root, 1 node by alias,
//! 2 node by id, 3 nodegroup by id); it works only during the call.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use alizarin_core::derive_abi::{
    LOOKUP_NODEGROUP_BY_ID, LOOKUP_NODE_BY_ALIAS, LOOKUP_NODE_BY_ID, LOOKUP_ROOT,
};
use alizarin_core::{
    ComputeTilesConfig, DeriveProvider, GraphLookup, RegisteredFunction, StaticTile,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

thread_local! {
    /// The JS `derive` callbacks, by provider id. JS functions aren't Send, so
    /// they live here and the registered provider holds only the id.
    static JS_DERIVE_PROVIDERS: RefCell<HashMap<String, js_sys::Function>> =
        RefCell::new(HashMap::new());
}

/// Register a derive provider implemented in JS under `providerId`:
/// `options.derive(resourceId, tiles, config, lookup)` returns the derived
/// tiles. Replaces any provider already registered under that id.
#[wasm_bindgen(js_name = registerDeriveProvider)]
pub fn register_derive_provider(provider_id: &str, options: JsValue) -> Result<(), JsValue> {
    let derive = js_sys::Reflect::get(&options, &JsValue::from_str("derive"))
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
        .ok_or_else(|| JsValue::from_str("options.derive must be a function"))?;
    JS_DERIVE_PROVIDERS.with(|providers| {
        providers
            .borrow_mut()
            .insert(provider_id.to_string(), derive)
    });
    alizarin_core::with_global_functions_registry_mut(|registry| {
        registry.register(
            provider_id,
            RegisteredFunction::Derive(Arc::new(JsDeriveProvider {
                provider_id: provider_id.to_string(),
            })),
        )
    })
    .ok_or_else(|| JsValue::from_str("the global functions registry is unavailable"))
}

/// Whether a derive provider is registered under `providerId`, from JS or
/// compiled in. As on NAPI and Python.
#[wasm_bindgen(js_name = hasDeriveProvider)]
pub fn has_derive_provider(provider_id: &str) -> bool {
    alizarin_core::get_global_functions_registry()
        .derive(provider_id)
        .is_some()
}

/// A [`DeriveProvider`] that calls a registered JS `derive` callback.
struct JsDeriveProvider {
    provider_id: String,
}

// JS functions aren't thread-safe, but WASM runs on one thread, and the
// callback itself stays in the thread-local above.
unsafe impl Send for JsDeriveProvider {}
unsafe impl Sync for JsDeriveProvider {}

fn to_js(value: &impl Serialize) -> Result<JsValue, String> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|e| e.to_string())
}

impl DeriveProvider for JsDeriveProvider {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let derive = JS_DERIVE_PROVIDERS
            .with(|providers| providers.borrow().get(&self.provider_id).cloned())
            .ok_or_else(|| format!("no JS derive provider {}", self.provider_id))?;

        // `lookup` answers from `graph` during this call only: after it, the
        // flag is down and the closure answers null, so a JS reference kept
        // past the call can never reach a graph that has gone.
        let live = Rc::new(Cell::new(true));
        // SAFETY: only dereferenced while `live` is set, i.e. within this call,
        // while `graph` is borrowed.
        let graph: *const (dyn GraphLookup + 'static) =
            unsafe { std::mem::transmute(graph as *const dyn GraphLookup) };
        let lookup = {
            let live = live.clone();
            Closure::<dyn Fn(u32, String) -> JsValue>::new(move |kind: u32, key: String| {
                if !live.get() {
                    return JsValue::NULL;
                }
                let graph = unsafe { &*graph };
                let found = match kind {
                    LOOKUP_ROOT => to_js(graph.get_root()).ok(),
                    LOOKUP_NODE_BY_ALIAS => {
                        graph.get_node_by_alias(&key).and_then(|n| to_js(n).ok())
                    }
                    LOOKUP_NODE_BY_ID => graph.get_node_by_id(&key).and_then(|n| to_js(n).ok()),
                    LOOKUP_NODEGROUP_BY_ID => graph
                        .get_nodegroup_by_id(&key)
                        .and_then(|ng| to_js(ng).ok()),
                    _ => None,
                };
                found.unwrap_or(JsValue::NULL)
            })
        };

        let args = js_sys::Array::of4(
            &JsValue::from_str(resource_id),
            &to_js(&tiles)?,
            &to_js(config)?,
            lookup.as_ref(),
        );
        let result = derive.apply(&JsValue::NULL, &args);
        live.set(false);
        let result = result.map_err(|e| {
            format!(
                "derive provider {}: {}",
                self.provider_id,
                e.as_string().unwrap_or_else(|| format!("{e:?}"))
            )
        })?;
        serde_wasm_bindgen::from_value(result)
            .map_err(|e| format!("derive provider {}: derived tiles: {e}", self.provider_id))
    }
}
