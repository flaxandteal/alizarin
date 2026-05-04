//! RDM Namespace utilities for deterministic UUID generation.
//!
//! This module provides:
//! - Parsing namespace strings (UUID or URL)
//! - Generating deterministic UUIDs for collections and concepts
//! - Global namespace storage (used by import_prebuild and bindings)

use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

lazy_static::lazy_static! {
    /// Global namespace UUID for deterministic RDM ID generation.
    /// Must be set before creating collections/concepts from labels.
    static ref GLOBAL_RDM_NAMESPACE: RwLock<Option<Uuid>> = RwLock::new(None);
}

/// Set the global RDM namespace for deterministic UUID generation.
///
/// This namespace is used when creating collections and concepts from labels
/// without explicit IDs. Must be set before using from_labels, from_nested_labels,
/// add_from_label, or add_child_from_label with auto-generated IDs.
///
/// Accepts either a UUID string or a URL (http/https), which is converted to
/// a deterministic UUID5 using the standard URL namespace.
pub fn set_rdm_namespace(namespace: &str) -> Result<(), String> {
    let uuid = parse_rdm_namespace(namespace)?;
    if let Ok(mut guard) = GLOBAL_RDM_NAMESPACE.write() {
        *guard = Some(uuid);
    }
    Ok(())
}

/// Get the current global RDM namespace, if set.
pub fn get_rdm_namespace() -> Option<Uuid> {
    GLOBAL_RDM_NAMESPACE.read().ok().and_then(|guard| *guard)
}

/// Check if a global RDM namespace is set.
pub fn has_rdm_namespace() -> bool {
    GLOBAL_RDM_NAMESPACE
        .read()
        .ok()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// Clear the global RDM namespace.
pub fn clear_rdm_namespace() {
    if let Ok(mut guard) = GLOBAL_RDM_NAMESPACE.write() {
        *guard = None;
    }
}

// =============================================================================
// Namespace Parsing
// =============================================================================

/// Parse a namespace string into a UUID.
///
/// Accepts either:
/// - A valid UUID string (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// - A URL (http:// or https://) which is converted to UUID5 using the standard URL namespace
///
/// # Arguments
/// * `namespace` - UUID string or URL
///
/// # Returns
/// * `Ok(Uuid)` - The parsed or derived namespace UUID
/// * `Err(String)` - Error message if parsing fails
///
/// # Examples
/// ```
/// use alizarin_core::rdm_namespace::parse_rdm_namespace;
///
/// // Direct UUID
/// let uuid = parse_rdm_namespace("550e8400-e29b-41d4-a716-446655440000").unwrap();
///
/// // URL converted to UUID5
/// let uuid = parse_rdm_namespace("http://example.org/rdm/").unwrap();
/// ```
pub fn parse_rdm_namespace(namespace: &str) -> Result<Uuid, String> {
    if namespace.starts_with("http://") || namespace.starts_with("https://") {
        // URL provided - derive a UUID5 from it using the standard URL namespace
        Ok(Uuid::new_v5(&Uuid::NAMESPACE_URL, namespace.as_bytes()))
    } else {
        // Try to parse as UUID directly
        Uuid::parse_str(namespace).map_err(|e| {
            format!(
                "Invalid namespace: expected UUID or URL (http/https), got: {} ({})",
                namespace, e
            )
        })
    }
}

// =============================================================================
// UUID Generation
// =============================================================================

/// Generate a deterministic UUID for a collection from its name.
///
/// Uses: uuid5(namespace, "collection/" + name)
///
/// # Arguments
/// * `namespace` - The global RDM namespace UUID
/// * `name` - The collection name
///
/// # Returns
/// A deterministic UUID for the collection
pub fn generate_collection_uuid(namespace: &Uuid, name: &str) -> Uuid {
    let key = format!("collection/{}", name);
    Uuid::new_v5(namespace, key.as_bytes())
}

/// Generate a deterministic UUID for a concept from its collection and label.
///
/// Uses: uuid5(collection_id, label_string)
///
/// # Arguments
/// * `collection_id` - The collection UUID (used as namespace)
/// * `label` - The concept label (or deterministic string representation)
///
/// # Returns
/// A deterministic UUID for the concept
pub fn generate_concept_uuid(collection_id: &Uuid, label: &str) -> Uuid {
    Uuid::new_v5(collection_id, label.as_bytes())
}

/// Generate a deterministic UUID for a concept from collection ID string and label.
///
/// Convenience function that parses the collection ID first.
///
/// # Arguments
/// * `collection_id` - The collection UUID as a string
/// * `label` - The concept label
///
/// # Returns
/// * `Ok(Uuid)` - The generated concept UUID
/// * `Err(String)` - Error if collection_id is not a valid UUID
pub fn generate_concept_uuid_from_str(collection_id: &str, label: &str) -> Result<Uuid, String> {
    let namespace = Uuid::parse_str(collection_id).map_err(|e| {
        format!(
            "Collection ID must be a valid UUID for auto-generation: {}",
            e
        )
    })?;
    Ok(generate_concept_uuid(&namespace, label))
}

/// Generate a deterministic UUID for a value (label in a specific language).
///
/// Uses: uuid5(uuid5(NAMESPACE_URL, "value"), "{concept_id}/prefLabel/{value}/{language}")
///
/// This matches the behavior in RdmValue::generate_id for consistency.
///
/// # Arguments
/// * `concept_id` - The concept UUID
/// * `value` - The label text
/// * `language` - The language code (e.g., "en")
///
/// # Returns
/// A deterministic UUID for the value
pub fn generate_value_uuid(concept_id: &str, value: &str, language: &str) -> Uuid {
    let namespace = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"value");
    let name = format!("{}/prefLabel/{}/{}", concept_id, value, language);
    Uuid::new_v5(&namespace, name.as_bytes())
}

