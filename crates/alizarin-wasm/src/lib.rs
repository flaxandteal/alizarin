mod utils;
pub mod graph;
pub mod pseudos;
pub mod pseudo_value;
pub mod model_wrapper;
pub mod instance_wrapper;
pub mod json_conversion;
pub mod wasm_wrappers;
pub mod tracing;
pub mod skos_wasm;
pub mod node_config_wasm;
pub mod type_coercion_wasm;
pub mod label_resolution_wasm;

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

#[wasm_bindgen]
pub fn greet() {
}
