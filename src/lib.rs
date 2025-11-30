mod utils;
pub mod graph;
pub mod pseudos;
pub mod pseudo_value;
pub mod model_wrapper;
pub mod instance_wrapper;
pub mod json_conversion;
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
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    Ok(())
}

#[wasm_bindgen]
pub fn greet() {
}
