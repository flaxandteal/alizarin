/// Python bindings for Alizarin using PyO3
///
/// This demonstrates how the same core logic can be exposed to Python
/// with minimal binding code, just like the WASM version.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use alizarin_core::{PseudoNodeCore, PseudoNodeBuilder, NodeLike};
use serde::{Serialize, Deserialize};

/// Python-compatible version of StaticNode
/// In practice, you'd share this with the WASM version via alizarin-core
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticNode {
    #[pyo3(get, set)]
    pub nodeid: String,
    #[pyo3(get, set)]
    pub alias: Option<String>,
    #[pyo3(get, set)]
    pub datatype: String,
    #[pyo3(get, set)]
    pub name: Option<String>,
    #[pyo3(get, set)]
    pub ontologyclass: Option<String>,
}

#[pymethods]
impl StaticNode {
    #[new]
    fn new(
        nodeid: String,
        alias: Option<String>,
        datatype: String,
        name: Option<String>,
        ontologyclass: Option<String>,
    ) -> Self {
        StaticNode {
            nodeid,
            alias,
            datatype,
            name,
            ontologyclass,
        }
    }
}

// Implement NodeLike for StaticNode
impl NodeLike for StaticNode {
    fn get_alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    fn get_datatype(&self) -> Option<&str> {
        Some(&self.datatype)
    }
}

/// Python wrapper for PseudoNode
///
/// Key differences from WASM version:
/// 1. Uses Py<PseudoNode> for circular references (Python's native ref counting)
/// 2. No need for JsValue - Python objects work directly
/// 3. Can use actual Python dicts instead of Maps
/// 4. Much cleaner circular reference handling!
#[pyclass]
pub struct PseudoNode {
    // The core logic (same as WASM)
    core: Rc<RefCell<PseudoNodeCore<StaticNode>>>,

    // Python-specific: can use actual Python objects for circular refs!
    parent_node: Option<Py<PseudoNode>>,
    inner: Option<Py<PseudoNode>>,
}

#[pymethods]
impl PseudoNode {
    #[new]
    fn new(
        py: Python,
        node: Py<StaticNode>,
        child_nodes: Option<&PyDict>,
        inner: Option<PyObject>,
    ) -> PyResult<Self> {
        // Convert Python node to Rust StaticNode
        let static_node = node.borrow(py).clone();

        // Convert child_nodes dict to HashMap
        let mut child_map = HashMap::new();
        if let Some(children) = child_nodes {
            for (key, value) in children.iter() {
                let key_str: String = key.extract()?;
                let child_node: StaticNode = value.extract()?;
                child_map.insert(key_str, child_node);
            }
        }

        // Determine inner flag
        let is_inner = if let Some(inner_obj) = &inner {
            // Check if it's a boolean True
            inner_obj.extract::<bool>(py).unwrap_or(false)
        } else {
            false
        };

        // Build the core using the builder
        let core = if is_inner {
            PseudoNodeBuilder::new(static_node, child_map)
                .with_inner_flag(true)
                .build()
        } else {
            PseudoNodeBuilder::new(static_node, child_map)
                .build()
        };

        // Handle inner pseudo if it's a PseudoNode instance
        let inner_pseudo = if let Some(inner_obj) = &inner {
            if let Ok(pseudo) = inner_obj.extract::<Py<PseudoNode>>(py) {
                Some(pseudo)
            } else {
                None
            }
        } else {
            None
        };

        Ok(PseudoNode {
            core: Rc::new(RefCell::new(core)),
            parent_node: None,
            inner: inner_pseudo,
        })
    }

    /// Check if this node is iterable
    fn is_iterable(&self) -> bool {
        self.core.borrow().is_iterable()
    }

    /// Get the node placeholder string
    fn get_node_placeholder(&self) -> String {
        self.core.borrow().get_node_placeholder()
    }

