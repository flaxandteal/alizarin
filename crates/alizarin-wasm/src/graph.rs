use std::collections::{BTreeMap, HashMap};
use wasm_bindgen::prelude::*;
use wasm_wrapper_derive::wasm_wrapper;

// Import core types
use alizarin_core::StaticCard as CoreStaticCard;
use alizarin_core::StaticCardsXNodesXWidgets as CoreStaticCardsXNodesXWidgets;
use alizarin_core::StaticConstraint as CoreStaticConstraint;
use alizarin_core::StaticEdge as CoreStaticEdge;
use alizarin_core::StaticFunctionsXGraphs as CoreStaticFunctionsXGraphs;
use alizarin_core::StaticNodegroup as CoreStaticNodegroup;
use alizarin_core::StaticPublication as CoreStaticPublication;
use alizarin_core::StaticResource as CoreStaticResource;
use alizarin_core::StaticResourceReference as CoreStaticResourceReference;
use alizarin_core::StaticResourceRegistry as CoreStaticResourceRegistry;
use alizarin_core::StaticResourceSummary as CoreStaticResourceSummary;
use alizarin_core::StaticTile as CoreStaticTile;
use alizarin_core::StaticTranslatableString as CoreTranslatableString;

// ============================================================================
// StaticTranslatableString - WASM wrapper around core type
// ============================================================================

// Generate the base wrapper with Deref, From, Serialize, Deserialize, Default
// but skip auto-generated constructor/toJSON/copy since we have custom ones
wasm_wrapper! {
    pub struct StaticTranslatableString wraps alizarin_core::StaticTranslatableString {
        no_constructor,
        no_copy,
        impl_default,
    }
}

// Custom WASM methods for StaticTranslatableString
#[wasm_bindgen]
impl StaticTranslatableString {
    #[wasm_bindgen(constructor)]
    pub fn new(value: JsValue, lang: Option<String>) -> StaticTranslatableString {
        let default_lang = lang.unwrap_or_else(|| "en".to_string());

        // Try to parse as object (translations map)
        if let Ok(translations) =
            crate::js_json::from_value::<HashMap<String, String>>(value.clone())
        {
            return StaticTranslatableString(CoreTranslatableString::from_translations(
                translations,
                Some(default_lang),
            ));
        }

        // Try as string
        if let Some(s) = value.as_string() {
            let mut translations = HashMap::new();
            translations.insert(default_lang.clone(), s);
            return StaticTranslatableString(CoreTranslatableString::from_translations(
                translations,
                Some(default_lang),
            ));
        }

        // Empty string fallback
        StaticTranslatableString(CoreTranslatableString::from_string(""))
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        self.0.to_string_default()
    }

    #[wasm_bindgen(js_name = copy)]
    pub fn copy(&self) -> StaticTranslatableString {
        self.clone()
    }

    #[wasm_bindgen(getter = lang)]
    pub fn get_lang(&self) -> String {
        self.0.lang.clone()
    }

    #[wasm_bindgen(getter = translations)]
    pub fn get_translations(&self) -> JsValue {
        let js_obj = js_sys::Object::new();
        for (key, val) in &self.0.translations {
            js_sys::Reflect::set(&js_obj, &JsValue::from_str(key), &JsValue::from_str(val)).ok();
        }
        js_obj.into()
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
        }
        Value::String(s) => JsValue::from_str(s),
        Value::Array(arr) => {
            let js_array = js_sys::Array::new();
            for item in arr {
                js_array.push(&json_to_js_value(item));
            }
            js_array.into()
        }
        Value::Object(obj) => {
            let js_obj = js_sys::Object::new();
            for (key, val) in obj {
                js_sys::Reflect::set(&js_obj, &JsValue::from_str(key), &json_to_js_value(val)).ok();
            }
            js_obj.into()
        }
    }
}

// ============================================================================
// StaticGraphMeta - WASM wrapper around core type
// ============================================================================

// Generate the base wrapper with Deref, From, Serialize, Deserialize
// Custom methods needed for typed returns (StaticTranslatableString, StaticNode wrappers)
wasm_wrapper! {
    pub struct StaticGraphMeta wraps alizarin_core::StaticGraphMeta {
        no_constructor,
        no_copy,
    }
}

#[wasm_bindgen]
impl StaticGraphMeta {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticGraphMeta, JsValue> {
        let data: StaticGraphMeta = crate::js_json::from_value(json_data).map_err(|e| {
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

    #[wasm_bindgen(getter = author, unchecked_return_type = "string | null")]
    pub fn get_author_property(&self) -> JsValue {
        crate::js_json::or_null(self.author.clone())
    }

    #[wasm_bindgen(js_name = getAuthor, unchecked_return_type = "string | null")]
    pub fn get_author(&self) -> JsValue {
        crate::js_json::or_null(self.author.clone())
    }

    #[wasm_bindgen(js_name = setAuthor)]
    pub fn set_author(&mut self, value: Option<String>) {
        self.author = value;
    }

    #[wasm_bindgen(getter = isresource, unchecked_return_type = "boolean | null")]
    pub fn get_isresource_property(&self) -> JsValue {
        crate::js_json::or_null(self.isresource)
    }

    #[wasm_bindgen(js_name = getIsResource, unchecked_return_type = "boolean | null")]
    pub fn get_isresource(&self) -> JsValue {
        crate::js_json::or_null(self.isresource)
    }

    #[wasm_bindgen(js_name = setIsResource)]
    pub fn set_isresource(&mut self, value: Option<bool>) {
        self.isresource = value;
    }

    // Additional getters/setters for all fields
    #[wasm_bindgen(js_name = getCards, unchecked_return_type = "number | null")]
    pub fn get_cards(&self) -> JsValue {
        crate::js_json::or_null(self.cards)
    }

    #[wasm_bindgen(js_name = setCards)]
    pub fn set_cards(&mut self, value: Option<u32>) {
        self.cards = value;
    }

    #[wasm_bindgen(js_name = getCardsXNodesXWidgets, unchecked_return_type = "number | null")]
    pub fn get_cards_x_nodes_x_widgets(&self) -> JsValue {
        crate::js_json::or_null(self.cards_x_nodes_x_widgets)
    }

    #[wasm_bindgen(js_name = setCardsXNodesXWidgets)]
    pub fn set_cards_x_nodes_x_widgets(&mut self, value: Option<u32>) {
        self.cards_x_nodes_x_widgets = value;
    }

    #[wasm_bindgen(js_name = getColor, unchecked_return_type = "string | null")]
    pub fn get_color(&self) -> JsValue {
        crate::js_json::or_null(self.color.clone())
    }

    #[wasm_bindgen(js_name = setColor)]
    pub fn set_color(&mut self, value: Option<String>) {
        self.color = value;
    }

    #[wasm_bindgen(
        getter = description,
        unchecked_return_type = "StaticTranslatableString | null"
    )]
    pub fn get_description(&self) -> JsValue {
        crate::js_json::or_null(
            self.0
                .description
                .clone()
                .map(StaticTranslatableString::from),
        )
    }

    #[wasm_bindgen(setter = description)]
    pub fn set_description(&mut self, value: Option<StaticTranslatableString>) {
        self.0.description = value.map(|v| v.0);
    }

    #[wasm_bindgen(js_name = getEdges, unchecked_return_type = "number | null")]
    pub fn get_edges(&self) -> JsValue {
        crate::js_json::or_null(self.edges)
    }

    #[wasm_bindgen(js_name = setEdges)]
    pub fn set_edges(&mut self, value: Option<u32>) {
        self.edges = value;
    }

    #[wasm_bindgen(js_name = getIconClass, unchecked_return_type = "string | null")]
    pub fn get_iconclass(&self) -> JsValue {
        crate::js_json::or_null(self.iconclass.clone())
    }

    #[wasm_bindgen(js_name = setIconClass)]
    pub fn set_iconclass(&mut self, value: Option<String>) {
        self.iconclass = value;
    }

    #[wasm_bindgen(js_name = getIsEditable, unchecked_return_type = "boolean | null")]
    pub fn get_is_editable(&self) -> JsValue {
        crate::js_json::or_null(self.is_editable)
    }

    #[wasm_bindgen(js_name = setIsEditable)]
    pub fn set_is_editable(&mut self, value: Option<bool>) {
        self.is_editable = value;
    }

    #[wasm_bindgen(js_name = getJsonLdContext)]
    pub fn get_jsonldcontext(&self) -> Result<JsValue, JsValue> {
        Ok(crate::js_json::to_value(&self.jsonldcontext)?)
    }

    #[wasm_bindgen(js_name = setJsonLdContext)]
    pub fn set_jsonldcontext(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.jsonldcontext = crate::js_json::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(getter = name, unchecked_return_type = "StaticTranslatableString | null")]
    pub fn get_name(&self) -> JsValue {
        crate::js_json::or_null(self.0.name.clone().map(StaticTranslatableString::from))
    }

    #[wasm_bindgen(setter = name)]
    pub fn set_name(&mut self, value: Option<StaticTranslatableString>) {
        self.0.name = value.map(|v| v.0);
    }

    #[wasm_bindgen(js_name = getNodeGroups, unchecked_return_type = "number | null")]
    pub fn get_nodegroups(&self) -> JsValue {
        crate::js_json::or_null(self.nodegroups)
    }

    #[wasm_bindgen(js_name = setNodeGroups)]
    pub fn set_nodegroups(&mut self, value: Option<u32>) {
        self.nodegroups = value;
    }

    #[wasm_bindgen(js_name = getNodes, unchecked_return_type = "number | null")]
    pub fn get_nodes(&self) -> JsValue {
        crate::js_json::or_null(self.nodes)
    }

    #[wasm_bindgen(js_name = setNodes)]
    pub fn set_nodes(&mut self, value: Option<u32>) {
        self.nodes = value;
    }

    /// Get the ontology IDs attached to this graph. Returns a plain string
    /// for single-ontology graphs (legacy Arches shape), an array of strings
    /// for multi-ontology graphs, or `null`.
    #[wasm_bindgen(js_name = getOntologyId)]
    pub fn get_ontology_id(&self) -> JsValue {
        crate::utils::classes_to_js_value(self.0.ontology_id.as_ref())
    }

