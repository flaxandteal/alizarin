//! Graph mutations in an extension: the extension side of the C ABI
//! ([`mutate_with`], [`mutation_handler_info`]).
//!
//! A mutation here implements [`ExtMutationHandler`] over the graph as JSON,
//! and needs no alizarin-core: the host serialises its graph, calls the
//! extension, and parses the graph it hands back. alizarin-core registers it
//! through `mutation_abi::register_mutation_handler_from_ptr`, on Python and
//! NAPI; a WASM extension exports the same transform to JS.

use std::panic::{catch_unwind, AssertUnwindSafe};

use serde_json::Value;

use crate::{
    alizarin_free_mutation_output, alizarin_free_mutation_result, MutationConformanceLevel,
    MutationHandlerFn, MutationHandlerInfo, MutationResult,
};

/// A graph mutation written against the extension API: changes `graph` (a
/// `StaticGraph`'s JSON) as `params` say, or says why it can't.
pub trait ExtMutationHandler: Send + Sync {
    fn apply(&self, graph: &mut Value, params: &Value) -> Result<(), String>;
}

/// Run `handler` for one [`MutationHandlerFn`] call: what an extension's
/// exported handler fn does with its arguments. The mutated graph's JSON goes
/// out through `output_ptr`/`output_len`; an error or panic comes back as the
/// result's error.
///
/// # Safety
/// The arguments are a [`MutationHandlerFn`]'s, from the host: the input
/// slices are live for the call, and the output pointers writable.
pub unsafe fn mutate_with<H: ExtMutationHandler + ?Sized>(
    handler: &H,
    graph_ptr: *const u8,
    graph_len: usize,
    params_ptr: *const u8,
    params_len: usize,
    output_ptr: *mut *mut u8,
    output_len: *mut usize,
) -> MutationResult {
    let run = || -> Result<Vec<u8>, String> {
        let mut graph: Value =
            serde_json::from_slice(std::slice::from_raw_parts(graph_ptr, graph_len))
                .map_err(|e| format!("graph: {e}"))?;
        let params: Value =
            serde_json::from_slice(std::slice::from_raw_parts(params_ptr, params_len))
                .map_err(|e| format!("params: {e}"))?;
        handler.apply(&mut graph, &params)?;
        serde_json::to_vec(&graph).map_err(|e| format!("serialize graph: {e}"))
    };
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(Ok(json)) => {
            *output_len = json.len();
            *output_ptr = Box::into_raw(json.into_boxed_slice()) as *mut u8;
            MutationResult::success()
        }
        Ok(Err(error)) => MutationResult::error(error),
        Err(_) => MutationResult::error("the mutation handler panicked".to_string()),
    }
}

/// The [`MutationHandlerInfo`] an extension exports for `handler_fn` (which
/// calls [`mutate_with`]) under `name`, freed by this build's free fns.
pub fn mutation_handler_info(
    name: &'static str,
    handler_fn: MutationHandlerFn,
    conformance: MutationConformanceLevel,
) -> MutationHandlerInfo {
    MutationHandlerInfo::new(
        name,
        handler_fn,
        alizarin_free_mutation_result,
        alizarin_free_mutation_output,
        conformance,
    )
}
