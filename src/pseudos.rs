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
    // Store as StaticNode - node data is now private to Rust
    node: crate::graph::StaticNode,
    // Must use JsValue for circular references (can't use PseudoNode due to recursion)
    parent_node: Option<JsValue>,
    // Store as JsValue (the original Map) to preserve reference
    child_nodes: JsValue,
    is_outer: bool,
    is_inner: bool,
    // Must use JsValue for circular references (can't use PseudoNode due to recursion)
    inner: Option<JsValue>,
}

#[wasm_bindgen]
impl PseudoNode {
    // JavaScript-exposed constructor
    #[wasm_bindgen(constructor)]
    pub fn new(
        static_node: crate::graph::StaticNode,
        child_nodes: JsValue,
        inner: JsValue,
    ) -> PseudoNode {
        Self::new_from_static_node(static_node, child_nodes, inner)
            .expect("Failed to create PseudoNode")
    }

    // Internal constructor that takes StaticNode directly (for Rust use)
    pub(crate) fn new_from_static_node(
        static_node: crate::graph::StaticNode,
        child_nodes: JsValue,
        inner: JsValue,
    ) -> Result<PseudoNode, JsValue> {
        let mut is_outer = false;
        let mut is_inner = false;
        let mut inner_val: Option<JsValue> = None;

        // Handle the inner parameter
        if !inner.is_null() && !inner.is_undefined() {
            // Check if it's a boolean true
            if let Some(bool_val) = inner.as_bool() {
                if bool_val {
                    is_inner = true;
                }
            } else {
                // It's a PseudoValue/PseudoNode instance
                is_outer = true;
                inner_val = Some(inner);
            }
        }

        Ok(PseudoNode {
            node: static_node,
            parent_node: None,
            child_nodes,
            is_outer,
            is_inner,
            inner: inner_val,
        })
    }

    #[wasm_bindgen(js_name = isIterable)]
    pub fn is_iterable(&self) -> bool {
        ITERABLE_DATATYPES.contains(&self.node.datatype.as_str())
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

        // Add this node's alias - direct field access from StaticNode
        if let Some(ref alias) = self.node.alias {
            placeholder.push_str(alias);
        }

        // Add [*] if iterable
        if self.is_iterable() {
            placeholder.push_str("[*]");
        }

        Ok(placeholder)
    }

    // Individual getters for StaticNode fields (node is now private)

    // Getter that returns a JavaScript object with all node fields
    // This maintains compatibility while keeping StaticNode private
    #[wasm_bindgen(getter = node)]
    pub fn get_node(&self) -> JsValue {
        let obj = js_sys::Object::new();

        // Set all fields on the JavaScript object
        js_sys::Reflect::set(&obj, &"nodeid".into(), &self.node.nodeid.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"name".into(), &self.node.name.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"datatype".into(), &self.node.datatype.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"graph_id".into(), &self.node.graph_id.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"exportable".into(), &self.node.exportable.into()).ok();
        js_sys::Reflect::set(&obj, &"hascustomalias".into(), &self.node.hascustomalias.into()).ok();
        js_sys::Reflect::set(&obj, &"is_collector".into(), &self.node.is_collector.into()).ok();
        js_sys::Reflect::set(&obj, &"isrequired".into(), &self.node.isrequired.into()).ok();
        js_sys::Reflect::set(&obj, &"issearchable".into(), &self.node.issearchable.into()).ok();
        js_sys::Reflect::set(&obj, &"istopnode".into(), &self.node.istopnode.into()).ok();
        js_sys::Reflect::set(&obj, &"sortorder".into(), &self.node.sortorder.into()).ok();

        // Handle Option fields
        if let Some(ref alias) = self.node.alias {
            js_sys::Reflect::set(&obj, &"alias".into(), &alias.clone().into()).ok();
        }
        if let Some(ref description) = self.node.description {
            js_sys::Reflect::set(&obj, &"description".into(), &description.clone().into()).ok();
        }
        if let Some(ref ontologyclass) = self.node.ontologyclass {
            js_sys::Reflect::set(&obj, &"ontologyclass".into(), &ontologyclass.clone().into()).ok();
        }
        if let Some(ref nodegroup_id) = self.node.nodegroup_id {
            js_sys::Reflect::set(&obj, &"nodegroup_id".into(), &nodegroup_id.clone().into()).ok();
        }
        if let Some(ref fieldname) = self.node.fieldname {
            js_sys::Reflect::set(&obj, &"fieldname".into(), &fieldname.clone().into()).ok();
        }
        if let Some(ref parentproperty) = self.node.parentproperty {
            js_sys::Reflect::set(&obj, &"parentproperty".into(), &parentproperty.clone().into()).ok();
        }
        if let Some(ref sourcebranchpublication_id) = self.node.sourcebranchpublication_id {
            js_sys::Reflect::set(&obj, &"sourcebranchpublication_id".into(), &sourcebranchpublication_id.clone().into()).ok();
        }

        // Convert config HashMap to plain JavaScript object
        let config_obj = js_sys::Object::new();
        for (key, value) in &self.node.config {
            let js_value = serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL);
            js_sys::Reflect::set(&config_obj, &JsValue::from_str(key), &js_value).ok();
        }
        js_sys::Reflect::set(&obj, &"config".into(), &config_obj).ok();