    /// Set the ontology IDs for this graph. Accepts a JS string, a JS array
    /// of strings, `null`, or `undefined`.
    #[wasm_bindgen(js_name = setOntologyId)]
    pub fn set_ontology_id(&mut self, value: JsValue) {
        self.0.ontology_id = crate::utils::parse_js_class_list(value);
    }

    #[wasm_bindgen(js_name = getPublication)]
    pub fn get_publication(&self) -> Result<JsValue, JsValue> {
        Ok(crate::js_json::to_value(&self.publication)?)
    }

    #[wasm_bindgen(js_name = setPublication)]
    pub fn set_publication(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.publication = crate::js_json::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getRelatableResourceModelIds)]
    pub fn get_relatable_resource_model_ids(&self) -> Result<JsValue, JsValue> {
        Ok(crate::js_json::to_value(
            &self.relatable_resource_model_ids,
        )?)
    }

    #[wasm_bindgen(js_name = setRelatableResourceModelIds)]
    pub fn set_relatable_resource_model_ids(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.relatable_resource_model_ids = crate::js_json::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getResource2ResourceConstraints)]
    pub fn get_resource_2_resource_constraints(&self) -> Result<JsValue, JsValue> {
        Ok(crate::js_json::to_value(
            &self.resource_2_resource_constraints,
        )?)
    }

    #[wasm_bindgen(js_name = setResource2ResourceConstraints)]
    pub fn set_resource_2_resource_constraints(&mut self, value: JsValue) -> Result<(), JsValue> {
        self.resource_2_resource_constraints = crate::js_json::from_value(value)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getRoot, unchecked_return_type = "StaticNode | null")]
    pub fn get_root(&self) -> JsValue {
        crate::js_json::or_null(
            self.0
                .root
                .as_ref()
                .map(|b| StaticNode::from((**b).clone())),
        )
    }

    #[wasm_bindgen(js_name = setRoot)]
    pub fn set_root(&mut self, value: Option<StaticNode>) {
        self.0.root = value.map(|v| Box::new(v.0));
    }

    #[wasm_bindgen(getter = slug, unchecked_return_type = "string | null")]
    pub fn get_slug(&self) -> JsValue {
        crate::js_json::or_null(self.slug.clone())
    }

    #[wasm_bindgen(setter = slug)]
    pub fn set_slug(&mut self, value: Option<String>) {
        self.slug = value;
    }

    #[wasm_bindgen(
        getter = subtitle,
        unchecked_return_type = "StaticTranslatableString | null"
    )]
    pub fn get_subtitle(&self) -> JsValue {
        crate::js_json::or_null(self.0.subtitle.clone().map(StaticTranslatableString::from))
    }

    #[wasm_bindgen(setter = subtitle)]
    pub fn set_subtitle(&mut self, value: Option<StaticTranslatableString>) {
        self.0.subtitle = value.map(|v| v.0);
    }

    #[wasm_bindgen(js_name = getVersion, unchecked_return_type = "string | null")]
    pub fn get_version(&self) -> JsValue {
        crate::js_json::or_null(self.version.clone())
    }

    #[wasm_bindgen(js_name = setVersion)]
    pub fn set_version(&mut self, value: Option<String>) {
        self.version = value;
    }
}

// ============================================================================
// StaticNode - WASM wrapper around core type
// ============================================================================

// Generate the base wrapper with Deref, From, Serialize, Deserialize
// All methods are custom due to typed returns and custom toJSON
wasm_wrapper! {
    pub struct StaticNode wraps alizarin_core::StaticNode {
        no_constructor,
        no_copy,
    }
}

#[wasm_bindgen]
impl StaticNode {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticNode, JsValue> {
        let data: StaticNode = crate::js_json::from_value(json_data)?;
        Ok(data)
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticNode {
        self.clone()
    }

    // Getters and setters for wasm-bindgen
    #[wasm_bindgen(getter = alias, unchecked_return_type = "string | null")]
    pub fn get_alias(&self) -> JsValue {
        crate::js_json::or_null(self.alias.clone())
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
        self.0.datatype = value;
    }

