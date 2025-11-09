/// Python bindings for Alizarin using PyO3
///
/// This provides Python bindings with identical semantics to the JavaScript/WASM version.
/// The core Rust logic is shared, only the binding layer differs.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use serde_json;

// Re-export core types we'll need
// TODO: Once alizarin-core is expanded, import from there
// For now, we'll define minimal versions here

/// Python-compatible version of StaticTranslatableString
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticTranslatableString {
    #[pyo3(get)]
    translations: HashMap<String, String>,
    #[pyo3(get, set)]
    lang: String,
}

#[pymethods]
impl StaticTranslatableString {
    #[new]
    fn new(value: &PyAny, lang: Option<String>) -> PyResult<Self> {
        let default_lang = lang.unwrap_or_else(|| "en".to_string());

        // Try to parse as dict (translations map)
        if let Ok(dict) = value.downcast::<PyDict>() {
            let mut translations = HashMap::new();
            for (key, val) in dict.iter() {
                let key_str: String = key.extract()?;
                let val_str: String = val.extract()?;
                translations.insert(key_str, val_str);
            }

            let current_lang = if translations.contains_key(&default_lang) {
                default_lang
            } else {
                translations.keys().next().cloned().unwrap_or_else(|| "en".to_string())
            };

            return Ok(StaticTranslatableString {
                translations,
                lang: current_lang,
            });
        }

        // Try as string
        if let Ok(s) = value.extract::<String>() {
            let mut translations = HashMap::new();
            translations.insert(default_lang.clone(), s);
            return Ok(StaticTranslatableString {
                translations,
                lang: default_lang,
            });
        }

        // Empty string fallback
        let mut translations = HashMap::new();
        translations.insert(default_lang.clone(), String::new());
        Ok(StaticTranslatableString {
            translations,
            lang: default_lang,
        })
    }

    fn __str__(&self) -> String {
        self.translations.get(&self.lang)
            .or_else(|| self.translations.values().next())
            .cloned()
            .unwrap_or_default()
    }

    fn __repr__(&self) -> String {
        format!("StaticTranslatableString('{}')", self.__str__())
    }

    fn __eq__(&self, other: &PyAny) -> PyResult<bool> {
        // Support comparison with strings
        if let Ok(s) = other.extract::<String>() {
            return Ok(self.__str__() == s);
        }

        // Support comparison with other StaticTranslatableString
        if let Ok(other_sts) = other.extract::<StaticTranslatableString>() {
            return Ok(self.__str__() == other_sts.__str__());
        }

        Ok(false)
    }

    fn __ne__(&self, other: &PyAny) -> PyResult<bool> {
        Ok(!self.__eq__(other)?)
    }

    /// Get translation in specific language
    fn lang(&self, language: &str) -> Option<String> {
        self.translations.get(language).cloned()
    }
}

/// Python-compatible version of StaticEdge
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticEdge {
    #[pyo3(get, set)]
    pub domainnode_id: String,

    #[pyo3(get, set)]
    pub rangenode_id: String,

    #[pyo3(get, set)]
    pub edgeid: String,

    #[pyo3(get, set)]
    pub graph_id: String,
}

#[pymethods]
impl StaticEdge {
    #[new]
    fn new(json_str: &str) -> PyResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!("StaticEdge({} -> {})", self.domainnode_id, self.rangenode_id)
    }
}

/// Python-compatible version of StaticGraph (complete graph model)
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticGraph {
    #[pyo3(get, set)]
    pub graphid: String,

    #[pyo3(get, set)]
    pub name: StaticTranslatableString,

    #[pyo3(get, set)]
    pub description: StaticTranslatableString,

    #[pyo3(get, set)]
    pub subtitle: StaticTranslatableString,

    #[pyo3(get, set)]
    pub author: String,

    #[pyo3(get, set)]
    pub isresource: bool,

    // Core graph structure
    nodes: Vec<StaticNode>,
    nodegroups: Vec<StaticNodegroup>,
    edges: Vec<StaticEdge>,
    root: StaticNode,
}

