use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize, Deserializer};
use std::collections::HashMap;

// StaticTranslatableString - A string with translations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct StaticTranslatableString {
    translations: HashMap<String, String>,
    lang: String,
}

#[wasm_bindgen]
impl StaticTranslatableString {
    #[wasm_bindgen(constructor)]
    pub fn new(value: JsValue, lang: Option<String>) -> StaticTranslatableString {
        let default_lang = lang.unwrap_or_else(|| "en".to_string());

        // Try to parse as object (translations map)
        if let Ok(obj) = serde_wasm_bindgen::from_value::<HashMap<String, String>>(value.clone()) {
            let current_lang = if obj.contains_key(&default_lang) {
                default_lang
            } else {
                obj.keys().next().cloned().unwrap_or_else(|| "en".to_string())
            };

            return StaticTranslatableString {
                translations: obj,
                lang: current_lang,
            };
        }

        // Try as string
        if let Some(s) = value.as_string() {
            let mut translations = HashMap::new();
            translations.insert(default_lang.clone(), s);
            return StaticTranslatableString {
                translations,
                lang: default_lang,
            };
        }

        // Empty string fallback
        let mut translations = HashMap::new();
        translations.insert(default_lang.clone(), String::new());
        StaticTranslatableString {
            translations,
            lang: default_lang,
        }
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        self.translations.get(&self.lang)
            .or_else(|| self.translations.values().next())
            .cloned()
            .unwrap_or_default()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        // Create a plain JS object (not a Map)
        let js_obj = js_sys::Object::new();
        for (key, val) in &self.translations {
            js_sys::Reflect::set(
                &js_obj,
                &JsValue::from_str(key),
                &JsValue::from_str(val)
            ).ok();
        }
        js_obj.into()
    }

    #[wasm_bindgen(js_name = copy)]
    pub fn copy(&self) -> StaticTranslatableString {
        self.clone()
    }

    #[wasm_bindgen(getter = lang)]
    pub fn get_lang(&self) -> String {
        self.lang.clone()
    }

    #[wasm_bindgen(getter = translations)]
    pub fn get_translations(&self) -> JsValue {
        // Create a plain JS object (not a Map)
        let js_obj = js_sys::Object::new();
        for (key, val) in &self.translations {
            js_sys::Reflect::set(
                &js_obj,
                &JsValue::from_str(key),
                &JsValue::from_str(val)
            ).ok();
        }
        js_obj.into()
    }
}

// Custom serde implementation for StaticTranslatableString
impl Serialize for StaticTranslatableString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.translations.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StaticTranslatableString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde_json::Value;
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Object(map) => {
                let translations: HashMap<String, String> = map
                    .into_iter()
                    .filter_map(|(k, v)| {
                        v.as_str().map(|s| (k, s.to_string()))
                    })
                    .collect();

                let lang = translations.keys().next()
                    .cloned()
                    .unwrap_or_else(|| "en".to_string());

                Ok(StaticTranslatableString { translations, lang })
            }
            Value::String(s) => {
                let mut translations = HashMap::new();
                translations.insert("en".to_string(), s);
                Ok(StaticTranslatableString {
                    translations,
                    lang: "en".to_string(),
                })
            }
            _ => {
                let mut translations = HashMap::new();
                translations.insert("en".to_string(), String::new());
                Ok(StaticTranslatableString {
                    translations,
                    lang: "en".to_string(),
                })
            }
        }
    }
}

// Helper to deserialize array or number as Option<u32> (count)
fn deserialize_array_or_count<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::Number(n) => Ok(n.as_u64().map(|v| v as u32)),
        Value::Array(arr) => Ok(Some(arr.len() as u32)),
        _ => Ok(None),
    }
}

// Helper to deserialize string or object as Option<String>
fn deserialize_string_or_object<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::String(s) => Ok(Some(s)),
        Value::Object(_) => {
            // If it's an object (like a TranslatableString), serialize it back to string
            Ok(Some(serde_json::to_string(&value).unwrap_or_default()))
        }
        _ => Ok(None),
    }
}

