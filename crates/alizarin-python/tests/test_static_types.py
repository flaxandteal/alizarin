"""
Tests for static_types.py dataclasses and serialization.

Ports tests from tests/static-types.test.ts
Tests creation, serialization, and deserialization of static types.
"""

import pytest
import json
from alizarin.static_types import (
    StaticTranslatableString,
    StaticNode,
    StaticEdge,
    StaticNodegroup,
    StaticTile,
    StaticGraph,
    StaticResource,
    StaticPublication,
    StaticGraphMeta,
    StaticCard,
    StaticConstraint,
    StaticValue,
    StaticConcept,
    StaticCollection,
    StaticDomainValue,
    StaticResourceReference,
)


# =============================================================================
# StaticTranslatableString Tests
# =============================================================================

class TestStaticTranslatableString:
    """Tests for StaticTranslatableString."""

    def test_create_from_string(self):
        """Should create from a plain string with 'en' as default."""
        ts = StaticTranslatableString("Hello")
        assert str(ts) == "Hello"
        assert ts["en"] == "Hello"

    def test_create_from_dict(self):
        """Should create from a language dict."""
        ts = StaticTranslatableString({"en": "Hello", "es": "Hola"})
        assert ts["en"] == "Hello"
        assert ts["es"] == "Hola"

    def test_get_with_default(self):
        """Should return default for missing language."""
        ts = StaticTranslatableString("Hello")
        assert ts.get("fr", "default") == "default"

    def test_to_json(self):
        """Should serialize to JSON."""
        ts = StaticTranslatableString({"en": "Hello", "es": "Hola"})
        result = ts.to_json()
        assert result == {"en": "Hello", "es": "Hola"}

    def test_str_returns_first_value_if_no_en(self):
        """Should return first value if 'en' is missing."""
        ts = StaticTranslatableString({"es": "Hola", "fr": "Bonjour"})
        # Should return one of the values (order may vary)
        assert str(ts) in ["Hola", "Bonjour"]


# =============================================================================
# StaticNode Tests
# =============================================================================

class TestStaticNode:
    """Tests for StaticNode."""

    def test_create_minimal_node(self):
        """Should create a node with required fields."""
        node = StaticNode(
            nodeid="test-node-id",
            name="Test Node",
            datatype="string",
            nodegroup_id="test-ng",
            alias="test_alias",
            graph_id="test-graph",
            is_collector=False,
            isrequired=False,
            exportable=True,
        )
        assert node.nodeid == "test-node-id"
        assert node.name == "Test Node"
        assert node.datatype == "string"
        assert node.alias == "test_alias"

    def test_to_dict(self):
        """Should serialize to dict."""
        node = StaticNode(
            nodeid="node-1",
            name="Node One",
            datatype="string",
            nodegroup_id="ng-1",
            alias="node_one",
            graph_id="graph-1",
            is_collector=False,
            isrequired=True,
            exportable=True,
        )
        d = node.to_dict()
        assert d["nodeid"] == "node-1"
        assert d["name"] == "Node One"
        assert d["isrequired"] == True

    def test_from_dict(self):
        """Should deserialize from dict."""
        data = {
            "nodeid": "node-2",
            "name": "Node Two",
            "datatype": "concept",
            "nodegroup_id": "ng-2",
            "alias": "node_two",
            "graph_id": "graph-1",
            "is_collector": True,
            "isrequired": False,
            "exportable": False,
        }
        node = StaticNode.from_dict(data)
        assert node.nodeid == "node-2"
        assert node.datatype == "concept"
        assert node.is_collector == True

    def test_optional_fields_default_to_none(self):
        """Should default optional fields to None."""
        node = StaticNode(
            nodeid="node-3",
            name="Node Three",
            datatype="string",
            nodegroup_id=None,
            alias=None,
            graph_id="graph-1",
            is_collector=False,
            isrequired=False,
            exportable=False,
        )
        assert node.ontologyclass is None
        assert node.parentproperty is None
        assert node.sortorder is None


# =============================================================================
# StaticEdge Tests
# =============================================================================

class TestStaticEdge:
    """Tests for StaticEdge."""

    def test_create_edge(self):
        """Should create an edge."""
        edge = StaticEdge(
            edgeid="edge-1",
            domainnode_id="node-parent",
            rangenode_id="node-child",
            graph_id="graph-1",
        )
        assert edge.edgeid == "edge-1"
        assert edge.domainnode_id == "node-parent"
        assert edge.rangenode_id == "node-child"

    def test_to_dict(self):
        """Should serialize to dict."""
        edge = StaticEdge(
            edgeid="edge-2",
            domainnode_id="node-a",
            rangenode_id="node-b",
            graph_id="graph-1",
            ontologyproperty="http://example.org/hasChild",
        )
        d = edge.to_dict()
        assert d["edgeid"] == "edge-2"
        assert d["ontologyproperty"] == "http://example.org/hasChild"


# =============================================================================
# StaticNodegroup Tests
# =============================================================================

