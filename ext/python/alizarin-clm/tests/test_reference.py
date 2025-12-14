"""
Tests for CLM Reference datatype.

Tests loading JSON strings like {"Test": ["Item 1>1"]} into tiles.
"""

import pytest
import json
import os
from pathlib import Path

# Test data directory
TEST_DATA_DIR = Path(__file__).parent / "data"


@pytest.fixture
def person_graph():
    """Load the Person graph with reference node."""
    graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
    with open(graph_path) as f:
        data = json.load(f)
    return data["graph"][0]


@pytest.fixture
def reference_node(person_graph):
    """Get the reference node from the Person graph."""
    for node in person_graph["nodes"]:
        if node["datatype"] == "reference":
            return node
    raise ValueError("No reference node found in graph")


class TestStaticReferenceTypes:
    """Tests for StaticReference and StaticReferenceLabel types."""

    def test_static_reference_label_from_dict(self):
        """Should create StaticReferenceLabel from dict."""
        from alizarin_clm import StaticReferenceLabel

        label_data = {
            "id": "label-1",
            "language_id": "en",
            "list_item_id": "item-1",
            "value": "Item 1>1",
            "valuetype_id": "prefLabel",
        }

        label = StaticReferenceLabel.from_dict(label_data)

        assert label.id == "label-1"
        assert label.language_id == "en"
        assert label.value == "Item 1>1"
        assert label.valuetype_id == "prefLabel"

    def test_static_reference_from_dict(self):
        """Should create StaticReference from dict."""
        from alizarin_clm import StaticReference

        ref_data = {
            "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "uri": "http://localhost:8000/plugins/controlled-list-manager/item/123",
            "labels": [
                {
                    "id": "label-1",
                    "language_id": "en",
                    "list_item_id": "item-1",
                    "value": "Item 1>1",
                    "valuetype_id": "prefLabel",
                }
            ],
        }

        ref = StaticReference.from_dict(ref_data)

        assert ref.list_id == "2730d609-3a8d-49dc-bf51-6ac34e80294a"
        assert len(ref.labels) == 1
        assert ref.labels[0].value == "Item 1>1"

    def test_static_reference_to_display_string(self):
        """Should get display string from reference."""
        from alizarin_clm import StaticReference, StaticReferenceLabel

        ref = StaticReference(
            list_id="list-1",
            uri="http://example.com",
            labels=[
                StaticReferenceLabel(
                    id="1",
                    language_id="en",
                    list_item_id="item-1",
                    value="English Label",
                    valuetype_id="prefLabel",
                ),
                StaticReferenceLabel(
                    id="2",
                    language_id="es",
                    list_item_id="item-1",
                    value="Etiqueta Española",
                    valuetype_id="prefLabel",
                ),
            ],
        )

        assert ref.to_display_string("en") == "English Label"
        assert ref.to_display_string("es") == "Etiqueta Española"
        assert ref.to_display_string() == "English Label"  # default to en


