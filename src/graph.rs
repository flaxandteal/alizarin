use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
pub struct StaticGraphMeta {
    author: Option<String>,
    cards: Option<u32>,
    cards_x_nodes_x_widgets: Option<u32>,
    color: Option<String>,
    description: Option<HashMap<String, String>>,
    edges: Option<u32>,
    graphid: String,
    iconclass: Option<String>,
    is_editable: Option<bool>,
    isresource: Option<bool>,
    jsonldcontext: Option<HashMap<String, serde_json::Value>>,
    name: Option<HashMap<String, String>>,
    nodegroups: Option<u32>,
    nodes: Option<u32>,
    ontology_id: Option<String>,
    publication: Option<HashMap<String, Option<String>>>,
    relatable_resource_model_ids: Vec<String>,
    resource_2_resource_constraints: Vec<serde_json::Value>,
    root: Option<Box<StaticNode>>,
    slug: Option<String>,
    subtitle: Option<HashMap<String, String>>,
    version: Option<String>,
    extra_fields: HashMap<String, serde_json::Value>,
}

#[wasm_bindgen]
impl StaticGraphMeta {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticGraphMeta, JsValue> {
        let data: StaticGraphMeta = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen(getter = graphid)]
    pub fn get_graphid(&self) -> String {
        self.graphid.clone()
    }

    #[wasm_bindgen(setter = graphid)]
    pub fn set_graphid(&mut self, value: String) {
        self.graphid = value;
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }

    #[wasm_bindgen(js_name = getAuthor)]
    pub fn get_author(&self) -> Option<String> {
        self.author.clone()
    }

    #[wasm_bindgen(js_name = setAuthor)]
    pub fn set_author(&mut self, value: Option<String>) {
        self.author = value;
    }

    #[wasm_bindgen(js_name = getIsResource)]
    pub fn get_isresource(&self) -> Option<bool> {
        self.isresource
    }

    #[wasm_bindgen(js_name = setIsResource)]
    pub fn set_isresource(&mut self, value: Option<bool>) {
        self.isresource = value;
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
pub struct StaticNode {
    alias: Option<String>,
    config: HashMap<String, serde_json::Value>,
    datatype: String,
    description: Option<String>,
    exportable: bool,
    fieldname: Option<String>,
    graph_id: String,
    hascustomalias: bool,
    is_collector: bool,
    isrequired: bool,
    issearchable: bool,
    istopnode: bool,
    name: String,
    nodegroup_id: Option<String>,
    nodeid: String,
    ontologyclass: Option<String>,
    parentproperty: Option<String>,
    sortorder: i32,
    sourcebranchpublication_id: Option<String>,
}

#[wasm_bindgen]
impl StaticNode {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticNode, JsValue> {
        let data: StaticNode = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticNode {
        self.clone()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }

    // Getters and setters for wasm-bindgen
    #[wasm_bindgen(getter = alias)]
    pub fn get_alias(&self) -> Option<String> {
        self.alias.clone()
    }

    #[wasm_bindgen(setter = alias)]
    pub fn set_alias(&mut self, value: Option<String>) {
        self.alias = value;
    }

    #[wasm_bindgen(getter = datatype)]
    pub fn get_datatype(&self) -> String {
        self.datatype.clone()
    }

    #[wasm_bindgen(setter = datatype)]
    pub fn set_datatype(&mut self, value: String) {
        self.datatype = value;
    }

    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(setter = description)]
    pub fn set_description(&mut self, value: Option<String>) {
        self.description = value;
    }

    #[wasm_bindgen(getter = exportable)]
    pub fn get_exportable(&self) -> bool {
        self.exportable
    }

    #[wasm_bindgen(setter = exportable)]
    pub fn set_exportable(&mut self, value: bool) {
        self.exportable = value;
    }

    #[wasm_bindgen(getter = fieldname)]
    pub fn get_fieldname(&self) -> Option<String> {
        self.fieldname.clone()
    }

    #[wasm_bindgen(setter = fieldname)]
    pub fn set_fieldname(&mut self, value: Option<String>) {
        self.fieldname = value;
    }

    #[wasm_bindgen(getter = graph_id)]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(setter = graph_id)]
    pub fn set_graph_id(&mut self, value: String) {
        self.graph_id = value;
    }

    #[wasm_bindgen(getter = hascustomalias)]
    pub fn get_hascustomalias(&self) -> bool {
        self.hascustomalias
    }

    #[wasm_bindgen(setter = hascustomalias)]
    pub fn set_hascustomalias(&mut self, value: bool) {
        self.hascustomalias = value;
    }

    #[wasm_bindgen(getter = is_collector)]
    pub fn get_is_collector(&self) -> bool {
        self.is_collector
    }

    #[wasm_bindgen(setter = is_collector)]
    pub fn set_is_collector(&mut self, value: bool) {
        self.is_collector = value;
    }

    #[wasm_bindgen(getter = isrequired)]
    pub fn get_isrequired(&self) -> bool {
        self.isrequired
    }

    #[wasm_bindgen(setter = isrequired)]
    pub fn set_isrequired(&mut self, value: bool) {
        self.isrequired = value;
    }

    #[wasm_bindgen(getter = issearchable)]
    pub fn get_issearchable(&self) -> bool {
        self.issearchable
    }

    #[wasm_bindgen(setter = issearchable)]
    pub fn set_issearchable(&mut self, value: bool) {
        self.issearchable = value;
    }

    #[wasm_bindgen(getter = istopnode)]
    pub fn get_istopnode(&self) -> bool {
        self.istopnode
    }

    #[wasm_bindgen(setter = istopnode)]
    pub fn set_istopnode(&mut self, value: bool) {
        self.istopnode = value;
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(setter = name)]
    pub fn set_name(&mut self, value: String) {
        self.name = value;
    }

    #[wasm_bindgen(getter = nodegroup_id)]
    pub fn get_nodegroup_id(&self) -> Option<String> {
        self.nodegroup_id.clone()
    }

    #[wasm_bindgen(setter = nodegroup_id)]
    pub fn set_nodegroup_id(&mut self, value: Option<String>) {
        self.nodegroup_id = value;
    }

    #[wasm_bindgen(getter = nodeid)]
    pub fn get_nodeid(&self) -> String {
        self.nodeid.clone()
    }

    #[wasm_bindgen(setter = nodeid)]
    pub fn set_nodeid(&mut self, value: String) {
        self.nodeid = value;
    }

    #[wasm_bindgen(getter = ontologyclass)]
    pub fn get_ontologyclass(&self) -> Option<String> {
        self.ontologyclass.clone()
    }

    #[wasm_bindgen(setter = ontologyclass)]
    pub fn set_ontologyclass(&mut self, value: Option<String>) {
        self.ontologyclass = value;
    }

    #[wasm_bindgen(getter = parentproperty)]
    pub fn get_parentproperty(&self) -> Option<String> {
        self.parentproperty.clone()
    }

    #[wasm_bindgen(setter = parentproperty)]
    pub fn set_parentproperty(&mut self, value: Option<String>) {
        self.parentproperty = value;
    }

    #[wasm_bindgen(getter = sortorder)]
    pub fn get_sortorder(&self) -> i32 {
        self.sortorder
    }

    #[wasm_bindgen(setter = sortorder)]
    pub fn set_sortorder(&mut self, value: i32) {
        self.sortorder = value;
    }

    #[wasm_bindgen(getter = sourcebranchpublication_id)]
    pub fn get_sourcebranchpublication_id(&self) -> Option<String> {
        self.sourcebranchpublication_id.clone()
    }

    #[wasm_bindgen(setter = sourcebranchpublication_id)]
    pub fn set_sourcebranchpublication_id(&mut self, value: Option<String>) {
        self.sourcebranchpublication_id = value;
    }

    // Config getter/setter using JsValue
    #[wasm_bindgen(js_name = getConfig)]
    pub fn get_config(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.config)?)
    }

