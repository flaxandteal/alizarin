//! A graph node as a navigable **pseudo node**, for walking a model's structure
//! without tiles (`NodeViewModel` and `getRoot` on every binding).
//!
//! It holds the node and its children by alias. A non-semantic node with
//! children is split as pseudo values are: the *outer* is the value node and
//! has no children of its own; its *inner* is the same node, read as
//! `semantic`, holding them. A node made as a child records its parent's
//! placeholder, so [`PseudoNodeCore::placeholder`] gives the whole path
//! (`.group.basic_info`, with `[*]` after a list node) on every binding.

use std::sync::Arc;

use crate::instance_wrapper_core::{ChildNodes, ModelAccess};
use crate::StaticNode;

/// Datatypes whose node holds a list, marked `[*]` in a placeholder.
const ITERABLE_DATATYPES: &[&str] = &[
    "concept-list",
    "resource-instance-list",
    "domain-value-list",
];

/// A navigable node: see the module docs.
#[derive(Clone, Debug)]
pub struct PseudoNodeCore {
    pub node: Arc<StaticNode>,
    /// Children by alias, shared with the model (empty on an outer node: its
    /// inner holds them).
    pub child_nodes: Arc<ChildNodes>,
    /// Whether this is an inner node, read as `semantic`.
    pub is_inner: bool,
    /// An outer node's inner.
    pub inner: Option<Arc<PseudoNodeCore>>,
    /// The parent's placeholder, when this node was made as a child.
    parent_placeholder: Option<String>,
}

impl PseudoNodeCore {
    /// The pseudo node for `node` with its children, split into outer and
    /// inner when it isn't semantic and has children.
    pub fn new(node: Arc<StaticNode>, child_nodes: Arc<ChildNodes>) -> Self {
        Self::with_parent(node, child_nodes, None)
    }

    fn with_parent(
        node: Arc<StaticNode>,
        child_nodes: Arc<ChildNodes>,
        parent_placeholder: Option<String>,
    ) -> Self {
        if node.datatype != "semantic" && !child_nodes.is_empty() {
            let inner = PseudoNodeCore {
                node: Arc::clone(&node),
                child_nodes,
                is_inner: true,
                inner: None,
                parent_placeholder: parent_placeholder.clone(),
            };
            PseudoNodeCore {
                node,
                child_nodes: Arc::default(),
                is_inner: false,
                inner: Some(Arc::new(inner)),
                parent_placeholder,
            }
        } else {
            PseudoNodeCore {
                node,
                child_nodes,
                is_inner: false,
                inner: None,
                parent_placeholder,
            }
        }
    }

    /// The model's root node.
    pub fn root(model: &dyn ModelAccess) -> Result<Self, String> {
        let node = model.get_root_node()?;
        Self::for_node(node, None, model)
    }

    /// The node with this alias, as a starting point (no parent).
    pub fn by_alias(alias: &str, model: &dyn ModelAccess) -> Result<Self, String> {
        let node = model
            .get_node_by_alias(alias)
            .ok_or_else(|| format!("Could not find node with alias: {alias}"))?;
        Self::for_node(node, None, model)
    }

    /// This node's child `alias`, which records this node as its parent.
    pub fn child(&self, alias: &str, model: &dyn ModelAccess) -> Result<Self, String> {
        let node = self.child_nodes.get(alias).cloned().ok_or_else(|| {
            format!(
                "This parent node does not have this child: {} {alias}",
                self.node.nodeid
            )
        })?;
        Self::for_node(node, Some(self.placeholder()), model)
    }

    fn for_node(
        node: Arc<StaticNode>,
        parent_placeholder: Option<String>,
        model: &dyn ModelAccess,
    ) -> Result<Self, String> {
        let child_nodes = model.get_child_nodes(&node.nodeid)?;
        Ok(Self::with_parent(node, child_nodes, parent_placeholder))
    }

    /// Whether the node holds a list.
    pub fn is_iterable(&self) -> bool {
        ITERABLE_DATATYPES.contains(&self.node.datatype.as_str())
    }

    /// The node's datatype, read as `semantic` on an inner node.
    pub fn datatype(&self) -> &str {
        if self.is_inner {
            "semantic"
        } else {
            &self.node.datatype
        }
    }

    /// Whether this is an outer node (it has an inner).
    pub fn is_outer(&self) -> bool {
        self.inner.is_some()
    }

    /// The path from where navigation started: the parent's placeholder, then
    /// `.` and this node's alias, then `[*]` if it holds a list.
    pub fn placeholder(&self) -> String {
        let mut placeholder = self.parent_placeholder.clone().unwrap_or_default();
        if let Some(alias) = &self.node.alias {
            placeholder.push('.');
            placeholder.push_str(alias);
        }
        if self.is_iterable() {
            placeholder.push_str("[*]");
        }
        placeholder
    }
}
