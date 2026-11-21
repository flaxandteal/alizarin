//! A JSON argument from JavaScript, converted through V8's own JSON.stringify.
//!
//! napi-rs converts a `serde_json::Value` argument by walking the object
//! property by property, with no cycle check: a cyclic object (a view model
//! pointing back at its pseudo, say) recursed until the native stack
//! overflowed and V8 aborted the whole process. JSON.stringify throws a
//! TypeError on a cycle instead, as on WASM (a RangeError) and Python (a
//! ValueError), and is one native call rather than one per property.

use napi::bindgen_prelude::{FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue};
use napi::{sys, Env, JsFunction, JsObject, JsUnknown, NapiRaw, NapiValue, ValueType};

/// A JSON value passed from JavaScript: what JSON.stringify gives it (an
/// `undefined` is null). Take this, not `serde_json::Value`, for any argument
/// built by callers.
pub struct JsonArg(pub serde_json::Value);

impl TypeName for JsonArg {
    fn type_name() -> &'static str {
        "any"
    }

    fn value_type() -> ValueType {
        ValueType::Unknown
    }
}

impl ValidateNapiValue for JsonArg {}

impl FromNapiValue for JsonArg {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        let env = Env::from_raw(env);
        let json: JsObject = env.get_global()?.get_named_property("JSON")?;
        let stringify: JsFunction = json.get_named_property("stringify")?;
        let value = JsUnknown::from_raw_unchecked(env.raw(), napi_val);
        // Throws (TypeError: circular structure) back to the caller on a cycle.
        let text = stringify.call(Some(&json), &[value])?;
        if text.get_type()? != ValueType::String {
            return Ok(JsonArg(serde_json::Value::Null));
        }
        let text = text.coerce_to_string()?.into_utf8()?.into_owned()?;
        serde_json::from_str(&text)
            .map(JsonArg)
            .map_err(|e| napi::Error::from_reason(format!("Invalid JSON value: {e}")))
    }
}

/// A JSON value returned to JavaScript through V8's own JSON.parse, the mirror
/// of [`JsonArg`]: one native call, where napi-rs's `serde_json::Value` return
/// makes one per property (about ten times slower on a large business data
/// file, with a far higher peak). Keys keep `serde_json::Value`'s sorted order,
/// so the result stringifies as a `serde_json::Value` return does.
pub struct JsonOut(pub serde_json::Value);

impl TypeName for JsonOut {
    fn type_name() -> &'static str {
        "any"
    }

    fn value_type() -> ValueType {
        ValueType::Unknown
    }
}

impl ToNapiValue for JsonOut {
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
        let env = Env::from_raw(env);
        let text = serde_json::to_string(&val.0)
            .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {e}")))?;
        drop(val);
        let json: JsObject = env.get_global()?.get_named_property("JSON")?;
        let parse: JsFunction = json.get_named_property("parse")?;
        let parsed = parse.call(Some(&json), &[env.create_string_from_std(text)?])?;
        Ok(parsed.raw())
    }
}

/// A Rust value as a JS value, through `serde_json::Value`: keys in its sorted
/// order, as every `serde_json::Value` return and WASM's `js_json::to_value`
/// give. `env.to_js_value` on a struct writes field order instead, so the same
/// value stringified differently by route. `tests/dual-backend/outputParity.test.ts`
/// holds every route to it.
pub(crate) fn to_js<T: serde::Serialize + ?Sized>(env: &Env, value: &T) -> napi::Result<JsUnknown> {
    let json = serde_json::to_value(value).map_err(|e| napi::Error::from_reason(e.to_string()))?;
    env.to_js_value(&json)
}
