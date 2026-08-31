//! Shared C-ABI extension-handler machinery.
//!
//! Wraps a raw [`TypeHandlerInfo`] (fn-pointers exported by an extension's
//! cdylib) as an [`ExtensionTypeHandler`], so every binding (Python via
//! `PyCapsule`, NAPI via a pointer passed from JS) shares ONE copy of this
//! `unsafe` plumbing instead of duplicating it. Values cross the boundary by
//! pointer where possible — no per-value serialization beyond what each fn needs.
//!
//! Lifted from the per-binding implementations; `core` depends on the leaf
//! `alizarin-extension-api` crate for the ABI types, so extensions themselves
//! still only need `alizarin-extension-api` (not all of core).

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, RwLock};

use alizarin_extension_api::{
    abi_fingerprint, CoerceFn, FreeDisplayFn, FreeFn, FreeIndexSpecFn, FreeRenderSearchFn,
    FreeResolveMarkersFn, FreeValidateFn, IndexSpecFn, RenderDisplayFn, RenderSearchFn,
    ResolveMarkersFn, TypeHandlerInfo, ValidateFn,
};

use crate::extension_type_registry::{
    ExtensionError, ExtensionTypeHandler, ExtensionTypeRegistry, HandlerCapabilities,
};

lazy_static::lazy_static! {
    /// Registered C-ABI handlers, keyed by datatype name.
    static ref TYPE_HANDLERS: RwLock<HashMap<String, RegisteredHandler>> =
        RwLock::new(HashMap::new());
}

struct RegisteredHandler {
    /// Coercion (None for a validate-only handler; core then coerces this type).
    coerce_fn: Option<CoerceFn>,
    free_fn: Option<FreeFn>,
    /// Optional display renderer (for toDisplayJson support)
    render_display_fn: Option<RenderDisplayFn>,
    free_display_fn: Option<FreeDisplayFn>,
    /// Optional marker resolver (for resolving __needs_rdm_lookup etc.)
    resolve_markers_fn: Option<ResolveMarkersFn>,
    free_resolve_markers_fn: Option<FreeResolveMarkersFn>,
    /// Optional validator (value passed by pointer — no per-value serialization).
    validate_fn: Option<ValidateFn>,
    free_validate_fn: Option<FreeValidateFn>,
    /// Optional search renderer (value by pointer; returns search-indexable JSON).
    render_search_fn: Option<RenderSearchFn>,
    free_render_search_fn: Option<FreeRenderSearchFn>,
    /// Optional index-spec declarer (value by pointer; returns class + raw keys).
    index_spec_fn: Option<IndexSpecFn>,
    free_index_spec_fn: Option<FreeIndexSpecFn>,
}

// =============================================================================
// ExtensionTypeHandler Implementation for Python
// =============================================================================

/// Wrapper that implements ExtensionTypeHandler using C ABI callbacks.
///
/// This allows Python extension handlers to be used with the unified
/// ExtensionTypeRegistry from alizarin-core.
pub struct AbiExtensionTypeHandler {
    type_name: String,
}

impl AbiExtensionTypeHandler {
    pub fn new(type_name: String) -> Self {
        Self { type_name }
    }
}

// The C ABI callbacks are thread-safe (they're compiled code)
unsafe impl Send for AbiExtensionTypeHandler {}
unsafe impl Sync for AbiExtensionTypeHandler {}

