/// Alizarin Core Library
///
/// Platform-agnostic core functionality that can be used from:
/// - JavaScript/TypeScript via WASM bindings (alizarin-wasm)
/// - Python via PyO3 bindings (alizarin-python)
/// - Other languages via C FFI

pub mod pseudo_node;
pub mod pseudo_node_lite;

pub use pseudo_node::{PseudoNodeCore, PseudoNodeBuilder, NodeLike};
pub use pseudo_node_lite::{PseudoNodeState, build_placeholder, ITERABLE_DATATYPES};
