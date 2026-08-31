//! NAPI binding for the Alizarin file-list extension.
//!
//! Exposes the `file-list` handler's C-ABI [`TypeHandlerInfo`] pointer as a
//! `BigInt`. The JS side hands it to `@alizarin/napi`'s `registerExtensionHandler`
//! — the NAPI analogue of Python's `PyCapsule`. The coerce / display fn-pointers
//! are then called directly Rust→Rust in-process.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// The file-list handler's `TypeHandlerInfo` pointer, as a `BigInt`. Pass to
/// `registerExtensionHandler` from `@alizarin/napi`.
#[napi(js_name = "fileListHandlerPtr")]
pub fn file_list_handler_ptr() -> BigInt {
    let ptr = alizarin_filelist_core::c_abi::filelist_handler_type_info() as usize as u64;
    BigInt::from(ptr)
}
