//! Extension registry for WASM
//!
//! Provides a global registry for JS extensions to register type handlers.
//! This uses the unified ExtensionTypeRegistry from alizarin-core.
//!
//! # Usage from JS
//!
//! ```javascript
//! import { registerExtensionHandler, registerDisplaySerializer } from 'alizarin-wasm';
//!
//! // Register a full extension handler (coercion + display + markers)
//! registerExtensionHandler('reference', {
//!     coerce: (value, config) => ({ tileData: {...}, displayValue: {...} }),
//!     renderDisplay: (tileData, language) => 'Label',
//!     resolveMarkers: (tileData, language) => resolvedTileData,
//! });
//!
//! // Or register just a display serializer (backward compatible)
//! registerDisplaySerializer('reference', (tileData, language) => 'Label');
//! ```

use alizarin_core::{
    CoercionResult, DisplaySerializerRegistry, ExtensionDisplaySerializer, ExtensionError,
    ExtensionTypeHandler, ExtensionTypeRegistry, HandlerCapabilities, SerializationOptions,
    SerializationResult,
};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

// Thread-local registry of JS callbacks
// WASM is single-threaded, so thread_local is safe and avoids Send/Sync requirements
thread_local! {
    // Legacy display-only callbacks (backward compatibility)
    static JS_DISPLAY_SERIALIZERS: RefCell<HashMap<String, js_sys::Function>> =
        RefCell::new(HashMap::new());

    // New unified handler callbacks
    static JS_TYPE_HANDLERS: RefCell<HashMap<String, JsHandlerCallbacks>> =
        RefCell::new(HashMap::new());
}

/// JS callbacks for a type handler
struct JsHandlerCallbacks {
    coerce_fn: Option<js_sys::Function>,
    render_display_fn: Option<js_sys::Function>,
    resolve_markers_fn: Option<js_sys::Function>,
}

// =============================================================================
// New Unified Extension Handler API
// =============================================================================

/// Register an extension type handler from JS.
///
/// The options object can have any of these callbacks:
/// - `coerce(value, config) => { tileData, displayValue }` - for ETL coercion
/// - `renderDisplay(tileData, language) => string | null` - for display rendering
/// - `resolveMarkers(tileData, language) => resolvedTileData` - for marker resolution
///
/// @param datatype - The datatype name (e.g., "reference")
/// @param options - Object with callback functions
#[wasm_bindgen(js_name = registerExtensionHandler)]
pub fn register_extension_handler(datatype: &str, options: JsValue) -> Result<(), JsValue> {
    let coerce_fn = js_sys::Reflect::get(&options, &JsValue::from_str("coerce"))
        .ok()
        .filter(|v| v.is_function())
        .map(|v| v.dyn_into::<js_sys::Function>().unwrap());

    let render_display_fn = js_sys::Reflect::get(&options, &JsValue::from_str("renderDisplay"))
        .ok()
        .filter(|v| v.is_function())
        .map(|v| v.dyn_into::<js_sys::Function>().unwrap());

    let resolve_markers_fn = js_sys::Reflect::get(&options, &JsValue::from_str("resolveMarkers"))
        .ok()
        .filter(|v| v.is_function())
        .map(|v| v.dyn_into::<js_sys::Function>().unwrap());

    if coerce_fn.is_none() && render_display_fn.is_none() && resolve_markers_fn.is_none() {
        return Err(JsValue::from_str(
            "At least one callback (coerce, renderDisplay, resolveMarkers) is required",
        ));
    }

    JS_TYPE_HANDLERS.with(|handlers| {
        handlers.borrow_mut().insert(
            datatype.to_string(),
            JsHandlerCallbacks {
                coerce_fn,
                render_display_fn,
                resolve_markers_fn,
            },
        );
    });

    Ok(())
}

/// Check if an extension handler is registered for a datatype.
#[wasm_bindgen(js_name = hasExtensionHandler)]
pub fn has_extension_handler(datatype: &str) -> bool {
    JS_TYPE_HANDLERS.with(|handlers| handlers.borrow().contains_key(datatype))
}

