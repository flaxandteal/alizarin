/// Python bindings for Alizarin
///
/// This provides Python bindings for:
/// - Tree conversion (tiles ↔ JSON)
/// - Resource instance wrapper with three-layer caching
/// - Resource model wrapper with pruning
/// - Graph management utilities
/// - Extension type handler registration
/// - Node configuration management for type coercion
use pyo3::prelude::*;

mod graph_mutator_py;
mod instance_wrapper_py;
mod node_config_py;
mod pseudo_value_py;
mod python_json;
mod rdm_cache_py;
mod skos_py;
mod type_coercion_py;
use pyo3::types::PyCapsule;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use alizarin_extension_api::{
    CoerceFn, FreeDisplayFn, FreeFn, FreeResolveMarkersFn, RenderDisplayFn, ResolveMarkersFn,
    TypeHandlerInfo,
};
use std::ffi::c_void;

// Extension type registry (unified infrastructure)
use alizarin_core::{
    ExtensionError, ExtensionTypeHandler, ExtensionTypeRegistry, HandlerCapabilities,
};

lazy_static::lazy_static! {
    /// Global registry of extension type handlers
    static ref TYPE_HANDLERS: RwLock<HashMap<String, RegisteredHandler>> =
        RwLock::new(HashMap::new());
}

/// Build a fresh extension registry from dynamically registered PyCapsule handlers.
///
/// Extensions register via PyCapsule (e.g. `import alizarin_clm` registers the
/// "reference" handler). This function wraps those C ABI handlers as
/// `PyExtensionTypeHandler` instances in an `ExtensionTypeRegistry`.
fn build_extension_registry() -> ExtensionTypeRegistry {
    let mut registry = ExtensionTypeRegistry::new();

    let handlers = TYPE_HANDLERS.read().unwrap();
    for type_name in handlers.keys() {
        registry.register(
            type_name.clone(),
            Arc::new(PyExtensionTypeHandler::new(type_name.clone())),
        );
    }

    registry
}

/// Internal representation of a registered handler
struct RegisteredHandler {
    coerce_fn: CoerceFn,
    free_fn: FreeFn,
    /// Optional display renderer (for toDisplayJson support)
    render_display_fn: Option<RenderDisplayFn>,
    free_display_fn: Option<FreeDisplayFn>,
    /// Optional marker resolver (for resolving __needs_rdm_lookup etc.)
    resolve_markers_fn: Option<ResolveMarkersFn>,
    free_resolve_markers_fn: Option<FreeResolveMarkersFn>,
}

// =============================================================================
// ExtensionTypeHandler Implementation for Python
// =============================================================================

/// Wrapper that implements ExtensionTypeHandler using C ABI callbacks.
///
/// This allows Python extension handlers to be used with the unified
/// ExtensionTypeRegistry from alizarin-core.
pub struct PyExtensionTypeHandler {
    type_name: String,
}

impl PyExtensionTypeHandler {
    pub fn new(type_name: String) -> Self {
        Self { type_name }
    }
}

// The C ABI callbacks are thread-safe (they're compiled code)
unsafe impl Send for PyExtensionTypeHandler {}
unsafe impl Sync for PyExtensionTypeHandler {}

impl ExtensionTypeHandler for PyExtensionTypeHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        let handlers = TYPE_HANDLERS.read().unwrap();
        if let Some(handler) = handlers.get(&self.type_name) {
            HandlerCapabilities {
                can_coerce: true, // All handlers have coerce_fn
                can_render_display: handler.render_display_fn.is_some(),
                can_render_search: false,
                can_resolve_markers: handler.resolve_markers_fn.is_some(),
            }
        } else {
            HandlerCapabilities::default()
        }
    }

    fn coerce(
        &self,
        value: &serde_json::Value,
        config: Option<&serde_json::Value>,
    ) -> Result<alizarin_core::CoercionResult, ExtensionError> {
        let handlers = TYPE_HANDLERS.read().unwrap();

        if let Some(handler) = handlers.get(&self.type_name) {
            let value_json = serde_json::to_string(value)
                .map_err(|e| ExtensionError::new(format!("Failed to serialize value: {}", e)))?;

            let config_json = match config {
                Some(c) => serde_json::to_string(c).map_err(|e| {
                    ExtensionError::new(format!("Failed to serialize config: {}", e))
                })?,
                None => "null".to_string(),
            };

            // SAFETY: Calling extension handler via FFI function pointers registered through
            // PyCapsule. The handler owns the result memory and we free it via handler.free_fn
            // before the result goes out of scope. Pointers are valid UTF-8 JSON produced by
            // the extension's Rust code (same process, no IPC).
            unsafe {
                let result = (handler.coerce_fn)(
                    value_json.as_ptr(),
                    value_json.len(),
                    config_json.as_ptr(),
                    config_json.len(),
                );

                if result.error_ptr.is_null() {
                    let tile_json = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        result.json_ptr,
                        result.json_len,
                    ));

                    let resolved_json = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        result.resolved_ptr,
                        result.resolved_len,
                    ));

                    let tile_data: serde_json::Value =
                        serde_json::from_str(tile_json).map_err(|e| {
                            ExtensionError::new(format!("Failed to parse tile_data: {}", e))
                        })?;

                    let display_value: serde_json::Value = serde_json::from_str(resolved_json)
                        .map_err(|e| {
                            ExtensionError::new(format!("Failed to parse display_value: {}", e))
                        })?;

                    // Free the result memory
                    (handler.free_fn)(result);

                    Ok(alizarin_core::CoercionResult::success(
                        tile_data,
                        display_value,
                    ))
                } else {
                    let error_msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        result.error_ptr,
                        result.error_len,
                    ))
                    .to_string();

                    (handler.free_fn)(result);

                    Err(ExtensionError::new(error_msg))
                }
            }
        } else {
            // No handler - pass through
            Ok(alizarin_core::CoercionResult::success(
                value.clone(),
                value.clone(),
            ))
        }
    }

    fn render_display(
        &self,
        tile_data: &serde_json::Value,
        language: &str,
    ) -> Result<Option<String>, ExtensionError> {
        let handlers = TYPE_HANDLERS.read().unwrap();

        if let Some(handler) = handlers.get(&self.type_name) {
            if let (Some(render_fn), Some(free_fn)) =
                (handler.render_display_fn, handler.free_display_fn)
            {
                let tile_json = serde_json::to_string(tile_data).map_err(|e| {
                    ExtensionError::new(format!("Failed to serialize tile_data: {}", e))
                })?;

                // SAFETY: Calling render_display FFI function pointer registered via PyCapsule.
                // The handler owns the result memory and we free it via free_fn before returning.
                // Pointers are valid UTF-8 JSON produced by the extension's Rust code (same process).
                unsafe {
                    let result = render_fn(
                        tile_json.as_ptr(),
                        tile_json.len(),
                        language.as_ptr(),
                        language.len(),
                    );

                    if result.error_ptr.is_null() {
                        if result.display_ptr.is_null() {
                            free_fn(result);
                            return Ok(None);
                        }

                        let display_str = std::str::from_utf8_unchecked(
                            std::slice::from_raw_parts(result.display_ptr, result.display_len),
                        )
                        .to_string();

                        free_fn(result);
                        Ok(Some(display_str))
                    } else {
                        let error_msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                            result.error_ptr,
                            result.error_len,
                        ))
                        .to_string();

                        free_fn(result);
                        Err(ExtensionError::new(error_msg))
                    }
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn resolve_markers(
        &self,
        tile_data: &serde_json::Value,
        language: &str,
    ) -> Result<serde_json::Value, ExtensionError> {
        let handlers = TYPE_HANDLERS.read().unwrap();

        if let Some(handler) = handlers.get(&self.type_name) {
            if let (Some(resolve_fn), Some(free_fn)) =
                (handler.resolve_markers_fn, handler.free_resolve_markers_fn)
            {
                let tile_json = serde_json::to_string(tile_data).map_err(|e| {
                    ExtensionError::new(format!("Failed to serialize tile_data: {}", e))
                })?;

                // Get RDM cache for lookups
                // Clone the inner cache and wrap in Arc (same pattern as batch_trees_to_tiles)
                let core_cache: Option<Arc<CoreRdmCache>> = rdm_cache_py::get_global_rdm_cache()
                    .map(|cache| Arc::new(cache.inner().clone()));

                // SAFETY: Calling resolve_markers FFI function pointer registered via PyCapsule.
                // cache_ptr is either null or points to an Arc<CoreRdmCache> kept alive by
                // core_cache for the duration of this call. RDM callback fn pointers
                // (rdm_has_collection, rdm_lookup_by_id, etc.) are safe extern "C" functions
                // defined in this crate. Result memory is freed via free_fn before returning.
                unsafe {
                    let cache_ptr = if let Some(ref cache) = core_cache {
                        Arc::as_ptr(cache) as *mut c_void
                    } else {
                        std::ptr::null_mut()
                    };

                    // The resolve_fn expects: (json_ptr, json_len, config_ptr, config_len,
                    //                          has_collection_fn, lookup_by_id_fn, lookup_by_label_fn,
                    //                          free_concept_fn, user_data)
                    // We pass language as the config (it's used for localization)
                    let result = resolve_fn(
                        tile_json.as_ptr(),
                        tile_json.len(),
                        language.as_ptr(),
                        language.len(),
                        rdm_has_collection,
                        rdm_lookup_by_id,
                        rdm_lookup_by_label,
                        free_concept_json,
                        cache_ptr,
                    );

                    if result.error_ptr.is_null() {
                        if result.modified && !result.json_ptr.is_null() {
                            let resolved_json = std::str::from_utf8_unchecked(
                                std::slice::from_raw_parts(result.json_ptr, result.json_len),
                            );

                            let resolved: serde_json::Value = serde_json::from_str(resolved_json)
                                .map_err(|e| {
                                ExtensionError::new(format!("Failed to parse resolved: {}", e))
                            })?;

                            free_fn(result);
                            Ok(resolved)
                        } else {
                            free_fn(result);
                            Ok(tile_data.clone())
                        }
                    } else {
                        let error_msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                            result.error_ptr,
                            result.error_len,
                        ))
                        .to_string();

                        free_fn(result);
                        Err(ExtensionError::new(error_msg))
                    }
                }
            } else {
                Ok(tile_data.clone())
            }
        } else {
            Ok(tile_data.clone())
        }
    }

    fn description(&self) -> &str {
        "Python extension type handler (C ABI)"
    }
}

/// Get the list of registered extension handler datatypes.
#[pyfunction]
fn get_registered_extension_handlers() -> Vec<String> {
    TYPE_HANDLERS.read().unwrap().keys().cloned().collect()
}

// =============================================================================
// RDM Lookup Callbacks for Marker Resolution
// =============================================================================

use alizarin_core::rdm_cache::RdmCache as CoreRdmCache;

/// C ABI callback to look up a concept by ID in the RDM cache
///
/// The user_data pointer points to an Arc<CoreRdmCache>
unsafe extern "C" fn rdm_lookup_by_id(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
    concept_id_ptr: *const u8,
    concept_id_len: usize,
    concept_json_ptr: *mut *mut u8,
    concept_json_len: *mut usize,
) -> bool {
    let cache = &*(user_data as *const CoreRdmCache);

    let collection_id = match std::str::from_utf8(std::slice::from_raw_parts(
        collection_id_ptr,
        collection_id_len,
    )) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let concept_id =
        match std::str::from_utf8(std::slice::from_raw_parts(concept_id_ptr, concept_id_len)) {
            Ok(s) => s,
            Err(_) => return false,
        };

    // Look up the concept
    if let Some(concept) = cache.lookup_concept(collection_id, concept_id) {
        // Serialize to JSON
        let json = serde_json::json!({
            "id": concept.id,
            "pref_label": concept.pref_label,
            "alt_labels": concept.alt_labels,
            "scope_notes": concept.scope_note,
            "narrower": concept.narrower,
        });

        if let Ok(json_bytes) = serde_json::to_vec(&json) {
            let len = json_bytes.len();
            let ptr = Box::into_raw(json_bytes.into_boxed_slice()) as *mut u8;
            *concept_json_ptr = ptr;
            *concept_json_len = len;
            return true;
        }
    }

    false
}