// Helper function to convert serde_json::Value to JsValue as plain JS object
// This avoids the HashMap -> Map conversion issue with serde_wasm_bindgen
fn json_to_js_value(value: &serde_json::Value) -> JsValue {
    use serde_json::Value;

    match value {
        Value::Null => JsValue::NULL,
        Value::Bool(b) => JsValue::from_bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsValue::from_f64(i as f64)
            } else if let Some(u) = n.as_u64() {
                JsValue::from_f64(u as f64)
            } else if let Some(f) = n.as_f64() {
                JsValue::from_f64(f)
            } else {
                JsValue::NULL
            }
        },
        Value::String(s) => JsValue::from_str(s),
        Value::Array(arr) => {
            let js_array = js_sys::Array::new();
            for item in arr {
                js_array.push(&json_to_js_value(item));
            }
            js_array.into()
        },
        Value::Object(obj) => {
            let js_obj = js_sys::Object::new();
            for (key, val) in obj {
                js_sys::Reflect::set(
                    &js_obj,
                    &JsValue::from_str(key),
                    &json_to_js_value(val)
                ).ok();
            }
            js_obj.into()
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
pub struct StaticGraphMeta {
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    cards: Option<u32>,
    #[serde(default)]
    cards_x_nodes_x_widgets: Option<u32>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    description: Option<StaticTranslatableString>,
    #[serde(default)]
    edges: Option<u32>,
    #[serde(default)]
    graphid: String,
    #[serde(default)]
    iconclass: Option<String>,
    #[serde(default)]
    is_editable: Option<bool>,
    #[serde(default)]
    isresource: Option<bool>,
    #[serde(default)]
    jsonldcontext: Option<serde_json::Value>,  // Can be a string (URL) or an object (inline context)
    #[serde(default)]
    name: Option<StaticTranslatableString>,
    #[serde(default)]
    nodegroups: Option<u32>,
    #[serde(default)]
    nodes: Option<u32>,
    #[serde(default)]
    ontology_id: Option<String>,
    #[serde(default)]
    publication: Option<HashMap<String, Option<String>>>,
    #[serde(default)]
    relatable_resource_model_ids: Vec<String>,
    #[serde(default)]
    resource_2_resource_constraints: Vec<serde_json::Value>,
    #[serde(default)]
    root: Option<Box<StaticNode>>,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    subtitle: Option<StaticTranslatableString>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    extra_fields: HashMap<String, serde_json::Value>,
}

#[wasm_bindgen]
impl StaticGraphMeta {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticGraphMeta, JsValue> {
        let data: StaticGraphMeta = serde_wasm_bindgen::from_value(json_data).map_err(|e| {
            let error_msg = format!("Failed to deserialize StaticGraphMeta: {:?}", e);
            web_sys::console::error_1(&error_msg.clone().into());
            JsValue::from_str(&error_msg)
        })?;
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
    pub fn to_json(&self) -> JsValue {
        // Manual serialization to plain JS object using our helper
        use serde_json::json;

        let mut obj = serde_json::Map::new();

        if let Some(ref val) = self.author {
            obj.insert("author".to_string(), json!(val));
        }
        if let Some(val) = self.cards {
            obj.insert("cards".to_string(), json!(val));
        }
        if let Some(val) = self.cards_x_nodes_x_widgets {
            obj.insert("cards_x_nodes_x_widgets".to_string(), json!(val));
        }
        if let Some(ref val) = self.color {
            obj.insert("color".to_string(), json!(val));
        }
        if let Some(ref val) = self.description {
            obj.insert("description".to_string(), json!(val));
        }
        if let Some(val) = self.edges {
            obj.insert("edges".to_string(), json!(val));
        }
        obj.insert("graphid".to_string(), json!(self.graphid));
        if let Some(ref val) = self.iconclass {
            obj.insert("iconclass".to_string(), json!(val));
        }
        if let Some(val) = self.is_editable {
            obj.insert("is_editable".to_string(), json!(val));
        }
        if let Some(val) = self.isresource {
            obj.insert("isresource".to_string(), json!(val));
        }
        if let Some(ref val) = self.jsonldcontext {
            obj.insert("jsonldcontext".to_string(), json!(val));
        }
        if let Some(ref val) = self.name {
            obj.insert("name".to_string(), json!(val));
        }
        if let Some(val) = self.nodegroups {
            obj.insert("nodegroups".to_string(), json!(val));
        }
        if let Some(val) = self.nodes {
            obj.insert("nodes".to_string(), json!(val));
        }
        if let Some(ref val) = self.ontology_id {
            obj.insert("ontology_id".to_string(), json!(val));
        }
        if let Some(ref val) = self.publication {
            obj.insert("publication".to_string(), json!(val));
        }
        if !self.relatable_resource_model_ids.is_empty() {
            obj.insert("relatable_resource_model_ids".to_string(), json!(self.relatable_resource_model_ids));
        }
        if !self.resource_2_resource_constraints.is_empty() {
            obj.insert("resource_2_resource_constraints".to_string(), json!(self.resource_2_resource_constraints));
        }
        if let Some(ref val) = self.root {
            obj.insert("root".to_string(), serde_json::to_value(val).unwrap_or(json!(null)));
        }
        if let Some(ref val) = self.slug {
            obj.insert("slug".to_string(), json!(val));
        }
        if let Some(ref val) = self.subtitle {
            obj.insert("subtitle".to_string(), json!(val));
        }
        if let Some(ref val) = self.version {
            obj.insert("version".to_string(), json!(val));
        }

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = author)]
    pub fn get_author_property(&self) -> Option<String> {
        self.author.clone()
    }

    #[wasm_bindgen(js_name = getAuthor)]
    pub fn get_author(&self) -> Option<String> {
        self.author.clone()
    }

    #[wasm_bindgen(js_name = setAuthor)]
    pub fn set_author(&mut self, value: Option<String>) {
        self.author = value;
    }

    #[wasm_bindgen(getter = isresource)]
    pub fn get_isresource_property(&self) -> Option<bool> {
        self.isresource
    }

    #[wasm_bindgen(js_name = getIsResource)]
    pub fn get_isresource(&self) -> Option<bool> {
        self.isresource
    }

    #[wasm_bindgen(js_name = setIsResource)]
    pub fn set_isresource(&mut self, value: Option<bool>) {
        self.isresource = value;
    }

    // Additional getters/setters for all fields
    #[wasm_bindgen(js_name = getCards)]
    pub fn get_cards(&self) -> Option<u32> {
        self.cards
    }

    #[wasm_bindgen(js_name = setCards)]
    pub fn set_cards(&mut self, value: Option<u32>) {
        self.cards = value;
    }

    #[wasm_bindgen(js_name = getCardsXNodesXWidgets)]
    pub fn get_cards_x_nodes_x_widgets(&self) -> Option<u32> {
        self.cards_x_nodes_x_widgets
    }

    #[wasm_bindgen(js_name = setCardsXNodesXWidgets)]
    pub fn set_cards_x_nodes_x_widgets(&mut self, value: Option<u32>) {
        self.cards_x_nodes_x_widgets = value;
    }

    #[wasm_bindgen(js_name = getColor)]
    pub fn get_color(&self) -> Option<String> {
        self.color.clone()
    }

    #[wasm_bindgen(js_name = setColor)]
    pub fn set_color(&mut self, value: Option<String>) {
        self.color = value;
    }

    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> Option<StaticTranslatableString> {
        self.description.clone()
    }

    #[wasm_bindgen(setter = description)]
    pub fn set_description(&mut self, value: Option<StaticTranslatableString>) {
        self.description = value;
    }

    #[wasm_bindgen(js_name = getEdges)]
    pub fn get_edges(&self) -> Option<u32> {
        self.edges
    }

    #[wasm_bindgen(js_name = setEdges)]
    pub fn set_edges(&mut self, value: Option<u32>) {
        self.edges = value;
    }

    #[wasm_bindgen(js_name = getIconClass)]
    pub fn get_iconclass(&self) -> Option<String> {
        self.iconclass.clone()
    }

    #[wasm_bindgen(js_name = setIconClass)]
    pub fn set_iconclass(&mut self, value: Option<String>) {
        self.iconclass = value;
    }

    #[wasm_bindgen(js_name = getIsEditable)]
    pub fn get_is_editable(&self) -> Option<bool> {
        self.is_editable
    }

    #[wasm_bindgen(js_name = setIsEditable)]
    pub fn set_is_editable(&mut self, value: Option<bool>) {
        self.is_editable = value;
    }

    #[wasm_bindgen(js_name = getJsonLdContext)]
    pub fn get_jsonldcontext(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.jsonldcontext)?)
    }

    #[wasm_bindgen(js_name = setJsonLdContext)]
    pub fn set_jsonldcontext(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.jsonldcontext = serde_wasm_bindgen::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> Option<StaticTranslatableString> {
        self.name.clone()
    }

    #[wasm_bindgen(setter = name)]
    pub fn set_name(&mut self, value: Option<StaticTranslatableString>) {
        self.name = value;
    }

    #[wasm_bindgen(js_name = getNodeGroups)]
    pub fn get_nodegroups(&self) -> Option<u32> {
        self.nodegroups
    }

    #[wasm_bindgen(js_name = setNodeGroups)]
    pub fn set_nodegroups(&mut self, value: Option<u32>) {
        self.nodegroups = value;
    }

    #[wasm_bindgen(js_name = getNodes)]
    pub fn get_nodes(&self) -> Option<u32> {
        self.nodes
    }

    #[wasm_bindgen(js_name = setNodes)]
    pub fn set_nodes(&mut self, value: Option<u32>) {
        self.nodes = value;
    }

    #[wasm_bindgen(js_name = getOntologyId)]
    pub fn get_ontology_id(&self) -> Option<String> {
        self.ontology_id.clone()
    }

    #[wasm_bindgen(js_name = setOntologyId)]
    pub fn set_ontology_id(&mut self, value: Option<String>) {
        self.ontology_id = value;
    }

    #[wasm_bindgen(js_name = getPublication)]
    pub fn get_publication(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.publication)?)
    }

    #[wasm_bindgen(js_name = setPublication)]
    pub fn set_publication(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.publication = serde_wasm_bindgen::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getRelatableResourceModelIds)]
    pub fn get_relatable_resource_model_ids(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.relatable_resource_model_ids)?)
    }

    #[wasm_bindgen(js_name = setRelatableResourceModelIds)]
    pub fn set_relatable_resource_model_ids(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.relatable_resource_model_ids = serde_wasm_bindgen::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getResource2ResourceConstraints)]
    pub fn get_resource_2_resource_constraints(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.resource_2_resource_constraints)?)
    }

    #[wasm_bindgen(js_name = setResource2ResourceConstraints)]
    pub fn set_resource_2_resource_constraints(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.resource_2_resource_constraints = serde_wasm_bindgen::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getRoot)]
    pub fn get_root(&self) -> Option<StaticNode> {
        self.root.as_ref().map(|b| (**b).clone())
    }

    #[wasm_bindgen(js_name = setRoot)]
    pub fn set_root(&mut self, value: Option<StaticNode>) {
        self.root = value.map(Box::new);
    }

    #[wasm_bindgen(getter = slug)]
    pub fn get_slug(&self) -> Option<String> {
        self.slug.clone()
    }

    #[wasm_bindgen(setter = slug)]
    pub fn set_slug(&mut self, value: Option<String>) {
        self.slug = value;
    }

    #[wasm_bindgen(getter = subtitle)]
    pub fn get_subtitle(&self) -> Option<StaticTranslatableString> {
        self.subtitle.clone()
    }

    #[wasm_bindgen(setter = subtitle)]
    pub fn set_subtitle(&mut self, value: Option<StaticTranslatableString>) {
        self.subtitle = value;
    }

    #[wasm_bindgen(js_name = getVersion)]
    pub fn get_version(&self) -> Option<String> {
        self.version.clone()
    }

    #[wasm_bindgen(js_name = setVersion)]
    pub fn set_version(&mut self, value: Option<String>) {
        self.version = value;
    }

}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) alias: Option<String>,
    #[serde(default)]
    pub(crate) config: HashMap<String, serde_json::Value>,
    pub(crate) datatype: String,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string_or_object", default)]
    pub(crate) description: Option<String>,
    pub(crate) exportable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fieldname: Option<String>,
    pub(crate) graph_id: String,
    pub(crate) hascustomalias: bool,
    pub(crate) is_collector: bool,
    pub(crate) isrequired: bool,
    pub(crate) issearchable: bool,
    pub(crate) istopnode: bool,
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) nodegroup_id: Option<String>,
    pub(crate) nodeid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) ontologyclass: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parentproperty: Option<String>,
    #[serde(default)]
    pub(crate) sortorder: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sourcebranchpublication_id: Option<String>,
}