// =============================================================================
// Label Utilities
// =============================================================================

/// Convert a multilingual label map to a deterministic string for UUID generation.
///
/// Sorts by language code and concatenates as "lang1:value1|lang2:value2".
/// This ensures deterministic UUID generation regardless of map ordering.
///
/// # Arguments
/// * `labels` - Map of language code to label text
///
/// # Returns
/// A deterministic string representation
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use alizarin_core::rdm_namespace::labels_to_deterministic_string;
///
/// let mut labels = HashMap::new();
/// labels.insert("en".to_string(), "Hello".to_string());
/// labels.insert("de".to_string(), "Hallo".to_string());
///
/// let s = labels_to_deterministic_string(&labels);
/// assert_eq!(s, "de:Hallo|en:Hello");
/// ```
pub fn labels_to_deterministic_string(labels: &HashMap<String, String>) -> String {
    let mut entries: Vec<(&String, &String)> = labels.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    entries
        .iter()
        .map(|(lang, val)| format!("{}:{}", lang, val))
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_string() {
        let uuid = parse_rdm_namespace("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_parse_url_to_uuid() {
        let uuid1 = parse_rdm_namespace("http://example.org/rdm/").unwrap();
        let uuid2 = parse_rdm_namespace("http://example.org/rdm/").unwrap();
        // Same URL should produce same UUID
        assert_eq!(uuid1, uuid2);
        // URL-derived UUID should be different from direct UUID namespace
        assert_ne!(uuid1.to_string(), "http://example.org/rdm/");
    }

    #[test]
    fn test_parse_https_url() {
        let uuid = parse_rdm_namespace("https://example.org/rdm/").unwrap();
        // Should succeed for HTTPS
        assert!(uuid.to_string().len() == 36);
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse_rdm_namespace("not-a-uuid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid namespace"));
    }

    #[test]
    fn test_generate_collection_uuid() {
        let namespace = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid1 = generate_collection_uuid(&namespace, "TestCollection");
        let uuid2 = generate_collection_uuid(&namespace, "TestCollection");
        // Same inputs should produce same UUID
        assert_eq!(uuid1, uuid2);

        // Different name should produce different UUID
        let uuid3 = generate_collection_uuid(&namespace, "OtherCollection");
        assert_ne!(uuid1, uuid3);
    }

    #[test]
    fn test_generate_concept_uuid() {
        let collection_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid1 = generate_concept_uuid(&collection_id, "ConceptA");
        let uuid2 = generate_concept_uuid(&collection_id, "ConceptA");
        assert_eq!(uuid1, uuid2);

        let uuid3 = generate_concept_uuid(&collection_id, "ConceptB");
        assert_ne!(uuid1, uuid3);
    }

    #[test]
    fn test_generate_concept_uuid_from_str() {
        let uuid =
            generate_concept_uuid_from_str("550e8400-e29b-41d4-a716-446655440000", "ConceptA")
                .unwrap();
        assert!(uuid.to_string().len() == 36);

        // Invalid collection ID should error
        let result = generate_concept_uuid_from_str("not-a-uuid", "ConceptA");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_value_uuid() {
        let uuid1 = generate_value_uuid("concept-123", "Hello", "en");
        let uuid2 = generate_value_uuid("concept-123", "Hello", "en");
        assert_eq!(uuid1, uuid2);

        // Different language should produce different UUID
        let uuid3 = generate_value_uuid("concept-123", "Hallo", "de");
        assert_ne!(uuid1, uuid3);
    }

    #[test]
    fn test_labels_to_deterministic_string() {
        let mut labels = HashMap::new();
        labels.insert("en".to_string(), "Hello".to_string());
        labels.insert("de".to_string(), "Hallo".to_string());
        labels.insert("fr".to_string(), "Bonjour".to_string());

        let s = labels_to_deterministic_string(&labels);
        // Should be sorted by language code
        assert_eq!(s, "de:Hallo|en:Hello|fr:Bonjour");
    }

    #[test]
    fn test_labels_to_deterministic_string_single() {
        let mut labels = HashMap::new();
        labels.insert("en".to_string(), "Hello".to_string());

        let s = labels_to_deterministic_string(&labels);
        assert_eq!(s, "en:Hello");
    }

    #[test]
    fn test_labels_to_deterministic_string_empty() {
        let labels = HashMap::new();
        let s = labels_to_deterministic_string(&labels);
        assert_eq!(s, "");
    }
}
