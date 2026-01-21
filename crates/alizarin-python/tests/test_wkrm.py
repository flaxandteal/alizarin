"""
Tests for WKRM (Well-Known Resource Model).

Ports tests from tests/graphManager.test.ts
"""

import pytest
from typing import Any, Dict, List, Optional

from alizarin.static_types import (
    StaticNode,
    StaticEdge,
    StaticNodegroup,
    StaticGraph,
    StaticGraphMeta,
    StaticTranslatableString,
)
from alizarin.graph_manager import (
    WKRM,
    IRIVM,
    IWKRM,
    IModelWrapper,
    IInstanceWrapper,
)


# =============================================================================
# Test Fixtures
# =============================================================================


def create_test_graph_meta() -> StaticGraphMeta:
    """Create a test StaticGraphMeta."""
    return StaticGraphMeta(
        graphid="test-graph-id",
        name={"en": "Test Model"},
        slug="test-model",
        iconclass="fa fa-test",
        isresource=True,
        relatable_resource_model_ids=["related-1", "related-2"],
    )


def create_test_graph() -> StaticGraph:
    """Create a minimal test graph."""
    root = StaticNode(
        nodeid="root-node",
        name="Root",
        datatype="semantic",
        nodegroup_id=None,
        alias="root",
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        istopnode=True,
    )
    return StaticGraph(
        graphid="test-graph",
        name={"en": "Test Graph"},
        nodes=[root],
        edges=[],
        nodegroups=[],
        root=root,
    )


# =============================================================================
# WKRM Tests
# =============================================================================


class TestWKRM:
    """Tests for WKRM dataclass."""

    def test_create_wkrm(self):
        """Should create WKRM with required fields."""
        wkrm = WKRM(
            graphid="graph-1",
            name="Test Model",
            slug="test-model",
            model_name="TestModel",
            model_class_name="TestModel",
        )
        assert wkrm.graphid == "graph-1"
        assert wkrm.name == "Test Model"
        assert wkrm.slug == "test-model"
        assert wkrm.model_name == "TestModel"

    def test_graph_id_property(self):
        """Should return graphid via graph_id property."""
        wkrm = WKRM(graphid="graph-id-123")
        assert wkrm.graph_id == "graph-id-123"

    def test_from_meta(self):
        """Should create WKRM from StaticGraphMeta."""
        meta = create_test_graph_meta()
        wkrm = WKRM.from_meta(meta)

        assert wkrm.graphid == "test-graph-id"
        assert wkrm.name == "Test Model"
        assert wkrm.slug == "test-model"
        assert wkrm.model_name == "Test Model"
        assert wkrm.model_class_name == "TestModel"
        assert wkrm.relatable_resource_model_ids == ["related-1", "related-2"]
        assert wkrm.meta is meta

    def test_from_meta_with_string_name(self):
        """Should handle string name in meta."""
        meta = StaticGraphMeta(
            graphid="graph-1",
            name="Simple Name",
            slug="simple-name",
            iconclass="",
            isresource=True,
            relatable_resource_model_ids=[],
        )
        wkrm = WKRM.from_meta(meta)

        assert wkrm.name == "Simple Name"
        assert wkrm.model_class_name == "SimpleName"

    def test_from_meta_with_dict_name(self):
        """Should extract 'en' from dict name."""
        meta = StaticGraphMeta(
            graphid="graph-1",
            name={"en": "English Name", "es": "Nombre"},
            slug="english-name",
            iconclass="",
            isresource=True,
            relatable_resource_model_ids=[],
        )
        wkrm = WKRM.from_meta(meta)

        assert wkrm.name == "English Name"
        assert wkrm.model_class_name == "EnglishName"

    def test_default_relatable_models_empty(self):
        """Should default relatable_resource_model_ids to empty list."""
        wkrm = WKRM(graphid="graph-1")
        assert wkrm.relatable_resource_model_ids == []


# =============================================================================
# Protocol Tests
# =============================================================================


class TestProtocols:
    """Tests for protocol definitions."""

    def test_wkrm_satisfies_iwkrm(self):
        """WKRM should satisfy IWKRM protocol."""
        meta = create_test_graph_meta()
        wkrm = WKRM.from_meta(meta)

        # These are the required attributes
        assert hasattr(wkrm, "model_name")
        assert hasattr(wkrm, "model_class_name")
        assert hasattr(wkrm, "graph_id")
        assert hasattr(wkrm, "meta")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