class TestStaticNodegroup:
    """Tests for StaticNodegroup."""

    def test_create_nodegroup(self):
        """Should create a nodegroup."""
        ng = StaticNodegroup(
            nodegroupid="ng-1",
            cardinality="n",
        )
        assert ng.nodegroupid == "ng-1"
        assert ng.cardinality == "n"

    def test_cardinality_one(self):
        """Should handle cardinality '1'."""
        ng = StaticNodegroup(
            nodegroupid="ng-2",
            cardinality="1",
        )
        assert ng.cardinality == "1"


# =============================================================================
# StaticTile Tests
# =============================================================================

class TestStaticTile:
    """Tests for StaticTile."""

    def test_create_tile(self):
        """Should create a tile."""
        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={"node-1": "value-1"},
        )
        assert tile.tileid == "tile-1"
        assert tile.data["node-1"] == "value-1"

    def test_tile_with_parent(self):
        """Should handle parent tile."""
        tile = StaticTile(
            tileid="tile-child",
            nodegroup_id="ng-child",
            resourceinstance_id="resource-1",
            data={},
            parenttile_id="tile-parent",
        )
        assert tile.parenttile_id == "tile-parent"

    def test_to_dict(self):
        """Should serialize to dict."""
        tile = StaticTile(
            tileid="tile-2",
            nodegroup_id="ng-2",
            resourceinstance_id="resource-2",
            data={"node-2": {"en": "English", "ga": "Gaeilge"}},
        )
        d = tile.to_dict()
        assert d["tileid"] == "tile-2"
        assert d["data"]["node-2"]["en"] == "English"


# =============================================================================
# StaticGraph Tests
# =============================================================================

class TestStaticGraph:
    """Tests for StaticGraph."""

    def test_create_minimal_graph(self):
        """Should create a minimal graph."""
        root = StaticNode(
            nodeid="root",
            name="Root",
            datatype="semantic",
            nodegroup_id=None,
            alias="",
            graph_id="graph-1",
            is_collector=False,
            isrequired=False,
            exportable=True,
            istopnode=True,
        )
        graph = StaticGraph(
            graphid="graph-1",
            name={"en": "Test Graph"},
            nodes=[root],
            edges=[],
            nodegroups=[],
            root=root,
        )
        assert graph.graphid == "graph-1"
        assert graph.root.nodeid == "root"

    def test_build_indices(self):
        """Should build lookup indices."""
        root = StaticNode(
            nodeid="root",
            name="Root",
            datatype="semantic",
            nodegroup_id=None,
            alias="root",
            graph_id="graph-1",
            is_collector=False,
            isrequired=False,
            exportable=True,
            istopnode=True,
        )
        child = StaticNode(
            nodeid="child",
            name="Child",
            datatype="string",
            nodegroup_id="ng-child",
            alias="child_alias",
            graph_id="graph-1",
            is_collector=False,
            isrequired=False,
            exportable=True,
        )
        edge = StaticEdge(
            edgeid="edge-1",
            domainnode_id="root",
            rangenode_id="child",
            graph_id="graph-1",
        )
        graph = StaticGraph(
            graphid="graph-1",
            name={"en": "Test Graph"},
            nodes=[root, child],
            edges=[edge],
            nodegroups=[],
            root=root,
        )
        graph.build_indices()

        assert graph.get_node("root") == root
        assert graph.get_node("child") == child
        assert graph.get_node_by_alias("child_alias") == child
        assert len(graph.get_child_edges("root")) == 1

    def test_from_dict(self):
        """Should deserialize from dict."""
        data = {
            "graphid": "graph-2",
            "name": {"en": "Graph Two"},
            "nodes": [
                {
                    "nodeid": "root",
                    "name": "Root",
                    "datatype": "semantic",
                    "nodegroup_id": None,
                    "alias": "",
                    "graph_id": "graph-2",
                    "is_collector": False,
                    "isrequired": False,
                    "exportable": True,
                    "istopnode": True,
                }
            ],
            "edges": [],
            "nodegroups": [],
            "root": {
                "nodeid": "root",
                "name": "Root",
                "datatype": "semantic",
                "nodegroup_id": None,
                "alias": "",
                "graph_id": "graph-2",
                "is_collector": False,
                "isrequired": False,
                "exportable": True,
                "istopnode": True,
            },
        }
        graph = StaticGraph.from_dict(data)
        assert graph.graphid == "graph-2"
        assert len(graph.nodes) == 1

    def test_from_json_wrapped_format(self):
        """Should handle wrapped JSON format."""
        json_str = json.dumps({
            "graph": [{
                "graphid": "graph-3",
                "name": {"en": "Graph Three"},
                "nodes": [{
                    "nodeid": "root",
                    "name": "Root",
                    "datatype": "semantic",
                    "nodegroup_id": None,
                    "alias": "",
                    "graph_id": "graph-3",
                    "is_collector": False,
                    "isrequired": False,
                    "exportable": True,
                    "istopnode": True,
                }],
                "edges": [],
                "nodegroups": [],
                "root": {
                    "nodeid": "root",
                    "name": "Root",
                    "datatype": "semantic",
                    "nodegroup_id": None,
                    "alias": "",
                    "graph_id": "graph-3",
                    "is_collector": False,
                    "isrequired": False,
                    "exportable": True,
                    "istopnode": True,
                },
            }]
        })
        graph = StaticGraph.from_json(json_str)
        assert graph.graphid == "graph-3"