fn default_datatype() -> String {
    "string".to_string()
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
    pub fn to_json(&self) -> JsValue {
        // Manual serialization to plain JS object using our helper
        use serde_json::json;

        let mut obj = serde_json::Map::new();

        if let Some(ref val) = self.alias {
            obj.insert("alias".to_string(), json!(val));
        }
        obj.insert("config".to_string(), json!(self.config));
        obj.insert("datatype".to_string(), json!(self.datatype));
        if let Some(ref val) = self.description {
            obj.insert("description".to_string(), json!(val));
        }
        obj.insert("exportable".to_string(), json!(self.exportable));
        if let Some(ref val) = self.fieldname {
            obj.insert("fieldname".to_string(), json!(val));
        }
        obj.insert("graph_id".to_string(), json!(self.graph_id));
        obj.insert("hascustomalias".to_string(), json!(self.hascustomalias));
        obj.insert("is_collector".to_string(), json!(self.is_collector));
        obj.insert("isrequired".to_string(), json!(self.isrequired));
        obj.insert("issearchable".to_string(), json!(self.issearchable));
        obj.insert("istopnode".to_string(), json!(self.istopnode));
        obj.insert("name".to_string(), json!(self.name));
        if let Some(ref val) = self.nodegroup_id {
            obj.insert("nodegroup_id".to_string(), json!(val));
        }
        obj.insert("nodeid".to_string(), json!(self.nodeid));
        if let Some(ref val) = self.ontologyclass {
            obj.insert("ontologyclass".to_string(), json!(val));
        }
        if let Some(ref val) = self.parentproperty {
            obj.insert("parentproperty".to_string(), json!(val));
        }
        if let Some(val) = self.sortorder {
            obj.insert("sortorder".to_string(), json!(val));
        }
        if let Some(ref val) = self.sourcebranchpublication_id {
            obj.insert("sourcebranchpublication_id".to_string(), json!(val));
        }

        json_to_js_value(&serde_json::Value::Object(obj))
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
    pub fn get_sortorder(&self) -> Option<i32> {
        self.sortorder
    }

    #[wasm_bindgen(setter = sortorder)]
    pub fn set_sortorder(&mut self, value: Option<i32>) {
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

    // Config getter/setter - config as plain JS object
    #[wasm_bindgen(getter = config)]
    pub fn get_config_property(&self) -> JsValue {
        // Convert HashMap to plain JS object (not Map)
        json_to_js_value(&serde_json::to_value(&self.config).unwrap_or(serde_json::json!({})))
    }

    #[wasm_bindgen(js_name = getConfig)]
    pub fn get_config(&self) -> JsValue {
        // Same as property getter, for method call compatibility
        self.get_config_property()
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config_property(&mut self, value: JsValue) {
        // Transparently deserialize JS object to HashMap
        match serde_wasm_bindgen::from_value(value) {
            Ok(config) => self.config = config,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to set config: {}", e).into());
            }
        }
    }

    #[wasm_bindgen(js_name = setConfig)]
    pub fn set_config(&mut self, value: JsValue) {
        // Same as property setter, for method call compatibility
        self.set_config_property(value);
    }

    #[wasm_bindgen(js_name = compare)]
    pub fn compare_static(node_a: JsValue, node_b: JsValue) -> JsValue {
        // Helper to call toJSON if it exists, otherwise return as-is
        let normalize = |val: &JsValue| -> Result<JsValue, String> {
            // Check if the object has a toJSON method
            if let Ok(to_json) = js_sys::Reflect::get(val, &JsValue::from_str("toJSON")) {
                if to_json.is_function() {
                    let func: js_sys::Function = to_json.dyn_into().map_err(|_| "toJSON is not a function".to_string())?;
                    return func.call0(val).map_err(|e| format!("Failed to call toJSON: {:?}", e));
                }
            }
            Ok(val.clone())
        };

        // Normalize both inputs (call toJSON if available)
        let a_normalized = match normalize(&node_a) {
            Ok(v) => v,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to normalize node_a: {}", e).into());
                return JsValue::NULL;
            }
        };
        let b_normalized = match normalize(&node_b) {
            Ok(v) => v,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to normalize node_b: {}", e).into());
                return JsValue::NULL;
            }
        };

        // Convert to serde_json::Value to handle partial objects
        let a: serde_json::Value = match serde_wasm_bindgen::from_value(a_normalized) {
            Ok(v) => v,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to deserialize node_a: {:?}", e).into());
                return JsValue::NULL;
            }
        };
        let b: serde_json::Value = match serde_wasm_bindgen::from_value(b_normalized) {
            Ok(v) => v,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to deserialize node_b: {:?}", e).into());
                return JsValue::NULL;
            }
        };

        let result = StaticNode::compare_values(&a, &b);
        json_to_js_value(&result)
    }
}

// StaticNodegroup
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticNodegroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cardinality: Option<String>, // "1" | "n" | null
    pub(crate) legacygroupid: Option<String>, // Always null in practice
    pub(crate) nodegroupid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parentnodegroup_id: Option<String>,
}

#[wasm_bindgen]
impl StaticNodegroup {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticNodegroup, JsValue> {
        let data: StaticNodegroup = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticNodegroup {
        self.clone()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;
        let mut obj = serde_json::Map::new();

        if let Some(ref val) = self.cardinality {
            obj.insert("cardinality".to_string(), json!(val));
        } else {
            obj.insert("cardinality".to_string(), json!(null));
        }
        obj.insert("legacygroupid".to_string(), json!(null));
        obj.insert("nodegroupid".to_string(), json!(self.nodegroupid));
        if let Some(ref val) = self.parentnodegroup_id {
            obj.insert("parentnodegroup_id".to_string(), json!(val));
        } else {
            obj.insert("parentnodegroup_id".to_string(), json!(null));
        }

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = cardinality)]
    pub fn get_cardinality(&self) -> Option<String> {
        self.cardinality.clone()
    }

    #[wasm_bindgen(setter = cardinality)]
    pub fn set_cardinality(&mut self, value: Option<String>) {
        self.cardinality = value;
    }

    #[wasm_bindgen(getter = legacygroupid)]
    pub fn get_legacygroupid(&self) -> JsValue {
        JsValue::NULL
    }

    #[wasm_bindgen(getter = nodegroupid)]
    pub fn get_nodegroupid(&self) -> String {
        self.nodegroupid.clone()
    }

    #[wasm_bindgen(setter = nodegroupid)]
    pub fn set_nodegroupid(&mut self, value: String) {
        self.nodegroupid = value;
    }

    #[wasm_bindgen(getter = parentnodegroup_id)]
    pub fn get_parentnodegroup_id(&self) -> JsValue {
        match &self.parentnodegroup_id {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = parentnodegroup_id)]
    pub fn set_parentnodegroup_id(&mut self, value: Option<String>) {
        self.parentnodegroup_id = value;
    }
}

// StaticConstraint
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticConstraint {
    card_id: String,
    constraintid: String,
    nodes: Vec<String>,
    uniquetoallinstances: bool,
}

#[wasm_bindgen]
impl StaticConstraint {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticConstraint, JsValue> {
        let data: StaticConstraint = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;
        let mut obj = serde_json::Map::new();

        obj.insert("card_id".to_string(), json!(self.card_id));
        obj.insert("constraintid".to_string(), json!(self.constraintid));
        obj.insert("nodes".to_string(), json!(self.nodes));
        obj.insert("uniquetoallinstances".to_string(), json!(self.uniquetoallinstances));

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = card_id)]
    pub fn get_card_id(&self) -> String {
        self.card_id.clone()
    }

    #[wasm_bindgen(setter = card_id)]
    pub fn set_card_id(&mut self, value: String) {
        self.card_id = value;
    }

    #[wasm_bindgen(getter = constraintid)]
    pub fn get_constraintid(&self) -> String {
        self.constraintid.clone()
    }

    #[wasm_bindgen(setter = constraintid)]
    pub fn set_constraintid(&mut self, value: String) {
        self.constraintid = value;
    }

    #[wasm_bindgen(getter = nodes)]
    pub fn get_nodes(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.nodes).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = nodes)]
    pub fn set_nodes(&mut self, value: JsValue) {
        if let Ok(nodes) = serde_wasm_bindgen::from_value(value) {
            self.nodes = nodes;
        }
    }

    #[wasm_bindgen(getter = uniquetoallinstances)]
    pub fn get_uniquetoallinstances(&self) -> bool {
        self.uniquetoallinstances
    }

    #[wasm_bindgen(setter = uniquetoallinstances)]
    pub fn set_uniquetoallinstances(&mut self, value: bool) {
        self.uniquetoallinstances = value;
    }
}

// StaticPublication
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticPublication {
    graph_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    publicationid: String,
    published_time: String,
}

#[wasm_bindgen]
impl StaticPublication {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticPublication, JsValue> {
        let data: StaticPublication = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticPublication {
        self.clone()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;
        let mut obj = serde_json::Map::new();

        obj.insert("graph_id".to_string(), json!(self.graph_id));
        if let Some(ref val) = self.notes {
            obj.insert("notes".to_string(), json!(val));
        } else {
            obj.insert("notes".to_string(), json!(null));
        }
        obj.insert("publicationid".to_string(), json!(self.publicationid));
        obj.insert("published_time".to_string(), json!(self.published_time));

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = graph_id)]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(setter = graph_id)]
    pub fn set_graph_id(&mut self, value: String) {
        self.graph_id = value;
    }

    #[wasm_bindgen(getter = notes)]
    pub fn get_notes(&self) -> JsValue {
        match &self.notes {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = notes)]
    pub fn set_notes(&mut self, value: Option<String>) {
        self.notes = value;
    }

    #[wasm_bindgen(getter = publicationid)]
    pub fn get_publicationid(&self) -> String {
        self.publicationid.clone()
    }

    #[wasm_bindgen(setter = publicationid)]
    pub fn set_publicationid(&mut self, value: String) {
        self.publicationid = value;
    }

    #[wasm_bindgen(getter = published_time)]
    pub fn get_published_time(&self) -> String {
        self.published_time.clone()
    }

    #[wasm_bindgen(setter = published_time)]
    pub fn set_published_time(&mut self, value: String) {
        self.published_time = value;
    }
}

// StaticCardsXNodesXWidgets
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticCardsXNodesXWidgets {
    card_id: String,
    #[serde(default)]
    config: serde_json::Value,
    id: String,
    label: StaticTranslatableString,
    node_id: String,
    #[serde(default)]
    sortorder: Option<i32>,
    visible: bool,
    widget_id: String,
}

#[wasm_bindgen]
impl StaticCardsXNodesXWidgets {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticCardsXNodesXWidgets, JsValue> {
        let data: StaticCardsXNodesXWidgets = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;
        let mut obj = serde_json::Map::new();

