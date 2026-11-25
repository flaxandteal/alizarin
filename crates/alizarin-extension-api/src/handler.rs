//! What an extension's type handler hands the host, as data: errors,
//! validation results and index specs. alizarin-core re-exports these from
//! `extension_type_registry`, so host and extension name the same types;
//! across the C ABI they still cross as JSON.

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
