//! WASM bindings for type coercion functions.
//!
//! These wrappers expose the platform-agnostic type coercion logic
//! from alizarin-core to TypeScript/JavaScript via WASM.

use alizarin_core::type_coercion::{
    // Phase 3
    coerce_boolean,
    coerce_concept_list,
    // Phase 4
    coerce_concept_value,
    // Phase 1
    coerce_date,
    coerce_domain_value,
    coerce_domain_value_list,
    coerce_edtf,
    // Phase 2
    coerce_geojson,
    coerce_non_localized_string,
    coerce_number,
    // Phase 5
    coerce_resource_instance,
    coerce_resource_instance_list,
    coerce_string,
    coerce_url,
    // Dispatcher
    coerce_value,
    // Language config
    get_current_language,
    set_current_language,
    CoercionResult,
};
use wasm_bindgen::prelude::*;

/// WASM wrapper for CoercionResult
#[wasm_bindgen]
pub struct WasmCoercionResult {
    inner: CoercionResult,
}

#[wasm_bindgen]
impl WasmCoercionResult {
    /// Get the tile data (value for tile.data[nodeid])
    #[wasm_bindgen(getter, js_name = tileData)]
    pub fn tile_data(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.tile_data).unwrap_or(JsValue::NULL)
    }

    /// Get the display value (for ViewModel construction)
    #[wasm_bindgen(getter, js_name = displayValue)]
    pub fn display_value(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.display_value).unwrap_or(JsValue::NULL)
    }

    /// Check if the result is an error
    #[wasm_bindgen(getter, js_name = isError)]
    pub fn is_error(&self) -> bool {
        self.inner.is_error()
    }

    /// Get the error message (if any)
    #[wasm_bindgen(getter, js_name = error)]
    pub fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }

    /// Check if the result is null (no value, no error)
    #[wasm_bindgen(getter, js_name = isNull)]
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

/// Coerce a value to a number
#[wasm_bindgen(js_name = coerceNumber)]
pub fn wasm_coerce_number(value: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    WasmCoercionResult {
        inner: coerce_number(&val),
    }
}

/// Coerce a value to a non-localized string
#[wasm_bindgen(js_name = coerceNonLocalizedString)]
pub fn wasm_coerce_non_localized_string(value: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    WasmCoercionResult {
        inner: coerce_non_localized_string(&val),
    }
}

/// Coerce a value to an EDTF string
#[wasm_bindgen(js_name = coerceEdtf)]
pub fn wasm_coerce_edtf(value: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    WasmCoercionResult {
        inner: coerce_edtf(&val),
    }
}

/// Coerce a value to a date
#[wasm_bindgen(js_name = coerceDate)]
pub fn wasm_coerce_date(value: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    WasmCoercionResult {
        inner: coerce_date(&val),
    }
}

/// Coerce a value based on datatype
#[wasm_bindgen(js_name = coerceValue)]
pub fn wasm_coerce_value(datatype: &str, value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_value(datatype, &val, cfg.as_ref()),
    }
}

// =============================================================================
// Phase 2: Dict types
// =============================================================================

/// Coerce a value to a localized string
#[wasm_bindgen(js_name = coerceString)]
pub fn wasm_coerce_string(value: JsValue, language: Option<String>) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    WasmCoercionResult {
        inner: coerce_string(&val, language.as_deref()),
    }
}

/// Coerce a value to a URL
#[wasm_bindgen(js_name = coerceUrl)]
pub fn wasm_coerce_url(value: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    WasmCoercionResult {
        inner: coerce_url(&val),
    }
}

/// Coerce a value to GeoJSON
#[wasm_bindgen(js_name = coerceGeoJson)]
pub fn wasm_coerce_geojson(value: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    WasmCoercionResult {
        inner: coerce_geojson(&val),
    }
}

// =============================================================================
// Phase 3: Config-dependent types
// =============================================================================

/// Coerce a value to a boolean
#[wasm_bindgen(js_name = coerceBoolean)]
pub fn wasm_coerce_boolean(value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_boolean(&val, cfg.as_ref()),
    }
}

/// Coerce a value to a domain value
#[wasm_bindgen(js_name = coerceDomainValue)]
pub fn wasm_coerce_domain_value(value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_domain_value(&val, cfg.as_ref()),
    }
}

/// Coerce a value to a domain value list
#[wasm_bindgen(js_name = coerceDomainValueList)]
pub fn wasm_coerce_domain_value_list(value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_domain_value_list(&val, cfg.as_ref()),
    }
}

// =============================================================================
// Phase 4: RDM-dependent types
// =============================================================================

/// Coerce a value to a concept value
#[wasm_bindgen(js_name = coerceConceptValue)]
pub fn wasm_coerce_concept_value(value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_concept_value(&val, cfg.as_ref()),
    }
}

/// Coerce a value to a concept list
#[wasm_bindgen(js_name = coerceConceptList)]
pub fn wasm_coerce_concept_list(value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_concept_list(&val, cfg.as_ref()),
    }
}

// =============================================================================
// Phase 5: Format normalization
// =============================================================================

/// Coerce a value to a resource instance
#[wasm_bindgen(js_name = coerceResourceInstance)]
pub fn wasm_coerce_resource_instance(value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_resource_instance(&val, cfg.as_ref()),
    }
}

/// Coerce a value to a resource instance list
#[wasm_bindgen(js_name = coerceResourceInstanceList)]
pub fn wasm_coerce_resource_instance_list(value: JsValue, config: JsValue) -> WasmCoercionResult {
    let val: serde_json::Value =
        serde_wasm_bindgen::from_value(value).unwrap_or(serde_json::Value::Null);
    let cfg: Option<serde_json::Value> = if config.is_null() || config.is_undefined() {
        None
    } else {
        serde_wasm_bindgen::from_value(config).ok()
    };
    WasmCoercionResult {
        inner: coerce_resource_instance_list(&val, cfg.as_ref()),
    }
}

// =============================================================================
// Language configuration
// =============================================================================

/// Get the current language setting
#[wasm_bindgen(js_name = getCurrentLanguage)]
pub fn wasm_get_current_language() -> String {
    get_current_language()
}

/// Set the current language
#[wasm_bindgen(js_name = setCurrentLanguage)]
pub fn wasm_set_current_language(language: &str) {
    set_current_language(language);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coercion_result_creation() {
        let result = coerce_number(&serde_json::json!(42));
        let wasm_result = WasmCoercionResult { inner: result };
        assert!(!wasm_result.is_error());
        assert!(!wasm_result.is_null());
    }
}
