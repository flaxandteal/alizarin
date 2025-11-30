//! WASM wrapper types generated using the wasm_wrapper! macro.
//!
//! This module demonstrates how the proc-macro can replace boilerplate.
//! Compare this to the manual implementations in graph.rs.

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_wrapper_derive::wasm_wrapper;

// Example: This single declaration replaces ~50 lines of boilerplate
//
// What it generates:
// - #[wasm_bindgen] #[repr(transparent)] struct with the inner core type
// - Deref/DerefMut impls
// - From impls (both directions)
// - Serialize/Deserialize delegation
// - constructor, copy(), toJSON() methods
// - Getters/setters for specified fields

// Minimal example - just the type with standard methods
wasm_wrapper! {
    pub struct ExampleNodegroupWrapper wraps alizarin_core::StaticNodegroup {
        get nodegroupid,
        get set cardinality,
        get set parentnodegroup_id,
    }
}

// The above expands to approximately:
//
// #[wasm_bindgen]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct ExampleNodegroupWrapper(#[wasm_bindgen(skip)] pub alizarin_core::StaticNodegroup);
//
// impl Deref for ExampleNodegroupWrapper { ... }
// impl DerefMut for ExampleNodegroupWrapper { ... }
// impl From<alizarin_core::StaticNodegroup> for ExampleNodegroupWrapper { ... }
// impl From<ExampleNodegroupWrapper> for alizarin_core::StaticNodegroup { ... }
// impl Serialize for ExampleNodegroupWrapper { ... }
// impl Deserialize for ExampleNodegroupWrapper { ... }
//
// #[wasm_bindgen]
// impl ExampleNodegroupWrapper {
//     #[wasm_bindgen(constructor)]
//     pub fn new(json_data: JsValue) -> Result<Self, JsValue> { ... }
//     pub fn copy(&self) -> Self { ... }
//     #[wasm_bindgen(js_name = toJSON)]
//     pub fn to_json(&self) -> JsValue { ... }
//     #[wasm_bindgen(getter = nodegroupid)]
//     pub fn get_nodegroupid(&self) -> JsValue { ... }
//     // ... more getters/setters ...
// }

// Another example with different fields
wasm_wrapper! {
    pub struct ExampleEdgeWrapper wraps alizarin_core::StaticEdge {
        get edgeid,
        get domainnode_id,
        get rangenode_id,
        get name,
        get ontologyproperty,
    }
}

// For types that need custom methods beyond the generated ones,
// you can add an additional impl block:
//
// #[wasm_bindgen]
// impl ExampleEdgeWrapper {
//     #[wasm_bindgen(js_name = customMethod)]
//     pub fn custom_method(&self) -> String {
//         // Custom logic here
//     }
// }
