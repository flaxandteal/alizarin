//! Core RDM Cache for concept lookups
//!
//! This module provides a platform-agnostic cache for Reference Data Manager (RDM) collections,
//! enabling UUID -> label lookups for concept and concept-list datatypes.
//!
//! The WASM bindings (alizarin-wasm) wrap this with WasmRdmCache for JavaScript interop.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};

use crate::rdm_namespace::generate_value_uuid;

// =============================================================================
// RDM Value (label with its own ID)
// =============================================================================

/// A label value with its own ID (for StaticValue compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdmValue {
    /// Value ID (UUID) - unique identifier for this specific label
    pub id: String,
    /// The label text
    pub value: String,
    /// Back-reference to concept ID (not serialized, set during indexing)
    #[serde(skip)]
    pub concept_id: String,
    /// Language code (not serialized, set during indexing)
    #[serde(skip)]
    pub language: String,
}

impl RdmValue {
    /// Create a new RdmValue
    pub fn new(id: String, value: String) -> Self {
        Self {
            id,
            value,
            concept_id: String::new(),
            language: String::new(),
        }
    }

    /// Create a new RdmValue with back-references
    pub fn with_context(id: String, value: String, concept_id: String, language: String) -> Self {
        Self {
            id,
            value,
            concept_id,
            language,
        }
    }

    /// Generate a deterministic value ID from concept info
    /// Uses UUID5 with namespace "value" and path: "{concept_id}/prefLabel/{value}/{language}"
    ///
    /// Delegates to `rdm_namespace::generate_value_uuid` for the actual generation.
    pub fn generate_id(concept_id: &str, value: &str, language: &str) -> String {
        generate_value_uuid(concept_id, value, language).to_string()
    }
}

/// Intermediate type for deserializing pref_label that can be either:
/// - A simple string: "Label"
/// - A value object: { "id": "...", "value": "Label" }
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum PrefLabelEntry {
    Simple(String),
    WithId { id: String, value: String },
}

// =============================================================================
// RDM Concept
// =============================================================================

/// A concept from an RDM collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdmConcept {
    /// Concept ID (UUID)
    pub id: String,
    /// Preferred labels by language code (with value IDs)
    #[serde(
        default,
        alias = "prefLabel",
        alias = "prefLabels",
        deserialize_with = "deserialize_pref_labels"
    )]
    pub pref_label: HashMap<String, RdmValue>,
    /// Alternative labels by language code
    #[serde(default, rename = "altLabels")]
    pub alt_labels: HashMap<String, Vec<String>>,
    /// Broader concepts (parent IDs)
    #[serde(default)]
    pub broader: Vec<String>,
    /// Narrower concepts (child IDs)
    #[serde(default)]
    pub narrower: Vec<String>,
    /// Scope notes by language
    #[serde(default, rename = "scopeNote")]
    pub scope_note: HashMap<String, String>,
}

/// Custom deserializer that handles both JSON formats for pref_label:
/// - Simple: { "en": "Label" }
/// - WithId: { "en": { "id": "...", "value": "Label" } }
fn deserialize_pref_labels<'de, D>(deserializer: D) -> Result<HashMap<String, RdmValue>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, PrefLabelEntry> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::new();

    for (lang, entry) in raw {
        let value = match entry {
            PrefLabelEntry::Simple(text) => {
                // Generate a placeholder ID - will be replaced during indexing
                RdmValue::new("__pending__".to_string(), text)
            }
            PrefLabelEntry::WithId { id, value } => RdmValue::new(id, value),
        };
        result.insert(lang, value);
    }

    Ok(result)
}

impl RdmConcept {
    /// Get the preferred label for a language, with fallbacks
    pub fn get_label(&self, language: &str) -> Option<String> {
        self.pref_label
            .get(language)
            .or_else(|| self.pref_label.get("en"))
            .or_else(|| self.pref_label.values().next())
            .map(|v| v.value.clone())
    }

    /// Get the RdmValue for a language, with fallbacks
    pub fn get_value(&self, language: &str) -> Option<&RdmValue> {
        self.pref_label
            .get(language)
            .or_else(|| self.pref_label.get("en"))
            .or_else(|| self.pref_label.values().next())
    }
}

// =============================================================================
// RDM Collection
// =============================================================================

/// A collection of RDM concepts
#[derive(Debug, Clone, Default)]
pub struct RdmCollection {
    /// Collection ID
    pub id: String,
    /// Collection name (optional, for display)
    pub name: Option<String>,
    /// Concepts indexed by their ID
    concepts: HashMap<String, RdmConcept>,
    /// Top-level concepts (no broader)
    top_concepts: Vec<String>,
    /// Index from VALUE ID to (concept_id, language) for fast lookup
    value_index: HashMap<String, (String, String)>,
}

