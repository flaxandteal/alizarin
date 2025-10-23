/// Core PseudoNode implementation - pure Rust, no platform-specific bindings
///
/// This is the platform-agnostic core that can be wrapped by:
/// - WASM bindings (for JavaScript/TypeScript)
/// - PyO3 bindings (for Python)
/// - C FFI (for other languages)

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

// Constants for iterable datatypes
const ITERABLE_DATATYPES: &[&str] = &[
    "concept-list",
    "resource-instance-list",
    "domain-value-list"
];

/// Core PseudoNode that contains all the business logic
/// Uses Rc<RefCell<>> for circular references (Python/JavaScript wrappers handle this differently)
#[derive(Clone, Debug)]
pub struct PseudoNodeCore<TNode: Clone> {
    /// The wrapped node
    pub node: TNode,
    /// Parent node (can be circular)
    pub parent_node: Option<Rc<RefCell<PseudoNodeCore<TNode>>>>,
    /// Cached datatype for quick access
    pub datatype: Option<String>,
    /// Child nodes by alias
    pub child_nodes: HashMap<String, TNode>,
    /// Whether this is an outer node (has inner pseudo)
    pub is_outer: bool,
    /// Whether this is an inner node (semantic wrapper)
    pub is_inner: bool,
    /// Inner pseudo node (for semantic wrapping)
    pub inner: Option<Rc<RefCell<PseudoNodeCore<TNode>>>>,
}

impl<TNode: Clone> PseudoNodeCore<TNode> {
    /// Create a new PseudoNodeCore
    pub fn new(
        node: TNode,
        child_nodes: HashMap<String, TNode>,
        datatype: Option<String>,
        is_inner: bool,
    ) -> Self {
        let is_outer = false;
        let inner = None;

        PseudoNodeCore {
            node,
            parent_node: None,
            datatype,
            child_nodes,
            is_outer,
            is_inner,
            inner,
        }
    }

    /// Create a new PseudoNodeCore with an inner pseudo
    pub fn new_with_inner(
        node: TNode,
        datatype: Option<String>,
        inner: Rc<RefCell<PseudoNodeCore<TNode>>>,
    ) -> Self {
        PseudoNodeCore {
            node,
            parent_node: None,
            datatype,
            child_nodes: HashMap::new(),
            is_outer: true,
            is_inner: false,
            inner: Some(inner),
        }
    }

    /// Check if this node represents an iterable datatype
    pub fn is_iterable(&self) -> bool {
        if let Some(ref dt) = self.datatype {
            ITERABLE_DATATYPES.contains(&dt.as_str())
        } else {
            false
        }
    }

    /// Set the parent node
    pub fn set_parent(&mut self, parent: Rc<RefCell<PseudoNodeCore<TNode>>>) {
        self.parent_node = Some(parent);
    }

    /// Set the inner pseudo node
    pub fn set_inner(&mut self, inner: Rc<RefCell<PseudoNodeCore<TNode>>>) {
        self.inner = Some(inner);
        self.is_outer = true;
    }
}

/// Trait that nodes must implement to work with PseudoNode
pub trait NodeLike {
    fn get_alias(&self) -> Option<&str>;
    fn get_datatype(&self) -> Option<&str>;
}

/// Builder for creating PseudoNodes with proper logic
pub struct PseudoNodeBuilder<TNode: Clone + NodeLike> {
    node: TNode,
    child_nodes: HashMap<String, TNode>,
    inner_as_bool: bool,
    inner_as_pseudo: Option<Rc<RefCell<PseudoNodeCore<TNode>>>>,
}

impl<TNode: Clone + NodeLike> PseudoNodeBuilder<TNode> {
    pub fn new(node: TNode, child_nodes: HashMap<String, TNode>) -> Self {
        PseudoNodeBuilder {
            node,
            child_nodes,
            inner_as_bool: false,
            inner_as_pseudo: None,
        }
    }

    pub fn with_inner_flag(mut self, inner: bool) -> Self {
        self.inner_as_bool = inner;
        self
    }

    pub fn with_inner_pseudo(mut self, inner: Rc<RefCell<PseudoNodeCore<TNode>>>) -> Self {
        self.inner_as_pseudo = Some(inner);
        self
    }

    pub fn build(self) -> PseudoNodeCore<TNode> {
        let datatype = self.node.get_datatype().map(|s| s.to_string());

        // Handle the inner parameter logic
        if let Some(inner_pseudo) = self.inner_as_pseudo {
            // It's a PseudoNode instance (outer node)
            PseudoNodeCore::new_with_inner(self.node, datatype, inner_pseudo)
        } else if self.inner_as_bool {
            // It's true - create semantic inner
            let final_datatype = Some("semantic".to_string());
            PseudoNodeCore::new(self.node, HashMap::new(), final_datatype, true)
        } else {
            // Normal node
            let is_inner = false;
            PseudoNodeCore::new(self.node, self.child_nodes, datatype, is_inner)
        }
    }
}

/// Helper to get node placeholder (for display/debugging)
impl<TNode: Clone + NodeLike> PseudoNodeCore<TNode> {
    pub fn get_node_placeholder(&self) -> String {
        let mut placeholder = String::from(".");

        // If we have a parent node, recursively get its placeholder
        if let Some(ref parent_rc) = self.parent_node {
            let parent = parent_rc.borrow();
            placeholder.push_str(&parent.get_node_placeholder());
        }

        // Add this node's alias
        if let Some(alias) = self.node.get_alias() {
            placeholder.push_str(alias);
        }

        // Add [*] if iterable
        if self.is_iterable() {
            placeholder.push_str("[*]");
        }

        placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestNode {
        alias: String,
        datatype: String,
    }

    impl NodeLike for TestNode {
        fn get_alias(&self) -> Option<&str> {
            Some(&self.alias)
        }

        fn get_datatype(&self) -> Option<&str> {
            Some(&self.datatype)
        }
    }

    #[test]
    fn test_is_iterable() {
        let node = TestNode {
            alias: "test".to_string(),
            datatype: "concept-list".to_string(),
        };
        let pseudo = PseudoNodeBuilder::new(node, HashMap::new()).build();
        assert!(pseudo.is_iterable());
    }

    #[test]
    fn test_not_iterable() {
        let node = TestNode {
            alias: "test".to_string(),
            datatype: "string".to_string(),
        };
        let pseudo = PseudoNodeBuilder::new(node, HashMap::new()).build();
        assert!(!pseudo.is_iterable());
    }

    #[test]
    fn test_node_placeholder() {
        let node = TestNode {
            alias: "test_alias".to_string(),
            datatype: "string".to_string(),
        };
        let pseudo = PseudoNodeBuilder::new(node, HashMap::new()).build();
        assert_eq!(pseudo.get_node_placeholder(), ".test_alias");
    }

    #[test]
    fn test_iterable_placeholder() {
        let node = TestNode {
            alias: "my_list".to_string(),
            datatype: "concept-list".to_string(),
        };
        let pseudo = PseudoNodeBuilder::new(node, HashMap::new()).build();
        assert_eq!(pseudo.get_node_placeholder(), ".my_list[*]");
    }
}
