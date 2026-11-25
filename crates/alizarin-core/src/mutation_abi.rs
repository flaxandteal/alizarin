//! Extension graph mutations across a binding boundary: the host side of
//! [`MutationHandlerInfo`], as [`derive_abi`](crate::derive_abi) is for
//! derive providers, and the one registry Python and NAPI register into.
//!
//! An extension's mutation implements the extension API's
//! `ExtMutationHandler` over the graph as JSON, and exports a
//! [`MutationHandlerInfo`] (a capsule on Python, a pointer on NAPI). Here it
//! becomes an [`ExtensionMutationHandler`]: the host serialises its graph,
//! calls the extension, and parses the graph it hands back. WASM registers JS
//! functions into its own registry, as it does datatype handlers.

use std::sync::{Arc, RwLock};

use alizarin_extension_api::{abi_fingerprint, MutationConformanceLevel, MutationHandlerInfo};

use crate::graph::StaticGraph;
use crate::graph_mutator::{
    apply_mutations_from_json_with_extensions, ExtensionMutationHandler, ExtensionMutationRegistry,
    MutationConformance, MutationError, MutatorOptions,
};

lazy_static::lazy_static! {
    static ref EXTENSION_MUTATIONS: RwLock<ExtensionMutationRegistry> =
        RwLock::new(ExtensionMutationRegistry::new());
}

/// Register `handler` under `name` in the host's extension-mutation
/// registry, replacing any handler already there.
pub fn register_extension_mutation(name: &str, handler: Arc<dyn ExtensionMutationHandler>) {
    if let Ok(mut registry) = EXTENSION_MUTATIONS.write() {
        registry.register(name, handler);
    }
}

/// Remove the handler registered under `name`; whether there was one.
pub fn unregister_extension_mutation(name: &str) -> bool {
    EXTENSION_MUTATIONS
        .write()
        .map(|mut registry| registry.remove(name))
        .unwrap_or(false)
}

/// Whether a handler is registered under `name`.
pub fn has_extension_mutation(name: &str) -> bool {
    EXTENSION_MUTATIONS
        .read()
        .map(|registry| registry.has(name))
        .unwrap_or(false)
}

/// The registered mutation names, sorted.
pub fn registered_extension_mutations() -> Vec<String> {
    let mut names: Vec<String> = EXTENSION_MUTATIONS
        .read()
        .map(|registry| registry.list().into_iter().map(str::to_string).collect())
        .unwrap_or_default();
    names.sort();
    names
}

/// Run `f` with the registered extension mutations.
pub fn with_extension_mutations<R>(
    f: impl FnOnce(&ExtensionMutationRegistry) -> R,
) -> Result<R, String> {
    let registry = EXTENSION_MUTATIONS
        .read()
        .map_err(|_| "the extension mutation registry is unavailable".to_string())?;
    Ok(f(&registry))
}

/// Apply a mutation request (core's `MutationRequest` JSON) to `graph`, with
/// the registered extension mutations: what each binding's
/// `apply_mutations_with_extensions` does.
pub fn apply_mutations_with_registered_extensions(
    graph: &StaticGraph,
    mutations_json: &str,
) -> Result<StaticGraph, String> {
    with_extension_mutations(|registry| {
        apply_mutations_from_json_with_extensions(graph, mutations_json, Some(registry))
    })?
}

/// An extension's mutation, called through its exported functions.
struct AbiMutationHandler {
    name: String,
    info: &'static MutationHandlerInfo,
}

impl ExtensionMutationHandler for AbiMutationHandler {
    fn apply(
        &self,
        graph: &mut StaticGraph,
        params: &serde_json::Value,
        _options: &MutatorOptions,
    ) -> Result<(), MutationError> {
        let graph_json = serde_json::to_vec(&*graph)
            .map_err(|e| MutationError::Other(format!("serialize graph: {e}")))?;
        let params_json = serde_json::to_vec(params)
            .map_err(|e| MutationError::Other(format!("serialize params: {e}")))?;
        let mut output_ptr: *mut u8 = std::ptr::null_mut();
        let mut output_len = 0;
        // SAFETY: the fn pointers come from an extension whose ABI fingerprint
        // was checked at registration; the inputs live for the call, and the
        // output (and the result's error) are freed by the extension's own
        // free fns, once each.
        let mutated = unsafe {
            let result = (self.info.handler_fn)(
                graph_json.as_ptr(),
                graph_json.len(),
                params_json.as_ptr(),
                params_json.len(),
                &mut output_ptr,
                &mut output_len,
            );
            let error = result.is_error().then(|| {
                let bytes = std::slice::from_raw_parts(result.error_ptr, result.error_len);
                String::from_utf8_lossy(bytes).into_owned()
            });
            (self.info.free_result_fn)(result);
            let mutated = match error {
                Some(error) => Err(error),
                None if output_ptr.is_null() => Err("no graph returned".to_string()),
                None => {
                    let bytes = std::slice::from_raw_parts(output_ptr, output_len);
                    serde_json::from_slice::<StaticGraph>(bytes)
                        .map_err(|e| format!("invalid graph returned: {e}"))
                }
            };
            if !output_ptr.is_null() {
                (self.info.free_output_fn)(output_ptr, output_len);
            }
            mutated
        };
        *graph = mutated.map_err(|e| MutationError::Other(format!("{}: {e}", self.name)))?;
        Ok(())
    }

