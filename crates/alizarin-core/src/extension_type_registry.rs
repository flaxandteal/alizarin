//! Unified extension type registry for custom datatype handlers.
//!
//! This module provides a platform-agnostic registry for extension type handlers
//! that can be used by both WASM and Python bindings. Extensions can register
//! handlers for custom datatypes. The trait exposes six capabilities:
//!
//! - **Coercion**: Transform input values during ETL (tree → tiles)
//! - **Display rendering**: Convert tile data to display strings
//! - **Search rendering**: Convert tile data to search-indexable JSON
//! - **Marker resolution**: Resolve references/lookups in tile data
//! - **Validation**: Check values against node constraints during tree → tiles
//! - **Index spec**: Report how the datatype's value is head-indexed
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
use crate::type_serialization::SerializationContext;

/// Describes what capabilities an extension handler provides.
///
/// Not all handlers need to implement all capabilities. For example,
/// a simple type might only need coercion, while a reference type
/// exercises most of them.
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
    /// Can produce an index spec (index class + keys) for head indexing.
    ///
    /// A handler that sets this owns the decision of how its datatype is
    /// head-indexed — the core emitter routes through `index_spec` instead
    /// of any hardcoded per-datatype logic.
    pub can_index: bool,
    /// Participates in value validation during tree→tiles. The validation pass
    /// only calls `validate` on handlers that set this, so a handler with no
    /// constraints need not be visited. Validation is Rust-native (the pass runs
    /// entirely on the core side — no per-value FFI round-trip to Python/JS).
    pub can_validate: bool,
}

impl HandlerCapabilities {
    /// Create capabilities for a coercion-only handler.
    pub fn coercion_only() -> Self {
        Self {
            can_coerce: true,
            can_render_display: false,
            can_render_search: false,
            can_resolve_markers: false,
            can_index: false,
            can_validate: false,
        }
    }

    /// Create capabilities for a display-only handler.
    pub fn display_only() -> Self {
        Self {
            can_coerce: false,
            can_render_display: true,
            can_render_search: false,
            can_resolve_markers: false,
            can_index: false,
            can_validate: false,
        }
    }

    /// Create capabilities for a full handler (all capabilities).
    pub fn full() -> Self {
        Self {
            can_coerce: true,
            can_render_display: true,
            can_render_search: false,
            can_resolve_markers: true,
            can_index: true,
            can_validate: true,
        }
    }
}

/// How a datatype's value is head-indexed.
///
/// This is the domain-agnostic shape the emitter routes on: a concept-like
/// datatype is hierarchical (its keys sit inside DFS intervals); a resource
/// link is a coarse target; everything else is detail-only (lives in the
/// tile chunks, not the head). The `collection` on the hierarchical variant
/// lets a handler carry the vocabulary/collection its keys belong to —
/// the handler owns that resolution, core does not hardcode it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IndexClass {
    /// Concept-like: keys are hierarchy members (DFS-interval indexed),
    /// optionally scoped to a named collection/controlled list.
    ConceptHierarchical { collection: Option<String> },
    /// Resource link: keys are coarse link targets.
    Link,
    /// Ordered scalar: keys are values with a total order, head-indexed as a
    /// sortable integer so the head can answer RANGE queries (`BETWEEN lo AND
    /// hi`). The class is deliberately OPAQUE about what the value means — the
    /// datatype decides the quantizer (date → days-from-civil), and the emitter
    /// owns that encoding. Keys here are the RAW values (e.g. the ISO date
    /// string); the emitter quantizes them.
    Ordered,
    /// Spatial geometry, head-indexed as its axis-aligned BOUNDING BOX (A8.2).
    /// The head stores `(min_lng, min_lat, max_lng, max_lat)`; a bbox query
    /// matches on box OVERLAP, which is a SUPERSET of true `sfIntersects` — so
    /// the coarse result never misses a geometry, and the client verifies exact
    /// intersection on the hydrated tile. NOT a centroid (a polygon whose
    /// centroid is outside a query box can still intersect it — centroid would
    /// drop it). Keys are the RAW GeoJSON; the emitter extracts the bbox.
    SpatialBbox,
    /// Not head-indexed: the value lives only in the tile chunks.
    DetailOnly,
}

/// An index spec: the index class plus the extracted keys (concept ids,
/// link target ids, …) for a single tile value. `keys` is empty for
/// `DetailOnly` and may be empty for the other classes when the value
/// carries no indexable ids.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IndexSpec {
    pub class: IndexClass,
    pub keys: Vec<String>,
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

