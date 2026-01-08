//! Resource types for resource instances and metadata.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use super::descriptors::StaticResourceDescriptors;
use super::tile::StaticTile;

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
    let first_metadata = resources[0].metadata.clone();
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

    for resource in resources {
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
    }

    Ok(MergeResult {
        resource: StaticResource {
            resourceinstance: first_instance,
            tiles: Some(merged_tiles),
            metadata: first_metadata,
            cache: None,
            scopes: None,
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
///
/// # Returns
/// * `BatchMergeResult` - Contains merged resources (one per unique ID) and all warnings
///
/// # Example
/// ```ignore
/// // Process disjoint subgraphs from multiple files
/// let batch1: Vec<StaticResource> = parse_file("part1.json");
/// let batch2: Vec<StaticResource> = parse_file("part2.json");
/// let result = batch_merge_resources(vec![batch1, batch2]);
/// // result.resources contains one entry per unique resourceinstanceid
/// ```
pub fn batch_merge_resources(resource_batches: Vec<Vec<StaticResource>>) -> BatchMergeResult {
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

    // Merge each group
    for (resource_id, resources) in grouped {
        match merge_resources(resources) {
            Ok(result) => {
                // Prefix warnings with resource ID for clarity
                for warning in result.warnings {
                    all_warnings.push(format!("[{}] {}", resource_id, warning));
                }
                merged_resources.push(result.resource);
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
    }
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
        assert!(value.get("resourceinstance").is_some(), "Should have nested resourceinstance");
    }

    fn make_test_resource(resource_id: &str, tile_ids: &[&str]) -> StaticResource {
        let tiles: Vec<StaticTile> = tile_ids.iter().map(|id| {
            StaticTile {
                tileid: Some(id.to_string()),
                nodegroup_id: "ng1".to_string(),
                resourceinstance_id: resource_id.to_string(),
                parenttile_id: None,
                data: HashMap::new(),
                provisionaledits: None,
                sortorder: None,
            }
        }).collect();

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
        let r2 = make_test_resource("res-1", &["tile-b", "tile-c"]);  // tile-b is duplicate

        let result = merge_resources(vec![r1, r2]).unwrap();

        let tiles = result.resource.tiles.unwrap();
        assert_eq!(tiles.len(), 3);  // tile-b counted once
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("tile-b"));
    }

    #[test]
    fn test_merge_resources_id_mismatch() {
        let r1 = make_test_resource("res-1", &["tile-a"]);
        let r2 = make_test_resource("res-2", &["tile-b"]);  // Different ID

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
}
