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
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// The one way a WASM binding hands a Rust value to JS: through
/// `serde_json::Value`, as NAPI does, so both backends give the same plain
/// objects (no `Map`s), with `None` as `null` (not `undefined`, which
/// `JSON.stringify` drops) and keys in `serde_json`'s sorted order (not struct
/// order). `tests/dual-backend/outputParity.test.ts` holds every binding to it.
pub fn to_value<T: Serialize + ?Sized>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    serde_json::to_value(value)
        .map_err(|e| serde_wasm_bindgen::Error::new(e.to_string()))?
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
}

/// An optional result for JS: `None` as `null`, as NAPI gives it.
/// wasm-bindgen returns `undefined` for an `Option` result, so an exported
/// function or getter that may have nothing returns `JsValue`, declared with
/// `unchecked_return_type = "T | null"`, and returns this.
pub fn or_null<T: Into<JsValue>>(value: Option<T>) -> JsValue {
    value.map(Into::into).unwrap_or(JsValue::NULL)
}

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