impl ExtensionTypeHandler for AbiExtensionTypeHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        let handlers = TYPE_HANDLERS.read().unwrap();
        if let Some(handler) = handlers.get(&self.type_name) {
            HandlerCapabilities {
                can_coerce: handler.coerce_fn.is_some(),
                can_render_display: handler.render_display_fn.is_some(),
                can_render_search: handler.render_search_fn.is_some(),
                can_resolve_markers: handler.resolve_markers_fn.is_some(),
                can_index: handler.index_spec_fn.is_some(),
                can_validate: handler.validate_fn.is_some(),
            }
        } else {
            HandlerCapabilities::default()
        }
    }

    fn coerce(
        &self,
        value: &serde_json::Value,
        config: Option<&serde_json::Value>,
    ) -> Result<crate::CoercionResult, ExtensionError> {
        let handlers = TYPE_HANDLERS.read().unwrap();

        if let Some(handler) = handlers.get(&self.type_name) {
            let (Some(coerce_fn), Some(free_fn)) = (handler.coerce_fn, handler.free_fn) else {
                return Err(ExtensionError::new(format!(
                    "handler '{}' has no coerce function",
                    self.type_name
                )));
            };
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
                let result = (coerce_fn)(
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
                    (free_fn)(result);

                    Ok(crate::CoercionResult::success(tile_data, display_value))
                } else {
                    let error_msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        result.error_ptr,
                        result.error_len,
                    ))
                    .to_string();

                    (free_fn)(result);

                    Err(ExtensionError::new(error_msg))
                }
            }
        } else {
            // No handler - pass through
            Ok(crate::CoercionResult::success(value.clone(), value.clone()))
        }
    }

    fn render_display(
        &self,
        tile_data: &serde_json::Value,
        language: &str,
        _ctx: Option<&crate::type_serialization::SerializationContext>,
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

                // Cheap Arc handle to the global RDM cache — a refcount bump, not
                // a per-value deep copy of every collection/concept (which made
                // this the batch hotpath's real bottleneck).
                let core_cache: Option<Arc<crate::rdm_cache::RdmCache>> =
                    crate::get_global_rdm_cache_arc();

                // SAFETY: Calling resolve_markers FFI function pointer registered via PyCapsule.
                // cache_ptr is either null or points to an Arc<crate::rdm_cache::RdmCache> kept alive by
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

    fn validate(
        &self,
        value: &serde_json::Value,
        config: Option<&serde_json::Value>,
    ) -> Result<crate::extension_type_registry::ValidationResult, ExtensionError> {
        use crate::extension_type_registry::ValidationResult;

        // Copy out the fn pointers, then drop the lock before crossing the FFI.
        let (validate_fn, free_validate_fn) = {
            let handlers = TYPE_HANDLERS.read().unwrap();
            match handlers.get(&self.type_name) {
                Some(h) => (h.validate_fn, h.free_validate_fn),
                None => return Ok(ValidationResult::valid()),
            }
        };
        let (Some(validate_fn), Some(free_validate_fn)) = (validate_fn, free_validate_fn) else {
            return Ok(ValidationResult::valid());
        };

        // Pass the value (and config) BY POINTER — no serialization on the hot
        // path. Sound because the ABI handshake at load confirmed the extension's
        // `serde_json::Value` layout matches ours.
        let value_ptr = value as *const serde_json::Value as *const c_void;
        let config_ptr = config
            .map(|c| c as *const serde_json::Value as *const c_void)
            .unwrap_or(std::ptr::null());

        // SAFETY: value_ptr/config_ptr point to live `serde_json::Value`s for the
        // duration of this call; the ext casts them back to the same type (ABI
        // confirmed). The small result is freed via free_validate_fn.
        unsafe {
            let result = validate_fn(value_ptr, config_ptr);
            if !result.error_ptr.is_null() {
                let msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    result.error_ptr,
                    result.error_len,
                ))
                .to_string();
                free_validate_fn(result);
                return Err(ExtensionError::new(msg));
            }
            let json = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                result.json_ptr,
                result.json_len,
            ));
            let parsed: Result<ValidationResult, ExtensionError> = serde_json::from_str(json)
                .map_err(|e| {
                    ExtensionError::new(format!("Failed to parse ValidationResult: {}", e))
                });
            free_validate_fn(result);
            parsed
        }
    }

    fn render_search(
        &self,
        tile_data: &serde_json::Value,
        language: &str,
    ) -> Result<Option<serde_json::Value>, ExtensionError> {
        let (render_search_fn, free_render_search_fn) = {
            let handlers = TYPE_HANDLERS.read().unwrap();
            match handlers.get(&self.type_name) {
                Some(h) => (h.render_search_fn, h.free_render_search_fn),
                None => return Ok(None),
            }
        };
        let (Some(render_search_fn), Some(free_render_search_fn)) =
            (render_search_fn, free_render_search_fn)
        else {
            return Ok(None);
        };

        // Value BY POINTER (ABI-confirmed); language as UTF-8 bytes.
        let value_ptr = tile_data as *const serde_json::Value as *const c_void;
        // SAFETY: pointers are live for the call; the ext casts the value back to
        // the same `serde_json::Value` type; the result is freed below.
        unsafe {
            let result = render_search_fn(value_ptr, language.as_ptr(), language.len());
            if !result.error_ptr.is_null() {
                let msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    result.error_ptr,
                    result.error_len,
                ))
                .to_string();
                free_render_search_fn(result);
                return Err(ExtensionError::new(msg));
            }
            let json = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                result.json_ptr,
                result.json_len,
            ));
            let parsed: Result<Option<serde_json::Value>, ExtensionError> =
                serde_json::from_str(json).map_err(|e| {
                    ExtensionError::new(format!("Failed to parse render_search result: {}", e))
                });
            free_render_search_fn(result);
            parsed
        }
    }

    fn index_spec(
        &self,
        tile_data: &serde_json::Value,
        config: Option<&serde_json::Value>,
    ) -> Result<Option<crate::extension_type_registry::IndexSpec>, ExtensionError> {
        use crate::extension_type_registry::IndexSpec;

        let (index_spec_fn, free_index_spec_fn) = {
            let handlers = TYPE_HANDLERS.read().unwrap();
            match handlers.get(&self.type_name) {
                Some(h) => (h.index_spec_fn, h.free_index_spec_fn),
                None => return Ok(None),
            }
        };
        let (Some(index_spec_fn), Some(free_index_spec_fn)) = (index_spec_fn, free_index_spec_fn)
        else {
            return Ok(None);
        };

        // Value and config BY POINTER (ABI-confirmed).
        let value_ptr = tile_data as *const serde_json::Value as *const c_void;
        let config_ptr = config
            .map(|c| c as *const serde_json::Value as *const c_void)
            .unwrap_or(std::ptr::null());
        // SAFETY: pointers live for the call; the ext casts them back to the same
        // type; the result is freed below.
        unsafe {
            let result = index_spec_fn(value_ptr, config_ptr);
            if !result.error_ptr.is_null() {
                let msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    result.error_ptr,
                    result.error_len,
                ))
                .to_string();
                free_index_spec_fn(result);
                return Err(ExtensionError::new(msg));
            }
            let json = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                result.json_ptr,
                result.json_len,
            ));
            let parsed: Result<Option<IndexSpec>, ExtensionError> = serde_json::from_str(json)
                .map_err(|e| {
                    ExtensionError::new(format!("Failed to parse IndexSpec result: {}", e))
                });
            free_index_spec_fn(result);
            parsed
        }
    }

    fn description(&self) -> &str {
        "Python extension type handler (C ABI)"
    }
}