/// C ABI callback to look up a concept by label in the RDM cache
///
/// The user_data pointer points to a CoreRdmCache
unsafe extern "C" fn rdm_lookup_by_label(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
    label_ptr: *const u8,
    label_len: usize,
    concept_json_ptr: *mut *mut u8,
    concept_json_len: *mut usize,
) -> bool {
    let cache = &*(user_data as *const CoreRdmCache);

    let collection_id = match std::str::from_utf8(std::slice::from_raw_parts(
        collection_id_ptr,
        collection_id_len,
    )) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let label = match std::str::from_utf8(std::slice::from_raw_parts(label_ptr, label_len)) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Look up the concept by label
    if let Some(concept) = cache.lookup_by_label(collection_id, label) {
        // Serialize to JSON
        let json = serde_json::json!({
            "id": concept.id,
            "pref_label": concept.pref_label,
            "alt_labels": concept.alt_labels,
            "scope_notes": concept.scope_note,
            "narrower": concept.narrower,
        });

        if let Ok(json_bytes) = serde_json::to_vec(&json) {
            let len = json_bytes.len();
            let ptr = Box::into_raw(json_bytes.into_boxed_slice()) as *mut u8;
            *concept_json_ptr = ptr;
            *concept_json_len = len;
            return true;
        }
    }

    false
}

/// C ABI callback to free concept JSON returned by lookup functions
unsafe extern "C" fn free_concept_json(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len));
    }
}

/// C ABI callback to check if a collection exists in the RDM cache
///
/// The user_data pointer points to a CoreRdmCache
unsafe extern "C" fn rdm_has_collection(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
) -> bool {
    let cache = &*(user_data as *const CoreRdmCache);

    let collection_id = match std::str::from_utf8(std::slice::from_raw_parts(
        collection_id_ptr,
        collection_id_len,
    )) {
        Ok(s) => s,
        Err(_) => return false,
    };

    cache.has_collection(collection_id)
}

/// Register a type handler from a PyCapsule
///
/// This is called once at import time by extensions to register their handlers.
/// After registration, the handler can be called directly from Rust without
/// crossing the Python boundary.
#[pyfunction]
#[pyo3(name = "register_type_handler")]
fn register_extension_handler(capsule: &PyCapsule) -> PyResult<()> {
    let ptr = capsule.pointer() as *const TypeHandlerInfo;

    // SAFETY: ptr comes from PyCapsule::pointer(), which returns the pointer set by the
    // extension when creating the capsule. The TypeHandlerInfo struct and its type_name
    // buffer are owned by the capsule and remain valid for its lifetime. Function pointers
    // within the struct are valid Rust fn pointers exported by the extension's cdylib.
    unsafe {
        let info = &*ptr;
        let type_name = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            info.type_name_ptr,
            info.type_name_len,
        ))
        .to_string();

        let has_display = info.render_display_fn.is_some();
        let has_markers = info.resolve_markers_fn.is_some();

        TYPE_HANDLERS.write().unwrap().insert(
            type_name.clone(),
            RegisteredHandler {
                coerce_fn: info.coerce_fn,
                free_fn: info.free_fn,
                render_display_fn: info.render_display_fn,
                free_display_fn: info.free_display_fn,
                resolve_markers_fn: info.resolve_markers_fn,
                free_resolve_markers_fn: info.free_resolve_markers_fn,
            },
        );

        let features = match (has_display, has_markers) {
            (true, true) => " (with display renderer, marker resolver)",
            (true, false) => " (with display renderer)",
            (false, true) => " (with marker resolver)",
            (false, false) => "",
        };
        println!("Registered type handler for: {}{}", type_name, features);
    }

    Ok(())
}

/// Check if a type handler is registered
#[pyfunction]
#[pyo3(name = "has_type_handler")]
fn has_extension_handler(type_name: &str) -> bool {
    TYPE_HANDLERS.read().unwrap().contains_key(type_name)
}

/// Coerce a value using a registered extension handler
///
/// Returns (tile_data_json, resolved_json) or raises an error
#[pyfunction]
fn coerce_with_extension(
    type_name: &str,
    value_json: &str,
    config_json: Option<&str>,
) -> PyResult<(String, String)> {
    let handlers = TYPE_HANDLERS.read().unwrap();

    if let Some(handler) = handlers.get(type_name) {
        let config = config_json.unwrap_or("null");

        // SAFETY: Calling coerce_fn FFI function pointer registered via PyCapsule.
        // The handler owns the result memory and we free it via handler.free_fn before
        // returning. Pointers are valid UTF-8 JSON produced by the extension's Rust code.
        unsafe {
            let result = (handler.coerce_fn)(
                value_json.as_ptr(),
                value_json.len(),
                config.as_ptr(),
                config.len(),
            );

            if result.error_ptr.is_null() {
                let tile_json = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    result.json_ptr,
                    result.json_len,
                ))
                .to_string();

                let resolved_json = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    result.resolved_ptr,
                    result.resolved_len,
                ))
                .to_string();

                (handler.free_fn)(result);
                Ok((tile_json, resolved_json))
            } else {
                let error = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    result.error_ptr,
                    result.error_len,
                ))
                .to_string();

                (handler.free_fn)(result);
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(error))
            }
        }
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
            "No handler registered for type: {}",
            type_name
        )))
    }
}

/// Check if a type handler has display rendering support
#[pyfunction]
fn has_display_renderer(type_name: &str) -> bool {
    TYPE_HANDLERS
        .read()
        .unwrap()
        .get(type_name)
        .map(|h| h.render_display_fn.is_some())
        .unwrap_or(false)
}

/// Render a resolved value to a display string using a registered extension handler
///
/// Returns the display string or raises an error.
/// Raises KeyError if no handler is registered or handler doesn't support display rendering.
#[pyfunction]
fn render_display_with_extension(
    type_name: &str,
    resolved_json: &str,
    lang: &str,
) -> PyResult<String> {
    let handlers = TYPE_HANDLERS.read().unwrap();

    if let Some(handler) = handlers.get(type_name) {
        if let (Some(render_fn), Some(free_fn)) =
            (handler.render_display_fn, handler.free_display_fn)
        {
            // SAFETY: Calling render_display FFI function pointer registered via PyCapsule.
            // The handler owns the result memory and we free it via free_fn before returning.
            // Pointers are valid UTF-8 produced by the extension's Rust code (same process).
            unsafe {
                let result = render_fn(
                    resolved_json.as_ptr(),
                    resolved_json.len(),
                    lang.as_ptr(),
                    lang.len(),
                );

                if result.error_ptr.is_null() {
                    let display = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        result.display_ptr,
                        result.display_len,
                    ))
                    .to_string();

                    free_fn(result);
                    Ok(display)
                } else {
                    let error = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        result.error_ptr,
                        result.error_len,
                    ))
                    .to_string();

                    free_fn(result);
                    Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(error))
                }
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
                "Handler for type '{}' does not support display rendering",
                type_name
            )))
        }
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
            "No handler registered for type: {}",
            type_name
        )))
    }
}

// Import core types and functions directly (no WASM dependency needed)
use alizarin_core::{
    batch_merge_resources as core_batch_merge_resources,
    // JSON conversion
    cards_to_tree,
    create_static_resource,
    evaluate_tile_path,
    // Semantic child matching
    matches_semantic_child as core_matches_semantic_child,
    merge_resources as core_merge_resources,
    parse_resources_from_json_str as core_parse_resources,
    // Label resolution
    resolve_labels as core_resolve_labels,
    tiles_to_tree,
    tree_to_tiles_with_options,
    // Graph model access
    GraphModelAccess,
    MergeAccumulator,
    // Permission rules
    PermissionRule,
    StaticGraph as AlizarinCoreStaticGraph,
    StaticNode as AlizarinStaticNode,
    StaticResource as AlizarinStaticResource,
    // Resource registry for relationship resolution
    StaticResourceRegistry as CoreStaticResourceRegistry,
    StaticResourceSummary as CoreStaticResourceSummary,
    // Graph types
    StaticTile as AlizarinStaticTile,
    DEFAULT_CONFIG_KEYS,
    DEFAULT_RESOLVABLE_DATATYPES,
};

/// Build alias-to-collection mapping directly from StaticGraph.
///
/// Includes core resolvable datatypes (concept, concept-list) plus any
/// extension-registered datatypes that have marker resolution support.
fn build_alias_to_collection_from_graph(
    graph: &AlizarinCoreStaticGraph,
) -> HashMap<String, String> {
    use std::collections::HashSet;

    let mut resolvable_set: HashSet<&str> = DEFAULT_RESOLVABLE_DATATYPES.iter().copied().collect();

    // Include extension datatypes that have resolve_markers_fn
    let handlers = TYPE_HANDLERS.read().unwrap();
    let extension_types: Vec<String> = handlers
        .iter()
        .filter(|(_, h)| h.resolve_markers_fn.is_some())
        .map(|(name, _)| name.clone())
        .collect();
    for dt in &extension_types {
        resolvable_set.insert(dt.as_str());
    }

    let mut alias_to_collection = HashMap::new();
    for node in &graph.nodes {
        if let Some(alias) = &node.alias {
            if !resolvable_set.contains(node.datatype.as_str()) {
                continue;
            }
            for key in DEFAULT_CONFIG_KEYS {
                if let Some(collection_id) = node.config.get(*key).and_then(|v| v.as_str()) {
                    alias_to_collection.insert(alias.clone(), collection_id.to_string());
                    break;
                }
            }
        }
    }
    alias_to_collection
}

/// Helper to get a graph from the registry
fn get_registered_graph(graph_id: &str) -> PyResult<std::sync::Arc<AlizarinCoreStaticGraph>> {
    alizarin_core::get_graph(graph_id).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!(
            "Graph '{}' not registered. Call register_graph() first.",
            graph_id
        ))
    })
}