#[pymethods]
impl StaticGraph {
    #[new]
    fn new(json_str: &str) -> PyResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))
    }

    /// Get all nodes as Python list
    fn get_nodes(&self, py: Python) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for node in &self.nodes {
            list.append(Py::new(py, node.clone())?)?;
        }
        Ok(list.into())
    }

    /// Get all nodegroups as Python list
    fn get_nodegroups(&self, py: Python) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for ng in &self.nodegroups {
            list.append(Py::new(py, ng.clone())?)?;
        }
        Ok(list.into())
    }

    /// Get all edges as Python list
    fn get_edges(&self, py: Python) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for edge in &self.edges {
            list.append(Py::new(py, edge.clone())?)?;
        }
        Ok(list.into())
    }

    /// Get root node
    fn get_root(&self, py: Python) -> PyResult<PyObject> {
        Py::new(py, self.root.clone()).map(|p| p.into())
    }

    /// Find node by ID
    fn get_node_by_id(&self, node_id: &str, py: Python) -> PyResult<Option<PyObject>> {
        for node in &self.nodes {
            if node.nodeid == node_id {
                return Ok(Some(Py::new(py, node.clone())?.into()));
            }
        }
        Ok(None)
    }

    /// Find node by alias
    fn get_node_by_alias(&self, alias: &str, py: Python) -> PyResult<Option<PyObject>> {
        for node in &self.nodes {
            if let Some(ref node_alias) = node.alias {
                if node_alias == alias {
                    return Ok(Some(Py::new(py, node.clone())?.into()));
                }
            }
        }
        Ok(None)
    }

    /// Get child nodes of a given node (following edges)
    fn get_child_nodes(&self, node_id: &str, py: Python) -> PyResult<PyObject> {
        let list = PyList::empty(py);

        // Find edges where this node is the domain (parent)
        for edge in &self.edges {
            if edge.domainnode_id == node_id {
                // Find the range (child) node
                if let Some(child) = self.get_node_by_id(&edge.rangenode_id, py)? {
                    list.append(child)?;
                }
            }
        }

        Ok(list.into())
    }

    fn __repr__(&self) -> String {
        format!("StaticGraph(graphid='{}', name='{}', nodes={}, nodegroups={})",
                self.graphid, self.name.__str__(), self.nodes.len(), self.nodegroups.len())
    }
}

/// Python-compatible version of StaticNode
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticNode {
    #[pyo3(get, set)]
    pub nodeid: String,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,

    #[pyo3(get, set)]
    pub datatype: String,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ontologyclass: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodegroup_id: Option<String>,

    #[pyo3(get, set)]
    pub graph_id: String,
}

#[pymethods]
impl StaticNode {
    #[new]
    fn new(json_str: &str) -> PyResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!("StaticNode(nodeid='{}', alias={:?})", self.nodeid, self.alias)
    }
}

/// Python-compatible version of StaticNodegroup
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticNodegroup {
    #[pyo3(get, set)]
    pub nodegroupid: String,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardinality: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentnodegroup_id: Option<String>,
}

#[pymethods]
impl StaticNodegroup {
    #[new]
    fn new(json_str: &str) -> PyResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!("StaticNodegroup(nodegroupid='{}')", self.nodegroupid)
    }
}

/// Python-compatible version of StaticTile
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticTile {
    #[pyo3(get, set)]
    pub nodegroup_id: String,

    #[pyo3(get, set)]
    pub resourceinstance_id: String,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tileid: Option<String>,

    #[pyo3(get, set)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parenttile_id: Option<String>,

    // Data is stored as JSON dict
    #[serde(default)]
    data: HashMap<String, serde_json::Value>,
}

#[pymethods]
impl StaticTile {
    #[new]
    fn new(json_str: &str) -> PyResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))
    }

    /// Get tile data as Python dict
    fn get_data(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        for (key, value) in &self.data {
            let py_value = serde_json_to_py(py, value)?;
            dict.set_item(key, py_value)?;
        }
        Ok(dict.into())
    }

    /// Set tile data from Python dict
    fn set_data(&mut self, data: &PyDict) -> PyResult<()> {
        self.data.clear();
        for (key, value) in data.iter() {
            let key_str: String = key.extract()?;
            let json_value = py_to_serde_json(value)?;
            self.data.insert(key_str, json_value);
        }
        Ok(())
    }

    fn ensure_id(&mut self) -> String {
        if self.tileid.is_none() {
            self.tileid = Some(uuid::Uuid::new_v4().to_string());
        }
        self.tileid.clone().unwrap()
    }

    fn __repr__(&self) -> String {
        format!("StaticTile(tileid={:?}, nodegroup_id='{}')", self.tileid, self.nodegroup_id)
    }
}

/// Python-compatible version of StaticResource
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResource {
    #[pyo3(get, set)]
    pub graph_id: String,

    #[pyo3(get, set)]
    pub resourceinstanceid: String,

    #[serde(default)]
    tiles: Vec<StaticTile>,
}