impl RdmCollection {
    pub fn new(id: String) -> Self {
        Self {
            id,
            name: None,
            concepts: HashMap::new(),
            top_concepts: vec![],
            value_index: HashMap::new(),
        }
    }

    /// Create a new collection with a name
    pub fn with_name(id: String, name: String) -> Self {
        Self {
            id,
            name: Some(name),
            concepts: HashMap::new(),
            top_concepts: vec![],
            value_index: HashMap::new(),
        }
    }

    /// Add a concept to the collection
    ///
    /// This also builds the value index for all labels in the concept.
    /// If a label has a placeholder ID ("__pending__"), a deterministic ID is generated.
    pub fn add_concept(&mut self, mut concept: RdmConcept) {
        let concept_id = concept.id.clone();

        // Process each label: set back-references and build value index
        for (lang, value) in concept.pref_label.iter_mut() {
            // Generate ID if it's a placeholder
            if value.id == "__pending__" {
                value.id = RdmValue::generate_id(&concept_id, &value.value, lang);
            }

            // Set back-references
            value.concept_id = concept_id.clone();
            value.language = lang.clone();

            // Add to value index
            self.value_index
                .insert(value.id.clone(), (concept_id.clone(), lang.clone()));
        }

        if concept.broader.is_empty() {
            self.top_concepts.push(concept_id.clone());
        }
        self.concepts.insert(concept_id, concept);
    }

    /// Get top-level concepts (no broader)
    pub fn get_top_concepts(&self) -> Vec<&RdmConcept> {
        self.top_concepts
            .iter()
            .filter_map(|id| self.concepts.get(id))
            .collect()
    }

    /// Get a concept by ID
    pub fn get_concept(&self, concept_id: &str) -> Option<&RdmConcept> {
        self.concepts.get(concept_id)
    }

    /// Get a mutable concept by ID
    pub fn get_concept_mut(&mut self, concept_id: &str) -> Option<&mut RdmConcept> {
        self.concepts.get_mut(concept_id)
    }

    /// Get the label for a concept in this collection
    pub fn get_label(&self, concept_id: &str, language: &str) -> Option<String> {
        self.get_concept(concept_id)
            .and_then(|c| c.get_label(language))
    }

    /// Parse collection from JSON array of concepts
    pub fn from_concepts_json(id: String, json: &str) -> Result<Self, String> {
        let concepts: Vec<RdmConcept> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse concepts JSON: {}", e))?;

        let mut collection = Self::new(id);
        for concept in concepts {
            collection.add_concept(concept);
        }
        Ok(collection)
    }

    /// Get the number of concepts in this collection
    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    /// Check if a concept exists in the collection
    pub fn has_concept(&self, concept_id: &str) -> bool {
        self.concepts.contains_key(concept_id)
    }

    /// Get all concept IDs
    pub fn get_concept_ids(&self) -> Vec<&String> {
        self.concepts.keys().collect()
    }

    /// Get the first parent ID for a concept (from broader field)
    ///
    /// Returns None if the concept doesn't exist or has no parent (top-level concept).
    /// SKOS concepts can have multiple parents; this returns only the first one.
    pub fn get_parent_id(&self, concept_id: &str) -> Option<String> {
        self.get_concept(concept_id)
            .and_then(|c| c.broader.first().cloned())
    }

    // =========================================================================
    // Value ID Lookups (for StaticValue compatibility)
    // =========================================================================

    /// Look up a value by its VALUE ID
    ///
    /// This is the primary lookup method used by ViewModels.
    /// Returns None if the value ID is not found in this collection.
    pub fn get_value_by_id(&self, value_id: &str) -> Option<&RdmValue> {
        self.value_index
            .get(value_id)
            .and_then(|(concept_id, lang)| {
                self.concepts
                    .get(concept_id)
                    .and_then(|c| c.pref_label.get(lang))
            })
    }

    /// Get concept ID from value ID
    ///
    /// Returns the concept ID that contains the given value ID.
    pub fn get_concept_id_for_value(&self, value_id: &str) -> Option<&str> {
        self.value_index
            .get(value_id)
            .map(|(concept_id, _)| concept_id.as_str())
    }

    /// Check if a value ID exists in this collection
    pub fn has_value(&self, value_id: &str) -> bool {
        self.value_index.contains_key(value_id)
    }

    /// Get all value IDs in this collection
    pub fn get_value_ids(&self) -> Vec<&String> {
        self.value_index.keys().collect()
    }

    // =========================================================================
    // Label-based Lookups
    // =========================================================================

