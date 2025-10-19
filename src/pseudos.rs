use wasm_bindgen::prelude::*;

// Constants for iterable datatypes
const ITERABLE_DATATYPES: &[&str] = &[
    "concept-list",
    "resource-instance-list",
    "domain-value-list"
];

#[wasm_bindgen]
#[derive(Clone)]
pub struct PseudoNode {
    // Store as JsValue to preserve object identity
    node: JsValue,
    // Must use JsValue for circular references (can't use PseudoNode due to recursion)
    parent_node: Option<JsValue>,
    datatype: Option<String>,
    // Store as JsValue (the original Map) to preserve reference
    child_nodes: JsValue,
    is_outer: bool,
    is_inner: bool,
    // Must use JsValue for circular references (can't use PseudoNode due to recursion)
    inner: Option<JsValue>,
}

#[wasm_bindgen]
impl PseudoNode {
    #[wasm_bindgen(constructor)]
    pub fn new(
        node: JsValue,
        child_nodes: JsValue,
        inner: JsValue,
    ) -> Result<PseudoNode, JsValue> {
        // Get datatype from the node
        let datatype = if let Ok(dt) = js_sys::Reflect::get(&node, &JsValue::from_str("datatype")) {
            dt.as_string()
        } else {
            None
        };

        let mut is_outer = false;
        let mut is_inner = false;
        let mut inner_val: Option<JsValue> = None;
        let mut final_datatype = datatype;

        // Handle the inner parameter
        if !inner.is_null() && !inner.is_undefined() {
            // Check if it's a boolean true
            if let Some(bool_val) = inner.as_bool() {
                if bool_val {
                    is_inner = true;
                    final_datatype = Some("semantic".to_string());
                }
            } else {
                // It's a PseudoValue/PseudoNode instance
                is_outer = true;
                inner_val = Some(inner);
            }
        }

        Ok(PseudoNode {
            node,
            parent_node: None,
            datatype: final_datatype,
            child_nodes,
            is_outer,
            is_inner,
            inner: inner_val,
        })
    }

    #[wasm_bindgen(js_name = isIterable)]
    pub fn is_iterable(&self) -> bool {
        if let Some(ref dt) = self.datatype {
            ITERABLE_DATATYPES.contains(&dt.as_str())
        } else {
            false
        }
    }

    #[wasm_bindgen(js_name = getNodePlaceholder)]
    pub fn get_node_placeholder(&self) -> Result<String, JsValue> {
        let mut placeholder = String::from(".");

        // If we have a parent node, recursively get its placeholder
        if let Some(ref parent) = self.parent_node {
            // Call getNodePlaceholder on the parent via reflection
            if let Ok(get_placeholder) = js_sys::Reflect::get(parent, &JsValue::from_str("getNodePlaceholder")) {
                if let Ok(func) = get_placeholder.dyn_into::<js_sys::Function>() {
                    if let Ok(parent_placeholder) = func.call0(parent) {
                        if let Some(parent_str) = parent_placeholder.as_string() {
                            placeholder.push_str(&parent_str);
                        }
                    }
                }
            }
        }

        // Add this node's alias
        if let Ok(alias_val) = js_sys::Reflect::get(&self.node, &JsValue::from_str("alias")) {
            if let Some(alias) = alias_val.as_string() {
                placeholder.push_str(&alias);
            }
        }

        // Add [*] if iterable
        if self.is_iterable() {
            placeholder.push_str("[*]");
        }

        Ok(placeholder)
    }

    // Getters and setters
    #[wasm_bindgen(getter = node)]
    pub fn get_node(&self) -> JsValue {
        self.node.clone()
    }

    #[wasm_bindgen(setter = node)]
    pub fn set_node(&mut self, value: JsValue) {
        // Update datatype when node changes
        if let Ok(dt) = js_sys::Reflect::get(&value, &JsValue::from_str("datatype")) {
            self.datatype = dt.as_string();
        }
        self.node = value;
    }

    #[wasm_bindgen(getter = parentNode)]
    pub fn get_parent_node(&self) -> JsValue {
        self.parent_node.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = parentNode)]
    pub fn set_parent_node(&mut self, value: JsValue) {
        if value.is_null() {
            self.parent_node = None;
        } else {
            self.parent_node = Some(value);
        }
    }

    #[wasm_bindgen(getter = datatype)]
    pub fn get_datatype_property(&self) -> JsValue {
        match &self.datatype {
            Some(dt) => JsValue::from_str(dt),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = datatype)]
    pub fn set_datatype(&mut self, value: JsValue) {
        if value.is_null() {
            self.datatype = None;
        } else if let Some(s) = value.as_string() {
            self.datatype = Some(s);
        }
    }

    #[wasm_bindgen(getter = childNodes)]
    pub fn get_child_nodes(&self) -> JsValue {
        // Return the original Map to preserve reference
        self.child_nodes.clone()
    }

    #[wasm_bindgen(setter = childNodes)]
    pub fn set_child_nodes(&mut self, value: JsValue) {
        // Store the Map as-is to preserve reference
        self.child_nodes = value;
    }

    #[wasm_bindgen(getter = isOuter)]
    pub fn get_is_outer(&self) -> bool {
        self.is_outer
    }

    #[wasm_bindgen(setter = isOuter)]
    pub fn set_is_outer(&mut self, value: bool) {
        self.is_outer = value;
    }

    #[wasm_bindgen(getter = isInner)]
    pub fn get_is_inner(&self) -> bool {
        self.is_inner
    }

    #[wasm_bindgen(setter = isInner)]
    pub fn set_is_inner(&mut self, value: bool) {
        self.is_inner = value;
    }

    #[wasm_bindgen(getter = inner)]
    pub fn get_inner(&self) -> JsValue {
        self.inner.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = inner)]
    pub fn set_inner(&mut self, value: JsValue) {
        if value.is_null() {
            self.inner = None;
        } else {
            self.inner = Some(value);
        }
    }
}
