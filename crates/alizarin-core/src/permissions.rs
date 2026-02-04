//! Permission rules for conditional tile filtering.
//!
//! Supports both simple boolean permissions and conditional permissions
//! based on tile data values (set membership).

use crate::StaticTile;
use std::collections::HashSet;

// =============================================================================
// Permission Rules
// =============================================================================

/// Permission rule for a nodegroup - either a simple boolean or a conditional
/// based on tile data values.
#[derive(Clone, Debug)]
pub enum PermissionRule {
    /// Simple boolean permission - true = allowed, false = denied
    Boolean(bool),
    /// Conditional permission - tile is allowed if value at path is in allowed set
    Conditional {
        /// JSON path to evaluate (e.g., ".data.uuid.field.name")
        path: String,
        /// Set of allowed values - tile is permitted if value at path is in this set
        allowed: HashSet<String>,
    },
}

impl PermissionRule {
    /// Check if this rule permits the nodegroup itself (not tile-level)
    /// Boolean rules return their value; Conditional rules return true
    /// (the nodegroup is permitted, individual tiles may be filtered)
    pub fn permits_nodegroup(&self) -> bool {
        match self {
            PermissionRule::Boolean(b) => *b,
            PermissionRule::Conditional { .. } => true, // Nodegroup permitted, tiles filtered
        }
    }

    /// Check if a specific tile is permitted by this rule
    pub fn permits_tile(&self, tile: &StaticTile) -> bool {
        match self {
            PermissionRule::Boolean(b) => *b,
            PermissionRule::Conditional { path, allowed } => {
                evaluate_tile_path(tile, path)
                    .map(|value| allowed.contains(&value))
                    .unwrap_or(false) // If path doesn't resolve, deny
            }
        }
    }
}

/// Evaluate a JSON path against a tile's data
/// Path format: ".data.uuid.field.subfield" or "data.uuid.field.subfield"
/// Supports array indexing with bracket notation: ".data.uuid.labels[0].value"
/// Returns the string value at that path, or None if not found/not a string
pub fn evaluate_tile_path(tile: &StaticTile, path: &str) -> Option<String> {
    let path = path.trim_start_matches('.');

    // Parse path segments, handling bracket notation for arrays
    // e.g., "data.uuid.labels[0].value" -> ["data", "uuid", "labels", "[0]", "value"]
    let segments = parse_path_segments(path);

    if segments.is_empty() {
        return None;
    }

    // Start navigation - first segment should be "data" for tile data
    let mut current: &serde_json::Value = if segments[0] == "data" {
        // Convert tile.data HashMap to navigate
        // We need to start from the tile's data map
        if segments.len() < 2 {
            return None;
        }
        // Get the node's data by the next segment (node_id/uuid)
        tile.data.get(&segments[1])?
    } else {
        // Direct path into data - treat first segment as node_id
        tile.data.get(&segments[0])?
    };

    // Navigate remaining segments (skip first 1 or 2 depending on format)
    let start_idx = if segments[0] == "data" { 2 } else { 1 };
    for segment in &segments[start_idx..] {
        current = navigate_segment(current, segment)?;
    }

    // Extract string value (handle both direct strings and nested values)
    if let Some(s) = current.as_str() {
        return Some(s.to_string());
    }

    // Try to get string from common nested patterns
    // e.g., {"en": "value"} or {"value": "x"}
    if let Some(obj) = current.as_object() {
        // Try common language keys
        for key in &["en", "value", "label", "name"] {
            if let Some(v) = obj.get(*key).and_then(|v| v.as_str()) {
                return Some(v.to_string());
            }
        }
    }

    None
}

