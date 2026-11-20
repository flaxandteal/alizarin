"""
CLM (controlled list) collections.

In alizarin a controlled list is an RDM collection (RustRdmCollection), held in
the same RustRdmCache as SKOS concept collections. Its labels resolve in two
places: on a `concept-list` node, to concept ids (core), and on a CLM
`reference` node, to StaticReferences (this extension's handler). A
resource-instance-list is unrelated: it points at resource instances, which no
collection holds.

Uses the Person model, whose `test` node is a multi-value `reference`.
"""

import json
import os
import tempfile
import uuid

import alizarin
import alizarin_clm  # noqa: F401 - registers the reference handler


def load_person_graph():
    """Load Person graph model"""
    test_data_path = os.path.join(
        os.path.dirname(__file__),
        'data/graphs/resource_models/Person.json'
    )

    with open(test_data_path) as f:
        data = json.load(f)
        return data['graph'][0]


def find_node_by_alias(graph_data, alias):
    """Find a node by its alias"""
    for node in graph_data.get('nodes', []):
        if node.get('alias') == alias:
            return node
    return None


def node_values(resource, node_id):
    """The values a resource's tiles hold for a node."""
    return [tile['data'][node_id] for tile in resource['tiles'] if node_id in tile.get('data', {})]


def assert_references_resolve(resource, reference_node, collection, labels):
    """The reference node holds one StaticReference per label, from `collection`."""
    found = node_values(resource, reference_node['nodeid'])
    assert found, "References should be in the tiles"
    references = found[0]
    assert isinstance(references, list)
    assert [ref['labels'][0]['value'] for ref in references] == labels
    for ref, label in zip(references, labels):
        assert ref['list_id'] == collection.id
        assert ref['labels'][0]['list_item_id'] == collection.find_by_label(label).id


def test_create_clm_collection():
    """Test creating a CLM collection for activities"""
    collection = alizarin.RustRdmCollection.from_labels(
        name="Activity Types",
        labels=["Hiking", "Swimming", "Cycling", "Reading"]
    )

    assert collection.id is not None
    assert collection.name == "Activity Types"
    assert len(collection) == 4

    # Verify lookup
    activity = collection.find_by_label("Hiking")
    assert activity is not None
    assert activity.get_label("en") == "Hiking"


def test_collection_labels_resolve_as_clm_references():
    """
    A controlled list's labels, given for a `reference` node, resolve during
    conversion to StaticReferences from that list.
    """
    graph_data = load_person_graph()
    reference_node = find_node_by_alias(graph_data, 'test')
    assert reference_node and reference_node['datatype'] == 'reference'

    collection = alizarin.RustRdmCollection.from_labels(
        name="Activities",
        labels=["Hiking Adventure", "Swimming Competition", "Cycling Tour"],
        id=reference_node['config']['controlledList'],
    )
    cache = alizarin.RustRdmCache()
    cache.add_collection(collection)
    alizarin.set_global_rdm_cache(cache)

    try:
        graph_id = alizarin.register_graph(json.dumps(graph_data))
        result = alizarin.batch_trees_to_tiles(
            trees_json=json.dumps([{'test': ["Hiking Adventure", "Cycling Tour"]}]),
            graph_id=graph_id,
            strict=True,
            random_ids=True,
        )

        resources = result['business_data']['resources']
        assert len(resources) == 1
        assert_references_resolve(resources[0], reference_node, collection, ["Hiking Adventure", "Cycling Tour"])

    finally:
        alizarin.clear_global_rdm_cache()


