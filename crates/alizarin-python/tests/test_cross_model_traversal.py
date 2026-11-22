"""
Tests for cross-model traversal via resource-instance references.

Demonstrates that when a Group has members (resource-instance-list pointing to Person),
you can traverse: group.members[0].name.forenames[0].forename
"""

import json
import pytest
import os
import uuid

from alizarin.graph_manager import GraphManager
from alizarin.view_models import viewContext


# Test data paths
TEST_DATA_DIR = os.path.join(os.path.dirname(__file__), '../../../tests/data')
MODELS_DIR = os.path.join(TEST_DATA_DIR, 'models')


def load_graph(name: str) -> dict:
    """Load a graph model from the test data directory."""
    path = os.path.join(MODELS_DIR, f'{name}.json')
    with open(path) as f:
        data = json.load(f)
    # Handle wrapped format
    if 'graph' in data and isinstance(data['graph'], list):
        return data['graph'][0]
    return data


@pytest.fixture
def group_graph():
    """Load the Group graph model."""
    return load_graph('Group')


@pytest.fixture
def person_graph():
    """Load the Person graph model."""
    return load_graph('Person')


@pytest.fixture
def graph_manager(group_graph, person_graph):
    """Set up a GraphManager with both models and test resources."""
    gm = GraphManager()

    # Register both graphs
    group_graph_id = gm.register_graph(group_graph)
    person_graph_id = gm.register_graph(person_graph)

    # Create a Person resource
    person_id = str(uuid.uuid4())
    person_resource = {
        "resourceinstance": {
            "resourceinstanceid": person_id,
            "graph_id": person_graph_id,
            "name": "Test Person"
        },
        "tiles": []
    }

    # name -> forenames -> forename: all three nodes are in name's nodegroup,
    # so it is one tile, with forename's value in Arches' per-language form.
    person_nodes = {n['alias']: n for n in person_graph['nodes']}
    forename_node = person_nodes['forename']
    person_resource["tiles"] = [{
        "tileid": str(uuid.uuid4()),
        "nodegroup_id": forename_node['nodegroup_id'],
        "resourceinstance_id": person_id,
        "parenttile_id": None,
        "data": {
            forename_node['nodeid']: {"en": {"direction": "ltr", "value": "Alice"}},
        },
        "sortorder": 0
    }]

    # Register the Person resource
    gm.register_resource(person_resource)

    # Create a Group resource with members pointing to the Person
    group_id = str(uuid.uuid4())
    members_node = {n['alias']: n for n in group_graph['nodes']}['members']
    group_resource = {
        "resourceinstance": {
            "resourceinstanceid": group_id,
            "graph_id": group_graph_id,
            "name": "Test Group"
        },
        "tiles": [{
            "tileid": str(uuid.uuid4()),
            "nodegroup_id": members_node['nodegroup_id'],
            "resourceinstance_id": group_id,
            "parenttile_id": None,
            "data": {
                members_node['nodeid']: [
                    {"resourceId": person_id, "ontologyProperty": "", "inverseOntologyProperty": ""}
                ]
            },
            "sortorder": 0
        }]
    }

    gm.register_resource(group_resource)

    # Store IDs for tests
    gm._test_group_id = group_id
    gm._test_person_id = person_id

    return gm


@pytest.mark.asyncio
async def test_cross_model_traversal_basic(graph_manager):
    """
    Test basic cross-model traversal: group.members[0] returns a ResourceInstanceViewModel.
    """
    # Set the global context
    viewContext.graphManager = graph_manager

    try:
        # Get the group resource
        group = await graph_manager.getResource(graph_manager._test_group_id)

        assert group is not None, "Should get group resource"

        # Access members - this should return a list of ResourceInstanceViewModels
        members = await group.members

        assert members is not None, "Should have members property"
        # members is a list ViewModel

    finally:
        viewContext.graphManager = None


@pytest.mark.asyncio
async def test_cross_model_traversal_to_related_resource(graph_manager):
    """
    Test that accessing properties on a member triggers lazy loading of the Person resource.
    """
    viewContext.graphManager = graph_manager

    try:
        # Get the group resource
        group = await graph_manager.getResource(graph_manager._test_group_id)

        # Get members
        members = await group.members

        assert len(members) == 1
        first_member = members[0]
        assert first_member.id == graph_manager._test_person_id
        # Not loaded yet: reading through it must retrieve() the Person.
        assert first_member._i is None

        # Crossing from the Group model to the Person model. No try/except:
        # an error here is the failure this test exists to catch.
        # name's nodegroup has cardinality n (a list); forenames and forename
        # are in the same tile, so single.
        name = await first_member.name
        forename = await (await name[0].forenames).forename
        assert str(forename) == "Alice"
        assert first_member._i is not None

    finally:
        viewContext.graphManager = None


@pytest.mark.asyncio
async def test_view_model_describes_its_field(graph_manager):
    """
    A view model describes its field through its parent pseudo
    (PARITY-RESEARCH.md C-1: it called the pseudo's missing describeField).
    """
    viewContext.graphManager = graph_manager

    try:
        person = await graph_manager.getResource(graph_manager._test_person_id)
        name = await person.name
        forename = await (await name[0].forenames).forename
        forename_node = {n['alias']: n for n in load_graph('Person')['nodes']}['forename']
        assert forename.describeField() == "forename"
        assert forename.describeFieldGroup() == forename_node['nodegroup_id']

    finally:
        viewContext.graphManager = None


@pytest.mark.asyncio
async def test_graph_manager_resource_lookup(graph_manager):
    """
    Test that GraphManager can look up resources by ID.
    """
    viewContext.graphManager = graph_manager

    try:
        # Should be able to look up the Person directly
        person = await graph_manager.getResource(graph_manager._test_person_id)

        assert person is not None, "Should be able to look up Person by ID"

        # Should be able to look up the Group
        group = await graph_manager.getResource(graph_manager._test_group_id)

        assert group is not None, "Should be able to look up Group by ID"

    finally:
        viewContext.graphManager = None


def test_graph_manager_registers_both_models(graph_manager):
    """
    Test that GraphManager properly registers both Group and Person models.
    """
    # Both graphs should be registered (models are lazily populated on getResource)
    assert len(graph_manager._graphs) == 2, "Should have both graph definitions"
    assert len(graph_manager._graph_ids) == 2, "Should have both graph IDs tracked"


def test_graph_manager_has_resources(graph_manager):
    """
    Test that GraphManager has the test resources registered.
    """
    assert graph_manager.registry.has_full(graph_manager._test_group_id)
    assert graph_manager.registry.has_full(graph_manager._test_person_id)


@pytest.mark.asyncio
async def test_resource_instance_viewmodel_retrieve_requires_graph_manager():
    """
    Test that traversing without a GraphManager raises an appropriate error.
    """
    from alizarin.view_models import ResourceInstanceViewModel

    # Clear any existing graph manager
    viewContext.graphManager = None

    # Create a ResourceInstanceViewModel without a graph manager
    rivm = ResourceInstanceViewModel(
        id="some-resource-id",
        modelWrapper=None,
        instanceWrapperFactory=None,
        cacheEntry=None
    )

    # Trying to retrieve should fail with a clear error
    with pytest.raises(ValueError, match="Cannot traverse resource relationships without a GraphManager"):
        await rivm.retrieve()


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