# =============================================================================
# StaticResource Tests
# =============================================================================

class TestStaticResource:
    """Tests for StaticResource."""

    def test_create_resource(self):
        """Should create a resource."""
        resource = StaticResource(
            resourceinstanceid="resource-1",
            graph_id="graph-1",
            tiles=[],
        )
        assert resource.resourceinstanceid == "resource-1"
        assert resource.graph_id == "graph-1"

    def test_resource_with_tiles(self):
        """Should store tiles."""
        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={"node-1": "value"},
        )
        resource = StaticResource(
            resourceinstanceid="resource-1",
            graph_id="graph-1",
            tiles=[tile],
        )
        assert len(resource.tiles) == 1
        assert resource.tiles[0].tileid == "tile-1"

    def test_to_dict(self):
        """Should serialize to dict."""
        resource = StaticResource(
            resourceinstanceid="resource-2",
            graph_id="graph-2",
            tiles=[],
            name="Test Resource",
        )
        d = resource.to_dict()
        assert d["resourceinstanceid"] == "resource-2"
        assert d["name"] == "Test Resource"


# =============================================================================
# StaticValue and StaticConcept Tests
# =============================================================================

class TestStaticValueAndConcept:
    """Tests for StaticValue and StaticConcept."""

    def test_create_static_value(self):
        """Should create a static value."""
        value = StaticValue(id="value-1", value="Test Value")
        assert value.id == "value-1"
        assert str(value) == "Test Value"

    def test_static_value_to_dict(self):
        """Should serialize value to dict."""
        value = StaticValue(id="value-2", value="Another Value")
        d = value.to_dict()
        assert d["id"] == "value-2"
        assert d["value"] == "Another Value"

    def test_create_static_concept(self):
        """Should create a static concept."""
        value = StaticValue(id="val-1", value="Primary")
        concept = StaticConcept(
            id="concept-1",
            prefLabels={"en": value},
        )
        assert concept.id == "concept-1"
        assert str(concept) == "Primary"

    def test_concept_get_pref_label(self):
        """Should get preferred label."""
        en_value = StaticValue(id="val-en", value="English Label")
        es_value = StaticValue(id="val-es", value="Spanish Label")
        concept = StaticConcept(
            id="concept-2",
            prefLabels={"en": en_value, "es": es_value},
        )
        label = concept.get_pref_label("en")
        assert label.value == "English Label"


# =============================================================================
# StaticCollection Tests
# =============================================================================

class TestStaticCollection:
    """Tests for StaticCollection."""

    def test_create_collection(self):
        """Should create a collection."""
        value = StaticValue(id="val-1", value="Primary")
        concept = StaticConcept(
            id="concept-1",
            prefLabels={"en": value},
        )
        collection = StaticCollection.create(
            name="Test Collection",
            concepts=[concept],
        )
        assert collection.id is not None
        assert len(collection.concepts) == 1

    def test_get_concept_by_value(self):
        """Should find concept by label value."""
        value = StaticValue(id="val-1", value="Primary")
        concept = StaticConcept(
            id="concept-1",
            prefLabels={"en": value},
        )
        collection = StaticCollection.create(
            name="Test Collection",
            concepts=[concept],
        )
        found = collection.get_concept_by_value("Primary")
        assert found is not None
        assert found.id == "concept-1"


# =============================================================================
# StaticDomainValue Tests
# =============================================================================

class TestStaticDomainValue:
    """Tests for StaticDomainValue."""

    def test_create_domain_value(self):
        """Should create a domain value."""
        dv = StaticDomainValue(
            id="dv-1",
            selected=True,
            text={"en": "Option A", "es": "Opción A"},
        )
        assert dv.id == "dv-1"
        assert dv.selected == True
        assert str(dv) == "Option A"

    def test_lang_method(self):
        """Should get text in specific language."""
        dv = StaticDomainValue(
            id="dv-2",
            selected=False,
            text={"en": "Option B", "es": "Opción B"},
        )
        assert dv.lang("es") == "Opción B"
        assert dv.lang("fr") is None


# =============================================================================
# StaticResourceReference Tests
# =============================================================================

class TestStaticResourceReference:
    """Tests for StaticResourceReference."""

    def test_create_resource_reference(self):
        """Should create a resource reference."""
        ref = StaticResourceReference(
            id="ref-1",
            type="Person",
            graphId="person-graph",
            title="John Doe",
        )
        assert ref.id == "ref-1"
        assert ref.type == "Person"
        assert ref.title == "John Doe"

    def test_from_dict(self):
        """Should deserialize from dict."""
        data = {
            "id": "ref-2",
            "type": "Place",
            "graphId": "place-graph",
            "title": "Dublin",
        }
        ref = StaticResourceReference.from_dict(data)
        assert ref.id == "ref-2"
        assert ref.type == "Place"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