    /// Find a concept by exact label match (case-insensitive)
    ///
    /// Searches pref_label and alt_labels across all languages.
    /// Returns None if no match or multiple matches (ambiguous).
    pub fn find_by_label(&self, label: &str) -> Option<&RdmConcept> {
        let label_lower = label.trim().to_lowercase();
        let matches: Vec<_> = self
            .concepts
            .values()
            .filter(|c| {
                // Check pref_label in any language (trim stored values too)
                c.pref_label.values().any(|p| p.value.trim().to_lowercase() == label_lower) ||
                // Check alt_labels in any language
                c.alt_labels.values().any(|alts|
                    alts.iter().any(|l| l.trim().to_lowercase() == label_lower)
                )
            })
            .collect();

        // Only return if exactly one match (unambiguous)
        if matches.len() == 1 {
            matches.into_iter().next()
        } else {
            None
        }
    }

    /// Find all concepts by exact label match (case-insensitive)
    pub fn find_all_by_label(&self, label: &str) -> Vec<&RdmConcept> {
        let label_lower = label.trim().to_lowercase();
        self.concepts
            .values()
            .filter(|c| {
                c.pref_label
                    .values()
                    .any(|p| p.value.trim().to_lowercase() == label_lower)
                    || c.alt_labels
                        .values()
                        .any(|alts| alts.iter().any(|l| l.trim().to_lowercase() == label_lower))
            })
            .collect()
    }

    /// Search concepts by label prefix (case-insensitive)
    pub fn search(&self, query: &str, language: Option<&str>) -> Vec<&RdmConcept> {
        let lang = language.unwrap_or("en");
        let query_lower = query.to_lowercase();

        self.concepts
            .values()
            .filter(|c| {
                // Check pref_label
                if let Some(label) = c.pref_label.get(lang) {
                    if label.value.to_lowercase().starts_with(&query_lower) {
                        return true;
                    }
                }
                // Check alt_labels
                if let Some(alts) = c.alt_labels.get(lang) {
                    if alts
                        .iter()
                        .any(|l| l.to_lowercase().starts_with(&query_lower))
                    {
                        return true;
                    }
                }
                false
            })
            .collect()
    }
}

// =============================================================================
// RDM Cache
// =============================================================================

/// Cache for RDM collections, enabling concept UUID -> label lookups
#[derive(Debug, Clone, Default)]
pub struct RdmCache {
    collections: HashMap<String, RdmCollection>,
}

impl RdmCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    /// Add a collection from JSON
    ///
    /// @param collection_id - The collection identifier
    /// @param concepts_json - JSON array of concepts with {id, prefLabel: {lang: label}}
    pub fn add_collection_from_json(
        &mut self,
        collection_id: &str,
        concepts_json: &str,
    ) -> Result<(), String> {
        let collection =
            RdmCollection::from_concepts_json(collection_id.to_string(), concepts_json)?;

        self.collections
            .insert(collection_id.to_string(), collection);
        Ok(())
    }

    /// Add a collection directly
    pub fn add_collection(&mut self, collection: RdmCollection) {
        self.collections.insert(collection.id.clone(), collection);
    }

    /// Check if a collection is loaded
    pub fn has_collection(&self, collection_id: &str) -> bool {
        self.collections.contains_key(collection_id)
    }

    /// Get all loaded collection IDs
    pub fn get_collection_ids(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }

    /// Look up the label for a concept
    ///
    /// @param collection_id - The collection to search in
    /// @param concept_id - The concept UUID
    /// @param language - The language code (e.g., "en")
    /// @returns The label string, or None if not found
    pub fn lookup_label(
        &self,
        collection_id: &str,
        concept_id: &str,
        language: &str,
    ) -> Option<String> {
        self.collections
            .get(collection_id)
            .and_then(|c| c.get_label(concept_id, language))
    }

    /// Look up full concept info
    pub fn lookup_concept(&self, collection_id: &str, concept_id: &str) -> Option<&RdmConcept> {
        self.collections
            .get(collection_id)
            .and_then(|c| c.get_concept(concept_id))
    }

    /// Get the first parent ID for a concept
    ///
    /// Returns None if the collection doesn't exist, concept doesn't exist,
    /// or concept has no parent (top-level concept).
    pub fn get_parent_id(&self, collection_id: &str, concept_id: &str) -> Option<String> {
        self.collections
            .get(collection_id)
            .and_then(|c| c.get_parent_id(concept_id))
    }

    // =========================================================================
    // Value ID Lookups (for StaticValue compatibility)
    // =========================================================================

    /// Look up a value by its VALUE ID
    ///
    /// This is the primary lookup method used by ViewModels.
    /// Returns None if the collection or value ID is not found.
    pub fn lookup_value(&self, collection_id: &str, value_id: &str) -> Option<&RdmValue> {
        self.collections
            .get(collection_id)
            .and_then(|c| c.get_value_by_id(value_id))
    }

    /// Get concept ID from value ID
    ///
    /// Returns the concept ID that contains the given value ID.
    pub fn get_concept_id_for_value(&self, collection_id: &str, value_id: &str) -> Option<&str> {
        self.collections
            .get(collection_id)
            .and_then(|c| c.get_concept_id_for_value(value_id))
    }

    /// Validate that a value exists in a collection
    pub fn validate_value(&self, collection_id: &str, value_id: &str) -> bool {
        self.collections
            .get(collection_id)
            .map(|c| c.has_value(value_id))
            .unwrap_or(false)
    }

    /// Get a collection by ID
    pub fn get_collection(&self, collection_id: &str) -> Option<&RdmCollection> {
        self.collections.get(collection_id)
    }

    /// Get a mutable reference to a collection by ID
    pub fn get_collection_mut(&mut self, collection_id: &str) -> Option<&mut RdmCollection> {
        self.collections.get_mut(collection_id)
    }

    /// Clear all cached collections
    pub fn clear(&mut self) {
        self.collections.clear();
    }

    /// Remove a specific collection from the cache
    pub fn remove_collection(&mut self, collection_id: &str) -> bool {
        self.collections.remove(collection_id).is_some()
    }

    /// Get the number of cached collections
    pub fn len(&self) -> usize {
        self.collections.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }

    /// Validate that a concept exists in a collection
    pub fn validate_concept(&self, collection_id: &str, concept_id: &str) -> bool {
        self.collections
            .get(collection_id)
            .map(|c| c.has_concept(concept_id))
            .unwrap_or(false)
    }

    /// Look up a concept by label in a specific collection
    ///
    /// Returns the concept if exactly one match is found.
    /// Returns None if no match or ambiguous (multiple matches).
    pub fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<&RdmConcept> {
        self.collections
            .get(collection_id)
            .and_then(|c| c.find_by_label(label))
    }

    /// Look up a concept by label, returning all matches
    pub fn lookup_all_by_label(&self, collection_id: &str, label: &str) -> Vec<&RdmConcept> {
        self.collections
            .get(collection_id)
            .map(|c| c.find_all_by_label(label))
            .unwrap_or_default()
    }

    /// Search across all collections (for autocomplete)
    pub fn search_all(&self, query: &str, language: Option<&str>) -> Vec<(&str, &RdmConcept)> {
        self.collections
            .iter()
            .flat_map(|(coll_id, collection)| {
                collection
                    .search(query, language)
                    .into_iter()
                    .map(move |c| (coll_id.as_str(), c))
            })
            .collect()
    }
}

