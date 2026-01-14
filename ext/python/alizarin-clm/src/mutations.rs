//! CLM Mutation Handlers
//!
//! This module provides graph mutation handlers for CLM-specific operations.

use alizarin_core::graph_mutator::{
    ExtensionMutationHandler, MutationConformance, MutationError, MutatorOptions,
};
use alizarin_core::StaticGraph;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Mutation name for reference_change_collection
pub const REFERENCE_CHANGE_COLLECTION: &str = "clm.reference_change_collection";

/// Parameters for the reference_change_collection mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceChangeCollectionParams {
    /// Node ID or alias of the reference node to update
    pub node_id: String,
    /// New collection ID to set
    pub collection_id: String,
    /// Config key to use (defaults to "controlledList")
    #[serde(default = "default_config_key")]
    pub config_key: String,
}

fn default_config_key() -> String {
    "controlledList".to_string()
}

/// Handler for changing a reference node's controlled list collection
///
/// This mutation updates the config of a reference node to point to a new
/// controlled list collection. It can be used to migrate nodes between
/// different CLM lists.
///
/// ## Parameters
///
/// - `node_id`: The node ID or alias of the reference node
/// - `collection_id`: The UUID of the new controlled list
/// - `config_key`: The config key to update (default: "controlledList")
///
/// ## Example
///
/// ```ignore
/// use alizarin_core::graph_mutator::{GraphMutation, ExtensionMutationParams, MutationConformance};
/// use serde_json::json;
///
/// let mutation = GraphMutation::Extension(ExtensionMutationParams {
///     name: "clm.reference_change_collection".to_string(),
///     params: json!({
///         "node_id": "my_reference_node",
///         "collection_id": "new-collection-uuid"
///     }),
///     conformance: MutationConformance::AlwaysConformant,
/// });
/// ```
pub struct ReferenceChangeCollectionHandler;