/// Result of validating a single value: whether it is acceptable, plus any
/// human-readable messages. `errors` make a value invalid; `warnings` are
/// advisory (a value can be `valid` while carrying warnings).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// A passing result with no messages.
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// A failing result carrying a single error message.
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![error.into()],
            warnings: Vec::new(),
        }
    }

    /// Build from a set of error messages — valid iff `errors` is empty.
    pub fn from_errors(errors: Vec<String>) -> Self {
        Self {
            valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
        }
    }

    /// Attach a warning (does not affect validity).
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

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
    /// * `tile_data` - The tile data (may be a resolved object, or a bare id /
    ///   `__needs_rdm_lookup` marker that this handler resolves via `ctx`)
    /// * `language` - The language code for localization
    /// * `ctx` - Optional serialization context, carrying the `ExternalResolver`
    ///   and per-node config. This is what lets an extension type resolve its own
    ///   ids to labels — symmetric with how built-in `concept` serialization reads
    ///   `ctx.external_resolver`. `None` in contexts that render pre-resolved data.
    ///
    /// # Returns
    /// `Some(String)` if rendered, `None` to use default rendering
    fn render_display(
        &self,
        _tile_data: &Value,
        _language: &str,
        _ctx: Option<&SerializationContext>,
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

    /// Produce an index spec (index class + keys) for a tile value.
    ///
    /// The handler owns the mapping from its datatype's tile value to a
    /// domain-agnostic [`IndexSpec`] — including resolving which
    /// collection/vocabulary its concept keys belong to from `config`.
    /// This is why `config` is threaded here: collection resolution is the
    /// handler's job, not the core emitter's.
    ///
    /// # Arguments
    /// * `tile_data` - The tile value to extract index keys from
    /// * `config` - Optional node configuration (e.g., collection id)
    ///
    /// # Returns
    /// `Some(IndexSpec)` if this handler indexes the value, `None` to defer
    /// to the caller's built-in handling.
    fn index_spec(
        &self,
        _tile_data: &Value,
        _config: Option<&Value>,
    ) -> Result<Option<IndexSpec>, ExtensionError> {
        // Default: no index spec (handler does not participate in indexing)
        Ok(None)
    }

    /// Validate a single value for this datatype during tree→tiles.
    ///
    /// Mirrors [`coerce`](Self::coerce): `value` is the (coerced) tile value and
    /// `config` is the node configuration (required flags, regex, min/max,
    /// allowed collections, …). RDM/graph context is reached the same way
    /// `coerce` reaches it (the global RDM cache), so no extra params are needed.
    /// Runs entirely on the core side — no per-value FFI round-trip.
    ///
    /// # Returns
    /// A [`ValidationResult`]. The default accepts everything (a handler with no
    /// constraints); `Err` is reserved for a handler that could not run at all.
    fn validate(
        &self,
        _value: &Value,
        _config: Option<&Value>,
    ) -> Result<ValidationResult, ExtensionError> {
        Ok(ValidationResult::valid())
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
#[derive(Clone)]
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
        ctx: Option<&SerializationContext>,
    ) -> Result<Option<String>, ExtensionError> {
        match self.handlers.get(datatype) {
            Some(handler) if handler.capabilities().can_render_display => {
                handler.render_display(tile_data, language, ctx)
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

    /// Produce an index spec using the registered handler, if any.
    ///
    /// Capability-gated on `can_index`. A handler that fails is treated as
    /// "no spec" (the caller falls back to its built-in handling) — index
    /// derivation must never abort an emit.
    ///
    /// # Returns
    /// - `Some(spec)` if a handler is registered, capable, and returned one
    /// - `None` if no handler, `!can_index`, the handler returned `None`,
    ///   or the handler errored
    pub fn index_spec(
        &self,
        datatype: &str,
        tile_data: &Value,
        config: Option<&Value>,
    ) -> Option<IndexSpec> {
        match self.handlers.get(datatype) {
            Some(handler) if handler.capabilities().can_index => {
                handler.index_spec(tile_data, config).ok().flatten()
            }
            _ => None,
        }
    }

    /// Validate a value via the registered handler, if it participates.
    ///
    /// `Ok(None)` — no handler for `datatype`, or it does not set `can_validate`
    /// (nothing to check ⇒ implicitly valid). `Ok(Some(result))` — the handler
    /// ran; inspect [`ValidationResult::valid`]. `Err` — the handler could not
    /// run at all (distinct from an invalid value).
    pub fn validate(
        &self,
        datatype: &str,
        value: &Value,
        config: Option<&Value>,
    ) -> Result<Option<ValidationResult>, ExtensionError> {
        match self.handlers.get(datatype) {
            Some(handler) if handler.capabilities().can_validate => {
                Ok(Some(handler.validate(value, config)?))
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
            _ctx: Option<&SerializationContext>,
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
            .render_display("test-type", &tile_data, "en", None)
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
            .render_display("display-only", &value, "en", None)
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