// =============================================================================
// SKOS → RDM Conversion
// =============================================================================

use crate::skos::{SkosCollection, SkosConcept, SkosNodeType, SkosValue};

impl RdmCache {
    /// Convert a `SkosCollection` to an `RdmCollection` and add it to the cache.
    ///
    /// Returns the collection ID.
    pub fn add_from_skos_collection(&mut self, skos: &SkosCollection) -> String {
        let rdm = skos_to_rdm_collection(skos);
        let id = rdm.id.clone();
        self.add_collection(rdm);
        id
    }

    /// Add multiple SKOS collections to the cache.
    ///
    /// After adding, enriches any existing collections that have bare concept stubs
    /// (concepts with no labels) if the newly-added data provides labels for those
    /// concept IDs. This handles the common Arches pattern where collections.xml and
    /// concepts.xml are separate files.
    ///
    /// Returns the list of collection IDs added.
    pub fn add_from_skos_collections(&mut self, collections: &[SkosCollection]) -> Vec<String> {
        let added_ids: Vec<String> = collections
            .iter()
            .map(|skos| self.add_from_skos_collection(skos))
            .collect();

        // Cross-collection enrichment: if any existing collection has bare concepts
        // (no labels), and a newly-added collection has the same concept ID with labels,
        // copy the labels across.
        self.enrich_bare_concepts(&added_ids);

        added_ids
    }

