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
#[derive(Debug, Clone)]
pub struct SerializationOptions {
    /// Output mode: TileData or Display
    pub mode: SerializationMode,
    /// Language for display output (default: "en")
    pub language: String,
    /// Include all language variants for strings (only in TileData mode)
    pub include_all_languages: bool,
    /// Flatten outer nodes when children are empty (default: true)
    ///
    /// When true: `monument_type` with no `monument_metatype` → `"Hotel/Inn"`
    /// When false: `monument_type` with no `monument_metatype` → `{"_": "Hotel/Inn"}`
    pub flatten_empty_outer_nodes: bool,
    /// Key name for outer node's own value when merged with children (default: "_")
    ///
    /// When an outer node has both its own value and children, the output is:
    /// `{ "<outer_value_key>": ownValue, "child1": ..., "child2": ... }`
    pub outer_value_key: String,
}

impl Default for SerializationOptions {
    fn default() -> Self {
        Self {
            mode: SerializationMode::default(),
            language: String::new(),
            include_all_languages: false,
            flatten_empty_outer_nodes: true,
            outer_value_key: "_".to_string(),
        }
    }
}

impl SerializationOptions {
    /// Create options for tile_data output
    pub fn tile_data() -> Self {
        Self {
            mode: SerializationMode::TileData,
            language: "en".to_string(),
            include_all_languages: true,
            flatten_empty_outer_nodes: true,
            outer_value_key: "_".to_string(),
        }
    }

    /// Create options for display output
    pub fn display(language: impl Into<String>) -> Self {
        Self {
            mode: SerializationMode::Display,
            language: language.into(),
            include_all_languages: false,
            flatten_empty_outer_nodes: true,
            outer_value_key: "_".to_string(),
        }
    }

    /// Check if we're in display mode
    pub fn is_display(&self) -> bool {
        self.mode == SerializationMode::Display
    }

    /// Merge an outer node's own value with its children JSON.
    ///
    /// This handles the common pattern where a node has both a value (e.g., concept)
    /// and semantic children. The output format depends on options:
    ///
    /// - If `flatten_empty_outer_nodes` is true and children are empty: returns just `own_value`
    /// - Otherwise: returns `{ "<outer_value_key>": own_value, ...children }`
    ///
    /// # Arguments
    /// * `own_value` - The serialized value of the outer node itself
    /// * `children_json` - The serialized children (should be a JSON object)
    ///
    /// # Returns
    /// The merged JSON value
    pub fn merge_outer_with_children(&self, own_value: Value, children_json: Value) -> Value {
        if let Value::Object(mut children_map) = children_json {
            // If flatten_empty_outer_nodes is true and children are empty,
            // return just the own_value instead of wrapping
            if children_map.is_empty() && self.flatten_empty_outer_nodes {
                return own_value;
            }

            if !own_value.is_null() {
                children_map.insert(self.outer_value_key.clone(), own_value);
            }
            return Value::Object(children_map);
        }

        // No valid children object - just return own value
        own_value
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_outer_flattens_when_children_empty() {
        let opts = SerializationOptions::display("en");
        let own_value = json!("Hotel/Inn");
        let children_json = json!({});

        let result = opts.merge_outer_with_children(own_value.clone(), children_json);

        // Should flatten to just the own_value when children are empty
        assert_eq!(result, json!("Hotel/Inn"));
    }

    #[test]
    fn test_merge_outer_wraps_when_children_present() {
        let opts = SerializationOptions::display("en");
        let own_value = json!("Hotel/Inn");
        let children_json = json!({"monument_metatype": "Some Value"});

        let result = opts.merge_outer_with_children(own_value, children_json);

        // Should wrap with "_" key when children exist
        assert_eq!(
            result,
            json!({
                "_": "Hotel/Inn",
                "monument_metatype": "Some Value"
            })
        );
    }

    #[test]
    fn test_merge_outer_no_flatten_when_disabled() {
        let mut opts = SerializationOptions::display("en");
        opts.flatten_empty_outer_nodes = false;

        let own_value = json!("Hotel/Inn");
        let children_json = json!({});

        let result = opts.merge_outer_with_children(own_value, children_json);

        // Should wrap even with empty children when flatten is disabled
        assert_eq!(result, json!({"_": "Hotel/Inn"}));
    }

    #[test]
    fn test_merge_outer_custom_key() {
        let mut opts = SerializationOptions::display("en");
        opts.flatten_empty_outer_nodes = false;
        opts.outer_value_key = "_value".to_string();

        let own_value = json!("Hotel/Inn");
        let children_json = json!({});

        let result = opts.merge_outer_with_children(own_value, children_json);

        // Should use custom key
        assert_eq!(result, json!({"_value": "Hotel/Inn"}));
    }

    #[test]
    fn test_merge_outer_null_own_value_with_children() {
        let opts = SerializationOptions::display("en");
        let own_value = Value::Null;
        let children_json = json!({"child": "value"});

        let result = opts.merge_outer_with_children(own_value, children_json);

        // Should return children without adding null own_value
        assert_eq!(result, json!({"child": "value"}));
    }

    #[test]
    fn test_merge_outer_non_object_children() {
        let opts = SerializationOptions::display("en");
        let own_value = json!("Hotel/Inn");
        let children_json = json!("not an object");

        let result = opts.merge_outer_with_children(own_value.clone(), children_json);

        // Should return own_value when children_json is not an object
        assert_eq!(result, own_value);
    }
}