    #[wasm_bindgen(
        getter = description,
        unchecked_return_type = "StaticTranslatableString | null"
    )]
    pub fn get_description(&self) -> JsValue {
        crate::js_json::or_null(
            self.0
                .description
                .clone()
                .map(StaticTranslatableString::from),
        )
    }

    #[wasm_bindgen(setter = description)]
    pub fn set_description(&mut self, value: Option<StaticTranslatableString>) {
        self.0.description = value.map(|v| v.0);
    }

    #[wasm_bindgen(getter = exportable)]
    pub fn get_exportable(&self) -> bool {
        self.0.exportable
    }

    #[wasm_bindgen(setter = exportable)]
    pub fn set_exportable(&mut self, value: bool) {
        self.exportable = value;
    }

    #[wasm_bindgen(getter = fieldname, unchecked_return_type = "string | null")]
    pub fn get_fieldname(&self) -> JsValue {
        crate::js_json::or_null(self.fieldname.clone())
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

    #[wasm_bindgen(getter = nodegroup_id, unchecked_return_type = "string | null")]
    pub fn get_nodegroup_id(&self) -> JsValue {
        crate::js_json::or_null(self.nodegroup_id.clone())
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

    /// Get the ontology class list. Returns a JS string when exactly one
    /// class is set (back-compat), a JS array when multiple are declared,
    /// or `null` when none.
    #[wasm_bindgen(getter = ontologyclass)]
    pub fn get_ontologyclass(&self) -> JsValue {
        crate::utils::classes_to_js_value(self.0.ontologyclass.as_ref())
    }

    /// Set the ontology class list. Accepts a JS string, a JS array of
    /// strings, `null`, or `undefined`. Empty/blank entries are stripped.
    #[wasm_bindgen(setter = ontologyclass)]
    pub fn set_ontologyclass(&mut self, value: JsValue) {
        self.0.ontologyclass = crate::utils::parse_js_class_list(value);
    }

    #[wasm_bindgen(getter = parentproperty, unchecked_return_type = "string | null")]
    pub fn get_parentproperty(&self) -> JsValue {
        crate::js_json::or_null(self.parentproperty.clone())
    }

    #[wasm_bindgen(setter = parentproperty)]
    pub fn set_parentproperty(&mut self, value: Option<String>) {
        self.parentproperty = value;
    }

    #[wasm_bindgen(getter = sortorder, unchecked_return_type = "number | null")]
    pub fn get_sortorder(&self) -> JsValue {
        crate::js_json::or_null(self.sortorder)
    }

    #[wasm_bindgen(setter = sortorder)]
    pub fn set_sortorder(&mut self, value: Option<i32>) {
        self.sortorder = value;
    }

    #[wasm_bindgen(
        getter = sourcebranchpublication_id,
        unchecked_return_type = "string | null"
    )]
    pub fn get_sourcebranchpublication_id(&self) -> JsValue {
        crate::js_json::or_null(self.sourcebranchpublication_id.clone())
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
        match crate::js_json::from_value(value) {
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

    /// Whether two nodes (StaticNodes or plain, possibly partial, objects) are
    /// the same apart from their ids: core's `compare_nodes`, as NAPI's
    /// `compareNodes` and Python's `compare_nodes`. One of `"same"`,
    /// `"same_ids_partial"`, `"nodeid_conflict"`, `"nodegroup_id_conflict"`,
    /// `"graph_id_conflict"`, `"different"`.
    #[wasm_bindgen(js_name = compare)]
    pub fn compare_static(node_a: JsValue, node_b: JsValue) -> Result<String, JsValue> {
        let json = |v: &JsValue| -> Result<String, JsValue> {
            js_sys::JSON::stringify(v)?
                .as_string()
                .ok_or_else(|| JsValue::from_str("Invalid node: not JSON"))
        };
        alizarin_core::compare_nodes_from_json(&json(&node_a)?, &json(&node_b)?)
            .map(|c| c.as_str().to_string())
            .map_err(|e| JsValue::from_str(&e))
    }
}

// ============================================================================
// StaticNodegroup - WASM wrapper around core type
// ============================================================================

// Generate the base wrapper. Custom methods needed for:
// - toJSON: explicit null handling
// - Typed getters (Option<String>, String) instead of JsValue
// - legacygroupid always returns null
// - parentnodegroup_id returns JsValue not Option<String>
wasm_wrapper! {
    pub struct StaticNodegroup wraps alizarin_core::StaticNodegroup {
        no_constructor,
        no_copy,
    }
}

#[wasm_bindgen]
impl StaticNodegroup {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticNodegroup, JsValue> {
        let data: CoreStaticNodegroup = crate::js_json::from_value(json_data)?;
        Ok(StaticNodegroup(data))
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticNodegroup {
        self.clone()
    }

    #[wasm_bindgen(getter = cardinality, unchecked_return_type = "string | null")]
    pub fn get_cardinality(&self) -> JsValue {
        crate::js_json::or_null(self.0.cardinality.clone())
    }

    #[wasm_bindgen(setter = cardinality)]
    pub fn set_cardinality(&mut self, value: Option<String>) {
        self.0.cardinality = value;
    }

    #[wasm_bindgen(getter = legacygroupid, unchecked_return_type = "string | null")]
    pub fn get_legacygroupid(&self) -> JsValue {
        crate::js_json::or_null(self.0.legacygroupid.clone())
    }

    #[wasm_bindgen(getter = nodegroupid)]
    pub fn get_nodegroupid(&self) -> String {
        self.0.nodegroupid.clone()
    }

    #[wasm_bindgen(setter = nodegroupid)]
    pub fn set_nodegroupid(&mut self, value: String) {
        self.0.nodegroupid = value;
    }

    #[wasm_bindgen(getter = parentnodegroup_id)]
    pub fn get_parentnodegroup_id(&self) -> JsValue {
        match &self.0.parentnodegroup_id {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = parentnodegroup_id)]
    pub fn set_parentnodegroup_id(&mut self, value: Option<String>) {
        self.0.parentnodegroup_id = value;
    }
}

// ============================================================================
// StaticConstraint - WASM wrapper around core type
// ============================================================================

wasm_wrapper! {
    pub struct StaticConstraint wraps alizarin_core::StaticConstraint {
        no_constructor,
        no_copy,
        get set card_id,
        get set constraintid,
        get set uniquetoallinstances,
    }
}

#[wasm_bindgen]
impl StaticConstraint {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticConstraint, JsValue> {
        let data: CoreStaticConstraint = crate::js_json::from_value(json_data)?;
        Ok(StaticConstraint(data))
    }

    #[wasm_bindgen(getter = nodes)]
    pub fn get_nodes(&self) -> JsValue {
        crate::js_json::to_value(&self.0.nodes).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = nodes)]
    pub fn set_nodes(&mut self, value: JsValue) {
        if let Ok(nodes) = crate::js_json::from_value(value) {
            self.0.nodes = nodes;
        }
    }
}

// ============================================================================
// StaticPublication - WASM wrapper around core type
// ============================================================================

wasm_wrapper! {
    pub struct StaticPublication wraps alizarin_core::StaticPublication {
        no_constructor,
        no_copy,
        get set graph_id,
        get set publicationid,
        get set published_time,
    }
}

#[wasm_bindgen]
impl StaticPublication {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticPublication, JsValue> {
        let data: CoreStaticPublication = crate::js_json::from_value(json_data)?;
        Ok(StaticPublication(data))
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticPublication {
        self.clone()
    }

    #[wasm_bindgen(getter = notes)]
    pub fn get_notes(&self) -> JsValue {
        match &self.0.notes {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = notes)]
    pub fn set_notes(&mut self, value: Option<String>) {
        self.0.notes = value;
    }
}

// ============================================================================
// StaticCardsXNodesXWidgets - WASM wrapper around core type
// ============================================================================

wasm_wrapper! {
    pub struct StaticCardsXNodesXWidgets wraps alizarin_core::StaticCardsXNodesXWidgets {
        no_constructor,
        no_copy,
        get set card_id,
        get set id,
        get set node_id,
        get set sortorder,
        get set visible,
        get set widget_id,
    }
}

#[wasm_bindgen]
impl StaticCardsXNodesXWidgets {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticCardsXNodesXWidgets, JsValue> {
        let data: CoreStaticCardsXNodesXWidgets = crate::js_json::from_value(json_data)?;
        Ok(StaticCardsXNodesXWidgets(data))
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        json_to_js_value(&self.0.config)
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config(&mut self, value: JsValue) {
        if let Ok(config) = crate::js_json::from_value(value) {
            self.0.config = config;
        }
    }

    #[wasm_bindgen(getter = label)]
    pub fn get_label(&self) -> StaticTranslatableString {
        StaticTranslatableString::from(self.0.label.clone())
    }

    #[wasm_bindgen(setter = label)]
    pub fn set_label(&mut self, value: StaticTranslatableString) {
        self.0.label = value.0;
    }
}

// ============================================================================
// StaticFunctionsXGraphs - WASM wrapper around core type
// ============================================================================

wasm_wrapper! {
    pub struct StaticFunctionsXGraphs wraps alizarin_core::StaticFunctionsXGraphs {
        no_constructor,
        no_copy,
        get set function_id,
        get set graph_id,
        get set id,
    }
}

#[wasm_bindgen]
impl StaticFunctionsXGraphs {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticFunctionsXGraphs, JsValue> {
        let data: CoreStaticFunctionsXGraphs = crate::js_json::from_value(json_data)?;
        Ok(StaticFunctionsXGraphs(data))
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticFunctionsXGraphs {
        self.clone()
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        json_to_js_value(&self.0.config)
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config(&mut self, value: JsValue) {
        if let Ok(config) = crate::js_json::from_value(value) {
            self.0.config = config;
        }
    }
}

// ============================================================================
// StaticCard - WASM wrapper around core type
// ============================================================================

wasm_wrapper! {
    pub struct StaticCard wraps alizarin_core::StaticCard {
        no_constructor,
        no_copy,
        get set active,
        get set cardid,
        get set component_id,
        get set cssclass,
        get set graph_id,
        get set helpenabled,
        get set is_editable,
        get set nodegroup_id,
        get set sortorder,
        get set visible,
    }
}

#[wasm_bindgen]
impl StaticCard {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticCard, JsValue> {
        let data: CoreStaticCard = crate::js_json::from_value(json_data)?;
        Ok(StaticCard(data))
    }

    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        self.0
            .config
            .as_ref()
            .map(json_to_js_value)
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = config)]
    pub fn set_config(&mut self, value: JsValue) {
        if let Ok(config) = crate::js_json::from_value(value) {
            self.0.config = Some(config);
        }
    }

    #[wasm_bindgen(getter = constraints)]
    pub fn get_constraints(&self) -> JsValue {
        crate::js_json::to_value(&self.0.constraints).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter = constraints)]
    pub fn set_constraints(&mut self, value: JsValue) {
        if let Ok(constraints) = crate::js_json::from_value(value) {
            self.0.constraints = constraints;
        }
    }

    #[wasm_bindgen(
        getter = description,
        unchecked_return_type = "StaticTranslatableString | null"
    )]
    pub fn get_description(&self) -> JsValue {
        crate::js_json::or_null(
            self.0
                .description
                .as_ref()
                .map(|d| StaticTranslatableString::from(d.clone())),
        )
    }

    #[wasm_bindgen(setter = description)]
    pub fn set_description(&mut self, value: Option<StaticTranslatableString>) {
        self.0.description = value.map(|v| v.0);
    }

    #[wasm_bindgen(getter = helptext)]
    pub fn get_helptext(&self) -> StaticTranslatableString {
        StaticTranslatableString::from(self.0.helptext.clone())
    }

    #[wasm_bindgen(setter = helptext)]
    pub fn set_helptext(&mut self, value: StaticTranslatableString) {
        self.0.helptext = value.0;
    }

    #[wasm_bindgen(getter = helptitle)]
    pub fn get_helptitle(&self) -> StaticTranslatableString {
        StaticTranslatableString::from(self.0.helptitle.clone())
    }

    #[wasm_bindgen(setter = helptitle)]
    pub fn set_helptitle(&mut self, value: StaticTranslatableString) {
        self.0.helptitle = value.0;
    }

    #[wasm_bindgen(getter = instructions)]
    pub fn get_instructions(&self) -> StaticTranslatableString {
        StaticTranslatableString::from(self.0.instructions.clone())
    }

    #[wasm_bindgen(setter = instructions)]
    pub fn set_instructions(&mut self, value: StaticTranslatableString) {
        self.0.instructions = value.0;
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> StaticTranslatableString {
        StaticTranslatableString::from(self.0.name.clone())
    }

    #[wasm_bindgen(setter = name)]
    pub fn set_name(&mut self, value: StaticTranslatableString) {
        self.0.name = value.0;
    }
}

// StaticTile - wrapper around core type with custom data handling
wasm_wrapper! {
    pub struct StaticTile wraps alizarin_core::StaticTile {
        no_constructor,
        no_copy,
    }
}

// Rust-only methods (not exposed to JS)
impl StaticTile {
    /// Get a reference to the inner core type (for Rust-side access)
    pub fn inner(&self) -> &CoreStaticTile {
        &self.0
    }

    /// Consume and return the inner core type (for Rust-side access)
    pub fn into_inner(self) -> CoreStaticTile {
        self.0
    }
}

#[wasm_bindgen]
impl StaticTile {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticTile, JsValue> {
        let data: CoreStaticTile = crate::js_json::from_value(json_data)?;
        Ok(StaticTile(data))
    }

    #[wasm_bindgen(js_name = ensureId)]
    pub fn ensure_id(&mut self) -> String {
        if self.0.tileid.is_none() {
            // Generate a UUID
            self.0.tileid = Some(uuid::Uuid::new_v4().to_string());
        }
        self.0.tileid.clone().unwrap()
    }

    #[wasm_bindgen(getter = data)]
    pub fn get_data(&self) -> JsValue {
        // Return as a JavaScript Map (keys sorted, as tile data is a BTreeMap)
        let js_map = js_sys::Map::new();
        for (key, val) in &self.0.data {
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
                let mut new_data = BTreeMap::new();
                map.for_each(&mut |val, key| {
                    if let Some(key_str) = key.as_string() {
                        if let Ok(json_val) = crate::js_json::from_value(val) {
                            new_data.insert(key_str, json_val);
                        }
                    }
                });
                self.0.data = new_data;
            } else if let Ok(obj_data) =
                crate::js_json::from_value::<BTreeMap<String, serde_json::Value>>(value)
            {
                self.0.data = obj_data;
            }
        }
    }

    #[wasm_bindgen(getter = nodegroup_id)]
    pub fn get_nodegroup_id(&self) -> String {
        self.0.nodegroup_id.clone()
    }

    #[wasm_bindgen(setter = nodegroup_id)]
    pub fn set_nodegroup_id(&mut self, value: String) {
        self.0.nodegroup_id = value;
    }

    #[wasm_bindgen(getter = resourceinstance_id)]
    pub fn get_resourceinstance_id(&self) -> String {
        self.0.resourceinstance_id.clone()
    }

    #[wasm_bindgen(setter = resourceinstance_id)]
    pub fn set_resourceinstance_id(&mut self, value: String) {
        self.0.resourceinstance_id = value;
    }

    #[wasm_bindgen(getter = tileid)]
    pub fn get_tileid(&self) -> JsValue {
        match &self.0.tileid {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = tileid)]
    pub fn set_tileid(&mut self, value: JsValue) {
        if value.is_null() {
            self.0.tileid = None;
        } else if let Some(s) = value.as_string() {
            self.0.tileid = Some(s);
        }
    }

    #[wasm_bindgen(getter = parenttile_id)]
    pub fn get_parenttile_id(&self) -> JsValue {
        match &self.0.parenttile_id {
            Some(val) => JsValue::from_str(val),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = parenttile_id)]
    pub fn set_parenttile_id(&mut self, value: JsValue) {
        if value.is_null() {
            self.0.parenttile_id = None;
        } else if let Some(s) = value.as_string() {
            self.0.parenttile_id = Some(s);
        }
    }

    #[wasm_bindgen(getter = provisionaledits)]
    pub fn get_provisionaledits(&self) -> JsValue {
        match &self.0.provisionaledits {
            Some(val) => {
                // Convert Vec<serde_json::Value> to plain JS array with plain objects
                let js_array = js_sys::Array::new();
                for item in val {
                    js_array.push(&json_to_js_value(item));
                }
                js_array.into()
            }
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = provisionaledits)]
    pub fn set_provisionaledits(&mut self, value: JsValue) {
        if value.is_null() {
            self.0.provisionaledits = None;
        } else if let Ok(edits) = crate::js_json::from_value(value) {
            self.0.provisionaledits = Some(edits);
        }
    }

    #[wasm_bindgen(getter = sortorder)]
    pub fn get_sortorder(&self) -> JsValue {
        match self.0.sortorder {
            Some(val) => JsValue::from_f64(val as f64),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = sortorder)]
    pub fn set_sortorder(&mut self, value: JsValue) {
        if value.is_null() {
            self.0.sortorder = None;
        } else if let Some(n) = value.as_f64() {
            self.0.sortorder = Some(n as i32);
        }
    }
}

