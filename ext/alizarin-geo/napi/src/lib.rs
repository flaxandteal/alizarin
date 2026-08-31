//! NAPI binding for the Alizarin geo extension.
//!
//! Exposes the geo validate handler's C-ABI [`TypeHandlerInfo`] pointer as a
//! `BigInt`. The JS side hands it to `@alizarin/napi`'s `registerExtensionHandler`
//! — the NAPI analogue of Python's `PyCapsule`. The fn-pointers are then called
//! directly Rust→Rust in-process, so no per-value serialization is needed.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// The geo handler's `TypeHandlerInfo` pointer, as a `BigInt`. Pass to
/// `registerExtensionHandler` from `@alizarin/napi`.
#[napi(js_name = "geoHandlerPtr")]
pub fn geo_handler_ptr() -> BigInt {
    let ptr = alizarin_geo_core::c_abi::geo_handler_type_info() as usize as u64;
    BigInt::from(ptr)
}

/// Set the max coordinate count per feature collection (mirrors the Python knob).
#[napi(js_name = "setCoordLimit")]
pub fn set_coord_limit(limit: Option<u32>) {
    alizarin_geo_core::set_coord_limit(limit.map(|n| n as usize));
}
