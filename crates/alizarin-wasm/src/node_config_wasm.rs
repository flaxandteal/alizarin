use js_sys::{Array, Object, Reflect};
/// WASM bindings for Node configuration types.
///
/// This module wraps the platform-agnostic types from alizarin-core
/// with wasm-bindgen bindings for TypeScript/JavaScript access.
use wasm_bindgen::prelude::*;

// Import core types
use alizarin_core::node_config::{
    NodeConfigBoolean as CoreNodeConfigBoolean, NodeConfigConcept as CoreNodeConfigConcept,
    NodeConfigDomain as CoreNodeConfigDomain, NodeConfigManager as CoreNodeConfigManager,
    NodeConfigReference as CoreNodeConfigReference, StaticDomainValue as CoreStaticDomainValue,
};

// =============================================================================
// StaticDomainValue WASM Wrapper
// =============================================================================

/// A domain value option from node config.
#[wasm_bindgen]
pub struct WasmStaticDomainValue {
    inner: CoreStaticDomainValue,
}

#[wasm_bindgen]
impl WasmStaticDomainValue {
    /// Get the value ID
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    /// Get whether this is selected
    #[wasm_bindgen(getter)]
    pub fn selected(&self) -> bool {
        self.inner.selected
    }

    /// Get text for a specific language
    pub fn lang(&self, language: &str) -> Option<String> {
        self.inner.lang(language).map(|s| s.to_string())
    }

    /// Get display string (defaults to English)
    pub fn display(&self) -> String {
        self.inner.display().to_string()
    }

    /// Get all text labels as a JS object
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> JsValue {
        let obj = Object::new();
        for (key, value) in &self.inner.text {
            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value)).ok();
        }
        obj.into()
    }
}

impl From<CoreStaticDomainValue> for WasmStaticDomainValue {
    fn from(inner: CoreStaticDomainValue) -> Self {
        Self { inner }
    }
}

impl From<&CoreStaticDomainValue> for WasmStaticDomainValue {
    fn from(inner: &CoreStaticDomainValue) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

// =============================================================================
// NodeConfigBoolean WASM Wrapper
// =============================================================================

/// Boolean node configuration with true/false labels.
#[wasm_bindgen]
pub struct WasmNodeConfigBoolean {
    inner: CoreNodeConfigBoolean,
}

#[wasm_bindgen]
impl WasmNodeConfigBoolean {
    /// Get label for a boolean value in a specific language
    #[wasm_bindgen(js_name = getLabel)]
    pub fn get_label(&self, value: bool, language: &str) -> Option<String> {
        self.inner.get_label(value, language).map(|s| s.to_string())
    }

    /// Get true labels as a JS object
    #[wasm_bindgen(getter, js_name = trueLabel)]
    pub fn true_label(&self) -> JsValue {
        let obj = Object::new();
        for (key, value) in &self.inner.true_label {
            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value)).ok();
        }
        obj.into()
    }

    /// Get false labels as a JS object
    #[wasm_bindgen(getter, js_name = falseLabel)]
    pub fn false_label(&self) -> JsValue {
        let obj = Object::new();
        for (key, value) in &self.inner.false_label {
            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value)).ok();
        }
        obj.into()
    }
}

