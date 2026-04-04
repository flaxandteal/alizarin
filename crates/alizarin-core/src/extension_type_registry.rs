//! Unified extension type registry for custom datatype handlers.
//!
//! This module provides a platform-agnostic registry for extension type handlers
//! that can be used by both WASM and Python bindings. Extensions can register
//! handlers for custom datatypes that provide:
//!
//! - **Coercion**: Transform input values during ETL (tree → tiles)
//! - **Display rendering**: Convert tile data to display strings
//! - **Marker resolution**: Resolve references/lookups in tile data
//!
//! # Architecture
//!
//! The registry stores handlers implementing the `ExtensionTypeHandler` trait.
//! Platform-specific bindings (WASM, Python) wrap their native callbacks
//! (JS functions, Python callables) to implement this trait.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    ExtensionTypeRegistry                     │
//! │                    (alizarin-core)                          │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!              ┌───────────────┼───────────────┐
//!              │               │               │
//!              ▼               ▼               ▼
//!     ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
//!     │ WASM Handler │  │ Python Handler│  │ Future...  │
//!     │ (JS callback)│  │ (C ABI)       │  │            │
//!     └─────────────┘  └─────────────┘  └─────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use alizarin_core::extension_type_registry::{
//!     ExtensionTypeRegistry, ExtensionTypeHandler, HandlerCapabilities,
//! };
//!
//! // Create registry
//! let mut registry = ExtensionTypeRegistry::new();
//!
//! // Register a handler
//! registry.register("my-custom-type", Arc::new(MyHandler));
//!
//! // Use in coercion/serialization
//! if let Some(handler) = registry.get("my-custom-type") {
//!     let result = handler.coerce(&value, None)?;
//! }
//! ```

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::type_coercion::CoercionResult;

/// Describes what capabilities an extension handler provides.
///
/// Not all handlers need to implement all capabilities. For example,
/// a simple type might only need coercion, while a reference type
/// needs all three.
#[derive(Debug, Clone, Default)]
pub struct HandlerCapabilities {
    /// Can transform input values during ETL (tree → tiles)
    pub can_coerce: bool,
    /// Can render tile data to display strings
    pub can_render_display: bool,
    /// Can render tile data to search-indexable JSON
    pub can_render_search: bool,
    /// Can resolve markers (e.g., reference labels from RDM)
    pub can_resolve_markers: bool,
}

impl HandlerCapabilities {
    /// Create capabilities for a coercion-only handler.
    pub fn coercion_only() -> Self {
        Self {
            can_coerce: true,
            can_render_display: false,
            can_render_search: false,
            can_resolve_markers: false,
        }
    }

    /// Create capabilities for a display-only handler.
    pub fn display_only() -> Self {
        Self {
            can_coerce: false,
            can_render_display: true,
            can_render_search: false,
            can_resolve_markers: false,
        }
    }

    /// Create capabilities for a full handler (all capabilities).
    pub fn full() -> Self {
        Self {
            can_coerce: true,
            can_render_display: true,
            can_render_search: false,
            can_resolve_markers: true,
        }
    }
}

/// Error type for extension handler operations.
#[derive(Debug, Clone)]
pub struct ExtensionError {
    pub message: String,
}

impl ExtensionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExtensionError {}

/// Trait for extension type handlers.
///
/// Implementations wrap platform-specific callbacks (JS functions, Python callables)
/// to provide a unified interface for custom datatype handling.
///
/// # Required Methods
///
/// Only `capabilities()` is strictly required. Other methods have default
/// implementations that return errors or pass-through values.
///
/// # Thread Safety
///
/// Handlers must be `Send + Sync` to allow use in multi-threaded contexts
/// (e.g., Rayon parallel iteration in batch processing).
pub trait ExtensionTypeHandler: Send + Sync {
    /// Returns what this handler can do.
    fn capabilities(&self) -> HandlerCapabilities;

    /// Coerce an input value during ETL.
    ///
    /// # Arguments
    /// * `value` - The input value to coerce
    /// * `config` - Optional node configuration (e.g., collection ID for concepts)
    ///
    /// # Returns
    /// `CoercionResult` with `tile_data` (for storage) and `display_value` (for display)
    fn coerce(
        &self,
        value: &Value,
        _config: Option<&Value>,
    ) -> Result<CoercionResult, ExtensionError> {
        // Default: pass through unchanged
        Ok(CoercionResult::success(value.clone(), value.clone()))
    }

