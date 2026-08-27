//! Alizarin Extension API
//!
//! This crate defines the C ABI types and function signatures for Alizarin extensions.
//! Extensions can implement custom datatype handlers and graph mutations that are called
//! directly from Rust without crossing the Python/WASM boundary at runtime.
//!
//! # Type Coercion Handlers
//!
//! Extensions implement coercion handlers and register them via PyCapsule at import time.
//! The handlers are then called directly from Rust during tile processing.
//!
//! # Display Rendering
//!
//! Extensions can optionally implement a display renderer to convert resolved values
//! to human-readable strings. This is used by `toDisplayJson()` for export/indexing.
//!
//! # Custom Mutations
//!
//! Extensions can define custom graph mutations (e.g., `clm.reference_change_collection`)
//! that integrate with the graph mutator system. Use `MutationHandlerInfo` to register
//! mutation handlers that receive graph and params as JSON and return a mutated graph.

use std::ffi::c_void;

/// Version of the extension C ABI defined by this crate. Bump on any change to
/// the shared struct layouts or function signatures below. Extensions stamp
/// their compiled fingerprint into [`TypeHandlerInfo`]; the host refuses to load
/// an extension whose [`AbiFingerprint`] does not match its own.
pub const ABI_VERSION: u32 = 1;

/// A compatibility token an extension embeds so the host can confirm — once, at
/// load — that both sides share the same ABI before any raw pointers cross the
/// boundary. `value_size` guards against `serde_json::Value` layout drift even
/// if `ABI_VERSION` was not bumped.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiFingerprint {
    pub version: u32,
    pub value_size: usize,
    pub handler_info_size: usize,
}

/// The ABI fingerprint of the crate the caller was compiled against.
pub fn abi_fingerprint() -> AbiFingerprint {
    AbiFingerprint {
        version: ABI_VERSION,
        value_size: std::mem::size_of::<serde_json::Value>(),
        handler_info_size: std::mem::size_of::<TypeHandlerInfo>(),
    }
}

/// Result of a coercion operation
///
/// All pointers are owned by the extension and must be freed via `free_fn`.
#[repr(C)]
pub struct CoerceResult {
    /// JSON string output (null if error)
    pub json_ptr: *mut u8,
    pub json_len: usize,

    /// Resolved object JSON (for ViewModel constructor, may be same as json_ptr)
    pub resolved_ptr: *mut u8,
    pub resolved_len: usize,

    /// Error message (null if success)
    pub error_ptr: *mut u8,
    pub error_len: usize,
}