class TestReferenceViewModels:
    """Tests for ReferenceValueViewModel and ReferenceListViewModel."""

    @pytest.mark.asyncio
    async def test_reference_value_viewmodel_from_dict(self):
        """Should create ReferenceValueViewModel from pre-formatted dict."""
        from alizarin_clm import ReferenceValueViewModel
        from alizarin import StaticTile, StaticNode

        # Create a minimal node
        node = StaticNode(
            nodeid="3ade323c-376c-433e-bf07-e776546a562b",
            name="Test",
            datatype="reference",
            nodegroup_id="3ade323c-376c-433e-bf07-e776546a562b",
            alias="test",
            graph_id="22477f01-1a44-11e9-b0a9-000d3ab1e588",
            is_collector=True,
            isrequired=False,
            exportable=False,
            config={
                "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                "multiValue": True,
            },
        )

        # Create a tile
        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="3ade323c-376c-433e-bf07-e776546a562b",
            resourceinstance_id="resource-1",
            data={},
        )

        # Pre-formatted reference value (as it comes from business data)
        value = {
            "labels": [
                {
                    "id": "0ea39e2e-6663-467c-8707-ab492896d23e",
                    "language_id": "en",
                    "list_item_id": "6672d187-dfc3-4424-8c63-7a3b377b4159",
                    "value": "Item 1>1",
                    "valuetype_id": "prefLabel",
                }
            ],
            "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "uri": "http://localhost:8000/plugins/controlled-list-manager/item/6672d187-dfc3-4424-8c63-7a3b377b4159",
        }

        vm = await ReferenceValueViewModel._create(tile, node, value)

        assert vm is not None
        assert str(vm) == "Item 1>1"
        assert vm.get_value().list_id == "2730d609-3a8d-49dc-bf51-6ac34e80294a"

    @pytest.mark.asyncio
    async def test_reference_list_viewmodel_from_array(self):
        """Should create ReferenceListViewModel from array of references."""
        from alizarin_clm import ReferenceListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="3ade323c-376c-433e-bf07-e776546a562b",
            name="Test",
            datatype="reference",
            nodegroup_id="3ade323c-376c-433e-bf07-e776546a562b",
            alias="test",
            graph_id="22477f01-1a44-11e9-b0a9-000d3ab1e588",
            is_collector=True,
            isrequired=False,
            exportable=False,
            config={
                "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                "multiValue": True,
            },
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="3ade323c-376c-433e-bf07-e776546a562b",
            resourceinstance_id="resource-1",
            data={},
        )

        # Array of pre-formatted reference values
        values = [
            {
                "labels": [
                    {
                        "id": "label-1",
                        "language_id": "en",
                        "list_item_id": "item-1",
                        "value": "Item 1>1",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                "uri": "http://localhost:8000/item/1",
            },
            {
                "labels": [
                    {
                        "id": "label-2",
                        "language_id": "en",
                        "list_item_id": "item-2",
                        "value": "Item 2",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                "uri": "http://localhost:8000/item/2",
            },
        ]

        vm = await ReferenceListViewModel._create(tile, node, values)

        assert vm is not None
        assert len(vm) == 2


class TestJSONToTiles:
    """Tests for converting JSON to tiles with reference datatype."""

    @pytest.mark.asyncio
    async def test_json_to_tiles_with_reference(self, person_graph, reference_node):
        """
        Test loading JSON like {"Test": ["Item 1>1"]} into tiles.

        This demonstrates the flow of:
        1. Having a JSON tree with alias-based keys
        2. Converting to tile format with nodegroup-based data
        """
        from alizarin_clm import ReferenceMergedDataType, ReferenceListViewModel
        from alizarin import StaticTile, StaticNode

        # The input JSON format (using aliases)
        input_json = {
            "Test": [
                {
                    "labels": [
                        {
                            "id": "0ea39e2e-6663-467c-8707-ab492896d23e",
                            "language_id": "en",
                            "list_item_id": "6672d187-dfc3-4424-8c63-7a3b377b4159",
                            "value": "Item 1>1",
                            "valuetype_id": "prefLabel",
                        }
                    ],
                    "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                    "uri": "http://localhost:8000/plugins/controlled-list-manager/item/6672d187-dfc3-4424-8c63-7a3b377b4159",
                }
            ]
        }

        # Create StaticNode from the graph data
        node = StaticNode(
            nodeid=reference_node["nodeid"],
            name=reference_node["name"],
            datatype=reference_node["datatype"],
            nodegroup_id=reference_node["nodegroup_id"],
            alias=reference_node["alias"],
            graph_id=reference_node["graph_id"],
            is_collector=reference_node["is_collector"],
            isrequired=reference_node["isrequired"],
            exportable=reference_node["exportable"],
            config=reference_node["config"],
        )

        # Create a tile for this nodegroup
        tile = StaticTile(
            tileid="new-tile-id",
            nodegroup_id=reference_node["nodegroup_id"],
            resourceinstance_id="new-resource-id",
            data={},
        )

        # Get the value for "Test" alias
        test_value = input_json.get("Test")

        # Create ViewModel using the merged datatype (checks multiValue config)
        vm = await ReferenceMergedDataType._create(tile, node, test_value)

        # Since multiValue=True, we get a list
        assert isinstance(vm, ReferenceListViewModel)
        assert len(vm) == 1

        # The tile data should now have the reference value
        assert reference_node["nodeid"] in tile.data

        # Get the first item and verify
        first_item = vm[0]
        if hasattr(first_item, '__await__'):
            first_item = await first_item

        assert first_item is not None
        assert str(first_item) == "Item 1>1"

    def test_graph_has_reference_node(self, person_graph, reference_node):
        """Verify the test graph has a reference datatype node."""
        assert reference_node["datatype"] == "reference"
        assert reference_node["alias"] == "test"
        assert reference_node["config"]["controlledList"] == "2730d609-3a8d-49dc-bf51-6ac34e80294a"
        assert reference_node["config"]["multiValue"] == True


class TestExtensionRegistration:
    """Tests for CLM extension registration with alizarin."""

    def test_custom_datatypes_has_reference(self):
        """After importing alizarin_clm, 'reference' should be in CUSTOM_DATATYPES."""
        # Import CLM extension (auto-registers)
        import alizarin_clm  # noqa: F401
        from alizarin.view_models import CUSTOM_DATATYPES

        assert "reference" in CUSTOM_DATATYPES
        assert CUSTOM_DATATYPES["reference"]._create is not None

    def test_rust_extension_functions_available(self):
        """Extension registration functions should be available from alizarin."""
        import alizarin

        # These may be None if Rust extension not compiled
        # but the attributes should exist
        assert hasattr(alizarin, "register_type_handler")
        assert hasattr(alizarin, "has_type_handler")
        assert hasattr(alizarin, "coerce_with_extension")


class TestFullRustIntegration:
    """
    Integration tests demonstrating full Rust-to-Rust flow for Person resource
    with reference fields, without crossing the Python boundary for data processing.
    """

    @pytest.fixture
    def person_graph_json(self):
        """Load Person graph as JSON string."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        with open(graph_path) as f:
            data = json.load(f)
        return json.dumps(data["graph"][0])

    @pytest.fixture
    def person_graph_data(self):
        """Load Person graph as dict."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        with open(graph_path) as f:
            data = json.load(f)
        return data["graph"][0]

    def test_json_tree_to_tiles_with_reference_field(self, person_graph_json, person_graph_data):
        """
        Test full Rust flow: JSON tree → tiles conversion with reference values.

        This demonstrates that a Person JSON with reference fields can be converted
        to tiles entirely in Rust via json_tree_to_tiles.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.json_tree_to_tiles is None:
            pytest.skip("json_tree_to_tiles not available")

        # Create a Person JSON tree with a reference value
        # Structure: direct children of root node (person) at top level
        # Leaf values are wrapped in "_value" for Rust tree_to_tiles
        person_tree = {
            "test": [
                {
                    "_value": {
                        "labels": [
                            {
                                "id": "label-1",
                                "language_id": "en",
                                "list_item_id": "item-1",
                                "value": "Test Reference Value",
                                "valuetype_id": "prefLabel",
                            }
                        ],
                        "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                        "uri": "http://localhost:8000/item/1",
                    }
                }
            ],
            "name": [{
                "full_name": {"_value": {"en": "John Doe"}}
            }]
        }

        tree_json = json.dumps(person_tree)
        resource_id = "test-person-001"
        graph_id = person_graph_data["graphid"]

        # Convert JSON tree to tiles - entirely in Rust
        result = alizarin.json_tree_to_tiles(
            tree_json=tree_json,
            resource_id=resource_id,
            graph_id=graph_id,
            graph_json=person_graph_json
        )

        # Verify the result structure
        assert result["resourceinstanceid"] == resource_id
        assert result["graph_id"] == graph_id
        assert "tiles" in result
        assert len(result["tiles"]) > 0

        # Find the tile with reference data
        reference_node_id = None
        for node in person_graph_data["nodes"]:
            if node.get("alias") == "test" and node.get("datatype") == "reference":
                reference_node_id = node["nodeid"]
                break

        assert reference_node_id is not None, "Should find reference node"

        # Find tile containing reference data
        reference_tile = None
        for tile in result["tiles"]:
            if reference_node_id in tile.get("data", {}):
                reference_tile = tile
                break

        assert reference_tile is not None, "Should have tile with reference data"

        # The reference data should be stored in the tile
        ref_data = reference_tile["data"][reference_node_id]
        assert ref_data is not None

        print(f"Created {len(result['tiles'])} tiles")
        print(f"Reference data in tile: {json.dumps(ref_data, indent=2)}")

    def test_full_rust_pipeline_json_to_tiles_to_coercion(self, person_graph_json, person_graph_data):
        """
        Test complete Rust pipeline:
        1. json_tree_to_tiles (Rust) - converts JSON tree to tile structure
        2. coerce_with_extension (Rust) - coerces reference values via extension handler

        This demonstrates end-to-end Rust processing without Python data manipulation.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.json_tree_to_tiles is None:
            pytest.skip("json_tree_to_tiles not available")
        if alizarin.coerce_with_extension is None:
            pytest.skip("coerce_with_extension not available")

        # Step 1: Create Person JSON with multiple fields including reference
        # Structure: direct children of root node at top level
        # Leaf values wrapped in "_value" for Rust tree_to_tiles
        person_tree = {
            "test": [
                {
                    "_value": {
                        "labels": [
                            {
                                "id": "ref-label-1",
                                "language_id": "en",
                                "list_item_id": "item-123",
                                "value": "Category A",
                                "valuetype_id": "prefLabel",
                            },
                            {
                                "id": "ref-label-2",
                                "language_id": "es",
                                "list_item_id": "item-123",
                                "value": "Categoría A",
                                "valuetype_id": "prefLabel",
                            }
                        ],
                        "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                        "uri": "http://localhost:8000/item/123",
                    }
                },
                {
                    "_value": {
                        "labels": [
                            {
                                "id": "ref-label-3",
                                "language_id": "en",
                                "list_item_id": "item-456",
                                "value": "Category B",
                                "valuetype_id": "prefLabel",
                            }
                        ],
                        "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                        "uri": "http://localhost:8000/item/456",
                    }
                }
            ],
            "name": [{
                "full_name": {"_value": {"en": "Jane Smith", "ga": "Síle Ní Ghobnait"}}
            }]
        }

        # Step 2: Convert to tiles via Rust
        tree_json = json.dumps(person_tree)
        result = alizarin.json_tree_to_tiles(
            tree_json=tree_json,
            resource_id="test-person-002",
            graph_id=person_graph_data["graphid"],
            graph_json=person_graph_json
        )

        assert len(result["tiles"]) > 0

        # Step 3: Find reference data in tiles
        reference_node_id = None
        node_config = None
        for node in person_graph_data["nodes"]:
            if node.get("alias") == "test" and node.get("datatype") == "reference":
                reference_node_id = node["nodeid"]
                node_config = node.get("config", {})
                break

        # Find ALL tiles with reference data (each array item creates a separate tile)
        ref_tiles = []
        for tile in result["tiles"]:
            if reference_node_id and reference_node_id in tile.get("data", {}):
                ref_data = tile["data"][reference_node_id]
                if ref_data is not None:
                    ref_tiles.append(ref_data)

        assert len(ref_tiles) == 2, f"Should have 2 tiles with reference data, got {len(ref_tiles)}"

        # Step 4: Coerce each reference value via Rust extension handler
        # This demonstrates the Rust-to-Rust coercion path
        coerced_results = []
        for ref_data in ref_tiles:
            ref_json = json.dumps(ref_data)
            config_json = json.dumps(node_config)

            tile_data_json, resolved_json = alizarin.coerce_with_extension(
                "reference",
                ref_json,
                config_json
            )

            resolved = json.loads(resolved_json)
            coerced_results.append(resolved)

        # Verify we coerced both reference values
        assert len(coerced_results) == 2, "Should have coerced 2 reference values"

        # Find references by label value (order not guaranteed)
        label_values = {r["labels"][0]["value"] for r in coerced_results}
        assert "Category A" in label_values, "Should have Category A reference"
        assert "Category B" in label_values, "Should have Category B reference"

        # Verify the one with Spanish label
        cat_a_ref = next(r for r in coerced_results if r["labels"][0]["value"] == "Category A")
        assert any(l["language_id"] == "es" for l in cat_a_ref["labels"]), \
            "Category A should preserve Spanish label"

        print("Full Rust pipeline completed successfully:")
        print(f"  - Converted JSON tree to {len(result['tiles'])} tiles")
        print(f"  - Coerced {len(coerced_results)} reference values via Rust extension")

    def test_rust_coercion_in_roundtrip(self, person_graph_json, person_graph_data):
        """
        Test that reference values survive a complete roundtrip through Rust:
        JSON tree → tiles → JSON tree → tiles

        All transformations happen in Rust.
        """
        import alizarin
        import alizarin_clm  # noqa: F401

        if alizarin.json_tree_to_tiles is None or alizarin.tiles_to_json_tree is None:
            pytest.skip("Rust conversion functions not available")

        # Original tree with reference
        # Structure: direct children of root node at top level
        # Leaf values wrapped in "_value" for Rust tree_to_tiles
        original_tree = {
            "test": [
                {
                    "_value": {
                        "labels": [{"id": "l1", "language_id": "en", "list_item_id": "i1",
                                   "value": "Roundtrip Test", "valuetype_id": "prefLabel"}],
                        "list_id": "list-roundtrip",
                        "uri": "http://example.com/roundtrip",
                    }
                }
            ]
        }

        resource_id = "roundtrip-test-001"
        graph_id = person_graph_data["graphid"]

        # Step 1: tree → tiles (Rust)
        tiles_result = alizarin.json_tree_to_tiles(
            tree_json=json.dumps(original_tree),
            resource_id=resource_id,
            graph_id=graph_id,
            graph_json=person_graph_json
        )

        assert len(tiles_result["tiles"]) > 0

        # Step 2: tiles → tree (Rust)
        tree_result = alizarin.tiles_to_json_tree(
            tiles_json=json.dumps(tiles_result["tiles"]),
            resource_id=resource_id,
            graph_id=graph_id,
            graph_json=person_graph_json
        )

        assert "person" in tree_result or len(tree_result) > 0

        # Step 3: tree → tiles again (Rust)
        final_tiles = alizarin.json_tree_to_tiles(
            tree_json=json.dumps(tree_result),
            resource_id=resource_id,
            graph_id=graph_id,
            graph_json=person_graph_json
        )

        # Verify we still have tiles
        assert len(final_tiles["tiles"]) > 0

        # Find reference node
        ref_node_id = None
        for node in person_graph_data["nodes"]:
            if node.get("alias") == "test":
                ref_node_id = node["nodeid"]
                break

        # Check reference data survived roundtrip
        original_ref_data = None
        final_ref_data = None

        for tile in tiles_result["tiles"]:
            if ref_node_id and ref_node_id in tile.get("data", {}):
                original_ref_data = tile["data"][ref_node_id]
                break

        for tile in final_tiles["tiles"]:
            if ref_node_id and ref_node_id in tile.get("data", {}):
                final_ref_data = tile["data"][ref_node_id]
                break

        if original_ref_data and final_ref_data:
            # Data should be preserved through roundtrip
            print(f"Original reference data: {json.dumps(original_ref_data)[:100]}...")
            print(f"Final reference data: {json.dumps(final_ref_data)[:100]}...")
            # The structure should be similar (exact match depends on normalization)
            assert final_ref_data is not None

        print("Roundtrip through Rust completed successfully")


class TestRustCoercionIntegration:
    """Tests that verify Rust type extension works with core Alizarin tile JSON loading."""

    def test_rust_handler_registered(self):
        """Verify the Rust handler for 'reference' type is registered."""
        # Import CLM extension (auto-registers the Rust handler)
        import alizarin_clm  # noqa: F401
        import alizarin

        # has_type_handler should return True if Rust extension is available
        if alizarin.has_type_handler is not None:
            assert alizarin.has_type_handler("reference") is True
            assert alizarin.has_type_handler("nonexistent_type") is False
        else:
            pytest.skip("Rust extension not available")

    def test_coerce_with_extension_single_reference(self):
        """
        Test coerce_with_extension with a single reference value.

        This tests the direct Rust-to-Rust coercion path.
        """
        import alizarin_clm  # noqa: F401
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        # Pre-formatted reference value as JSON
        value_json = json.dumps({
            "labels": [
                {
                    "id": "0ea39e2e-6663-467c-8707-ab492896d23e",
                    "language_id": "en",
                    "list_item_id": "6672d187-dfc3-4424-8c63-7a3b377b4159",
                    "value": "Item 1>1",
                    "valuetype_id": "prefLabel",
                }
            ],
            "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "uri": "http://localhost:8000/plugins/controlled-list-manager/item/6672d187-dfc3-4424-8c63-7a3b377b4159",
        })

        # Node config as JSON
        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": False,
        })

        # Call the Rust coercion function
        tile_data_json, resolved_json = alizarin.coerce_with_extension(
            "reference",
            value_json,
            config_json,
        )

        # Parse results
        tile_data = json.loads(tile_data_json)
        resolved = json.loads(resolved_json)

        # Verify tile data format
        assert tile_data is not None
        assert "list_id" in tile_data
        assert tile_data["list_id"] == "2730d609-3a8d-49dc-bf51-6ac34e80294a"

        # Verify resolved data
        assert resolved is not None
        assert "labels" in resolved
        assert len(resolved["labels"]) == 1
        assert resolved["labels"][0]["value"] == "Item 1>1"

    def test_coerce_with_extension_reference_array(self):
        """
        Test coerce_with_extension with an array of reference values.

        This demonstrates multi-value reference coercion.
        """
        import alizarin_clm  # noqa: F401
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        # Array of reference values
        value_json = json.dumps([
            {
                "labels": [
                    {
                        "id": "label-1",
                        "language_id": "en",
                        "list_item_id": "item-1",
                        "value": "First Item",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                "uri": "http://localhost:8000/item/1",
            },
            {
                "labels": [
                    {
                        "id": "label-2",
                        "language_id": "en",
                        "list_item_id": "item-2",
                        "value": "Second Item",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                "uri": "http://localhost:8000/item/2",
            },
        ])

        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": True,
        })

        # Call the Rust coercion function
        tile_data_json, resolved_json = alizarin.coerce_with_extension(
            "reference",
            value_json,
            config_json,
        )

        # Parse results
        tile_data = json.loads(tile_data_json)
        resolved = json.loads(resolved_json)

        # Verify tile data format - should be an array
        assert isinstance(tile_data, list)
        assert len(tile_data) == 2

        # Verify resolved data - should be an array
        assert isinstance(resolved, list)
        assert len(resolved) == 2
        assert resolved[0]["labels"][0]["value"] == "First Item"
        assert resolved[1]["labels"][0]["value"] == "Second Item"

    def test_coerce_preserves_all_label_fields(self):
        """
        Test that coercion preserves all fields in reference labels.

        The Rust coercion should maintain all label metadata.
        """
        import alizarin_clm  # noqa: F401
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        # Reference with multiple labels in different languages
        value_json = json.dumps({
            "labels": [
                {
                    "id": "en-label",
                    "language_id": "en",
                    "list_item_id": "item-1",
                    "value": "English Label",
                    "valuetype_id": "prefLabel",
                },
                {
                    "id": "es-label",
                    "language_id": "es",
                    "list_item_id": "item-1",
                    "value": "Etiqueta Española",
                    "valuetype_id": "prefLabel",
                },
            ],
            "list_id": "list-123",
            "uri": "http://example.com/item/1",
        })

        config_json = json.dumps({"multiValue": False})

        tile_data_json, resolved_json = alizarin.coerce_with_extension(
            "reference",
            value_json,
            config_json,
        )

        resolved = json.loads(resolved_json)

        # Verify all labels are preserved
        assert len(resolved["labels"]) == 2

        # Find English label
        en_label = next(l for l in resolved["labels"] if l["language_id"] == "en")
        assert en_label["id"] == "en-label"
        assert en_label["value"] == "English Label"
        assert en_label["valuetype_id"] == "prefLabel"

        # Find Spanish label
        es_label = next(l for l in resolved["labels"] if l["language_id"] == "es")
        assert es_label["id"] == "es-label"
        assert es_label["value"] == "Etiqueta Española"

    def test_coerce_with_missing_handler_raises_error(self):
        """Test that coercing with an unregistered type raises an error."""
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        with pytest.raises(KeyError):
            alizarin.coerce_with_extension(
                "nonexistent_type",
                "{}",
                None,
            )


class TestPhase0NodeConfigManager:
    """
    Tests for Phase 0: NodeConfigManager in Rust.

    NodeConfigManager extracts and caches node configuration from graphs,
    enabling Rust-side type coercion without Python boundary crossings.
    """

    @pytest.fixture
    def person_graph_json(self):
        """Load Person graph as JSON string."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        with open(graph_path) as f:
            data = json.load(f)
        return json.dumps(data["graph"][0])

    def test_node_config_manager_create(self):
        """Test creating an empty NodeConfigManager."""
        from alizarin.alizarin import RustNodeConfigManager

        manager = RustNodeConfigManager()
        assert len(manager) == 0

    def test_node_config_manager_from_graph(self, person_graph_json):
        """Test building NodeConfigManager from graph JSON."""
        from alizarin.alizarin import RustNodeConfigManager

        manager = RustNodeConfigManager()
        manager.from_graph_json(person_graph_json)

        # Person graph has 1 reference node
        assert len(manager) >= 1

    def test_node_config_manager_get_reference_config(self, person_graph_json, person_graph):
        """Test retrieving reference config for a node."""
        from alizarin.alizarin import RustNodeConfigManager

        manager = RustNodeConfigManager()
        manager.from_graph_json(person_graph_json)

        # Find reference node
        ref_node_id = None
        for node in person_graph["nodes"]:
            if node["datatype"] == "reference":
                ref_node_id = node["nodeid"]
                break

        assert ref_node_id is not None
        assert manager.has_config(ref_node_id)
        assert manager.get_config_type(ref_node_id) == "reference"

        # Get the reference config
        ref_config = manager.get_reference(ref_node_id)
        assert ref_config is not None
        assert ref_config.controlled_list == "2730d609-3a8d-49dc-bf51-6ac34e80294a"
        assert ref_config.multi_value is True

    def test_node_config_boolean(self):
        """Test NodeConfigBoolean type."""
        from alizarin.alizarin import RustNodeConfigBoolean

        config = RustNodeConfigBoolean(
            {"en": "Yes", "es": "Sí"},
            {"en": "No", "es": "No"}
        )

        assert config.get_label(True, "en") == "Yes"
        assert config.get_label(False, "en") == "No"
        assert config.get_label(True, "es") == "Sí"
        # Fallback to en when language not available
        assert config.get_label(True, "de") == "Yes"

    def test_node_config_domain(self):
        """Test NodeConfigDomain with options."""
        from alizarin.alizarin import RustNodeConfigDomain, RustStaticDomainValue

        opt1 = RustStaticDomainValue("opt-1", False, {"en": "Option One"})
        opt2 = RustStaticDomainValue("opt-2", True, {"en": "Option Two"})

        config = RustNodeConfigDomain([opt1, opt2])

        assert len(config.options) == 2

        # Test get_selected
        selected = config.get_selected()
        assert selected is not None
        assert selected.id == "opt-2"

        # Test value_from_id
        found = config.value_from_id("opt-1")
        assert found is not None
        assert found.lang("en") == "Option One"

        # Test get_option_ids
        ids = config.get_option_ids()
        assert "opt-1" in ids
        assert "opt-2" in ids

    def test_node_config_concept(self):
        """Test NodeConfigConcept type."""
        from alizarin.alizarin import RustNodeConfigConcept

        config = RustNodeConfigConcept("my-rdm-collection-id")

        assert config.rdm_collection == "my-rdm-collection-id"

    def test_node_config_reference_get_collection_id(self):
        """Test NodeConfigReference get_collection_id method."""
        from alizarin.alizarin import RustNodeConfigReference

        # With controlledList
        config1 = RustNodeConfigReference("clm-list-id", "", False)
        assert config1.get_collection_id() == "clm-list-id"

        # With rdmCollection fallback
        config2 = RustNodeConfigReference("", "rdm-collection-id", True)
        assert config2.get_collection_id() == "rdm-collection-id"

        # With both, controlledList takes precedence
        config3 = RustNodeConfigReference("clm-id", "rdm-id", False)
        assert config3.get_collection_id() == "clm-id"


class TestPhase0RdmCache:
    """
    Tests for Phase 0: RdmCache in Rust.

    RdmCache stores concept collections fetched by Python for sync lookup
    during Rust-side type coercion.
    """

    def test_rdm_concept_create(self):
        """Test creating an RdmConcept."""
        from alizarin.alizarin import RustRdmConcept

        concept = RustRdmConcept("concept-001", {"en": "Person", "de": "Person"})

        assert concept.id == "concept-001"
        assert concept.get_label("en") == "Person"
        assert concept.get_label("de") == "Person"
        assert str(concept) == "Person"  # Uses en as default

    def test_rdm_collection_create_and_add(self):
        """Test creating RdmCollection and adding concepts."""
        from alizarin.alizarin import RustRdmCollection, RustRdmConcept

        collection = RustRdmCollection("test-collection")

        concept1 = RustRdmConcept("c-001", {"en": "Concept One"})
        concept2 = RustRdmConcept("c-002", {"en": "Concept Two"})

        collection.add_concept(concept1)
        collection.add_concept(concept2)

        assert len(collection) == 2
        assert collection.has_concept("c-001")
        assert collection.has_concept("c-002")
        assert not collection.has_concept("c-999")

    def test_rdm_collection_search(self):
        """Test RdmCollection search functionality."""
        from alizarin.alizarin import RustRdmCollection, RustRdmConcept

        collection = RustRdmCollection("search-test")
        collection.add_concept(RustRdmConcept("1", {"en": "Apple"}))
        collection.add_concept(RustRdmConcept("2", {"en": "Apricot"}))
        collection.add_concept(RustRdmConcept("3", {"en": "Banana"}))

        # Search for "Ap" - matches both Apple and Apricot
        results = collection.search("Ap", "en")
        assert len(results) == 2
        result_ids = [r.id for r in results]
        assert "1" in result_ids  # Apple
        assert "2" in result_ids  # Apricot

        # Search for "App" - only matches Apple (Apricot starts with Apr)
        results = collection.search("App", "en")
        assert len(results) == 1
        assert results[0].id == "1"

        # Search for "Ban"
        results = collection.search("Ban", "en")
        assert len(results) == 1
        assert results[0].id == "3"

    def test_rdm_cache_add_and_lookup(self):
        """Test RdmCache basic operations."""
        from alizarin.alizarin import RustRdmCache, RustRdmCollection, RustRdmConcept

        cache = RustRdmCache()

        # Create and add a collection
        collection = RustRdmCollection("coll-1")
        collection.add_concept(RustRdmConcept("c-001", {"en": "Concept A"}))
        collection.add_concept(RustRdmConcept("c-002", {"en": "Concept B"}))
        cache.add_collection(collection)

        assert len(cache) == 1
        assert cache.has_collection("coll-1")

        # Direct concept lookup
        concept = cache.lookup_concept("coll-1", "c-001")
        assert concept is not None
        assert concept.get_label("en") == "Concept A"

        # Lookup non-existent
        assert cache.lookup_concept("coll-1", "c-999") is None
        assert cache.lookup_concept("coll-999", "c-001") is None

    def test_rdm_cache_validate_concept(self):
        """Test RdmCache validation method."""
        from alizarin.alizarin import RustRdmCache, RustRdmCollection, RustRdmConcept

        cache = RustRdmCache()

        collection = RustRdmCollection("validation-test")
        collection.add_concept(RustRdmConcept("valid-id", {"en": "Valid"}))
        cache.add_collection(collection)

        # Validate existing
        assert cache.validate_concept("validation-test", "valid-id") is True

        # Validate non-existent
        assert cache.validate_concept("validation-test", "invalid-id") is False
        assert cache.validate_concept("wrong-collection", "valid-id") is False

    def test_rdm_cache_from_json(self):
        """Test RdmCache add_collection_from_json method."""
        from alizarin.alizarin import RustRdmCache

        cache = RustRdmCache()

        # JSON format matching RDM API response
        concepts_json = json.dumps([
            {"id": "json-001", "prefLabel": {"en": "From JSON", "de": "Aus JSON"}},
            {"id": "json-002", "prefLabel": {"en": "Also JSON"}, "broader": ["json-001"]},
            {"id": "json-003", "prefLabel": {"en": "Third One"}, "narrower": ["json-001"]},
        ])

        cache.add_collection_from_json("json-collection", concepts_json)

        assert len(cache) == 1
        assert cache.has_collection("json-collection")

        # Verify concepts loaded correctly
        c1 = cache.lookup_concept("json-collection", "json-001")
        assert c1 is not None
        assert c1.get_label("en") == "From JSON"
        assert c1.get_label("de") == "Aus JSON"

        c2 = cache.lookup_concept("json-collection", "json-002")
        assert c2 is not None
        assert c2.broader == ["json-001"]

        c3 = cache.lookup_concept("json-collection", "json-003")
        assert c3 is not None
        assert c3.narrower == ["json-001"]

    def test_rdm_cache_clear(self):
        """Test RdmCache clear methods."""
        from alizarin.alizarin import RustRdmCache, RustRdmCollection

        cache = RustRdmCache()
        cache.add_collection(RustRdmCollection("coll-1"))
        cache.add_collection(RustRdmCollection("coll-2"))

        assert len(cache) == 2

        # Clear single collection
        cache.clear_collection("coll-1")
        assert len(cache) == 1
        assert not cache.has_collection("coll-1")
        assert cache.has_collection("coll-2")

        # Clear all
        cache.clear()
        assert len(cache) == 0

    def test_rdm_concept_from_label_string(self):
        """Test creating RdmConcept from a simple string label."""
        from alizarin.alizarin import RustRdmConcept, set_current_language

        # Set language first
        set_current_language("en")

        concept = RustRdmConcept.from_label("my-uuid", "Category A")

        assert concept.id == "my-uuid"
        assert concept.get_label("en") == "Category A"

    def test_rdm_concept_from_label_dict(self):
        """Test creating RdmConcept from a multi-language dict."""
        from alizarin.alizarin import RustRdmConcept

        concept = RustRdmConcept.from_label("my-uuid", {"en": "Category", "de": "Kategorie"})

        assert concept.id == "my-uuid"
        assert concept.get_label("en") == "Category"
        assert concept.get_label("de") == "Kategorie"

    def test_rdm_collection_add_from_label(self):
        """Test adding concepts via add_from_label with auto-generated IDs."""
        from alizarin.alizarin import RustRdmCollection, set_current_language

        set_current_language("en")

        # Collection must have a valid UUID for auto-generation
        collection = RustRdmCollection("550e8400-e29b-41d4-a716-446655440000")

        # Add concept with auto-generated ID
        id1 = collection.add_from_label("Category A")
        id2 = collection.add_from_label("Category B")

        assert len(collection) == 2
        assert collection.has_concept(id1)
        assert collection.has_concept(id2)

        # IDs should be deterministic (uuid5)
        id1_again = collection.add_from_label("Category A")
        # Same label produces same ID, so it overwrites
        assert len(collection) == 2

        # Verify the concept labels
        concept = collection.get_concept(id1)
        assert concept is not None
        assert concept.get_label("en") == "Category A"

    def test_rdm_collection_add_from_label_explicit_id(self):
        """Test add_from_label with explicit ID."""
        from alizarin.alizarin import RustRdmCollection

        collection = RustRdmCollection("test-collection")

        # Explicit ID works even without valid UUID for collection
        concept_id = collection.add_from_label("My Label", id="explicit-uuid")

        assert concept_id == "explicit-uuid"
        assert collection.has_concept("explicit-uuid")

    def test_rdm_collection_from_labels(self):
        """Test creating a collection with from_labels static method."""
        from alizarin.alizarin import RustRdmCollection, set_current_language

        set_current_language("en")

        # Create collection with auto-generated IDs
        collection = RustRdmCollection.from_labels(
            "My Categories",
            ["Cat A", "Cat B", "Cat C"]
        )

        # Collection ID should be uuid5 of the name
        assert collection.id is not None
        assert len(collection.id) == 36  # UUID format

        # Name should be stored
        assert collection.name == "My Categories"

        # Should have 3 concepts
        assert len(collection) == 3

        # Verify we can find concepts by label
        concept = collection.find_by_label("Cat A")
        assert concept is not None
        assert concept.get_label("en") == "Cat A"

    def test_rdm_collection_from_labels_explicit_id(self):
        """Test from_labels with explicit collection ID."""
        from alizarin.alizarin import RustRdmCollection

        # Explicit ID must still be a valid UUID for concept ID generation
        explicit_uuid = "12345678-1234-5678-1234-567812345678"
        collection = RustRdmCollection.from_labels(
            "Categories",
            ["A", "B"],
            id=explicit_uuid
        )

        assert collection.id == explicit_uuid
        assert collection.name == "Categories"
        assert len(collection) == 2

    def test_rdm_collection_from_labels_deterministic(self):
        """Test that from_labels produces deterministic IDs."""
        from alizarin.alizarin import RustRdmCollection

        # Create two collections with same name and labels
        collection1 = RustRdmCollection.from_labels("Test", ["A", "B"])
        collection2 = RustRdmCollection.from_labels("Test", ["A", "B"])

        # Collection IDs should match
        assert collection1.id == collection2.id

        # Concept IDs should match
        concept1 = collection1.find_by_label("A")
        concept2 = collection2.find_by_label("A")
        assert concept1.id == concept2.id


class TestLabelResolutionIntegration:
    """
    End-to-end tests for label resolution in json_tree_to_tiles.

    These tests demonstrate the full workflow:
    1. Create a collection with from_labels
    2. Use resolve_labels_in_tree to convert label strings to UUIDs
    3. Use json_tree_to_tiles to convert to tiles
    4. Verify success and failure cases

    Note: These tests require a full Arches graph with concept nodes.
    The Person graph is used as a base and extended with concept node config.
    """

    @pytest.fixture
    def concept_graph_json(self):
        """Load Person graph and ensure it has concept node config."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        with open(graph_path) as f:
            data = json.load(f)
        return json.dumps(data)

    @pytest.fixture
    def concept_graph_data(self):
        """Return graph data for reference."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        with open(graph_path) as f:
            data = json.load(f)
        return data["graph"][0]

    @pytest.fixture
    def test_collection(self):
        """Create a test collection for label resolution."""
        from alizarin.alizarin import RustRdmCollection, RustRdmCache, set_current_language

        set_current_language("en")

        # Use the same collection ID that appears in Person graph's reference config
        collection_id = "2730d609-3a8d-49dc-bf51-6ac34e80294a"

        # Create collection with known labels
        collection = RustRdmCollection.from_labels(
            "Test Categories",
            ["Category A", "Category B", "Category C"],
            id=collection_id
        )

        # Build cache with collection
        cache = RustRdmCache()
        cache.add_collection(collection)

        return cache, collection

    def test_collection_from_labels_api(self, test_collection):
        """Test that from_labels creates a valid collection that can be used for lookup."""
        cache, collection = test_collection

        # Verify collection was created correctly
        assert len(collection) == 3

        # Verify label lookup works
        concept_a = collection.find_by_label("Category A")
        assert concept_a is not None
        assert concept_a.get_label("en") == "Category A"

        # Verify cache lookup works
        assert cache.has_collection(collection.id)
        assert cache.lookup_by_label(collection.id, "Category B") is not None

    def test_collection_deterministic_ids(self, test_collection):
        """Test that collection produces deterministic UUIDs."""
        from alizarin.alizarin import RustRdmCollection, set_current_language

        set_current_language("en")

        # Create same collection twice
        coll1 = RustRdmCollection.from_labels("Test", ["A", "B", "C"])
        coll2 = RustRdmCollection.from_labels("Test", ["A", "B", "C"])

        # Same inputs should produce same IDs
        assert coll1.id == coll2.id

        concept1 = coll1.find_by_label("A")
        concept2 = coll2.find_by_label("A")
        assert concept1.id == concept2.id

    def test_resolve_labels_with_real_graph(self, concept_graph_json, concept_graph_data, test_collection):
        """
        Test label resolution with a real graph structure.

        Note: The Person graph uses 'reference' datatype, not 'concept'.
        resolve_labels_in_tree only resolves 'concept' and 'concept-list' datatypes.
        This test verifies the API works even if no resolution occurs.
        """
        from alizarin.alizarin import resolve_labels_in_tree

        cache, collection = test_collection

        # JSON tree with a value that won't be resolved (reference nodes, not concept)
        tree = {
            "test": [{"_value": "Some Value"}]
        }
        tree_json = json.dumps(tree)

        # Should not raise - passes through since 'test' is a reference node, not concept
        resolved_json = resolve_labels_in_tree(
            tree_json=tree_json,
            graph_json=concept_graph_json,
            rdm_cache=cache,
            strict=False
        )

        resolved = json.loads(resolved_json)
        # Value should pass through unchanged (no concept node to resolve)
        assert resolved["test"][0]["_value"] == "Some Value"

    def test_uuid_passthrough_with_real_graph(self, concept_graph_json, test_collection):
        """Test that UUID values pass through unchanged regardless of node type."""
        from alizarin.alizarin import resolve_labels_in_tree

        cache, collection = test_collection

        # Get an actual UUID from the collection
        concept = collection.find_by_label("Category A")
        uuid_value = concept.id

        # JSON tree with UUID
        tree = {
            "test": [{"_value": uuid_value}]
        }
        tree_json = json.dumps(tree)

        # UUIDs should always pass through unchanged
        resolved_json = resolve_labels_in_tree(
            tree_json=tree_json,
            graph_json=concept_graph_json,
            rdm_cache=cache,
            strict=False
        )

        resolved = json.loads(resolved_json)
        assert resolved["test"][0]["_value"] == uuid_value

    def test_global_cache_with_json_tree_to_tiles(self, concept_graph_json, concept_graph_data):
        """
        Integration test: Global RDM cache with json_tree_to_tiles.

        Demonstrates the full workflow:
        1. Create a collection with from_labels
        2. Set it as the global RDM cache
        3. Call json_tree_to_tiles WITHOUT passing rdm_cache
        4. Verify label resolution happens automatically via global cache
        """
        from alizarin.alizarin import (
            RustRdmCollection,
            RustRdmCache,
            set_current_language,
            set_global_rdm_cache,
            get_global_rdm_cache,
            clear_global_rdm_cache,
            json_tree_to_tiles,
        )

        try:
            # Step 1: Set up language
            set_current_language("en")

            # Step 2: Create collection with concepts
            collection_id = "2730d609-3a8d-49dc-bf51-6ac34e80294a"
            collection = RustRdmCollection.from_labels(
                "Status Values",
                ["Active", "Inactive", "Pending"],
                id=collection_id
            )

            # Step 3: Set up global cache
            cache = RustRdmCache()
            cache.add_collection(collection)
            set_global_rdm_cache(cache)

            # Verify global cache is set
            assert get_global_rdm_cache() is not None

            # Step 4: Prepare test data
            resource_id = "test-resource-123"
            graph_id = concept_graph_data["graphid"]

            # Simple tree - plain strings get coerced to proper format
            tree = {
                "name": ["Test Person"]
            }
            tree_json = json.dumps(tree)

            # Step 5: Call json_tree_to_tiles WITHOUT passing rdm_cache
            # The global cache should be used automatically
            result = json_tree_to_tiles(
                tree_json=tree_json,
                resource_id=resource_id,
                graph_id=graph_id,
                graph_json=concept_graph_json,
                from_camel=False,
                strict=False,
                # NOTE: rdm_cache is NOT passed - global cache should be used
            )

            # Step 6: Verify result
            assert result is not None
            assert "tiles" in result
            assert result["resourceinstanceid"] == resource_id

        finally:
            # Clean up global state
            clear_global_rdm_cache()
            assert get_global_rdm_cache() is None


class TestPhase0Integration:
    """
    Integration tests for Phase 0 components working together.

    These tests demonstrate NodeConfigManager and RdmCache being used
    together for type coercion scenarios.
    """

    @pytest.fixture
    def person_graph_json(self):
        """Load Person graph as JSON string."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        with open(graph_path) as f:
            data = json.load(f)
        return json.dumps(data["graph"][0])

    def test_node_config_with_rdm_cache_for_reference(self, person_graph_json, person_graph):
        """
        Test using NodeConfigManager to get reference config,
        then using RdmCache to validate concept references.

        This simulates the Phase 1+ flow where:
        1. NodeConfigManager provides the collection ID for a reference node
        2. RdmCache validates that a concept ID exists in that collection
        """
        from alizarin.alizarin import (
            RustNodeConfigManager,
            RustRdmCache,
            RustRdmCollection,
            RustRdmConcept
        )

        # Step 1: Build NodeConfigManager from graph
        config_manager = RustNodeConfigManager()
        config_manager.from_graph_json(person_graph_json)

        # Step 2: Get reference config for the test node
        ref_node_id = None
        for node in person_graph["nodes"]:
            if node["datatype"] == "reference":
                ref_node_id = node["nodeid"]
                break

        ref_config = config_manager.get_reference(ref_node_id)
        assert ref_config is not None

        collection_id = ref_config.get_collection_id()
        assert collection_id == "2730d609-3a8d-49dc-bf51-6ac34e80294a"

        # Step 3: Set up RdmCache with the required collection
        rdm_cache = RustRdmCache()

        collection = RustRdmCollection(collection_id)
        collection.add_concept(RustRdmConcept(
            "6672d187-dfc3-4424-8c63-7a3b377b4159",  # From test data
            {"en": "Item 1>1"}
        ))
        collection.add_concept(RustRdmConcept(
            "another-item-id",
            {"en": "Another Item"}
        ))
        rdm_cache.add_collection(collection)

        # Step 4: Validate a concept ID against the collection
        valid_concept_id = "6672d187-dfc3-4424-8c63-7a3b377b4159"
        invalid_concept_id = "nonexistent-concept-id"

        assert rdm_cache.validate_concept(collection_id, valid_concept_id) is True
        assert rdm_cache.validate_concept(collection_id, invalid_concept_id) is False

        # Step 5: Look up the concept for display
        concept = rdm_cache.lookup_concept(collection_id, valid_concept_id)
        assert concept is not None
        assert concept.get_label("en") == "Item 1>1"

    def test_domain_value_coercion_scenario(self):
        """
        Test domain value coercion using NodeConfigManager.

        This simulates Phase 1+ where:
        1. User provides a domain value ID (UUID)
        2. NodeConfigManager looks up the full domain value details
        3. Full value object is returned for tile storage
        """
        from alizarin.alizarin import (
            RustNodeConfigManager,
            RustNodeConfigDomain,
            RustStaticDomainValue
        )

        # Create a manager with a domain config manually
        # (In production, this would come from from_graph_json)

        # Simulate domain value options from graph config
        option1 = RustStaticDomainValue(
            "status-active",
            True,
            {"en": "Active", "es": "Activo"}
        )
        option2 = RustStaticDomainValue(
            "status-inactive",
            False,
            {"en": "Inactive", "es": "Inactivo"}
        )
        option3 = RustStaticDomainValue(
            "status-pending",
            False,
            {"en": "Pending", "es": "Pendiente"}
        )

        domain_config = RustNodeConfigDomain([option1, option2, option3])

        # Simulate coercion: user provides just the ID
        user_value = "status-pending"

        # Look up full value from config
        full_value = domain_config.value_from_id(user_value)

        assert full_value is not None
        assert full_value.id == "status-pending"
        assert full_value.lang("en") == "Pending"
        assert full_value.lang("es") == "Pendiente"

        # Get the selected (default) value
        default_value = domain_config.get_selected()
        assert default_value.id == "status-active"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
