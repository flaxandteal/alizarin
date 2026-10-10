import json
import os
import alizarin


def load_test_data():
    """Load test graph model from Person.json"""
    test_data_path = os.path.join(
        os.path.dirname(__file__),
        '../../../tests/data/models/Person.json'
    )

    with open(test_data_path) as f:
        data = json.load(f)
        return data['graph'][0]


def test_batch_tiles_to_trees_conversion():
    """Test that semantic nodes within cardinality-n nodegroups don't inherit array behavior.

    Tests that:
    - contact_details (grouping node, cardinality n) IS an array
    - contact_names (semantic leaf node within cardinality-n nodegroup) is NOT an array
    - contact_name_for_correspondence (string leaf within same nodegroup) is NOT an array

    This verifies that only grouping nodes get cardinality-n array behavior, not all nodes
    within a cardinality-n nodegroup.
    """
    graph_data = load_test_data()
    graph_json = json.dumps(graph_data)

    # Register graph first (required by new API)
    graph_id = alizarin.register_graph(graph_json)

    resources = [
        {
            "__scopes": ["public"],
            "metadata": {},
            "resourceinstance": {
                "descriptors": {
                    "name": "Test Person"
                },
                "graph_id": graph_id,
                "name": "Test Person",
                "resourceinstanceid": "test-person-123"
            },
            "tiles": [
                {
                    "data": {
                        # contact_details nodegroup (cardinality n)
                        # contact_name_for_correspondence
                        "2beefb56-4084-11eb-bcc5-f875a44e0e11": "John Doe"
                    },
                    "nodegroup_id": "2547c12f-9505-11ea-a507-f875a44e0e11",
                    "resourceinstance_id": "test-person-123",
                    "sortorder": 0,
                    "tileid": "test-tile-001"
                }
            ]
        }
    ]

    resources_json = json.dumps(resources)
    result = alizarin.batch_tiles_to_trees(
        resources_json=resources_json
    )

    assert 'results' in result, "Result should have 'results' key"
    assert len(result['results']) == 1, "Should have 1 result"

    tree = result['results'][0]

    # contact_details is the grouping node (cardinality n), should be an array
    contact_details_list = tree.get('contact_details', [])
    assert isinstance(contact_details_list, list), \
        "contact_details (grouping node, cardinality n) should be an array"
    assert len(contact_details_list) > 0, "contact_details should have at least one entry"

    contact_detail = contact_details_list[0]

    # contact_names is a semantic node WITHIN the cardinality-n nodegroup,
    # but is NOT the grouping node itself - should NOT be an array
    contact_names = contact_detail.get('contact_names', {})
    assert not isinstance(contact_names, list), \
        "contact_names (semantic leaf within cardinality-n nodegroup) should NOT be an array"

    # contact_name_for_correspondence is a string node within the same nodegroup,
    # should also NOT be an array
    name_for_correspondence = contact_names.get('contact_name_for_correspondence', {})
    assert not isinstance(name_for_correspondence, list), \
        "contact_name_for_correspondence (leaf within cardinality-n nodegroup) should NOT be an array"


def test_compose_resource_layers_is_exported():
    """L4: compose_resource_layers is registered on the native module but must also
    be re-exported from the `alizarin` package facade (__init__.py), or
    `alizarin.compose_resource_layers(...)` raises AttributeError."""
    assert hasattr(alizarin, "compose_resource_layers"), (
        "compose_resource_layers missing from the alizarin package facade"
    )
    assert callable(alizarin.compose_resource_layers)
    assert "compose_resource_layers" in alizarin.__all__


def test_build_resources_from_business_csv_exposes_uuid_namespace():
    """L5: the Python CSV loader must expose `uuid_namespace` so a layer's tile ids
    can be isolated (parity with WASM/NAPI), or Python cannot build composable
    layers from CSV."""
    assert hasattr(alizarin, "build_resources_from_business_csv")
    assert "build_resources_from_business_csv" in alizarin.__all__
    sig = alizarin.build_resources_from_business_csv.__text_signature__ or ""
    assert "uuid_namespace" in sig, f"uuid_namespace absent from signature: {sig!r}"