impl ExtensionMutationHandler for ReferenceChangeCollectionHandler {
    fn apply(
        &self,
        graph: &mut StaticGraph,
        params: &Value,
        _options: &MutatorOptions,
    ) -> Result<(), MutationError> {
        // Parse parameters
        let params: ReferenceChangeCollectionParams = serde_json::from_value(params.clone())
            .map_err(|e| MutationError::Other(format!("Invalid params: {}", e)))?;

        // Find the node by ID or alias
        let node_id = {
            // Try to find by alias first
            if let Some(node) = graph.find_node_by_alias(&params.node_id) {
                node.nodeid.clone()
            } else if graph.nodes.iter().any(|n| n.nodeid == params.node_id) {
                params.node_id.clone()
            } else {
                return Err(MutationError::NodeNotFound(params.node_id));
            }
        };

        // Find and update the node's config
        let node = graph.nodes.iter_mut()
            .find(|n| n.nodeid == node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

        // Verify it's a reference type node
        if node.datatype != "reference" && node.datatype != "reference-list" {
            return Err(MutationError::InvalidDatatype {
                expected: "reference or reference-list".to_string(),
                found: node.datatype.clone(),
                node_id,
            });
        }

        // Update the config with the new collection ID
        node.config.insert(
            params.config_key,
            Value::String(params.collection_id),
        );

        Ok(())
    }

    fn conformance(&self) -> MutationConformance {
        // Collection changes are valid for both branches and models
        MutationConformance::AlwaysConformant
    }

    fn description(&self) -> &str {
        "Change the controlled list collection for a reference node"
    }
}

/// Create a pre-configured registry with CLM mutation handlers
///
/// ## Example
///
/// ```ignore
/// use alizarin_clm::mutations::create_clm_registry;
/// use alizarin_core::graph_mutator::apply_mutations_with_extensions;
///
/// let registry = create_clm_registry();
/// let result = apply_mutations_with_extensions(
///     &graph,
///     mutations,
///     options,
///     Some(&registry),
/// )?;
/// ```
pub fn create_clm_registry() -> alizarin_core::graph_mutator::ExtensionMutationRegistry {
    let mut registry = alizarin_core::graph_mutator::ExtensionMutationRegistry::new();
    registry.register(
        REFERENCE_CHANGE_COLLECTION,
        Arc::new(ReferenceChangeCollectionHandler),
    );
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use alizarin_core::graph_mutator::{
        apply_mutations_with_extensions, ExtensionMutationParams, GraphMutation,
    };
    use serde_json::json;

    fn create_test_graph_with_reference() -> StaticGraph {
        let graph_json = r#"{
            "graphid": "test-graph-id",
            "name": {"en": "Test Graph"},
            "isresource": true,
            "is_editable": true,
            "nodes": [
                {
                    "nodeid": "root-node-id",
                    "name": "Root",
                    "alias": "root",
                    "datatype": "semantic",
                    "nodegroup_id": null,
                    "graph_id": "test-graph-id",
                    "is_collector": false,
                    "isrequired": false,
                    "exportable": false,
                    "ontologyclass": "E1_CRM_Entity",
                    "hascustomalias": false,
                    "issearchable": false,
                    "istopnode": true
                },
                {
                    "nodeid": "ref-node-id",
                    "name": "Type",
                    "alias": "type_ref",
                    "datatype": "reference",
                    "nodegroup_id": "ref-nodegroup-id",
                    "graph_id": "test-graph-id",
                    "is_collector": false,
                    "isrequired": false,
                    "exportable": true,
                    "ontologyclass": "E55_Type",
                    "hascustomalias": true,
                    "issearchable": true,
                    "istopnode": false,
                    "config": {
                        "controlledList": "old-collection-uuid"
                    }
                }
            ],
            "nodegroups": [
                {
                    "nodegroupid": "ref-nodegroup-id",
                    "cardinality": "1"
                }
            ],
            "edges": [
                {
                    "domainnode_id": "root-node-id",
                    "rangenode_id": "ref-node-id",
                    "edgeid": "edge-1",
                    "graph_id": "test-graph-id"
                }
            ],
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "root": {
                "nodeid": "root-node-id",
                "name": "Root",
                "alias": "root",
                "datatype": "semantic",
                "nodegroup_id": null,
                "graph_id": "test-graph-id",
                "is_collector": false,
                "isrequired": false,
                "exportable": false,
                "ontologyclass": "E1_CRM_Entity",
                "hascustomalias": false,
                "issearchable": false,
                "istopnode": true
            }
        }"#;

        let mut graph: StaticGraph = serde_json::from_str(graph_json)
            .expect("Failed to parse test graph JSON");
        graph.build_indices();
        graph
    }

