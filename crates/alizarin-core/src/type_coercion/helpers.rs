//! Helper functions for type coercion.

use serde_json::Value;

pub use crate::label_resolution::is_valid_uuid;

/// Get the name of a JSON value type for error messages.
pub fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