impl CoerceResult {
    /// Create a successful result with tile data and resolved object
    pub fn success(tile_json: Vec<u8>, resolved_json: Vec<u8>) -> Self {
        let json_len = tile_json.len();
        let resolved_len = resolved_json.len();

        let json_ptr = Box::into_raw(tile_json.into_boxed_slice()) as *mut u8;
        let resolved_ptr = Box::into_raw(resolved_json.into_boxed_slice()) as *mut u8;

        CoerceResult {
            json_ptr,
            json_len,
            resolved_ptr,
            resolved_len,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create a successful result where tile data and resolved are the same
    pub fn success_same(json: Vec<u8>) -> Self {
        let len = json.len();
        let ptr = Box::into_raw(json.into_boxed_slice()) as *mut u8;

        CoerceResult {
            json_ptr: ptr,
            json_len: len,
            resolved_ptr: ptr,
            resolved_len: len,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create an error result
    pub fn error(message: String) -> Self {
        let bytes = message.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        CoerceResult {
            json_ptr: std::ptr::null_mut(),
            json_len: 0,
            resolved_ptr: std::ptr::null_mut(),
            resolved_len: 0,
            error_ptr: ptr,
            error_len: len,
        }
    }

    /// Check if this result is an error
    pub fn is_error(&self) -> bool {
        !self.error_ptr.is_null()
    }
}

/// Function signature for coercion handlers
///
/// # Arguments
/// * `value_ptr` - JSON string of the value to coerce
/// * `value_len` - Length of value JSON
/// * `config_ptr` - JSON string of node config (may be null)
/// * `config_len` - Length of config JSON
///
/// # Returns
/// * `CoerceResult` - The coerced value and/or error
pub type CoerceFn = unsafe extern "C" fn(
    value_ptr: *const u8,
    value_len: usize,
    config_ptr: *const u8,
    config_len: usize,
) -> CoerceResult;

/// Function signature for freeing CoerceResult
pub type FreeFn = unsafe extern "C" fn(result: CoerceResult);

// =============================================================================
// Display Rendering API
// =============================================================================

/// Result of a display render operation
///
/// Returns the human-readable display string for a resolved value.
/// All pointers are owned by the extension and must be freed via `free_display_fn`.
#[repr(C)]
pub struct RenderDisplayResult {
    /// Display string (null if error)
    pub display_ptr: *mut u8,
    pub display_len: usize,

    /// Error message (null if success)
    pub error_ptr: *mut u8,
    pub error_len: usize,
}

impl RenderDisplayResult {
    /// Create a successful result with display string
    pub fn success(display: String) -> Self {
        let bytes = display.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        RenderDisplayResult {
            display_ptr: ptr,
            display_len: len,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create an error result
    pub fn error(message: String) -> Self {
        let bytes = message.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        RenderDisplayResult {
            display_ptr: std::ptr::null_mut(),
            display_len: 0,
            error_ptr: ptr,
            error_len: len,
        }
    }

    /// Check if this result is an error
    pub fn is_error(&self) -> bool {
        !self.error_ptr.is_null()
    }
}

/// Function signature for display rendering
///
/// Takes a resolved value (from CoerceResult.resolved_ptr) and returns
/// a human-readable display string.
///
/// # Arguments
/// * `resolved_ptr` - JSON string of the resolved value
/// * `resolved_len` - Length of resolved JSON
/// * `lang_ptr` - Language code (e.g., "en")
/// * `lang_len` - Length of language code
///
/// # Returns
/// * `RenderDisplayResult` - The display string and/or error
pub type RenderDisplayFn = unsafe extern "C" fn(
    resolved_ptr: *const u8,
    resolved_len: usize,
    lang_ptr: *const u8,
    lang_len: usize,
) -> RenderDisplayResult;

/// Function signature for freeing RenderDisplayResult
pub type FreeDisplayFn = unsafe extern "C" fn(result: RenderDisplayResult);

// =============================================================================
// Marker Resolution API
// =============================================================================

/// Result of a marker resolution operation
///
/// Extensions use this to resolve `__needs_rdm_lookup` and similar markers
/// to full resolved values (e.g., StaticReference with embedded labels).
#[repr(C)]
pub struct ResolveMarkersResult {
    /// Resolved JSON (null if error or no change)
    pub json_ptr: *mut u8,
    pub json_len: usize,

    /// Whether the value was modified (false if no markers found)
    pub modified: bool,

    /// Error message (null if success)
    pub error_ptr: *mut u8,
    pub error_len: usize,
}

impl ResolveMarkersResult {
    /// Create a successful result with resolved value
    pub fn success(json: Vec<u8>) -> Self {
        let len = json.len();
        let ptr = Box::into_raw(json.into_boxed_slice()) as *mut u8;

        ResolveMarkersResult {
            json_ptr: ptr,
            json_len: len,
            modified: true,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create a result indicating no modification was needed
    pub fn unchanged() -> Self {
        ResolveMarkersResult {
            json_ptr: std::ptr::null_mut(),
            json_len: 0,
            modified: false,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create an error result
    pub fn error(message: String) -> Self {
        let bytes = message.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        ResolveMarkersResult {
            json_ptr: std::ptr::null_mut(),
            json_len: 0,
            modified: false,
            error_ptr: ptr,
            error_len: len,
        }
    }

    /// Check if this result is an error
    pub fn is_error(&self) -> bool {
        !self.error_ptr.is_null()
    }
}

/// Result of a validation operation — a serialized `ValidationResult`
/// (`{valid, errors, warnings}`).
///
/// The value being validated is passed to the extension **by pointer**
/// (`*const serde_json::Value`), so nothing is serialized on the hot path — only
/// this small result crosses back. Owned by the extension; freed via the
/// handler's `free_validate_fn`.
#[repr(C)]
pub struct ValidateResult {
    /// Serialized `{valid, errors, warnings}` JSON (null on error).
    pub json_ptr: *mut u8,
    pub json_len: usize,
    /// Extension-error message (null on success).
    pub error_ptr: *mut u8,
    pub error_len: usize,
}

impl ValidateResult {
    /// A successful validation carrying the serialized `ValidationResult`.
    pub fn success(json: Vec<u8>) -> Self {
        let len = json.len();
        let ptr = Box::into_raw(json.into_boxed_slice()) as *mut u8;
        ValidateResult {
            json_ptr: ptr,
            json_len: len,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// A handler-level error (distinct from a validation failure).
    pub fn error(message: String) -> Self {
        let bytes = message.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;
        ValidateResult {
            json_ptr: std::ptr::null_mut(),
            json_len: 0,
            error_ptr: ptr,
            error_len: len,
        }
    }

    pub fn is_error(&self) -> bool {
        !self.error_ptr.is_null()
    }
}

/// Validate a tile value. `value_ptr` is `*const serde_json::Value` — cast it
/// back with `&*(value_ptr as *const serde_json::Value)`; sound because the host
/// confirmed the [`AbiFingerprint`] at load. `config_ptr` is the node config as
/// `*const serde_json::Value`, or null for no config.
pub type ValidateFn =
    unsafe extern "C" fn(value_ptr: *const c_void, config_ptr: *const c_void) -> ValidateResult;

/// Free a [`ValidateResult`] allocated by a [`ValidateFn`].
pub type FreeValidateFn = unsafe extern "C" fn(result: ValidateResult);

/// Callback to lookup a concept by ID in the RDM cache
///
/// Returns true if concept was found, false otherwise.
/// If found, writes concept JSON to output pointers.
pub type ConceptLookupByIdFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
    concept_id_ptr: *const u8,
    concept_id_len: usize,
    // Output
    concept_json_ptr: *mut *mut u8,
    concept_json_len: *mut usize,
) -> bool;

/// Callback to lookup a concept by label in the RDM cache
///
/// Returns true if concept was found, false otherwise.
/// If found, writes concept JSON to output pointers.
pub type ConceptLookupByLabelFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
    label_ptr: *const u8,
    label_len: usize,
    // Output
    concept_json_ptr: *mut *mut u8,
    concept_json_len: *mut usize,
) -> bool;

/// Callback to free concept JSON returned by lookup functions
pub type FreeConceptJsonFn = unsafe extern "C" fn(ptr: *mut u8, len: usize);

/// Callback to check if a collection exists in the RDM cache
///
/// Returns true if the collection is in the cache, false otherwise.
/// Use this to check collection existence before attempting lookups,
/// enabling clear error messages for missing collections.
pub type HasCollectionFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
) -> bool;

/// Function signature for marker resolution
///
/// Extensions implement this to resolve markers (e.g., `__needs_rdm_lookup`)
/// in coerced values to fully resolved objects.
///
/// # Arguments
/// * `value_ptr` - JSON string of the value (may contain markers)
/// * `value_len` - Length of value JSON
/// * `config_ptr` - JSON string of node config (contains collection ID)
/// * `config_len` - Length of config JSON
/// * `has_collection` - Callback to check if collection is in cache
/// * `lookup_by_id` - Callback to lookup concept by ID
/// * `lookup_by_label` - Callback to lookup concept by label
/// * `free_concept_json` - Callback to free concept JSON from lookups
/// * `lookup_user_data` - Opaque pointer passed to lookup callbacks
///
/// # Returns
/// * `ResolveMarkersResult` - The resolved value or unchanged/error
pub type ResolveMarkersFn = unsafe extern "C" fn(
    value_ptr: *const u8,
    value_len: usize,
    config_ptr: *const u8,
    config_len: usize,
    has_collection: HasCollectionFn,
    lookup_by_id: ConceptLookupByIdFn,
    lookup_by_label: ConceptLookupByLabelFn,
    free_concept_json: FreeConceptJsonFn,
    lookup_user_data: *mut c_void,
) -> ResolveMarkersResult;

/// Function signature for freeing ResolveMarkersResult
pub type FreeResolveMarkersFn = unsafe extern "C" fn(result: ResolveMarkersResult);

/// Standard free function for ResolveMarkersResult
///
/// # Safety
/// This function must only be called once for a given `ResolveMarkersResult`.
/// The pointers in the result must have been allocated by this library using
/// `Box::into_raw(slice.into_boxed_slice())`. Calling this function multiple
/// times with the same result or with results from other sources will cause
/// undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn alizarin_free_resolve_markers_result(result: ResolveMarkersResult) {
    if !result.json_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.json_ptr,
            result.json_len,
        ));
    }
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

/// Standard free function for RenderDisplayResult
///
/// # Safety
/// This function must only be called once for a given `RenderDisplayResult`.
/// The pointers in the result must have been allocated by this library using
/// `Box::into_raw(slice.into_boxed_slice())`. Calling this function multiple
/// times with the same result or with results from other sources will cause
/// undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn alizarin_free_render_display_result(result: RenderDisplayResult) {
    if !result.display_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.display_ptr,
            result.display_len,
        ));
    }
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

// =============================================================================
// Handler Registration
// =============================================================================

/// Handler registration info passed via PyCapsule
///
/// Extensions must provide coerce_fn and free_fn.
/// render_display_fn, free_display_fn, resolve_markers_fn, and free_resolve_markers_fn
/// are optional (null if not implemented).
#[repr(C)]
pub struct TypeHandlerInfo {
    /// Datatype name (e.g., "reference")
    pub type_name_ptr: *const u8,
    pub type_name_len: usize,

    /// Coercion function. `None` for a validate-only handler — the host then
    /// leaves coercion of this datatype to core.
    pub coerce_fn: Option<CoerceFn>,

    /// Free function for CoerceResult (`None` iff `coerce_fn` is `None`).
    pub free_fn: Option<FreeFn>,

    /// Display render function (optional - null if not implemented)
    /// Used by toDisplayJson() to get human-readable strings
    pub render_display_fn: Option<RenderDisplayFn>,

    /// Free function for RenderDisplayResult (optional - null if render_display_fn is null)
    pub free_display_fn: Option<FreeDisplayFn>,

    /// Marker resolution function (optional - null if not implemented)
    /// Called after coercion to resolve __needs_rdm_lookup and similar markers
    pub resolve_markers_fn: Option<ResolveMarkersFn>,

    /// Free function for ResolveMarkersResult (optional - null if resolve_markers_fn is null)
    pub free_resolve_markers_fn: Option<FreeResolveMarkersFn>,

    /// Validation function (optional). The value is passed **by pointer**, so no
    /// serialization happens on the hot path.
    pub validate_fn: Option<ValidateFn>,

    /// Free function for ValidateResult (optional - null if validate_fn is null)
    pub free_validate_fn: Option<FreeValidateFn>,

    /// ABI fingerprint of the crate the extension was built against. The host
    /// checks this at load and refuses a mismatch before any pointer crosses.
    pub abi: AbiFingerprint,

    /// Opaque data pointer (for extension use)
    pub user_data: *mut c_void,
}

/// Helper to create a TypeHandlerInfo
impl TypeHandlerInfo {
    /// Create a handler without display rendering or marker resolution support
    pub fn new(type_name: &'static str, coerce_fn: CoerceFn, free_fn: FreeFn) -> Self {
        TypeHandlerInfo {
            type_name_ptr: type_name.as_ptr(),
            type_name_len: type_name.len(),
            coerce_fn: Some(coerce_fn),
            free_fn: Some(free_fn),
            render_display_fn: None,
            free_display_fn: None,
            resolve_markers_fn: None,
            free_resolve_markers_fn: None,
            validate_fn: None,
            free_validate_fn: None,
            abi: abi_fingerprint(),
            user_data: std::ptr::null_mut(),
        }
    }

    /// Create a validate-only handler: no coercion (core keeps that), just a
    /// pointer-based validation function.
    pub fn new_validating(
        type_name: &'static str,
        validate_fn: ValidateFn,
        free_validate_fn: FreeValidateFn,
    ) -> Self {
        TypeHandlerInfo {
            type_name_ptr: type_name.as_ptr(),
            type_name_len: type_name.len(),
            coerce_fn: None,
            free_fn: None,
            render_display_fn: None,
            free_display_fn: None,
            resolve_markers_fn: None,
            free_resolve_markers_fn: None,
            validate_fn: Some(validate_fn),
            free_validate_fn: Some(free_validate_fn),
            abi: abi_fingerprint(),
            user_data: std::ptr::null_mut(),
        }
    }

    /// Create a handler with display rendering support
    pub fn with_display_renderer(
        type_name: &'static str,
        coerce_fn: CoerceFn,
        free_fn: FreeFn,
        render_display_fn: RenderDisplayFn,
        free_display_fn: FreeDisplayFn,
    ) -> Self {
        TypeHandlerInfo {
            type_name_ptr: type_name.as_ptr(),
            type_name_len: type_name.len(),
            coerce_fn: Some(coerce_fn),
            free_fn: Some(free_fn),
            render_display_fn: Some(render_display_fn),
            free_display_fn: Some(free_display_fn),
            resolve_markers_fn: None,
            free_resolve_markers_fn: None,
            validate_fn: None,
            free_validate_fn: None,
            abi: abi_fingerprint(),
            user_data: std::ptr::null_mut(),
        }
    }

    /// Create a handler with display rendering and marker resolution support
    pub fn with_display_and_markers(
        type_name: &'static str,
        coerce_fn: CoerceFn,
        free_fn: FreeFn,
        render_display_fn: RenderDisplayFn,
        free_display_fn: FreeDisplayFn,
        resolve_markers_fn: ResolveMarkersFn,
        free_resolve_markers_fn: FreeResolveMarkersFn,
    ) -> Self {
        TypeHandlerInfo {
            type_name_ptr: type_name.as_ptr(),
            type_name_len: type_name.len(),
            coerce_fn: Some(coerce_fn),
            free_fn: Some(free_fn),
            render_display_fn: Some(render_display_fn),
            free_display_fn: Some(free_display_fn),
            resolve_markers_fn: Some(resolve_markers_fn),
            free_resolve_markers_fn: Some(free_resolve_markers_fn),
            validate_fn: None,
            free_validate_fn: None,
            abi: abi_fingerprint(),
            user_data: std::ptr::null_mut(),
        }
    }

    /// Check if this handler supports display rendering
    pub fn has_display_renderer(&self) -> bool {
        self.render_display_fn.is_some()
    }

    /// Check if this handler supports marker resolution
    pub fn has_marker_resolver(&self) -> bool {
        self.resolve_markers_fn.is_some()
    }
}

/// Standard free function for CoerceResult
///
/// Extensions can use this if they allocate results using the standard methods.
///
/// # Safety
/// This function must only be called once for a given `CoerceResult`.
/// The pointers in the result must have been allocated by this library using
/// `Box::into_raw(slice.into_boxed_slice())`. Calling this function multiple
/// times with the same result or with results from other sources will cause
/// undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn alizarin_free_coerce_result(result: CoerceResult) {
    if !result.json_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.json_ptr,
            result.json_len,
        ));
    }
    // Only free resolved if different from json
    if !result.resolved_ptr.is_null() && result.resolved_ptr != result.json_ptr {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.resolved_ptr,
            result.resolved_len,
        ));
    }
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

