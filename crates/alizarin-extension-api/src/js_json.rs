//! JSON arguments from JavaScript, converted through JSON.stringify.
//!
//! `serde_wasm_bindgen::from_value` walks a JS value property by property with
//! no cycle check: a cyclic object recursed until the WASM stack overflowed,
//! and the trap could leave the module's memory corrupt for every later call
//! (`memory access out of bounds`). JSON.stringify throws a TypeError on a
//! cycle before anything is converted, as NAPI's `JsonArg` does, so both
//! JavaScript backends read their arguments the same way. JS Maps convert as
//! objects of their entries; anything with `toJSON` (a WASM StaticTile, a Date)
//! converts through it.
//!
//! Shared by alizarin-wasm and every extension's WASM crate: each is its own
//! WASM module, each at the same risk.

use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "export function json_text(value) {
  return JSON.stringify(value, (_key, item) => (item instanceof Map ? Object.fromEntries(item) : item));
}")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn json_text(value: &JsValue) -> Result<JsValue, JsValue>;
}

/// `serde_wasm_bindgen::from_value`, through JSON.stringify: a cycle is an
/// error, not a stack overflow. An `undefined` reads as null.
pub fn from_value<T: DeserializeOwned>(value: JsValue) -> Result<T, serde_wasm_bindgen::Error> {
    let text = json_text(&value).map_err(|error| {
        let message = js_sys::Error::from(error)
            .message()
            .as_string()
            .unwrap_or_else(|| "value cannot be converted to JSON".to_string());
        serde_wasm_bindgen::Error::new(message)
    })?;
    let text = text.as_string().unwrap_or_else(|| "null".to_string());
    serde_json::from_str(&text).map_err(serde_wasm_bindgen::Error::new)
}
