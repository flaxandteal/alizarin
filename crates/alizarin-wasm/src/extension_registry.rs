//! Extension registry for WASM
//!
//! Provides a global registry for JS extensions to register type handlers.
//! This uses the unified ExtensionTypeRegistry from alizarin-core.
//!
//! # Usage from JS
//!
//! ```javascript
//! import { registerExtensionHandler } from 'alizarin-wasm';
//!
//! registerExtensionHandler('reference', {
//!     coerce: (value, config) => ({ tileData: {...}, displayValue: {...} }),
//!     renderDisplay: (tileData, language) => 'Label',
//!     resolveMarkers: (tileData, language) => resolvedTileData,
//! });
//! ```

use alizarin_core::{
    CoercionResult, ExtensionError, ExtensionTypeHandler, ExtensionTypeRegistry,
    HandlerCapabilities,
};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

// Thread-local registry of JS callbacks
// WASM is single-threaded, so thread_local is safe and avoids Send/Sync requirements
thread_local! {
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
        .map_err(|_| JsValue::from_str("options must be an object (failed to read 'coerce')"))?;
    let coerce_fn = if coerce_fn.is_function() {
        Some(coerce_fn.dyn_into::<js_sys::Function>().unwrap())
    } else {
        None
    };

    let render_display_fn = js_sys::Reflect::get(&options, &JsValue::from_str("renderDisplay"))
        .map_err(|_| {
            JsValue::from_str("options must be an object (failed to read 'renderDisplay')")
        })?;
    let render_display_fn = if render_display_fn.is_function() {
        Some(render_display_fn.dyn_into::<js_sys::Function>().unwrap())
    } else {
        None
    };

    let resolve_markers_fn = js_sys::Reflect::get(&options, &JsValue::from_str("resolveMarkers"))
        .map_err(|_| {
        JsValue::from_str("options must be an object (failed to read 'resolveMarkers')")
    })?;
    let resolve_markers_fn = if resolve_markers_fn.is_function() {
        Some(resolve_markers_fn.dyn_into::<js_sys::Function>().unwrap())
    } else {
        None
    };

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

/// Build a fresh extension registry from JS-registered handlers.
///
/// Only includes handlers dynamically registered from JS via `registerExtensionHandler`.
/// Extension crates (e.g. CLM, file-list) must register themselves from JS.
pub fn build_extension_registry() -> ExtensionTypeRegistry {
    let mut registry = ExtensionTypeRegistry::new();

    JS_TYPE_HANDLERS.with(|handlers| {
        for datatype in handlers.borrow().keys() {
            registry.register(
                datatype.clone(),
                Arc::new(JsExtensionTypeHandler::new(datatype.clone())),
            );
        }
    });

    registry
}

/// Check if any extension handlers are registered from JS.
pub fn has_registered_handlers() -> bool {
    JS_TYPE_HANDLERS.with(|handlers| !handlers.borrow().is_empty())
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
                    can_render_search: false,
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
                    let js_value = serde_json::to_string(value)
                        .map_err(|e| {
                            ExtensionError::new(format!("Failed to serialize value: {}", e))
                        })
                        .and_then(|json| {
                            js_sys::JSON::parse(&json).map_err(|_| {
                                ExtensionError::new(
                                    "Failed to parse serialized value as JS".to_string(),
                                )
                            })
                        })?;

                    let js_config = match config {
                        Some(c) => {
                            let json = serde_json::to_string(c).map_err(|e| {
                                ExtensionError::new(format!("Failed to serialize config: {}", e))
                            })?;
                            js_sys::JSON::parse(&json).map_err(|_| {
                                ExtensionError::new(
                                    "Failed to parse serialized config as JS".to_string(),
                                )
                            })?
                        }
                        None => JsValue::NULL,
                    };

                    // Call JS coerce function
                    match coerce_fn.call2(&JsValue::NULL, &js_value, &js_config) {
                        Ok(result) => {
                            let tile_data_js =
                                js_sys::Reflect::get(&result, &JsValue::from_str("tileData"))
                                    .map_err(|_| {
                                        ExtensionError::new(
                                            "Failed to get tileData from coerce result".to_string(),
                                        )
                                    })?;
                            let tile_data_json = js_sys::JSON::stringify(&tile_data_js)
                                .map_err(|_| {
                                    ExtensionError::new(
                                        "Failed to stringify coerce tileData".to_string(),
                                    )
                                })?
                                .as_string()
                                .ok_or_else(|| {
                                    ExtensionError::new(
                                        "Coerce tileData stringify returned non-string".to_string(),
                                    )
                                })?;
                            let tile_data: Value =
                                serde_json::from_str(&tile_data_json).map_err(|e| {
                                    ExtensionError::new(format!(
                                        "Failed to parse coerce tileData: {}",
                                        e
                                    ))
                                })?;

                            let display_value_js =
                                js_sys::Reflect::get(&result, &JsValue::from_str("displayValue"))
                                    .map_err(|_| {
                                    ExtensionError::new(
                                        "Failed to get displayValue from coerce result".to_string(),
                                    )
                                })?;
                            let display_value_json = js_sys::JSON::stringify(&display_value_js)
                                .map_err(|_| {
                                    ExtensionError::new(
                                        "Failed to stringify coerce displayValue".to_string(),
                                    )
                                })?
                                .as_string()
                                .ok_or_else(|| {
                                    ExtensionError::new(
                                        "Coerce displayValue stringify returned non-string"
                                            .to_string(),
                                    )
                                })?;
                            let display_value: Value = serde_json::from_str(&display_value_json)
                                .map_err(|e| {
                                    ExtensionError::new(format!(
                                        "Failed to parse coerce displayValue: {}",
                                        e
                                    ))
                                })?;

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
                    let js_tile_data = serde_json::to_string(tile_data)
                        .map_err(|e| {
                            ExtensionError::new(format!("Failed to serialize tile_data: {}", e))
                        })
                        .and_then(|json| {
                            js_sys::JSON::parse(&json).map_err(|_| {
                                ExtensionError::new(
                                    "Failed to parse serialized tile_data as JS".to_string(),
                                )
                            })
                        })?;

                    let js_language = JsValue::from_str(language);

                    match render_fn.call2(&JsValue::NULL, &js_tile_data, &js_language) {
                        Ok(result) => {
                            if result.is_null() || result.is_undefined() {
                                Ok(None)
                            } else if let Some(s) = result.as_string() {
                                Ok(Some(s))
                            } else {
                                Err(ExtensionError::new(
                                    "renderDisplay must return a string, null, or undefined"
                                        .to_string(),
                                ))
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
                    let js_tile_data = serde_json::to_string(tile_data)
                        .map_err(|e| {
                            ExtensionError::new(format!("Failed to serialize tile_data: {}", e))
                        })
                        .and_then(|json| {
                            js_sys::JSON::parse(&json).map_err(|_| {
                                ExtensionError::new(
                                    "Failed to parse serialized tile_data as JS".to_string(),
                                )
                            })
                        })?;

                    let js_language = JsValue::from_str(language);

                    match resolve_fn.call2(&JsValue::NULL, &js_tile_data, &js_language) {
                        Ok(result) => {
                            let json = js_sys::JSON::stringify(&result)
                                .map_err(|_| {
                                    ExtensionError::new(
                                        "Failed to stringify resolveMarkers result".to_string(),
                                    )
                                })?
                                .as_string()
                                .ok_or_else(|| {
                                    ExtensionError::new(
                                        "resolveMarkers result was not a string".to_string(),
                                    )
                                })?;

                            serde_json::from_str(&json).map_err(|e| {
                                ExtensionError::new(format!("Failed to parse resolved data: {}", e))
                            })
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