        obj.into()
    }

    #[wasm_bindgen(getter = nodeid)]
    pub fn get_nodeid(&self) -> String {
        self.node.nodeid.clone()
    }

    #[wasm_bindgen(getter = alias)]
    pub fn get_alias(&self) -> JsValue {
        match &self.node.alias {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> String {
        self.node.name.clone()
    }

    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> JsValue {
        match &self.node.description {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = graphId)]
    pub fn get_graph_id(&self) -> String {
        self.node.graph_id.clone()
    }

    #[wasm_bindgen(getter = ontologyclass)]
    pub fn get_ontologyclass(&self) -> JsValue {
        match &self.node.ontologyclass {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = nodegroup_id)]
    pub fn get_nodegroup_id(&self) -> JsValue {
        match &self.node.nodegroup_id {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = fieldname)]
    pub fn get_fieldname(&self) -> JsValue {
        match &self.node.fieldname {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = parentproperty)]
    pub fn get_parentproperty(&self) -> JsValue {
        match &self.node.parentproperty {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = sourcebranchpublicationId)]
    pub fn get_sourcebranchpublication_id(&self) -> JsValue {
        match &self.node.sourcebranchpublication_id {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = exportable)]
    pub fn get_exportable(&self) -> bool {
        self.node.exportable
    }

    #[wasm_bindgen(getter = hascustomalias)]
    pub fn get_hascustomalias(&self) -> bool {
        self.node.hascustomalias
    }

    #[wasm_bindgen(getter = isCollector)]
    pub fn get_is_collector(&self) -> bool {
        self.node.is_collector
    }

    #[wasm_bindgen(getter = isrequired)]
    pub fn get_isrequired(&self) -> bool {
        self.node.isrequired
    }

    #[wasm_bindgen(getter = issearchable)]
    pub fn get_issearchable(&self) -> bool {
        self.node.issearchable
    }

    #[wasm_bindgen(getter = istopnode)]
    pub fn get_istopnode(&self) -> bool {
        self.node.istopnode
    }

    #[wasm_bindgen(getter = sortorder)]
    pub fn get_sortorder(&self) -> i32 {
        self.node.sortorder
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        // Convert HashMap to JavaScript object (not Map!)
        let obj = js_sys::Object::new();
        for (key, value) in &self.node.config {
            let js_value = serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL);
            js_sys::Reflect::set(&obj, &JsValue::from_str(key), &js_value).ok();
        }
        obj.into()
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config(&mut self, value: JsValue) {
        // Convert JavaScript object to HashMap
        self.node.config.clear();

        if value.is_object() && !value.is_null() {
            let keys = js_sys::Object::keys(&value.clone().into());
            for i in 0..keys.length() {
                if let Some(key_str) = keys.get(i).as_string() {
                    if let Ok(val) = js_sys::Reflect::get(&value, &JsValue::from_str(&key_str)) {
                        if let Ok(rust_val) = serde_wasm_bindgen::from_value(val) {
                            self.node.config.insert(key_str, rust_val);
                        }
                    }
                }
            }
        }
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
    pub fn get_datatype(&self) -> String {
        // Override to "semantic" if is_inner is true
        if self.is_inner {
            "semantic".to_string()
        } else {
            self.node.datatype.clone()
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

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        self.node.to_json()
    }
}