    #[test]
    fn test_reference_change_collection_by_alias() {
        let graph = create_test_graph_with_reference();
        let registry = create_clm_registry();
        let options = MutatorOptions::default();

        // Create mutation using alias
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: REFERENCE_CHANGE_COLLECTION.to_string(),
            params: json!({
                "node_id": "type_ref",
                "collection_id": "new-collection-uuid"
            }),
            conformance: MutationConformance::AlwaysConformant,
        });

        let result = apply_mutations_with_extensions(
            &graph,
            vec![mutation],
            options,
            Some(&registry),
        );

        assert!(result.is_ok());
        let mutated = result.unwrap();

        // Find the reference node and check its config
        let ref_node = mutated.nodes.iter()
            .find(|n| n.alias.as_deref() == Some("type_ref"))
            .unwrap();

        assert_eq!(
            ref_node.config.get("controlledList").and_then(|v| v.as_str()),
            Some("new-collection-uuid")
        );
    }

    #[test]
    fn test_reference_change_collection_by_id() {
        let graph = create_test_graph_with_reference();
        let registry = create_clm_registry();
        let options = MutatorOptions::default();

        // Create mutation using node ID
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: REFERENCE_CHANGE_COLLECTION.to_string(),
            params: json!({
                "node_id": "ref-node-id",
                "collection_id": "another-collection-uuid"
            }),
            conformance: MutationConformance::AlwaysConformant,
        });

        let result = apply_mutations_with_extensions(
            &graph,
            vec![mutation],
            options,
            Some(&registry),
        );

        assert!(result.is_ok());
        let mutated = result.unwrap();

        let ref_node = mutated.nodes.iter()
            .find(|n| n.nodeid == "ref-node-id")
            .unwrap();

        assert_eq!(
            ref_node.config.get("controlledList").and_then(|v| v.as_str()),
            Some("another-collection-uuid")
        );
    }

    #[test]
    fn test_reference_change_collection_custom_key() {
        let graph = create_test_graph_with_reference();
        let registry = create_clm_registry();
        let options = MutatorOptions::default();

        // Use custom config key
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: REFERENCE_CHANGE_COLLECTION.to_string(),
            params: json!({
                "node_id": "type_ref",
                "collection_id": "custom-collection",
                "config_key": "rdmCollection"
            }),
            conformance: MutationConformance::AlwaysConformant,
        });

        let result = apply_mutations_with_extensions(
            &graph,
            vec![mutation],
            options,
            Some(&registry),
        );

        assert!(result.is_ok());
        let mutated = result.unwrap();

        let ref_node = mutated.nodes.iter()
            .find(|n| n.alias.as_deref() == Some("type_ref"))
            .unwrap();

        assert_eq!(
            ref_node.config.get("rdmCollection").and_then(|v| v.as_str()),
            Some("custom-collection")
        );
    }

    #[test]
    fn test_reference_change_collection_node_not_found() {
        let graph = create_test_graph_with_reference();
        let registry = create_clm_registry();
        let options = MutatorOptions::default();

        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: REFERENCE_CHANGE_COLLECTION.to_string(),
            params: json!({
                "node_id": "nonexistent_node",
                "collection_id": "some-collection"
            }),
            conformance: MutationConformance::AlwaysConformant,
        });

        let result = apply_mutations_with_extensions(
            &graph,
            vec![mutation],
            options,
            Some(&registry),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_reference_change_collection_wrong_datatype() {
        let graph = create_test_graph_with_reference();
        let registry = create_clm_registry();
        let options = MutatorOptions::default();

        // Try to change collection on root node (semantic type)
        let mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: REFERENCE_CHANGE_COLLECTION.to_string(),
            params: json!({
                "node_id": "root",
                "collection_id": "some-collection"
            }),
            conformance: MutationConformance::AlwaysConformant,
        });

        let result = apply_mutations_with_extensions(
            &graph,
            vec![mutation],
            options,
            Some(&registry),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("reference"));
    }

    /// Integration test: Create graph, add reference node, change collection,
    /// then verify the collection is used when processing reference data
    #[test]
    fn test_integration_change_collection_and_verify() {
        use alizarin_core::graph_mutator::{
            AddNodeParams, Cardinality, NodeOptions,
        };

        // Start with a minimal graph
        let base_graph_json = r#"{
            "graphid": "integration-test-graph",
            "name": {"en": "Integration Test"},
            "isresource": true,
            "is_editable": true,
            "nodes": [{
                "nodeid": "root-node-id",
                "name": "Person",
                "alias": "person",
                "datatype": "semantic",
                "nodegroup_id": null,
                "graph_id": "integration-test-graph",
                "is_collector": false,
                "isrequired": false,
                "exportable": false,
                "ontologyclass": "E21_Person",
                "hascustomalias": false,
                "issearchable": false,
                "istopnode": true
            }],
            "nodegroups": [],
            "edges": [],
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "root": {
                "nodeid": "root-node-id",
                "name": "Person",
                "alias": "person",
                "datatype": "semantic",
                "nodegroup_id": null,
                "graph_id": "integration-test-graph",
                "is_collector": false,
                "isrequired": false,
                "exportable": false,
                "ontologyclass": "E21_Person",
                "hascustomalias": false,
                "issearchable": false,
                "istopnode": true
            }
        }"#;

        let mut graph: StaticGraph = serde_json::from_str(base_graph_json).unwrap();
        graph.build_indices();

        // Step 1: Add a reference node using the mutation system
        let options = MutatorOptions::default();
        let add_node_mutation = GraphMutation::AddNode(AddNodeParams {
            parent_alias: Some("person".to_string()),
            alias: "person_type".to_string(),
            name: "Person Type".to_string(),
            cardinality: Cardinality::One,
            datatype: "reference".to_string(),
            ontology_class: "E55_Type".to_string(),
            parent_property: "P2_has_type".to_string(),
            description: Some("Type of person".to_string()),
            config: Some(json!({
                "controlledList": "initial-collection-id"
            })),
            options: NodeOptions::default(),
        });

        let graph_with_ref = apply_mutations_with_extensions(
            &graph,
            vec![add_node_mutation],
            options.clone(),
            None, // No extension registry needed for AddNode
        ).unwrap();

        // Verify the reference node was added with initial collection
        let ref_node = graph_with_ref.nodes.iter()
            .find(|n| n.alias.as_deref() == Some("person_type"))
            .expect("Reference node should exist");
        assert_eq!(ref_node.datatype, "reference");
        assert_eq!(
            ref_node.config.get("controlledList").and_then(|v| v.as_str()),
            Some("initial-collection-id")
        );

        // Step 2: Change the collection using our CLM mutation
        let registry = create_clm_registry();
        let new_collection_id = "new-person-type-collection";

        let change_mutation = GraphMutation::Extension(ExtensionMutationParams {
            name: REFERENCE_CHANGE_COLLECTION.to_string(),
            params: json!({
                "node_id": "person_type",
                "collection_id": new_collection_id
            }),
            conformance: MutationConformance::AlwaysConformant,
        });

        let updated_graph = apply_mutations_with_extensions(
            &graph_with_ref,
            vec![change_mutation],
            options,
            Some(&registry),
        ).unwrap();

        // Step 3: Verify the collection was updated
        let updated_node = updated_graph.nodes.iter()
            .find(|n| n.alias.as_deref() == Some("person_type"))
            .expect("Reference node should still exist");

        assert_eq!(
            updated_node.config.get("controlledList").and_then(|v| v.as_str()),
            Some(new_collection_id),
            "Collection should be updated to the new value"
        );

        // Step 4: Verify the graph can be serialized (for use with tree_to_tiles)
        let graph_json = serde_json::to_string(&updated_graph).unwrap();
        assert!(graph_json.contains(new_collection_id));
    }

    /// Test that verifies display text rendering for reference values
    #[test]
    fn test_reference_display_text_integration() {
        use crate::{StaticReference, StaticReferenceLabel};

        // Create a reference value that would come from a controlled list
        let reference = StaticReference {
            labels: vec![
                StaticReferenceLabel {
                    id: "label-1".to_string(),
                    language_id: "en".to_string(),
                    list_item_id: "item-type-1".to_string(),
                    value: "Historian".to_string(),
                    valuetype_id: "prefLabel".to_string(),
                },
                StaticReferenceLabel {
                    id: "label-2".to_string(),
                    language_id: "es".to_string(),
                    list_item_id: "item-type-1".to_string(),
                    value: "Historiador".to_string(),
                    valuetype_id: "prefLabel".to_string(),
                },
            ],
            list_id: "person-types-collection".to_string(),
            uri: "http://example.com/vocab/historian".to_string(),
        };

        // Test display string rendering in different languages
        assert_eq!(reference.to_display_string(Some("en")), "Historian");
        assert_eq!(reference.to_display_string(Some("es")), "Historiador");
        assert_eq!(reference.to_display_string(None), "Historian"); // default

        // Verify the reference can be serialized to tile data format
        let tile_data = serde_json::to_value(&reference).unwrap();
        assert!(tile_data.get("labels").is_some());
        assert_eq!(tile_data["list_id"], "person-types-collection");
    }
}