/// Convert tiled resource to nested JSON tree
///
/// Args:
///     resource_json: Resource dict as JSON string containing:
///         - resourceinstanceid (or resourceinstance.resourceinstanceid)
///         - graph_id (or resourceinstance.graph_id)
///         - tiles array
///
/// Returns:
///     Nested dict structure representing the resource tree with resourceinstanceid and graph_id
#[pyfunction]
#[pyo3(signature = (resource_json))]
fn tiles_to_json_tree(py: Python, resource_json: String) -> PyResult<PyObject> {
    // Parse resource from JSON
    let resource: serde_json::Value = serde_json::from_str(&resource_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse resource: {}", e))
    })?;

    // Extract graph_id from resource (supports both formats)
    let graph_id = resource
        .get("resourceinstance")
        .and_then(|ri| ri.get("graph_id"))
        .or_else(|| resource.get("graph_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing graph_id in resource")
        })?
        .to_string();

    // Extract resource_id from resource (supports both formats)
    let resource_id = resource
        .get("resourceinstance")
        .and_then(|ri| ri.get("resourceinstanceid"))
        .or_else(|| resource.get("resourceinstanceid"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Get graph from registry
    let graph = get_registered_graph(&graph_id)?;

    // Check if input is already in StaticResource format (has resourceinstance)
    let input_json = if resource.get("resourceinstance").is_some() {
        // Already in StaticResource format, use directly
        resource.clone()
    } else {
        // Old format - convert to StaticResource
        let tiles: Vec<AlizarinStaticTile> = resource
            .get("tiles")
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();

        let static_resource =
            create_static_resource(resource_id.clone(), graph_id.clone(), tiles, &graph);
        serde_json::to_value(&static_resource).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize resource: {}",
                e
            ))
        })?
    };

    // Call shared Rust conversion function (returns array)
    let json_tree_array = tiles_to_tree(&input_json, &graph)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Extract first element (single resource case)
    let json_tree = json_tree_array
        .as_array()
        .and_then(|arr| arr.first().cloned())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    // Wrap tree with metadata for consistency with json_tree_to_tiles
    let mut result = json_tree.clone();
    if let serde_json::Value::Object(ref mut map) = result {
        map.insert(
            "resourceinstanceid".to_string(),
            serde_json::Value::String(resource_id),
        );
        map.insert("graph_id".to_string(), serde_json::Value::String(graph_id));
    }

    // Convert to Python dict directly
    pythonize::pythonize(py, &result)
        .map(|obj| obj.into())
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert result to Python: {}",
                e
            ))
        })
}

/// Build a tree from pre-extracted tiles (no resource wrapper needed).
///
/// Thin shim around `alizarin_core::build_tree_from_tiles` — all logic
/// lives in core so NAPI/WASM consumers get the same function for free.
///
/// Args:
///     tiles_json: JSON array of tile objects
///     resource_id: Resource instance ID
///     graph_id: Must be registered via register_graph()
///
/// Returns:
///     Nested dict structure representing the resource tree
#[pyfunction]
#[pyo3(signature = (tiles_json, resource_id, graph_id))]
fn build_tree_from_tiles(
    py: Python,
    tiles_json: String,
    resource_id: String,
    graph_id: String,
) -> PyResult<PyObject> {
    let result = alizarin_core::build_tree_from_tiles(&tiles_json, &resource_id, &graph_id)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    pythonize::pythonize(py, &result)
        .map(|obj| obj.into())
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert result to Python: {}",
                e
            ))
        })
}

/// Convert tiles to a card-structured tree.
///
/// Groups data by the card/widget hierarchy (UI structure) rather than by
/// node edges (data structure). Each card contains its widgets' values and
/// nested child cards.
///
/// Args:
///     resource_json: Resource JSON string (same format as tiles_to_json_tree)
///
/// Returns:
///     Card-structured dict with widgets and nested cards
#[pyfunction]
#[pyo3(signature = (resource_json))]
fn cards_to_json_tree(py: Python, resource_json: String) -> PyResult<PyObject> {
    let resource: serde_json::Value = serde_json::from_str(&resource_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse resource: {}", e))
    })?;

    let graph_id = resource
        .get("resourceinstance")
        .and_then(|ri| ri.get("graph_id"))
        .or_else(|| resource.get("graph_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing graph_id in resource")
        })?
        .to_string();

    let resource_id = resource
        .get("resourceinstance")
        .and_then(|ri| ri.get("resourceinstanceid"))
        .or_else(|| resource.get("resourceinstanceid"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let graph = get_registered_graph(&graph_id)?;

    let input_json = if resource.get("resourceinstance").is_some() {
        resource.clone()
    } else {
        let tiles: Vec<AlizarinStaticTile> = resource
            .get("tiles")
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();

        let static_resource =
            create_static_resource(resource_id.clone(), graph_id.clone(), tiles, &graph);
        serde_json::to_value(&static_resource).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize resource: {}",
                e
            ))
        })?
    };

    let json_tree_array = cards_to_tree(&input_json, &graph)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    let json_tree = json_tree_array
        .as_array()
        .and_then(|arr| arr.first().cloned())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let mut result = json_tree.clone();
    if let serde_json::Value::Object(ref mut map) = result {
        map.insert(
            "resourceinstanceid".to_string(),
            serde_json::Value::String(resource_id),
        );
        map.insert("graph_id".to_string(), serde_json::Value::String(graph_id));
    }

    pythonize::pythonize(py, &result)
        .map(|obj| obj.into())
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert result to Python: {}",
                e
            ))
        })
}

// =============================================================================
// Shared extension coercion helper
// =============================================================================

/// Build a node_id → (datatype, config) lookup from a graph's nodes.
fn build_node_lookup(
    graph: &AlizarinCoreStaticGraph,
) -> std::collections::HashMap<String, (String, Option<serde_json::Value>)> {
    let mut lookup = std::collections::HashMap::new();
    for node in &graph.nodes {
        let config_value = if node.config.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&node.config).unwrap_or(serde_json::Value::Null))
        };
        lookup.insert(node.nodeid.clone(), (node.datatype.clone(), config_value));
    }
    lookup
}

/// Apply registered extension coercion handlers to a resource's tiles.
///
/// Iterates each tile's data, checks for registered extension handlers by datatype,
/// calls coerce_fn (and optionally resolve_markers_fn), and replaces tile data values.
///
/// # Safety
/// Calls FFI function pointers from registered extension handlers. See safety comments inline.
fn apply_extension_coercion(
    resource: &mut AlizarinStaticResource,
    node_lookup: &std::collections::HashMap<String, (String, Option<serde_json::Value>)>,
    handlers: &HashMap<String, RegisteredHandler>,
    core_cache: &Option<Arc<CoreRdmCache>>,
    strict: bool,
    tree_index: usize,
) -> Result<(), String> {
    if let Some(ref mut tiles) = resource.tiles {
        for tile in tiles.iter_mut() {
            let mut new_data = std::collections::HashMap::new();
            let mut _coercion_errors: Vec<String> = Vec::new();

            for (node_id, value) in tile.data.iter() {
                if let Some((datatype, config)) = node_lookup.get(node_id) {
                    if let Some(handler) = handlers.get(datatype) {
                        let value_json = serde_json::to_string(value)
                            .map_err(|e| format!("Failed to serialize value: {}", e))?;
                        let config_json = config
                            .as_ref()
                            .map(|c| serde_json::to_string(c).unwrap_or_default())
                            .unwrap_or_else(|| "null".to_string());

                        // SAFETY: Calling coerce_fn and optionally resolve_markers_fn FFI
                        // function pointers registered via PyCapsule. All result memory is
                        // freed via the corresponding free_fn before moving on. cache_ptr
                        // points to an Arc<CoreRdmCache> kept alive by core_cache for the
                        // duration of the call. RDM callback fn pointers are safe extern "C"
                        // functions defined in this crate.
                        unsafe {
                            let result = (handler.coerce_fn)(
                                value_json.as_ptr(),
                                value_json.len(),
                                config_json.as_ptr(),
                                config_json.len(),
                            );

                            if result.error_ptr.is_null() {
                                let tile_json = std::str::from_utf8_unchecked(
                                    std::slice::from_raw_parts(result.json_ptr, result.json_len),
                                );
                                let mut coerced_value: serde_json::Value =
                                    serde_json::from_str(tile_json).unwrap_or(value.clone());
                                (handler.free_fn)(result);

                                if let (Some(resolve_fn), Some(free_fn), Some(ref cache)) = (
                                    handler.resolve_markers_fn,
                                    handler.free_resolve_markers_fn,
                                    core_cache,
                                ) {
                                    let coerced_json =
                                        serde_json::to_string(&coerced_value).unwrap_or_default();
                                    let cache_ptr = Arc::as_ptr(cache) as *mut c_void;

                                    let resolve_result = resolve_fn(
                                        coerced_json.as_ptr(),
                                        coerced_json.len(),
                                        config_json.as_ptr(),
                                        config_json.len(),
                                        rdm_has_collection,
                                        rdm_lookup_by_id,
                                        rdm_lookup_by_label,
                                        free_concept_json,
                                        cache_ptr,
                                    );

                                    if resolve_result.modified && !resolve_result.json_ptr.is_null()
                                    {
                                        let resolved_json = std::str::from_utf8_unchecked(
                                            std::slice::from_raw_parts(
                                                resolve_result.json_ptr,
                                                resolve_result.json_len,
                                            ),
                                        );
                                        if let Ok(resolved) = serde_json::from_str(resolved_json) {
                                            coerced_value = resolved;
                                        }
                                        free_fn(resolve_result);
                                    } else if !resolve_result.error_ptr.is_null() {
                                        let error = std::str::from_utf8_unchecked(
                                            std::slice::from_raw_parts(
                                                resolve_result.error_ptr,
                                                resolve_result.error_len,
                                            ),
                                        )
                                        .to_string();
                                        free_fn(resolve_result);

                                        if strict {
                                            return Err(format!(
                                                "Tree {}, Node {}: Marker resolution failed: {}",
                                                tree_index, node_id, error
                                            ));
                                        } else {
                                            _coercion_errors.push(format!(
                                                "Node {} marker resolution: {}",
                                                node_id, error
                                            ));
                                        }
                                    } else {
                                        free_fn(resolve_result);
                                    }
                                }

                                new_data.insert(node_id.clone(), coerced_value);
                            } else {
                                let error = std::str::from_utf8_unchecked(
                                    std::slice::from_raw_parts(result.error_ptr, result.error_len),
                                )
                                .to_string();
                                (handler.free_fn)(result);

                                if strict {
                                    return Err(format!(
                                        "Tree {}, Node {}: Extension coercion failed: {}",
                                        tree_index, node_id, error
                                    ));
                                } else {
                                    _coercion_errors.push(format!("Node {}: {}", node_id, error));
                                    new_data.insert(node_id.clone(), value.clone());
                                }
                            }
                        }
                    } else {
                        new_data.insert(node_id.clone(), value.clone());
                    }
                } else {
                    new_data.insert(node_id.clone(), value.clone());
                }
            }

            tile.data = new_data;
        }
    }
    Ok(())
}