    // Python properties
    #[getter]
    fn node(&self, py: Python) -> PyResult<Py<StaticNode>> {
        let node = self.core.borrow().node.clone();
        Py::new(py, node)
    }

    #[setter]
    fn set_node(&mut self, node: Py<StaticNode>, py: Python) -> PyResult<()> {
        self.core.borrow_mut().node = node.borrow(py).clone();
        Ok(())
    }

    #[getter]
    fn parent_node(&self) -> Option<Py<PseudoNode>> {
        self.parent_node.clone()
    }

    #[setter]
    fn set_parent_node(&mut self, parent: Option<Py<PseudoNode>>) {
        self.parent_node = parent;
    }

    #[getter]
    fn datatype(&self) -> Option<String> {
        self.core.borrow().datatype.clone()
    }

    #[setter]
    fn set_datatype(&mut self, dt: Option<String>) {
        self.core.borrow_mut().datatype = dt;
    }

    #[getter]
    fn child_nodes(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (key, node) in &self.core.borrow().child_nodes {
            dict.set_item(key, Py::new(py, node.clone())?)?;
        }
        Ok(dict.into())
    }

    #[setter]
    fn set_child_nodes(&mut self, children: &PyDict, py: Python) -> PyResult<()> {
        let mut child_map = HashMap::new();
        for (key, value) in children.iter() {
            let key_str: String = key.extract()?;
            let child_node: StaticNode = value.extract()?;
            child_map.insert(key_str, child_node);
        }
        self.core.borrow_mut().child_nodes = child_map;
        Ok(())
    }

    #[getter]
    fn is_outer(&self) -> bool {
        self.core.borrow().is_outer
    }

    #[setter]
    fn set_is_outer(&mut self, value: bool) {
        self.core.borrow_mut().is_outer = value;
    }

    #[getter]
    fn is_inner(&self) -> bool {
        self.core.borrow().is_inner
    }

    #[setter]
    fn set_is_inner(&mut self, value: bool) {
        self.core.borrow_mut().is_inner = value;
    }

    #[getter]
    fn inner(&self) -> Option<Py<PseudoNode>> {
        self.inner.clone()
    }

    #[setter]
    fn set_inner(&mut self, inner: Option<Py<PseudoNode>>) {
        self.inner = inner;
    }

    /// Python-friendly repr
    fn __repr__(&self) -> String {
        format!(
            "PseudoNode(placeholder='{}', is_iterable={})",
            self.get_node_placeholder(),
            self.is_iterable()
        )
    }
}

/// Python module definition
#[pymodule]
fn alizarin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<StaticNode>()?;
    m.add_class::<PseudoNode>()?;
    Ok(())
}

/// This demonstrates the key advantages of using PyO3:
///
/// 1. **Cleaner Circular References**: Python natively supports circular refs via reference counting
///    - No need for JsValue workarounds
///    - Can use Py<PseudoNode> directly for parent_node and inner
///
/// 2. **Native Python Objects**: Work with PyDict, not Maps
///    - More Pythonic API
///    - Better integration with Python code
///
/// 3. **Same Core Logic**: Both WASM and Python use PseudoNodeCore
///    - All business logic is tested once
///    - Bugs fixed in one place
///    - Easy to add new platforms
///
/// 4. **Type Safety**: PyO3 provides compile-time type checking
///    - Python type hints work naturally
///    - Runtime errors become compile errors
///
/// Example Python usage:
/// ```python
/// import alizarin
///
/// # Create a node
/// node = alizarin.StaticNode(
///     nodeid="123",
///     alias="my_field",
///     datatype="concept-list",
///     name="My Field",
///     ontologyclass="http://example.org/MyClass"
/// )
///
/// # Create a pseudo node
/// pseudo = alizarin.PseudoNode(node, {}, None)
///
/// # Use it
/// print(pseudo.is_iterable())  # True (because datatype is concept-list)
/// print(pseudo.get_node_placeholder())  # .my_field[*]
/// ```