/// Unregister an extension handler.
#[wasm_bindgen(js_name = unregisterExtensionHandler)]
pub fn unregister_extension_handler(datatype: &str) {
    JS_TYPE_HANDLERS.with(|handlers| {
        handlers.borrow_mut().remove(datatype);
    });
}

/// Get the list of registered extension handler datatypes.
#[wasm_bindgen(js_name = getRegisteredExtensionHandlers)]
pub fn get_registered_extension_handlers() -> js_sys::Array {
    JS_TYPE_HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();
        let arr = js_sys::Array::new();
        for datatype in handlers.keys() {
            arr.push(&JsValue::from_str(datatype));
        }
        arr
    })
}

/// A type handler that delegates to JS callbacks.
///
/// Note: This struct is created fresh each time we need to use the callbacks
/// because JS functions aren't Send+Sync. We use thread_local storage instead.
pub struct JsExtensionTypeHandler {
    datatype: String,
    capabilities: HandlerCapabilities,
}

impl JsExtensionTypeHandler {
    pub fn new(datatype: String) -> Self {
        let capabilities = JS_TYPE_HANDLERS.with(|handlers| {
            let handlers = handlers.borrow();
            if let Some(cbs) = handlers.get(&datatype) {
                HandlerCapabilities {
                    can_coerce: cbs.coerce_fn.is_some(),
                    can_render_display: cbs.render_display_fn.is_some(),
                    can_resolve_markers: cbs.resolve_markers_fn.is_some(),
                }
            } else {
                HandlerCapabilities::default()
            }
        });

        Self {
            datatype,
            capabilities,
        }
    }
}

// Note: We need Send+Sync for ExtensionTypeHandler, but JS functions aren't thread-safe.
// Since WASM is single-threaded, this is safe.
unsafe impl Send for JsExtensionTypeHandler {}
unsafe impl Sync for JsExtensionTypeHandler {}