        obj.insert("card_id".to_string(), json!(self.card_id));
        obj.insert("config".to_string(), self.config.clone());
        obj.insert("id".to_string(), json!(self.id));
        obj.insert("label".to_string(), json!(self.label));
        obj.insert("node_id".to_string(), json!(self.node_id));
        if let Some(val) = self.sortorder {
            obj.insert("sortorder".to_string(), json!(val));
        }
        obj.insert("visible".to_string(), json!(self.visible));
        obj.insert("widget_id".to_string(), json!(self.widget_id));

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = card_id)]
    pub fn get_card_id(&self) -> String {
        self.card_id.clone()
    }

    #[wasm_bindgen(setter = card_id)]
    pub fn set_card_id(&mut self, value: String) {
        self.card_id = value;
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        json_to_js_value(&self.config)
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config(&mut self, value: JsValue) {
        if let Ok(config) = serde_wasm_bindgen::from_value(value) {
            self.config = config;
        }
    }

    #[wasm_bindgen(getter = id)]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(setter = id)]
    pub fn set_id(&mut self, value: String) {
        self.id = value;
    }

    #[wasm_bindgen(getter = label)]
    pub fn get_label(&self) -> StaticTranslatableString {
        self.label.clone()
    }

    #[wasm_bindgen(setter = label)]
    pub fn set_label(&mut self, value: StaticTranslatableString) {
        self.label = value;
    }

    #[wasm_bindgen(getter = node_id)]
    pub fn get_node_id(&self) -> String {
        self.node_id.clone()
    }

    #[wasm_bindgen(setter = node_id)]
    pub fn set_node_id(&mut self, value: String) {
        self.node_id = value;
    }

    #[wasm_bindgen(getter = sortorder)]
    pub fn get_sortorder(&self) -> Option<i32> {
        self.sortorder
    }

    #[wasm_bindgen(setter = sortorder)]
    pub fn set_sortorder(&mut self, value: Option<i32>) {
        self.sortorder = value;
    }

    #[wasm_bindgen(getter = visible)]
    pub fn get_visible(&self) -> bool {
        self.visible
    }

    #[wasm_bindgen(setter = visible)]
    pub fn set_visible(&mut self, value: bool) {
        self.visible = value;
    }

    #[wasm_bindgen(getter = widget_id)]
    pub fn get_widget_id(&self) -> String {
        self.widget_id.clone()
    }

    #[wasm_bindgen(setter = widget_id)]
    pub fn set_widget_id(&mut self, value: String) {
        self.widget_id = value;
    }
}

// StaticFunctionsXGraphs
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticFunctionsXGraphs {
    #[serde(default)]
    config: serde_json::Value,
    function_id: String,
    graph_id: String,
    id: String,
}

#[wasm_bindgen]
impl StaticFunctionsXGraphs {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticFunctionsXGraphs, JsValue> {
        let data: StaticFunctionsXGraphs = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticFunctionsXGraphs {
        self.clone()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;
        let mut obj = serde_json::Map::new();

        obj.insert("config".to_string(), json!(self.config));
        obj.insert("function_id".to_string(), json!(self.function_id));
        obj.insert("graph_id".to_string(), json!(self.graph_id));
        obj.insert("id".to_string(), json!(self.id));

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        json_to_js_value(&self.config)
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config(&mut self, value: JsValue) {
        if let Ok(config) = serde_wasm_bindgen::from_value(value) {
            self.config = config;
        }
    }

    #[wasm_bindgen(getter = function_id)]
    pub fn get_function_id(&self) -> String {
        self.function_id.clone()
    }

    #[wasm_bindgen(setter = function_id)]
    pub fn set_function_id(&mut self, value: String) {
        self.function_id = value;
    }

    #[wasm_bindgen(getter = graph_id)]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(setter = graph_id)]
    pub fn set_graph_id(&mut self, value: String) {
        self.graph_id = value;
    }

    #[wasm_bindgen(getter = id)]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(setter = id)]
    pub fn set_id(&mut self, value: String) {
        self.id = value;
    }
}

// StaticCard
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticCard {
    active: bool,
    cardid: String,
    component_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<serde_json::Value>,
    #[serde(default)]
    constraints: Vec<StaticConstraint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cssclass: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string_or_object", default)]
    description: Option<String>,
    graph_id: String,
    helpenabled: bool,
    helptext: StaticTranslatableString,
    helptitle: StaticTranslatableString,
    instructions: StaticTranslatableString,
    is_editable: bool,
    name: StaticTranslatableString,
    nodegroup_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sortorder: Option<i32>,
    visible: bool,
}

#[wasm_bindgen]
impl StaticCard {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticCard, JsValue> {
        let data: StaticCard = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;
        let mut obj = serde_json::Map::new();

        obj.insert("active".to_string(), json!(self.active));
        obj.insert("cardid".to_string(), json!(self.cardid));
        obj.insert("component_id".to_string(), json!(self.component_id));
        if let Some(ref val) = self.config {
            obj.insert("config".to_string(), val.clone());
        }
        obj.insert("constraints".to_string(), json!(self.constraints));
        if let Some(ref val) = self.cssclass {
            obj.insert("cssclass".to_string(), json!(val));
        } else {
            obj.insert("cssclass".to_string(), json!(null));
        }
        if let Some(ref val) = self.description {
            obj.insert("description".to_string(), json!(val));
        } else {
            obj.insert("description".to_string(), json!(null));
        }
        obj.insert("graph_id".to_string(), json!(self.graph_id));
        obj.insert("helpenabled".to_string(), json!(self.helpenabled));
        obj.insert("helptext".to_string(), json!(self.helptext));
        obj.insert("helptitle".to_string(), json!(self.helptitle));
        obj.insert("instructions".to_string(), json!(self.instructions));
        obj.insert("is_editable".to_string(), json!(self.is_editable));
        obj.insert("name".to_string(), json!(self.name));
        obj.insert("nodegroup_id".to_string(), json!(self.nodegroup_id));
        if let Some(val) = self.sortorder {
            obj.insert("sortorder".to_string(), json!(val));
        } else {
            obj.insert("sortorder".to_string(), json!(null));
        }
        obj.insert("visible".to_string(), json!(self.visible));

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = active)]
    pub fn get_active(&self) -> bool {
        self.active
    }

    #[wasm_bindgen(setter = active)]
    pub fn set_active(&mut self, value: bool) {
        self.active = value;
    }

    #[wasm_bindgen(getter = cardid)]
    pub fn get_cardid(&self) -> String {
        self.cardid.clone()
    }

    #[wasm_bindgen(setter = cardid)]
    pub fn set_cardid(&mut self, value: String) {
        self.cardid = value;
    }

    #[wasm_bindgen(getter = component_id)]
    pub fn get_component_id(&self) -> String {
        self.component_id.clone()
    }

    #[wasm_bindgen(setter = component_id)]
    pub fn set_component_id(&mut self, value: String) {
        self.component_id = value;
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        self.config.as_ref().map(|c| json_to_js_value(c)).unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config(&mut self, value: JsValue) {
        if let Ok(config) = serde_wasm_bindgen::from_value(value) {
            self.config = Some(config);
        }
    }

    #[wasm_bindgen(getter = constraints)]
    pub fn get_constraints(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.constraints).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = constraints)]
    pub fn set_constraints(&mut self, value: JsValue) {
        if let Ok(constraints) = serde_wasm_bindgen::from_value(value) {
            self.constraints = constraints;
        }
    }

    #[wasm_bindgen(getter = cssclass)]
    pub fn get_cssclass(&self) -> Option<String> {
        self.cssclass.clone()
    }

    #[wasm_bindgen(setter = cssclass)]
    pub fn set_cssclass(&mut self, value: Option<String>) {
        self.cssclass = value;
    }

    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(setter = description)]
    pub fn set_description(&mut self, value: Option<String>) {
        self.description = value;
    }

    #[wasm_bindgen(getter = graph_id)]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(setter = graph_id)]
    pub fn set_graph_id(&mut self, value: String) {
        self.graph_id = value;
    }

    #[wasm_bindgen(getter = helpenabled)]
    pub fn get_helpenabled(&self) -> bool {
        self.helpenabled
    }

    #[wasm_bindgen(setter = helpenabled)]
    pub fn set_helpenabled(&mut self, value: bool) {
        self.helpenabled = value;
    }

    #[wasm_bindgen(getter = helptext)]
    pub fn get_helptext(&self) -> StaticTranslatableString {
        self.helptext.clone()
    }

    #[wasm_bindgen(setter = helptext)]
    pub fn set_helptext(&mut self, value: StaticTranslatableString) {
        self.helptext = value;
    }

    #[wasm_bindgen(getter = helptitle)]
    pub fn get_helptitle(&self) -> StaticTranslatableString {
        self.helptitle.clone()
    }

    #[wasm_bindgen(setter = helptitle)]
    pub fn set_helptitle(&mut self, value: StaticTranslatableString) {
        self.helptitle = value;
    }

    #[wasm_bindgen(getter = instructions)]
    pub fn get_instructions(&self) -> StaticTranslatableString {
        self.instructions.clone()
    }

    #[wasm_bindgen(setter = instructions)]
    pub fn set_instructions(&mut self, value: StaticTranslatableString) {
        self.instructions = value;
    }

    #[wasm_bindgen(getter = is_editable)]
    pub fn get_is_editable(&self) -> bool {
        self.is_editable
    }

    #[wasm_bindgen(setter = is_editable)]
    pub fn set_is_editable(&mut self, value: bool) {
        self.is_editable = value;
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> StaticTranslatableString {
        self.name.clone()
    }

    #[wasm_bindgen(setter = name)]
    pub fn set_name(&mut self, value: StaticTranslatableString) {
        self.name = value;
    }

    #[wasm_bindgen(getter = nodegroup_id)]
    pub fn get_nodegroup_id(&self) -> String {
        self.nodegroup_id.clone()
    }

    #[wasm_bindgen(setter = nodegroup_id)]
    pub fn set_nodegroup_id(&mut self, value: String) {
        self.nodegroup_id = value;
    }

    #[wasm_bindgen(getter = sortorder)]
    pub fn get_sortorder(&self) -> Option<i32> {
        self.sortorder
    }

    #[wasm_bindgen(setter = sortorder)]
    pub fn set_sortorder(&mut self, value: Option<i32>) {
        self.sortorder = value;
    }

    #[wasm_bindgen(getter = visible)]
    pub fn get_visible(&self) -> bool {
        self.visible
    }

    #[wasm_bindgen(setter = visible)]
    pub fn set_visible(&mut self, value: bool) {
        self.visible = value;
    }
}