// Non-WASM methods for internal Rust use
impl StaticTile {
    /// Create a new empty tile for a nodegroup (internal Rust constructor)
    /// Used by json_conversion module for tree_to_tiles
    #[allow(dead_code)]
    pub(crate) fn new_empty(nodegroup_id: String) -> Self {
        StaticTile(CoreStaticTile::new_empty(nodegroup_id))
    }
}

// ============================================================================
// StaticEdge - WASM wrapper around core type
// ============================================================================

// Generate the base wrapper. Custom methods needed for:
// - toJSON: only includes optional fields if present
// - Typed getters (Option<String>, String) instead of JsValue
wasm_wrapper! {
    pub struct StaticEdge wraps alizarin_core::StaticEdge {
        no_constructor,
        no_copy,
    }
}

#[wasm_bindgen]
impl StaticEdge {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticEdge, JsValue> {
        let data: CoreStaticEdge = crate::js_json::from_value(json_data)?;
        Ok(StaticEdge(data))
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticEdge {
        self.clone()
    }

    // Getters
    #[wasm_bindgen(getter = description, unchecked_return_type = "string | null")]
    pub fn get_description(&self) -> JsValue {
        crate::js_json::or_null(self.0.description.clone())
    }

    #[wasm_bindgen(getter = domainnode_id)]
    pub fn get_domainnode_id(&self) -> String {
        self.0.domainnode_id.clone()
    }

    #[wasm_bindgen(getter = edgeid)]
    pub fn get_edgeid(&self) -> String {
        self.0.edgeid.clone()
    }

    #[wasm_bindgen(getter = graph_id)]
    pub fn get_graph_id(&self) -> String {
        self.0.graph_id.clone()
    }

    #[wasm_bindgen(getter = name, unchecked_return_type = "string | null")]
    pub fn get_name(&self) -> JsValue {
        crate::js_json::or_null(self.0.name.clone())
    }

    #[wasm_bindgen(getter = ontologyproperty, unchecked_return_type = "string | null")]
    pub fn get_ontologyproperty(&self) -> JsValue {
        crate::js_json::or_null(self.0.ontologyproperty.clone())
    }

    #[wasm_bindgen(getter = rangenode_id)]
    pub fn get_rangenode_id(&self) -> String {
        self.0.rangenode_id.clone()
    }
}

// StaticGraph - The main graph structure (wraps core type)
use alizarin_core::StaticGraph as CoreStaticGraph;

wasm_wrapper! {
    pub struct StaticGraph wraps alizarin_core::StaticGraph {
        no_constructor,
        no_copy,
        // Note: Most getters defined manually in impl block below to wrap types properly
        get slug,
        get is_editable,
        get color,
        get ontology_id,
        get deploymentdate,
        get deploymentfile,
        get jsonldcontext,
        // config has a manual getter below so it returns a plain JS object
        // (preserving arbitrary host keys) rather than a serde_wasm_bindgen Map,
        // which inspected as `{}` and dropped custom keys (C11).
        get relatable_resource_model_ids,
        get resource_2_resource_constraints,
    }
}

#[wasm_bindgen]
impl StaticGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: JsValue) -> Result<StaticGraph, JsValue> {
        // Log the incoming data for debugging
        let mut data: CoreStaticGraph = crate::js_json::from_value(json_data).map_err(|e| {
            let error_string = e.to_string();

            // Log detailed error information
            web_sys::console::error_1(&format!("Error: {}", error_string).into());
            web_sys::console::error_1(&format!("Debug: {:?}", e).into());

            // Try to extract field name from error message
            if error_string.contains("missing field") {
                web_sys::console::error_1(&"^ This is a MISSING FIELD error".into());
            } else if error_string.contains("invalid type") {
                web_sys::console::error_1(&"^ This is an INVALID TYPE error".into());
            }

            JsValue::from_str(&format!(
                "StaticGraph deserialization failed: {}",
                error_string
            ))
        })?;
        data.build_indices();
        Ok(StaticGraph(data))
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> StaticGraph {
        let mut cloned = self.0.clone();
        cloned.build_indices();
        StaticGraph(cloned)
    }

    /// Parse JSON string with {graph: [StaticGraph]} structure
    /// Returns the first graph from the array, or error if none found
    #[wasm_bindgen(js_name = fromJsonString)]
    pub fn from_json_string(json_str: &str) -> Result<StaticGraph, JsValue> {
        // Core's parsing, as NAPI uses: a graph, or `{graph: [...]}`.
        CoreStaticGraph::from_json_string(json_str)
            .map(StaticGraph)
            .map_err(|e| JsValue::from_str(&e))
    }

    // Getters for key fields
    #[wasm_bindgen(getter = graphid)]
    pub fn get_graphid(&self) -> String {
        self.0.graphid.clone()
    }

    /// Graph-level config as a plain JS object (not a Map), preserving any
    /// host-specific keys the model carries (C11).
    #[wasm_bindgen(getter = config)]
    pub fn get_config(&self) -> JsValue {
        json_to_js_value(&self.0.config)
    }

    #[wasm_bindgen(getter = name)]
    pub fn get_name(&self) -> StaticTranslatableString {
        StaticTranslatableString(self.0.name.clone())
    }

