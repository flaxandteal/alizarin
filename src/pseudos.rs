use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::{Rc, Weak};
use crate::graph::{WKRM, StaticGraph, StaticNode, StaticNodegroup, StaticEdge};

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
    node: Arc<StaticNode>,
    // Must use JsValue for circular references (can't use PseudoNode due to recursion)
    parent_node: Option<JsValue>,
    // Store as JsValue (the original Map) to preserve reference
    pub(crate)
    child_nodes: HashMap<String, Arc<StaticNode>>,
    is_inner: bool,
    // Must use JsValue for circular references (can't use PseudoNode due to recursion)
    inner: Option<Rc<PseudoNode>>,
}

#[wasm_bindgen]
impl PseudoNode {
    // Internal constructor that takes StaticNode directly (for Rust use)
    pub(crate) fn new_from_static_node(
        static_node: Arc<crate::graph::StaticNode>,
        child_nodes: HashMap<String, Arc<StaticNode>>,
    ) -> Result<PseudoNode, JsValue> {
        let is_semantic = static_node.datatype == "semantic";
        let has_children = !child_nodes.is_empty();
        let has_inner = has_children && !is_semantic;
        let (inner, final_child_nodes) = if has_inner {
            let inner = Rc::new({
                PseudoNode {
                    node: static_node.clone(),
                    parent_node: None,
                    child_nodes: child_nodes,
                    is_inner: true,
                    inner: None,
                }
            });
            (Some(inner), HashMap::new()) // Why is this not just None in 2nd?
        } else {
            (None, child_nodes)
        };

        Ok(PseudoNode {
            node: static_node,
            parent_node: None,
            child_nodes: final_child_nodes,
            is_inner: false,
            inner,
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
        if let Some(sortorder) = self.node.sortorder {
            js_sys::Reflect::set(&obj, &"sortorder".into(), &sortorder.into()).ok();
        }

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

    #[wasm_bindgen(getter = size)]
    pub fn get_size(&self) -> usize {
        self.child_nodes.len()
    }

    #[wasm_bindgen(getter = childNodes)]
    pub fn get_child_nodes(&self) -> JsValue {
        let map = js_sys::Map::new();
        for (key, value) in &self.child_nodes {
            let node_obj = value.to_json();
            map.set(&JsValue::from_str(key), &node_obj);
        }
        map.into()
    }

    #[wasm_bindgen(getter = nodeid)]
    pub fn get_nodeid(&self) -> String {
        self.node.nodeid.clone()
    }

    #[wasm_bindgen(getter = alias)]
    pub fn get_alias(&self) -> JsValue {
        match &self.node.alias {
            Some(s) => JsValue::from_str(s),
            None => JsValue::from_str(""),
        }
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> String {
        self.node.name.clone()
    }

    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> JsValue {
        if let Some(description) = self.node.description.clone() {
            description.to_json()
        } else { JsValue::NULL }
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
    pub fn get_sortorder(&self) -> Option<i32> {
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

    #[wasm_bindgen(getter = isOuter)]
    pub fn get_is_outer(&self) -> bool {
        self.inner.is_some()
    }

    #[wasm_bindgen(getter = inner)]
    pub fn get_inner(&self) -> Result<PseudoNode, JsValue> {
        match self.inner.as_ref() {
            Some(inner) => Ok((**inner).clone()),
            None => Err(JsValue::from_str("No inner node"))
        }
    }

    #[wasm_bindgen(getter = isInner)]
    pub fn get_is_inner(&self) -> bool {
        self.is_inner
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        self.node.to_json()
    }
}
