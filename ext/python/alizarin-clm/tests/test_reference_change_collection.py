"""
Integration tests for the clm.reference_change_collection mutation.

These tests demonstrate:
1. Creating a graph with a reference node
2. Changing the collection using the extension mutation
3. Verifying display text rendering with the updated collection
"""

import pytest
import json
from pathlib import Path

# Test data directory
TEST_DATA_DIR = Path(__file__).parent / "data"


def reference_change_collection_handler(graph_json: str, params_json: str) -> str:
    """
    Python implementation of the reference_change_collection mutation.

    This handler modifies a reference node's controlledList config.

    Args:
        graph_json: The graph as JSON string
        params_json: Mutation parameters as JSON string
            - node_id: Node ID or alias to modify
            - collection_id: New collection ID
            - config_key: Config key to update (default: "controlledList")

    Returns:
        Modified graph as JSON string
    """
    graph = json.loads(graph_json)
    params = json.loads(params_json)

    node_id = params["node_id"]
    collection_id = params["collection_id"]
    config_key = params.get("config_key", "controlledList")

    # Find node by ID or alias
    target_node = None
    for node in graph["nodes"]:
        if node.get("nodeid") == node_id or node.get("alias") == node_id:
            target_node = node
            break

    if target_node is None:
        raise ValueError(f"Node not found: {node_id}")

    # Validate it's a reference type
    if target_node["datatype"] not in ("reference", "reference-list"):
        raise ValueError(
            f"Node {node_id} is not a reference type: {target_node['datatype']}"
        )

    # Update config
    if "config" not in target_node or target_node["config"] is None:
        target_node["config"] = {}
    target_node["config"][config_key] = collection_id

    return json.dumps(graph)