/// Build a fresh [`ExtensionTypeRegistry`] from the registered C-ABI handlers.
pub fn build_extension_registry_from_registered() -> ExtensionTypeRegistry {
    let mut registry = ExtensionTypeRegistry::new();
    let handlers = TYPE_HANDLERS.read().unwrap();
    for type_name in handlers.keys() {
        registry.register(
            type_name.clone(),
            Arc::new(AbiExtensionTypeHandler::new(type_name.clone())),
        );
    }
    registry
}

/// Is a handler registered for `type_name`?
pub fn has_registered_handler(type_name: &str) -> bool {
    TYPE_HANDLERS.read().unwrap().contains_key(type_name)
}

/// Register a C-ABI type handler from a raw [`TypeHandlerInfo`] pointer, then
/// rebuild + install the global extension registry. Returns the datatype name.
///
/// # Safety
/// `ptr` must point to a valid `TypeHandlerInfo` (and its `type_name` buffer +
/// fn-pointers) that outlives this call, produced by an extension's cdylib. The
/// ABI fingerprint is checked BEFORE any fn-pointer is read or called.
pub unsafe fn register_handler_from_ptr(ptr: *const TypeHandlerInfo) -> Result<String, String> {
    if ptr.is_null() {
        return Err("null TypeHandlerInfo pointer".to_string());
    }
    let info = &*ptr;

    if info.abi != abi_fingerprint() {
        return Err(format!(
            "extension ABI mismatch: built against {:?}, host is {:?} — \
             rebuild the extension against this alizarin version.",
            info.abi,
            abi_fingerprint()
        ));
    }

    let type_name = std::str::from_utf8(std::slice::from_raw_parts(
        info.type_name_ptr,
        info.type_name_len,
    ))
    .map_err(|e| format!("handler type_name is not valid UTF-8: {e}"))?
    .to_string();

    TYPE_HANDLERS.write().unwrap().insert(
        type_name.clone(),
        RegisteredHandler {
            coerce_fn: info.coerce_fn,
            free_fn: info.free_fn,
            render_display_fn: info.render_display_fn,
            free_display_fn: info.free_display_fn,
            resolve_markers_fn: info.resolve_markers_fn,
            free_resolve_markers_fn: info.free_resolve_markers_fn,
            validate_fn: info.validate_fn,
            free_validate_fn: info.free_validate_fn,
            render_search_fn: info.render_search_fn,
            free_render_search_fn: info.free_render_search_fn,
            index_spec_fn: info.index_spec_fn,
            free_index_spec_fn: info.free_index_spec_fn,
        },
    );

    crate::set_global_extension_registry(build_extension_registry_from_registered());
    Ok(type_name)
}

