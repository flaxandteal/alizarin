#!/usr/bin/env python3
"""
Example showing how to convert between tiled resources and nested JSON.

Usage:
    # Convert resource to JSON
    json_dict = alizarin.resource_to_json(resource, graph_model)

    # Convert JSON back to resource
    resource = alizarin.json_to_resource(json_dict, graph_model)

The nested JSON structure mirrors the TypeScript API:
    {
        "resourceinstanceid": "...",
        "graph_id": "...",
        "basic_info": [
            {
                "name": "Global Group",  # StaticTranslatableString
                "description": ...
            }
        ],
        "permissions": [
            {
                "action": ["Reading"],
                ...
            }
        ]
    }

Where field names come from node aliases and arrays represent nodegroups with cardinality='n'.
"""

import json
import alizarin

def example_resource_to_json():
    """Convert a tiled resource to nested JSON"""

    # Load graph model (contains complete node/nodegroup/edge structure)
    with open('tests/data/models/Group.json') as f:
        graph_data = json.load(f)
        graph = alizarin.StaticGraph(json.dumps(graph_data['graph'][0]))

    # Load resource with tiles
    with open('tests/data/resources/group_instance.json') as f:
        resource_data = json.load(f)
        resource = alizarin.StaticResource(json.dumps(resource_data))

    # Convert to nested JSON
    nested_json = alizarin.resource_to_json(resource, graph)

    # Access fields like JavaScript:
    # nested_json['basic_info'][0]['name'] == "Global Group"
    # nested_json['basic_info'][0]['name'].lang('ga') == "Grúpa Domhanda"

    print(json.dumps(nested_json, indent=2))
    return nested_json


def example_json_to_resource():
    """Convert nested JSON to a tiled resource"""

    # Load graph model
    with open('tests/data/models/Group.json') as f:
        graph_data = json.load(f)
        graph = alizarin.StaticGraph(json.dumps(graph_data['graph'][0]))

    # Create nested JSON structure
    nested_json = {
        "resourceinstanceid": "new-resource-id",
        "graph_id": graph.graphid,
        "basic_info": [
            {
                "name": {"en": "My New Group", "ga": "Mo Ghrúpa Nua"},
                "description": "A test group"
            }
        ]
    }

    # Convert to tiled resource
    resource = alizarin.json_to_resource(nested_json, graph)

    # Resource now has tiles structured according to the graph model
    print(f"Created resource with {len(resource.get_tiles())} tiles")
    return resource


if __name__ == "__main__":
    print("Example 1: Resource to JSON")
    print("=" * 50)
    example_resource_to_json()

    print("\n\nExample 2: JSON to Resource")
    print("=" * 50)
    example_json_to_resource()
