/// Example of how to wrap the core PseudoNode with wasm-bindgen
///
/// This would go in crates/alizarin-wasm/src/pseudos.rs
///
/// Key Points:
/// 1. Core logic is in alizarin-core (platform-agnostic)
/// 2. This layer just handles JavaScript interop
/// 3. Uses JsValue for JavaScript object identity preservation
/// 4. The heavy lifting is done by the core

use wasm_bindgen::prelude::*;
use alizarin_core::{PseudoNodeCore, NodeLike};
use std::rc::Rc;
use std::cell::RefCell;

// Your existing StaticNode would implement NodeLike
impl NodeLike for crate::graph::StaticNode {
    fn get_alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    fn get_datatype(&self) -> Option<&str> {
        Some(&self.datatype)
    }
}

/// WASM wrapper for PseudoNode - handles JavaScript interop
#[wasm_bindgen]
#[derive(Clone)]
pub struct PseudoNode {
    // The core logic (pure Rust)
    core: Rc<RefCell<PseudoNodeCore<crate::graph::StaticNode>>>,

    // JavaScript-specific fields for object identity
    // These are ONLY for the WASM binding layer
    node_js: JsValue,
    child_nodes_js: JsValue,
    parent_node_js: Option<JsValue>,
    inner_js: Option<JsValue>,
}

#[wasm_bindgen]
impl PseudoNode {
    #[wasm_bindgen(constructor)]
    pub fn new(
        node: JsValue,
        child_nodes: JsValue,
        inner: JsValue,
    ) -> Result<PseudoNode, JsValue> {
        // Convert JsValue to StaticNode
        let static_node: crate::graph::StaticNode = serde_wasm_bindgen::from_value(node.clone())?;

        // Convert child_nodes Map to HashMap
        let child_map = if let Ok(map) = js_sys::Map::try_from(&child_nodes) {
            let mut hash_map = std::collections::HashMap::new();

            map.for_each(&mut |value, key| {
                if let Some(key_str) = key.as_string() {
                    if let Ok(child_node) = serde_wasm_bindgen::from_value::<crate::graph::StaticNode>(value) {
                        hash_map.insert(key_str, child_node);
                    }
                }
            });

            hash_map
        } else {
            std::collections::HashMap::new()
        };

        // Determine inner flag
        let is_inner = if !inner.is_null() && !inner.is_undefined() {
            inner.as_bool().unwrap_or(false)
        } else {
            false
        };

        // Build the core using the builder
        let core = if is_inner {
            alizarin_core::PseudoNodeBuilder::new(static_node, child_map)
                .with_inner_flag(true)
                .build()
        } else {
            alizarin_core::PseudoNodeBuilder::new(static_node, child_map)
                .build()
        };

        Ok(PseudoNode {
            core: Rc::new(RefCell::new(core)),
            node_js: node,
            child_nodes_js: child_nodes,
            parent_node_js: None,
            inner_js: if !inner.is_null() && !inner.is_undefined() { Some(inner) } else { None },
        })
    }

    /// Delegate to core logic
    #[wasm_bindgen(js_name = isIterable)]
    pub fn is_iterable(&self) -> bool {
        self.core.borrow().is_iterable()
    }

    /// Delegate to core logic
    #[wasm_bindgen(js_name = getNodePlaceholder)]
    pub fn get_node_placeholder(&self) -> Result<String, JsValue> {
        Ok(self.core.borrow().get_node_placeholder())
    }

    // Getters - return JsValue for JavaScript compatibility
    #[wasm_bindgen(getter = node)]
    pub fn get_node(&self) -> JsValue {
        self.node_js.clone()
    }

    #[wasm_bindgen(setter = node)]
    pub fn set_node(&mut self, value: JsValue) {
        // Update both the JS value and the core
        if let Ok(static_node) = serde_wasm_bindgen::from_value::<crate::graph::StaticNode>(value.clone()) {
            self.core.borrow_mut().node = static_node;
        }
        self.node_js = value;
    }

    #[wasm_bindgen(getter = parentNode)]
    pub fn get_parent_node(&self) -> JsValue {
        self.parent_node_js.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = parentNode)]
    pub fn set_parent_node(&mut self, value: JsValue) {
        if value.is_null() {
            self.parent_node_js = None;
        } else {
            self.parent_node_js = Some(value);
        }
    }

    #[wasm_bindgen(getter = datatype)]
    pub fn get_datatype_property(&self) -> JsValue {
        match &self.core.borrow().datatype {
            Some(dt) => JsValue::from_str(dt),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = datatype)]
    pub fn set_datatype(&mut self, value: JsValue) {
        if value.is_null() {
            self.core.borrow_mut().datatype = None;
        } else if let Some(s) = value.as_string() {
            self.core.borrow_mut().datatype = Some(s);
        }
    }

    #[wasm_bindgen(getter = childNodes)]
    pub fn get_child_nodes(&self) -> JsValue {
        self.child_nodes_js.clone()
    }

    #[wasm_bindgen(setter = childNodes)]
    pub fn set_child_nodes(&mut self, value: JsValue) {
        // Update the JS value
        self.child_nodes_js = value.clone();

        // Update the core (convert Map to HashMap)
        if let Ok(map) = js_sys::Map::try_from(&value) {
            let mut hash_map = std::collections::HashMap::new();

            map.for_each(&mut |val, key| {
                if let Some(key_str) = key.as_string() {
                    if let Ok(child_node) = serde_wasm_bindgen::from_value::<crate::graph::StaticNode>(val) {
                        hash_map.insert(key_str, child_node);
                    }
                }
            });

            self.core.borrow_mut().child_nodes = hash_map;
        }
    }

    #[wasm_bindgen(getter = isOuter)]
    pub fn get_is_outer(&self) -> bool {
        self.core.borrow().is_outer
    }

    #[wasm_bindgen(setter = isOuter)]
    pub fn set_is_outer(&mut self, value: bool) {
        self.core.borrow_mut().is_outer = value;
    }

    #[wasm_bindgen(getter = isInner)]
    pub fn get_is_inner(&self) -> bool {
        self.core.borrow().is_inner
    }

    #[wasm_bindgen(setter = isInner)]
    pub fn set_is_inner(&mut self, value: bool) {
        self.core.borrow_mut().is_inner = value;
    }

    #[wasm_bindgen(getter = inner)]
    pub fn get_inner(&self) -> JsValue {
        self.inner_js.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = inner)]
    pub fn set_inner(&mut self, value: JsValue) {
        if value.is_null() {
            self.inner_js = None;
        } else {
            self.inner_js = Some(value);
        }
    }
}

/// This shows the key architecture:
///
/// 1. **Core Logic** (alizarin-core): Pure Rust, testable, platform-agnostic
///    - PseudoNodeCore<TNode> contains all business logic
///    - Works with any TNode that implements NodeLike
///    - Uses Rc<RefCell<>> for circular references
///    - All methods are pure Rust
///
/// 2. **WASM Binding** (this file): Thin wrapper for JavaScript
///    - Wraps PseudoNodeCore with wasm-bindgen attributes
///    - Stores JsValue for JavaScript object identity
///    - Delegates logic to core
///    - Handles conversion between JsValue and Rust types
///
/// 3. **Benefits**:
///    - Core can be tested without WASM overhead
///    - Core can be reused for Python bindings (or any other platform)
///    - WASM layer is minimal and focused on interop
///    - Type safety is maintained in the core
///    - JavaScript gets familiar API