    /// Enrich bare concepts across all collections.
    ///
    /// A "bare concept" is one that exists in a collection (as a member reference)
    /// but has no pref_labels — typically because it was loaded from a collections.xml
    /// that only declared member URIs without inline concept definitions.
    ///
    /// This checks bidirectionally: newly-added collections can enrich existing ones,
    /// and existing collections can enrich newly-added ones.
    fn enrich_bare_concepts(&mut self, newly_added_ids: &[String]) {
        // Build a global lookup of concept_id -> labels from ALL collections
        let mut concept_labels: HashMap<String, HashMap<String, RdmValue>> = HashMap::new();

        for coll in self.collections.values() {
            for (concept_id, concept) in &coll.concepts {
                if !concept.pref_label.is_empty() {
                    concept_labels
                        .entry(concept_id.clone())
                        .or_insert_with(|| concept.pref_label.clone());
                }
            }
        }

        if concept_labels.is_empty() {
            return;
        }

        // Find and enrich bare concepts in all collections that were involved
        // (either newly added or existing ones that reference newly-added concepts)
        let all_collection_ids: Vec<String> = self.collections.keys().cloned().collect();
        for coll_id in &all_collection_ids {
            let needs_enrichment: Vec<String> = {
                if let Some(coll) = self.collections.get(coll_id) {
                    coll.concepts
                        .iter()
                        .filter(|(_, concept)| concept.pref_label.is_empty())
                        .filter(|(id, _)| concept_labels.contains_key(*id))
                        .map(|(id, _)| id.clone())
                        .collect()
                } else {
                    vec![]
                }
            };

            if needs_enrichment.is_empty() {
                continue;
            }

            // Only enrich if this collection is newly added OR if it references
            // concepts from a newly-added collection
            let dominated_by_new = newly_added_ids.contains(coll_id)
                || needs_enrichment
                    .iter()
                    .any(|cid| self.concept_in_collections(cid, newly_added_ids));

            if !dominated_by_new {
                continue;
            }

            if let Some(coll) = self.collections.get_mut(coll_id) {
                for concept_id in needs_enrichment {
                    if let Some(labels) = concept_labels.get(&concept_id) {
                        // Build value index entries first
                        let mut new_index_entries: Vec<(String, String, String)> = Vec::new();
                        let mut enriched_labels = labels.clone();
                        for (lang, value) in enriched_labels.iter_mut() {
                            value.concept_id = concept_id.clone();
                            value.language = lang.clone();
                            new_index_entries.push((
                                value.id.clone(),
                                concept_id.clone(),
                                lang.clone(),
                            ));
                        }

                        // Apply to concept
                        if let Some(concept) = coll.get_concept_mut(&concept_id) {
                            concept.pref_label = enriched_labels;
                        }

                        // Update value index
                        for (value_id, cid, lang) in new_index_entries {
                            coll.value_index.insert(value_id, (cid, lang));
                        }
                    }
                }
            }
        }
    }

    /// Check if a concept ID exists (with labels) in any of the specified collections.
    fn concept_in_collections(&self, concept_id: &str, collection_ids: &[String]) -> bool {
        collection_ids.iter().any(|coll_id| {
            self.collections
                .get(coll_id)
                .and_then(|c| c.get_concept(concept_id))
                .map(|concept| !concept.pref_label.is_empty())
                .unwrap_or(false)
        })
    }
}

/// Convert a `SkosCollection` (parsed from SKOS XML or JSON) to an `RdmCollection`
/// suitable for label lookups.
///
/// Walks the hierarchical `concepts` tree recursively, setting broader/narrower
/// relationships. Falls back to `all_concepts` for flat structures.
pub fn skos_to_rdm_collection(skos: &SkosCollection) -> RdmCollection {
    let mut rdm = RdmCollection::with_name(
        skos.id.clone(),
        skos.pref_labels
            .get("en")
            .map(|v| v.value.clone())
            .unwrap_or_else(|| skos.id.clone()),
    );

    fn add_concept_recursive(
        rdm: &mut RdmCollection,
        skos_concept: &SkosConcept,
        parent_id: Option<&str>,
    ) {
        let mut pref_label: HashMap<String, RdmValue> = HashMap::new();
        for (lang, skos_value) in &skos_concept.pref_labels {
            pref_label.insert(
                lang.clone(),
                RdmValue::new(skos_value.id.clone(), skos_value.value.clone()),
            );
        }

        let narrower: Vec<String> = skos_concept
            .children
            .as_ref()
            .map(|children| children.iter().map(|c| c.id.clone()).collect())
            .unwrap_or_default();

        let broader = parent_id.map(|p| vec![p.to_string()]).unwrap_or_default();

        let rdm_concept = RdmConcept {
            id: skos_concept.id.clone(),
            pref_label,
            alt_labels: HashMap::new(),
            broader,
            narrower,
            scope_note: HashMap::new(),
        };

        rdm.add_concept(rdm_concept);

        if let Some(ref children) = skos_concept.children {
            for child in children {
                add_concept_recursive(rdm, child, Some(&skos_concept.id));
            }
        }
    }

    for skos_concept in skos.concepts.values() {
        add_concept_recursive(&mut rdm, skos_concept, None);
    }

    // Fallback for flat structures (all_concepts without hierarchy)
    if rdm.is_empty() && !skos.all_concepts.is_empty() {
        for skos_concept in skos.all_concepts.values() {
            if !rdm.has_concept(&skos_concept.id) {
                add_concept_recursive(&mut rdm, skos_concept, None);
            }
        }
    }

    rdm
}

/// Convert an `RdmCollection` to a `SkosCollection` for SKOS XML serialization.
///
/// This is the inverse of `skos_to_rdm_collection`. The `node_type` parameter
/// determines whether the output uses `skos:ConceptScheme` (with narrower/broader)
/// or `skos:Collection` (with member relations, Arches-compatible).
pub fn rdm_to_skos_collection(rdm: &RdmCollection, node_type: &str) -> SkosCollection {
    rdm_to_skos_collection_excluding(rdm, node_type, &HashSet::new())
}