/// Convert nested JSON tree to tiled resource
///
/// Args:
///     tree_json: Nested JSON tree as string
///     resource_id: Resource instance ID
///     graph_id: Graph ID (must be registered via register_graph first)
///     from_camel: If True, convert keys from camelCase to snake_case before resolving
///     strict: If True, raise an error for any validation failures or unparseable data
///
/// Returns:
///     Dict with 'resourceinstanceid', 'graph_id', and 'tiles'
///
/// If a global RDM cache is set (via set_global_rdm_cache), label strings
/// will automatically be resolved to UUIDs before conversion.
///
/// If id_key is provided and the tree does not contain a resourceinstanceid,
/// a deterministic UUID v5 will be generated using the key and graph ID.
///
/// If scopes is provided (as JSON string), it will be set on all resulting resources.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (tree_json, resource_id, graph_id, from_camel=false, strict=true, rdm_cache=None, id_key=None, scopes=None))]
fn json_tree_to_tiles(
    py: Python,
    tree_json: String,
    resource_id: String,
    graph_id: String,
    from_camel: bool,
    strict: bool,
    rdm_cache: Option<&rdm_cache_py::RdmCache>,
    id_key: Option<String>,
    scopes: Option<String>,
) -> PyResult<PyObject> {
    // Parse tree from JSON
    let mut tree: serde_json::Value = serde_json::from_str(&tree_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse tree: {}", e))
    })?;

    // Parse scopes if provided
    let scopes_value: Option<serde_json::Value> = scopes
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse scopes: {}",
                e
            ))
        })?;

    // Get graph from registry
    let graph = get_registered_graph(&graph_id)?;

    // Resolve labels to UUIDs if an RDM cache is available
    // Use explicit cache if provided, otherwise try global cache
    let global_cache = rdm_cache_py::get_global_rdm_cache();
    let cache_to_use: Option<&rdm_cache_py::RdmCache> = rdm_cache.or(global_cache.as_ref());

    if let Some(cache) = cache_to_use {
        let alias_map = build_alias_to_collection_from_graph(&graph);
        tree = core_resolve_labels(tree, &alias_map, cache, strict)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.message))?;
    }

    // Add resourceinstanceid and graph_id to tree for new API
    if let serde_json::Value::Object(ref mut map) = tree {
        map.insert(
            "resourceinstanceid".to_string(),
            serde_json::Value::String(resource_id.clone()),
        );
        map.insert(
            "graph_id".to_string(),
            serde_json::Value::String(graph_id.clone()),
        );
    }

    // Call shared Rust conversion function with from_camel support
    // This handles camelCase keys at lookup time, preserving value structures
    // NOTE: We pass None for extension registry here because extension coercion
    // (+ marker resolution) is handled separately by apply_extension_coercion below.
    // Passing the registry here would cause double coercion — the first pass creates
    // marker objects that the second pass can't re-coerce.
    let id_key_ref = id_key.as_deref();
    let has_extension_handlers = {
        let handlers = TYPE_HANDLERS.read().unwrap();
        !handlers.is_empty()
    };
    let mut business_data = tree_to_tiles_with_options(
        &tree,
        &graph,
        strict,
        id_key_ref,
        from_camel,
        false,
        has_extension_handlers,
        None,
    )
    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    // Set scopes on all resources if provided
    if let Some(ref scopes_val) = scopes_value {
        for resource in &mut business_data.business_data.resources {
            resource.scopes = Some(scopes_val.clone());
        }
    }

    // Apply extension coercion via C ABI handlers if any are registered
    {
        let handlers = TYPE_HANDLERS.read().unwrap();
        if !handlers.is_empty() {
            let node_lookup = build_node_lookup(&graph);
            let core_cache: Option<Arc<CoreRdmCache>> =
                rdm_cache_py::get_global_rdm_cache().map(|cache| Arc::new(cache.inner().clone()));

            for resource in &mut business_data.business_data.resources {
                apply_extension_coercion(resource, &node_lookup, &handlers, &core_cache, strict, 0)
                    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
            }
        }
    }

    // Return full BusinessDataWrapper structure: {business_data: {resources: [...]}}
    pythonize::pythonize(py, &business_data)
        .map(|obj| obj.into())
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert result to Python: {}",
                e
            ))
        })
}

/// Batch convert multiple JSON trees to tiles in parallel.
///
/// Applies registered extension handlers (e.g. CLM "reference" type) by default.
/// Set `skip_extensions=True` to bypass extension coercion for a minor performance gain
/// when your graph has no extension-type nodes.
///
/// Args:
///     trees_json: Array of trees as JSON string
///     graph_id: Graph ID (must be registered via register_graph first)
///     from_camel: If True, convert keys from camelCase to snake_case
///     strict: If True, fail on first error (including extension coercion errors)
///     id_keys: Optional list of keys for deterministic UUID generation
///     scopes: Optional JSON string to set as scopes on all resources
///     resolve_markers: If True, resolve extension markers (e.g. label→UUID) via RDM cache
///     random_ids: If True, generate random UUIDs instead of slug-based IDs
///     skip_extensions: If True, skip extension coercion (unknown datatypes will error in strict mode)
///
/// Returns:
///     BusinessDataWrapper format: {business_data: {resources: [...]}, errors: [...], error_count: int}
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (trees_json, graph_id, from_camel=false, strict=true, id_keys=None, scopes=None, resolve_markers=true, random_ids=false, skip_extensions=false))]
fn batch_trees_to_tiles(
    py: Python,
    trees_json: String,
    graph_id: String,
    from_camel: bool,
    strict: bool,
    id_keys: Option<Vec<String>>,
    scopes: Option<String>,
    resolve_markers: bool,
    random_ids: bool,
    skip_extensions: bool,
) -> PyResult<PyObject> {
    let trees: Vec<serde_json::Value> = serde_json::from_str(&trees_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse trees: {}", e))
    })?;

    // Parse scopes if provided
    let scopes_value: Option<serde_json::Value> = scopes
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse scopes: {}",
                e
            ))
        })?;

    // Validate id_keys length if provided
    if let Some(ref keys) = id_keys {
        if keys.len() != trees.len() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "id_keys length ({}) must match trees length ({})",
                keys.len(),
                trees.len()
            )));
        }
    }

    // Get graph from registry
    let graph = get_registered_graph(&graph_id)?;

    // Build node lookup and extension handler state only if extensions are enabled
    let node_lookup: Option<
        std::collections::HashMap<String, (String, Option<serde_json::Value>)>,
    > = if !skip_extensions {
        Some(build_node_lookup(&graph))
    } else {
        None
    };

    let handlers = if !skip_extensions {
        Some(TYPE_HANDLERS.read().unwrap())
    } else {
        None
    };

    let core_cache: Option<Arc<CoreRdmCache>> = if !skip_extensions && resolve_markers {
        rdm_cache_py::get_global_rdm_cache().map(|cache| Arc::new(cache.inner().clone()))
    } else {
        None
    };

    let has_extension_handlers = !skip_extensions;

    let results: Vec<Result<serde_json::Value, String>> = trees
        .into_par_iter()
        .enumerate()
        .map(|(i, mut tree)| {
            // Add graph_id to tree if not present
            if let serde_json::Value::Object(ref mut map) = tree {
                if !map.contains_key("graph_id") {
                    map.insert(
                        "graph_id".to_string(),
                        serde_json::Value::String(graph_id.clone()),
                    );
                }
            }

            // Get id_key for this tree (if id_keys array provided)
            let id_key_ref = id_keys.as_ref().map(|keys| keys[i].as_str());

            // NOTE: Pass None for extension registry — extension coercion + marker
            // resolution is handled by apply_extension_coercion below. Passing the
            // registry here would cause double coercion.
            let business_data = tree_to_tiles_with_options(
                &tree,
                &graph,
                strict,
                id_key_ref,
                from_camel,
                random_ids,
                has_extension_handlers,
                None,
            )
            .map_err(|e| format!("Tree {}: {}", i, e))?;

            // Extract first resource
            let mut resource = business_data
                .business_data
                .resources
                .into_iter()
                .next()
                .ok_or_else(|| format!("Tree {}: No resources returned", i))?;

            // Set scopes if provided
            if let Some(ref scopes_val) = scopes_value {
                resource.scopes = Some(scopes_val.clone());
            }

            // Apply extension coercion (unless skipped)
            if let (Some(ref node_lookup), Some(ref handlers)) = (&node_lookup, &handlers) {
                apply_extension_coercion(
                    &mut resource,
                    node_lookup,
                    handlers,
                    &core_cache,
                    strict,
                    i,
                )?;
            }

            serde_json::to_value(&resource)
                .map_err(|e| format!("Tree {}: Failed to serialize: {}", i, e))
        })
        .collect();

    // Separate successes and errors
    let (successes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
    let resources: Vec<_> = successes.into_iter().map(Result::unwrap).collect();
    let mut errors: Vec<_> = errors.into_iter().map(|e| e.unwrap_err()).collect();

    // In strict mode, fail if any errors
    if strict && !errors.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Strict mode error: {}",
            errors[0]
        )));
    }

    // Check slug uniqueness if slug-based IDs were used
    if id_keys.is_none() && !random_ids {
        let mut seen_slugs: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for (i, resource_value) in resources.iter().enumerate() {
            if let Some(slug) = resource_value
                .get("resourceinstance")
                .and_then(|ri| ri.get("descriptors"))
                .and_then(|d| d.get("slug"))
                .and_then(|s| s.as_str())
            {
                if let Some(prev_idx) = seen_slugs.insert(slug.to_string(), i) {
                    let err_msg =
                        format!("Duplicate slug '{}' on trees {} and {}", slug, prev_idx, i);
                    if strict {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_msg));
                    }
                    errors.push(err_msg);
                }
            }
        }
    }

    // Return BusinessDataWrapper format with errors alongside
    let output = serde_json::json!({
        "business_data": {
            "resources": resources
        },
        "errors": errors,
        "error_count": errors.len()
    });

    pythonize::pythonize(py, &output).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

// =============================================================================
// ResourceRegistry - For relationship resolution and cache population
// =============================================================================

/// Registry of known resources for relationship resolution
///
/// Used to:
/// - Look up graph_id for referenced resources
/// - Populate __cache on resources with related resource summaries
/// - Enrich resource-instance tile data with ontologyProperty from node config
/// - Validate that referenced resources exist (in strict mode)
///
/// Usage:
///     registry = ResourceRegistry()
///     registry.merge_from_resources(resources_json)  # Load known resources
///     result = batch_trees_to_tiles(..., registry=registry)
#[pyclass]
pub struct ResourceRegistry {
    inner: CoreStaticResourceRegistry,
}

impl ResourceRegistry {
    /// Get the inner core registry (for internal Rust use)
    pub(crate) fn inner(&self) -> &CoreStaticResourceRegistry {
        &self.inner
    }
}

#[pymethods]
impl ResourceRegistry {
    #[new]
    fn new() -> Self {
        ResourceRegistry {
            inner: CoreStaticResourceRegistry::new(),
        }
    }

    /// Number of resources in the registry
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Check if a resource ID is known
    fn __contains__(&self, resource_id: &str) -> bool {
        self.inner.contains(resource_id)
    }

    /// Check if registry is empty
    #[getter]
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the graph_id for a resource (or None if not known)
    fn get_graph_id(&self, resource_id: &str) -> Option<String> {
        self.inner.get_graph_id(resource_id).map(|s| s.to_string())
    }

