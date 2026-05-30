//! CoercionResult type for type coercion operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result of a coercion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoercionResult {
    /// The value suitable for tile storage (tile.data[nodeid])
    pub tile_data: Value,
    /// The "display" or resolved value (for ViewModel construction)
    pub display_value: Value,
    /// Error message if coercion failed
    pub error: Option<String>,
    /// Non-fatal warnings (e.g. out-of-range coordinates)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// True when the datatype was unknown and the value was passed through unchanged
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub passthrough: bool,
}

impl CoercionResult {
    pub fn success(tile_data: Value, display_value: Value) -> Self {
        CoercionResult {
            tile_data,
            display_value,
            error: None,
            warnings: vec![],
            passthrough: false,
        }
    }

    pub fn success_same(value: Value) -> Self {
        CoercionResult {
            tile_data: value.clone(),
            display_value: value,
            error: None,
            warnings: vec![],
            passthrough: false,
        }
    }

    /// Value passed through unchanged because the datatype is unknown to core.
    pub fn success_passthrough(value: Value) -> Self {
        CoercionResult {
            tile_data: value.clone(),
            display_value: value,
            error: None,
            warnings: vec![],
            passthrough: true,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        CoercionResult {
            tile_data: Value::Null,
            display_value: Value::Null,
            error: Some(message.into()),
            warnings: vec![],
            passthrough: false,
        }
    }

    /// Add a warning to this result
    pub fn with_warning(mut self, message: impl Into<String>) -> Self {
        self.warnings.push(message.into());
        self
    }

    /// Add multiple warnings to this result
    pub fn with_warnings(mut self, messages: Vec<String>) -> Self {
        self.warnings.extend(messages);
        self
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_null(&self) -> bool {
        self.tile_data.is_null() && self.error.is_none()
    }
}