    /// Render tile data to a display string.
    ///
    /// # Arguments
    /// * `tile_data` - The resolved tile data
    /// * `language` - The language code for localization
    ///
    /// # Returns
    /// `Some(String)` if rendered, `None` to use default rendering
    fn render_display(
        &self,
        _tile_data: &Value,
        _language: &str,
    ) -> Result<Option<String>, ExtensionError> {
        // Default: no custom rendering
        Ok(None)
    }

    /// Render tile data to search-indexable JSON.
    ///
    /// Unlike `render_display` which returns a string, this returns a JSON Value
    /// suitable for search indexing. Falls back to `render_display` (wrapped as
    /// `Value::String`) if not implemented.
    ///
    /// # Arguments
    /// * `tile_data` - The resolved tile data
    /// * `language` - The language code for localization
    ///
    /// # Returns
    /// `Some(Value)` if rendered, `None` to fall back to display rendering
    fn render_search(
        &self,
        _tile_data: &Value,
        _language: &str,
    ) -> Result<Option<Value>, ExtensionError> {
        // Default: no custom search rendering (falls back to display)
        Ok(None)
    }

    /// Resolve markers in tile data (e.g., fetch reference labels).
    ///
    /// Some datatypes store IDs that need to be resolved to full objects
    /// with labels, etc. This method performs that resolution.
    ///
    /// # Arguments
    /// * `tile_data` - The tile data potentially containing markers
    /// * `language` - The language code for localization
    ///
    /// # Returns
    /// The resolved tile data (may be unchanged if no markers)
    fn resolve_markers(&self, tile_data: &Value, _language: &str) -> Result<Value, ExtensionError> {
        // Default: return unchanged
        Ok(tile_data.clone())
    }

    /// Get a description of this handler (for debugging/documentation).
    fn description(&self) -> &str {
        "Extension type handler"
    }
}

/// Registry for extension type handlers.
///
/// Stores handlers by datatype name. Thread-safe for concurrent access.
pub struct ExtensionTypeRegistry {
    handlers: HashMap<String, Arc<dyn ExtensionTypeHandler>>,
}