// =============================================================================
// Extension Mutation API
// =============================================================================

/// Result of a mutation operation
///
/// All pointers are owned by the extension and must be freed via `free_mutation_fn`.
#[repr(C)]
pub struct MutationResult {
    /// Success flag
    pub success: bool,

    /// Error message (null if success)
    pub error_ptr: *mut u8,
    pub error_len: usize,
}

impl MutationResult {
    /// Create a successful result
    pub fn success() -> Self {
        MutationResult {
            success: true,
            error_ptr: std::ptr::null_mut(),
            error_len: 0,
        }
    }

    /// Create an error result
    pub fn error(message: String) -> Self {
        let bytes = message.into_bytes();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

        MutationResult {
            success: false,
            error_ptr: ptr,
            error_len: len,
        }
    }

    /// Check if this result is an error
    pub fn is_error(&self) -> bool {
        !self.success
    }
}

/// Conformance level for mutations (must match alizarin-core's MutationConformance)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationConformanceLevel {
    /// Valid for both branches and resource models
    AlwaysConformant = 0,
    /// Valid only for branches (isresource=false)
    BranchConformant = 1,
    /// Valid only for resource models (isresource=true)
    ModelConformant = 2,
    /// Not conformant with standard workflows
    NonConformant = 3,
}

/// Function signature for mutation handlers
///
/// # Arguments
/// * `graph_ptr` - JSON string of the graph to mutate
/// * `graph_len` - Length of graph JSON
/// * `params_ptr` - JSON string of mutation parameters
/// * `params_len` - Length of params JSON
/// * `output_ptr` - Output pointer for mutated graph JSON (caller allocates)
/// * `output_len` - Output pointer for mutated graph JSON length
///
/// # Returns
/// * `MutationResult` - Success or error
///
/// # Note
/// The handler receives the graph as JSON, applies the mutation, and writes
/// the mutated graph JSON back via output_ptr. The caller is responsible for
/// freeing the output.
pub type MutationHandlerFn = unsafe extern "C" fn(
    graph_ptr: *const u8,
    graph_len: usize,
    params_ptr: *const u8,
    params_len: usize,
    output_ptr: *mut *mut u8,
    output_len: *mut usize,
) -> MutationResult;