    /// Get the summary for a resource (or None if not known)
    ///
    /// Returns a JSON string with {resourceinstanceid, graph_id, name, ...}
    fn get_summary(&self, py: Python, resource_id: &str) -> PyResult<Option<PyObject>> {
        match self.inner.get_summary(resource_id) {
            Some(summary) => {
                let json = serde_json::to_value(&summary).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize summary: {}",
                        e
                    ))
                })?;
                Ok(Some(pythonize::pythonize(py, &json)?))
            }
            None => Ok(None),
        }
    }

    /// Get the full resource if stored (or None if only summary or not known)
    ///
    /// Returns a JSON dict with full resource data including tiles
    fn get_full(&self, py: Python, resource_id: &str) -> PyResult<Option<PyObject>> {
        match self.inner.get_full(resource_id) {
            Some(resource) => {
                let json = serde_json::to_value(resource).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize resource: {}",
                        e
                    ))
                })?;
                Ok(Some(pythonize::pythonize(py, &json)?))
            }
            None => Ok(None),
        }
    }

    /// Check if a resource has full data stored (not just summary)
    fn has_full(&self, resource_id: &str) -> bool {
        self.inner.has_full(resource_id)
    }

    /// Get all full resources as a list of JSON dicts
    fn get_all_full(&self, py: Python) -> PyResult<PyObject> {
        let resources: Vec<&AlizarinStaticResource> =
            self.inner.iter_full().map(|(_, r)| r).collect();
        let json = serde_json::to_value(&resources).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize resources: {}",
                e
            ))
        })?;
        pythonize::pythonize(py, &json).map_err(|e| e.into())
    }

    /// Get all full resources for a specific graph
    fn get_all_full_for_graph(&self, py: Python, graph_id: &str) -> PyResult<PyObject> {
        let resources: Vec<&AlizarinStaticResource> = self
            .inner
            .iter_full()
            .filter(|(_, r)| r.resourceinstance.graph_id == graph_id)
            .map(|(_, r)| r)
            .collect();
        let json = serde_json::to_value(&resources).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize resources: {}",
                e
            ))
        })?;
        pythonize::pythonize(py, &json).map_err(|e| e.into())
    }

    /// Add a single resource summary to the registry
    ///
    /// Args:
    ///     summary_json: JSON string with {resourceinstanceid, graph_id, name, ...}
    fn insert(&mut self, summary_json: &str) -> PyResult<()> {
        let summary: CoreStaticResourceSummary =
            serde_json::from_str(summary_json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse summary: {}",
                    e
                ))
            })?;
        self.inner.insert(summary);
        Ok(())
    }

    /// Merge resources into the registry
    ///
    /// Registers each resource's ID → summary mapping or full resources.
    /// If store_full is True, stores full resources (for traversal).
    /// If store_full is False (default), stores only summaries (memory efficient).
    /// If include_caches is True (default), also merges any __cache.relatedResources as summaries.
    ///
    /// Args:
    ///     resources_json: JSON string containing array of StaticResource objects
    ///     store_full: If True, store full resources; if False, store only summaries
    ///     include_caches: If True, also merge related resources from __cache
    #[pyo3(signature = (resources_json, store_full=false, include_caches=true))]
    fn merge_from_resources(
        &mut self,
        resources_json: &str,
        store_full: bool,
        include_caches: bool,
    ) -> PyResult<()> {
        let resources: Vec<AlizarinStaticResource> =
            serde_json::from_str(resources_json).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse resources: {}",
                    e
                ))
            })?;
        self.inner
            .merge_from_resources(&resources, store_full, include_caches);
        Ok(())
    }

    /// Populate __cache on resources with summaries for referenced resources
    ///
    /// Uses the graph to identify resource-instance nodes, then populates
    /// cache entries for each referenced resource found in this registry.
    ///
    /// If enrich_relationships is True, also adds ontologyProperty/inverseOntologyProperty
    /// to tile data based on node config and the target resource's graph.
    ///
    /// Args:
    ///     resources_json: JSON string containing array of StaticResource objects
    ///     graph_id: ID of the graph to use for node lookup
    ///     enrich_relationships: If True, add ontologyProperty to tile data
    ///
    /// Returns:
    ///     Dict with:
    ///     - resources: Updated resources with __cache populated
    ///     - unknown_references: List of {source_resource_id, node_id, node_alias, referenced_id}
    #[pyo3(signature = (resources_json, graph_id, enrich_relationships=true, strict=false, recompute_descriptors=false))]
    fn populate_caches(
        &self,
        py: Python,
        resources_json: &str,
        graph_id: &str,
        enrich_relationships: bool,
        strict: bool,
        recompute_descriptors: bool,
    ) -> PyResult<PyObject> {
        let mut resources: Vec<AlizarinStaticResource> = serde_json::from_str(resources_json)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse resources: {}",
                    e
                ))
            })?;

        let graph = get_registered_graph(graph_id)?;
        let result = self
            .inner
            .populate_caches(
                &mut resources,
                &graph,
                enrich_relationships,
                strict,
                recompute_descriptors,
            )
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        let output = serde_json::json!({
            "resources": resources,
            "unknown_references": result.unknown_references,
            "has_unknown": result.has_unknown_references(),
        });

        pythonize::pythonize(py, &output).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Extract node values from all resources, returning a map from resource ID to values.
    ///
    /// Efficiently iterates through tiles in Rust, filtering by nodegroup and extracting
    /// values for the specified node.
    ///
    /// Args:
    ///     graph_id: ID of the graph (must be registered via register_graph)
    ///     node_identifier: Node alias or node ID to extract values for
    ///
    /// Returns:
    ///     Dict mapping resource_id -> list of values found for that node.
    ///     Values are returned as-is (may be strings, dicts for localized strings, etc.)
    ///
    /// Example:
    ///     >>> index = registry.get_node_values_index(graph_id, "resource_id")
    ///     >>> # Returns: {"uuid-1": ["REF-001"], "uuid-2": ["REF-002", "REF-003"]}
    fn get_node_values_index(
        &self,
        py: Python,
        graph_id: &str,
        node_identifier: &str,
    ) -> PyResult<PyObject> {
        let graph = get_registered_graph(graph_id)?;

        let index = self
            .inner
            .get_node_values_index(&graph, node_identifier)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        pythonize::pythonize(py, &index).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }

    /// Extract node values and return an inverted index: value -> list of resource IDs.
    ///
    /// Useful for looking up resources by a field value (e.g., find resource by external ID).
    ///
    /// Args:
    ///     graph_id: ID of the graph (must be registered via register_graph)
    ///     node_identifier: Node alias or node ID to extract values for
    ///     flatten_localized: If True, extract string from localized values {"en": "value"}
    ///
    /// Returns:
    ///     Dict mapping value -> list of resource_ids that have that value.
    ///
    /// Example:
    ///     >>> index = registry.get_value_to_resources_index(graph_id, "resource_id", True)
    ///     >>> # Returns: {"REF-001": ["uuid-1"], "REF-002": ["uuid-2", "uuid-3"]}
    #[pyo3(signature = (graph_id, node_identifier, flatten_localized=true))]
    fn get_value_to_resources_index(
        &self,
        py: Python,
        graph_id: &str,
        node_identifier: &str,
        flatten_localized: bool,
    ) -> PyResult<PyObject> {
        let graph = get_registered_graph(graph_id)?;

        let index = self
            .inner
            .get_value_to_resources_index(&graph, node_identifier, flatten_localized)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

        pythonize::pythonize(py, &index).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        })
    }
}

/// Iterator that processes trees to tiles one at a time (low memory)
///
/// Usage:
///     iter = TreeToTilesIterator(tree_strings, graph_id, from_camel=True, scopes='{"read": true}')
///     for result in iter:
///         # result is {"resource": {...}, "error": None} or {"resource": None, "error": "..."}
///         if result["resource"]:
///             process(result["resource"])
#[pyclass]
struct TreeToTilesIterator {
    trees: Vec<String>,
    graph: std::sync::Arc<AlizarinCoreStaticGraph>,
    graph_id: String,
    from_camel: bool,
    strict: bool,
    id_keys: Option<Vec<String>>,
    scopes: Option<serde_json::Value>,
    random_ids: bool,
    index: usize,
}

#[pymethods]
impl TreeToTilesIterator {
    #[new]
    #[pyo3(signature = (tree_strings, graph_id, from_camel=false, strict=true, id_keys=None, scopes=None, random_ids=false))]
    fn new(
        tree_strings: Vec<String>,
        graph_id: String,
        from_camel: bool,
        strict: bool,
        id_keys: Option<Vec<String>>,
        scopes: Option<String>,
        random_ids: bool,
    ) -> PyResult<Self> {
        // Get graph from registry
        let graph = get_registered_graph(&graph_id)?;

        // Parse scopes if provided
        let scopes_value: Option<serde_json::Value> = scopes
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse scopes: {}",
                    e
                ))
            })?;

        // Validate id_keys length if provided
        if let Some(ref keys) = id_keys {
            if keys.len() != tree_strings.len() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "id_keys length ({}) must match trees length ({})",
                    keys.len(),
                    tree_strings.len()
                )));
            }
        }

        Ok(TreeToTilesIterator {
            trees: tree_strings,
            graph,
            graph_id,
            from_camel,
            strict,
            id_keys,
            scopes: scopes_value,
            random_ids,
            index: 0,
        })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        if self.index >= self.trees.len() {
            return Ok(None);
        }

        let tree_str = &self.trees[self.index];
        let id_key = self.id_keys.as_ref().map(|keys| keys[self.index].as_str());
        let i = self.index;
        self.index += 1;

        // Parse tree
        let tree_result: Result<serde_json::Value, _> = serde_json::from_str(tree_str);
        let mut tree = match tree_result {
            Ok(t) => t,
            Err(e) => {
                let output = serde_json::json!({
                    "resource": null,
                    "error": format!("Tree {}: Failed to parse: {}", i, e),
                    "index": i
                });
                return pythonize::pythonize(py, &output)
                    .map(Some)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()));
            }
        };

        // Add graph_id to tree if not present
        if let serde_json::Value::Object(ref mut map) = tree {
            if !map.contains_key("graph_id") {
                map.insert(
                    "graph_id".to_string(),
                    serde_json::Value::String(self.graph_id.clone()),
                );
            }
        }

        // Convert tree to tiles with from_camel support
        // This handles camelCase keys at lookup time, preserving value structures
        // NOTE: Pass None for extension registry — extension coercion + marker
        // resolution is handled by apply_extension_coercion below.
        let has_extension_handlers = {
            let handlers = TYPE_HANDLERS.read().unwrap();
            !handlers.is_empty()
        };
        let result = tree_to_tiles_with_options(
            &tree,
            &self.graph,
            self.strict,
            id_key,
            self.from_camel,
            self.random_ids,
            has_extension_handlers,
            None,
        );

        let output = match result {
            Ok(business_data) => {
                if let Some(mut resource) = business_data.business_data.resources.into_iter().next()
                {
                    // Set scopes if provided
                    if let Some(ref scopes_val) = self.scopes {
                        resource.scopes = Some(scopes_val.clone());
                    }

                    // Apply extension coercion via C ABI handlers if registered
                    let ext_error: Option<String> = {
                        let handlers = TYPE_HANDLERS.read().unwrap();
                        if !handlers.is_empty() {
                            let node_lookup = build_node_lookup(&self.graph);
                            let core_cache: Option<Arc<CoreRdmCache>> =
                                rdm_cache_py::get_global_rdm_cache()
                                    .map(|cache| Arc::new(cache.inner().clone()));

                            apply_extension_coercion(
                                &mut resource,
                                &node_lookup,
                                &handlers,
                                &core_cache,
                                self.strict,
                                i,
                            )
                            .err()
                        } else {
                            None
                        }
                    };

                    if let Some(e) = ext_error {
                        serde_json::json!({
                            "resource": null,
                            "error": e,
                            "index": i
                        })
                    } else {
                        serde_json::json!({
                            "resource": resource,
                            "error": null,
                            "index": i
                        })
                    }
                } else {
                    serde_json::json!({
                        "resource": null,
                        "error": format!("Tree {}: No resources returned", i),
                        "index": i
                    })
                }
            }
            Err(e) => {
                serde_json::json!({
                    "resource": null,
                    "error": format!("Tree {}: {}", i, e),
                    "index": i
                })
            }
        };

        pythonize::pythonize(py, &output)
            .map(Some)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Get remaining count
    fn __len__(&self) -> usize {
        self.trees.len() - self.index
    }

    /// Total number of trees
    #[getter]
    fn total(&self) -> usize {
        self.trees.len()
    }

    /// Current index
    #[getter]
    fn current_index(&self) -> usize {
        self.index
    }
}

