//! Resource types for resource instances and metadata.

use super::descriptors::StaticResourceDescriptors;
use super::tile::StaticTile;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Metadata about a resource instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceMetadata {
    pub descriptors: StaticResourceDescriptors,
    pub graph_id: String,
    pub name: String,
    pub resourceinstanceid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmodified: Option<String>,
}

/// Summary info for a resource (used for lazy loading)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceSummary {
    pub resourceinstanceid: String,
    pub graph_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptors: Option<StaticResourceDescriptors>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmodified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_publication_id: Option<String>,
}

impl StaticResourceSummary {
    /// Convert summary to metadata
    pub fn to_metadata(&self) -> StaticResourceMetadata {
        StaticResourceMetadata {
            descriptors: self.descriptors.clone().unwrap_or_default(),
            graph_id: self.graph_id.clone(),
            name: self.name.clone(),
            resourceinstanceid: self.resourceinstanceid.clone(),
            publication_id: self.publication_id.clone(),
            principaluser_id: self.principaluser_id,
            legacyid: self.legacyid.clone(),
            graph_publication_id: self.graph_publication_id.clone(),
            createdtime: self.createdtime.clone(),
            lastmodified: self.lastmodified.clone(),
        }
    }
}

/// Reference to another resource instance (for resource-instance datatype)
///
/// Used in ResourceInstanceViewModel to represent relationships between resources.
/// Can include the full resource tree if cascade is enabled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceReference {
    /// Resource instance ID
    pub id: String,
    /// Graph ID for the resource model
    #[serde(rename = "graphId")]
    pub graph_id: String,
    /// Resource model type/name (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub resource_type: Option<String>,
    /// Display title for the resource (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Full resource tree data (when cascaded, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<serde_json::Value>,
    /// Additional metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl StaticResourceReference {
    /// Create a minimal resource reference with just ID and graph ID
    pub fn new(id: String, graph_id: String) -> Self {
        StaticResourceReference {
            id,
            graph_id,
            resource_type: None,
            title: None,
            root: None,
            meta: None,
        }
    }

    /// Create a reference with type information
    pub fn with_type(id: String, graph_id: String, resource_type: String) -> Self {
        StaticResourceReference {
            id,
            graph_id,
            resource_type: Some(resource_type),
            title: None,
            root: None,
            meta: None,
        }
    }

    /// Add title to reference (builder pattern)
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Add metadata to reference (builder pattern)
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Add root data to reference for cascaded loading (builder pattern)
    pub fn with_root(mut self, root: serde_json::Value) -> Self {
        self.root = Some(root);
        self
    }
}

/// Complete resource data with tiles
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResource {
    pub resourceinstance: StaticResourceMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<StaticTile>>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    // Optional cache and scopes - stored as JSON for platform independence
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "__cache")]
    pub cache: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "__scopes")]
    pub scopes: Option<serde_json::Value>,

    // Tracking flag for lazy loading
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tiles_loaded: Option<bool>,
}

impl StaticResource {
    /// Convert to a summary (for registry storage)
    pub fn to_summary(&self) -> StaticResourceSummary {
        StaticResourceSummary {
            resourceinstanceid: self.resourceinstance.resourceinstanceid.clone(),
            graph_id: self.resourceinstance.graph_id.clone(),
            name: self.resourceinstance.name.clone(),
            descriptors: Some(self.resourceinstance.descriptors.clone()),
            metadata: self.metadata.clone(),
            createdtime: self.resourceinstance.createdtime.clone(),
            lastmodified: self.resourceinstance.lastmodified.clone(),
            publication_id: self.resourceinstance.publication_id.clone(),
            principaluser_id: self.resourceinstance.principaluser_id,
            legacyid: self.resourceinstance.legacyid.clone(),
            graph_publication_id: self.resourceinstance.graph_publication_id.clone(),
        }
    }
}

/// Cache entry for a related resource, matching ResourceInstanceCacheEntry from TypeScript.
///
/// This structure is compatible with TypeScript's getValueCache format,
/// allowing direct lookup by tileId/nodeId when rendering ViewModels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelatedResourceEntry {
    /// Datatype marker (always "resource-instance")
    pub datatype: String,
    /// Resource instance ID (UUID)
    pub id: String,
    /// Model class name (derived from graph name, e.g., "Person")
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Graph ID
    #[serde(rename = "graphId")]
    pub graph_id: String,
    /// Display title (resource name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl RelatedResourceEntry {
    /// Create from a resource entry with optional model class name
    pub fn from_resource_entry(entry: &ResourceEntry, model_class_name: Option<&str>) -> Self {
        RelatedResourceEntry {
            datatype: "resource-instance".to_string(),
            id: entry.resourceinstanceid().to_string(),
            resource_type: model_class_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| entry.graph_id().to_string()),
            graph_id: entry.graph_id().to_string(),
            title: Some(entry.name().to_string()),
            meta: None,
        }
    }

    /// Create from a resource summary with optional model class name
    pub fn from_summary(summary: &StaticResourceSummary, model_class_name: Option<&str>) -> Self {
        RelatedResourceEntry {
            datatype: "resource-instance".to_string(),
            id: summary.resourceinstanceid.clone(),
            resource_type: model_class_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| summary.graph_id.clone()),
            graph_id: summary.graph_id.clone(),
            title: Some(summary.name.clone()),
            meta: None,
        }
    }

    /// Get the resource instance ID
    pub fn resourceinstanceid(&self) -> &str {
        &self.id
    }
}

impl From<&StaticResourceSummary> for RelatedResourceEntry {
    fn from(summary: &StaticResourceSummary) -> Self {
        RelatedResourceEntry::from_summary(summary, None)
    }
}

impl From<&StaticResource> for RelatedResourceEntry {
    fn from(resource: &StaticResource) -> Self {
        RelatedResourceEntry {
            datatype: "resource-instance".to_string(),
            id: resource.resourceinstance.resourceinstanceid.clone(),
            resource_type: resource.resourceinstance.graph_id.clone(),
            graph_id: resource.resourceinstance.graph_id.clone(),
            title: Some(resource.resourceinstance.name.clone()),
            meta: None,
        }
    }
}