// Custom deserializer for tile data - converts object to HashMap
fn deserialize_tile_data<'de, D>(deserializer: D) -> Result<HashMap<String, serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;

    match value {
        Value::Object(map) => {
            Ok(map.into_iter().collect())
        }
        Value::Null => Ok(HashMap::new()),
        _ => Ok(HashMap::new()),
    }
}

// StaticTile
#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticTile {
    #[serde(deserialize_with = "deserialize_tile_data", default)]
    pub(crate) data: HashMap<String, serde_json::Value>,
    pub(crate) nodegroup_id: String,
    pub(crate) resourceinstance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tileid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parenttile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provisionaledits: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sortorder: Option<i32>,
}

#[wasm_bindgen]
impl StaticTile {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticTile, JsValue> {
        let data: StaticTile = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen(js_name = ensureId)]
    pub fn ensure_id(&mut self) -> String {
        if self.tileid.is_none() {
            // Generate a UUID
            self.tileid = Some(uuid::Uuid::new_v4().to_string());
        }
        self.tileid.clone().unwrap()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;
        let mut obj = serde_json::Map::new();

        // Convert HashMap to plain object for data field
        let data_obj: serde_json::Map<String, serde_json::Value> =
            self.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        obj.insert("data".to_string(), json!(data_obj));

        obj.insert("nodegroup_id".to_string(), json!(self.nodegroup_id));
        obj.insert("resourceinstance_id".to_string(), json!(self.resourceinstance_id));

        if let Some(ref val) = self.tileid {
            obj.insert("tileid".to_string(), json!(val));
        } else {
            obj.insert("tileid".to_string(), json!(null));
        }

        if let Some(ref val) = self.parenttile_id {
            obj.insert("parenttile_id".to_string(), json!(val));
        } else {
            obj.insert("parenttile_id".to_string(), json!(null));
        }

        if let Some(ref val) = self.provisionaledits {
            obj.insert("provisionaledits".to_string(), json!(val));
        } else {
            obj.insert("provisionaledits".to_string(), json!(null));
        }

        if let Some(val) = self.sortorder {
            obj.insert("sortorder".to_string(), json!(val));
        } else {
            obj.insert("sortorder".to_string(), json!(null));
        }

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    #[wasm_bindgen(getter = data)]
    pub fn get_data(&self) -> JsValue {
        // Return as a JavaScript Map
        let js_map = js_sys::Map::new();
        for (key, val) in &self.data {
            js_map.set(&JsValue::from_str(key), &json_to_js_value(val));
        }
        js_map.into()
    }

    #[wasm_bindgen(setter = data)]
    pub fn set_data(&mut self, value: JsValue) {
        // Handle both Map and plain object
        if value.is_object() {
            // Check if it's a Map by checking for the forEach method
            if js_sys::Reflect::has(&value, &JsValue::from_str("forEach")).unwrap_or(false) {
                // It's a Map, iterate through it
                let map = js_sys::Map::from(value);
                let mut new_data = HashMap::new();
                map.for_each(&mut |val, key| {
                    if let Some(key_str) = key.as_string() {
                        if let Ok(json_val) = serde_wasm_bindgen::from_value(val) {
                            new_data.insert(key_str, json_val);
                        }
                    }
                });
                self.data = new_data;
            } else if let Ok(obj_data) = serde_wasm_bindgen::from_value::<HashMap<String, serde_json::Value>>(value) {
                self.data = obj_data;
            }
        }
    }

    #[wasm_bindgen(getter = nodegroup_id)]
    pub fn get_nodegroup_id(&self) -> String {
        self.nodegroup_id.clone()
    }

    #[wasm_bindgen(setter = nodegroup_id)]
    pub fn set_nodegroup_id(&mut self, value: String) {
        self.nodegroup_id = value;
    }

    #[wasm_bindgen(getter = resourceinstance_id)]
    pub fn get_resourceinstance_id(&self) -> String {
        self.resourceinstance_id.clone()
    }

    #[wasm_bindgen(setter = resourceinstance_id)]
    pub fn set_resourceinstance_id(&mut self, value: String) {
        self.resourceinstance_id = value;
    }

    #[wasm_bindgen(getter = tileid)]
    pub fn get_tileid(&self) -> JsValue {
        match &self.tileid {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = tileid)]
    pub fn set_tileid(&mut self, value: JsValue) {
        if value.is_null() {
            self.tileid = None;
        } else if let Some(s) = value.as_string() {
            self.tileid = Some(s);
        }
    }

    #[wasm_bindgen(getter = parenttile_id)]
    pub fn get_parenttile_id(&self) -> JsValue {
        match &self.parenttile_id {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = parenttile_id)]
    pub fn set_parenttile_id(&mut self, value: JsValue) {
        if value.is_null() {
            self.parenttile_id = None;
        } else if let Some(s) = value.as_string() {
            self.parenttile_id = Some(s);
        }
    }

    #[wasm_bindgen(getter = provisionaledits)]
    pub fn get_provisionaledits(&self) -> JsValue {
        match &self.provisionaledits {
            Some(val) => {
                // Convert Vec<serde_json::Value> to plain JS array with plain objects
                let js_array = js_sys::Array::new();
                for item in val {
                    js_array.push(&json_to_js_value(item));
                }
                js_array.into()
            },
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = provisionaledits)]
    pub fn set_provisionaledits(&mut self, value: JsValue) {
        if value.is_null() {
            self.provisionaledits = None;
        } else if let Ok(edits) = serde_wasm_bindgen::from_value(value) {
            self.provisionaledits = Some(edits);
        }
    }

    #[wasm_bindgen(getter = sortorder)]
    pub fn get_sortorder(&self) -> JsValue {
        match self.sortorder {
            Some(val) => JsValue::from_f64(val as f64),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = sortorder)]
    pub fn set_sortorder(&mut self, value: JsValue) {
        if value.is_null() {
            self.sortorder = None;
        } else if let Some(n) = value.as_f64() {
            self.sortorder = Some(n as i32);
        }
    }
}

// Non-WASM methods for internal Rust use
impl StaticTile {
    /// Create a new empty tile for a nodegroup (internal Rust constructor)
    /// Used by json_conversion module for tree_to_tiles
    pub(crate) fn new_empty(nodegroup_id: String) -> Self {
        StaticTile {
            tileid: Some(uuid::Uuid::new_v4().to_string()),
            nodegroup_id,
            parenttile_id: None,
            resourceinstance_id: String::new(),  // Empty string for now, will be set later
            sortorder: None,
            provisionaledits: None,
            data: std::collections::HashMap::new(),
        }
    }
}

impl StaticNode {
    // Helper function for deep comparison of JSON values
    fn deep_compare_values(val_a: &serde_json::Value, val_b: &serde_json::Value) -> bool {
        use serde_json::Value;

        match (val_a, val_b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(va, vb)| Self::deep_compare_values(va, vb))
            }
            (Value::Object(a), Value::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                let mut all_keys: std::collections::HashSet<&String> = a.keys().collect();
                all_keys.extend(b.keys());

                all_keys.iter().all(|key| {
                    match (a.get(*key), b.get(*key)) {
                        (Some(va), Some(vb)) => Self::deep_compare_values(va, vb),
                        (None, None) => true,
                        _ => false,
                    }
                })
            }
            _ => false,
        }
    }

    // Internal compare function that matches the JS behavior
    pub fn compare(node_a: &StaticNode, node_b: &StaticNode) -> serde_json::Value {
        use serde_json::json;

        // Check if they're the same object (comparing memory addresses)
        if std::ptr::eq(node_a, node_b) {
            return json!(true);
        }

        // Compare all fields except nodeid, graph_id, nodegroup_id
        // using deep comparison for config
        let mut identical = true;

        if node_a.alias != node_b.alias { identical = false; }

        // Deep compare config using JSON values
        if !Self::deep_compare_values(
            &serde_json::to_value(&node_a.config).unwrap_or(json!({})),
            &serde_json::to_value(&node_b.config).unwrap_or(json!({}))
        ) {
            identical = false;
        }

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

        // Now we know they're identical up to the IDs
        // Check for differences in IDs following the JS logic

        // Check graph_id mismatch (both non-empty and different)
        if !node_a.graph_id.is_empty() && !node_b.graph_id.is_empty() && node_a.graph_id != node_b.graph_id {
            return json!(-3);
        }

        // Check nodegroup_id mismatch (both non-None/non-empty and different)
        if let (Some(ref a_ng), Some(ref b_ng)) = (&node_a.nodegroup_id, &node_b.nodegroup_id) {
            if !a_ng.is_empty() && !b_ng.is_empty() && a_ng != b_ng {
                return json!(-2);
            }
        }

        // Check nodeid mismatch (both non-empty and different)
        if !node_a.nodeid.is_empty() && !node_b.nodeid.is_empty() && node_a.nodeid != node_b.nodeid {
            return json!(-1);
        }

        // Now determine if all IDs match
        // JS logic: (A && B) || (A === B)
        // This returns true if both are truthy OR if they're equal (including both empty)
        let graph_id_match =
            (!node_a.graph_id.is_empty() && !node_b.graph_id.is_empty()) ||
            node_a.graph_id == node_b.graph_id;

        let nodegroup_id_match = match (&node_a.nodegroup_id, &node_b.nodegroup_id) {
            (Some(a), Some(b)) => (!a.is_empty() && !b.is_empty()) || a == b,
            (None, None) => true,
            _ => false,
        };

        let nodeid_match =
            (!node_a.nodeid.is_empty() && !node_b.nodeid.is_empty()) ||
            node_a.nodeid == node_b.nodeid;

        // If all IDs match (either both truthy or equal), return 2
        // Otherwise return 1
        if graph_id_match && nodegroup_id_match && nodeid_match {
            json!(2)
        } else {
            json!(1)
        }
    }

    // Version of compare that works with JSON values (handles partial objects)
    pub fn compare_values(val_a: &serde_json::Value, val_b: &serde_json::Value) -> serde_json::Value {
        use serde_json::json;

        // Helper to get a field as a string, returning empty string if not present
        fn get_string(obj: &serde_json::Value, key: &str) -> String {
            obj.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
        }

        // Helper to check if a value is falsey (null, false, empty string, etc.)
        fn is_falsey(val: &serde_json::Value) -> bool {
            val.is_null() || val.as_str().map(|s| s.is_empty()).unwrap_or(false)
        }

        // List of keys to compare (excluding IDs)
        let exclude_keys = ["nodeid", "graph_id", "nodegroup_id", "compare", "copy"];

        // Get all keys from both objects
        let mut all_keys = std::collections::HashSet::new();
        if let Some(obj_a) = val_a.as_object() {
            all_keys.extend(obj_a.keys().filter(|k| !exclude_keys.contains(&k.as_str())));
        }
        if let Some(obj_b) = val_b.as_object() {
            all_keys.extend(obj_b.keys().filter(|k| !exclude_keys.contains(&k.as_str())));
        }

        // Compare all non-ID fields
        for key in &all_keys {
            let val_a_field = val_a.get(key.as_str());
            let val_b_field = val_b.get(key.as_str());

            // Special handling for config: treat null as equivalent to {}
            if key.as_str() == "config" {
                let is_empty_or_null = |v: Option<&serde_json::Value>| {
                    match v {
                        None | Some(serde_json::Value::Null) => true,
                        Some(serde_json::Value::Object(obj)) if obj.is_empty() => true,
                        _ => false
                    }
                };

                if is_empty_or_null(val_a_field) && is_empty_or_null(val_b_field) {
                    continue;
                }
            }

            match (val_a_field, val_b_field) {
                (Some(a), Some(b)) => {
                    if !Self::deep_compare_values(a, b) {
                        return json!(false);
                    }
                }
                (None, None) => continue,
                // Treat None and Null as equivalent
                (None, Some(serde_json::Value::Null)) | (Some(serde_json::Value::Null), None) => continue,
                // One has a value, the other doesn't
                _ => return json!(false),
            }
        }

        // Now check IDs for differences
        let graph_id_a = get_string(val_a, "graph_id");
        let graph_id_b = get_string(val_b, "graph_id");
        let nodegroup_id_a = val_a.get("nodegroup_id");
        let nodegroup_id_b = val_b.get("nodegroup_id");
        let nodeid_a = get_string(val_a, "nodeid");
        let nodeid_b = get_string(val_b, "nodeid");

        // Check for ID mismatches (both non-empty and different)
        if !graph_id_a.is_empty() && !graph_id_b.is_empty() && graph_id_a != graph_id_b {
            return json!(-3);
        }

        if let (Some(ng_a), Some(ng_b)) = (nodegroup_id_a, nodegroup_id_b) {
            if !is_falsey(ng_a) && !is_falsey(ng_b) {
                let ng_a_str = ng_a.as_str().unwrap_or("");
                let ng_b_str = ng_b.as_str().unwrap_or("");
                if !ng_a_str.is_empty() && !ng_b_str.is_empty() && ng_a_str != ng_b_str {
                    return json!(-2);
                }
            }
        }

        if !nodeid_a.is_empty() && !nodeid_b.is_empty() && nodeid_a != nodeid_b {
            return json!(-1);
        }

        // Check if all IDs match (using JS logic: (A && B) || (A === B))
        let graph_id_match =
            (!graph_id_a.is_empty() && !graph_id_b.is_empty()) ||
            graph_id_a == graph_id_b;

        let nodegroup_id_match = match (nodegroup_id_a, nodegroup_id_b) {
            (Some(a), Some(b)) if !is_falsey(a) && !is_falsey(b) => {
                let a_str = a.as_str().unwrap_or("");
                let b_str = b.as_str().unwrap_or("");
                (!a_str.is_empty() && !b_str.is_empty()) || a_str == b_str
            },
            (Some(a), Some(b)) => is_falsey(a) && is_falsey(b) || a == b,
            (None, None) => true,
            _ => false,
        };

        let nodeid_match =
            (!nodeid_a.is_empty() && !nodeid_b.is_empty()) ||
            nodeid_a == nodeid_b;

        if graph_id_match && nodegroup_id_match && nodeid_match {
            json!(2)
        } else {
            json!(1)
        }
    }
}

