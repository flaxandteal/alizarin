//! The WASM `PseudoNode`: core's [`PseudoNodeCore`] with JS getters. The logic
//! (the outer/inner split, placeholders, navigation) is core's, shared with
//! NAPI and Python.

use alizarin_core::PseudoNodeCore;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct PseudoNode {
    core: PseudoNodeCore,
}

impl PseudoNode {
    pub(crate) fn from_core(core: PseudoNodeCore) -> PseudoNode {
        PseudoNode { core }
    }

    pub(crate) fn core(&self) -> &PseudoNodeCore {
        &self.core
    }
}

#[wasm_bindgen]
impl PseudoNode {
    #[wasm_bindgen(js_name = isIterable)]
    pub fn is_iterable(&self) -> bool {
        self.core.is_iterable()
    }

    /// The path from where navigation started (`.group.basic_info`), with
    /// `[*]` after a list node: core's.
    #[wasm_bindgen(js_name = getNodePlaceholder)]
    pub fn get_node_placeholder(&self) -> String {
        self.core.placeholder()
    }

    // Getter that returns a JavaScript object with all node fields
    // This maintains compatibility while keeping StaticNode private
    #[wasm_bindgen(getter = node)]
    pub fn get_node(&self) -> JsValue {
        let node = &self.core.node;
        let obj = js_sys::Object::new();

        // SILENT: Reflect::set on Object::new() cannot fail (not frozen/proxied).
        js_sys::Reflect::set(&obj, &"nodeid".into(), &node.nodeid.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"name".into(), &node.name.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"datatype".into(), &node.datatype.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"graph_id".into(), &node.graph_id.clone().into()).ok();
        js_sys::Reflect::set(&obj, &"exportable".into(), &node.exportable.into()).ok();
        js_sys::Reflect::set(&obj, &"hascustomalias".into(), &node.hascustomalias.into()).ok();
        js_sys::Reflect::set(&obj, &"is_collector".into(), &node.is_collector.into()).ok();
        js_sys::Reflect::set(&obj, &"isrequired".into(), &node.isrequired.into()).ok();
        js_sys::Reflect::set(&obj, &"issearchable".into(), &node.issearchable.into()).ok();
        js_sys::Reflect::set(&obj, &"istopnode".into(), &node.istopnode.into()).ok();
        if let Some(sortorder) = node.sortorder {
            js_sys::Reflect::set(&obj, &"sortorder".into(), &sortorder.into()).ok();
        }

        // Handle Option fields
        if let Some(ref alias) = node.alias {
            js_sys::Reflect::set(&obj, &"alias".into(), &alias.clone().into()).ok();
        }
        if let Some(ref description) = node.description {
            // Convert core type to WASM wrapper for JS serialization
            let wasm_desc = crate::graph::StaticTranslatableString(description.clone());
            js_sys::Reflect::set(&obj, &"description".into(), &wasm_desc.to_json()).ok();
        }
        // Serialise ontology classes using the shared Arches wire-shape helper.
        let classes_js = crate::utils::classes_to_js_value(node.ontologyclass.as_ref());
        if !classes_js.is_null() {
            js_sys::Reflect::set(&obj, &"ontologyclass".into(), &classes_js).ok();
        }
        if let Some(ref nodegroup_id) = node.nodegroup_id {
            js_sys::Reflect::set(&obj, &"nodegroup_id".into(), &nodegroup_id.clone().into()).ok();
        }
        if let Some(ref fieldname) = node.fieldname {
            js_sys::Reflect::set(&obj, &"fieldname".into(), &fieldname.clone().into()).ok();
        }
        if let Some(ref parentproperty) = node.parentproperty {
            js_sys::Reflect::set(
                &obj,
                &"parentproperty".into(),
                &parentproperty.clone().into(),
            )
            .ok();
        }
        if let Some(ref sourcebranchpublication_id) = node.sourcebranchpublication_id {
            js_sys::Reflect::set(
                &obj,
                &"sourcebranchpublication_id".into(),
                &sourcebranchpublication_id.clone().into(),
            )
            .ok();
        }

        // Convert config HashMap to plain JavaScript object
        // SILENT: serde_json::Value -> JsValue can't fail (already valid JSON);
        // Reflect::set on Object::new() can't fail.
        let config_obj = js_sys::Object::new();
        for (key, value) in &node.config {
            let js_value = crate::js_json::to_value(value).unwrap_or(JsValue::NULL);
            js_sys::Reflect::set(&config_obj, &JsValue::from_str(key), &js_value).ok();
        }
        js_sys::Reflect::set(&obj, &"config".into(), &config_obj).ok();

        obj.into()
    }