impl From<RelatedResourceEntry> for StaticResourceSummary {
    /// Hydrate a cache entry back to a full summary (with optional fields as None/empty)
    fn from(entry: RelatedResourceEntry) -> Self {
        StaticResourceSummary {
            resourceinstanceid: entry.id,
            graph_id: entry.graph_id,
            name: entry.title.unwrap_or_default(),
            descriptors: None,
            metadata: HashMap::new(),
            createdtime: None,
            lastmodified: None,
            publication_id: None,
            principaluser_id: None,
            legacyid: None,
            graph_publication_id: None,
        }
    }
}

impl From<&ResourceEntry> for RelatedResourceEntry {
    fn from(entry: &ResourceEntry) -> Self {
        RelatedResourceEntry::from_resource_entry(entry, None)
    }
}

/// Cache entry for resource-instance-list nodes.
///
/// Contains an array of resource entries to match TypeScript's ResourceInstanceListCacheEntry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelatedResourceListEntry {
    /// Datatype marker (always "resource-instance-list")
    pub datatype: String,
    /// List of resource entries
    #[serde(rename = "_")]
    pub entries: Vec<RelatedResourceEntry>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl RelatedResourceListEntry {
    /// Create a new empty list entry
    pub fn new() -> Self {
        RelatedResourceListEntry {
            datatype: "resource-instance-list".to_string(),
            entries: Vec::new(),
            meta: None,
        }
    }

    /// Add an entry to the list
    pub fn push(&mut self, entry: RelatedResourceEntry) {
        self.entries.push(entry);
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for RelatedResourceListEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache entry that can be either a single resource or a list of resources.
///
/// Uses untagged serialization to match TypeScript's expected format:
/// - resource-instance: `{datatype: "resource-instance", id, type, graphId, title}`
/// - resource-instance-list: `{datatype: "resource-instance-list", _: [...], meta}`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CacheEntry {
    /// Single resource reference (for resource-instance datatype)
    Single(RelatedResourceEntry),
    /// List of resource references (for resource-instance-list datatype)
    List(RelatedResourceListEntry),
}

/// Cache structure for __cache field, matching TypeScript's getValueCache format.
///
/// Structure: { tileId: { nodeId: CacheEntry, ... }, ... }
///
/// This allows direct lookup in TypeScript via:
/// `cacheEntries[tile.tileid][node.nodeid]`
pub type ResourceCache = HashMap<String, HashMap<String, CacheEntry>>;

/// Reference to an unknown resource found during cache population
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnknownReference {
    /// Resource that contains the reference
    pub source_resource_id: String,
    /// Node ID where the reference was found
    pub node_id: String,
    /// Node alias (if any)
    pub node_alias: Option<String>,
    /// The unknown resource ID that was referenced
    pub referenced_id: String,
}

/// Result of populate_caches operation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PopulateCachesResult {
    /// References to resources not found in the registry
    pub unknown_references: Vec<UnknownReference>,
}

impl PopulateCachesResult {
    /// Check if there were any unknown references
    pub fn has_unknown_references(&self) -> bool {
        !self.unknown_references.is_empty()
    }

    /// Get error messages for unknown references
    pub fn error_messages(&self) -> Vec<String> {
        self.unknown_references
            .iter()
            .map(|r| {
                let node_desc = r
                    .node_alias
                    .as_ref()
                    .map(|a| format!("node '{}' ({})", a, r.node_id))
                    .unwrap_or_else(|| format!("node '{}'", r.node_id));
                format!(
                    "Resource '{}': {} references unknown resource '{}'",
                    r.source_resource_id, node_desc, r.referenced_id
                )
            })
            .collect()
    }
}

/// Entry in the resource registry - either full resource or summary only
///
/// This allows the registry to store minimal summaries (memory efficient) or
/// full resources with tiles (for traversal), similar to staticStore's cacheMetadataOnly pattern.
#[derive(Clone, Debug)]
pub enum ResourceEntry {
    /// Summary only - minimal memory, no tiles
    Summary(StaticResourceSummary),
    /// Full resource with tiles
    Full(Box<StaticResource>),
}

impl ResourceEntry {
    /// Get the resource instance ID
    pub fn resourceinstanceid(&self) -> &str {
        match self {
            ResourceEntry::Summary(s) => &s.resourceinstanceid,
            ResourceEntry::Full(r) => &r.resourceinstance.resourceinstanceid,
        }
    }

    /// Get the graph ID
    pub fn graph_id(&self) -> &str {
        match self {
            ResourceEntry::Summary(s) => &s.graph_id,
            ResourceEntry::Full(r) => &r.resourceinstance.graph_id,
        }
    }

    /// Get the resource name
    pub fn name(&self) -> &str {
        match self {
            ResourceEntry::Summary(s) => &s.name,
            ResourceEntry::Full(r) => &r.resourceinstance.name,
        }
    }

    /// Check if this entry has tiles (is a full resource with tiles loaded)
    pub fn has_tiles(&self) -> bool {
        match self {
            ResourceEntry::Summary(_) => false,
            ResourceEntry::Full(r) => r.tiles.as_ref().map(|t| !t.is_empty()).unwrap_or(false),
        }
    }

    /// Check if this is a full resource entry
    pub fn is_full(&self) -> bool {
        matches!(self, ResourceEntry::Full(_))
    }

    /// Get as full resource reference (if available)
    pub fn as_full(&self) -> Option<&StaticResource> {
        match self {
            ResourceEntry::Full(r) => Some(r),
            ResourceEntry::Summary(_) => None,
        }
    }

    /// Get as full resource mutable reference (if available)
    pub fn as_full_mut(&mut self) -> Option<&mut StaticResource> {
        match self {
            ResourceEntry::Full(r) => Some(r),
            ResourceEntry::Summary(_) => None,
        }
    }

    /// Convert to summary (extracts summary from full resource if needed)
    pub fn to_summary(&self) -> StaticResourceSummary {
        match self {
            ResourceEntry::Summary(s) => s.clone(),
            ResourceEntry::Full(r) => r.to_summary(),
        }
    }

    /// Convert to minimal cache entry
    pub fn to_cache_entry(&self) -> RelatedResourceEntry {
        match self {
            ResourceEntry::Summary(s) => RelatedResourceEntry::from(s),
            ResourceEntry::Full(r) => RelatedResourceEntry::from(r.as_ref()),
        }
    }
}

impl From<StaticResourceSummary> for ResourceEntry {
    fn from(summary: StaticResourceSummary) -> Self {
        ResourceEntry::Summary(summary)
    }
}