// StaticEdge - Represents an edge in the graph
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticEdge {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    pub(crate) domainnode_id: String,
    edgeid: String,
    graph_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ontologyproperty: Option<String>,
    pub(crate) rangenode_id: String,
}

#[wasm_bindgen]
impl StaticEdge {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticEdge, JsValue> {
        let data: StaticEdge = serde_wasm_bindgen::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticEdge {
        self.clone()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        use serde_json::json;

        let mut obj = serde_json::Map::new();

        if let Some(ref val) = self.description {
            obj.insert("description".to_string(), json!(val));
        }
        obj.insert("domainnode_id".to_string(), json!(self.domainnode_id));
        obj.insert("edgeid".to_string(), json!(self.edgeid));
        obj.insert("graph_id".to_string(), json!(self.graph_id));
        if let Some(ref val) = self.name {
            obj.insert("name".to_string(), json!(val));
        }
        if let Some(ref val) = self.ontologyproperty {
            obj.insert("ontologyproperty".to_string(), json!(val));
        }
        obj.insert("rangenode_id".to_string(), json!(self.rangenode_id));

        json_to_js_value(&serde_json::Value::Object(obj))
    }

    // Getters
    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(getter = domainnode_id)]
    pub fn get_domainnode_id(&self) -> String {
        self.domainnode_id.clone()
    }

    #[wasm_bindgen(getter = edgeid)]
    pub fn get_edgeid(&self) -> String {
        self.edgeid.clone()
    }

    #[wasm_bindgen(getter = graph_id)]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    #[wasm_bindgen(getter = ontologyproperty)]
    pub fn get_ontologyproperty(&self) -> Option<String> {
        self.ontologyproperty.clone()
    }

    #[wasm_bindgen(getter = rangenode_id)]
    pub fn get_rangenode_id(&self) -> String {
        self.rangenode_id.clone()
    }
}

// StaticGraph - The main graph structure
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticGraph {
    author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cards: Option<Vec<StaticCard>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cards_x_nodes_x_widgets: Option<Vec<StaticCardsXNodesXWidgets>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(default)]
    config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    deploymentdate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deploymentfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<StaticTranslatableString>,
    pub(crate) edges: Vec<StaticEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    functions_x_graphs: Option<Vec<StaticFunctionsXGraphs>>,
    graphid: String,
    iconclass: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_editable: Option<bool>,
    isresource: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    jsonldcontext: Option<String>,
    name: StaticTranslatableString,
    pub(crate) nodegroups: Vec<StaticNodegroup>,
    pub(crate) nodes: Vec<StaticNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ontology_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publication: Option<StaticPublication>,
    #[serde(default)]
    relatable_resource_model_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_2_resource_constraints: Option<Vec<serde_json::Value>>,
    root: StaticNode,
    #[serde(skip_serializing_if = "Option::is_none")]
    slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<StaticTranslatableString>,
    template_id: String,
    version: String,

    // Internal lookup tables (not serialized)
    #[serde(skip)]
    #[wasm_bindgen(skip)]
    node_by_id: Option<HashMap<String, usize>>,
    #[serde(skip)]
    #[wasm_bindgen(skip)]
    node_by_alias: Option<HashMap<String, usize>>,
}

#[wasm_bindgen]
impl StaticGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticGraph, JsValue> {
        // Log the incoming data for debugging
        web_sys::console::log_1(&"Deserializing StaticGraph with data:".into());
        web_sys::console::log_1(&json_data);

        let mut data: StaticGraph = serde_wasm_bindgen::from_value(json_data).map_err(|e| {
            let error_string = e.to_string();

            // Log detailed error information
            web_sys::console::error_1(&"=== StaticGraph Deserialization Error ===".into());
            web_sys::console::error_1(&format!("Error: {}", error_string).into());
            web_sys::console::error_1(&format!("Debug: {:?}", e).into());

            // Try to extract field name from error message
            if error_string.contains("missing field") {
                web_sys::console::error_1(&"^ This is a MISSING FIELD error".into());
            } else if error_string.contains("invalid type") {
                web_sys::console::error_1(&"^ This is an INVALID TYPE error".into());
            }

            JsValue::from_str(&format!("StaticGraph deserialization failed: {}", error_string))
        })?;
        data.build_indices();
        Ok(data)
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticGraph {
        let mut cloned = self.clone();
        cloned.build_indices();
        cloned
    }

    /// Parse JSON string with {graph: [StaticGraph]} structure
    /// Returns the first graph from the array, or error if none found
    #[wasm_bindgen(js_name = fromJsonString)]
    pub fn from_json_string(json_str: &str) -> Result<StaticGraph, JsValue> {
        // Helper struct to deserialize the wrapper
        #[derive(Deserialize)]
        struct GraphWrapper {
            graph: Vec<StaticGraph>,
        }

        let wrapper: GraphWrapper = serde_json::from_str(json_str)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

        let mut graph = wrapper.graph
            .into_iter()
            .next()
            .ok_or_else(|| JsValue::from_str("No graphs found in JSON"))?;

        graph.build_indices();
        Ok(graph)
    }

