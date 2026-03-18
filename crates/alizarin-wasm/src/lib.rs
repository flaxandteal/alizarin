pub mod batch_conversion;
pub mod extension_registry;
pub mod graph;
pub mod graph_mutator_wasm;
pub mod instance_wrapper;
pub mod json_conversion;
pub mod label_resolution_wasm;
pub mod model_wrapper;
pub mod node_config_wasm;
pub mod pseudo_value;
pub mod pseudos;
pub mod rdm_cache_wasm;
pub mod rdm_namespace_wasm;
pub mod skos_wasm;
pub mod tracing;
pub mod type_coercion_wasm;
mod utils;
pub mod wasm_wrappers;

// Re-export core types for native Rust consumers
// These are the platform-agnostic types without WASM bindings
pub mod core {
    pub use alizarin_core::*;
}

use wasm_bindgen::prelude::*;

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages in development
    // Always enabled to help debug WASM panics
    console_error_panic_hook::set_once();

    Ok(())
}