impl From<StaticResource> for ResourceEntry {
    fn from(resource: StaticResource) -> Self {
        ResourceEntry::Full(Box::new(resource))
    }
}

/// In-memory registry of resources for relationship resolution and caching
///
/// Stores either full resources or summaries, allowing memory-efficient storage
/// when only metadata is needed, with the ability to upgrade to full resources
/// when tiles are required.
///
/// Used to:
/// - Look up graph_id for referenced resources
/// - Populate __cache on resources with related resource summaries
/// - Enrich resource-instance tile data with ontologyProperty from node config
/// - Cache full resources for traversal (like staticStore)
#[derive(Clone, Debug, Default)]
pub struct StaticResourceRegistry {
    resources: HashMap<String, ResourceEntry>,
}

impl StaticResourceRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Get the graph_id for a resource
    pub fn get_graph_id(&self, resource_id: &str) -> Option<&str> {
        self.resources.get(resource_id).map(|e| e.graph_id())
    }

    /// Get the entry for a resource
    pub fn get(&self, resource_id: &str) -> Option<&ResourceEntry> {
        self.resources.get(resource_id)
    }

    /// Get a mutable entry for a resource
    pub fn get_mut(&mut self, resource_id: &str) -> Option<&mut ResourceEntry> {
        self.resources.get_mut(resource_id)
    }

    /// Get the full resource if available (returns None if only summary stored)
    pub fn get_full(&self, resource_id: &str) -> Option<&StaticResource> {
        self.resources.get(resource_id).and_then(|e| e.as_full())
    }

    /// Get a summary for a resource (works for both summary and full entries)
    pub fn get_summary(&self, resource_id: &str) -> Option<StaticResourceSummary> {
        self.resources.get(resource_id).map(|e| e.to_summary())
    }

    /// Check if a resource is known
    pub fn contains(&self, resource_id: &str) -> bool {
        self.resources.contains_key(resource_id)
    }

    /// Check if a resource has full data with tiles
    pub fn has_full(&self, resource_id: &str) -> bool {
        self.resources
            .get(resource_id)
            .map(|e| e.is_full())
            .unwrap_or(false)
    }

    /// Number of resources in the registry
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Add a single resource summary (won't overwrite full resources)
    pub fn insert_summary(&mut self, summary: StaticResourceSummary) {
        let id = summary.resourceinstanceid.clone();
        // Don't downgrade full → summary
        if !self.has_full(&id) {
            self.resources.insert(id, ResourceEntry::Summary(summary));
        }
    }

    /// Add a single resource summary (legacy alias for insert_summary)
    pub fn insert(&mut self, summary: StaticResourceSummary) {
        self.insert_summary(summary);
    }

    /// Add a full resource (always overwrites, as it's more complete)
    pub fn insert_full(&mut self, resource: StaticResource) {
        let id = resource.resourceinstance.resourceinstanceid.clone();
        self.resources
            .insert(id, ResourceEntry::Full(Box::new(resource)));
    }

    /// Upgrade a summary to a full resource (if the resource exists)
    pub fn upgrade_to_full(&mut self, resource: StaticResource) {
        let id = resource.resourceinstance.resourceinstanceid.clone();
        self.resources
            .insert(id, ResourceEntry::Full(Box::new(resource)));
    }

    /// Merge resources into registry
    ///
    /// - If store_full is true, stores full resources (for traversal)
    /// - If store_full is false, stores only summaries (memory efficient)
    /// - If include_caches is true, also merges any __cache.relatedResources as summaries
    pub fn merge_from_resources(
        &mut self,
        resources: &[StaticResource],
        store_full: bool,
        include_caches: bool,
    ) {
        for resource in resources {
            // Register the resource itself
            if store_full {
                self.insert_full(resource.clone());
            } else {
                self.insert_summary(resource.to_summary());
            }

            // Merge from __cache if present and requested (always as summaries)
            if include_caches {
                if let Some(ref cache_json) = resource.cache {
                    if let Ok(cache) = serde_json::from_value::<ResourceCache>(cache_json.clone()) {
                        // Cache is now keyed by tileId -> nodeId -> entry
                        for (_tile_id, node_entries) in cache {
                            for (_node_id, cache_entry) in node_entries {
                                // Extract entries based on cache entry type
                                let entries: Vec<&RelatedResourceEntry> = match &cache_entry {
                                    CacheEntry::Single(entry) => vec![entry],
                                    CacheEntry::List(list) => list.entries.iter().collect(),
                                };

                                for entry in entries {
                                    let id = entry.id.clone();
                                    // Don't overwrite existing entries (first wins, and don't downgrade)
                                    if !self.resources.contains_key(&id) {
                                        self.resources.insert(
                                            id,
                                            ResourceEntry::Summary(StaticResourceSummary::from(
                                                entry.clone(),
                                            )),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ResourceEntry)> {
        self.resources.iter()
    }

    /// Iterate over all full resources
    pub fn iter_full(&self) -> impl Iterator<Item = (&String, &StaticResource)> {
        self.resources
            .iter()
            .filter_map(|(id, entry)| entry.as_full().map(|r| (id, r)))
    }

    /// Get all resource IDs
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.resources.keys()
    }

    /// Populate __cache on resources with summaries for referenced resources
    ///
    /// Uses the graph to identify resource-instance/resource-instance-list nodes,
    /// then populates cache entries for each referenced resource.
    ///
    /// If `enrich_relationships` is true, also adds ontologyProperty/inverseOntologyProperty
    /// to tile data based on node config and the target resource's graph.
    ///
    /// Returns information about unknown references found during processing.
    pub fn populate_caches(
        &self,
        resources: &mut [StaticResource],
        graph: &super::StaticGraph,
        enrich_relationships: bool,
    ) -> PopulateCachesResult {
        let mut result = PopulateCachesResult::default();

        for resource in resources.iter_mut() {
            let mut cache: ResourceCache = HashMap::new();
            let resource_id = resource.resourceinstance.resourceinstanceid.clone();

            if let Some(ref mut tiles) = resource.tiles {
                for tile in tiles.iter_mut() {
                    // Get tile ID (generate if missing)
                    let tile_id = tile
                        .tileid
                        .clone()
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                    // Get all nodes in this nodegroup
                    let nodes = graph.get_nodes_in_nodegroup(&tile.nodegroup_id);

                    for node in nodes {
                        // Only process resource-instance datatypes
                        if node.datatype != "resource-instance"
                            && node.datatype != "resource-instance-list"
                        {
                            continue;
                        }

                        // Get tile data for this node
                        if let Some(data) = tile.data.get_mut(&node.nodeid) {
                            self.process_resource_instance_data(
                                data,
                                node,
                                &tile_id,
                                &mut cache,
                                enrich_relationships,
                                &resource_id,
                                &mut result,
                            );
                        }
                    }
                }
            }

            // Merge with existing cache if present
            if !cache.is_empty() {
                if let Some(ref existing_json) = resource.cache {
                    if let Ok(existing) =
                        serde_json::from_value::<ResourceCache>(existing_json.clone())
                    {
                        // Merge existing into new cache (new wins for conflicts)
                        for (tile_id, node_entries) in existing {
                            let tile_cache = cache.entry(tile_id).or_default();
                            for (node_id, entry) in node_entries {
                                tile_cache.entry(node_id).or_insert(entry);
                            }
                        }
                    }
                }
                resource.cache = serde_json::to_value(&cache).ok();
            }
        }

        result
    }

    /// Process resource-instance data: populate cache and optionally enrich with relationship properties
    fn process_resource_instance_data(
        &self,
        data: &mut serde_json::Value,
        node: &super::StaticNode,
        tile_id: &str,
        cache: &mut ResourceCache,
        enrich_relationships: bool,
        source_resource_id: &str,
        result: &mut PopulateCachesResult,
    ) {
        let is_list = node.datatype == "resource-instance-list";

        // For lists, collect all entries first
        let mut list_entries: Vec<RelatedResourceEntry> = Vec::new();

        // resource-instance data is an array of {resourceId: "..."}
        if let Some(arr) = data.as_array_mut() {
            for entry in arr.iter_mut() {
                if let Some(resource_id) = entry.get("resourceId").and_then(|r| r.as_str()) {
                    // Add to cache if we know this resource
                    if let Some(resource_entry) = self.resources.get(resource_id) {
                        // Get model class name from graph registry if available
                        let model_class_name = crate::get_graph(resource_entry.graph_id())
                            .and_then(|g| g.get_model_class_name());

                        let related_entry = RelatedResourceEntry::from_resource_entry(
                            resource_entry,
                            model_class_name.as_deref(),
                        );

                        if is_list {
                            // Collect entries for list
                            list_entries.push(related_entry);
                        } else {
                            // Store single entry in cache keyed by tileId -> nodeId
                            let tile_cache = cache.entry(tile_id.to_string()).or_default();
                            tile_cache
                                .insert(node.nodeid.clone(), CacheEntry::Single(related_entry));
                        }

                        // Enrich with relationship properties if requested
                        if enrich_relationships {
                            self.enrich_entry_with_relationship(entry, resource_entry, node);
                        }
                    } else {
                        // Track unknown reference
                        result.unknown_references.push(UnknownReference {
                            source_resource_id: source_resource_id.to_string(),
                            node_id: node.nodeid.clone(),
                            node_alias: node.alias.clone(),
                            referenced_id: resource_id.to_string(),
                        });
                    }
                }
            }
        }

        // For list datatype, store all collected entries as a list
        if is_list && !list_entries.is_empty() {
            let tile_cache = cache.entry(tile_id.to_string()).or_default();
            tile_cache.insert(
                node.nodeid.clone(),
                CacheEntry::List(RelatedResourceListEntry {
                    datatype: "resource-instance-list".to_string(),
                    entries: list_entries,
                    meta: None,
                }),
            );
        }
    }

    /// Add ontologyProperty/inverseOntologyProperty to a resource-instance entry
    /// based on node config and target resource's graph
    fn enrich_entry_with_relationship(
        &self,
        entry: &mut serde_json::Value,
        target_entry: &ResourceEntry,
        node: &super::StaticNode,
    ) {
        // Skip if already has ontologyProperty
        if entry.get("ontologyProperty").is_some() {
            return;
        }

        // Get node config graphs array
        let graphs = match node.config.get("graphs").and_then(|g| g.as_array()) {
            Some(g) => g,
            None => return,
        };

        // Find matching graph config for target resource's graph_id
        let target_graph_id = target_entry.graph_id();
        let graph_config = graphs.iter().find(|g| {
            g.get("graphid")
                .and_then(|id| id.as_str())
                .map(|id| id == target_graph_id)
                .unwrap_or(false)
        });

        let graph_config = match graph_config {
            Some(g) => g,
            None => return, // Target graph not configured for this node
        };

        // Determine which properties to use based on useOntologyRelationship
        let use_ontology = graph_config
            .get("useOntologyRelationship")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let (ont_key, inv_key) = if use_ontology {
            ("ontologyProperty", "inverseOntologyProperty")
        } else {
            ("relationshipConcept", "inverseRelationshipConcept")
        };

        // Add properties to entry (using ontologyProperty key for Arches compatibility)
        if let Some(prop) = graph_config.get(ont_key).and_then(|v| v.as_str()) {
            if !prop.is_empty() {
                entry["ontologyProperty"] = serde_json::json!(prop);
            }
        }
        if let Some(prop) = graph_config.get(inv_key).and_then(|v| v.as_str()) {
            if !prop.is_empty() {
                entry["inverseOntologyProperty"] = serde_json::json!(prop);
            }
        }
    }

    /// Build an index from resource IDs to node values for a given node.
    ///
    /// Efficiently iterates through tiles, filtering by nodegroup and extracting
    /// values for the specified node.
    ///
    /// # Arguments
    /// * `graph` - The graph to use for node lookup
    /// * `node_identifier` - Node alias or node ID to extract values for
    ///
    /// # Returns
    /// * `Ok(HashMap<String, Vec<Value>>)` - Map from resource_id to list of values
    /// * `Err(String)` - Error if node not found
    pub fn get_node_values_index(
        &self,
        graph: &super::StaticGraph,
        node_identifier: &str,
    ) -> Result<HashMap<String, Vec<serde_json::Value>>, String> {
        // Find the node by alias or ID
        let node = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some(node_identifier) || n.nodeid == node_identifier)
            .ok_or_else(|| {
                format!(
                    "Node '{}' not found in graph {}",
                    node_identifier, graph.graphid
                )
            })?;

        let node_id = &node.nodeid;
        let nodegroup_id = node
            .nodegroup_id
            .as_ref()
            .ok_or_else(|| format!("Node '{}' has no nodegroup_id", node_identifier))?;

        let mut index: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

        for (_, resource) in self.iter_full() {
            // Filter by graph
            if resource.resourceinstance.graph_id != graph.graphid {
                continue;
            }

            let resource_id = &resource.resourceinstance.resourceinstanceid;

            // Find tiles matching the nodegroup
            if let Some(ref tiles) = resource.tiles {
                for tile in tiles {
                    if tile.nodegroup_id.as_str() == nodegroup_id {
                        if let Some(value) = tile.data.get(node_id) {
                            index
                                .entry(resource_id.clone())
                                .or_default()
                                .push(value.clone());
                        }
                    }
                }
            }
        }

        Ok(index)
    }

    /// Build an inverted index from node values to resource IDs.
    ///
    /// Useful for looking up resources by a field value (e.g., find resource by external ID).
    ///
    /// # Arguments
    /// * `graph` - The graph to use for node lookup
    /// * `node_identifier` - Node alias or node ID to extract values for
    /// * `flatten_localized` - If true, extract string from localized values {"en": "value"}
    ///
    /// # Returns
    /// * `Ok(HashMap<String, Vec<String>>)` - Map from value to list of resource_ids
    /// * `Err(String)` - Error if node not found
    pub fn get_value_to_resources_index(
        &self,
        graph: &super::StaticGraph,
        node_identifier: &str,
        flatten_localized: bool,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        // Find the node by alias or ID
        let node = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some(node_identifier) || n.nodeid == node_identifier)
            .ok_or_else(|| {
                format!(
                    "Node '{}' not found in graph {}",
                    node_identifier, graph.graphid
                )
            })?;

        let node_id = &node.nodeid;
        let nodegroup_id = node
            .nodegroup_id
            .as_ref()
            .ok_or_else(|| format!("Node '{}' has no nodegroup_id", node_identifier))?;

        let mut index: HashMap<String, Vec<String>> = HashMap::new();

        for (_, resource) in self.iter_full() {
            // Filter by graph
            if resource.resourceinstance.graph_id != graph.graphid {
                continue;
            }

            let resource_id = &resource.resourceinstance.resourceinstanceid;

            // Find tiles matching the nodegroup
            if let Some(ref tiles) = resource.tiles {
                for tile in tiles {
                    if tile.nodegroup_id.as_str() == nodegroup_id {
                        if let Some(value) = tile.data.get(node_id) {
                            // Convert value to string key
                            let key = if flatten_localized {
                                // Try to extract from localized format {"en": "value"}
                                if let Some(obj) = value.as_object() {
                                    obj.get("en")
                                        .or_else(|| obj.values().next())
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                } else {
                                    value.as_str().map(|s| s.to_string())
                                }
                            } else {
                                // Use JSON representation as key
                                Some(value.to_string())
                            };

                            if let Some(k) = key {
                                index.entry(k).or_default().push(resource_id.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(index)
    }
}

/// Result of merging multiple resources (single resourceinstanceid)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MergeResult {
    /// The merged resource with combined tiles
    pub resource: StaticResource,
    /// Warnings about duplicate tileids that were skipped
    pub warnings: Vec<String>,
}

/// Result of batch merging resources grouped by resourceinstanceid
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchMergeResult {
    /// Merged resources, one per unique resourceinstanceid
    pub resources: Vec<StaticResource>,
    /// All warnings from merging (including which resource had issues)
    pub warnings: Vec<String>,
    /// Fatal error message if strict mode aborted early
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Merge multiple StaticResources into one
///
/// All resources must have the same `resourceinstanceid`. Tiles are concatenated,
/// with duplicate `tileid` values detected and skipped (first occurrence kept).
///
/// # Arguments
/// * `resources` - Vector of StaticResources to merge
///
/// # Returns
/// * `Ok(MergeResult)` - Merged resource and any warnings about duplicates
/// * `Err(String)` - Error if resources is empty or IDs don't match
///
/// # Example
/// ```ignore
/// let result = merge_resources(vec![resource1, resource2])?;
/// if !result.warnings.is_empty() {
///     eprintln!("Merge warnings: {:?}", result.warnings);
/// }
/// let merged = result.resource;
/// ```
pub fn merge_resources(resources: Vec<StaticResource>) -> Result<MergeResult, String> {
    if resources.is_empty() {
        return Err("No resources to merge".to_string());
    }

    // Clone first resource's metadata before we consume the vector
    let first_instance = resources[0].resourceinstance.clone();
    let resource_id = first_instance.resourceinstanceid.clone();

    // Verify all resources have the same resourceinstanceid
    for (i, r) in resources.iter().enumerate().skip(1) {
        if r.resourceinstance.resourceinstanceid != resource_id {
            return Err(format!(
                "Resource ID mismatch at index {}: expected '{}', found '{}'",
                i, resource_id, r.resourceinstance.resourceinstanceid
            ));
        }
    }

    let mut seen_tileids: HashSet<String> = HashSet::new();
    let mut merged_tiles: Vec<StaticTile> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut merged_metadata: HashMap<String, String> = HashMap::new();
    let mut merged_cache: ResourceCache = ResourceCache::default();
    let mut merged_scopes: Option<serde_json::Value> = None;
    let mut first_scopes_index: Option<usize> = None;

    for (i, resource) in resources.into_iter().enumerate() {
        // Merge tiles with duplicate detection
        if let Some(tiles) = resource.tiles {
            for tile in tiles {
                if let Some(ref tileid) = tile.tileid {
                    if seen_tileids.contains(tileid) {
                        warnings.push(format!("Duplicate tileid '{}' skipped", tileid));
                        continue;
                    }
                    seen_tileids.insert(tileid.clone());
                }
                merged_tiles.push(tile);
            }
        }

        // Merge metadata dicts (later values override earlier ones)
        for (key, value) in resource.metadata {
            if let Some(existing) = merged_metadata.get(&key) {
                if existing != &value {
                    warnings.push(format!(
                        "Metadata key '{}' has conflicting values: '{}' vs '{}' (using latter)",
                        key, existing, value
                    ));
                }
            }
            merged_metadata.insert(key, value);
        }

        // Handle scopes: warn if different, use first non-None value
        if let Some(ref scopes) = resource.scopes {
            match &merged_scopes {
                None => {
                    merged_scopes = Some(scopes.clone());
                    first_scopes_index = Some(i);
                }
                Some(existing) if existing != scopes => {
                    warnings.push(format!(
                        "Scopes mismatch: resource {} has different scopes than resource {} (using first)",
                        i, first_scopes_index.unwrap_or(0)
                    ));
                }
                _ => {}
            }
        }

        // Merge cache entries (first wins for conflicts)
        // Cache is now keyed by tileId -> nodeId -> entry
        if let Some(ref cache_json) = resource.cache {
            if let Ok(cache) = serde_json::from_value::<ResourceCache>(cache_json.clone()) {
                for (tile_id, node_entries) in cache {
                    let tile_cache = merged_cache.entry(tile_id).or_default();
                    for (node_id, entry) in node_entries {
                        // First wins - don't overwrite existing entries
                        tile_cache.entry(node_id).or_insert(entry);
                    }
                }
            }
        }
    }

    // Sort merged tiles by (nodegroup_id, sortorder) for consistent ordering
    merged_tiles.sort_by(|a, b| {
        let ng_cmp = a.nodegroup_id.cmp(&b.nodegroup_id);
        if ng_cmp != std::cmp::Ordering::Equal {
            return ng_cmp;
        }
        let a_sort = a.sortorder.unwrap_or(i32::MAX);
        let b_sort = b.sortorder.unwrap_or(i32::MAX);
        a_sort.cmp(&b_sort)
    });

    // Convert merged_cache to JSON value if non-empty
    let final_cache = if merged_cache.is_empty() {
        None
    } else {
        serde_json::to_value(&merged_cache).ok()
    };

    Ok(MergeResult {
        resource: StaticResource {
            resourceinstance: first_instance,
            tiles: Some(merged_tiles),
            metadata: merged_metadata,
            cache: final_cache,
            scopes: merged_scopes,
            tiles_loaded: Some(true),
        },
        warnings,
    })
}

/// Batch merge resources from multiple sources, grouping by resourceinstanceid
///
/// Takes multiple collections of resources (e.g., from different JSON files or API responses),
/// groups all resources by their `resourceinstanceid`, and merges each group.
///
/// # Arguments
/// * `resource_batches` - Vector of resource collections to merge
/// * `recompute_descriptors` - If true, recomputes descriptors from tiles after merging
///   using the graph from the registry (looked up by graph_id from the resource)
///
/// # Returns
/// * `BatchMergeResult` - Contains merged resources (one per unique ID) and all warnings
///
/// # Example
/// ```ignore
/// // Process disjoint subgraphs from multiple files
/// let batch1: Vec<StaticResource> = parse_file("part1.json");
/// let batch2: Vec<StaticResource> = parse_file("part2.json");
/// let result = batch_merge_resources(vec![batch1, batch2], true);
/// // result.resources contains one entry per unique resourceinstanceid
/// ```
pub fn batch_merge_resources(
    resource_batches: Vec<Vec<StaticResource>>,
    recompute_descriptors: bool,
    strict: bool,
) -> BatchMergeResult {
    use crate::registry::get_graph;
    use crate::IndexedGraph;
    use std::collections::BTreeMap;

    // Group all resources by resourceinstanceid
    let mut grouped: BTreeMap<String, Vec<StaticResource>> = BTreeMap::new();

    for batch in resource_batches {
        for resource in batch {
            let id = resource.resourceinstance.resourceinstanceid.clone();
            grouped.entry(id).or_default().push(resource);
        }
    }

    let mut merged_resources = Vec::new();
    let mut all_warnings = Vec::new();

    // Cache IndexedGraphs by graph_id to avoid rebuilding for each resource
    let mut indexed_graphs: BTreeMap<String, IndexedGraph> = BTreeMap::new();

    // Merge each group
    for (resource_id, resources) in grouped {
        match merge_resources(resources) {
            Ok(result) => {
                // Prefix warnings with resource ID for clarity
                for warning in result.warnings {
                    all_warnings.push(format!("[{}] {}", resource_id, warning));
                }

                let mut resource = result.resource;
                let graph_id = resource.resourceinstance.graph_id.clone();

                // Get or create IndexedGraph for this graph_id (needed for both unification and descriptors)
                if !indexed_graphs.contains_key(&graph_id) {
                    if let Some(graph) = get_graph(&graph_id) {
                        indexed_graphs
                            .insert(graph_id.clone(), IndexedGraph::new((*graph).clone()));
                    }
                }

                // Unify cardinality-1 tiles if we have the graph
                if let Some(indexed) = indexed_graphs.get(&graph_id) {
                    if let Some(ref mut tiles) = resource.tiles {
                        match unify_cardinality_one_tiles(tiles, indexed, strict) {
                            Ok(unify_warnings) => {
                                for warning in unify_warnings {
                                    all_warnings.push(format!("[{}] {}", resource_id, warning));
                                }
                            }
                            Err(e) => {
                                all_warnings.push(format!("[{}] Unify error: {}", resource_id, e));
                                if strict {
                                    return BatchMergeResult {
                                        resources: merged_resources,
                                        warnings: all_warnings,
                                        error: Some(format!("[{}] {}", resource_id, e)),
                                    };
                                }
                            }
                        }
                    }
                }

                // Recompute descriptors if requested (graph already fetched above for unification)
                if recompute_descriptors {
                    if let Some(indexed) = indexed_graphs.get(&graph_id) {
                        // Compute descriptors from merged tiles with diagnostics
                        let tiles = resource.tiles.as_deref().unwrap_or(&[]);
                        let mut descriptor_warnings = Vec::new();
                        let descriptors = indexed
                            .build_descriptors_with_diagnostics(tiles, &mut descriptor_warnings);

                        // Add descriptor warnings with resource context
                        for warning in descriptor_warnings {
                            all_warnings.push(format!("[{}] Descriptor: {}", resource_id, warning));
                        }

                        // Update resource with computed descriptors
                        resource.resourceinstance.descriptors = descriptors.clone();

                        // Update name from descriptors if available
                        if let Some(ref name) = descriptors.name {
                            if !name.is_empty() {
                                resource.resourceinstance.name = name.clone();
                            }
                        }
                    } else {
                        all_warnings.push(format!(
                            "[{}] Graph not found in registry for descriptor computation: {}",
                            resource_id, graph_id
                        ));
                    }
                }

                merged_resources.push(resource);
            }
            Err(e) => {
                // This shouldn't happen since we grouped by ID, but handle gracefully
                all_warnings.push(format!("[{}] Merge error: {}", resource_id, e));
            }
        }
    }

    BatchMergeResult {
        resources: merged_resources,
        warnings: all_warnings,
        error: None,
    }
}

/// Type alias for tile data merge mapping: canonical_idx -> Vec<(source_tile_id, data)>
type TileDataMergeMap = HashMap<usize, Vec<(String, HashMap<String, serde_json::Value>)>>;

/// Unify tiles for cardinality-1 nodegroups and update parenttile_id references.
///
/// When merging resources from multiple sources, cardinality-1 nodegroups may end up
/// with multiple tiles (one from each source). This function:
/// 1. Identifies cardinality-1 nodegroups with multiple tiles
/// 2. Keeps the first tile as canonical, merges data from duplicates
/// 3. Updates parenttile_id references in child tiles to point to the canonical tile
/// 4. Warns if there are conflicting data values
///
/// # Arguments
/// * `tiles` - Mutable reference to the tiles vector
/// * `indexed_graph` - The indexed graph for looking up nodegroup cardinality
///
/// # Returns
/// * Vector of warning messages about unified tiles and data conflicts
pub fn unify_cardinality_one_tiles(
    tiles: &mut Vec<StaticTile>,
    indexed_graph: &crate::IndexedGraph,
    strict: bool,
) -> Result<Vec<String>, String> {
    use std::collections::BTreeMap;

    let mut warnings = Vec::new();

    // Group tile indices by (nodegroup_id, parenttile_id).
    // Cardinality-1 means one tile per parent context, not one tile total —
    // tiles under different parent tiles are separate instances and must not be unified.
    let mut tiles_by_context: BTreeMap<(String, Option<String>), Vec<usize>> = BTreeMap::new();
    for (idx, tile) in tiles.iter().enumerate() {
        tiles_by_context
            .entry((tile.nodegroup_id.clone(), tile.parenttile_id.clone()))
            .or_default()
            .push(idx);
    }

    // Build mapping of old_tile_id -> canonical_tile_id for cardinality-1 nodegroups
    let mut tile_redirect: HashMap<String, String> = HashMap::new();
    let mut tiles_to_remove: HashSet<usize> = HashSet::new();
    // Store data to merge: canonical_idx -> Vec<(source_tile_id, data)>
    let mut data_to_merge: TileDataMergeMap = HashMap::new();

    for ((nodegroup_id, _parent_tile_id), tile_indices) in &tiles_by_context {
        if tile_indices.len() <= 1 {
            continue; // No unification needed
        }

        // Check cardinality
        let nodegroup = match indexed_graph.graph.get_nodegroup_by_id(nodegroup_id) {
            Some(ng) => ng,
            None => continue,
        };

        let is_single = nodegroup
            .cardinality
            .as_ref()
            .map(|c| c != "n")
            .unwrap_or(true);

        if !is_single {
            continue; // cardinality-n, multiple tiles are allowed
        }

        // Cardinality-1 with multiple tiles under the same parent - need to unify
        let canonical_idx = tile_indices[0];
        let canonical_tile_id = tiles[canonical_idx].tileid.clone();

        for &idx in tile_indices.iter().skip(1) {
            let tile = &tiles[idx];
            let tile_id = tile
                .tileid
                .clone()
                .unwrap_or_else(|| format!("(index {})", idx));

            // Record tile redirect
            if let Some(ref old_tile_id) = tile.tileid {
                if let Some(ref canon_id) = canonical_tile_id {
                    tile_redirect.insert(old_tile_id.clone(), canon_id.clone());
                }
            }

            // Collect data to merge
            if !tile.data.is_empty() {
                data_to_merge
                    .entry(canonical_idx)
                    .or_default()
                    .push((tile_id, tile.data.clone()));
            }

            tiles_to_remove.insert(idx);
        }

        if tile_indices.len() > 1 {
            warnings.push(format!(
                "Unified cardinality-1 nodegroup '{}': kept 1 tile, removed {} duplicate(s)",
                nodegroup_id,
                tile_indices.len() - 1
            ));
        }
    }

    // Merge data into canonical tiles
    for (canonical_idx, sources) in data_to_merge {
        let canonical_tile = &mut tiles[canonical_idx];
        let canonical_tile_id = canonical_tile
            .tileid
            .clone()
            .unwrap_or_else(|| format!("(index {})", canonical_idx));

        for (source_tile_id, source_data) in sources {
            for (key, value) in source_data {
                if let Some(existing) = canonical_tile.data.get(&key) {
                    if existing != &value {
                        let msg = format!(
                            "Data conflict in nodegroup '{}': key '{}' has different values in tiles '{}' and '{}'",
                            canonical_tile.nodegroup_id,
                            key,
                            canonical_tile_id,
                            source_tile_id
                        );
                        if strict {
                            return Err(msg);
                        }
                        warnings.push(format!("{} (keeping first)", msg));
                    }
                    // Keep existing value (first wins)
                } else {
                    // New key, add it
                    canonical_tile.data.insert(key, value);
                }
            }
        }
    }

    // Update parenttile_id references
    for tile in tiles.iter_mut() {
        if let Some(ref old_parent_id) = tile.parenttile_id {
            if let Some(new_parent_id) = tile_redirect.get(old_parent_id) {
                tile.parenttile_id = Some(new_parent_id.clone());
            }
        }
    }

    // Remove duplicate tiles (in reverse order to preserve indices)
    let mut indices: Vec<usize> = tiles_to_remove.into_iter().collect();
    indices.sort_by(|a, b| b.cmp(a)); // Reverse order
    for idx in indices {
        tiles.remove(idx);
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_resource_serialization() {
        let resource = StaticResource {
            resourceinstance: StaticResourceMetadata {
                descriptors: StaticResourceDescriptors::default(),
                graph_id: "test-graph".to_string(),
                name: "Test".to_string(),
                resourceinstanceid: "test-id".to_string(),
                publication_id: None,
                principaluser_id: None,
                legacyid: None,
                graph_publication_id: None,
                createdtime: None,
                lastmodified: None,
            },
            tiles: Some(vec![]),
            metadata: HashMap::new(),
            cache: None,
            scopes: None,
            tiles_loaded: None,
        };

        let json = serde_json::to_string_pretty(&resource).unwrap();
        println!("StaticResource JSON:\n{}", json);

        // Check that resourceinstance is nested
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(
            value.get("resourceinstance").is_some(),
            "Should have nested resourceinstance"
        );
    }

    fn make_test_resource(resource_id: &str, tile_ids: &[&str]) -> StaticResource {
        let tiles: Vec<StaticTile> = tile_ids
            .iter()
            .map(|id| StaticTile {
                tileid: Some(id.to_string()),
                nodegroup_id: "ng1".to_string(),
                resourceinstance_id: resource_id.to_string(),
                parenttile_id: None,
                data: HashMap::new(),
                provisionaledits: None,
                sortorder: None,
            })
            .collect();

        StaticResource {
            resourceinstance: StaticResourceMetadata {
                descriptors: StaticResourceDescriptors::default(),
                graph_id: "test-graph".to_string(),
                name: "Test".to_string(),
                resourceinstanceid: resource_id.to_string(),
                publication_id: None,
                principaluser_id: None,
                legacyid: None,
                graph_publication_id: None,
                createdtime: None,
                lastmodified: None,
            },
            tiles: Some(tiles),
            metadata: HashMap::new(),
            cache: None,
            scopes: None,
            tiles_loaded: None,
        }
    }

    #[test]
    fn test_merge_resources_basic() {
        let r1 = make_test_resource("res-1", &["tile-a", "tile-b"]);
        let r2 = make_test_resource("res-1", &["tile-c", "tile-d"]);

        let result = merge_resources(vec![r1, r2]).unwrap();

        assert_eq!(result.resource.resourceinstance.resourceinstanceid, "res-1");
        let tiles = result.resource.tiles.unwrap();
        assert_eq!(tiles.len(), 4);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_merge_resources_duplicate_detection() {
        let r1 = make_test_resource("res-1", &["tile-a", "tile-b"]);
        let r2 = make_test_resource("res-1", &["tile-b", "tile-c"]); // tile-b is duplicate

        let result = merge_resources(vec![r1, r2]).unwrap();

        let tiles = result.resource.tiles.unwrap();
        assert_eq!(tiles.len(), 3); // tile-b counted once
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("tile-b"));
    }

    #[test]
    fn test_merge_resources_id_mismatch() {
        let r1 = make_test_resource("res-1", &["tile-a"]);
        let r2 = make_test_resource("res-2", &["tile-b"]); // Different ID

        let result = merge_resources(vec![r1, r2]);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mismatch"));
    }

    #[test]
    fn test_merge_resources_empty() {
        let result = merge_resources(vec![]);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No resources"));
    }

    #[test]
    fn test_merge_resources_preserves_cache() {
        // Create resources with cache entries
        let mut r1 = make_test_resource("res-1", &["tile-a"]);
        let mut r2 = make_test_resource("res-1", &["tile-b"]);

        // Set up cache for r1: tileId -> nodeId -> entry
        let mut cache1: ResourceCache = HashMap::new();
        let mut tile_a_entries: HashMap<String, CacheEntry> = HashMap::new();
        tile_a_entries.insert(
            "node-1".to_string(),
            CacheEntry::Single(RelatedResourceEntry {
                datatype: "resource-instance".to_string(),
                id: "related-1".to_string(),
                resource_type: "TestModel".to_string(),
                graph_id: "graph-a".to_string(),
                title: Some("Related 1".to_string()),
                meta: None,
            }),
        );
        tile_a_entries.insert(
            "node-2".to_string(),
            CacheEntry::Single(RelatedResourceEntry {
                datatype: "resource-instance".to_string(),
                id: "related-2".to_string(),
                resource_type: "TestModel".to_string(),
                graph_id: "graph-a".to_string(),
                title: Some("Related 2".to_string()),
                meta: None,
            }),
        );
        cache1.insert("tile-a".to_string(), tile_a_entries);
        r1.cache = serde_json::to_value(&cache1).ok();

        // Set up cache for r2 with overlapping tile/node and new entries
        let mut cache2: ResourceCache = HashMap::new();
        let mut tile_a_entries_2: HashMap<String, CacheEntry> = HashMap::new();
        tile_a_entries_2.insert(
            "node-2".to_string(),
            CacheEntry::Single(RelatedResourceEntry {
                datatype: "resource-instance".to_string(),
                id: "related-2".to_string(),
                resource_type: "TestModel".to_string(),
                graph_id: "graph-a".to_string(),
                title: Some("Related 2 - Different Name".to_string()), // Should be ignored (first wins)
                meta: None,
            }),
        );
        cache2.insert("tile-a".to_string(), tile_a_entries_2);

        let mut tile_b_entries: HashMap<String, CacheEntry> = HashMap::new();
        tile_b_entries.insert(
            "node-3".to_string(),
            CacheEntry::Single(RelatedResourceEntry {
                datatype: "resource-instance".to_string(),
                id: "related-3".to_string(),
                resource_type: "OtherModel".to_string(),
                graph_id: "graph-b".to_string(),
                title: Some("Related 3".to_string()),
                meta: None,
            }),
        );
        cache2.insert("tile-b".to_string(), tile_b_entries);
        r2.cache = serde_json::to_value(&cache2).ok();

        let result = merge_resources(vec![r1, r2]).unwrap();

        // Check that cache was preserved
        assert!(result.resource.cache.is_some(), "Cache should be present");

        let merged_cache: ResourceCache =
            serde_json::from_value(result.resource.cache.unwrap()).unwrap();

        // Should have 2 tiles (tile-a, tile-b)
        assert_eq!(merged_cache.len(), 2);
        assert!(merged_cache.contains_key("tile-a"));
        assert!(merged_cache.contains_key("tile-b"));

        // tile-a should have 2 entries (node-1, node-2)
        let tile_a = merged_cache.get("tile-a").unwrap();
        assert_eq!(tile_a.len(), 2);
        assert!(tile_a.contains_key("node-1"));
        assert!(tile_a.contains_key("node-2"));

        // node-2 in tile-a should have the first resource's version (first wins)
        if let CacheEntry::Single(entry) = tile_a.get("node-2").unwrap() {
            assert_eq!(entry.title.as_deref(), Some("Related 2"));
        } else {
            panic!("Expected CacheEntry::Single");
        }

        // tile-b should have 1 entry (node-3)
        let tile_b = merged_cache.get("tile-b").unwrap();
        assert_eq!(tile_b.len(), 1);
        assert!(tile_b.contains_key("node-3"));
    }
}