impl From<CoreNodeConfigBoolean> for WasmNodeConfigBoolean {
    fn from(inner: CoreNodeConfigBoolean) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigBoolean> for WasmNodeConfigBoolean {
    fn from(inner: &CoreNodeConfigBoolean) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

// =============================================================================
// NodeConfigConcept WASM Wrapper
// =============================================================================

/// Concept node configuration (RDM collection reference).
#[wasm_bindgen]
pub struct WasmNodeConfigConcept {
    inner: CoreNodeConfigConcept,
}

#[wasm_bindgen]
impl WasmNodeConfigConcept {
    /// Get the RDM collection ID
    #[wasm_bindgen(getter, js_name = rdmCollection)]
    pub fn rdm_collection(&self) -> String {
        self.inner.rdm_collection.clone()
    }
}

impl From<CoreNodeConfigConcept> for WasmNodeConfigConcept {
    fn from(inner: CoreNodeConfigConcept) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigConcept> for WasmNodeConfigConcept {
    fn from(inner: &CoreNodeConfigConcept) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

// =============================================================================
// NodeConfigReference WASM Wrapper
// =============================================================================

/// Reference node configuration (CLM controlled list).
#[wasm_bindgen]
pub struct WasmNodeConfigReference {
    inner: CoreNodeConfigReference,
}

#[wasm_bindgen]
impl WasmNodeConfigReference {
    /// Get the controlled list ID
    #[wasm_bindgen(getter, js_name = controlledList)]
    pub fn controlled_list(&self) -> String {
        self.inner.controlled_list.clone()
    }

    /// Get the RDM collection ID
    #[wasm_bindgen(getter, js_name = rdmCollection)]
    pub fn rdm_collection(&self) -> String {
        self.inner.rdm_collection.clone()
    }

    /// Get whether multi-value is enabled
    #[wasm_bindgen(getter, js_name = multiValue)]
    pub fn multi_value(&self) -> bool {
        self.inner.multi_value
    }

    /// Get the collection ID (controlledList or rdmCollection)
    #[wasm_bindgen(js_name = getCollectionId)]
    pub fn get_collection_id(&self) -> Option<String> {
        self.inner.get_collection_id().map(|s| s.to_string())
    }
}

impl From<CoreNodeConfigReference> for WasmNodeConfigReference {
    fn from(inner: CoreNodeConfigReference) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigReference> for WasmNodeConfigReference {
    fn from(inner: &CoreNodeConfigReference) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

// =============================================================================
// NodeConfigDomain WASM Wrapper
// =============================================================================

/// Domain value node configuration with options list.
#[wasm_bindgen]
pub struct WasmNodeConfigDomain {
    inner: CoreNodeConfigDomain,
}

#[wasm_bindgen]
impl WasmNodeConfigDomain {
    /// Get all options as an array
    #[wasm_bindgen(getter)]
    pub fn options(&self) -> Array {
        self.inner
            .options
            .iter()
            .map(|opt| {
                let wasm_opt: WasmStaticDomainValue = opt.into();
                JsValue::from(wasm_opt)
            })
            .collect()
    }

    /// Get the selected option (if any)
    #[wasm_bindgen(js_name = getSelected)]
    pub fn get_selected(&self) -> Option<WasmStaticDomainValue> {
        self.inner.get_selected().map(WasmStaticDomainValue::from)
    }

    /// Find option by ID
    #[wasm_bindgen(js_name = valueFromId)]
    pub fn value_from_id(&self, id: &str) -> Option<WasmStaticDomainValue> {
        self.inner
            .value_from_id(id)
            .map(WasmStaticDomainValue::from)
    }

    /// Get all option IDs
    #[wasm_bindgen(js_name = getOptionIds)]
    pub fn get_option_ids(&self) -> Array {
        self.inner
            .get_option_ids()
            .iter()
            .map(|id| JsValue::from_str(id))
            .collect()
    }
}

impl From<CoreNodeConfigDomain> for WasmNodeConfigDomain {
    fn from(inner: CoreNodeConfigDomain) -> Self {
        Self { inner }
    }
}

impl From<&CoreNodeConfigDomain> for WasmNodeConfigDomain {
    fn from(inner: &CoreNodeConfigDomain) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

// =============================================================================
// NodeConfigManager WASM Wrapper
// =============================================================================

/// Manager for caching and retrieving node configurations.
#[wasm_bindgen]
pub struct WasmNodeConfigManager {
    inner: CoreNodeConfigManager,
}

#[wasm_bindgen]
impl WasmNodeConfigManager {
    /// Create a new empty manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreNodeConfigManager::new(),
        }
    }

    /// Build configs from a graph JSON string
    #[wasm_bindgen(js_name = fromGraphJson)]
    pub fn from_graph_json(&mut self, graph_json: &str) -> Result<(), JsValue> {
        self.inner
            .from_graph_json(graph_json)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get boolean config for a node
    #[wasm_bindgen(js_name = getBoolean)]
    pub fn get_boolean(&self, nodeid: &str) -> Option<WasmNodeConfigBoolean> {
        self.inner
            .get_boolean(nodeid)
            .map(WasmNodeConfigBoolean::from)
    }

    /// Get concept config for a node
    #[wasm_bindgen(js_name = getConcept)]
    pub fn get_concept(&self, nodeid: &str) -> Option<WasmNodeConfigConcept> {
        self.inner
            .get_concept(nodeid)
            .map(WasmNodeConfigConcept::from)
    }

    /// Get reference config for a node
    #[wasm_bindgen(js_name = getReference)]
    pub fn get_reference(&self, nodeid: &str) -> Option<WasmNodeConfigReference> {
        self.inner
            .get_reference(nodeid)
            .map(WasmNodeConfigReference::from)
    }

    /// Get domain config for a node
    #[wasm_bindgen(js_name = getDomain)]
    pub fn get_domain(&self, nodeid: &str) -> Option<WasmNodeConfigDomain> {
        self.inner
            .get_domain(nodeid)
            .map(WasmNodeConfigDomain::from)
    }

    /// Look up domain value by ID
    #[wasm_bindgen(js_name = lookupDomainValue)]
    pub fn lookup_domain_value(
        &self,
        nodeid: &str,
        value_id: &str,
    ) -> Option<WasmStaticDomainValue> {
        self.inner
            .lookup_domain_value(nodeid, value_id)
            .map(WasmStaticDomainValue::from)
    }

    /// Check if a node has config
    #[wasm_bindgen(js_name = hasConfig)]
    pub fn has_config(&self, nodeid: &str) -> bool {
        self.inner.has_config(nodeid)
    }

    /// Get the config type for a node
    #[wasm_bindgen(js_name = getConfigType)]
    pub fn get_config_type(&self, nodeid: &str) -> Option<String> {
        self.inner.get_config_type(nodeid).map(|s| s.to_string())
    }

    /// Clear all cached configs
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get number of cached configs
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.inner.len()
    }
}

impl Default for WasmNodeConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Internal API for use by instance_wrapper
// =============================================================================

impl WasmNodeConfigManager {
    /// Get the inner CoreNodeConfigManager (for internal use)
    pub(crate) fn inner(&self) -> &CoreNodeConfigManager {
        &self.inner
    }
}