/// Batch convert multiple tiled resources to JSON trees in parallel
///
/// Resources must contain graph_id in resourceinstance.graph_id or graph_id field.
/// Graphs must be registered via register_graph first.
#[pyfunction]
#[pyo3(signature = (resources_json, strict=true))]
fn batch_tiles_to_trees(py: Python, resources_json: String, strict: bool) -> PyResult<PyObject> {
    let resources: Vec<serde_json::Value> = serde_json::from_str(&resources_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse resources: {}", e))
    })?;

    let results: Vec<Result<serde_json::Value, String>> = resources
        .into_par_iter()
        .enumerate()
        .map(|(i, resource)| {
            // Extract graph_id from resource
            let graph_id = resource
                .get("resourceinstance")
                .and_then(|ri| ri.get("graph_id"))
                .or_else(|| resource.get("graph_id"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Resource {}: Missing graph_id", i))?
                .to_string();

            // Get graph from registry
            let graph = alizarin_core::get_graph(&graph_id)
                .ok_or_else(|| format!("Resource {}: Graph '{}' not registered", i, graph_id))?;

            // Extract resource_id from either format:
            // - New format: resource.resourceinstance.resourceinstanceid
            // - Old format: resource.resourceinstanceid
            let resource_id = resource
                .get("resourceinstance")
                .and_then(|ri| ri.get("resourceinstanceid"))
                .or_else(|| resource.get("resourceinstanceid"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            // Check if input is already in StaticResource format (has resourceinstance)
            let input_json = if resource.get("resourceinstance").is_some() {
                // Already in StaticResource format, use directly
                resource.clone()
            } else {
                // Old format - convert to StaticResource
                let tiles: Vec<AlizarinStaticTile> = resource
                    .get("tiles")
                    .map(|t| serde_json::from_value(t.clone()))
                    .transpose()
                    .map_err(|e| format!("Resource {}: Failed to parse tiles: {}", i, e))?
                    .unwrap_or_default();

                let static_resource =
                    create_static_resource(resource_id.clone(), graph_id.clone(), tiles, &graph);

                serde_json::to_value(&static_resource)
                    .map_err(|e| format!("Resource {}: Failed to serialize: {}", i, e))?
            };

            // Call tiles_to_tree (returns array)
            let json_tree_array =
                tiles_to_tree(&input_json, &graph).map_err(|e| format!("Resource {}: {}", i, e))?;

            // Extract first element and add metadata
            let mut tree = json_tree_array
                .as_array()
                .and_then(|arr| arr.first().cloned())
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            if let serde_json::Value::Object(ref mut map) = tree {
                map.insert(
                    "resourceinstanceid".to_string(),
                    serde_json::Value::String(resource_id),
                );
                map.insert(
                    "graph_id".to_string(),
                    serde_json::Value::String(graph_id.clone()),
                );
            }

            Ok(tree)
        })
        .collect();

    // Handle results based on strict mode
    if strict {
        // In strict mode, fail on first error
        let successes: Result<Vec<_>, _> = results.into_iter().collect();
        let successes = successes.map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Strict mode error: {}", e))
        })?;

        let output = serde_json::json!({
            "success": true,
            "results": successes,
            "count": successes.len()
        });

        return pythonize::pythonize(py, &output).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert to Python: {}",
                e
            ))
        });
    }

    // Non-strict: separate successes and errors
    let (successes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
    let successes: Vec<_> = successes.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(|e| e.unwrap_err()).collect();

    let output = if errors.is_empty() {
        serde_json::json!({ "success": true, "results": successes, "count": successes.len() })
    } else {
        serde_json::json!({
            "success": false, "results": successes, "count": successes.len(),
            "errors": errors, "error_count": errors.len()
        })
    };

    pythonize::pythonize(py, &output).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

// camel_to_snake and transform_keys_to_snake are now in alizarin-core/src/string_utils.rs

/// Python wrapper for ResourceModelWrapperCore
///
/// Provides graph pruning and permission checking.
/// Graph must be registered via register_graph first.
#[pyclass]
struct PyResourceModelWrapper {
    model_access: GraphModelAccess,
}

#[pymethods]
impl PyResourceModelWrapper {
    #[new]
    fn new(graph_id: String) -> PyResult<Self> {
        // Get graph from registry
        let graph = get_registered_graph(&graph_id)?;

        Ok(PyResourceModelWrapper {
            model_access: GraphModelAccess::new_eager(graph, true),
        })
    }

    /// Build index caches (nodes, edges, nodegroups, etc.)
    /// Call after construction or after graph mutation.
    fn build_nodes(&mut self) -> PyResult<()> {
        self.model_access.ensure_built().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to build nodes: {}",
                e
            ))
        })
    }

    /// Get child nodes for a given node ID, keyed by alias.
    /// Returns a dict of {alias: node_json}.
    fn get_child_nodes(&self, py: Python, node_id: &str) -> PyResult<PyObject> {
        let children = self.model_access.get_child_nodes(node_id);
        let dict = pyo3::types::PyDict::new_bound(py);
        for (alias, node) in &children {
            let node_json = serde_json::to_string(node.as_ref()).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize node: {}",
                    e
                ))
            })?;
            dict.set_item(alias, node_json)?;
        }
        Ok(dict.to_object(py))
    }

    /// Get all nodes as a dict of {node_id: node_json}.
    #[pyo3(name = "get_nodes")]
    fn get_node_objects(&self, py: Python) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        if let Some(nodes) = alizarin_core::ModelAccess::get_nodes(&self.model_access) {
            for (id, node) in nodes {
                let node_json = serde_json::to_string(node.as_ref()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize node: {}",
                        e
                    ))
                })?;
                dict.set_item(id, node_json)?;
            }
        }
        Ok(dict.to_object(py))
    }

    /// Get all edges as a dict of {node_id: [child_node_id, ...]}.
    fn get_edges(&self, py: Python) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        if let Some(edges) = alizarin_core::ModelAccess::get_edges(&self.model_access) {
            for (id, children) in edges {
                let py_list =
                    pyo3::types::PyList::new_bound(py, children.iter().map(|s| s.as_str()));
                dict.set_item(id, py_list)?;
            }
        }
        Ok(dict.to_object(py))
    }

    /// Get nodes indexed by alias as a dict of {alias: node_json}.
    #[pyo3(name = "get_nodes_by_alias")]
    fn get_node_objects_by_alias(&self, py: Python) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        if let Some(nodes_by_alias) = self.model_access.get_nodes_by_alias() {
            for (alias, node) in nodes_by_alias {
                let node_json = serde_json::to_string(node.as_ref()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize node: {}",
                        e
                    ))
                })?;
                dict.set_item(alias, node_json)?;
            }
        }
        Ok(dict.to_object(py))
    }

    /// Set permitted nodegroups with support for both boolean and conditional rules.
    ///
    /// Accepts a dict where values can be:
    /// - bool: True/False for simple allow/deny
    /// - dict: {"path": ".data.uuid.field", "allowed": ["value1", "value2"]}
    ///   for conditional filtering based on tile data
    ///
    /// Raises ValueError if any permission rule is invalid.
    fn set_permitted_nodegroups(&mut self, py: Python, permissions: PyObject) -> PyResult<()> {
        let perms = instance_wrapper_py::parse_permissions_dict(py, &permissions)?;
        self.model_access.set_permitted_nodegroups_rules(perms);
        Ok(())
    }

    /// Check if a nodegroup is permitted (for nodegroup-level filtering)
    fn is_nodegroup_permitted(&self, nodegroup_id: &str) -> bool {
        self.model_access.is_nodegroup_permitted(nodegroup_id)
    }

    /// Check if a specific tile is permitted by the permission rules.
    ///
    /// Args:
    ///     nodegroup_id: The nodegroup ID
    ///     tile_json: JSON string of the tile data
    ///
    /// Returns:
    ///     True if the tile is permitted
    fn is_tile_permitted(&self, py: Python, nodegroup_id: &str, tile_json: &str) -> PyResult<bool> {
        let tile: AlizarinStaticTile = serde_json::from_str(tile_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse tile JSON: {}",
                e
            ))
        })?;

        let result = self.model_access.is_tile_permitted(&tile);

        // Debug logging for conditional filtering
        if let Some(rule) = self.model_access.get_permission_rule(nodegroup_id) {
            if let PermissionRule::Conditional { path, allowed } = rule {
                let tile_id = tile.tileid.as_deref().unwrap_or("(no id)");
                let value = evaluate_tile_path(&tile, path);

                if let Ok(logging) = py.import_bound("logging") {
                    if let Ok(logger) = logging.call_method1("getLogger", ("alizarin",)) {
                        let _ = logger.call_method1("debug",
                            (format!(
                                "[alizarin] Conditional filter: nodegroup={}, tile={}, path={}, value={:?}, allowed={:?}, permitted={}",
                                nodegroup_id, tile_id, path, value, allowed, result
                            ),)
                        );
                    }
                }
            }
        }

        Ok(result)
    }

    /// Prune graph to only include permitted nodegroups.
    fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> PyResult<()> {
        self.model_access
            .prune_graph(keep_functions.as_deref())
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to prune graph: {}",
                    e
                ))
            })
    }

    /// Export graph as JSON
    fn export_graph(&self, py: Python) -> PyResult<PyObject> {
        let graph = self.model_access.get_graph();
        pythonize::pythonize(py, graph)
            .map(|obj| obj.into())
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to convert graph to Python: {}",
                    e
                ))
            })
    }
}

// parse_conditional_rule removed — permission parsing is now centralised
// in instance_wrapper_py::parse_permissions_dict

/// Static method: Check if a tile/node matches as a semantic child
///
/// Note: `parent_nodegroup_id` was previously named `parent_node_id` - callers
/// should pass the nodegroup ID of the parent node, not the node ID.
#[pyfunction]
#[pyo3(signature = (parent_tile_id, parent_nodegroup_id, child_node_json, tile_json))]
fn matches_semantic_child(
    parent_tile_id: Option<String>,
    parent_nodegroup_id: Option<String>,
    child_node_json: String,
    tile_json: String,
) -> PyResult<bool> {
    let child_node: AlizarinStaticNode = serde_json::from_str(&child_node_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to parse child node: {}",
            e
        ))
    })?;

    let tile: AlizarinStaticTile = serde_json::from_str(&tile_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse tile: {}", e))
    })?;

    Ok(core_matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        &tile,
    ))
}

/// Resolve labels to UUIDs in a JSON tree using an RDM cache
///
/// This pre-processes a tree by looking up label strings in the appropriate
/// RDM collection (based on node config) and replacing them with UUIDs.
///
/// Args:
///     tree_json: The input tree as JSON string
///     graph_id: Graph ID (must be registered via register_graph first)
///     rdm_cache: The RDM cache containing collections (optional, uses global if not provided)
///     strict: If true, fail on unresolved labels; if false, leave them as-is
///
/// Returns:
///     The tree with labels resolved to UUIDs (as JSON string)
#[pyfunction]
#[pyo3(signature = (tree_json, graph_id, rdm_cache=None, strict = false))]
fn resolve_labels_in_tree(
    tree_json: String,
    graph_id: String,
    rdm_cache: Option<&rdm_cache_py::RdmCache>,
    strict: bool,
) -> PyResult<String> {
    // Parse tree
    let tree: serde_json::Value = serde_json::from_str(&tree_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse tree: {}", e))
    })?;

    // Get graph from registry
    let graph = get_registered_graph(&graph_id)?;

    // Use explicit cache or fall back to global
    let global_cache = rdm_cache_py::get_global_rdm_cache();
    let cache_to_use: Option<&rdm_cache_py::RdmCache> = rdm_cache.or(global_cache.as_ref());

    let resolved_tree = if let Some(cache) = cache_to_use {
        let alias_map = build_alias_to_collection_from_graph(&graph);
        core_resolve_labels(tree, &alias_map, cache, strict)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.message))?
    } else {
        tree
    };

    serde_json::to_string(&resolved_tree).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to serialize result: {}",
            e
        ))
    })
}

