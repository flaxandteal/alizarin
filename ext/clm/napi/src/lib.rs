//! NAPI binding for the Alizarin CLM (Controlled List Manager) extension.
//!
//! Exposes the `reference` handler's C-ABI [`TypeHandlerInfo`] pointer as a
//! `BigInt`. The JS side hands it to `@alizarin/napi`'s `registerExtensionHandler`
//! — the NAPI analogue of Python's `PyCapsule`. The coerce / display /
//! resolve-markers fn-pointers are then called directly Rust→Rust in-process.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// The CLM `reference` handler's `TypeHandlerInfo` pointer, as a `BigInt`. Pass to
/// `registerExtensionHandler` from `@alizarin/napi`.
#[napi(js_name = "referenceHandlerPtr")]
pub fn reference_handler_ptr() -> BigInt {
    let ptr = alizarin_clm_core::c_abi::reference_handler_type_info() as usize as u64;
    BigInt::from(ptr)
}

/// The `clm.reference_change_collection` mutation's `MutationHandlerInfo`
/// pointer, as a `BigInt`. Pass to `registerExtensionMutation` from
/// `@alizarin/napi`.
#[napi(js_name = "referenceChangeCollectionPtr")]
pub fn reference_change_collection_ptr() -> BigInt {
    let ptr = alizarin_clm_core::c_abi::reference_change_collection_info() as usize as u64;
    BigInt::from(ptr)
}

/// Set the base URI reference item URIs are built on, as Python's
/// `set_clm_base_uri` does. It lives in this module, beside the handler that
/// builds them.
#[napi(js_name = "setClmBaseUri")]
pub fn set_clm_base_uri(uri: String) {
    alizarin_clm_core::set_clm_base_uri(&uri);
}

/// The base URI set with `setClmBaseUri`, or null.
#[napi(js_name = "getClmBaseUri")]
pub fn get_clm_base_uri() -> Option<String> {
    alizarin_clm_core::get_clm_base_uri()
}

/// Clear the base URI, back to clm's default.
#[napi(js_name = "clearClmBaseUri")]
pub fn clear_clm_base_uri() {
    alizarin_clm_core::clear_clm_base_uri();
}