class TestReferenceChangeCollectionMutation:
    """Test the clm.reference_change_collection mutation."""

    @pytest.fixture
    def minimal_graph_with_reference(self):
        """Create a minimal graph with a reference node."""
        return {
            "graphid": "test-graph-id",
            "name": {"en": "Test Graph"},
            "isresource": True,
            "is_editable": True,
            "nodes": [
                {
                    "nodeid": "root-node-id",
                    "name": "Root",
                    "alias": "root",
                    "datatype": "semantic",
                    "nodegroup_id": None,
                    "graph_id": "test-graph-id",
                    "is_collector": False,
                    "isrequired": False,
                    "exportable": False,
                    "ontologyclass": "E1_CRM_Entity",
                    "hascustomalias": False,
                    "issearchable": False,
                    "istopnode": True,
                    "config": {},
                },
                {
                    "nodeid": "ref-node-id",
                    "name": "Type",
                    "alias": "type_ref",
                    "datatype": "reference",
                    "nodegroup_id": "ref-nodegroup-id",
                    "graph_id": "test-graph-id",
                    "is_collector": False,
                    "isrequired": False,
                    "exportable": True,
                    "ontologyclass": "E55_Type",
                    "hascustomalias": True,
                    "issearchable": True,
                    "istopnode": False,
                    "config": {
                        "controlledList": "old-collection-uuid"
                    },
                },
            ],
            "nodegroups": [
                {
                    "nodegroupid": "ref-nodegroup-id",
                    "cardinality": "1",
                }
            ],
            "edges": [
                {
                    "domainnode_id": "root-node-id",
                    "rangenode_id": "ref-node-id",
                    "edgeid": "edge-1",
                    "graph_id": "test-graph-id",
                }
            ],
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "root": {
                "nodeid": "root-node-id",
                "name": "Root",
                "alias": "root",
                "datatype": "semantic",
                "nodegroup_id": None,
                "graph_id": "test-graph-id",
                "is_collector": False,
                "isrequired": False,
                "exportable": False,
                "ontologyclass": "E1_CRM_Entity",
                "hascustomalias": False,
                "issearchable": False,
                "istopnode": True,
                "config": {},
            },
        }

    def test_change_collection_by_alias(self, minimal_graph_with_reference):
        """Test changing collection using node alias."""
        graph = minimal_graph_with_reference
        graph_json = json.dumps(graph)

        params = {
            "node_id": "type_ref",
            "collection_id": "new-collection-uuid",
        }
        params_json = json.dumps(params)

        # Apply mutation
        result_json = reference_change_collection_handler(graph_json, params_json)
        result = json.loads(result_json)

        # Find the reference node
        ref_node = next(n for n in result["nodes"] if n.get("alias") == "type_ref")

        assert ref_node["config"]["controlledList"] == "new-collection-uuid"

    def test_change_collection_by_id(self, minimal_graph_with_reference):
        """Test changing collection using node ID."""
        graph = minimal_graph_with_reference
        graph_json = json.dumps(graph)

        params = {
            "node_id": "ref-node-id",
            "collection_id": "another-collection-uuid",
        }
        params_json = json.dumps(params)

        # Apply mutation
        result_json = reference_change_collection_handler(graph_json, params_json)
        result = json.loads(result_json)

        # Find the reference node
        ref_node = next(n for n in result["nodes"] if n["nodeid"] == "ref-node-id")

        assert ref_node["config"]["controlledList"] == "another-collection-uuid"

    def test_change_collection_custom_key(self, minimal_graph_with_reference):
        """Test changing collection with custom config key."""
        graph = minimal_graph_with_reference
        graph_json = json.dumps(graph)

        params = {
            "node_id": "type_ref",
            "collection_id": "custom-collection",
            "config_key": "rdmCollection",
        }
        params_json = json.dumps(params)

        # Apply mutation
        result_json = reference_change_collection_handler(graph_json, params_json)
        result = json.loads(result_json)

        # Find the reference node
        ref_node = next(n for n in result["nodes"] if n.get("alias") == "type_ref")

        # Original key should still be there
        assert ref_node["config"]["controlledList"] == "old-collection-uuid"
        # New key should be added
        assert ref_node["config"]["rdmCollection"] == "custom-collection"

    def test_change_collection_node_not_found(self, minimal_graph_with_reference):
        """Test error when node not found."""
        graph = minimal_graph_with_reference
        graph_json = json.dumps(graph)

        params = {
            "node_id": "nonexistent_node",
            "collection_id": "some-collection",
        }
        params_json = json.dumps(params)

        with pytest.raises(ValueError, match="Node not found"):
            reference_change_collection_handler(graph_json, params_json)

    def test_change_collection_wrong_datatype(self, minimal_graph_with_reference):
        """Test error when node is not reference type."""
        graph = minimal_graph_with_reference
        graph_json = json.dumps(graph)

        params = {
            "node_id": "root",  # semantic node, not reference
            "collection_id": "some-collection",
        }
        params_json = json.dumps(params)

        with pytest.raises(ValueError, match="not a reference type"):
            reference_change_collection_handler(graph_json, params_json)