/// Convert an [`RdmCollection`] to a [`SkosCollection`], excluding any concept
/// IDs in `exclude_ids`. This is used during export to avoid emitting the same
/// concept in multiple XML files (which causes duplicate-key errors in the
/// `arches_controlled_lists` importer).
pub fn rdm_to_skos_collection_excluding(
    rdm: &RdmCollection,
    node_type: &str,
    exclude_ids: &HashSet<String>,
) -> SkosCollection {
    // Build collection pref_labels
    let mut collection_pref_labels = HashMap::new();
    if let Some(ref name) = rdm.name {
        collection_pref_labels.insert(
            "en".to_string(),
            SkosValue {
                id: generate_value_uuid(&rdm.id, name, "en").to_string(),
                value: name.clone(),
            },
        );
    }

    // Convert all concepts (flat list first), skipping excluded IDs
    let mut all_skos_concepts: HashMap<String, SkosConcept> = HashMap::new();
    let mut all_narrower_ids: HashSet<String> = HashSet::new();

    for concept_id in rdm.get_concept_ids() {
        if exclude_ids.contains(concept_id.as_str()) {
            continue;
        }
        if let Some(rdm_concept) = rdm.get_concept(concept_id) {
            let mut pref_labels = HashMap::new();
            for (lang, rdm_value) in &rdm_concept.pref_label {
                let value_id = if rdm_value.id.is_empty() || rdm_value.id == "__pending__" {
                    generate_value_uuid(concept_id, &rdm_value.value, lang).to_string()
                } else {
                    rdm_value.id.clone()
                };
                pref_labels.insert(
                    lang.clone(),
                    SkosValue {
                        id: value_id,
                        value: rdm_value.value.clone(),
                    },
                );
            }

            let skos_concept = SkosConcept {
                id: concept_id.clone(),
                uri: None,
                pref_labels,
                source: Some(concept_id.clone()),
                sort_order: None,
                children: None,
            };

            all_skos_concepts.insert(concept_id.clone(), skos_concept);
            all_narrower_ids.extend(rdm_concept.narrower.iter().cloned());
        }
    }

    let skos_node_type = if node_type == "Collection" {
        SkosNodeType::Collection
    } else {
        SkosNodeType::ConceptScheme
    };

    // Build hierarchy — top-level concepts are those not in any narrower list.
    // `placed` tracks concepts already claimed by a parent to avoid emitting
    // the same concept under multiple parents (diamond hierarchies).
    let mut hierarchy: HashMap<String, SkosConcept> = HashMap::new();
    let mut placed: HashSet<String> = HashSet::new();

    for concept_id in rdm.get_concept_ids() {
        if exclude_ids.contains(concept_id.as_str()) {
            continue;
        }
        if !all_narrower_ids.contains(concept_id) {
            if let Some(concept_with_children) =
                build_concept_tree_from_rdm(concept_id, &all_skos_concepts, rdm, &mut placed)
            {
                hierarchy.insert(concept_id.clone(), concept_with_children);
            }
        }
    }

    let top_level_concepts = if hierarchy.is_empty() {
        all_skos_concepts.clone()
    } else {
        hierarchy
    };

    SkosCollection {
        id: rdm.id.clone(),
        uri: None,
        pref_labels: collection_pref_labels,
        alt_labels: HashMap::new(),
        scope_notes: HashMap::new(),
        node_type: skos_node_type,
        concepts: top_level_concepts,
        all_concepts: all_skos_concepts,
        values: HashMap::new(),
    }
}

/// Build a concept tree recursively from RDM narrower relationships.
///
/// `placed` tracks concept IDs already claimed by a parent, preventing the
/// same concept from appearing under multiple parents (diamond hierarchies)
/// which would produce duplicate `<skos:Concept>` elements in SKOS XML.
fn build_concept_tree_from_rdm(
    concept_id: &str,
    all_concepts: &HashMap<String, SkosConcept>,
    rdm_collection: &RdmCollection,
    placed: &mut HashSet<String>,
) -> Option<SkosConcept> {
    if !placed.insert(concept_id.to_string()) {
        return None; // already claimed by another parent
    }

    let mut concept = all_concepts.get(concept_id)?.clone();

    if let Some(rdm_concept) = rdm_collection.get_concept(concept_id) {
        if !rdm_concept.narrower.is_empty() {
            let mut children = Vec::new();
            for child_id in &rdm_concept.narrower {
                if let Some(child) =
                    build_concept_tree_from_rdm(child_id, all_concepts, rdm_collection, placed)
                {
                    children.push(child);
                }
            }
            if !children.is_empty() {
                concept.children = Some(children);
            }
        }
    }

    Some(concept)
}