/// Parse a path string into segments, extracting array indices from bracket notation
/// e.g., "data.uuid.labels[0].value" -> ["data", "uuid", "labels", "[0]", "value"]
fn parse_path_segments(path: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !current.is_empty() {
                    segments.push(current);
                    current = String::new();
                }
            }
            '[' => {
                // Push current segment if not empty
                if !current.is_empty() {
                    segments.push(current);
                    current = String::new();
                }
                // Collect the bracket content as a separate segment
                let mut bracket_content = String::from("[");
                while let Some(&next_c) = chars.peek() {
                    bracket_content.push(chars.next().unwrap());
                    if next_c == ']' {
                        break;
                    }
                }
                segments.push(bracket_content);
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Push final segment
    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

/// Navigate a single segment in a JSON value
/// Handles both object keys and array indices (bracket notation like "[0]")
fn navigate_segment<'a>(
    current: &'a serde_json::Value,
    segment: &str,
) -> Option<&'a serde_json::Value> {
    // Check if it's an array index (bracket notation)
    if segment.starts_with('[') && segment.ends_with(']') {
        let index_str = &segment[1..segment.len() - 1];
        if let Ok(index) = index_str.parse::<usize>() {
            return current.get(index);
        }
        return None;
    }

    // Regular object key access
    current.get(segment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_tile(node_id: &str, data: serde_json::Value) -> StaticTile {
        let mut tile_data = HashMap::new();
        tile_data.insert(node_id.to_string(), data);
        StaticTile {
            tileid: Some("test-tile".to_string()),
            nodegroup_id: "test-nodegroup".to_string(),
            parenttile_id: None,
            resourceinstance_id: "test-resource".to_string(),
            sortorder: Some(0),
            data: tile_data,
            provisionaledits: None,
        }
    }

    #[test]
    fn test_boolean_permission() {
        let tile = make_test_tile("node1", serde_json::json!({"field": "value"}));

        let allow = PermissionRule::Boolean(true);
        assert!(allow.permits_nodegroup());
        assert!(allow.permits_tile(&tile));

        let deny = PermissionRule::Boolean(false);
        assert!(!deny.permits_nodegroup());
        assert!(!deny.permits_tile(&tile));
    }

    #[test]
    fn test_conditional_permission_match() {
        let tile = make_test_tile(
            "node1",
            serde_json::json!({
                "label": {"name": "Hotel/Inn"}
            }),
        );

        let mut allowed = HashSet::new();
        allowed.insert("Hotel/Inn".to_string());
        allowed.insert("Restaurant".to_string());

        let rule = PermissionRule::Conditional {
            path: ".data.node1.label.name".to_string(),
            allowed,
        };

        assert!(rule.permits_nodegroup()); // Conditional always permits nodegroup
        assert!(rule.permits_tile(&tile)); // Value matches
    }

    #[test]
    fn test_conditional_permission_no_match() {
        let tile = make_test_tile(
            "node1",
            serde_json::json!({
                "label": {"name": "Castle"}
            }),
        );

        let mut allowed = HashSet::new();
        allowed.insert("Hotel/Inn".to_string());

        let rule = PermissionRule::Conditional {
            path: ".data.node1.label.name".to_string(),
            allowed,
        };

        assert!(rule.permits_nodegroup()); // Conditional always permits nodegroup
        assert!(!rule.permits_tile(&tile)); // Value doesn't match
    }

    #[test]
    fn test_conditional_permission_missing_path() {
        let tile = make_test_tile(
            "node1",
            serde_json::json!({
                "other_field": "value"
            }),
        );

        let mut allowed = HashSet::new();
        allowed.insert("Hotel/Inn".to_string());

        let rule = PermissionRule::Conditional {
            path: ".data.node1.label.name".to_string(),
            allowed,
        };

        assert!(!rule.permits_tile(&tile)); // Path doesn't exist, deny
    }

    #[test]
    fn test_path_without_data_prefix() {
        let tile = make_test_tile(
            "node1",
            serde_json::json!({
                "label": {"name": "Hotel/Inn"}
            }),
        );

        let mut allowed = HashSet::new();
        allowed.insert("Hotel/Inn".to_string());

        let rule = PermissionRule::Conditional {
            path: "node1.label.name".to_string(), // Without .data prefix
            allowed,
        };

        assert!(rule.permits_tile(&tile));
    }

    #[test]
    fn test_array_indexing() {
        // Test array access with bracket notation like labels[0].value
        let tile = make_test_tile(
            "node1",
            serde_json::json!({
                "labels": [
                    {"value": "Description", "language": "en"},
                    {"value": "Title", "language": "en"}
                ]
            }),
        );

        let mut allowed = HashSet::new();
        allowed.insert("Description".to_string());

        let rule = PermissionRule::Conditional {
            path: ".data.node1.labels[0].value".to_string(),
            allowed,
        };

        assert!(rule.permits_tile(&tile)); // First label has "Description"

        // Test that second element doesn't match
        let mut allowed2 = HashSet::new();
        allowed2.insert("Description".to_string());

        let rule2 = PermissionRule::Conditional {
            path: ".data.node1.labels[1].value".to_string(),
            allowed: allowed2,
        };

        assert!(!rule2.permits_tile(&tile)); // Second label has "Title", not "Description"
    }

    #[test]
    fn test_parse_path_segments() {
        // Test the path segment parser
        let segments = parse_path_segments("data.uuid.labels[0].value");
        assert_eq!(segments, vec!["data", "uuid", "labels", "[0]", "value"]);

        let segments2 = parse_path_segments(".data.node.array[2].nested.field");
        assert_eq!(
            segments2,
            vec!["data", "node", "array", "[2]", "nested", "field"]
        );
    }
}