    #[wasm_bindgen(
        getter = description,
        unchecked_return_type = "StaticTranslatableString | null"
    )]
    pub fn get_description(&self) -> JsValue {
        crate::js_json::or_null(self.0.description.clone().map(StaticTranslatableString))
    }

    #[wasm_bindgen(
        getter = subtitle,
        unchecked_return_type = "StaticTranslatableString | null"
    )]
    pub fn get_subtitle(&self) -> JsValue {
        crate::js_json::or_null(self.0.subtitle.clone().map(StaticTranslatableString))
    }

    #[wasm_bindgen(getter = nodes)]
    pub fn get_nodes(&self) -> js_sys::Array {
        // Return an array of StaticNode WASM instances, not plain JS objects
        // This preserves the getters/setters on each node
        let array = js_sys::Array::new();
        for node in &self.0.nodes {
            array.push(&JsValue::from(StaticNode(node.clone())));
        }
        array
    }

    #[wasm_bindgen(setter = nodes)]
    pub fn set_nodes(&mut self, value: JsValue) {
        if let Ok(nodes) = crate::js_json::from_value(value) {
            self.0.nodes = nodes;
            self.0.build_indices();
        }
    }

    #[wasm_bindgen(getter = edges)]
    pub fn get_edges(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for edge in &self.0.edges {
            array.push(&JsValue::from(StaticEdge(edge.clone())));
        }
        array
    }

    #[wasm_bindgen(setter = edges)]
    pub fn set_edges(&mut self, value: JsValue) {
        if let Ok(edges) = crate::js_json::from_value(value) {
            self.0.edges = edges;
        }
    }

    #[wasm_bindgen(getter = nodegroups)]
    pub fn get_nodegroups(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for nodegroup in &self.0.nodegroups {
            array.push(&JsValue::from(StaticNodegroup(nodegroup.clone())));
        }
        array
    }

    #[wasm_bindgen(setter = nodegroups)]
    pub fn set_nodegroups(&mut self, value: JsValue) {
        if let Ok(nodegroups) = crate::js_json::from_value(value) {
            self.0.nodegroups = nodegroups;
        }
    }

    #[wasm_bindgen(getter = cards)]
    pub fn get_cards(&self) -> JsValue {
        match &self.0.cards {
            Some(cards) => {
                let array = js_sys::Array::new();
                for card in cards.iter() {
                    array.push(&JsValue::from(StaticCard(card.clone())));
                }
                JsValue::from(array)
            }
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = cards)]
    pub fn set_cards(&mut self, value: JsValue) {
        if let Ok(cards) =
            crate::js_json::from_value::<Option<Vec<alizarin_core::StaticCard>>>(value)
        {
            self.0.cards = cards;
        }
    }

    #[wasm_bindgen(getter = cards_x_nodes_x_widgets)]
    pub fn get_cards_x_nodes_x_widgets(&self) -> JsValue {
        match &self.0.cards_x_nodes_x_widgets {
            Some(cxnxws) => {
                let array = js_sys::Array::new();
                for cxnxw in cxnxws.iter() {
                    array.push(&JsValue::from(StaticCardsXNodesXWidgets(cxnxw.clone())));
                }
                JsValue::from(array)
            }
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = cards_x_nodes_x_widgets)]
    pub fn set_cards_x_nodes_x_widgets(&mut self, value: JsValue) {
        if let Ok(cards_x_nodes_x_widgets) = crate::js_json::from_value::<
            Option<Vec<alizarin_core::StaticCardsXNodesXWidgets>>,
        >(value)
        {
            self.0.cards_x_nodes_x_widgets = cards_x_nodes_x_widgets;
        }
    }

    /// The graph's function attachments, as NAPI's `functions_x_graphs`.
    #[wasm_bindgen(
        getter = functions_x_graphs,
        unchecked_return_type = "StaticFunctionsXGraphs[] | null"
    )]
    pub fn get_functions_x_graphs(&self) -> JsValue {
        crate::js_json::or_null(self.0.functions_x_graphs.as_ref().map(|rows| {
            rows.iter()
                .map(|row| JsValue::from(StaticFunctionsXGraphs(row.clone())))
                .collect::<js_sys::Array>()
        }))
    }

    #[wasm_bindgen(getter = root)]
    pub fn get_root(&self) -> StaticNode {
        StaticNode(self.0.root.clone())
    }

    #[wasm_bindgen(getter = isresource, unchecked_return_type = "boolean | null")]
    pub fn get_isresource(&self) -> JsValue {
        crate::js_json::or_null(self.0.isresource)
    }

    #[wasm_bindgen(getter = author, unchecked_return_type = "string | null")]
    pub fn get_author(&self) -> JsValue {
        crate::js_json::or_null(self.0.author.clone())
    }

    #[wasm_bindgen(getter = iconclass, unchecked_return_type = "string | null")]
    pub fn get_iconclass(&self) -> JsValue {
        crate::js_json::or_null(self.0.iconclass.clone())
    }

    #[wasm_bindgen(getter = template_id, unchecked_return_type = "string | null")]
    pub fn get_template_id(&self) -> JsValue {
        crate::js_json::or_null(self.0.template_id.clone())
    }

    #[wasm_bindgen(getter = version, unchecked_return_type = "string | null")]
    pub fn get_version(&self) -> JsValue {
        crate::js_json::or_null(self.0.version.clone())
    }

    // Helper methods for modifying collections
    #[wasm_bindgen(js_name = pushNode)]
    pub fn push_node(&mut self, node: StaticNode) {
        self.0.nodes.push(node.0);
        self.0.build_indices();
    }

    #[wasm_bindgen(js_name = pushEdge)]
    pub fn push_edge(&mut self, edge: StaticEdge) {
        self.0.edges.push(edge.0);
    }

    #[wasm_bindgen(js_name = pushNodegroup)]
    pub fn push_nodegroup(&mut self, nodegroup: StaticNodegroup) {
        self.0.nodegroups.push(nodegroup.0);
    }

    #[wasm_bindgen(js_name = pushCard)]
    pub fn push_card(&mut self, card: StaticCard) {
        if self.0.cards.is_none() {
            self.0.cards = Some(Vec::new());
        }
        if let Some(ref mut cards) = self.0.cards {
            cards.push(card.0);
        }
    }

    #[wasm_bindgen(js_name = pushCardXNodeXWidget)]
    pub fn push_card_x_node_x_widget(&mut self, cxnxw: StaticCardsXNodesXWidgets) {
        if self.0.cards_x_nodes_x_widgets.is_none() {
            self.0.cards_x_nodes_x_widgets = Some(Vec::new());
        }
        if let Some(ref mut cxnxws) = self.0.cards_x_nodes_x_widgets {
            cxnxws.push(cxnxw.0);
        }
    }

    // JS-facing lookup methods (accept Option to handle null/undefined gracefully)
    #[wasm_bindgen(js_name = getNodeById)]
    pub fn get_node_by_id_js(&self, id: Option<String>) -> JsValue {
        id.and_then(|id| self.0.get_node_by_id(&id))
            .map(|node| JsValue::from(StaticNode(node.clone())))
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = getNodeByAlias)]
    pub fn get_node_by_alias_js(&self, alias: Option<String>) -> JsValue {
        alias
            .and_then(|a| self.0.get_node_by_alias(&a))
            .map(|node| JsValue::from(StaticNode(node.clone())))
            .unwrap_or(JsValue::NULL)
    }

    /// Get a simplified schema view of the graph showing node aliases and structure.
    ///
    /// Returns a nested object representing the tree with:
    /// - Keys are node aliases (or nodeid if no alias)
    /// - Values contain 'datatype', 'nodeid', optionally 'required', and 'children'
    ///
    /// Useful for understanding what keys are available in tree output.
    ///
    /// @example
    /// ```typescript
    /// const schema = graph.getSchema();
    /// console.log(JSON.stringify(schema, null, 2));
    /// // {
    /// //   "name": {
    /// //     "datatype": "semantic",
    /// //     "children": {
    /// //       "forenames": { ... }
    /// //     }
    /// //   }
    /// // }
    /// ```
    #[wasm_bindgen(js_name = getSchema)]
    pub fn get_schema(&self) -> Result<JsValue, JsValue> {
        let schema = self.0.get_schema();
        crate::js_json::to_value(&schema)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize schema: {}", e)))
    }

    /// Set a descriptor template for a given type (e.g. "slug", "name").
    /// The nodegroup_id is inferred from the placeholder node names in the template.
    #[wasm_bindgen(js_name = setDescriptorTemplate)]
    pub fn set_descriptor_template(
        &mut self,
        descriptor_type: &str,
        string_template: &str,
    ) -> Result<(), JsValue> {
        self.0
            .set_descriptor_template(descriptor_type, string_template)
            .map_err(|e| JsValue::from_str(&e))
    }
}

// Rust-internal methods (not exposed to JS) - delegate to core
// Many of these are used only in tests
impl StaticGraph {
    // Accessors for json_conversion module - return core types
    #[allow(dead_code)] // Used in tests
    pub(crate) fn nodes_slice(&self) -> &[alizarin_core::StaticNode] {
        self.0.nodes_slice()
    }

    #[allow(dead_code)] // Used in tests
    pub(crate) fn nodegroups_slice(&self) -> &[alizarin_core::StaticNodegroup] {
        self.0.nodegroups_slice()
    }

    #[allow(dead_code)] // Available for internal use
    pub(crate) fn root_node(&self) -> &alizarin_core::StaticNode {
        self.0.root_node()
    }

    #[allow(dead_code)] // Used in tests
    pub(crate) fn graph_id(&self) -> &str {
        self.0.graph_id()
    }

    /// Build the internal lookup indices (for use after deserialization)
    #[allow(dead_code)] // Used internally via self.0.build_indices()
    pub(crate) fn build_indices(&mut self) {
        self.0.build_indices()
    }

    /// Get cached edges map (parent_nodeid -> child_nodeids)
    #[allow(dead_code)] // Available for internal use
    pub(crate) fn edges_map(&self) -> Option<&std::collections::HashMap<String, Vec<String>>> {
        self.0.edges_map()
    }

    /// Get child node IDs for a given node
    #[allow(dead_code)] // Available for internal use
    pub(crate) fn get_child_ids(&self, node_id: &str) -> Option<&Vec<String>> {
        self.0.get_child_ids(node_id)
    }

    /// Get nodegroup by ID
    #[allow(dead_code)] // Available for internal use
    pub(crate) fn get_nodegroup_by_id(
        &self,
        nodegroup_id: &str,
    ) -> Option<&alizarin_core::StaticNodegroup> {
        self.0.get_nodegroup_by_id(nodegroup_id)
    }

    /// Get nodes in a specific nodegroup
    #[allow(dead_code)] // Available for internal use
    pub(crate) fn get_nodes_in_nodegroup(
        &self,
        nodegroup_id: &str,
    ) -> Vec<&alizarin_core::StaticNode> {
        self.0.get_nodes_in_nodegroup(nodegroup_id)
    }

    /// Get Arc-wrapped nodes by alias map (for pseudo_value infrastructure)
    #[allow(dead_code)] // Available for internal use
    pub(crate) fn nodes_by_alias_arc(
        &self,
    ) -> Option<&std::collections::HashMap<String, std::sync::Arc<alizarin_core::StaticNode>>> {
        self.0.nodes_by_alias_arc()
    }
}

// WKRM - Well-Known Resource Model
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WKRM {
    model_name: String,
    model_class_name: String,
    graph_id: String,
    meta: alizarin_core::StaticGraphMeta, // Store deserialized core type
}

#[wasm_bindgen]
impl WKRM {
    #[wasm_bindgen(constructor)]
    pub fn new(meta_js: JsValue) -> Result<WKRM, JsValue> {
        // Check if this is a WASM wrapper object (has __wbg_ptr field)
        // If so, call toJSON() to get a plain object we can deserialize
        let value_to_deserialize = if let Ok(ptr) =
            js_sys::Reflect::get(&meta_js, &JsValue::from_str("__wbg_ptr"))
        {
            if !ptr.is_undefined() && !ptr.is_null() {
                // This is a WASM wrapper, call toJSON() to get a plain object
                if let Ok(to_json_fn) = js_sys::Reflect::get(&meta_js, &JsValue::from_str("toJSON"))
                {
                    if to_json_fn.is_function() {
                        let func = js_sys::Function::from(to_json_fn);
                        func.call0(&meta_js).unwrap_or(meta_js.clone())
                    } else {
                        meta_js.clone()
                    }
                } else {
                    meta_js.clone()
                }
            } else {
                meta_js.clone()
            }
        } else {
            meta_js.clone()
        };

        let meta: alizarin_core::StaticGraphMeta = crate::js_json::from_value(value_to_deserialize)
            .map_err(|e| {
                let error_msg = format!("Failed to deserialize StaticGraphMeta for WKRM: {:?}", e);
                web_sys::console::error_1(&error_msg.clone().into());
                JsValue::from_str(&error_msg)
            })?;

        let graph_id = meta.graphid.clone();

        // Get the model name from the name field
        let model_name = meta
            .name
            .as_ref()
            .map(|n| n.to_string_default())
            .unwrap_or_else(|| "Unnamed".to_string());

        // Get the slug for model_class_name, or fall back to model_name
        let base_name = meta.slug.as_ref().unwrap_or(&model_name);
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
        let normalized = s.replace(['_', '-'], " ");

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
    pub fn get_meta(&self) -> StaticGraphMeta {
        StaticGraphMeta(self.meta.clone())
    }

    #[wasm_bindgen(setter = meta)]
    pub fn set_meta(&mut self, value: JsValue) {
        if let Ok(meta) = crate::js_json::from_value::<alizarin_core::StaticGraphMeta>(value) {
            self.meta = meta;
        }
    }
}

// Non-WASM impl block for WKRM - works on all platforms
impl WKRM {
    /// Create WKRM from StaticGraphMeta (works on all platforms, not just WASM)
    pub fn from_meta(meta: alizarin_core::StaticGraphMeta) -> WKRM {
        let graph_id = meta.graphid.clone();

        let model_name = meta
            .name
            .as_ref()
            .map(|n| n.to_string_default())
            .unwrap_or_else(|| "Unnamed".to_string());

        let base_name = meta.slug.as_ref().unwrap_or(&model_name);
        let model_class_name = Self::to_pascal_case(base_name);

        WKRM {
            model_name,
            model_class_name,
            graph_id,
            meta,
        }
    }