#[pymethods]
impl StaticResource {
    #[new]
    fn new(json_str: &str) -> PyResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))
    }

    /// Get tiles as Python list
    fn get_tiles(&self, py: Python) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for tile in &self.tiles {
            list.append(Py::new(py, tile.clone())?)?;
        }
        Ok(list.into())
    }

    /// Set tiles from Python list
    fn set_tiles(&mut self, tiles: &PyList) -> PyResult<()> {
        self.tiles.clear();
        for item in tiles.iter() {
            let tile: StaticTile = item.extract()?;
            self.tiles.push(tile);
        }
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("StaticResource(resourceinstanceid='{}', tiles={})",
                self.resourceinstanceid, self.tiles.len())
    }
}

/// Helper to convert serde_json::Value to Python object
fn serde_json_to_py(py: Python, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.to_object(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Ok(n.to_string().to_object(py))
            }
        }
        serde_json::Value::String(s) => Ok(s.to_object(py)),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(serde_json_to_py(py, item)?)?;
            }
            Ok(list.into())
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (key, val) in obj {
                dict.set_item(key, serde_json_to_py(py, val)?)?;
            }
            Ok(dict.into())
        }
    }
}

/// Helper to convert Python object to serde_json::Value
fn py_to_serde_json(obj: &PyAny) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        return Ok(serde_json::Value::Null);
    }

    if let Ok(b) = obj.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }

    if let Ok(i) = obj.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }

    if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(serde_json::Value::Number(n));
        }
    }

    if let Ok(s) = obj.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }

    if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_to_serde_json(item)?);
        }
        return Ok(serde_json::Value::Array(arr));
    }

    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str: String = key.extract()?;
            map.insert(key_str, py_to_serde_json(value)?);
        }
        return Ok(serde_json::Value::Object(map));
    }

    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        format!("Cannot convert type {} to JSON", obj.get_type().name()?)
    ))
}

/// Convert a tiled resource to nested JSON structure
///
/// This is the Python equivalent of ResourceInstanceViewModel.forJson()
///
/// Args:
///     resource: StaticResource with tiles
///     graph: StaticGraph with complete node/nodegroup/edge structure
///
/// Returns:
///     Nested dict structure representing the resource
#[pyfunction]
fn resource_to_json(
    py: Python,
    resource: &StaticResource,
    graph: &StaticGraph,
) -> PyResult<PyObject> {
    // For now, return a placeholder
    // TODO: Implement the actual logic using Rust's existing code
    let dict = PyDict::new(py);
    dict.set_item("resourceinstanceid", &resource.resourceinstanceid)?;
    dict.set_item("graph_id", &resource.graph_id)?;

    // TODO: Walk through tiles and build nested structure based on graph nodes/edges
    // 1. Start from root node
    // 2. For each child node (via edges):
    //    - Find corresponding tiles by nodegroup_id
    //    - If nodegroup cardinality='n', create array
    //    - If cardinality='1', create single value
    //    - Extract value from tile.data[node.nodeid]
    //    - Recursively process child nodes
    // 3. Use node.alias as dict key

    Ok(dict.into())
}

/// Convert nested JSON structure to a tiled resource
///
/// This is the inverse of resource_to_json()
///
/// Args:
///     json_dict: Nested dict structure
///     graph: StaticGraph with complete node/nodegroup/edge structure
///
/// Returns:
///     StaticResource with tiles
#[pyfunction]
fn json_to_resource(
    py: Python,
    json_dict: &PyDict,
    graph: &StaticGraph,
) -> PyResult<StaticResource> {
    // For now, return a minimal resource
    // TODO: Implement the actual logic

    let resource_id = json_dict
        .get_item("resourceinstanceid")?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing resourceinstanceid"))?
        .extract::<String>()?;

    let graph_id = json_dict
        .get_item("graph_id")?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing graph_id"))?
        .extract::<String>()?;

    // TODO: Implement actual conversion
    // 1. Walk JSON dict structure
    // 2. For each key (node alias):
    //    - Find node by alias
    //    - Determine nodegroup from node.nodegroup_id
    //    - Create tile for that nodegroup
    //    - If value is array, create multiple tiles
    //    - If value is object, recursively process child fields
    //    - Set tile.data[node.nodeid] = value

    Ok(StaticResource {
        graph_id,
        resourceinstanceid: resource_id,
        tiles: Vec::new(),
    })
}

/// Python module definition
#[pymodule]
fn alizarin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<StaticTranslatableString>()?;
    m.add_class::<StaticGraph>()?;
    m.add_class::<StaticEdge>()?;
    m.add_class::<StaticNode>()?;
    m.add_class::<StaticNodegroup>()?;
    m.add_class::<StaticTile>()?;
    m.add_class::<StaticResource>()?;

    // Add conversion functions
    m.add_function(wrap_pyfunction!(resource_to_json, m)?)?;
    m.add_function(wrap_pyfunction!(json_to_resource, m)?)?;

    Ok(())
}