    #[wasm_bindgen(getter = size)]
    pub fn get_size(&self) -> usize {
        self.core.child_nodes.len()
    }

    #[wasm_bindgen(getter = childNodeAliases)]
    pub fn get_child_node_aliases(&self) -> JsValue {
        let map = js_sys::Array::new();
        for key in self.core.child_nodes.keys() {
            map.push(&JsValue::from_str(key));
        }
        map.into()
    }

    #[wasm_bindgen(getter = childNodes)]
    pub fn get_child_nodes(&self) -> JsValue {
        let map = js_sys::Map::new();
        // Create a serializer that uses plain objects instead of Maps
        for (key, value) in &self.core.child_nodes {
            let node_obj = crate::js_json::to_value(value.as_ref()).unwrap_or(JsValue::NULL);
            map.set(&JsValue::from_str(key), &node_obj);
        }
        map.into()
    }

    #[wasm_bindgen(getter = nodeid)]
    pub fn get_nodeid(&self) -> String {
        self.core.node.nodeid.clone()
    }

    #[wasm_bindgen(getter = alias)]
    pub fn get_alias(&self) -> JsValue {
        match &self.core.node.alias {
            Some(s) => JsValue::from_str(s),
            None => JsValue::from_str(""),
        }
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> String {
        self.core.node.name.clone()
    }

    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> JsValue {
        if let Some(ref description) = self.core.node.description {
            crate::js_json::to_value(&description.to_json()).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }

    #[wasm_bindgen(getter = graphId)]
    pub fn get_graph_id(&self) -> String {
        self.core.node.graph_id.clone()
    }

    #[wasm_bindgen(getter = ontologyclass)]
    pub fn get_ontologyclass(&self) -> JsValue {
        crate::utils::classes_to_js_value(self.core.node.ontologyclass.as_ref())
    }

    #[wasm_bindgen(getter = nodegroup_id)]
    pub fn get_nodegroup_id(&self) -> JsValue {
        match &self.core.node.nodegroup_id {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = fieldname)]
    pub fn get_fieldname(&self) -> JsValue {
        match &self.core.node.fieldname {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = parentproperty)]
    pub fn get_parentproperty(&self) -> JsValue {
        match &self.core.node.parentproperty {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = sourcebranchpublicationId)]
    pub fn get_sourcebranchpublication_id(&self) -> JsValue {
        match &self.core.node.sourcebranchpublication_id {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter = exportable)]
    pub fn get_exportable(&self) -> bool {
        self.core.node.exportable
    }

    #[wasm_bindgen(getter = hascustomalias)]
    pub fn get_hascustomalias(&self) -> bool {
        self.core.node.hascustomalias
    }

    #[wasm_bindgen(getter = isCollector)]
    pub fn get_is_collector(&self) -> bool {
        self.core.node.is_collector
    }

    #[wasm_bindgen(getter = isrequired)]
    pub fn get_isrequired(&self) -> bool {
        self.core.node.isrequired
    }

    #[wasm_bindgen(getter = issearchable)]
    pub fn get_issearchable(&self) -> bool {
        self.core.node.issearchable
    }

    #[wasm_bindgen(getter = istopnode)]
    pub fn get_istopnode(&self) -> bool {
        self.core.node.istopnode
    }

    #[wasm_bindgen(getter = sortorder, unchecked_return_type = "number | null")]
    pub fn get_sortorder(&self) -> JsValue {
        crate::js_json::or_null(self.core.node.sortorder)
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        // Convert HashMap to JavaScript object (not Map!)
        let obj = js_sys::Object::new();
        for (key, value) in &self.core.node.config {
            let js_value = crate::js_json::to_value(value).unwrap_or(JsValue::NULL);
            js_sys::Reflect::set(&obj, &JsValue::from_str(key), &js_value).ok();
        }
        obj.into()
    }

    /// The datatype, read as `semantic` on an inner node.
    #[wasm_bindgen(getter = datatype)]
    pub fn get_datatype(&self) -> String {
        self.core.datatype().to_string()
    }

    #[wasm_bindgen(getter = isOuter)]
    pub fn get_is_outer(&self) -> bool {
        self.core.is_outer()
    }

    #[wasm_bindgen(getter = inner)]
    pub fn get_inner(&self) -> Result<PseudoNode, JsValue> {
        match self.core.inner.as_ref() {
            Some(inner) => Ok(PseudoNode::from_core((**inner).clone())),
            None => Err(JsValue::from_str("No inner node")),
        }
    }

    #[wasm_bindgen(getter = isInner)]
    pub fn get_is_inner(&self) -> bool {
        self.core.is_inner
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        crate::js_json::to_value(&self.core.node.to_json()).unwrap_or(JsValue::NULL)
    }
}