/// Function signature for freeing MutationResult
pub type FreeMutationResultFn = unsafe extern "C" fn(result: MutationResult);

/// Function signature for freeing mutation output
pub type FreeMutationOutputFn = unsafe extern "C" fn(ptr: *mut u8, len: usize);

/// Mutation handler registration info passed via PyCapsule
#[repr(C)]
pub struct MutationHandlerInfo {
    /// Mutation name (e.g., "clm.reference_change_collection")
    pub mutation_name_ptr: *const u8,
    pub mutation_name_len: usize,

    /// Handler function (required)
    pub handler_fn: MutationHandlerFn,

    /// Free function for MutationResult (required)
    pub free_result_fn: FreeMutationResultFn,

    /// Free function for mutation output (required)
    pub free_output_fn: FreeMutationOutputFn,

    /// Default conformance level for this mutation
    pub conformance: MutationConformanceLevel,

    /// Opaque data pointer (for extension use)
    pub user_data: *mut c_void,
}

impl MutationHandlerInfo {
    /// Create a new mutation handler info
    pub fn new(
        mutation_name: &'static str,
        handler_fn: MutationHandlerFn,
        free_result_fn: FreeMutationResultFn,
        free_output_fn: FreeMutationOutputFn,
        conformance: MutationConformanceLevel,
    ) -> Self {
        MutationHandlerInfo {
            mutation_name_ptr: mutation_name.as_ptr(),
            mutation_name_len: mutation_name.len(),
            handler_fn,
            free_result_fn,
            free_output_fn,
            conformance,
            user_data: std::ptr::null_mut(),
        }
    }
}