    /// Get the inner StaticGraphMeta (for non-WASM use)
    pub fn meta(&self) -> &alizarin_core::StaticGraphMeta {
        &self.meta
    }
}

// StaticResourceDescriptors - Descriptors for resource display
wasm_wrapper! {
    pub struct StaticResourceDescriptors wraps alizarin_core::StaticResourceDescriptors {
        get set name,
        get set description,
        get set slug,
    }
}

#[wasm_bindgen]
impl StaticResourceDescriptors {
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[wasm_bindgen(js_name = empty)]
    pub fn empty() -> StaticResourceDescriptors {
        StaticResourceDescriptors(alizarin_core::StaticResourceDescriptors::empty())
    }

    #[wasm_bindgen(getter = mapPopup, unchecked_return_type = "string | null")]
    pub fn map_popup(&self) -> JsValue {
        crate::js_json::or_null(self.0.map_popup.clone())
    }

    #[wasm_bindgen(setter = mapPopup)]
    pub fn set_map_popup(&mut self, value: Option<String>) {
        self.0.map_popup = value;
    }
}

// StaticResourceMetadata - Metadata about a resource instance
wasm_wrapper! {
    pub struct StaticResourceMetadata wraps alizarin_core::StaticResourceMetadata {
        get graph_id,
        get name,
        get resourceinstanceid,
        get publication_id,
        get principaluser_id,
        get legacyid,
        get graph_publication_id,
        get createdtime,
        get lastmodified,
    }
}

#[wasm_bindgen]
impl StaticResourceMetadata {
    /// Descriptors as a plain JS object (`{name, description, slug, map_popup}`),
    /// matching the NAPI backend. Returning a WASM wrapper here made
    /// `JSON.stringify(metadata.descriptors)` yield `{}` (wrappers have no
    /// enumerable own properties), which silently broke `__cache` population in
    /// the ETL — see WARNING.md.
    #[wasm_bindgen(getter)]
    pub fn descriptors(&self) -> JsValue {
        crate::js_json::to_value(&self.0.descriptors).unwrap_or(JsValue::NULL)
    }
}

// StaticResourceSummary - Summary info for a resource (used for lazy loading)
wasm_wrapper! {
    pub struct StaticResourceSummary wraps alizarin_core::StaticResourceSummary {
        no_constructor,
        no_copy,
        get resourceinstanceid,
        get graph_id,
        get name,
    }
}

#[wasm_bindgen]
impl StaticResourceSummary {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Result<StaticResourceSummary, JsValue> {
        let mut summary: CoreStaticResourceSummary =
            crate::js_json::from_value(data).map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to deserialize StaticResourceSummary: {:?}",
                    e
                ))
            })?;

        // Default name to '<Unnamed>' if empty
        if summary.name.is_empty() {
            summary.name = "<Unnamed>".to_string();
        }

        Ok(StaticResourceSummary(summary))
    }

    /// Parse summary from JSON string - faster than constructor with JS object
    #[wasm_bindgen(js_name = fromJsonString)]
    pub fn from_json_string(json_str: &str) -> Result<StaticResourceSummary, JsValue> {
        let mut summary: CoreStaticResourceSummary = serde_json::from_str(json_str)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse summary JSON: {}", e)))?;

        if summary.name.is_empty() {
            summary.name = "<Unnamed>".to_string();
        }

        Ok(StaticResourceSummary(summary))
    }

    /// Extract summaries from a JSON string containing business_data with full resources
    /// This parses all resources but only keeps the summary fields, avoiding full tile parsing
    #[wasm_bindgen(js_name = summariesFromBusinessDataJsonString)]
    pub fn summaries_from_business_data_json_string(
        json_str: &str,
    ) -> Result<Vec<StaticResourceSummary>, JsValue> {
        // Use serde_json::Value for partial parsing - we only need resourceinstance fields
        let value: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
            JsValue::from_str(&format!("Failed to parse business_data JSON: {}", e))
        })?;

        let resources = value
            .get("business_data")
            .and_then(|bd| bd.get("resources"))
            .and_then(|r| r.as_array())
            .ok_or_else(|| JsValue::from_str("Invalid business_data structure"))?;

        let summaries: Result<Vec<StaticResourceSummary>, JsValue> = resources
            .iter()
            .map(|resource| {
                let ri = resource
                    .get("resourceinstance")
                    .ok_or_else(|| JsValue::from_str("Missing resourceinstance"))?;
                let metadata = resource.get("metadata");

                let mut summary = CoreStaticResourceSummary {
                    resourceinstanceid: ri
                        .get("resourceinstanceid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    graph_id: ri
                        .get("graph_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    name: ri
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    descriptors: ri
                        .get("descriptors")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    metadata: metadata
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default(),
                    createdtime: ri
                        .get("createdtime")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    lastmodified: ri
                        .get("lastmodified")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    publication_id: ri
                        .get("publication_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    principaluser_id: ri
                        .get("principaluser_id")
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32),
                    legacyid: ri
                        .get("legacyid")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    graph_publication_id: ri
                        .get("graph_publication_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    resource_instance_lifecycle_state_id: ri
                        .get("resource_instance_lifecycle_state_id")
                        .or_else(|| ri.get("resource_instance_lifecycle_state"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    lifecycle_state_conflict: ri
                        .get("lifecycle_state_conflict")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default(),
                };

                if summary.name.is_empty() {
                    summary.name = "<Unnamed>".to_string();
                }

                Ok(StaticResourceSummary(summary))
            })
            .collect();

        summaries
    }

    pub fn copy(&self) -> StaticResourceSummary {
        StaticResourceSummary(self.0.clone())
    }

    #[wasm_bindgen(js_name = toMetadata)]
    pub fn to_metadata(&self) -> StaticResourceMetadata {
        StaticResourceMetadata(self.0.to_metadata())
    }

    /// Descriptors as a plain JS object (or null), matching the NAPI backend.
    /// A WASM wrapper here serialised to `{}` under `JSON.stringify` and broke
    /// `__cache` population in the ETL — see WARNING.md.
    #[wasm_bindgen(getter)]
    pub fn descriptors(&self) -> JsValue {
        crate::js_json::to_value(&self.0.descriptors).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        crate::js_json::to_value(&self.0.metadata).unwrap_or(JsValue::NULL)
    }

    /// Create a summary from a full resource
    #[wasm_bindgen(js_name = fromResource)]
    pub fn from_resource(resource: &StaticResource) -> StaticResourceSummary {
        let ri = &resource.0.resourceinstance;
        StaticResourceSummary(CoreStaticResourceSummary {
            resourceinstanceid: ri.resourceinstanceid.clone(),
            graph_id: ri.graph_id.clone(),
            name: ri.name.clone(),
            descriptors: Some(ri.descriptors.clone()),
            metadata: resource.0.metadata.clone(),
            createdtime: ri.createdtime.clone(),
            lastmodified: ri.lastmodified.clone(),
            publication_id: ri.publication_id.clone(),
            principaluser_id: ri.principaluser_id,
            legacyid: ri.legacyid.clone(),
            graph_publication_id: ri.graph_publication_id.clone(),
            resource_instance_lifecycle_state_id: ri.resource_instance_lifecycle_state_id.clone(),
            lifecycle_state_conflict: ri.lifecycle_state_conflict.clone(),
        })
    }
}

// StaticResourceReference - Reference to a resource instance (for resource-instance datatype)
wasm_wrapper! {
    pub struct StaticResourceReference wraps alizarin_core::StaticResourceReference {
        no_constructor,
        no_copy,
        get id,
        get graph_id,
    }
}

#[wasm_bindgen]
impl StaticResourceReference {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Result<StaticResourceReference, JsValue> {
        let reference: CoreStaticResourceReference =
            crate::js_json::from_value(data).map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to deserialize StaticResourceReference: {:?}",
                    e
                ))
            })?;
        Ok(StaticResourceReference(reference))
    }

    pub fn copy(&self) -> StaticResourceReference {
        StaticResourceReference(self.0.clone())
    }

    #[wasm_bindgen(getter = type, unchecked_return_type = "string | null")]
    pub fn resource_type(&self) -> JsValue {
        crate::js_json::or_null(self.0.resource_type.clone())
    }

    #[wasm_bindgen(getter, unchecked_return_type = "string | null")]
    pub fn title(&self) -> JsValue {
        crate::js_json::or_null(self.0.title.clone())
    }

    #[wasm_bindgen(getter)]
    pub fn root(&self) -> JsValue {
        self.0
            .root
            .as_ref()
            .and_then(|r| crate::js_json::to_value(r).ok())
            .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter)]
    pub fn meta(&self) -> JsValue {
        self.0
            .meta
            .as_ref()
            .and_then(|m| crate::js_json::to_value(m).ok())
            .unwrap_or(JsValue::NULL)
    }
}

// StaticResource - Complete resource data with tiles
wasm_wrapper! {
    pub struct StaticResource wraps alizarin_core::StaticResource {
        no_constructor,
        no_copy,
    }
}

#[wasm_bindgen]
impl StaticResource {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Result<StaticResource, JsValue> {
        let mut core: CoreStaticResource = crate::js_json::from_value(data).map_err(|e| {
            JsValue::from_str(&format!("Failed to deserialize StaticResource: {:?}", e))
        })?;
        core.infer_tiles_loaded();
        Ok(StaticResource(core))
    }

    pub fn copy(&self) -> StaticResource {
        StaticResource(self.0.clone())
    }

