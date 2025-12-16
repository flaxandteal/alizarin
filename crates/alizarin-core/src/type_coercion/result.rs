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
}

impl CoercionResult {
    pub fn success(tile_data: Value, display_value: Value) -> Self {
        CoercionResult {
            tile_data,
            display_value,
            error: None,
        }
    }

    pub fn success_same(value: Value) -> Self {
        CoercionResult {
            tile_data: value.clone(),
            display_value: value,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        CoercionResult {
            tile_data: Value::Null,
            display_value: Value::Null,
            error: Some(message.into()),
        }
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_null(&self) -> bool {
        self.tile_data.is_null() && self.error.is_none()
    }
}
