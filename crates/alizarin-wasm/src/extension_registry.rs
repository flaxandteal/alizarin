//! Extension registry for WASM
//!
//! Provides a global registry for JS extensions to register display serializers.
//! This mirrors the pattern in alizarin-python where extensions register via PyCapsule.
//!
//! # Usage from JS
//!
//! ```javascript
//! import { registerDisplaySerializer } from 'alizarin-wasm';
//!
//! // Register a display serializer for the 'reference' datatype
//! registerDisplaySerializer('reference', (tileData, language) => {
//!     // Extract display string from StaticReference format
//!     if (tileData.labels && tileData.labels.length > 0) {
//!         const label = tileData.labels.find(l => l.language_id === language)
//!             || tileData.labels.find(l => l.valuetype_id === 'prefLabel')
//!             || tileData.labels[0];
//!         return label?.value || null;
//!     }
//!     return null;
//! });
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use serde_json::Value;
use alizarin_core::{ExtensionDisplaySerializer, SerializationOptions, SerializationResult, DisplaySerializerRegistry};

// Thread-local registry of JS display serializer callbacks
// WASM is single-threaded, so thread_local is safe and avoids Send/Sync requirements
thread_local! {
    static JS_DISPLAY_SERIALIZERS: RefCell<HashMap<String, js_sys::Function>> =
        RefCell::new(HashMap::new());
}

/// Register a display serializer callback from JS.
///
/// The callback signature is: `(tileData: any, language: string) => string | null`
///
/// @param datatype - The datatype name (e.g., "reference", "reference-list")
/// @param callback - JS function that takes (tileData, language) and returns display string or null
#[wasm_bindgen(js_name = registerDisplaySerializer)]
pub fn register_display_serializer(datatype: &str, callback: js_sys::Function) {
    JS_DISPLAY_SERIALIZERS.with(|serializers| {
        serializers.borrow_mut().insert(datatype.to_string(), callback);
    });
}

/// Check if a display serializer is registered for a datatype.
#[wasm_bindgen(js_name = hasDisplaySerializer)]
pub fn has_display_serializer(datatype: &str) -> bool {
    JS_DISPLAY_SERIALIZERS.with(|serializers| {
        serializers.borrow().contains_key(datatype)
    })
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
                // Convert tile_data to JsValue
                let js_tile_data = serde_wasm_bindgen::to_value(tile_data)
                    .unwrap_or(JsValue::NULL);

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
                        let msg = e.as_string().unwrap_or_else(|| "JS callback failed".to_string());
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
