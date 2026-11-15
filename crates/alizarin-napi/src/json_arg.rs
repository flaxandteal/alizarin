//! A JSON argument from JavaScript, converted through V8's own JSON.stringify.
//!
//! napi-rs converts a `serde_json::Value` argument by walking the object
//! property by property, with no cycle check: a cyclic object (a view model
//! pointing back at its pseudo, say) recursed until the native stack
//! overflowed and V8 aborted the whole process. JSON.stringify throws a
//! TypeError on a cycle instead, as on WASM (a RangeError) and Python (a
//! ValueError), and is one native call rather than one per property.

use napi::bindgen_prelude::{FromNapiValue, TypeName, ValidateNapiValue};
use napi::{sys, Env, JsFunction, JsObject, JsUnknown, NapiValue, ValueType};

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