class TestIntegrationWithAlizarin:
    """
    Integration tests using the actual alizarin module.

    These tests demonstrate the full workflow:
    1. Register the CLM mutation handler
    2. Create a graph with a reference node
    3. Apply the mutation via alizarin
    4. Verify the result
    """

    @pytest.fixture
    def graph_with_reference(self):
        """Create a graph with a reference node for testing."""
        return {
            "graphid": "integration-test-graph",
            "name": {"en": "Integration Test"},
            "isresource": True,
            "is_editable": True,
            "nodes": [
                {
                    "nodeid": "root-node-id",
                    "name": "Person",
                    "alias": "person",
                    "datatype": "semantic",
                    "nodegroup_id": None,
                    "graph_id": "integration-test-graph",
                    "is_collector": False,
                    "isrequired": False,
                    "exportable": False,
                    "ontologyclass": "E21_Person",
                    "hascustomalias": False,
                    "issearchable": False,
                    "istopnode": True,
                    "config": {},
                },
                {
                    "nodeid": "person-type-node-id",
                    "name": "Person Type",
                    "alias": "person_type",
                    "datatype": "reference",
                    "nodegroup_id": "person-type-nodegroup-id",
                    "graph_id": "integration-test-graph",
                    "is_collector": False,
                    "isrequired": False,
                    "exportable": True,
                    "ontologyclass": "E55_Type",
                    "hascustomalias": True,
                    "issearchable": True,
                    "istopnode": False,
                    "config": {
                        "controlledList": "initial-collection-id"
                    },
                },
            ],
            "nodegroups": [
                {
                    "nodegroupid": "person-type-nodegroup-id",
                    "cardinality": "1",
                }
            ],
            "edges": [
                {
                    "domainnode_id": "root-node-id",
                    "rangenode_id": "person-type-node-id",
                    "edgeid": "edge-1",
                    "graph_id": "integration-test-graph",
                }
            ],
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "root": {
                "nodeid": "root-node-id",
                "name": "Person",
                "alias": "person",
                "datatype": "semantic",
                "nodegroup_id": None,
                "graph_id": "integration-test-graph",
                "is_collector": False,
                "isrequired": False,
                "exportable": False,
                "ontologyclass": "E21_Person",
                "hascustomalias": False,
                "issearchable": False,
                "istopnode": True,
                "config": {},
            },
        }

    def test_register_and_apply_clm_mutation(self, graph_with_reference):
        """
        Integration test: Register CLM mutation handler and apply it.

        This test demonstrates the full Python → Rust → Python flow:
        1. Register a Python handler for clm.reference_change_collection
        2. Apply the mutation via alizarin's extension-aware mutation API
        3. Verify the collection was updated
        """
        import alizarin

        # Check if extension mutation functions are available
        if not hasattr(alizarin, "register_extension_mutation"):
            pytest.skip("Extension mutation API not available")

        # Step 1: Register the CLM mutation handler
        alizarin.register_extension_mutation(
            "clm.reference_change_collection",
            reference_change_collection_handler,
            "AlwaysConformant",
        )

        # Verify registration
        assert alizarin.has_extension_mutation("clm.reference_change_collection")
        assert "clm.reference_change_collection" in alizarin.list_extension_mutations()

        # Step 2: Create the mutation request
        mutation_request = {
            "mutations": [
                {
                    "Extension": {
                        "name": "clm.reference_change_collection",
                        "params": {
                            "node_id": "person_type",
                            "collection_id": "new-person-type-collection"
                        },
                        "conformance": "AlwaysConformant"
                    }
                }
            ],
            "options": {}
        }

        graph_json = json.dumps(graph_with_reference)
        mutations_json = json.dumps(mutation_request)

        # Step 3: Apply the mutation
        result_json = alizarin.apply_mutations_with_extensions(
            graph_json,
            mutations_json,
        )

        result = json.loads(result_json)

        # Step 4: Verify the collection was updated
        ref_node = next(
            n for n in result["nodes"]
            if n.get("alias") == "person_type"
        )

        assert ref_node["config"]["controlledList"] == "new-person-type-collection"

    def test_integration_add_node_then_change_collection(self, graph_with_reference):
        """
        Integration test: Use multiple mutations in sequence.

        1. Add a new reference node using AddNode mutation
        2. Change its collection using the CLM extension mutation
        """
        import alizarin

        if not hasattr(alizarin, "register_extension_mutation"):
            pytest.skip("Extension mutation API not available")

        # Ensure handler is registered
        if not alizarin.has_extension_mutation("clm.reference_change_collection"):
            alizarin.register_extension_mutation(
                "clm.reference_change_collection",
                reference_change_collection_handler,
                "AlwaysConformant",
            )

        # Step 1: Add a new reference node using standard AddNode mutation
        add_node_mutation = {
            "AddNode": {
                "parent_alias": "person",
                "alias": "secondary_type",
                "name": "Secondary Type",
                "cardinality": "N",
                "datatype": "reference",
                "ontology_class": "E55_Type",
                "parent_property": "P2_has_type",
                "config": {
                    "controlledList": "initial-secondary-collection"
                },
                "options": {}
            }
        }

        mutation_request = {
            "mutations": [add_node_mutation],
            "options": {"autocreate_card": False, "autocreate_widget": False}
        }

        graph_json = json.dumps(graph_with_reference)
        mutations_json = json.dumps(mutation_request)

        # Apply AddNode (doesn't need extension registry)
        intermediate_json = alizarin.apply_mutations_from_json(
            graph_json,
            mutations_json,
        )

        intermediate = json.loads(intermediate_json)

        # Verify node was added
        secondary_node = next(
            (n for n in intermediate["nodes"] if n.get("alias") == "secondary_type"),
            None
        )
        assert secondary_node is not None
        assert secondary_node["config"]["controlledList"] == "initial-secondary-collection"

        # Step 2: Change the collection using CLM mutation
        change_mutation = {
            "mutations": [
                {
                    "Extension": {
                        "name": "clm.reference_change_collection",
                        "params": {
                            "node_id": "secondary_type",
                            "collection_id": "updated-secondary-collection"
                        },
                        "conformance": "AlwaysConformant"
                    }
                }
            ],
            "options": {}
        }

        result_json = alizarin.apply_mutations_with_extensions(
            intermediate_json,
            json.dumps(change_mutation),
        )

        result = json.loads(result_json)

        # Verify the collection was changed
        updated_node = next(
            n for n in result["nodes"]
            if n.get("alias") == "secondary_type"
        )

        assert updated_node["config"]["controlledList"] == "updated-secondary-collection"