// =============================================================================
// ExternalResolver Implementation
// =============================================================================

use crate::type_serialization::ExternalResolver;

impl ExternalResolver for RdmCache {
    fn resolve_concept(
        &self,
        collection_id: &str,
        concept_id: &str,
        language: &str,
    ) -> Option<String> {
        self.lookup_label(collection_id, concept_id, language)
    }
}

// =============================================================================
// ConceptLookup Implementation
// =============================================================================

use crate::label_resolution::ConceptLookup;

impl ConceptLookup for RdmCache {
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<String> {
        self.collections
            .get(collection_id)
            .and_then(|c| c.find_by_label(label))
            .map(|c| c.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_label_lookup() {
        let mut cache = RdmCache::new();

        let concepts_json = r#"[
            {
                "id": "concept-1",
                "prefLabel": {"en": "English Label", "de": "German Label"}
            },
            {
                "id": "concept-2",
                "prefLabel": {"en": "Second Concept"}
            }
        ]"#;

        cache
            .add_collection_from_json("collection-1", concepts_json)
            .unwrap();

        assert!(cache.has_collection("collection-1"));
        assert!(!cache.has_collection("collection-2"));

        assert_eq!(
            cache.lookup_label("collection-1", "concept-1", "en"),
            Some("English Label".to_string())
        );
        assert_eq!(
            cache.lookup_label("collection-1", "concept-1", "de"),
            Some("German Label".to_string())
        );
        // Fallback to en
        assert_eq!(
            cache.lookup_label("collection-1", "concept-1", "fr"),
            Some("English Label".to_string())
        );
        // Not found
        assert_eq!(cache.lookup_label("collection-1", "concept-3", "en"), None);
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = RdmCache::new();

        cache
            .add_collection_from_json("coll-1", r#"[{"id": "c1", "prefLabel": {"en": "C1"}}]"#)
            .unwrap();
        cache
            .add_collection_from_json("coll-2", r#"[{"id": "c2", "prefLabel": {"en": "C2"}}]"#)
            .unwrap();

        assert_eq!(cache.get_collection_ids().len(), 2);

        cache.clear();
        assert_eq!(cache.get_collection_ids().len(), 0);
    }

    #[test]
    fn test_hierarchical_concepts() {
        let mut collection = RdmCollection::new("coll-1".to_string());

        // Create parent concept (no broader)
        let mut parent_labels = HashMap::new();
        parent_labels.insert(
            "en".to_string(),
            RdmValue::new("v-parent-en".to_string(), "Parent".to_string()),
        );
        let parent = RdmConcept {
            id: "parent".to_string(),
            pref_label: parent_labels,
            alt_labels: HashMap::new(),
            broader: vec![],
            narrower: vec!["child".to_string()],
            scope_note: HashMap::new(),
        };

        // Create child concept (has broader)
        let mut child_labels = HashMap::new();
        child_labels.insert(
            "en".to_string(),
            RdmValue::new("v-child-en".to_string(), "Child".to_string()),
        );
        let child = RdmConcept {
            id: "child".to_string(),
            pref_label: child_labels,
            alt_labels: HashMap::new(),
            broader: vec!["parent".to_string()],
            narrower: vec![],
            scope_note: HashMap::new(),
        };

        collection.add_concept(parent);
        collection.add_concept(child);

        // Collection should have 2 concepts
        assert_eq!(collection.len(), 2);

        // Only parent should be in top_concepts (child has broader)
        let top = collection.get_top_concepts();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].id, "parent");

        // Both concepts should be accessible
        assert!(collection.has_concept("parent"));
        assert!(collection.has_concept("child"));
    }

    #[test]
    fn test_get_concept_mut() {
        let mut collection = RdmCollection::new("coll-1".to_string());

        let mut labels = HashMap::new();
        labels.insert(
            "en".to_string(),
            RdmValue::new("v-c1-en".to_string(), "Original".to_string()),
        );
        let concept = RdmConcept {
            id: "c1".to_string(),
            pref_label: labels,
            alt_labels: HashMap::new(),
            broader: vec![],
            narrower: vec![],
            scope_note: HashMap::new(),
        };

        collection.add_concept(concept);

        // Modify the concept
        if let Some(c) = collection.get_concept_mut("c1") {
            c.narrower.push("c2".to_string());
            c.pref_label.insert(
                "de".to_string(),
                RdmValue::new("v-c1-de".to_string(), "Geändert".to_string()),
            );
        }

        // Verify changes persisted
        let c = collection.get_concept("c1").unwrap();
        assert_eq!(c.narrower, vec!["c2".to_string()]);
        assert_eq!(
            c.pref_label.get("de").map(|v| v.value.as_str()),
            Some("Geändert")
        );
    }

    #[test]
    fn test_add_child_concept_hierarchy() {
        let mut collection = RdmCollection::new("coll-1".to_string());

        // Add parent
        let mut parent_labels = HashMap::new();
        parent_labels.insert(
            "en".to_string(),
            RdmValue::new("v-animals-en".to_string(), "Animals".to_string()),
        );
        let parent = RdmConcept {
            id: "animals".to_string(),
            pref_label: parent_labels,
            alt_labels: HashMap::new(),
            broader: vec![],
            narrower: vec![],
            scope_note: HashMap::new(),
        };
        collection.add_concept(parent);

        // Update parent's narrower list
        if let Some(p) = collection.get_concept_mut("animals") {
            p.narrower.push("mammals".to_string());
        }

        // Add child with broader pointing to parent
        let mut child_labels = HashMap::new();
        child_labels.insert(
            "en".to_string(),
            RdmValue::new("v-mammals-en".to_string(), "Mammals".to_string()),
        );
        let child = RdmConcept {
            id: "mammals".to_string(),
            pref_label: child_labels,
            alt_labels: HashMap::new(),
            broader: vec!["animals".to_string()],
            narrower: vec![],
            scope_note: HashMap::new(),
        };
        collection.add_concept(child);

        // Verify hierarchy
        let top = collection.get_top_concepts();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].id, "animals");
        assert_eq!(top[0].narrower, vec!["mammals".to_string()]);

