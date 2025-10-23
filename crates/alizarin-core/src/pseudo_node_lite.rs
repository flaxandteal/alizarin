/// Lightweight core for PseudoNode that can be used by both WASM and Python
/// This version focuses on business logic without circular reference handling

// Constants for iterable datatypes - SINGLE SOURCE OF TRUTH
pub const ITERABLE_DATATYPES: &[&str] = &[
    "concept-list",
    "resource-instance-list",
    "domain-value-list"
];

/// Lightweight pseudo node state (no circular refs, just data)
#[derive(Clone, Debug)]
pub struct PseudoNodeState {
    pub datatype: Option<String>,
    pub is_outer: bool,
    pub is_inner: bool,
}

impl PseudoNodeState {
    /// Create new state from constructor parameters
    pub fn new(datatype: Option<String>, inner_is_bool: bool, inner_is_object: bool) -> Self {
        let (is_outer, is_inner, final_datatype) = if inner_is_object {
            // It's a PseudoValue/PseudoNode instance
            (true, false, datatype)
        } else if inner_is_bool {
            // It's true - create semantic inner
            (false, true, Some("semantic".to_string()))
        } else {
            // Normal node
            (false, false, datatype)
        };

        PseudoNodeState {
            datatype: final_datatype,
            is_outer,
            is_inner,
        }
    }

    /// Check if this datatype is iterable
    pub fn is_iterable(&self) -> bool {
        if let Some(ref dt) = self.datatype {
            ITERABLE_DATATYPES.contains(&dt.as_str())
        } else {
            false
        }
    }
}

/// Helper to build placeholder strings
pub fn build_placeholder(
    parent_placeholder: Option<&str>,
    alias: Option<&str>,
    is_iterable: bool,
) -> String {
    let mut placeholder = String::from(".");

    // Add parent placeholder if exists
    if let Some(parent) = parent_placeholder {
        placeholder.push_str(parent);
    }

    // Add this node's alias
    if let Some(alias) = alias {
        placeholder.push_str(alias);
    }

    // Add [*] if iterable
    if is_iterable {
        placeholder.push_str("[*]");
    }

    placeholder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_iterable() {
        let state = PseudoNodeState::new(Some("concept-list".to_string()), false, false);
        assert!(state.is_iterable());
    }

    #[test]
    fn test_not_iterable() {
        let state = PseudoNodeState::new(Some("string".to_string()), false, false);
        assert!(!state.is_iterable());
    }

    #[test]
    fn test_semantic_inner() {
        let state = PseudoNodeState::new(Some("number".to_string()), true, false);
        assert!(state.is_inner);
        assert_eq!(state.datatype, Some("semantic".to_string()));
    }

    #[test]
    fn test_outer() {
        let state = PseudoNodeState::new(Some("string".to_string()), false, true);
        assert!(state.is_outer);
        assert_eq!(state.datatype, Some("string".to_string()));
    }

    #[test]
    fn test_build_placeholder() {
        let result = build_placeholder(None, Some("test"), false);
        assert_eq!(result, ".test");
    }

    #[test]
    fn test_build_placeholder_iterable() {
        let result = build_placeholder(None, Some("my_list"), true);
        assert_eq!(result, ".my_list[*]");
    }

    #[test]
    fn test_build_placeholder_with_parent() {
        let result = build_placeholder(Some(".parent"), Some("child"), false);
        assert_eq!(result, ".parent.child");
    }
}