/// Register a graph in the core registry for batch_merge_resources descriptor computation
///
/// Call this once per graph before using batch_merge_resources with recompute_descriptors=True.
/// The graph will be looked up by its graph_id when computing descriptors.
///
/// Args:
///     graph_json: Graph model as JSON string (can be direct object or {"graph": [...]})
///
/// Returns:
///     The graph_id that was registered
#[pyfunction]
#[pyo3(signature = (graph_json,))]
fn register_graph(graph_json: String) -> PyResult<String> {
    let graph = AlizarinCoreStaticGraph::from_json_string(&graph_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse graph: {}", e))
    })?;

    let graph_id = graph.graphid.clone();
    alizarin_core::register_graph_owned(graph);

    Ok(graph_id)
}

/// Set a descriptor template on a registered graph.
///
/// Creates or updates the descriptor function config for the given type.
/// The nodegroup_id is inferred from the <Node Name> placeholders in the template.
/// All placeholder nodes must belong to exactly one nodegroup.
///
/// Args:
///     graph_id: The graph ID to update
///     descriptor_type: Type name (e.g. "slug", "name", "description", "map_popup")
///     string_template: Template string with <Node Name> placeholders
#[pyfunction]
#[pyo3(signature = (graph_id, descriptor_type, string_template))]
fn set_descriptor_template(
    graph_id: &str,
    descriptor_type: &str,
    string_template: &str,
) -> PyResult<()> {
    let graph_arc = alizarin_core::get_graph(graph_id).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Graph '{}' not found in registry",
            graph_id
        ))
    })?;
    let mut graph = (*graph_arc).clone();
    graph
        .set_descriptor_template(descriptor_type, string_template)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    alizarin_core::register_graph_owned(graph);
    Ok(())
}

/// Register a datatype as a list type.
///
/// List types are datatypes where the array IS the value (not multiple items).
/// Extensions should call this at initialization for their custom list datatypes.
///
/// Example:
///     alizarin.register_list_datatype("reference")  # CLM reference type
#[pyfunction]
fn register_list_datatype(datatype: &str) {
    alizarin_core::register_list_datatype(datatype);
}

/// Check if a datatype is a registered list type.
///
/// Returns True if arrays for this datatype should be treated as single values
/// rather than iterated over during tree-to-tiles conversion.
#[pyfunction]
fn is_list_datatype(datatype: &str) -> bool {
    alizarin_core::is_list_datatype(datatype)
}

/// Get all registered list datatypes.
#[pyfunction]
fn list_datatypes() -> Vec<String> {
    alizarin_core::list_datatypes()
}

// =============================================================================
// Widget Registry Functions
// =============================================================================

/// Register a widget mapping for a datatype.
///
/// Extensions should call this at initialization to register their custom
/// datatype-to-widget mappings. This allows `get_default_widget_for_datatype`
/// in graph mutations to find the correct widget for extension datatypes.
///
/// Example:
///     # In CLM extension initialization:
///     alizarin.register_widget_for_datatype("reference", "reference-select-widget")
///     alizarin.register_widget_for_datatype("reference-list", "reference-multiselect-widget")
#[pyfunction]
fn register_widget_for_datatype(datatype: &str, widget_name: &str) {
    alizarin_core::register_widget_for_datatype(datatype, widget_name);
}

/// Get the registered widget name for a datatype.
///
/// Returns None if no widget is registered for this datatype.
#[pyfunction]
fn get_widget_for_datatype(datatype: &str) -> Option<String> {
    alizarin_core::get_widget_for_datatype(datatype)
}

/// Register a widget definition.
///
/// Extensions should call this at initialization to register their custom widgets.
/// This allows the mutation system to find extension widgets when creating
/// nodes with extension datatypes.
///
/// Args:
///     widget_id: UUID of the widget (should be a valid UUID string)
///     widget_name: Name of the widget (e.g., "reference-select-widget")
///     datatype: The datatype this widget handles
///     default_config_json: JSON string of default config (or "{}" for empty)
///
/// Example:
///     # In CLM extension initialization:
///     alizarin.register_widget(
///         "10000000-0000-0000-0000-000000000017",
///         "reference-select-widget",
///         "reference",
///         '{"placeholder": "Select a reference"}'
///     )
#[pyfunction]
fn register_widget(widget_id: &str, widget_name: &str, datatype: &str, default_config_json: &str) {
    let widget =
        alizarin_core::RegisteredWidget::new(widget_id, widget_name, datatype, default_config_json);
    alizarin_core::register_widget(widget);
}

/// Get all registered widget names.
#[pyfunction]
fn registered_widgets() -> Vec<String> {
    alizarin_core::registered_widgets()
}

/// Get all registered widget mappings (datatype -> widget_name).
#[pyfunction]
fn widget_mappings() -> Vec<(String, String)> {
    alizarin_core::widget_mappings()
}

// sort_json_keys is provided by alizarin_core::sort_json_keys
use alizarin_core::sort_json_keys;

/// Get the JSON representation of a registered graph.
///
/// The output is deterministic (sorted keys) for reliable git diffing.
///
/// Args:
///     graph_id: The UUID of the graph to retrieve
///
/// Returns:
///     The graph as a JSON string
///
/// Raises:
///     ValueError: If the graph is not registered
///
/// Example:
///     graph_json = get_graph_json("12345678-1234-1234-1234-123456789012")
///     data = json.loads(graph_json)
#[pyfunction]
#[pyo3(signature = (graph_id,))]
fn get_graph_json(graph_id: String) -> PyResult<String> {
    let graph = alizarin_core::get_graph(&graph_id).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Graph '{}' not registered. Call register_graph() first.",
            graph_id
        ))
    })?;

    // Serialize to Value first, then sort all keys for deterministic output
    let value = serde_json::to_value(&*graph).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize graph: {}", e))
    })?;

    let sorted = sort_json_keys(value);

    serde_json::to_string_pretty(&sorted).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize graph: {}", e))
    })
}

/// Get a simplified schema view of a registered graph showing node aliases and structure.
///
/// Returns a nested dictionary representing the tree structure with:
/// - Keys are node aliases (or nodeid if no alias)
/// - Values are dicts with 'datatype', 'nodeid', optionally 'required', and 'children'
///
/// Useful for understanding what keys are available in batch_tiles_to_trees output.
///
/// Args:
///     graph_id: The graph UUID that was returned from register_graph()
///
/// Returns:
///     dict: Nested structure showing node hierarchy with aliases and datatypes
///
/// Example:
///     schema = get_graph_schema("12345678-1234-1234-1234-123456789012")
///     print(json.dumps(schema, indent=2))
#[pyfunction]
#[pyo3(signature = (graph_id,))]
fn get_graph_schema(py: Python, graph_id: String) -> PyResult<PyObject> {
    let graph = get_registered_graph(&graph_id)?;
    let schema = graph.get_schema();
    pythonize::pythonize(py, &schema).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

/// Merge multiple resources with the same resourceinstanceid into one
///
/// Concatenates tiles from all resources, detecting and warning about duplicate tileids.
/// All resources must have the same resourceinstanceid.
///
/// Args:
///     resources_json: JSON string containing array of StaticResource objects
///
/// Returns:
///     Dict with 'resource' (merged StaticResource) and 'warnings' (list of duplicate warnings)
#[pyfunction]
#[pyo3(signature = (resources_json,))]
fn merge_resources(py: Python, resources_json: String) -> PyResult<PyObject> {
    let resources: Vec<AlizarinStaticResource> =
        serde_json::from_str(&resources_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse resources: {}",
                e
            ))
        })?;

    match core_merge_resources(resources) {
        Ok(result) => {
            let output = serde_json::json!({
                "resource": result.resource,
                "warnings": result.warnings
            });
            pythonize::pythonize(py, &output).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to convert to Python: {}",
                    e
                ))
            })
        }
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
    }
}

/// Batch merge resources from multiple JSON strings
///
/// Takes an array of JSON strings. Each string can be:
/// - An array of StaticResource objects: [{...}, {...}]
/// - A single StaticResource object: {resourceinstance: {...}, tiles: [...]}
/// - A BusinessDataWrapper: {business_data: {resources: [...]}}
///
/// Groups all resources by resourceinstanceid and merges each group.
///
/// Args:
///     batches_json: List of JSON strings containing resources in any supported format
///     recompute_descriptors: If True, recomputes descriptors from merged tiles using
///         graphs from the registry. Graphs must be registered first.
///     strict: If True (default), raises ValueError on conflicting data when unifying
///         cardinality-1 tiles.
///
/// Returns:
///     Dict with 'resources' (list of merged StaticResources) and 'warnings' (list of warnings)
#[pyfunction]
#[pyo3(signature = (batches_json, recompute_descriptors=false, strict=true))]
fn batch_merge_resources(
    py: Python,
    batches_json: Vec<String>,
    recompute_descriptors: bool,
    strict: bool,
) -> PyResult<PyObject> {
    let mut resource_batches: Vec<Vec<AlizarinStaticResource>> = Vec::new();
    for (i, batch_str) in batches_json.iter().enumerate() {
        let batch = core_parse_resources(batch_str).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Batch {}: {}", i, e))
        })?;
        resource_batches.push(batch);
    }

    let result = core_batch_merge_resources(resource_batches, recompute_descriptors, strict);
    batch_merge_result_to_py(py, result)
}

/// Merge resources from multiple JSON files, reading one file at a time.
///
/// Memory-efficient alternative to loading all files and calling batch_merge_resources.
/// Each file is read from disk, parsed, and the raw JSON is freed before the next file
/// is processed. Only the accumulated StaticResource structs persist between files.
///
/// If output_path is provided, writes the result directly to a JSON file from Rust
/// (avoiding the large Python dict conversion) and returns only warnings.
///
/// Args:
///     file_paths: List of paths to JSON files containing resources
///     chunk_size: Number of files to merge per flush (default 1 = flush after every file)
///     recompute_descriptors: If True, recomputes descriptors in a final pass
///     strict: If True (default), raises ValueError on conflicting data
///     output_path: If provided, writes result JSON to this path and returns warnings only
///
/// Returns:
///     If output_path is None: Dict with 'resources' and 'warnings'
///     If output_path is set: Dict with 'warnings' and 'output_path'
#[pyfunction]
#[pyo3(signature = (file_paths, chunk_size=1, recompute_descriptors=true, strict=true, output_path=None))]
fn streamed_merge_from_files(
    py: Python,
    file_paths: Vec<String>,
    chunk_size: usize,
    recompute_descriptors: bool,
    strict: bool,
    output_path: Option<String>,
) -> PyResult<PyObject> {
    let mut accumulator = MergeAccumulator::new(chunk_size, strict);

    for (i, path) in file_paths.iter().enumerate() {
        let content = std::fs::read_to_string(path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to read file '{}' (index {}): {}",
                path, i, e
            ))
        })?;

        accumulator.add_json(&content).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse file '{}' (index {}): {}",
                path, i, e
            ))
        })?;
        // `content` dropped here — raw JSON string freed before next file
    }

    let result = accumulator.finish(recompute_descriptors);

    if let Some(ref error) = result.error {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            error.clone(),
        ));
    }

    if let Some(ref out_path) = output_path {
        // Write directly to file from Rust — serializes straight from the structs
        // to disk, avoiding any intermediate serde_json::Value or Python dict.
        let file = std::fs::File::create(out_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create output file '{}': {}",
                out_path, e
            ))
        })?;
        let writer = std::io::BufWriter::new(file);

        #[derive(serde::Serialize)]
        struct BusinessDataWrapper<'a> {
            business_data: ResourcesWrapper<'a>,
        }
        #[derive(serde::Serialize)]
        struct ResourcesWrapper<'a> {
            resources: &'a [AlizarinStaticResource],
        }

        let wrapper = BusinessDataWrapper {
            business_data: ResourcesWrapper {
                resources: &result.resources,
            },
        };
        serde_json::to_writer_pretty(writer, &wrapper).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to write output file '{}': {}",
                out_path, e
            ))
        })?;

        let warnings_output = serde_json::json!({
            "warnings": result.warnings,
            "output_path": out_path,
        });
        pythonize::pythonize(py, &warnings_output).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to convert warnings to Python: {}",
                e
            ))
        })
    } else {
        batch_merge_result_to_py(py, result)
    }
}

