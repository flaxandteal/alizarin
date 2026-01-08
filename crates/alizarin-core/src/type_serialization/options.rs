//! Serialization options and result types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Output mode for serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SerializationMode {
    /// Output tile_data format (UUIDs, language maps as-is)
    #[default]
    TileData,
    /// Output display format (resolve UUIDs to labels, extract display strings)
    Display,
}

/// Options for value serialization
#[derive(Debug, Clone, Default)]
pub struct SerializationOptions {
    /// Output mode: TileData or Display
    pub mode: SerializationMode,
    /// Language for display output (default: "en")
    pub language: String,
    /// Include all language variants for strings (only in TileData mode)
    pub include_all_languages: bool,
}

impl SerializationOptions {
    /// Create options for tile_data output
    pub fn tile_data() -> Self {
        Self {
            mode: SerializationMode::TileData,
            language: "en".to_string(),
            include_all_languages: true,
        }
    }

    /// Create options for display output
    pub fn display(language: impl Into<String>) -> Self {
        Self {
            mode: SerializationMode::Display,
            language: language.into(),
            include_all_languages: false,
        }
    }

    /// Check if we're in display mode
    pub fn is_display(&self) -> bool {
        self.mode == SerializationMode::Display
    }
}

/// Result of a serialization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationResult {
    /// The serialized value
    pub value: Value,
    /// Error message if serialization failed
    pub error: Option<String>,
}

impl SerializationResult {
    pub fn success(value: Value) -> Self {
        SerializationResult { value, error: None }
    }

    pub fn error(message: impl Into<String>) -> Self {
        SerializationResult {
            value: Value::Null,
            error: Some(message.into()),
        }
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Unwrap the value, panicking if there was an error
    pub fn unwrap(self) -> Value {
        if let Some(err) = self.error {
            panic!("Serialization error: {}", err);
        }
        self.value
    }

    /// Get the value or a default if there was an error
    pub fn unwrap_or(self, default: Value) -> Value {
        if self.error.is_some() {
            default
        } else {
            self.value
        }
    }
}