    fn conformance(&self) -> MutationConformance {
        match self.info.conformance {
            MutationConformanceLevel::AlwaysConformant => MutationConformance::AlwaysConformant,
            MutationConformanceLevel::BranchConformant => MutationConformance::BranchConformant,
            MutationConformanceLevel::ModelConformant => MutationConformance::ModelConformant,
            MutationConformanceLevel::NonConformant => MutationConformance::NonConformant,
        }
    }

    fn description(&self) -> &str {
        "extension mutation (C ABI)"
    }
}

/// Register an extension's mutation from its [`MutationHandlerInfo`] pointer
/// under the name it carries, which is returned. What
/// `registerExtensionMutation` does on NAPI, and on Python given a capsule.
///
/// # Safety
/// `ptr` is null or points to a `MutationHandlerInfo` (and its name bytes and
/// fn pointers) that outlives the process's use of it, as an extension's
/// static info does. The ABI fingerprint is checked before anything else is
/// read.
pub unsafe fn register_mutation_handler_from_ptr(
    ptr: *const MutationHandlerInfo,
) -> Result<String, String> {
    if ptr.is_null() {
        return Err("null MutationHandlerInfo pointer".to_string());
    }
    let info: &'static MutationHandlerInfo = &*ptr;
    if info.abi != abi_fingerprint() {
        return Err(format!(
            "extension ABI mismatch: built against {:?}, host is {:?}: rebuild the extension \
             against this alizarin version",
            info.abi,
            abi_fingerprint()
        ));
    }
    let name = std::str::from_utf8(std::slice::from_raw_parts(
        info.mutation_name_ptr,
        info.mutation_name_len,
    ))
    .map_err(|e| format!("mutation name is not valid UTF-8: {e}"))?
    .to_string();
    register_extension_mutation(
        &name,
        Arc::new(AbiMutationHandler {
            name: name.clone(),
            info,
        }),
    );
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    use alizarin_extension_api::{
        mutate_with, mutation_handler_info, ExtMutationHandler, MutationResult,
    };
    use serde_json::{json, Value};

    const NAME: &str = "mutation-abi-test.rename_root";

    /// Renames the graph's root node to `params.name`.
    struct RenameRoot;
    impl ExtMutationHandler for RenameRoot {
        fn apply(&self, graph: &mut Value, params: &Value) -> Result<(), String> {
            let name = params["name"].as_str().ok_or("no name")?;
            graph["root"]["name"] = json!(name);
            Ok(())
        }
    }

    unsafe extern "C" fn rename_root(
        graph_ptr: *const u8,
        graph_len: usize,
        params_ptr: *const u8,
        params_len: usize,
        output_ptr: *mut *mut u8,
        output_len: *mut usize,
    ) -> MutationResult {
        mutate_with(
            &RenameRoot,
            graph_ptr,
            graph_len,
            params_ptr,
            params_len,
            output_ptr,
            output_len,
        )
    }

    fn info() -> &'static MutationHandlerInfo {
        static INFO: OnceLock<MutationHandlerInfo> = OnceLock::new();
        INFO.get_or_init(|| {
            mutation_handler_info(
                NAME,
                rename_root,
                MutationConformanceLevel::AlwaysConformant,
            )
        })
    }

    fn graph() -> StaticGraph {
        serde_json::from_value(json!({
            "graphid": "g", "name": {"en": "G"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
            "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"}],
            "edges": [], "nodegroups": [], "cards": [], "cards_x_nodes_x_widgets": [],
        }))
        .unwrap()
    }

    fn request(params: Value) -> String {
        json!({"mutations": [{"Extension": {"name": NAME, "params": params,
            "conformance": "AlwaysConformant"}}]})
        .to_string()
    }

    #[test]
    fn a_mutation_registered_through_the_abi_applies_to_the_hosts_graph() {
        let name = unsafe { register_mutation_handler_from_ptr(info()) }.unwrap();
        assert_eq!(name, NAME);
        assert!(has_extension_mutation(NAME));
        assert!(registered_extension_mutations().contains(&NAME.to_string()));

        let mutated =
            apply_mutations_with_registered_extensions(&graph(), &request(json!({"name": "Top"})))
                .unwrap();
        assert_eq!(mutated.root.name, "Top");
    }

    #[test]
    fn a_mutation_error_crosses_back_as_an_error() {
        unsafe { register_mutation_handler_from_ptr(info()) }.unwrap();
        let err =
            apply_mutations_with_registered_extensions(&graph(), &request(json!({}))).unwrap_err();
        assert!(err.contains("no name"), "{err}");
    }

    #[test]
    fn an_abi_mismatch_is_refused() {
        let mut bad = mutation_handler_info(
            NAME,
            rename_root,
            MutationConformanceLevel::AlwaysConformant,
        );
        bad.abi.version += 1;
        let bad: &'static MutationHandlerInfo = Box::leak(Box::new(bad));
        let err = unsafe { register_mutation_handler_from_ptr(bad) }.unwrap_err();
        assert!(err.contains("ABI mismatch"), "{err}");
    }
}