def test_collection_labels_resolve_on_concept_list():
    """
    The same kind of collection, given for a `concept-list` node (here the
    resource-instance-list `associated_activities` overridden to concept-list),
    resolves its labels to concept ids.
    """
    graph_data = load_person_graph()

    activities_node = find_node_by_alias(graph_data, 'associated_activities')
    assert activities_node, "'associated_activities' node not found in Person model"
    activities_node['datatype'] = 'concept-list'

    collection_id = str(uuid.uuid5(uuid.NAMESPACE_DNS, 'test.activities.concepts'))
    activities_node['config']['rdmCollection'] = collection_id

    collection = alizarin.RustRdmCollection.from_labels(
        name="Activity Concepts",
        labels=["Hiking Adventure", "Swimming Competition", "Cycling Tour"],
        id=collection_id
    )
    cache = alizarin.RustRdmCache()
    cache.add_collection(collection)
    alizarin.set_global_rdm_cache(cache)

    try:
        trees = [
            {
                'name': [{
                    'forenames': [{
                        'forename': 'Jane'
                    }],
                    'surnames': [{
                        'surname': 'Doe'
                    }]
                }],
                'associated_activities': ["Hiking Adventure", "Cycling Tour"],
                'primary_reference_number': 67890
            }
        ]

        graph_id = alizarin.register_graph(json.dumps(graph_data))

        # Resolve labels to concept ids using the RDM cache
        resolved_trees_json = alizarin.resolve_labels_in_tree(
            tree_json=json.dumps(trees),
            graph_id=graph_id,
            strict=True
        )

        result = alizarin.batch_trees_to_tiles(
            trees_json=resolved_trees_json,
            graph_id=graph_id,
            from_camel=False,
            strict=True,
            random_ids=True,
        )

        resources = result['business_data']['resources']
        assert len(resources) == 1

        found = node_values(resources[0], activities_node['nodeid'])
        assert found, "Activity concepts should be in the tiles"
        concept_ids = found[0]
        assert concept_ids == [
            collection.find_by_label("Hiking Adventure").id,
            collection.find_by_label("Cycling Tour").id,
        ]

    finally:
        alizarin.clear_global_rdm_cache()


def test_skos_roundtrip_as_clm_references():
    """
    A controlled list written to SKOS XML and read back into the cache keeps
    its labels, and they resolve as CLM references.
    """
    graph_data = load_person_graph()
    reference_node = find_node_by_alias(graph_data, 'test')
    assert reference_node and reference_node['datatype'] == 'reference'

    original_labels = ["Archery", "Badminton", "Cricket", "Dancing"]
    collection = alizarin.RustRdmCollection.from_labels(
        name="Sports and Activities",
        labels=original_labels,
        id=reference_node['config']['controlledList'],
    )
    assert len(collection) == 4

    base_uri = "http://example.org/activities/"
    skos_xml = collection.to_skos_xml(base_uri)
    assert 'Sports and Activities' in skos_xml
    for label in original_labels:
        assert label in skos_xml

    with tempfile.NamedTemporaryFile(mode='w', suffix='.xml', delete=False) as f:
        temp_file = f.name
        f.write(skos_xml)

    try:
        with open(temp_file, 'r') as f:
            loaded_xml = f.read()

        cache = alizarin.RustRdmCache()
        loaded_ids = cache.add_from_skos_xml(loaded_xml, base_uri)
        assert collection.id in loaded_ids
        loaded_collection = cache.get_collection(collection.id)
        assert len(loaded_collection) == 4
        for label in original_labels:
            activity = cache.lookup_by_label(collection.id, label)
            assert activity is not None
            assert activity.get_label("en") == label

        alizarin.set_global_rdm_cache(cache)
        try:
            trees = [
                {
                    'name': [{
                        'forenames': [{
                            'forename': 'Alice'
                        }],
                        'surnames': [{
                            'surname': 'Johnson'
                        }]
                    }],
                    'test': ["Archery", "Dancing"],
                    'primary_reference_number': 11111,
                    'descriptions': [{
                        'description': 'Active person with varied interests'
                    }]
                }
            ]

            graph_id = alizarin.register_graph(json.dumps(graph_data))
            result = alizarin.batch_trees_to_tiles(
                trees_json=json.dumps(trees),
                graph_id=graph_id,
                strict=True,
                random_ids=True,
            )

            resources = result['business_data']['resources']
            assert len(resources) == 1
            assert_references_resolve(resources[0], reference_node, loaded_collection, ["Archery", "Dancing"])

        finally:
            alizarin.clear_global_rdm_cache()

    finally:
        if os.path.exists(temp_file):
            os.unlink(temp_file)


def test_multilanguage_collection():
    """Test collection with multilingual labels"""
    collection = alizarin.RustRdmCollection("multilang-activities")

    # Add activities with multiple languages
    collection.add_from_label(
        label={
            "en": "Swimming",
            "ga": "Snámh",
            "fr": "Natation"
        },
        id="swimming-id"
    )

    collection.add_from_label(
        label={
            "en": "Running",
            "ga": "Rith",
            "fr": "Course"
        },
        id="running-id"
    )

    assert len(collection) == 2

    # Verify English lookup
    swimming = collection.find_by_label("Swimming")
    assert swimming is not None
    assert swimming.get_label("en") == "Swimming"
    assert swimming.get_label("ga") == "Snámh"
    assert swimming.get_label("fr") == "Natation"

    # Verify Irish lookup
    swimming_ga = collection.find_by_label("Snámh")
    assert swimming_ga is not None
    assert swimming_ga.get_label("en") == "Swimming"
