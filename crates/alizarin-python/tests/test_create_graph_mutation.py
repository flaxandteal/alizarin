"""
Tests for the CreateGraph mutation.

This mutation allows creating new graphs (branches or resource models) entirely through
the mutation API, without needing to start from an existing graph.
"""

import json
import pytest
import alizarin


class TestCreateGraphMutation:
    """Tests for creating graphs via mutations."""

    def test_create_simple_branch(self):
        """Test creating a simple branch from scratch."""
        mutations_json = json.dumps({
            "mutations": [
                {
                    "CreateGraph": {
                        "name": "Test Branch",
                        "is_resource": False,
                        "root_alias": "test_branch",
                        "root_ontology_class": "http://example.org/TestBranch"
                    }
                }
            ]
        })

        result_json = alizarin.apply_mutations_create(mutations_json, graph_json=None)
        result = json.loads(result_json)

        assert result["name"]["en"] == "Test Branch"
        assert result["isresource"] is False
        assert len(result["nodes"]) == 1
        assert result["root"]["alias"] == "test_branch"
        assert result["root"]["ontologyclass"] == "http://example.org/TestBranch"

    def test_create_resource_model(self):
        """Test creating a resource model from scratch."""
        mutations_json = json.dumps({
            "mutations": [
                {
                    "CreateGraph": {
                        "name": "Person",
                        "is_resource": True,
                        "root_alias": "person",
                        "root_ontology_class": "http://example.org/Person",
                        "author": "Test Author",
                        "description": "A test person model"
                    }
                }
            ]
        })

        result_json = alizarin.apply_mutations_create(mutations_json, graph_json=None)
        result = json.loads(result_json)

        assert result["name"]["en"] == "Person"
        assert result["isresource"] is True
        assert result["author"] == "Test Author"
        # Description can be stored as translatable string or plain string depending on implementation
        assert result["description"] == "A test person model" or result["description"] == {"en": "A test person model"}

    def test_create_graph_then_add_nodes(self):
        """Test creating a graph and then adding nodes in a single mutation sequence."""
        mutations_json = json.dumps({
            "mutations": [
                {
                    "CreateGraph": {
                        "name": "Group",
                        "is_resource": True,
                        "root_alias": "group",
                        "root_ontology_class": "http://example.org/Group"
                    }
                },
                {
                    "AddNode": {
                        "parent_alias": "group",
                        "alias": "name",
                        "name": "Name",
                        "cardinality": "n",
                        "datatype": "string",
                        "ontology_class": "http://example.org/Name",
                        "parent_property": "http://example.org/hasName",
                        "options": {}
                    }
                },
                {
                    "AddNode": {
                        "parent_alias": "group",
                        "alias": "members",
                        "name": "Members",
                        "cardinality": "n",
                        "datatype": "resource-instance-list",
                        "ontology_class": "http://example.org/Members",
                        "parent_property": "http://example.org/hasMembers",
                        "options": {}
                    }
                }
            ],
            "options": {
                "autocreate_card": True,
                "autocreate_widget": False  # Disable widget creation - resource-instance-list has no default widget
            }
        })

        result_json = alizarin.apply_mutations_create(mutations_json, graph_json=None)
        result = json.loads(result_json)

        assert result["name"]["en"] == "Group"
        assert len(result["nodes"]) == 3  # root + name + members

        # Find the added nodes
        node_aliases = [n["alias"] for n in result["nodes"]]
        assert "group" in node_aliases
        assert "name" in node_aliases
        assert "members" in node_aliases

    def test_create_graph_with_custom_id(self):
        """Test creating a graph with a custom graph ID."""
        custom_id = "11111111-2222-3333-4444-555555555555"
        mutations_json = json.dumps({
            "mutations": [
                {
                    "CreateGraph": {
                        "name": "Custom ID Graph",
                        "is_resource": False,
                        "root_alias": "custom",
                        "root_ontology_class": "http://example.org/Custom",
                        "graph_id": custom_id
                    }
                }
            ]
        })

        result_json = alizarin.apply_mutations_create(mutations_json, graph_json=None)
        result = json.loads(result_json)

        assert result["graphid"] == custom_id

    def test_error_no_graph_and_no_create_graph(self):
        """Test that an error is raised when no graph is provided and first mutation is not CreateGraph."""
        mutations_json = json.dumps({
            "mutations": [
                {
                    "AddNode": {
                        "parent_alias": "root",
                        "alias": "child",
                        "name": "Child",
                        "cardinality": "n",
                        "datatype": "string",
                        "ontology_class": "http://example.org/Child",
                        "parent_property": "http://example.org/has",
                        "options": {}
                    }
                }
            ]
        })

        with pytest.raises(ValueError, match="No graph provided"):
            alizarin.apply_mutations_create(mutations_json, graph_json=None)

    def test_error_create_graph_with_existing_graph(self):
        """Test that an error is raised when CreateGraph is used with an existing graph."""
        # First create a graph
        create_json = json.dumps({
            "mutations": [
                {
                    "CreateGraph": {
                        "name": "First Graph",
                        "is_resource": False,
                        "root_alias": "first",
                        "root_ontology_class": "http://example.org/First"
                    }
                }
            ]
        })
        graph_json = alizarin.apply_mutations_create(create_json, graph_json=None)

        # Now try to apply CreateGraph to it
        mutations_json = json.dumps({
            "mutations": [
                {
                    "CreateGraph": {
                        "name": "Second Graph",
                        "is_resource": False,
                        "root_alias": "second",
                        "root_ontology_class": "http://example.org/Second"
                    }
                }
            ]
        })

        with pytest.raises(ValueError, match="CreateGraph cannot be used when a graph already exists"):
            alizarin.apply_mutations_create(mutations_json, graph_json=graph_json)

    def test_apply_mutations_create_with_existing_graph(self):
        """Test that apply_mutations_create works normally when graph is provided and no CreateGraph."""
        # First create a graph
        create_json = json.dumps({
            "mutations": [
                {
                    "CreateGraph": {
                        "name": "Base Graph",
                        "is_resource": False,
                        "root_alias": "base",
                        "root_ontology_class": "http://example.org/Base"
                    }
                }
            ]
        })
        graph_json = alizarin.apply_mutations_create(create_json, graph_json=None)

        # Now add a node to it
        mutations_json = json.dumps({
            "mutations": [
                {
                    "AddNode": {
                        "parent_alias": "base",
                        "alias": "child",
                        "name": "Child Node",
                        "cardinality": "n",
                        "datatype": "string",
                        "ontology_class": "http://example.org/Child",
                        "parent_property": "http://example.org/hasChild",
                        "options": {}
                    }
                }
            ]
        })

        result_json = alizarin.apply_mutations_create(mutations_json, graph_json=graph_json)
        result = json.loads(result_json)

        assert len(result["nodes"]) == 2  # root + child
        node_aliases = [n["alias"] for n in result["nodes"]]
        assert "base" in node_aliases
        assert "child" in node_aliases


class TestGetMutationSchema:
    """Tests for the mutation schema."""

    def test_schema_includes_create_graph(self):
        """Test that the mutation schema includes CreateGraph."""
        schema = alizarin.get_mutation_schema()

        # Check CreateGraph is in the GraphMutation oneOf list
        graph_mutation = schema["GraphMutation"]
        create_graph_present = any(
            "CreateGraph" in item
            for item in graph_mutation["oneOf"]
        )
        assert create_graph_present, "CreateGraph should be in GraphMutation.oneOf"

        # Check CreateGraphParams is defined
        assert "CreateGraphParams" in schema
        params = schema["CreateGraphParams"]
        assert "name" in params["required"]
        assert "root_alias" in params["required"]
        assert "root_ontology_class" in params["required"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