    // Getters for key fields
    #[wasm_bindgen(getter = graphid)]
    pub fn get_graphid(&self) -> String {
        self.graphid.clone()
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> StaticTranslatableString {
        self.name.clone()
    }

    #[wasm_bindgen(getter = description)]
    pub fn get_description(&self) -> StaticTranslatableString {
        match self.description {
            Some(ref description) => description.clone(),
            _ => StaticTranslatableString {
                translations: HashMap::new(),
                lang: String::new()
            }
        }
    }

    #[wasm_bindgen(getter = subtitle)]
    pub fn get_subtitle(&self) -> StaticTranslatableString {
        match self.subtitle {
            Some(ref subtitle) => subtitle.clone(),
            _ => StaticTranslatableString {
                translations: HashMap::new(),
                lang: String::new()
            }
        }
    }

    #[wasm_bindgen(getter = nodes)]
    pub fn get_nodes(&self) -> js_sys::Array {
        // Return an array of StaticNode WASM instances, not plain JS objects
        // This preserves the getters/setters on each node
        let array = js_sys::Array::new();
        for node in &self.nodes {
            array.push(&JsValue::from(node.clone()));
        }
        array
    }

    #[wasm_bindgen(setter = nodes)]
    pub fn set_nodes(&mut self, value: JsValue) {
        if let Ok(nodes) = serde_wasm_bindgen::from_value(value) {
            self.nodes = nodes;
            self.build_indices();
        }
    }

    #[wasm_bindgen(getter = edges)]
    pub fn get_edges(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for edge in &self.edges {
            array.push(&JsValue::from(edge.clone()));
        }
        array
    }

    #[wasm_bindgen(setter = edges)]
    pub fn set_edges(&mut self, value: JsValue) {
        if let Ok(edges) = serde_wasm_bindgen::from_value(value) {
            self.edges = edges;
        }
    }

    #[wasm_bindgen(getter = nodegroups)]
    pub fn get_nodegroups(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for nodegroup in &self.nodegroups {
            array.push(&JsValue::from(nodegroup.clone()));
        }
        array
    }

    #[wasm_bindgen(setter = nodegroups)]
    pub fn set_nodegroups(&mut self, value: JsValue) {
        if let Ok(nodegroups) = serde_wasm_bindgen::from_value(value) {
            self.nodegroups = nodegroups;
        }
    }

    #[wasm_bindgen(getter = cards)]
    pub fn get_cards(&self) -> JsValue {
        match &self.cards {
            Some(cards) => {
                let array = js_sys::Array::new();
                for card in cards {
                    array.push(&JsValue::from(card.clone()));
                }
                JsValue::from(array)
            }
            None => JsValue::NULL
        }
    }

    #[wasm_bindgen(setter = cards)]
    pub fn set_cards(&mut self, value: JsValue) {
        if let Ok(cards) = serde_wasm_bindgen::from_value(value) {
            self.cards = cards;
        }
    }

    #[wasm_bindgen(getter = cards_x_nodes_x_widgets)]
    pub fn get_cards_x_nodes_x_widgets(&self) -> JsValue {
        match &self.cards_x_nodes_x_widgets {
            Some(cxnxws) => {
                let array = js_sys::Array::new();
                for cxnxw in cxnxws {
                    array.push(&JsValue::from(cxnxw.clone()));
                }
                JsValue::from(array)
            }
            None => JsValue::NULL
        }
    }

    #[wasm_bindgen(setter = cards_x_nodes_x_widgets)]
    pub fn set_cards_x_nodes_x_widgets(&mut self, value: JsValue) {
        if let Ok(cards_x_nodes_x_widgets) = serde_wasm_bindgen::from_value(value) {
            self.cards_x_nodes_x_widgets = cards_x_nodes_x_widgets;
        }
    }

    #[wasm_bindgen(getter = root)]
    pub fn get_root(&self) -> StaticNode {
        self.root.clone()
    }

    #[wasm_bindgen(getter = isresource)]
    pub fn get_isresource(&self) -> bool {
        self.isresource
    }

    #[wasm_bindgen(getter = author)]
    pub fn get_author(&self) -> String {
        self.author.clone()
    }

    #[wasm_bindgen(getter = iconclass)]
    pub fn get_iconclass(&self) -> String {
        self.iconclass.clone()
    }

    #[wasm_bindgen(getter = template_id)]
    pub fn get_template_id(&self) -> String {
        self.template_id.clone()
    }

    #[wasm_bindgen(getter = version)]
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    // Helper methods for modifying collections
    #[wasm_bindgen(js_name = pushNode)]
    pub fn push_node(&mut self, node: StaticNode) {
        self.nodes.push(node);
        self.build_indices();
    }

    #[wasm_bindgen(js_name = pushEdge)]
    pub fn push_edge(&mut self, edge: StaticEdge) {
        self.edges.push(edge);
    }

    #[wasm_bindgen(js_name = pushNodegroup)]
    pub fn push_nodegroup(&mut self, nodegroup: StaticNodegroup) {
        self.nodegroups.push(nodegroup);
    }

    #[wasm_bindgen(js_name = pushCard)]
    pub fn push_card(&mut self, card: StaticCard) {
        if self.cards.is_none() {
            self.cards = Some(Vec::new());
        }
        if let Some(ref mut cards) = self.cards {
            cards.push(card);
        }
    }

    #[wasm_bindgen(js_name = pushCardXNodeXWidget)]
    pub fn push_card_x_node_x_widget(&mut self, cxnxw: StaticCardsXNodesXWidgets) {
        if self.cards_x_nodes_x_widgets.is_none() {
            self.cards_x_nodes_x_widgets = Some(Vec::new());
        }
        if let Some(ref mut cxnxws) = self.cards_x_nodes_x_widgets {
            cxnxws.push(cxnxw);
        }
    }

    // JS-facing lookup methods
    #[wasm_bindgen(js_name = getNodeById)]
    pub fn get_node_by_id_js(&self, id: &str) -> JsValue {
        self.get_node_by_id(id)
            .map(|node| serde_wasm_bindgen::to_value(node).unwrap())
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = getNodeByAlias)]
    pub fn get_node_by_alias_js(&self, alias: &str) -> JsValue {
        self.get_node_by_alias(alias)
            .map(|node| serde_wasm_bindgen::to_value(node).unwrap())
            .unwrap_or(JsValue::NULL)
    }
}

// Rust-internal methods (not exposed to JS)
impl StaticGraph {
    fn build_indices(&mut self) {
        let mut node_by_id = HashMap::new();
        let mut node_by_alias = HashMap::new();

        for (idx, node) in self.nodes.iter().enumerate() {
            node_by_id.insert(node.get_nodeid(), idx);
            if let Some(alias) = node.get_alias() {
                node_by_alias.insert(alias, idx);
            }
        }

        self.node_by_id = Some(node_by_id);
        self.node_by_alias = Some(node_by_alias);
    }

    pub(crate) fn get_node_by_index(&self, idx: usize) -> Option<&StaticNode> {
        self.nodes.get(idx)
    }

    pub(crate) fn get_node_by_id(&self, id: &str) -> Option<&StaticNode> {
        self.node_by_id
            .as_ref()?
            .get(id)
            .and_then(|&idx| self.nodes.get(idx))
    }

    pub(crate) fn get_node_by_alias(&self, alias: &str) -> Option<&StaticNode> {
        self.node_by_alias
            .as_ref()?
            .get(alias)
            .and_then(|&idx| self.nodes.get(idx))
    }

    // Accessors for json_conversion module
    pub(crate) fn nodes_slice(&self) -> &[StaticNode] {
        &self.nodes
    }

    pub(crate) fn nodegroups_slice(&self) -> &[StaticNodegroup] {
        &self.nodegroups
    }

    pub(crate) fn edges_slice(&self) -> &[StaticEdge] {
        &self.edges
    }

    pub(crate) fn root_node(&self) -> &StaticNode {
        &self.root
    }

    pub(crate) fn graph_id(&self) -> &str {
        &self.graphid
    }
}

// WKRM - Well-Known Resource Model
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WKRM {
    model_name: String,
    model_class_name: String,
    graph_id: String,
    meta: JsValue, // Store as JsValue to preserve object identity
}