        let child = collection.get_concept("mammals").unwrap();
        assert_eq!(child.broader, vec!["animals".to_string()]);
    }

    #[test]
    fn test_value_id_lookup() {
        let mut cache = RdmCache::new();

        // JSON with explicit value IDs
        let concepts_json = r#"[
            {
                "id": "concept-1",
                "prefLabels": {
                    "en": { "id": "value-1-en", "value": "English Label" },
                    "de": { "id": "value-1-de", "value": "German Label" }
                }
            },
            {
                "id": "concept-2",
                "prefLabels": {
                    "en": { "id": "value-2-en", "value": "Second Concept" }
                }
            }
        ]"#;

        cache
            .add_collection_from_json("collection-1", concepts_json)
            .unwrap();

        // Look up by value ID
        let value = cache.lookup_value("collection-1", "value-1-en").unwrap();
        assert_eq!(value.id, "value-1-en");
        assert_eq!(value.value, "English Label");
        assert_eq!(value.concept_id, "concept-1");
        assert_eq!(value.language, "en");

        // Get concept ID from value ID
        assert_eq!(
            cache.get_concept_id_for_value("collection-1", "value-1-de"),
            Some("concept-1")
        );

        // Non-existent value ID
        assert!(cache.lookup_value("collection-1", "nonexistent").is_none());
        assert!(cache
            .get_concept_id_for_value("collection-1", "nonexistent")
            .is_none());

        // Validate existence
        assert!(cache.validate_value("collection-1", "value-2-en"));
        assert!(!cache.validate_value("collection-1", "nonexistent"));
    }

    #[test]
    fn test_simple_preflabel_format_generates_ids() {
        let mut cache = RdmCache::new();

        // JSON with simple string format (no value IDs)
        let concepts_json = r#"[
            {
                "id": "concept-1",
                "prefLabel": {"en": "Label One", "de": "Etikett Eins"}
            }
        ]"#;

        cache
            .add_collection_from_json("collection-1", concepts_json)
            .unwrap();

        // Value IDs should be generated deterministically
        let collection = cache.get_collection("collection-1").unwrap();
        let concept = collection.get_concept("concept-1").unwrap();

        // Check that value IDs were generated (not __pending__)
        let en_value = concept.pref_label.get("en").unwrap();
        assert_ne!(en_value.id, "__pending__");
        assert_eq!(en_value.value, "Label One");

        // Should be able to look up by the generated value ID
        let looked_up = collection.get_value_by_id(&en_value.id).unwrap();
        assert_eq!(looked_up.value, "Label One");
        assert_eq!(looked_up.concept_id, "concept-1");
    }

    #[test]
    fn test_get_parent_id() {
        let mut cache = RdmCache::new();

        let concepts_json = r#"[
            {
                "id": "parent-concept",
                "prefLabel": {"en": "Parent"}
            },
            {
                "id": "child-concept",
                "prefLabel": {"en": "Child"},
                "broader": ["parent-concept"]
            }
        ]"#;

        cache
            .add_collection_from_json("coll-1", concepts_json)
            .unwrap();

        // Child should have parent
        assert_eq!(
            cache.get_parent_id("coll-1", "child-concept"),
            Some("parent-concept".to_string())
        );

        // Parent has no parent (top-level)
        assert_eq!(cache.get_parent_id("coll-1", "parent-concept"), None);

        // Non-existent concept
        assert_eq!(cache.get_parent_id("coll-1", "nonexistent"), None);
    }
}