    #[wasm_bindgen(js_name = setConfig)]
    pub fn set_config(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.config = serde_wasm_bindgen::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = compare)]
    pub fn compare_static(node_a: JsValue, node_b: JsValue) -> Result<JsValue, JsValue> {
        let a: StaticNode = serde_wasm_bindgen::from_value(node_a)?;
        let b: StaticNode = serde_wasm_bindgen::from_value(node_b)?;
        
        let result = StaticNode::compare(&a, &b);
        Ok(serde_wasm_bindgen::to_value(&result)?)
    }
}

impl StaticNode {
    // Internal compare function
    pub fn compare(node_a: &StaticNode, node_b: &StaticNode) -> serde_json::Value {
        use serde_json::json;
        
        // Check if they're the same object (comparing memory addresses)
        if std::ptr::eq(node_a, node_b) {
            return json!(true);
        }

        // Compare all fields except nodeid, graph_id, nodegroup_id
        let mut identical = true;
        
        if node_a.alias != node_b.alias { identical = false; }
        if node_a.config != node_b.config { identical = false; }
        if node_a.datatype != node_b.datatype { identical = false; }
        if node_a.description != node_b.description { identical = false; }
        if node_a.exportable != node_b.exportable { identical = false; }
        if node_a.fieldname != node_b.fieldname { identical = false; }
        if node_a.hascustomalias != node_b.hascustomalias { identical = false; }
        if node_a.is_collector != node_b.is_collector { identical = false; }
        if node_a.isrequired != node_b.isrequired { identical = false; }
        if node_a.issearchable != node_b.issearchable { identical = false; }
        if node_a.istopnode != node_b.istopnode { identical = false; }
        if node_a.name != node_b.name { identical = false; }
        if node_a.ontologyclass != node_b.ontologyclass { identical = false; }
        if node_a.parentproperty != node_b.parentproperty { identical = false; }
        if node_a.sortorder != node_b.sortorder { identical = false; }
        if node_a.sourcebranchpublication_id != node_b.sourcebranchpublication_id { identical = false; }

        if !identical {
            return json!(false);
        }

        // They're identical except possibly for IDs
        if node_a.graph_id.is_empty() || node_b.graph_id.is_empty() || node_a.graph_id != node_b.graph_id {
            if !node_a.graph_id.is_empty() && !node_b.graph_id.is_empty() {
                return json!(-3);
            }
        }

        if let (Some(ref a_ng), Some(ref b_ng)) = (&node_a.nodegroup_id, &node_b.nodegroup_id) {
            if a_ng != b_ng {
                return json!(-2);
            }
        }

        if node_a.nodeid.is_empty() || node_b.nodeid.is_empty() || node_a.nodeid != node_b.nodeid {
            if !node_a.nodeid.is_empty() && !node_b.nodeid.is_empty() {
                return json!(-1);
            }
        }

        // Check if all IDs match (when both are non-falsey)
        let graph_id_match = node_a.graph_id.is_empty() || node_b.graph_id.is_empty() || node_a.graph_id == node_b.graph_id;
        let nodegroup_id_match = match (&node_a.nodegroup_id, &node_b.nodegroup_id) {
            (None, _) | (_, None) => true,
            (Some(a), Some(b)) => a.is_empty() || b.is_empty() || a == b,
        };
        let nodeid_match = node_a.nodeid.is_empty() || node_b.nodeid.is_empty() || node_a.nodeid == node_b.nodeid;

        if graph_id_match && nodegroup_id_match && nodeid_match {
            json!(2)
        } else {
            json!(1)
        }
    }
}