#[wasm_bindgen]
impl WKRM {
    #[wasm_bindgen(constructor)]
    pub fn new(meta: JsValue) -> Result<WKRM, JsValue> {
        // Get the graphid
        let graph_id = js_sys::Reflect::get(&meta, &JsValue::from_str("graphid"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        // Get the name
        let name_val = js_sys::Reflect::get(&meta, &JsValue::from_str("name"))
            .ok()
            .unwrap_or(JsValue::NULL);

        let model_name = if !name_val.is_null() && !name_val.is_undefined() {
            // Try to get as string (could be a StaticTranslatableString with toString method)
            if let Some(name_str) = name_val.as_string() {
                name_str
            } else if let Ok(to_string_fn) = js_sys::Reflect::get(&name_val, &JsValue::from_str("toString")) {
                if let Ok(func) = to_string_fn.dyn_into::<js_sys::Function>() {
                    if let Ok(result) = func.call0(&name_val) {
                        result.as_string().unwrap_or_else(|| "Unnamed".to_string())
                    } else {
                        "Unnamed".to_string()
                    }
                } else {
                    "Unnamed".to_string()
                }
            } else {
                "Unnamed".to_string()
            }
        } else {
            "Unnamed".to_string()
        };

        // Get the slug
        let slug_val = js_sys::Reflect::get(&meta, &JsValue::from_str("slug"))
            .ok()
            .and_then(|v| v.as_string());

        // Convert slug or model_name to PascalCase for model_class_name
        let base_name = slug_val.as_ref().unwrap_or(&model_name);
        let model_class_name = Self::to_pascal_case(base_name);

        Ok(WKRM {
            model_name,
            model_class_name,
            graph_id,
            meta,
        })
    }

    // Convert a string with underscores, hyphens, or spaces to PascalCase
    fn to_pascal_case(s: &str) -> String {
        // Replace underscores and hyphens with spaces
        let normalized = s.replace('_', " ").replace('-', " ");

        // Split by whitespace and capitalize each word
        let pascal: String = normalized
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        // Capitalize first character, keep rest as-is (don't lowercase)
                        first.to_uppercase().collect::<String>() + chars.as_str()
                    }
                    None => String::new(),
                }
            })
            .collect();

        // Ensure first character is uppercase
        if pascal.is_empty() {
            pascal
        } else {
            let mut chars = pascal.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }

    #[wasm_bindgen(getter = modelName)]
    pub fn get_model_name(&self) -> String {
        self.model_name.clone()
    }

    #[wasm_bindgen(setter = modelName)]
    pub fn set_model_name(&mut self, value: String) {
        self.model_name = value;
    }

    #[wasm_bindgen(getter = modelClassName)]
    pub fn get_model_class_name(&self) -> String {
        self.model_class_name.clone()
    }

    #[wasm_bindgen(setter = modelClassName)]
    pub fn set_model_class_name(&mut self, value: String) {
        self.model_class_name = value;
    }

    #[wasm_bindgen(getter = graphId)]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(setter = graphId)]
    pub fn set_graph_id(&mut self, value: String) {
        self.graph_id = value;
    }

    #[wasm_bindgen(getter = meta)]
    pub fn get_meta(&self) -> JsValue {
        self.meta.clone()
    }

    #[wasm_bindgen(setter = meta)]
    pub fn set_meta(&mut self, value: JsValue) {
        self.meta = value;
    }
}

// StaticResourceDescriptors - Descriptors for resource display
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceDescriptors {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) map_popup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
}

#[wasm_bindgen]
impl StaticResourceDescriptors {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Result<StaticResourceDescriptors, JsValue> {
        serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize StaticResourceDescriptors: {:?}", e)))
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.map_popup.is_none() && self.description.is_none()
    }

    #[wasm_bindgen(js_name = empty)]
    pub fn empty() -> StaticResourceDescriptors {
        StaticResourceDescriptors {
            name: None,
            map_popup: None,
            description: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, value: Option<String>) {
        self.name = value;
    }

    #[wasm_bindgen(getter = mapPopup)]
    pub fn map_popup(&self) -> Option<String> {
        self.map_popup.clone()
    }

    #[wasm_bindgen(setter = mapPopup)]
    pub fn set_map_popup(&mut self, value: Option<String>) {
        self.map_popup = value;
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_description(&mut self, value: Option<String>) {
        self.description = value;
    }
}

// StaticResourceMetadata - Metadata about a resource instance
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceMetadata {
    pub(crate) descriptors: StaticResourceDescriptors,
    pub(crate) graph_id: String,
    pub(crate) name: String,
    pub(crate) resourceinstanceid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) graph_publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lastmodified: Option<String>,
}

#[wasm_bindgen]
impl StaticResourceMetadata {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Result<StaticResourceMetadata, JsValue> {
        serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize StaticResourceMetadata: {:?}", e)))
    }

    #[wasm_bindgen(getter)]
    pub fn descriptors(&self) -> StaticResourceDescriptors {
        self.descriptors.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter = resourceinstanceid)]
    pub fn resourceinstanceid(&self) -> String {
        self.resourceinstanceid.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn publication_id(&self) -> Option<String> {
        self.publication_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn principaluser_id(&self) -> Option<i32> {
        self.principaluser_id
    }

    #[wasm_bindgen(getter)]
    pub fn legacyid(&self) -> Option<String> {
        self.legacyid.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn graph_publication_id(&self) -> Option<String> {
        self.graph_publication_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn createdtime(&self) -> Option<String> {
        self.createdtime.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn lastmodified(&self) -> Option<String> {
        self.lastmodified.clone()
    }
}

// StaticResourceSummary - Summary info for a resource (used for lazy loading)
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceSummary {
    pub(crate) resourceinstanceid: String,
    pub(crate) graph_id: String,
    pub(crate) name: String,
    pub(crate) descriptors: StaticResourceDescriptors,
    #[serde(default)]
    pub(crate) metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lastmodified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) graph_publication_id: Option<String>,
}

#[wasm_bindgen]
impl StaticResourceSummary {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Result<StaticResourceSummary, JsValue> {
        let mut summary: StaticResourceSummary = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize StaticResourceSummary: {:?}", e)))?;

        // Default name to '<Unnamed>' if empty
        if summary.name.is_empty() {
            summary.name = "<Unnamed>".to_string();
        }

        Ok(summary)
    }

    #[wasm_bindgen(js_name = toMetadata)]
    pub fn to_metadata(&self) -> StaticResourceMetadata {
        StaticResourceMetadata {
            descriptors: self.descriptors.clone(),
            graph_id: self.graph_id.clone(),
            name: self.name.clone(),
            resourceinstanceid: self.resourceinstanceid.clone(),
            publication_id: self.publication_id.clone(),
            principaluser_id: self.principaluser_id,
            legacyid: self.legacyid.clone(),
            graph_publication_id: self.graph_publication_id.clone(),
            createdtime: self.createdtime.clone(),
            lastmodified: self.lastmodified.clone(),
        }
    }

    #[wasm_bindgen(getter = resourceinstanceid)]
    pub fn resourceinstanceid(&self) -> String {
        self.resourceinstanceid.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn graph_id(&self) -> String {
        self.graph_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn descriptors(&self) -> StaticResourceDescriptors {
        self.descriptors.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.metadata).unwrap_or(JsValue::NULL)
    }
}

// StaticResource - Complete resource data with tiles
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResource {
    pub(crate) resourceinstance: StaticResourceMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tiles: Option<Vec<StaticTile>>,
    #[serde(default)]
    pub(crate) metadata: HashMap<String, String>,

    // Optional JS-managed cache and scopes
    #[serde(skip)]
    pub(crate) __cache: Option<JsValue>,
    #[serde(skip)]
    pub(crate) __scopes: Option<JsValue>,

    // Private fields for caching/tracking
    #[serde(skip)]
    #[wasm_bindgen(skip)]
    __tiles_loaded: bool,
}

#[wasm_bindgen]
impl StaticResource {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Result<StaticResource, JsValue> {
        let mut resource: StaticResource = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize StaticResource: {:?}", e)))?;

            web_sys::console::error_1(&JsValue::from_str(&format!("{:?}", resource.resourceinstance.graph_id)));
        // Set tiles loaded flag based on whether tiles exist and are non-empty
        resource.__tiles_loaded = resource.tiles.as_ref().map(|t| !t.is_empty()).unwrap_or(false);

        Ok(resource)
    }

    #[wasm_bindgen(js_name = fromSummary)]
    pub fn from_summary(summary: StaticResourceSummary) -> StaticResource {
        StaticResource {
            resourceinstance: summary.to_metadata(),
            tiles: Some(Vec::new()),
            metadata: summary.metadata,
            __cache: None,
            __scopes: None,
            __tiles_loaded: false,
        }
    }

    #[wasm_bindgen(getter = resourceinstance)]
    pub fn resourceinstance(&self) -> StaticResourceMetadata {
        self.resourceinstance.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn tiles(&self) -> JsValue {
        match &self.tiles {
            Some(tiles) => serde_wasm_bindgen::to_value(tiles).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_tiles(&mut self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            self.tiles = None;
            self.__tiles_loaded = false;
        } else {
            match serde_wasm_bindgen::from_value(value) {
                Ok(tiles) => {
                    self.__tiles_loaded = true;
                    self.tiles = Some(tiles);
                }
                Err(_) => {
                    self.tiles = None;
                    self.__tiles_loaded = false;
                }
            }
        }
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.metadata).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter)]
    pub fn set_metadata(&mut self, value: JsValue) {
        if let Ok(metadata) = serde_wasm_bindgen::from_value(value) {
            self.metadata = metadata;
        }
    }

    #[wasm_bindgen(getter = __cache)]
    pub fn get_cache(&self) -> JsValue {
        self.__cache.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = __cache)]
    pub fn set_cache(&mut self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            self.__cache = None;
        } else {
            self.__cache = Some(value);
        }
    }

    #[wasm_bindgen(getter = __scopes)]
    pub fn get_scopes(&self) -> JsValue {
        self.__scopes.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = __scopes)]
    pub fn set_scopes(&mut self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            self.__scopes = None;
        } else {
            self.__scopes = Some(value);
        }
    }

    #[wasm_bindgen(getter = tilesLoaded)]
    pub fn tiles_loaded(&self) -> bool {
        self.__tiles_loaded
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        // Serialize using serde_json, then convert to plain JS object
        // This automatically handles nested structs
        match serde_json::to_value(self) {
            Ok(json_value) => json_to_js_value(&json_value),
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to serialize StaticResource: {}", e).into());
                JsValue::NULL
            }
        }
    }
}
