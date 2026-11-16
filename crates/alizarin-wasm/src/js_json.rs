//! The JSON boundary with JavaScript: cycle-safe arguments in and plain,
//! sorted, null-preserving values out, shared with every extension's WASM
//! crate (see `alizarin_extension_api::js_json`).

pub use alizarin_extension_api::js_json::{from_value, or_null, to_value};