    /// Parse resource from JSON string - faster than constructor with JS object
    /// because it avoids multiple string copies across WASM boundary
    #[wasm_bindgen(js_name = fromJsonString)]
    pub fn from_json_string(json_str: &str) -> Result<StaticResource, JsValue> {
        let mut core: CoreStaticResource = serde_json::from_str(json_str)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse resource JSON: {}", e)))?;
        core.infer_tiles_loaded();
        Ok(StaticResource(core))
    }

    /// Parse multiple resources from a JSON string containing a business_data wrapper
    /// This is much faster than parsing each resource individually
    #[wasm_bindgen(js_name = fromBusinessDataJsonString)]
    pub fn from_business_data_json_string(json_str: &str) -> Result<Vec<StaticResource>, JsValue> {
        let resources: Vec<StaticResource> =
            alizarin_core::resources_from_business_data_json(json_str)
                .map_err(|e| JsValue::from_str(&e))?
                .into_iter()
                .map(StaticResource)
                .collect();

        Ok(resources)
    }

    #[wasm_bindgen(js_name = fromSummary)]
    pub fn from_summary(summary: &StaticResourceSummary) -> StaticResource {
        StaticResource(CoreStaticResource::from_summary(&summary.0))
    }

    #[wasm_bindgen(getter = resourceinstance)]
    pub fn resourceinstance(&self) -> StaticResourceMetadata {
        StaticResourceMetadata(self.0.resourceinstance.clone())
    }

    #[wasm_bindgen(getter)]
    pub fn tiles(&self) -> JsValue {
        match &self.0.tiles {
            Some(tiles) => {
                // Return StaticTile WASM objects directly so that getters (like data -> Map) work
                let js_array = js_sys::Array::new();
                for tile in tiles {
                    let wrapper = StaticTile(tile.clone());
                    js_array.push(&wrapper.into());
                }
                js_array.into()
            }
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_tiles(&mut self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            self.0.tiles = None;
            self.0.tiles_loaded = Some(false);
        } else {
            match crate::js_json::from_value::<Vec<alizarin_core::StaticTile>>(value) {
                Ok(tiles) => {
                    self.0.tiles_loaded = Some(true);
                    self.0.tiles = Some(tiles);
                }
                Err(_) => {
                    self.0.tiles = None;
                    self.0.tiles_loaded = Some(false);
                }
            }
        }
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        crate::js_json::to_value(&self.0.metadata).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(setter)]
    pub fn set_metadata(&mut self, value: JsValue) {
        if let Ok(metadata) = crate::js_json::from_value(value) {
            self.0.metadata = metadata;
        }
    }

    #[wasm_bindgen(getter = __cache)]
    pub fn get_cache(&self) -> JsValue {
        match &self.0.cache {
            Some(json) => json_to_js_value(json),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = __cache)]
    pub fn set_cache(&mut self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            self.0.cache = None;
        } else {
            self.0.cache = crate::js_json::from_value(value).ok();
        }
    }

    #[wasm_bindgen(getter = __scopes)]
    pub fn get_scopes(&self) -> JsValue {
        match &self.0.scopes {
            Some(json) => json_to_js_value(json),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter = __scopes)]
    pub fn set_scopes(&mut self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            self.0.scopes = None;
        } else {
            self.0.scopes = crate::js_json::from_value(value).ok();
        }
    }

    #[wasm_bindgen(getter = tilesLoaded)]
    pub fn tiles_loaded(&self) -> bool {
        self.0.tiles_loaded.unwrap_or(false)
    }
}

// =============================================================================
// StaticResourceRegistry - Registry for relationship resolution
// =============================================================================

/// Registry of known resources for relationship resolution
///
/// Used to:
/// - Look up graph_id for referenced resources
/// - Populate __cache on resources with related resource summaries
/// - Enrich resource-instance tile data with ontologyProperty from node config
/// - Validate that referenced resources exist (in strict mode)
///
/// @example
/// ```typescript
/// const registry = new StaticResourceRegistry();
/// registry.mergeFromResourcesJson(existingResourcesJson);
///
/// // After conversion, populate caches
/// const result = registry.populateCaches(resourcesJson, graph, true);
/// if (result.hasUnknown) {
///   console.warn('Unknown references:', result.unknownReferences);
/// }
/// ```
#[wasm_bindgen]
pub struct StaticResourceRegistry(CoreStaticResourceRegistry);

impl StaticResourceRegistry {
    /// Get the inner core registry (for internal Rust use)
    pub(crate) fn inner(&self) -> &CoreStaticResourceRegistry {
        &self.0
    }
}