class TestDisplayTextWithUpdatedCollection:
    """
    Tests that verify display text rendering works correctly
    after the collection has been updated.
    """

    def test_display_text_for_reference_in_collection(self):
        """
        Test that reference values from the updated collection
        render correctly to display text.
        """
        from alizarin_clm import StaticReference, StaticReferenceLabel

        # Create a reference value that belongs to the new collection
        reference = StaticReference(
            list_id="new-person-type-collection",  # The updated collection
            uri="http://example.com/vocab/historian",
            labels=[
                StaticReferenceLabel(
                    id="label-1",
                    language_id="en",
                    list_item_id="historian-item",
                    value="Historian",
                    valuetype_id="prefLabel",
                ),
                StaticReferenceLabel(
                    id="label-2",
                    language_id="es",
                    list_item_id="historian-item",
                    value="Historiador",
                    valuetype_id="prefLabel",
                ),
            ],
        )

        # Verify display text rendering
        assert reference.to_display_string("en") == "Historian"
        assert reference.to_display_string("es") == "Historiador"
        assert reference.to_display_string() == "Historian"  # default

    def test_coerce_reference_value_after_collection_change(self):
        """
        Test that reference values are properly coerced after
        the node's collection has been changed.

        This simulates the workflow:
        1. Collection was changed from A to B
        2. User provides a value from collection B
        3. Coercion succeeds because node now points to B
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.coerce_with_extension is None:
            pytest.skip("coerce_with_extension not available")

        # Reference value from the "new" collection
        value_json = json.dumps({
            "labels": [
                {
                    "id": "label-1",
                    "language_id": "en",
                    "list_item_id": "item-from-new-collection",
                    "value": "New Collection Item",
                    "valuetype_id": "prefLabel",
                }
            ],
            "list_id": "new-person-type-collection",
            "uri": "http://example.com/new-collection/item-1",
        })

        # Node config now points to new collection
        config_json = json.dumps({
            "controlledList": "new-person-type-collection",
            "multiValue": False,
        })

        # Coerce the value
        tile_data_json, resolved_json = alizarin.coerce_with_extension(
            "reference",
            value_json,
            config_json,
        )

        resolved = json.loads(resolved_json)

        # Verify coercion worked
        assert resolved["list_id"] == "new-person-type-collection"
        assert resolved["labels"][0]["value"] == "New Collection Item"


class TestEndToEndWorkflow:
    """
    End-to-end tests demonstrating the complete workflow:
    1. Load a graph
    2. Add/modify nodes
    3. Change collections
    4. Load business data
    5. Render display text
    """

    @pytest.fixture
    def person_graph(self):
        """Load Person graph with reference node."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        if not graph_path.exists():
            pytest.skip(f"Test data not found: {graph_path}")
        with open(graph_path) as f:
            data = json.load(f)
        return data["graph"][0]

    def test_full_workflow_json_tree_to_tiles(self, person_graph):
        """
        End-to-end test: Change collection, then convert JSON tree to tiles.

        This demonstrates the real-world workflow where:
        1. A graph's reference node collection is updated
        2. Business data with references from the new collection is loaded
        3. The tiles are created successfully
        """
        import alizarin
        import alizarin_clm  # noqa: F401

        if alizarin.json_tree_to_tiles is None:
            pytest.skip("json_tree_to_tiles not available")

        if not hasattr(alizarin, "register_extension_mutation"):
            pytest.skip("Extension mutation API not available")

        # Ensure handler is registered
        if not alizarin.has_extension_mutation("clm.reference_change_collection"):
            alizarin.register_extension_mutation(
                "clm.reference_change_collection",
                reference_change_collection_handler,
                "AlwaysConformant",
            )

        # Step 1: Change the collection for the 'test' reference node
        new_collection_id = "updated-test-collection"

        change_mutation = {
            "mutations": [
                {
                    "Extension": {
                        "name": "clm.reference_change_collection",
                        "params": {
                            "node_id": "test",
                            "collection_id": new_collection_id
                        },
                        "conformance": "AlwaysConformant"
                    }
                }
            ],
            "options": {}
        }

        graph_json = json.dumps(person_graph)

        updated_graph_json = alizarin.apply_mutations_with_extensions(
            graph_json,
            json.dumps(change_mutation),
        )

        updated_graph = json.loads(updated_graph_json)

        # Verify collection was updated
        test_node = next(
            n for n in updated_graph["nodes"]
            if n.get("alias") == "test"
        )
        assert test_node["config"]["controlledList"] == new_collection_id

        # Step 2: Create a JSON tree with a reference from the new collection
        person_tree = {
            "test": [
                {
                    "_value": {
                        "labels": [
                            {
                                "id": "new-label-1",
                                "language_id": "en",
                                "list_item_id": "new-item-1",
                                "value": "Item From Updated Collection",
                                "valuetype_id": "prefLabel",
                            }
                        ],
                        "list_id": new_collection_id,
                        "uri": "http://example.com/updated-collection/item-1",
                    }
                }
            ]
        }

        tree_json = json.dumps(person_tree)
        resource_id = "test-person-001"

        # Step 3: Register graph and convert to tiles
        graph_id = alizarin.register_graph(updated_graph_json)
        result = alizarin.json_tree_to_tiles(
            tree_json=tree_json,
            resource_id=resource_id,
            graph_id=graph_id,
        )

        # Step 4: Verify the result
        assert "business_data" in result
        resources = result["business_data"]["resources"]
        assert len(resources) == 1

        resource = resources[0]
        assert resource["resourceinstance"]["resourceinstanceid"] == resource_id
        assert "tiles" in resource
        assert len(resource["tiles"]) > 0

        # Find the tile with our reference data
        ref_node_id = test_node["nodeid"]
        ref_tile = None
        for tile in resource["tiles"]:
            if ref_node_id in tile.get("data", {}):
                ref_tile = tile
                break

        assert ref_tile is not None, "Should find tile with reference data"

        # Verify the reference data is present
        ref_data = ref_tile["data"][ref_node_id]
        assert ref_data is not None

        print(f"Successfully created tiles with updated collection: {new_collection_id}")
        print(f"Reference data in tile: {json.dumps(ref_data, indent=2)[:200]}...")

    def test_full_workflow_with_collection_creation(self, person_graph):
        """
        Complete end-to-end test demonstrating:
        1. Create a new collection from test values
        2. Change a node's collection to use the new collection
        3. Use label resolution to convert human-readable labels to UUIDs
        4. Convert JSON tree to tiles
        5. Verify the full pipeline works

        This simulates the real-world workflow where:
        - An admin creates a new controlled vocabulary
        - Updates a graph to use that vocabulary
        - Users submit data using human-readable labels
        - The system resolves labels and stores as UUIDs
        """
        import alizarin
        import alizarin_clm  # noqa: F401

        if alizarin.json_tree_to_tiles is None:
            pytest.skip("json_tree_to_tiles not available")

        if not hasattr(alizarin, "register_extension_mutation"):
            pytest.skip("Extension mutation API not available")

        from alizarin import (
            RustRdmCollection,
            RustRdmCache,
            set_global_rdm_cache,
            clear_global_rdm_cache,
        )

        try:
            # Step 1: Create a new collection from test values
            test_labels = [
                "Archaeologist",
                "Historian",
                "Conservator",
                "Curator",
                "Researcher",
            ]

            new_collection = RustRdmCollection.from_labels(
                name="Person Roles",
                labels=test_labels,
            )

            new_collection_id = new_collection.id
            print(f"Created collection with ID: {new_collection_id}")
            print(f"Collection has {len(new_collection)} concepts")

            # Verify all labels are in the collection
            for label in test_labels:
                concept = new_collection.find_by_label(label)
                assert concept is not None, f"Label '{label}' should be in collection"
                print(f"  - '{label}' -> {concept.id}")

            # Step 2: Set up the global RDM cache with our collection
            cache = RustRdmCache()
            cache.add_collection(new_collection)
            set_global_rdm_cache(cache)

            # Step 3: Change the 'test' node's collection to our new collection
            change_mutation = {
                "mutations": [
                    {
                        "Extension": {
                            "name": "clm.reference_change_collection",
                            "params": {
                                "node_id": "test",
                                "collection_id": new_collection_id
                            },
                            "conformance": "AlwaysConformant"
                        }
                    }
                ],
                "options": {}
            }

            graph_json = json.dumps(person_graph)

            updated_graph_json = alizarin.apply_mutations_with_extensions(
                graph_json,
                json.dumps(change_mutation),
            )

            updated_graph = json.loads(updated_graph_json)

            # Verify collection was updated
            test_node = next(
                n for n in updated_graph["nodes"]
                if n.get("alias") == "test"
            )
            assert test_node["config"]["controlledList"] == new_collection_id

            # Step 4: Create business data using human-readable label
            # (this is what a user would submit)
            # Note: The 'test' node has cardinality "1", so we use a single value
            person_tree = {
                "test": {"_value": "Archaeologist"},  # Label, not UUID
            }

            tree_json = json.dumps(person_tree)
            resource_id = "test-person-with-roles"

            # Step 5: Register graph and convert to tiles - labels should be resolved to UUIDs
            graph_id = alizarin.register_graph(updated_graph_json)
            result = alizarin.json_tree_to_tiles(
                tree_json=tree_json,
                resource_id=resource_id,
                graph_id=graph_id,
            )

            # Step 6: Verify the result
            assert "business_data" in result
            resources = result["business_data"]["resources"]
            assert len(resources) == 1

            resource = resources[0]
            assert resource["resourceinstance"]["resourceinstanceid"] == resource_id

            # Find the tile with our reference data
            ref_node_id = test_node["nodeid"]
            ref_tile = None
            for tile in resource["tiles"]:
                if ref_node_id in tile.get("data", {}):
                    ref_tile = tile
                    break

            assert ref_tile is not None, "Should find tile with reference data"

            # Verify the reference data contains resolved UUIDs
            ref_data = ref_tile["data"][ref_node_id]
            assert ref_data is not None

            # The label should have been resolved to UUID from our collection
            archaeologist_concept = new_collection.find_by_label("Archaeologist")

            print(f"\nResolved reference data:")
            print(f"  Expected Archaeologist UUID: {archaeologist_concept.id}")
            print(f"  Actual tile data: {json.dumps(ref_data, indent=2)[:500]}")

            # Verify the UUID is present in the tile data
            ref_data_str = json.dumps(ref_data)
            assert archaeologist_concept.id in ref_data_str, \
                f"Archaeologist UUID {archaeologist_concept.id} should be in tile data: {ref_data_str}"

            print("\n✓ Full workflow completed successfully!")
            print(f"  - Created collection '{new_collection_id}' with {len(test_labels)} items")
            print(f"  - Updated node 'test' to use new collection")
            print(f"  - Resolved label 'Archaeologist' -> UUID {archaeologist_concept.id}")
            print(f"  - Generated {len(resource['tiles'])} tiles")

        finally:
            # Clean up global cache
            clear_global_rdm_cache()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