/// Standard free function for MutationResult
///
/// # Safety
/// This function must only be called once for a given `MutationResult`.
/// The error pointer in the result must have been allocated by this library using
/// `Box::into_raw(slice.into_boxed_slice())`. Calling this function multiple
/// times with the same result or with results from other sources will cause
/// undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn alizarin_free_mutation_result(result: MutationResult) {
    if !result.error_ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            result.error_ptr,
            result.error_len,
        ));
    }
}

/// Standard free function for mutation output
///
/// # Safety
/// This function must only be called once for a given output pointer/length pair.
/// The pointer must have been allocated by this library using
/// `Box::into_raw(slice.into_boxed_slice())`. Calling this function multiple
/// times with the same pointer or with pointers from other sources will cause
/// undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn alizarin_free_mutation_output(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coerce_result_success() {
        let result =
            CoerceResult::success(b"\"tile_data\"".to_vec(), b"{\"resolved\": true}".to_vec());
        assert!(!result.is_error());
        assert!(!result.json_ptr.is_null());
        assert!(!result.resolved_ptr.is_null());

        // Clean up
        unsafe { alizarin_free_coerce_result(result) };
    }

    #[test]
    fn test_coerce_result_error() {
        let result = CoerceResult::error("Test error".to_string());
        assert!(result.is_error());
        assert!(result.json_ptr.is_null());
        assert!(!result.error_ptr.is_null());

        // Clean up
        unsafe { alizarin_free_coerce_result(result) };
    }

    #[test]
    fn test_mutation_result_success() {
        let result = MutationResult::success();
        assert!(!result.is_error());
        assert!(result.success);
        assert!(result.error_ptr.is_null());
    }

    #[test]
    fn test_mutation_result_error() {
        let result = MutationResult::error("Mutation failed".to_string());
        assert!(result.is_error());
        assert!(!result.success);
        assert!(!result.error_ptr.is_null());

        // Clean up
        unsafe { alizarin_free_mutation_result(result) };
    }

    #[test]
    fn test_mutation_conformance_levels() {
        assert_eq!(MutationConformanceLevel::AlwaysConformant as u8, 0);
        assert_eq!(MutationConformanceLevel::BranchConformant as u8, 1);
        assert_eq!(MutationConformanceLevel::ModelConformant as u8, 2);
        assert_eq!(MutationConformanceLevel::NonConformant as u8, 3);
    }
}