impl Default for ExtensionTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionTypeRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a datatype.
    ///
    /// # Arguments
    /// * `datatype` - The datatype name (e.g., "reference", "custom-type")
    /// * `handler` - The handler implementation
    pub fn register(
        &mut self,
        datatype: impl Into<String>,
        handler: Arc<dyn ExtensionTypeHandler>,
    ) {
        self.handlers.insert(datatype.into(), handler);
    }

    /// Unregister a handler for a datatype.
    ///
    /// # Returns
    /// The removed handler, if any
    pub fn unregister(&mut self, datatype: &str) -> Option<Arc<dyn ExtensionTypeHandler>> {
        self.handlers.remove(datatype)
    }

    /// Get the handler for a datatype.
    pub fn get(&self, datatype: &str) -> Option<&Arc<dyn ExtensionTypeHandler>> {
        self.handlers.get(datatype)
    }

    /// Check if a handler is registered for a datatype.
    pub fn has(&self, datatype: &str) -> bool {
        self.handlers.contains_key(datatype)
    }

    /// List all registered datatype names.
    pub fn list(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered handlers.
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Coerce a value using the registered handler, if any.
    ///
    /// # Returns
    /// - `Ok(Some(result))` if handler exists and coercion succeeded
    /// - `Ok(None)` if no handler registered for this datatype
    /// - `Err(e)` if handler exists but coercion failed
    pub fn coerce(
        &self,
        datatype: &str,
        value: &Value,
        config: Option<&Value>,
    ) -> Result<Option<CoercionResult>, ExtensionError> {
        match self.handlers.get(datatype) {
            Some(handler) if handler.capabilities().can_coerce => {
                Ok(Some(handler.coerce(value, config)?))
            }
            _ => Ok(None),
        }
    }

    /// Render a display value using the registered handler, if any.
    ///
    /// # Returns
    /// - `Ok(Some(string))` if handler rendered a display string
    /// - `Ok(None)` if no handler or handler returned None
    /// - `Err(e)` if handler failed
    pub fn render_display(
        &self,
        datatype: &str,
        tile_data: &Value,
        language: &str,
    ) -> Result<Option<String>, ExtensionError> {
        match self.handlers.get(datatype) {
            Some(handler) if handler.capabilities().can_render_display => {
                handler.render_display(tile_data, language)
            }
            _ => Ok(None),
        }
    }

    /// Render a search value using the registered handler, if any.
    ///
    /// # Returns
    /// - `Ok(Some(value))` if handler rendered a search value
    /// - `Ok(None)` if no handler or handler returned None
    /// - `Err(e)` if handler failed
    pub fn render_search(
        &self,
        datatype: &str,
        tile_data: &Value,
        language: &str,
    ) -> Result<Option<Value>, ExtensionError> {
        match self.handlers.get(datatype) {
            Some(handler) if handler.capabilities().can_render_search => {
                handler.render_search(tile_data, language)
            }
            _ => Ok(None),
        }
    }

    /// Resolve markers using the registered handler, if any.
    ///
    /// # Returns
    /// - `Ok(resolved)` if handler resolved markers (or passed through)
    /// - `Err(e)` if handler failed
    pub fn resolve_markers(
        &self,
        datatype: &str,
        tile_data: &Value,
        language: &str,
    ) -> Result<Value, ExtensionError> {
        match self.handlers.get(datatype) {
            Some(handler) if handler.capabilities().can_resolve_markers => {
                handler.resolve_markers(tile_data, language)
            }
            _ => Ok(tile_data.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler {
        caps: HandlerCapabilities,
    }

    impl ExtensionTypeHandler for TestHandler {
        fn capabilities(&self) -> HandlerCapabilities {
            self.caps.clone()
        }

        fn coerce(
            &self,
            value: &Value,
            _config: Option<&Value>,
        ) -> Result<CoercionResult, ExtensionError> {
            // Test: wrap value in an object
            let tile_data = serde_json::json!({ "wrapped": value });
            let display_value = serde_json::json!({ "display": value });
            Ok(CoercionResult::success(tile_data, display_value))
        }

        fn render_display(
            &self,
            tile_data: &Value,
            _language: &str,
        ) -> Result<Option<String>, ExtensionError> {
            Ok(Some(format!("Display: {}", tile_data)))
        }
    }

    #[test]
    fn test_registry_basic() {
        let mut registry = ExtensionTypeRegistry::new();
        assert!(registry.is_empty());

        registry.register(
            "test-type",
            Arc::new(TestHandler {
                caps: HandlerCapabilities::full(),
            }),
        );

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.has("test-type"));
        assert!(!registry.has("other-type"));
    }

    #[test]
    fn test_coerce_with_handler() {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register(
            "test-type",
            Arc::new(TestHandler {
                caps: HandlerCapabilities::coercion_only(),
            }),
        );

        let value = serde_json::json!("test-value");
        let result = registry.coerce("test-type", &value, None).unwrap();

        assert!(result.is_some());
        let coerced = result.unwrap();
        assert_eq!(coerced.tile_data["wrapped"], "test-value");
    }

    #[test]
    fn test_coerce_no_handler() {
        let registry = ExtensionTypeRegistry::new();
        let value = serde_json::json!("test-value");
        let result = registry.coerce("unknown-type", &value, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_render_display() {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register(
            "test-type",
            Arc::new(TestHandler {
                caps: HandlerCapabilities::display_only(),
            }),
        );

        let tile_data = serde_json::json!({"id": "123"});
        let result = registry
            .render_display("test-type", &tile_data, "en")
            .unwrap();

        assert!(result.is_some());
        assert!(result.unwrap().contains("Display:"));
    }

    #[test]
    fn test_capability_check() {
        let mut registry = ExtensionTypeRegistry::new();

        // Handler that only does display
        registry.register(
            "display-only",
            Arc::new(TestHandler {
                caps: HandlerCapabilities::display_only(),
            }),
        );

        let value = serde_json::json!("test");

        // Coercion should return None because handler doesn't support it
        let coerce_result = registry.coerce("display-only", &value, None).unwrap();
        assert!(coerce_result.is_none());

        // Display should work
        let display_result = registry
            .render_display("display-only", &value, "en")
            .unwrap();
        assert!(display_result.is_some());
    }

    #[test]
    fn test_unregister() {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register(
            "test-type",
            Arc::new(TestHandler {
                caps: HandlerCapabilities::full(),
            }),
        );

        assert!(registry.has("test-type"));

        let removed = registry.unregister("test-type");
        assert!(removed.is_some());
        assert!(!registry.has("test-type"));
    }

    #[test]
    fn test_list() {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register(
            "type-a",
            Arc::new(TestHandler {
                caps: HandlerCapabilities::full(),
            }),
        );
        registry.register(
            "type-b",
            Arc::new(TestHandler {
                caps: HandlerCapabilities::full(),
            }),
        );

        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"type-a"));
        assert!(list.contains(&"type-b"));
    }
}