/// Convert a BatchMergeResult to a Python dict.
fn batch_merge_result_to_py(
    py: Python,
    result: alizarin_core::BatchMergeResult,
) -> PyResult<PyObject> {
    if let Some(ref error) = result.error {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            error.clone(),
        ));
    }

    let output = serde_json::json!({
        "resources": result.resources,
        "warnings": result.warnings
    });
    pythonize::pythonize(py, &output).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to convert to Python: {}",
            e
        ))
    })
}

// ============================================================================
// Prebuild Exporter
// ============================================================================

/// Export registered graphs to a directory.
///
/// Classifies graphs as resource_models or branches based on `isresource`,
/// writes as `{"graph": [graph_data]}` JSON files with deterministically sorted keys.
#[pyfunction]
#[pyo3(signature = (graph_ids, out_dir))]
fn export_graphs_to_dir(graph_ids: Vec<String>, out_dir: String) -> PyResult<Vec<String>> {
    let files = alizarin_core::export_graphs(&graph_ids)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    let export_data = alizarin_core::PrebuildExportData {
        graph_files: files,
        ..Default::default()
    };
    alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Export all registered graphs to a directory.
#[pyfunction]
#[pyo3(signature = (out_dir,))]
fn export_all_graphs_to_dir(out_dir: String) -> PyResult<Vec<String>> {
    let files = alizarin_core::export_all_graphs()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    let export_data = alizarin_core::PrebuildExportData {
        graph_files: files,
        ..Default::default()
    };
    alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Export RDM collections from the global cache as SKOS XML files.
#[pyfunction]
#[pyo3(signature = (out_dir, base_uri))]
fn export_collections_to_dir(out_dir: String, base_uri: String) -> PyResult<Vec<String>> {
    let cache = rdm_cache_py::get_global_rdm_cache().ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Global RDM cache not set. Call set_global_rdm_cache() first.",
        )
    })?;

    let files = alizarin_core::export_collections(cache.inner(), &base_uri)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let export_data = alizarin_core::PrebuildExportData {
        reference_data_files: files,
        ..Default::default()
    };
    alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Export a complete prebuild package (graphs + collections) to a directory.
#[pyfunction]
#[pyo3(signature = (out_dir, base_uri, graph_ids=None))]
fn export_prebuild(
    out_dir: String,
    base_uri: String,
    graph_ids: Option<Vec<String>>,
) -> PyResult<Vec<String>> {
    let rdm_cache = rdm_cache_py::get_global_rdm_cache();
    let core_cache = rdm_cache.as_ref().map(|c| c.inner().clone());

    alizarin_core::export_prebuild_to_directory(
        graph_ids.as_deref(),
        core_cache.as_ref(),
        &base_uri,
        std::path::Path::new(&out_dir),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Import a prebuild/pkg directory: register graphs, load SKOS collections, and load ontologies.
///
/// This is the inverse of `export_prebuild`. It reads the directory structure and:
/// 1. Loads and registers all graphs from graphs/resource_models/ and graphs/branches/
/// 2. Parses SKOS XML from reference_data/collections/ and adds to the global RDM cache
/// 3. Loads ontology RDFS files from ontologies/ (if present) and returns a validator
///
/// Args:
///     prebuild_dir: Path to the prebuild or pkg directory
///     base_uri: Base URI for SKOS XML parsing (default: "http://localhost:8000/")
///
/// Returns:
///     dict with keys:
///         graph_ids (list[str]): IDs of registered graphs
///         collection_ids (list[str]): IDs of loaded collections
///         ontology_validator (OntologyValidator | None): validator if ontologies were found
#[pyfunction]
#[pyo3(signature = (prebuild_dir, base_uri="http://localhost:8000/"))]
fn import_prebuild(py: Python, prebuild_dir: String, base_uri: &str) -> PyResult<PyObject> {
    let result = alizarin_core::import_prebuild(&prebuild_dir, base_uri)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let ontology_validator: Option<graph_mutator_py::PyOntologyValidator> = result
        .ontology_validators
        .into_iter()
        .next()
        .map(graph_mutator_py::PyOntologyValidator::from_inner);

    let dict = pyo3::types::PyDict::new(py);
    dict.set_item("graph_ids", result.graph_ids)?;
    dict.set_item("collection_ids", result.collection_ids)?;
    dict.set_item("ontology_validator", ontology_validator.into_py(py))?;
    Ok(dict.into())
}

/// Get IDs of all registered graphs.
#[pyfunction]
fn get_registered_graph_ids() -> Vec<String> {
    alizarin_core::get_registered_graph_ids()
}

/// Python module definition
#[pymodule]
fn alizarin(_py: Python, m: &PyModule) -> PyResult<()> {
    // Graph registry (for batch_merge_resources descriptor computation)
    m.add_function(wrap_pyfunction!(register_graph, m)?)?;
    m.add_function(wrap_pyfunction!(get_graph_json, m)?)?;
    m.add_function(wrap_pyfunction!(get_graph_schema, m)?)?;
    m.add_function(wrap_pyfunction!(set_descriptor_template, m)?)?;
    m.add_function(wrap_pyfunction!(get_registered_graph_ids, m)?)?;

    // Prebuild exporter / importer
    m.add_function(wrap_pyfunction!(export_graphs_to_dir, m)?)?;
    m.add_function(wrap_pyfunction!(export_all_graphs_to_dir, m)?)?;
    m.add_function(wrap_pyfunction!(export_collections_to_dir, m)?)?;
    m.add_function(wrap_pyfunction!(export_prebuild, m)?)?;
    m.add_function(wrap_pyfunction!(import_prebuild, m)?)?;

    // List datatype registry (for datatypes where array IS the value)
    m.add_function(wrap_pyfunction!(register_list_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(is_list_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(list_datatypes, m)?)?;

    // Widget registry (for extension datatype -> widget mappings)
    m.add_function(wrap_pyfunction!(register_widget_for_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(get_widget_for_datatype, m)?)?;
    m.add_function(wrap_pyfunction!(register_widget, m)?)?;
    m.add_function(wrap_pyfunction!(registered_widgets, m)?)?;
    m.add_function(wrap_pyfunction!(widget_mappings, m)?)?;

    // Low-level tree conversion functions (for compatibility)
    m.add_function(wrap_pyfunction!(tiles_to_json_tree, m)?)?;
    m.add_function(wrap_pyfunction!(build_tree_from_tiles, m)?)?;
    m.add_function(wrap_pyfunction!(json_tree_to_tiles, m)?)?;

    // Batch conversion functions (parallel processing with Rayon)
    m.add_function(wrap_pyfunction!(batch_trees_to_tiles, m)?)?;
    m.add_function(wrap_pyfunction!(batch_tiles_to_trees, m)?)?;
    m.add_function(wrap_pyfunction!(cards_to_json_tree, m)?)?;

    // Streaming iterator (low memory, processes one tree at a time)
    m.add_class::<TreeToTilesIterator>()?;

    // Resource registry (for relationship resolution and cache population)
    m.add_class::<ResourceRegistry>()?;

    // Resource merging (combine tiles from multiple resources)
    m.add_function(wrap_pyfunction!(merge_resources, m)?)?;
    m.add_function(wrap_pyfunction!(batch_merge_resources, m)?)?;
    m.add_function(wrap_pyfunction!(streamed_merge_from_files, m)?)?;

    // Label resolution (resolve concept labels to UUIDs before conversion)
    m.add_function(wrap_pyfunction!(resolve_labels_in_tree, m)?)?;

    // Wrapper classes
    m.add_class::<PyResourceModelWrapper>()?;

    // Utility functions
    m.add_function(wrap_pyfunction!(matches_semantic_child, m)?)?;

    // Extension type handler registration
    m.add_function(wrap_pyfunction!(register_extension_handler, m)?)?;
    m.add_function(wrap_pyfunction!(has_extension_handler, m)?)?;
    m.add_function(wrap_pyfunction!(coerce_with_extension, m)?)?;
    m.add_function(wrap_pyfunction!(has_display_renderer, m)?)?;
    m.add_function(wrap_pyfunction!(render_display_with_extension, m)?)?;
    m.add_function(wrap_pyfunction!(get_registered_extension_handlers, m)?)?;

    // Node configuration management (Phase 0 of type coercion)
    node_config_py::register_module(m)?;

    // RDM cache for concept collections (Phase 0 of type coercion)
    rdm_cache_py::register_module(m)?;

    // Type coercion functions (Phase 1: simple scalars)
    type_coercion_py::register_module(m)?;

    // Rust-backed pseudo values (single source of truth for matching_entries)
    pseudo_value_py::register_module(m)?;

    // Rust-backed instance wrapper (tile processing, pseudo cache population)
    instance_wrapper_py::register_module(m)?;

    // Graph mutation API (JSON-based)
    graph_mutator_py::register_module(m)?;

    // SKOS RDF/XML parsing and serialization
    skos_py::register_module(m)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use alizarin_core::string_utils::{camel_to_snake, transform_keys_to_snake};

    #[test]
    fn test_camel_to_snake_simple() {
        assert_eq!(camel_to_snake("camelCase"), "camel_case");
        assert_eq!(camel_to_snake("firstName"), "first_name");
        assert_eq!(camel_to_snake("lastName"), "last_name");
    }

    #[test]
    fn test_camel_to_snake_already_snake() {
        assert_eq!(camel_to_snake("snake_case"), "snake_case");
        assert_eq!(camel_to_snake("first_name"), "first_name");
    }

    #[test]
    fn test_camel_to_snake_single_word() {
        assert_eq!(camel_to_snake("name"), "name");
        assert_eq!(camel_to_snake("Name"), "name");
    }

    #[test]
    fn test_camel_to_snake_mixed() {
        // Note: The current implementation treats each uppercase letter as a word boundary
        // This is correct for standard camelCase used in JSON keys
        assert_eq!(camel_to_snake("graphId"), "graph_id");
        assert_eq!(camel_to_snake("resourceInstanceId"), "resource_instance_id");
        assert_eq!(camel_to_snake("nodeGroupId"), "node_group_id");
    }

    #[test]
    fn test_transform_keys_nested() {
        let input = serde_json::json!({
            "firstName": "John",
            "lastName": "Doe",
            "contactInfo": {
                "emailAddress": "john@example.com",
                "phoneNumber": "123-456"
            },
            "addresses": [
                {"streetName": "Main St", "zipCode": "12345"}
            ]
        });

        let result = transform_keys_to_snake(input);

        assert_eq!(result["first_name"], "John");
        assert_eq!(result["last_name"], "Doe");
        assert_eq!(result["contact_info"]["email_address"], "john@example.com");
        assert_eq!(result["contact_info"]["phone_number"], "123-456");
        assert_eq!(result["addresses"][0]["street_name"], "Main St");
        assert_eq!(result["addresses"][0]["zip_code"], "12345");
    }
}
