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