#[wasm_bindgen]
impl StaticResourceRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> StaticResourceRegistry {
        StaticResourceRegistry(CoreStaticResourceRegistry::new())
    }

    /// Number of resources in the registry
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.0.len()
    }

    /// Check if registry is empty
    #[wasm_bindgen(getter = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get diagnostic stats about registry contents (entry counts, tile counts, etc.)
    #[wasm_bindgen(js_name = memoryStats)]
    pub fn memory_stats(&self) -> Result<JsValue, JsValue> {
        let stats = self.0.memory_stats();
        crate::js_json::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    /// Get detailed stats including estimated byte sizes (expensive — re-serializes all data)
    #[wasm_bindgen(js_name = memoryStatsDetailed)]
    pub fn memory_stats_detailed(&self) -> Result<JsValue, JsValue> {
        let stats = self.0.memory_stats_detailed();
        crate::js_json::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    /// Check if a resource ID is known
    /// Returns false if resource_id is null/undefined
    pub fn contains(&self, resource_id: Option<String>) -> bool {
        match resource_id {
            Some(id) => self.0.contains(&id),
            None => false,
        }
    }

    /// Get the graph_id for a resource (or null if not known or resource_id is null)
    #[wasm_bindgen(js_name = getGraphId, unchecked_return_type = "string | null")]
    pub fn get_graph_id(&self, resource_id: Option<String>) -> JsValue {
        crate::js_json::or_null(
            resource_id.and_then(|id| self.0.get_graph_id(&id).map(|s| s.to_string())),
        )
    }

    /// Get the full summary for a resource (or null if not known or resource_id is null)
    #[wasm_bindgen(js_name = getSummary, unchecked_return_type = "StaticResourceSummary | null")]
    pub fn get_summary(&self, resource_id: Option<String>) -> JsValue {
        crate::js_json::or_null(resource_id.and_then(|id| {
            self.0
                .get_summary(&id)
                .map(|s| StaticResourceSummary(s.clone()))
        }))
    }

    /// Get the full resource if stored (or null if only summary, not known, or resource_id is null)
    #[wasm_bindgen(js_name = getFull, unchecked_return_type = "StaticResource | null")]
    pub fn get_full(&self, resource_id: Option<String>) -> JsValue {
        crate::js_json::or_null(
            resource_id.and_then(|id| self.0.get_full(&id).map(|r| StaticResource(r.clone()))),
        )
    }

    /// Run a resource's Aggregate functions, with this registry as the slice.
    ///
    /// The resource must be loaded in full here, and its graph and its members'
    /// graphs registered. Its Aggregate functions (e.g. the built-in fold) write
    /// node values into its tiles, which are updated in place; the resource
    /// itself is not part of the slice. Returns the node values that changed,
    /// `[{tileid, nodeid, value}]` (`[]` if nothing did), for a consumer caching
    /// them. Equivalent to NAPI's and Python's
    /// `applyAggregateFunctions` / `apply_aggregate_functions`.
    #[wasm_bindgen(js_name = applyAggregateFunctions)]
    pub fn apply_aggregate_functions(&mut self, target_id: String) -> Result<JsValue, JsValue> {
        let changed = alizarin_core::apply_aggregate_functions_in_registry(
            &mut self.0,
            &target_id,
            &crate::extension_registry::build_extension_registry(),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        crate::js_json::to_value(&changed).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Expand a resource's Aggregate node values with members added since the
    /// last call, without revisiting earlier ones (`addedIds` must already be
    /// loaded in full here). Call it when the node values are about to be
    /// consumed: additions compose, so the ids since the last call go in one
    /// batch. Add each member once; re-adding one counts it again. Errors if a
    /// provider can't add incrementally (run `applyAggregateFunctions` then).
    /// Returns the node values that changed, as `applyAggregateFunctions` does.
    #[wasm_bindgen(js_name = applyAggregateAdditions)]
    pub fn apply_aggregate_additions(
        &mut self,
        target_id: String,
        added_ids: Vec<String>,
    ) -> Result<JsValue, JsValue> {
        let changed = alizarin_core::apply_aggregate_additions_in_registry(
            &mut self.0,
            &target_id,
            &added_ids,
            &crate::extension_registry::build_extension_registry(),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        crate::js_json::to_value(&changed).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Add `count` to the pending count each of a resource's Aggregate functions
    /// declares (`pending` in its entry): members added but not yet applied, so
    /// a consumer can tell its node values are stale, and by how much. Runs and
    /// additions reset it to 0. Returns the node values that changed.
    /// Equivalent to NAPI's and Python's.
    #[wasm_bindgen(js_name = markAggregatePending)]
    pub fn mark_aggregate_pending(
        &mut self,
        target_id: String,
        count: u32,
    ) -> Result<JsValue, JsValue> {
        let changed = alizarin_core::mark_aggregate_pending_in_registry(
            &mut self.0,
            &target_id,
            count.into(),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        crate::js_json::to_value(&changed).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Check if a resource has full data stored (not just summary)
    /// Returns false if resource_id is null/undefined
    #[wasm_bindgen(js_name = hasFull)]
    pub fn has_full(&self, resource_id: Option<String>) -> bool {
        match resource_id {
            Some(id) => self.0.has_full(&id),
            None => false,
        }
    }

    /// Get all full resources as an array (for iteration)
    #[wasm_bindgen(js_name = getAllFull)]
    pub fn get_all_full(&self) -> Vec<StaticResource> {
        self.0
            .iter_full()
            .map(|(_, r)| StaticResource(r.clone()))
            .collect()
    }

    /// Get all full resources for a specific graph
    /// Returns empty array if graph_id is null/undefined
    #[wasm_bindgen(js_name = getAllFullForGraph)]
    pub fn get_all_full_for_graph(&self, graph_id: Option<String>) -> Vec<StaticResource> {
        match graph_id {
            Some(gid) => self
                .0
                .iter_full()
                .filter(|(_, r)| r.resourceinstance.graph_id == gid)
                .map(|(_, r)| StaticResource(r.clone()))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Add a single resource summary to the registry
    ///
    /// @param summaryJson - JSON string with {resourceinstanceid, graph_id, name, ...}
    #[wasm_bindgen(js_name = insertFromJson)]
    pub fn insert_from_json(&mut self, summary_json: &str) -> Result<(), JsValue> {
        let summary: CoreStaticResourceSummary = serde_json::from_str(summary_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse summary: {}", e)))?;
        self.0.insert(summary);
        Ok(())
    }

    /// Add a StaticResourceSummary to the registry
    pub fn insert(&mut self, summary: &StaticResourceSummary) {
        self.0.insert(summary.0.clone());
    }

    /// Merge resources into the registry from a JSON string
    ///
    /// Registers each resource's ID → summary mapping or full resources.
    /// If storeFull is true, stores full resources (for traversal).
    /// If storeFull is false (default), stores only summaries (memory efficient).
    /// If includeCaches is true (default), also merges any __cache.relatedResources as summaries.
    ///
    /// @param resourcesJson - JSON string containing array of StaticResource objects
    /// @param storeFull - If true, store full resources; if false, store only summaries
    /// @param includeCaches - If true, also merge related resources from __cache
    #[wasm_bindgen(js_name = mergeFromResourcesJson)]
    pub fn merge_from_resources_json(
        &mut self,
        resources_json: &str,
        store_full: Option<bool>,
        include_caches: Option<bool>,
    ) -> Result<(), JsValue> {
        let resources: Vec<CoreStaticResource> = serde_json::from_str(resources_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse resources: {}", e)))?;
        self.0.merge_from_resources(
            &resources,
            store_full.unwrap_or(false),
            include_caches.unwrap_or(true),
        );
        Ok(())
    }

    /// Merge resources into the registry from StaticResource array
    ///
    /// If storeFull is true, stores full resources (for traversal).
    /// If storeFull is false (default), stores only summaries (memory efficient).
    /// If includeCaches is true (default), also merges any __cache.relatedResources as summaries.
    ///
    /// @param resources - Array of StaticResource objects
    /// @param storeFull - If true, store full resources; if false, store only summaries
    /// @param includeCaches - If true, also merge related resources from __cache
    #[wasm_bindgen(js_name = mergeFromResources)]
    pub fn merge_from_resources(
        &mut self,
        resources: Vec<StaticResource>,
        store_full: Option<bool>,
        include_caches: Option<bool>,
    ) {
        let core_resources: Vec<CoreStaticResource> = resources.into_iter().map(|r| r.0).collect();
        self.0.merge_from_resources(
            &core_resources,
            store_full.unwrap_or(false),
            include_caches.unwrap_or(true),
        );
    }

    /// Load a business_data file from raw bytes, parse it entirely in Rust,
    /// and merge the resources into this registry. Returns a lightweight JS
    /// array of `{ resourceinstanceid, graph_id, isPublic }` objects for
    /// enumeration — no full resource data crosses to V8.
    ///
    /// Callers pass in the result of `fs.readFile(path)` (a Buffer / Uint8Array).
    ///
    /// @param bytes - Raw file content (business_data JSON as bytes)
    /// @param storeFull - If true, store full resources; if false, store only summaries
    /// @param includeCaches - If true, also merge related resources from __cache
    /// @returns Array of { resourceinstanceid, graph_id, isPublic } objects
    #[wasm_bindgen(js_name = loadFromBusinessDataBytes)]
    pub fn load_from_business_data_bytes(
        &mut self,
        bytes: &[u8],
        store_full: Option<bool>,
        include_caches: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let refs = self
            .0
            .load_from_business_data_bytes(
                bytes,
                store_full.unwrap_or(true),
                include_caches.unwrap_or(true),
            )
            .map_err(|e| JsValue::from_str(&format!("Failed to parse business data: {}", e)))?;
        crate::js_json::to_value(&refs)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize refs: {}", e)))
    }

    /// Populate __cache on resources with summaries for referenced resources
    ///
    /// Uses the graph to identify resource-instance nodes, then populates
    /// cache entries for each referenced resource found in this registry.
    ///
    /// If enrichRelationships is true, also adds ontologyProperty/inverseOntologyProperty
    /// to tile data based on node config and the target resource's graph.
    ///
    /// @param resourcesJson - JSON string containing array of StaticResource objects
    /// @param graph - StaticGraph to use for node lookup
    /// @param enrichRelationships - If true, add ontologyProperty to tile data
    /// @returns Object with resources (updated), unknownReferences, and hasUnknown flag
    #[wasm_bindgen(js_name = populateCachesFromJson)]
    pub fn populate_caches_from_json(
        &self,
        resources_json: &str,
        graph: &StaticGraph,
        enrich_relationships: Option<bool>,
        strict: Option<bool>,
        recompute_descriptors: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let output = self
            .0
            .populate_caches_from_json(
                resources_json,
                &graph.0,
                enrich_relationships.unwrap_or(true),
                strict.unwrap_or(false),
                recompute_descriptors.unwrap_or(false),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        crate::js_json::to_value(&output)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Populate __cache on resources with summaries for referenced resources
    ///
    /// @param resources - Array of StaticResource objects (modified in place)
    /// @param graph - StaticGraph to use for node lookup
    /// @param enrichRelationships - If true, add ontologyProperty to tile data
    /// @returns Object with unknownReferences array and hasUnknown flag
    #[wasm_bindgen(js_name = populateCaches)]
    pub fn populate_caches(
        &self,
        resources: Vec<StaticResource>,
        graph: &StaticGraph,
        enrich_relationships: Option<bool>,
        strict: Option<bool>,
        recompute_descriptors: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let mut core_resources: Vec<CoreStaticResource> =
            resources.into_iter().map(|r| r.0).collect();

        let result = self
            .0
            .populate_caches(
                &mut core_resources,
                &graph.0,
                enrich_relationships.unwrap_or(true),
                strict.unwrap_or(false),
                recompute_descriptors.unwrap_or(false),
            )
            .map_err(|e| JsValue::from_str(&e))?;

        let output = alizarin_core::PopulateCachesOutput {
            has_unknown: result.has_unknown_references(),
            unknown_references: result.unknown_references,
            resources: core_resources,
        };
        crate::js_json::to_value(&output)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Build an index from resource IDs to node values for a given node.
    ///
    /// Iterates through all full resources in the registry that match the graph,
    /// filtering tiles by the node's nodegroup_id, and collecting values for the node.
    ///
    /// @param graph - The graph definition containing the node
    /// @param nodeIdentifier - Node alias or node ID to look up
    /// @returns Object mapping resource IDs to arrays of values found in tiles
    ///
    /// @example
    /// ```typescript
    /// const index = registry.getNodeValuesIndex(graph, 'forename');
    /// // Returns: { "resource-uuid-1": ["Alice"], "resource-uuid-2": ["Bob", "Robert"] }
    /// ```
    #[wasm_bindgen(js_name = getNodeValuesIndex)]
    pub fn get_node_values_index(
        &self,
        graph: &StaticGraph,
        node_identifier: &str,
    ) -> Result<JsValue, JsValue> {
        let index = self
            .0
            .get_node_values_index(&graph.0, node_identifier)
            .map_err(|e| JsValue::from_str(&e))?;

        crate::js_json::to_value(&index)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Build an inverted index from node values to resource IDs.
    ///
    /// Iterates through all full resources in the registry that match the graph,
    /// collecting which resources have each value for the specified node.
    ///
    /// @param graph - The graph definition containing the node
    /// @param nodeIdentifier - Node alias or node ID to look up
    /// @param flattenLocalized - If true (default), extracts string from localized format {"en": "value"}
    /// @returns Object mapping string values to arrays of resource IDs
    ///
    /// @example
    /// ```typescript
    /// const index = registry.getValueToResourcesIndex(graph, 'forename', true);
    /// // Returns: { "Alice": ["resource-uuid-1"], "Bob": ["resource-uuid-2", "resource-uuid-3"] }
    /// ```
    #[wasm_bindgen(js_name = getValueToResourcesIndex)]
    pub fn get_value_to_resources_index(
        &self,
        graph: &StaticGraph,
        node_identifier: &str,
        flatten_localized: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        // Extension datatypes render through the JS-registered handlers, as
        // NAPI's through its registered ones.
        let ext_registry = crate::extension_registry::build_extension_registry();
        let ctx = alizarin_core::type_serialization::SerializationContext {
            extension_registry: Some(&ext_registry),
            ..alizarin_core::type_serialization::SerializationContext::empty()
        };
        let index = self
            .0
            .get_value_to_resources_index_with_context(
                &graph.0,
                node_identifier,
                flatten_localized.unwrap_or(true),
                Some(&ctx),
            )
            .map_err(|e| JsValue::from_str(&e))?;

        crate::js_json::to_value(&index)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Values of `extractNode` in the tiles whose `filterNode` display value
    /// contains any of `filterValues` (both nodes in one nodegroup), as NAPI's
    /// getFilteredTileValues. `requiredScope` keeps only resources with that
    /// `__scopes` entry.
    #[wasm_bindgen(js_name = getFilteredTileValues)]
    pub fn get_filtered_tile_values(
        &self,
        graph: &StaticGraph,
        filter_node: &str,
        filter_values: Vec<String>,
        extract_node: &str,
        flatten_localized: Option<bool>,
        required_scope: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let filter_refs: Vec<&str> = filter_values.iter().map(|s| s.as_str()).collect();
        let ext_registry = crate::extension_registry::build_extension_registry();
        let ctx = alizarin_core::type_serialization::SerializationContext {
            extension_registry: Some(&ext_registry),
            ..alizarin_core::type_serialization::SerializationContext::empty()
        };
        let values = self
            .0
            .get_filtered_tile_values(
                &graph.0,
                filter_node,
                &filter_refs,
                extract_node,
                flatten_localized.unwrap_or(true),
                Some(&ctx),
                required_scope.as_deref(),
            )
            .map_err(|e| JsValue::from_str(&e))?;
        crate::js_json::to_value(&values)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }
}

impl Default for StaticResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