// ---- RDM lookup callbacks passed to an extension's resolve_markers ----

unsafe extern "C" fn rdm_lookup_by_id(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
    concept_id_ptr: *const u8,
    concept_id_len: usize,
    concept_json_ptr: *mut *mut u8,
    concept_json_len: *mut usize,
) -> bool {
    // A null cache pointer means no global RDM cache exists (see `resolve_markers`,
    // which passes null when `get_global_rdm_cache_arc()` is None). Treat that as a
    // lookup miss rather than dereferencing null and segfaulting.
    if user_data.is_null() {
        return false;
    }
    let cache = &*(user_data as *const crate::rdm_cache::RdmCache);

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
/// The user_data pointer points to a crate::rdm_cache::RdmCache
unsafe extern "C" fn rdm_lookup_by_label(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
    label_ptr: *const u8,
    label_len: usize,
    concept_json_ptr: *mut *mut u8,
    concept_json_len: *mut usize,
) -> bool {
    // See `rdm_lookup_by_id`: null cache pointer => no cache => lookup miss.
    if user_data.is_null() {
        return false;
    }
    let cache = &*(user_data as *const crate::rdm_cache::RdmCache);

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

unsafe extern "C" fn rdm_has_collection(
    user_data: *mut c_void,
    collection_id_ptr: *const u8,
    collection_id_len: usize,
) -> bool {
    // See `rdm_lookup_by_id`: null cache pointer => no cache => collection absent.
    if user_data.is_null() {
        return false;
    }
    let cache = &*(user_data as *const crate::rdm_cache::RdmCache);

    let collection_id = match std::str::from_utf8(std::slice::from_raw_parts(
        collection_id_ptr,
        collection_id_len,
    )) {
        Ok(s) => s,
        Err(_) => return false,
    };

    cache.has_collection(collection_id)
}
