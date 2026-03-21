"""
Tests for CLM Reference datatype.

Tests loading JSON strings like {"Test": ["Item 1>1"]} into tiles.
"""

import pytest
import json
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


class TestReferenceDisplaySerialization:
    """Tests for display serialization of reference types, especially multiValue."""

    def test_static_reference_to_display_string_single(self):
        """StaticReference.to_display_string should return label for single reference."""
        from alizarin_clm import StaticReference, StaticReferenceLabel

        ref = StaticReference(
            list_id="list-123",
            uri="http://example.com/item/1",
            labels=[
                StaticReferenceLabel(
                    id="label-1",
                    language_id="en",
                    list_item_id="item-1",
                    value="Hotel/Inn",
                    valuetype_id="prefLabel",
                ),
            ],
        )

        assert ref.to_display_string("en") == "Hotel/Inn"

    def test_static_reference_multiple_labels_joined(self):
        """Multiple StaticReferences should be joinable for display."""
        from alizarin_clm import StaticReference, StaticReferenceLabel

        ref1 = StaticReference(
            list_id="list-123",
            uri="http://example.com/item/1",
            labels=[
                StaticReferenceLabel(
                    id="label-1",
                    language_id="en",
                    list_item_id="item-1",
                    value="Hotel/Inn",
                    valuetype_id="prefLabel",
                ),
            ],
        )

        ref2 = StaticReference(
            list_id="list-123",
            uri="http://example.com/item/2",
            labels=[
                StaticReferenceLabel(
                    id="label-2",
                    language_id="en",
                    list_item_id="item-2",
                    value="Picture theatre/Cinema",
                    valuetype_id="prefLabel",
                ),
            ],
        )

        # For multiValue display, references would be joined with ", "
        display_strings = [ref1.to_display_string("en"), ref2.to_display_string("en")]
        joined = ", ".join(display_strings)

        assert joined == "Hotel/Inn, Picture theatre/Cinema"

    @pytest.mark.asyncio
    async def test_reference_list_viewmodel_multivalue_preserves_array(self):
        """ReferenceListViewModel should preserve array structure for multiValue nodes."""
        from alizarin_clm import ReferenceListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="test-node-id",
            name="Test Reference",
            datatype="reference",
            nodegroup_id="test-nodegroup-id",
            alias="test_ref",
            graph_id="test-graph-id",
            is_collector=True,
            isrequired=False,
            exportable=False,
            config={
                "controlledList": "test-list-id",
                "multiValue": True,
            },
        )

        tile = StaticTile(
            tileid="test-tile-id",
            nodegroup_id="test-nodegroup-id",
            resourceinstance_id="test-resource-id",
            data={},
        )

        # Array of reference values (multiValue=true)
        values = [
            {
                "labels": [
                    {
                        "id": "label-1",
                        "language_id": "en",
                        "list_item_id": "item-1",
                        "value": "Hotel/Inn",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "test-list-id",
                "uri": "http://example.com/item/1",
            },
            {
                "labels": [
                    {
                        "id": "label-2",
                        "language_id": "en",
                        "list_item_id": "item-2",
                        "value": "Picture theatre/Cinema",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "test-list-id",
                "uri": "http://example.com/item/2",
            },
        ]

        vm = await ReferenceListViewModel._create(tile, node, values)

        # Should preserve as list, not flatten to string
        assert vm is not None
        assert len(vm) == 2

        # Tile data should have been set as array
        assert node.nodeid in tile.data
        tile_value = tile.data[node.nodeid]
        assert isinstance(tile_value, list), f"Expected list, got {type(tile_value)}"
        assert len(tile_value) == 2

    @pytest.mark.asyncio
    async def test_multivalue_reference_deserialization_from_tile_data(self):
        """
        Test that multiValue reference can be deserialized from existing tile data.

        This tests the round-trip: tile_data -> ViewModel -> tile_data
        and ensures array format is preserved (not flattened to string).
        """
        from alizarin_clm import ReferenceMergedDataType, ReferenceListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="test-node-id",
            name="Monument Type",
            datatype="reference",
            nodegroup_id="test-nodegroup-id",
            alias="monument_type",
            graph_id="test-graph-id",
            is_collector=False,
            isrequired=False,
            exportable=True,
            config={
                "controlledList": "test-list-id",
                "multiValue": True,  # Key: this is a multiValue field
            },
        )

        # Simulate existing tile data with array of references
        existing_tile_data = [
            {
                "labels": [
                    {
                        "id": "label-1",
                        "language_id": "en",
                        "list_item_id": "item-1",
                        "value": "Hotel/Inn",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "test-list-id",
                "uri": "http://example.com/item/1",
            },
        ]

        tile = StaticTile(
            tileid="test-tile-id",
            nodegroup_id="test-nodegroup-id",
            resourceinstance_id="test-resource-id",
            data={node.nodeid: existing_tile_data},  # Pre-populate with array
        )

        # Deserialize using ReferenceMergedDataType (checks multiValue config)
        vm = await ReferenceMergedDataType._create(tile, node, existing_tile_data)

        # Should create ReferenceListViewModel for multiValue=True
        assert isinstance(vm, ReferenceListViewModel), \
            f"Expected ReferenceListViewModel for multiValue=True, got {type(vm).__name__}"

        # Should have one item
        assert len(vm) == 1

        # Get the first reference and check its value
        first_ref = vm[0]
        if hasattr(first_ref, '__await__'):
            first_ref = await first_ref
        assert str(first_ref) == "Hotel/Inn"

    @pytest.mark.asyncio
    async def test_multivalue_reference_rejects_string_input(self):
        """
        Test that multiValue reference rejects string input (must be array).

        This verifies the error case where flattened display output
        was incorrectly used as tile data input.
        """
        from alizarin_clm import ReferenceMergedDataType
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="test-node-id",
            name="Monument Type",
            datatype="reference",
            nodegroup_id="test-nodegroup-id",
            alias="monument_type",
            graph_id="test-graph-id",
            is_collector=False,
            isrequired=False,
            exportable=True,
            config={
                "controlledList": "test-list-id",
                "multiValue": True,
            },
        )

        tile = StaticTile(
            tileid="test-tile-id",
            nodegroup_id="test-nodegroup-id",
            resourceinstance_id="test-resource-id",
            data={},
        )

        # Incorrectly passing a string (as would happen with flattened display output)
        string_value = "Hotel/Inn"

        # Should raise an error because multiValue expects an array
        with pytest.raises(Exception) as exc_info:
            await ReferenceMergedDataType._create(tile, node, string_value)

        # Verify the error message indicates the issue
        assert "array" in str(exc_info.value).lower() or "list" in str(exc_info.value).lower()

    @pytest.mark.asyncio
    async def test_multivalue_reference_roundtrip_preserves_structure(self):
        """
        Test complete round-trip: create VM -> get tile data -> create VM again.

        Ensures the array structure is preserved through serialization/deserialization.
        """
        from alizarin_clm import ReferenceMergedDataType, ReferenceListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="test-node-id",
            name="Monument Type",
            datatype="reference",
            nodegroup_id="test-nodegroup-id",
            alias="monument_type",
            graph_id="test-graph-id",
            is_collector=False,
            isrequired=False,
            exportable=True,
            config={
                "controlledList": "test-list-id",
                "multiValue": True,
            },
        )

        # Original array data
        original_data = [
            {
                "labels": [
                    {
                        "id": "label-1",
                        "language_id": "en",
                        "list_item_id": "item-1",
                        "value": "Hotel/Inn",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "test-list-id",
                "uri": "http://example.com/item/1",
            },
            {
                "labels": [
                    {
                        "id": "label-2",
                        "language_id": "en",
                        "list_item_id": "item-2",
                        "value": "Picture theatre/Cinema",
                        "valuetype_id": "prefLabel",
                    }
                ],
                "list_id": "test-list-id",
                "uri": "http://example.com/item/2",
            },
        ]

        # First pass: create ViewModel
        tile1 = StaticTile(
            tileid="test-tile-id",
            nodegroup_id="test-nodegroup-id",
            resourceinstance_id="test-resource-id",
            data={},
        )

        vm1 = await ReferenceMergedDataType._create(tile1, node, original_data)
        assert isinstance(vm1, ReferenceListViewModel)
        assert len(vm1) == 2

        # Get tile data from first VM
        tile_data_after_first = tile1.data.get(node.nodeid)
        assert tile_data_after_first is not None
        assert isinstance(tile_data_after_first, list), \
            f"Tile data should be list, got {type(tile_data_after_first)}"
        assert len(tile_data_after_first) == 2

        # Second pass: create new ViewModel from the tile data
        tile2 = StaticTile(
            tileid="test-tile-id-2",
            nodegroup_id="test-nodegroup-id",
            resourceinstance_id="test-resource-id",
            data={},
        )

        vm2 = await ReferenceMergedDataType._create(tile2, node, tile_data_after_first)
        assert isinstance(vm2, ReferenceListViewModel)
        assert len(vm2) == 2

        # Verify values match
        first_ref = vm2[0]
        if hasattr(first_ref, '__await__'):
            first_ref = await first_ref
        second_ref = vm2[1]
        if hasattr(second_ref, '__await__'):
            second_ref = await second_ref

        assert str(first_ref) == "Hotel/Inn"
        assert str(second_ref) == "Picture theatre/Cinema"


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
        Test full Rust flow: JSON tree with label strings → resolved reference tiles.

        Demonstrates the real user flow: submit human-readable labels, get back
        resolved StaticReference objects in tile data.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler
        from alizarin import (
            RustRdmCollection, RustRdmConcept, RustRdmCache,
            set_global_rdm_cache, clear_global_rdm_cache,
        )

        if alizarin.json_tree_to_tiles is None:
            pytest.skip("json_tree_to_tiles not available")

        # Set up RDM cache with a collection matching the graph config.
        # Concept ID must be UUID-formatted (as in production) so the handler
        # routes it to __needs_rdm_lookup (by UUID) after core_resolve_labels
        # has already resolved the label to a concept ID.
        collection = RustRdmCollection("2730d609-3a8d-49dc-bf51-6ac34e80294a")
        collection.add_concept(RustRdmConcept("a1b2c3d4-e5f6-7890-abcd-ef1234567890", {"en": "Test Reference Value"}))
        cache = RustRdmCache()
        cache.add_collection(collection)
        set_global_rdm_cache(cache)

        try:
            # Tree input uses plain label strings — the user-facing format
            person_tree = {
                "test": ["Test Reference Value"],
                "name": [{
                    "full_name": {"_value": {"en": "John Doe"}}
                }]
            }

            tree_json = json.dumps(person_tree)
            resource_id = "test-person-001"

            graph_id = alizarin.register_graph(person_graph_json)

            result = alizarin.json_tree_to_tiles(
                tree_json=tree_json,
                resource_id=resource_id,
                graph_id=graph_id,
            )

            assert 'business_data' in result
            resources = result['business_data']['resources']
            assert len(resources) == 1

            resource = resources[0]
            assert resource["resourceinstance"]["resourceinstanceid"] == resource_id
            assert resource["resourceinstance"]["graph_id"] == graph_id
            assert "tiles" in resource
            assert len(resource["tiles"]) > 0

            # Find the tile with reference data
            reference_node_id = None
            for node in person_graph_data["nodes"]:
                if node.get("alias") == "test" and node.get("datatype") == "reference":
                    reference_node_id = node["nodeid"]
                    break

            assert reference_node_id is not None, "Should find reference node"

            reference_tile = None
            for tile in resource["tiles"]:
                if reference_node_id in tile.get("data", {}):
                    reference_tile = tile
                    break

            assert reference_tile is not None, "Should have tile with reference data"

            # The label should have been resolved to a full StaticReference
            ref_data = reference_tile["data"][reference_node_id]
            assert isinstance(ref_data, list), f"multiValue reference should be array, got {type(ref_data)}"
            assert len(ref_data) == 1
            assert "labels" in ref_data[0], f"Should be resolved StaticReference: {ref_data[0]}"
            assert ref_data[0]["labels"][0]["value"] == "Test Reference Value"
        finally:
            clear_global_rdm_cache()

        # The reference data should be stored in the tile
        ref_data = reference_tile["data"][reference_node_id]
        assert ref_data is not None

        print(f"Created {len(resource['tiles'])} tiles")
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

        # Step 1: Set up RDM cache with labels matching the tree input
        from alizarin import (
            RustRdmCollection, RustRdmConcept, RustRdmCache,
            set_global_rdm_cache, clear_global_rdm_cache,
        )

        collection = RustRdmCollection("2730d609-3a8d-49dc-bf51-6ac34e80294a")
        collection.add_concept(RustRdmConcept("b1c2d3e4-f5a6-7890-abcd-ef1234567890", {"en": "Category A", "es": "Categoría A"}))
        collection.add_concept(RustRdmConcept("c2d3e4f5-a6b7-8901-bcde-f12345678901", {"en": "Category B"}))
        cache = RustRdmCache()
        cache.add_collection(collection)
        set_global_rdm_cache(cache)

        try:
            # Step 2: Create tree with label strings — the user-facing format
            person_tree = {
                "test": ["Category A", "Category B"],
                "name": [{
                    "full_name": {"_value": {"en": "Jane Smith", "ga": "Síle Ní Ghobnait"}}
                }]
            }

            tree_json = json.dumps(person_tree)
            graph_id = alizarin.register_graph(person_graph_json)
            result = alizarin.json_tree_to_tiles(
                tree_json=tree_json,
                resource_id="test-person-002",
                graph_id=graph_id,
            )

            assert 'business_data' in result
            resources = result['business_data']['resources']
            assert len(resources) == 1
            resource = resources[0]
            assert len(resource["tiles"]) > 0

            # Step 3: Find resolved reference data in tiles
            reference_node_id = None
            for node in person_graph_data["nodes"]:
                if node.get("alias") == "test" and node.get("datatype") == "reference":
                    reference_node_id = node["nodeid"]
                    break

            ref_data = None
            for tile in resource["tiles"]:
                if reference_node_id and reference_node_id in tile.get("data", {}):
                    ref_data = tile["data"][reference_node_id]
                    if ref_data is not None:
                        break

            assert ref_data is not None, "Should have a tile with reference data"
            assert isinstance(ref_data, list), f"Reference data should be a list, got {type(ref_data)}"
            assert len(ref_data) == 2, f"Should have 2 reference items, got {len(ref_data)}"

            # Step 4: Verify labels were resolved to full StaticReference objects
            label_values = {r["labels"][0]["value"] for r in ref_data}
            assert "Category A" in label_values, "Should have Category A reference"
            assert "Category B" in label_values, "Should have Category B reference"

            # Verify multilingual labels preserved
            cat_a_ref = next(r for r in ref_data if r["labels"][0]["value"] == "Category A")
            assert any(l["language_id"] == "es" for l in cat_a_ref["labels"]), \
                "Category A should have Spanish label from RDM"
        finally:
            clear_global_rdm_cache()

    def test_rust_coercion_in_roundtrip(self, person_graph_json, person_graph_data):
        """
        Test that reference values survive tree → tiles → tree through Rust.

        Input is a label string resolved via RDM cache. After tiles → tree,
        the tree contains resolved StaticReference objects — these are NOT
        valid for re-ingestion (tree → tiles rejects pre-formed objects).
        This test verifies each direction works correctly.
        """
        import alizarin
        import alizarin_clm  # noqa: F401
        from alizarin import (
            RustRdmCollection, RustRdmConcept, RustRdmCache,
            set_global_rdm_cache, clear_global_rdm_cache,
        )

        if alizarin.json_tree_to_tiles is None or alizarin.tiles_to_json_tree is None:
            pytest.skip("Rust conversion functions not available")

        collection = RustRdmCollection("2730d609-3a8d-49dc-bf51-6ac34e80294a")
        collection.add_concept(RustRdmConcept(
            "d4e5f6a7-b8c9-0123-def0-456789abcdef",
            {"en": "Roundtrip Test"},
        ))
        cache = RustRdmCache()
        cache.add_collection(collection)
        set_global_rdm_cache(cache)

        try:
            original_tree = {"test": ["Roundtrip Test"]}
            resource_id = "roundtrip-test-001"
            graph_id = alizarin.register_graph(person_graph_json)

            # Step 1: tree → tiles — label gets resolved to StaticReference
            tiles_result = alizarin.json_tree_to_tiles(
                tree_json=json.dumps(original_tree),
                resource_id=resource_id,
                graph_id=graph_id,
            )

            assert 'business_data' in tiles_result
            tiles_resource = tiles_result['business_data']['resources'][0]
            assert len(tiles_resource["tiles"]) > 0

            # Find reference data in tiles
            ref_node_id = None
            for node in person_graph_data["nodes"]:
                if node.get("alias") == "test":
                    ref_node_id = node["nodeid"]
                    break

            ref_data = None
            for tile in tiles_resource["tiles"]:
                if ref_node_id and ref_node_id in tile.get("data", {}):
                    ref_data = tile["data"][ref_node_id]
                    break

            assert ref_data is not None, "Should have resolved reference data"
            assert isinstance(ref_data, list)
            assert "labels" in ref_data[0], "Should be resolved StaticReference"

            # Step 2: tiles → tree — resolved data goes back to tree format
            tree_result = alizarin.tiles_to_json_tree(json.dumps({
                "graph_id": graph_id,
                "resourceinstanceid": resource_id,
                "tiles": tiles_resource["tiles"],
            }))

            assert len(tree_result) > 0
        finally:
            clear_global_rdm_cache()


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
        Test coerce_with_extension with a single label string.

        Label strings produce __needs_rdm_label_lookup markers.
        """
        import alizarin_clm  # noqa: F401
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        value_json = json.dumps("Item 1>1")
        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": False,
        })

        tile_data_json, resolved_json = alizarin.coerce_with_extension(
            "reference", value_json, config_json,
        )

        tile_data = json.loads(tile_data_json)
        assert tile_data.get("__needs_rdm_label_lookup") is True
        assert tile_data["label"] == "Item 1>1"
        assert tile_data["controlledList"] == "2730d609-3a8d-49dc-bf51-6ac34e80294a"

    def test_coerce_with_extension_single_uuid(self):
        """
        Test coerce_with_extension with a single UUID string.

        UUID strings produce __needs_rdm_lookup markers.
        """
        import alizarin_clm  # noqa: F401
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        uuid_val = "6672d187-dfc3-4424-8c63-7a3b377b4159"
        value_json = json.dumps(uuid_val)
        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": False,
        })

        tile_data_json, _ = alizarin.coerce_with_extension(
            "reference", value_json, config_json,
        )

        tile_data = json.loads(tile_data_json)
        assert tile_data.get("__needs_rdm_lookup") is True
        assert tile_data["uuid"] == uuid_val

    def test_coerce_with_extension_reference_array(self):
        """
        Test coerce_with_extension with an array of label strings.

        Each label produces a __needs_rdm_label_lookup marker.
        """
        import alizarin_clm  # noqa: F401
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        value_json = json.dumps(["First Item", "Second Item"])
        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": True,
        })

        tile_data_json, _ = alizarin.coerce_with_extension(
            "reference", value_json, config_json,
        )

        tile_data = json.loads(tile_data_json)
        assert isinstance(tile_data, list)
        assert len(tile_data) == 2
        assert tile_data[0]["label"] == "First Item"
        assert tile_data[1]["label"] == "Second Item"
        for item in tile_data:
            assert item.get("__needs_rdm_label_lookup") is True

    def test_coerce_rejects_preformed_reference_object(self):
        """
        Test that coerce_with_extension rejects pre-formed StaticReference objects.

        Pre-formed objects (with labels/list_id keys) are not valid coercion input.
        """
        import alizarin_clm  # noqa: F401
        import alizarin

        if alizarin.coerce_with_extension is None:
            pytest.skip("Rust extension not available")

        value_json = json.dumps({
            "labels": [
                {
                    "id": "en-label",
                    "language_id": "en",
                    "list_item_id": "item-1",
                    "value": "English Label",
                    "valuetype_id": "prefLabel",
                },
            ],
            "list_id": "list-123",
            "uri": "http://example.com/item/1",
        })

        config_json = json.dumps({"multiValue": False})

        with pytest.raises(ValueError, match="Pre-formed reference objects are not valid input"):
            alizarin.coerce_with_extension("reference", value_json, config_json)

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
        manager.load_graph_json(person_graph_json)

        # Person graph has 1 reference node
        assert len(manager) >= 1

    def test_node_config_manager_get_reference_config(self, person_graph_json, person_graph):
        """Test retrieving reference config for a node."""
        from alizarin.alizarin import RustNodeConfigManager

        manager = RustNodeConfigManager()
        manager.load_graph_json(person_graph_json)

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
        collection.add_from_label("Category A")
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
        from alizarin.alizarin import resolve_labels_in_tree, register_graph

        cache, collection = test_collection

        # JSON tree with a value that won't be resolved (reference nodes, not concept)
        tree = {
            "test": [{"_value": "Some Value"}]
        }
        tree_json = json.dumps(tree)

        # Register graph before resolution
        graph_id = register_graph(concept_graph_json)

        # Should not raise - passes through since 'test' is a reference node, not concept
        resolved_json = resolve_labels_in_tree(
            tree_json=tree_json,
            graph_id=graph_id,
            rdm_cache=cache,
            strict=False
        )

        resolved = json.loads(resolved_json)
        # Value should pass through unchanged (no concept node to resolve)
        assert resolved["test"][0]["_value"] == "Some Value"

    def test_uuid_passthrough_with_real_graph(self, concept_graph_json, test_collection):
        """Test that UUID values pass through unchanged regardless of node type."""
        from alizarin.alizarin import resolve_labels_in_tree, register_graph

        cache, collection = test_collection

        # Get an actual UUID from the collection
        concept = collection.find_by_label("Category A")
        uuid_value = concept.id

        # JSON tree with UUID
        tree = {
            "test": [{"_value": uuid_value}]
        }
        tree_json = json.dumps(tree)

        # Register graph before resolution
        graph_id = register_graph(concept_graph_json)

        # UUIDs should always pass through unchanged
        resolved_json = resolve_labels_in_tree(
            tree_json=tree_json,
            graph_id=graph_id,
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
            register_graph,
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

            # Simple tree - plain strings get coerced to proper format
            tree = {
                "name": ["Test Person"]
            }
            tree_json = json.dumps(tree)

            # Step 5: Register graph and call json_tree_to_tiles WITHOUT passing rdm_cache
            # The global cache should be used automatically
            graph_id = register_graph(concept_graph_json)
            result = json_tree_to_tiles(
                tree_json=tree_json,
                resource_id=resource_id,
                graph_id=graph_id,
                from_camel=False,
                strict=False,
                # NOTE: rdm_cache is NOT passed - global cache should be used
            )

            # Step 6: Verify result - BusinessDataWrapper format
            assert result is not None
            assert 'business_data' in result
            resources = result['business_data']['resources']
            assert len(resources) == 1
            assert resources[0]["resourceinstance"]["resourceinstanceid"] == resource_id
            assert "tiles" in resources[0]

        finally:
            # Clean up global state
            clear_global_rdm_cache()
            assert get_global_rdm_cache() is None


class TestCLMReferenceResolution:
    """
    Integration tests for CLM's resolve_reference_labels function.

    These tests demonstrate the CLM extension handling its own label resolution
    for 'reference' nodes with 'controlledList' config, independent of core.
    """

    @pytest.fixture
    def reference_graph_json(self):
        """Load Person graph which has reference nodes."""
        graph_path = TEST_DATA_DIR / "graphs" / "resource_models" / "Person.json"
        with open(graph_path) as f:
            data = json.load(f)
        return json.dumps(data)

    @pytest.fixture
    def reference_collection(self):
        """Create a test collection matching the Person graph's controlledList."""
        from alizarin.alizarin import RustRdmCollection, RustRdmCache, set_current_language

        set_current_language("en")

        # Person graph uses this collection ID for its 'test' reference node
        collection_id = "2730d609-3a8d-49dc-bf51-6ac34e80294a"

        collection = RustRdmCollection.from_labels(
            "Test Categories",
            ["Category A", "Category B", "Category C"],
            id=collection_id
        )

        cache = RustRdmCache()
        cache.add_collection(collection)

        return cache, collection

    @pytest.mark.asyncio
    async def test_resolve_reference_labels_with_explicit_cache(
        self, reference_graph_json, reference_collection
    ):
        """
        Test CLM's resolve_reference_labels with an explicit cache parameter.

        Demonstrates the extension handling its own label resolution.
        """
        from alizarin_clm import resolve_reference_labels

        cache, collection = reference_collection

        # Tree with label string for reference field
        tree = {"test": ["Category A"]}
        tree_json = json.dumps(tree)

        # Resolve using explicit cache
        resolved_json = await resolve_reference_labels(
            tree_json=tree_json,
            graph_json=reference_graph_json,
            rdm_cache=cache,
            strict=False,
        )

        resolved = json.loads(resolved_json)

        # Label should be resolved to UUID
        concept_a = collection.find_by_label("Category A")
        assert resolved["test"][0] == concept_a.id

    @pytest.mark.asyncio
    async def test_resolve_reference_labels_with_global_cache(
        self, reference_graph_json, reference_collection
    ):
        """
        Test CLM's resolve_reference_labels using the global RDM cache.

        Demonstrates the extension using the global singleton.
        """
        from alizarin.alizarin import (
            set_global_rdm_cache,
            clear_global_rdm_cache,
        )
        from alizarin_clm import resolve_reference_labels

        cache, collection = reference_collection

        try:
            # Set up global cache
            set_global_rdm_cache(cache)

            # Tree with label strings
            tree = {"test": ["Category B", "Category C"]}
            tree_json = json.dumps(tree)

            # Resolve WITHOUT passing cache - uses global
            resolved_json = await resolve_reference_labels(
                tree_json=tree_json,
                graph_json=reference_graph_json,
                # Note: no rdm_cache parameter
            )

            resolved = json.loads(resolved_json)

            # Labels should be resolved to UUIDs
            concept_b = collection.find_by_label("Category B")
            concept_c = collection.find_by_label("Category C")
            assert resolved["test"][0] == concept_b.id
            assert resolved["test"][1] == concept_c.id

        finally:
            clear_global_rdm_cache()

    @pytest.mark.asyncio
    async def test_resolve_reference_labels_strict_mode(
        self, reference_graph_json, reference_collection
    ):
        """
        Test strict mode raises error for unknown labels.
        """
        from alizarin_clm import resolve_reference_labels

        cache, _ = reference_collection

        # Tree with unknown label
        tree = {"test": ["Unknown Category"]}
        tree_json = json.dumps(tree)

        with pytest.raises(ValueError) as exc_info:
            await resolve_reference_labels(
                tree_json=tree_json,
                graph_json=reference_graph_json,
                rdm_cache=cache,
                strict=True,
            )

        assert "Unknown Category" in str(exc_info.value)
        assert "not found" in str(exc_info.value)

    @pytest.mark.asyncio
    async def test_resolve_reference_labels_uuid_passthrough(
        self, reference_graph_json, reference_collection
    ):
        """
        Test that existing UUIDs pass through unchanged.
        """
        from alizarin_clm import resolve_reference_labels

        cache, collection = reference_collection
        concept_a = collection.find_by_label("Category A")

        # Tree with UUID (already resolved)
        tree = {"test": [concept_a.id]}
        tree_json = json.dumps(tree)

        resolved_json = await resolve_reference_labels(
            tree_json=tree_json,
            graph_json=reference_graph_json,
            rdm_cache=cache,
        )

        resolved = json.loads(resolved_json)

        # UUID should pass through unchanged
        assert resolved["test"][0] == concept_a.id

    @pytest.mark.asyncio
    async def test_clm_resolution_then_core_conversion(
        self, reference_graph_json, reference_collection
    ):
        """
        End-to-end test: CLM resolves labels, then core converts to tiles.

        This demonstrates the extension pre-processing pattern:
        1. CLM resolves reference labels
        2. Core json_tree_to_tiles handles the rest
        """
        from alizarin.alizarin import json_tree_to_tiles, register_graph
        from alizarin_clm import resolve_reference_labels

        cache, collection = reference_collection

        # Tree with plain strings
        tree = {
            "name": ["Test Person"],
            "test": ["Category A"],
        }
        tree_json = json.dumps(tree)

        # Step 1: CLM resolves reference labels
        resolved_json = await resolve_reference_labels(
            tree_json=tree_json,
            graph_json=reference_graph_json,
            rdm_cache=cache,
        )

        # Verify resolution happened
        resolved = json.loads(resolved_json)
        concept_a = collection.find_by_label("Category A")
        assert resolved["test"][0] == concept_a.id

        # Step 2: Register graph and convert to tiles
        graph_id = register_graph(reference_graph_json)

        result = json_tree_to_tiles(
            tree_json=resolved_json,
            resource_id="test-resource-123",
            graph_id=graph_id,
        )

        # Verify tiles were created - BusinessDataWrapper format
        assert result is not None
        assert 'business_data' in result
        resources = result['business_data']['resources']
        assert len(resources) == 1
        assert "tiles" in resources[0]

    @pytest.mark.asyncio
    async def test_lazy_loading_with_loader(self, reference_graph_json):
        """
        Test lazy loading: cache fetches collection on-demand via loader.

        This demonstrates the efficient pattern where only needed collections
        are loaded from the API.
        """
        from alizarin.alizarin import RustRdmCache, RustRdmCollection, set_current_language
        from alizarin_clm import resolve_reference_labels

        set_current_language("en")

        # Track which collections were requested
        loaded_collections: list[str] = []

        # Async loader that creates collection on demand
        async def my_loader(collection_id: str):
            loaded_collections.append(collection_id)

            # Simulate API response - in real usage this would be an HTTP call
            if collection_id == "2730d609-3a8d-49dc-bf51-6ac34e80294a":
                return RustRdmCollection.from_labels(
                    "Test Categories",
                    ["Category A", "Category B", "Category C"],
                    id=collection_id
                )
            return None

        # Create cache with loader - no collections pre-loaded
        cache = RustRdmCache(loader=my_loader)
        assert len(cache) == 0  # Empty!

        # Tree that references the collection
        tree = {"test": ["Category A"]}
        tree_json = json.dumps(tree)

        # Resolve - this should trigger lazy loading
        resolved_json = await resolve_reference_labels(
            tree_json=tree_json,
            graph_json=reference_graph_json,
            rdm_cache=cache,
        )

        # Verify loader was called
        assert "2730d609-3a8d-49dc-bf51-6ac34e80294a" in loaded_collections

        # Verify collection is now cached
        assert len(cache) == 1

        # Verify resolution worked
        resolved = json.loads(resolved_json)
        assert resolved["test"][0] != "Category A"  # Should be UUID now

        # Second call should NOT trigger loader again
        loaded_collections.clear()
        await resolve_reference_labels(
            tree_json=tree_json,
            graph_json=reference_graph_json,
            rdm_cache=cache,
        )
        assert len(loaded_collections) == 0  # No new loads


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
        config_manager.load_graph_json(person_graph_json)

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


class TestBatchTreesToTilesWithMultiValueReference:
    """Tests for batch_trees_to_tiles with multiValue reference types."""

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

    def test_batch_trees_to_tiles_with_reference_array(self, person_graph_json, person_graph_data):
        """
        Test batch_trees_to_tiles with label strings for a multiValue reference node.

        Labels are resolved via RDM cache to full StaticReference objects.
        batch_trees_to_tiles does NOT call core_resolve_labels, so the CLM handler's
        __needs_rdm_label_lookup markers are the only resolution path.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler
        from alizarin import (
            RustRdmCollection, RustRdmConcept, RustRdmCache,
            set_global_rdm_cache, clear_global_rdm_cache,
        )

        if alizarin.batch_trees_to_tiles is None:
            pytest.skip("batch_trees_to_tiles not available")
        if alizarin.register_graph is None:
            pytest.skip("register_graph not available")

        graph_id = alizarin.register_graph(person_graph_json)

        # Set up RDM cache for label resolution
        collection = RustRdmCollection("2730d609-3a8d-49dc-bf51-6ac34e80294a")
        collection.add_concept(RustRdmConcept(
            "e1f2a3b4-c5d6-7890-abcd-ef1234567890", {"en": "Hotel/Inn"}
        ))
        collection.add_concept(RustRdmConcept(
            "f2a3b4c5-d6e7-8901-bcde-f12345678901", {"en": "Picture theatre/Cinema"}
        ))
        cache = RustRdmCache()
        cache.add_collection(collection)
        set_global_rdm_cache(cache)

        try:
            tree = {
                "graph_id": graph_id,
                "test": ["Hotel/Inn", "Picture theatre/Cinema"],
            }

            result = alizarin.batch_trees_to_tiles(
                trees_json=json.dumps([tree]),
                graph_id=graph_id,
                random_ids=True,
            )

            assert "business_data" in result
            assert len(result["business_data"]["resources"]) == 1

            resource = result["business_data"]["resources"][0]

            ref_node_id = None
            for node in person_graph_data["nodes"]:
                if node.get("alias") == "test" and node.get("datatype") == "reference":
                    ref_node_id = node["nodeid"]
                    break

            assert ref_node_id is not None

            # Find tile with resolved reference data
            ref_data = None
            for tile in resource["tiles"]:
                if ref_node_id in tile.get("data", {}):
                    ref_data = tile["data"][ref_node_id]
                    if ref_data is not None:
                        break

            assert ref_data is not None, "Should have reference data in tiles"
            assert isinstance(ref_data, list), f"multiValue should be array, got {type(ref_data)}"
            assert len(ref_data) == 2

            # Labels should be resolved to StaticReference objects
            label_values = {r["labels"][0]["value"] for r in ref_data}
            assert "Hotel/Inn" in label_values
            assert "Picture theatre/Cinema" in label_values
        finally:
            clear_global_rdm_cache()

    def test_batch_trees_to_tiles_strict_raises_on_invalid_reference(
        self, person_graph_json, person_graph_data
    ):
        """
        Test that batch_trees_to_tiles in strict mode raises an error
        when given an invalid reference value (non-UUID string).

        This verifies that coercion errors are properly propagated.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.batch_trees_to_tiles is None:
            pytest.skip("batch_trees_to_tiles not available")
        if alizarin.register_graph is None:
            pytest.skip("register_graph not available")

        # Register the graph (returns graph_id)
        graph_id = alizarin.register_graph(person_graph_json)

        # A plain string like "Hotel/Inn" is actually VALID — it's a label
        # that will be marked for RDM lookup via __needs_rdm_label_lookup.
        # To test strict-mode rejection, use a genuinely invalid value like a number.
        tree = {
            "graph_id": graph_id,
            "test": [42],  # Invalid: numbers are not valid reference input
        }

        trees_json = json.dumps([tree])

        # In strict mode, should raise an error for invalid reference input
        with pytest.raises(ValueError, match="Extension coercion failed"):
            alizarin.batch_trees_to_tiles(
                trees_json=trees_json,
                graph_id=graph_id,
                strict=True,
                random_ids=True,
            )

    def test_batch_trees_to_tiles_marks_label_for_lookup(
        self, person_graph_json, person_graph_data
    ):
        """
        Test that batch_trees_to_tiles marks plain strings
        for RDM label lookup.

        Plain strings like "Hotel/Inn" should be accepted and marked for lookup
        so they can be resolved against the collection.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.batch_trees_to_tiles is None:
            pytest.skip("batch_trees_to_tiles not available")
        if alizarin.register_graph is None:
            pytest.skip("register_graph not available")

        # Register the graph (returns graph_id)
        graph_id = alizarin.register_graph(person_graph_json)

        # Create a tree with a plain string value (will be marked for label lookup)
        tree = {
            "graph_id": graph_id,
            "test": ["Hotel/Inn"],  # Plain string - should be marked for label lookup
        }

        trees_json = json.dumps([tree])

        # Should succeed - plain strings are accepted and marked for lookup
        result = alizarin.batch_trees_to_tiles(
            trees_json=trees_json,
            graph_id=graph_id,
            strict=True,
            random_ids=True,
        )

        # Verify success
        assert "business_data" in result
        assert result.get("error_count", 0) == 0, f"Should have no errors, got: {result.get('errors')}"
        assert len(result["business_data"]["resources"]) == 1

        # Find the reference data and verify it has the label lookup marker
        resource = result["business_data"]["resources"][0]
        ref_node_id = None
        for node in person_graph_data["nodes"]:
            if node.get("alias") == "test" and node.get("datatype") == "reference":
                ref_node_id = node["nodeid"]
                break

        # Find tile with reference data
        ref_data = None
        for tile in resource["tiles"]:
            if ref_node_id and ref_node_id in tile.get("data", {}):
                ref_data = tile["data"][ref_node_id]
                break

        assert ref_data is not None, "Should have reference data in tiles"
        # For multiValue nodes, ref_data is an array
        if isinstance(ref_data, list):
            assert len(ref_data) == 1, f"Should have 1 item, got {len(ref_data)}"
            item = ref_data[0]
        else:
            item = ref_data
        assert item.get("__needs_rdm_label_lookup") is True, \
            f"Should be marked for label lookup: {item}"
        assert item.get("label") == "Hotel/Inn"
        print(f"Plain string correctly marked for label lookup: {item}")

    def test_batch_trees_to_tiles_accepts_uuid_references(
        self, person_graph_json, person_graph_data
    ):
        """
        Test that batch_trees_to_tiles correctly processes UUID strings
        as reference input. UUIDs are marked for RDM lookup.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler
        from alizarin import (
            RustRdmCollection, RustRdmConcept, RustRdmCache,
            set_global_rdm_cache, clear_global_rdm_cache,
        )

        if alizarin.batch_trees_to_tiles is None:
            pytest.skip("batch_trees_to_tiles not available")
        if alizarin.register_graph is None:
            pytest.skip("register_graph not available")

        graph_id = alizarin.register_graph(person_graph_json)

        concept_uuid = "e1f2a3b4-c5d6-7890-abcd-ef1234567890"

        # Set up RDM cache with the concept that will be looked up by UUID
        collection = RustRdmCollection("2730d609-3a8d-49dc-bf51-6ac34e80294a")
        collection.add_concept(RustRdmConcept(concept_uuid, {"en": "Hotel/Inn"}))
        cache = RustRdmCache()
        cache.add_collection(collection)
        set_global_rdm_cache(cache)

        try:
            tree = {
                "graph_id": graph_id,
                "test": [concept_uuid],
            }

            result = alizarin.batch_trees_to_tiles(
                trees_json=json.dumps([tree]),
                graph_id=graph_id,
                strict=True,
                random_ids=True,
            )

            assert "business_data" in result
            assert result.get("error_count", 0) == 0, f"Should have no errors, got: {result.get('errors')}"
            assert len(result["business_data"]["resources"]) == 1
        finally:
            clear_global_rdm_cache()

    def test_coerce_with_extension_marks_label_string_for_lookup(self):
        """
        Test that coerce_with_extension marks non-UUID strings for label lookup.

        Plain strings like "Hotel/Inn" should be marked for RDM label lookup
        so Python can resolve them against the collection.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.coerce_with_extension is None:
            pytest.skip("coerce_with_extension not available")

        # Plain string (not a UUID) - should be marked for label lookup
        label_value = json.dumps("Hotel/Inn")

        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": True,
        })

        # Should succeed and return a marker for label lookup
        tile_data_json, resolved_json = alizarin.coerce_with_extension(
            "reference", label_value, config_json
        )

        tile_data = json.loads(tile_data_json)

        # Result should be an array (multiValue=True) with label lookup marker
        assert isinstance(tile_data, list), f"Expected array for multiValue, got {type(tile_data)}"
        assert len(tile_data) == 1

        # The item should have __needs_rdm_label_lookup marker
        item = tile_data[0]
        assert item.get("__needs_rdm_label_lookup") is True, \
            f"Label string should be marked for RDM label lookup: {item}"
        assert item.get("label") == "Hotel/Inn"
        assert item.get("controlledList") == "2730d609-3a8d-49dc-bf51-6ac34e80294a"

    def test_coerce_with_extension_accepts_uuid_array(self):
        """
        Test that coerce_with_extension accepts an array of UUIDs
        for a multiValue reference field.

        UUIDs should be marked for RDM lookup.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.coerce_with_extension is None:
            pytest.skip("coerce_with_extension not available")

        # Valid: array of UUIDs
        uuid_array = json.dumps([
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001",
        ])

        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": True,
        })

        # Should succeed - UUIDs are valid input
        tile_data_json, resolved_json = alizarin.coerce_with_extension(
            "reference", uuid_array, config_json
        )

        tile_data = json.loads(tile_data_json)

        # Result should be an array (multiValue=True)
        assert isinstance(tile_data, list), f"Expected array, got {type(tile_data)}"
        assert len(tile_data) == 2

        # Each item should have __needs_rdm_lookup marker
        for item in tile_data:
            assert item.get("__needs_rdm_lookup") is True, \
                f"UUID should be marked for RDM lookup: {item}"

    def test_coerce_with_extension_rejects_static_reference_array(self):
        """
        Test that coerce_with_extension rejects an array of pre-formed
        StaticReference objects.
        """
        import alizarin
        import alizarin_clm  # noqa: F401 - registers handler

        if alizarin.coerce_with_extension is None:
            pytest.skip("coerce_with_extension not available")

        ref_array = json.dumps([
            {
                "labels": [{"id": "l1", "language_id": "en", "list_item_id": "i1",
                            "value": "Hotel/Inn", "valuetype_id": "prefLabel"}],
                "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
                "uri": "http://example.com/item/1",
            },
        ])

        config_json = json.dumps({
            "controlledList": "2730d609-3a8d-49dc-bf51-6ac34e80294a",
            "multiValue": True,
        })

        with pytest.raises(ValueError, match="Pre-formed reference objects are not valid input"):
            alizarin.coerce_with_extension("reference", ref_array, config_json)


class TestResolveReferenceMarkers:
    """Tests for resolve_reference_markers function."""

    @pytest.fixture
    def sample_graph(self):
        """Sample graph with a reference node."""
        return {
            "graphid": "test-graph",
            "nodes": [
                {
                    "nodeid": "ref-node-1",
                    "alias": "monument_type",
                    "datatype": "reference",
                    "config": {
                        "controlledList": "monument-types-collection"
                    }
                }
            ]
        }

    @pytest.fixture
    def sample_collection(self):
        """Create a sample RDM collection with concepts."""
        from alizarin import RustRdmCollection, RustRdmConcept

        collection = RustRdmCollection("monument-types-collection")

        # Add a concept with multilingual labels
        concept = RustRdmConcept("concept-castle-uuid", {"en": "Castle", "ga": "Caisleán"})
        collection.add_concept(concept)

        return collection

    @pytest.mark.asyncio
    async def test_resolve_uuid_marker(self, sample_graph, sample_collection):
        """Should resolve __needs_rdm_lookup marker to full StaticReference."""
        import json
        from alizarin import RustRdmCache
        from alizarin_clm import resolve_reference_markers

        # Create cache with collection
        cache = RustRdmCache()
        cache.add_collection(sample_collection)

        # Business data with UUID marker
        business_data = {
            "business_data": {
                "resources": [
                    {
                        "resourceinstanceid": "res-1",
                        "tiles": [
                            {
                                "tileid": "tile-1",
                                "data": {
                                    "ref-node-1": [
                                        {
                                            "__needs_rdm_lookup": True,
                                            "uuid": "concept-castle-uuid"
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                ]
            }
        }

        result = await resolve_reference_markers(
            json.dumps(business_data),
            json.dumps(sample_graph),
            cache
        )

        result_data = json.loads(result)
        tile_data = result_data["business_data"]["resources"][0]["tiles"][0]["data"]["ref-node-1"]

        assert len(tile_data) == 1
        ref = tile_data[0]

        # Should be a full StaticReference now
        assert "labels" in ref, f"Expected StaticReference, got: {ref}"
        assert "list_id" in ref
        assert "__needs_rdm_lookup" not in ref

        # Check labels have correct content
        assert ref["list_id"] == "monument-types-collection"
        labels = ref["labels"]
        assert len(labels) >= 1

        # Find English label
        en_label = next((l for l in labels if l["language_id"] == "en"), None)
        assert en_label is not None, f"Expected English label in {labels}"
        assert en_label["value"] == "Castle"

    @pytest.mark.asyncio
    async def test_resolve_label_marker(self, sample_graph, sample_collection):
        """Should resolve __needs_rdm_label_lookup marker to full StaticReference."""
        import json
        from alizarin import RustRdmCache
        from alizarin_clm import resolve_reference_markers

        # Create cache with collection
        cache = RustRdmCache()
        cache.add_collection(sample_collection)

        # Business data with label marker
        business_data = {
            "business_data": {
                "resources": [
                    {
                        "resourceinstanceid": "res-1",
                        "tiles": [
                            {
                                "tileid": "tile-1",
                                "data": {
                                    "ref-node-1": [
                                        {
                                            "__needs_rdm_label_lookup": True,
                                            "label": "Castle",
                                            "controlledList": "monument-types-collection"
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                ]
            }
        }

        result = await resolve_reference_markers(
            json.dumps(business_data),
            json.dumps(sample_graph),
            cache
        )

        result_data = json.loads(result)
        tile_data = result_data["business_data"]["resources"][0]["tiles"][0]["data"]["ref-node-1"]

        assert len(tile_data) == 1
        ref = tile_data[0]

        # Should be a full StaticReference now
        assert "labels" in ref, f"Expected StaticReference, got: {ref}"
        assert "__needs_rdm_label_lookup" not in ref

    @pytest.mark.asyncio
    async def test_passthrough_existing_reference(self, sample_graph, sample_collection):
        """Should pass through values that are already full StaticReferences."""
        import json
        from alizarin import RustRdmCache
        from alizarin_clm import resolve_reference_markers

        cache = RustRdmCache()
        cache.add_collection(sample_collection)

        # Business data with full StaticReference (no marker)
        business_data = {
            "business_data": {
                "resources": [
                    {
                        "resourceinstanceid": "res-1",
                        "tiles": [
                            {
                                "tileid": "tile-1",
                                "data": {
                                    "ref-node-1": [
                                        {
                                            "uri": "http://example.org/concepts/castle",
                                            "list_id": "monument-types-collection",
                                            "labels": [
                                                {
                                                    "id": "label-1",
                                                    "language_id": "en",
                                                    "list_item_id": "concept-castle-uuid",
                                                    "value": "Castle",
                                                    "valuetype_id": "prefLabel"
                                                }
                                            ]
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                ]
            }
        }

        result = await resolve_reference_markers(
            json.dumps(business_data),
            json.dumps(sample_graph),
            cache
        )

        result_data = json.loads(result)
        tile_data = result_data["business_data"]["resources"][0]["tiles"][0]["data"]["ref-node-1"]

        # Should be unchanged
        assert tile_data[0]["labels"][0]["value"] == "Castle"

    @pytest.mark.asyncio
    async def test_strict_mode_raises_on_missing_concept(self, sample_graph, sample_collection):
        """Should raise error in strict mode when concept not found."""
        import json
        from alizarin import RustRdmCache
        from alizarin_clm import resolve_reference_markers

        cache = RustRdmCache()
        cache.add_collection(sample_collection)

        # Business data with unknown UUID
        business_data = {
            "business_data": {
                "resources": [
                    {
                        "resourceinstanceid": "res-1",
                        "tiles": [
                            {
                                "tileid": "tile-1",
                                "data": {
                                    "ref-node-1": [
                                        {
                                            "__needs_rdm_lookup": True,
                                            "uuid": "nonexistent-uuid"
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                ]
            }
        }

        with pytest.raises(ValueError, match="not found"):
            await resolve_reference_markers(
                json.dumps(business_data),
                json.dumps(sample_graph),
                cache,
                strict=True
            )

    @pytest.mark.asyncio
    async def test_non_strict_mode_passes_through_on_missing(self, sample_graph, sample_collection):
        """Should pass through marker in non-strict mode when concept not found."""
        import json
        from alizarin import RustRdmCache
        from alizarin_clm import resolve_reference_markers

        cache = RustRdmCache()
        cache.add_collection(sample_collection)

        # Business data with unknown UUID
        business_data = {
            "business_data": {
                "resources": [
                    {
                        "resourceinstanceid": "res-1",
                        "tiles": [
                            {
                                "tileid": "tile-1",
                                "data": {
                                    "ref-node-1": [
                                        {
                                            "__needs_rdm_lookup": True,
                                            "uuid": "nonexistent-uuid"
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                ]
            }
        }

        # Should not raise in non-strict mode
        result = await resolve_reference_markers(
            json.dumps(business_data),
            json.dumps(sample_graph),
            cache,
            strict=False
        )

        result_data = json.loads(result)
        tile_data = result_data["business_data"]["resources"][0]["tiles"][0]["data"]["ref-node-1"]

        # Marker should be passed through unchanged
        assert tile_data[0].get("__needs_rdm_lookup") is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
