"""
Tests for WKRM, GraphManager, and StaticStore.

Ports tests from tests/graphManager.test.ts
"""

import pytest
import asyncio
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
    GraphManager,
    StaticStore,
    graph_manager,
    static_store,
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
# GraphManager Tests
# =============================================================================


class TestGraphManager:
    """Tests for GraphManager class."""

    def test_create_graph_manager(self):
        """Should create GraphManager instance."""
        gm = GraphManager()
        assert gm.models == {}
        assert gm.arches_client is None
        assert gm._initialized is False

    @pytest.mark.asyncio
    async def test_initialize_graph_manager(self):
        """Should initialize graph manager."""
        gm = GraphManager()
        await gm.initialize()
        assert gm._initialized is True

    @pytest.mark.asyncio
    async def test_initialize_idempotent(self):
        """Should not reinitialize if already initialized."""
        gm = GraphManager()
        await gm.initialize()
        await gm.initialize()  # Should not raise
        assert gm._initialized is True

    def test_register_model(self):
        """Should register a model wrapper."""
        gm = GraphManager()
        mock_wrapper = object()
        gm.register_model("TestModel", mock_wrapper)

        assert "TestModel" in gm.models
        assert gm.models["TestModel"] is mock_wrapper

    @pytest.mark.asyncio
    async def test_get_model_by_name(self):
        """Should get model by name."""
        gm = GraphManager()
        mock_wrapper = object()
        gm.register_model("TestModel", mock_wrapper)

        result = await gm.get("TestModel")
        assert result is mock_wrapper

    @pytest.mark.asyncio
    async def test_get_model_not_found_no_client(self):
        """Should raise when model not found and no client."""
        gm = GraphManager()

        with pytest.raises(ValueError, match="No Arches client configured"):
            await gm.get("NonexistentModel")

    @pytest.mark.asyncio
    async def test_get_resource_no_client(self):
        """Should raise when getting resource without client."""
        gm = GraphManager()

        with pytest.raises(ValueError, match="No Arches client configured"):
            await gm.get_resource("resource-1")

    @pytest.mark.asyncio
    async def test_get_resource_from_cache(self):
        """Should return cached resource."""
        gm = GraphManager()
        mock_resource = object()
        gm._resource_cache["resource-1"] = mock_resource

        result = await gm.get_resource("resource-1")
        assert result is mock_resource

    def test_clear_cache(self):
        """Should clear resource cache."""
        gm = GraphManager()
        gm._resource_cache["resource-1"] = object()
        gm._resource_cache["resource-2"] = object()

        gm.clear_cache()
        assert len(gm._resource_cache) == 0


# =============================================================================
# StaticStore Tests
# =============================================================================


class TestStaticStore:
    """Tests for StaticStore class."""

    def test_create_static_store(self):
        """Should create StaticStore instance."""
        store = StaticStore()
        assert store.resources == {}
        assert store.metadata == {}
        assert store.graphs == {}

    @pytest.mark.asyncio
    async def test_store_and_get_resource(self):
        """Should store and retrieve resources."""
        store = StaticStore()
        resource_data = {"id": "resource-1", "data": "test"}

        await store.store_resource("resource-1", resource_data)
        result = await store.get_resource("resource-1")

        assert result == resource_data

    @pytest.mark.asyncio
    async def test_get_resource_not_found(self):
        """Should return None for missing resource."""
        store = StaticStore()

        result = await store.get_resource("nonexistent")
        assert result is None

    def test_store_and_get_graph(self):
        """Should store and retrieve graphs."""
        store = StaticStore()
        graph = create_test_graph()

        store.store_graph("test-graph", graph)
        result = store.get_graph("test-graph")

        assert result is graph
        assert result.graphid == "test-graph"

    def test_get_graph_not_found(self):
        """Should return None for missing graph."""
        store = StaticStore()

        result = store.get_graph("nonexistent")
        assert result is None

    @pytest.mark.asyncio
    async def test_get_meta(self):
        """Should retrieve metadata."""
        store = StaticStore()
        store.metadata["resource-1"] = {"title": "Test", "author": "Tester"}

        result = await store.get_meta("resource-1")
        assert result == {"title": "Test", "author": "Tester"}

    @pytest.mark.asyncio
    async def test_get_meta_not_found(self):
        """Should return None for missing metadata."""
        store = StaticStore()

        result = await store.get_meta("nonexistent")
        assert result is None

    def test_clear(self):
        """Should clear all stored data."""
        store = StaticStore()
        store.resources["r1"] = object()
        store.metadata["m1"] = object()
        store.graphs["g1"] = create_test_graph()

        store.clear()

        assert len(store.resources) == 0
        assert len(store.metadata) == 0
        assert len(store.graphs) == 0


# =============================================================================
# Singleton Tests
# =============================================================================


class TestSingletons:
    """Tests for singleton instances."""

    def test_graph_manager_singleton_exists(self):
        """Should have graph_manager singleton."""
        assert graph_manager is not None
        assert isinstance(graph_manager, GraphManager)

    def test_static_store_singleton_exists(self):
        """Should have static_store singleton."""
        assert static_store is not None
        assert isinstance(static_store, StaticStore)


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


# =============================================================================
# Integration Tests
# =============================================================================


class TestGraphManagerIntegration:
    """Integration tests for GraphManager and StaticStore."""

    @pytest.mark.asyncio
    async def test_store_graph_then_use_in_manager(self):
        """Should use graph from store in manager."""
        store = StaticStore()
        graph = create_test_graph()

        # Store graph
        store.store_graph("test-graph", graph)

        # Verify stored
        retrieved = store.get_graph("test-graph")
        assert retrieved is graph

    @pytest.mark.asyncio
    async def test_multiple_graphs_in_store(self):
        """Should store multiple graphs."""
        store = StaticStore()

        for i in range(3):
            root = StaticNode(
                nodeid=f"root-{i}",
                name=f"Root {i}",
                datatype="semantic",
                nodegroup_id=None,
                alias=f"root_{i}",
                graph_id=f"graph-{i}",
                is_collector=False,
                isrequired=False,
                exportable=True,
                istopnode=True,
            )
            graph = StaticGraph(
                graphid=f"graph-{i}",
                name={"en": f"Graph {i}"},
                nodes=[root],
                edges=[],
                nodegroups=[],
                root=root,
            )
            store.store_graph(f"graph-{i}", graph)

        assert len(store.graphs) == 3
        for i in range(3):
            g = store.get_graph(f"graph-{i}")
            assert g is not None
            assert g.graphid == f"graph-{i}"

    @pytest.mark.asyncio
    async def test_multiple_models_in_manager(self):
        """Should register multiple models."""
        gm = GraphManager()

        for i in range(3):
            mock_wrapper = object()
            gm.register_model(f"Model{i}", mock_wrapper)

        assert len(gm.models) == 3
        for i in range(3):
            result = await gm.get(f"Model{i}")
            assert result is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