impl ExtensionTypeHandler for JsExtensionTypeHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        self.capabilities.clone()
    }

    fn coerce(
        &self,
        value: &Value,
        config: Option<&Value>,
    ) -> Result<CoercionResult, ExtensionError> {
        JS_TYPE_HANDLERS.with(|handlers| {
            let handlers = handlers.borrow();

            if let Some(cbs) = handlers.get(&self.datatype) {
                if let Some(ref coerce_fn) = cbs.coerce_fn {
                    // Convert value to JsValue
                    let js_value = match serde_json::to_string(value) {
                        Ok(json) => js_sys::JSON::parse(&json).unwrap_or(JsValue::NULL),
                        Err(_) => JsValue::NULL,
                    };

                    let js_config = match config {
                        Some(c) => match serde_json::to_string(c) {
                            Ok(json) => js_sys::JSON::parse(&json).unwrap_or(JsValue::NULL),
                            Err(_) => JsValue::NULL,
                        },
                        None => JsValue::NULL,
                    };

                    // Call JS coerce function
                    match coerce_fn.call2(&JsValue::NULL, &js_value, &js_config) {
                        Ok(result) => {
                            // Extract tileData and displayValue from result
                            let tile_data =
                                js_sys::Reflect::get(&result, &JsValue::from_str("tileData"))
                                    .ok()
                                    .and_then(|v| {
                                        let json = js_sys::JSON::stringify(&v).ok()?.as_string()?;
                                        serde_json::from_str(&json).ok()
                                    })
                                    .unwrap_or_else(|| value.clone());

                            let display_value =
                                js_sys::Reflect::get(&result, &JsValue::from_str("displayValue"))
                                    .ok()
                                    .and_then(|v| {
                                        let json = js_sys::JSON::stringify(&v).ok()?.as_string()?;
                                        serde_json::from_str(&json).ok()
                                    })
                                    .unwrap_or_else(|| tile_data.clone());

                            Ok(CoercionResult::success(tile_data, display_value))
                        }
                        Err(e) => {
                            let msg = e
                                .as_string()
                                .unwrap_or_else(|| "JS coerce callback failed".to_string());
                            Err(ExtensionError::new(msg))
                        }
                    }
                } else {
                    Ok(CoercionResult::success(value.clone(), value.clone()))
                }
            } else {
                Ok(CoercionResult::success(value.clone(), value.clone()))
            }
        })
    }

    fn render_display(
        &self,
        tile_data: &Value,
        language: &str,
    ) -> Result<Option<String>, ExtensionError> {
        JS_TYPE_HANDLERS.with(|handlers| {
            let handlers = handlers.borrow();

            if let Some(cbs) = handlers.get(&self.datatype) {
                if let Some(ref render_fn) = cbs.render_display_fn {
                    let js_tile_data = match serde_json::to_string(tile_data) {
                        Ok(json) => js_sys::JSON::parse(&json).unwrap_or(JsValue::NULL),
                        Err(_) => JsValue::NULL,
                    };

                    let js_language = JsValue::from_str(language);

                    match render_fn.call2(&JsValue::NULL, &js_tile_data, &js_language) {
                        Ok(result) => {
                            if result.is_null() || result.is_undefined() {
                                Ok(None)
                            } else if let Some(s) = result.as_string() {
                                Ok(Some(s))
                            } else {
                                Ok(None)
                            }
                        }
                        Err(e) => {
                            let msg = e
                                .as_string()
                                .unwrap_or_else(|| "JS renderDisplay callback failed".to_string());
                            Err(ExtensionError::new(msg))
                        }
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
    }

    fn resolve_markers(&self, tile_data: &Value, language: &str) -> Result<Value, ExtensionError> {
        JS_TYPE_HANDLERS.with(|handlers| {
            let handlers = handlers.borrow();

            if let Some(cbs) = handlers.get(&self.datatype) {
                if let Some(ref resolve_fn) = cbs.resolve_markers_fn {
                    let js_tile_data = match serde_json::to_string(tile_data) {
                        Ok(json) => js_sys::JSON::parse(&json).unwrap_or(JsValue::NULL),
                        Err(_) => JsValue::NULL,
                    };

                    let js_language = JsValue::from_str(language);

                    match resolve_fn.call2(&JsValue::NULL, &js_tile_data, &js_language) {
                        Ok(result) => {
                            let json = js_sys::JSON::stringify(&result)
                                .ok()
                                .and_then(|s| s.as_string());

                            match json {
                                Some(j) => serde_json::from_str(&j).map_err(|e| {
                                    ExtensionError::new(format!(
                                        "Failed to parse resolved data: {}",
                                        e
                                    ))
                                }),
                                None => Ok(tile_data.clone()),
                            }
                        }
                        Err(e) => {
                            let msg = e
                                .as_string()
                                .unwrap_or_else(|| "JS resolveMarkers callback failed".to_string());
                            Err(ExtensionError::new(msg))
                        }
                    }
                } else {
                    Ok(tile_data.clone())
                }
            } else {
                Ok(tile_data.clone())
            }
        })
    }

    fn description(&self) -> &str {
        "JS extension type handler"
    }
}

/// Build an ExtensionTypeRegistry from all registered JS handlers.
///
/// This creates a new registry each time, populated with JsExtensionTypeHandler
/// instances for each registered datatype.
pub fn build_extension_type_registry() -> ExtensionTypeRegistry {
    let mut registry = ExtensionTypeRegistry::new();

    JS_TYPE_HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();
        for datatype in handlers.keys() {
            registry.register(
                datatype.clone(),
                Arc::new(JsExtensionTypeHandler::new(datatype.clone())),
            );
        }
    });

    registry
}

// =============================================================================
// Legacy Display Serializer API (backward compatibility)
// =============================================================================

/// Register a display serializer callback from JS.
///
/// The callback signature is: `(tileData: any, language: string) => string | null`
///
/// @param datatype - The datatype name (e.g., "reference", "reference-list")
/// @param callback - JS function that takes (tileData, language) and returns display string or null
#[wasm_bindgen(js_name = registerDisplaySerializer)]
pub fn register_display_serializer(datatype: &str, callback: js_sys::Function) {
    JS_DISPLAY_SERIALIZERS.with(|serializers| {
        serializers
            .borrow_mut()
            .insert(datatype.to_string(), callback);
    });
}

/// Check if a display serializer is registered for a datatype.
#[wasm_bindgen(js_name = hasDisplaySerializer)]
pub fn has_display_serializer(datatype: &str) -> bool {
    JS_DISPLAY_SERIALIZERS.with(|serializers| serializers.borrow().contains_key(datatype))
}

/// Unregister a display serializer.
#[wasm_bindgen(js_name = unregisterDisplaySerializer)]
pub fn unregister_display_serializer(datatype: &str) {
    JS_DISPLAY_SERIALIZERS.with(|serializers| {
        serializers.borrow_mut().remove(datatype);
    });
}

/// A display serializer that delegates to a JS callback.
pub struct JsDisplaySerializer {
    datatype: String,
}

impl JsDisplaySerializer {
    pub fn new(datatype: String) -> Self {
        Self { datatype }
    }
}

impl ExtensionDisplaySerializer for JsDisplaySerializer {
    fn serialize_display(
        &self,
        tile_data: &Value,
        options: &SerializationOptions,
    ) -> SerializationResult {
        JS_DISPLAY_SERIALIZERS.with(|serializers| {
            let serializers = serializers.borrow();

            if let Some(callback) = serializers.get(&self.datatype) {
                // Convert tile_data to JsValue via JSON string (more reliable than serde_wasm_bindgen)
                let js_tile_data = match serde_json::to_string(tile_data) {
                    Ok(json_str) => js_sys::JSON::parse(&json_str).unwrap_or(JsValue::NULL),
                    Err(_) => JsValue::NULL,
                };

                let js_language = JsValue::from_str(&options.language);

                // Call the JS callback
                match callback.call2(&JsValue::NULL, &js_tile_data, &js_language) {
                    Ok(result) => {
                        if result.is_null() || result.is_undefined() {
                            // Callback returned null - pass through original
                            SerializationResult::success(tile_data.clone())
                        } else if let Some(s) = result.as_string() {
                            SerializationResult::success(Value::String(s))
                        } else {
                            // Try to convert back to serde_json::Value
                            match serde_wasm_bindgen::from_value(result) {
                                Ok(v) => SerializationResult::success(v),
                                Err(_) => SerializationResult::success(tile_data.clone()),
                            }
                        }
                    }
                    Err(e) => {
                        let msg = e
                            .as_string()
                            .unwrap_or_else(|| "JS callback failed".to_string());
                        SerializationResult::error(msg)
                    }
                }
            } else {
                // No callback registered - pass through
                SerializationResult::success(tile_data.clone())
            }
        })
    }

    fn description(&self) -> &str {
        "JS display serializer callback"
    }
}

/// Build a DisplaySerializerRegistry from all registered JS callbacks.
///
/// This creates a new registry each time, populated with JsDisplaySerializer
/// instances for each registered datatype.
pub fn build_display_registry() -> DisplaySerializerRegistry {
    let mut registry = DisplaySerializerRegistry::new();

    JS_DISPLAY_SERIALIZERS.with(|serializers| {
        let serializers = serializers.borrow();
        for datatype in serializers.keys() {
            registry.register(
                datatype.clone(),
                Arc::new(JsDisplaySerializer::new(datatype.clone())),
            );
        }
    });

    registry
}

/// Get the list of registered display serializer datatypes.
#[wasm_bindgen(js_name = getRegisteredDisplaySerializers)]
pub fn get_registered_display_serializers() -> js_sys::Array {
    JS_DISPLAY_SERIALIZERS.with(|serializers| {
        let serializers = serializers.borrow();
        let arr = js_sys::Array::new();
        for datatype in serializers.keys() {
            arr.push(&JsValue::from_str(datatype));
        }
        arr
    })
}
