"""
Tests for view_models.py ViewModels.

Tests creation and serialization of ViewModels matching TypeScript viewModels tests.
"""

import pytest
import asyncio
from alizarin.static_types import StaticNode, StaticTile, StaticDomainValue
from alizarin.view_models import (
    StringViewModel,
    DateViewModel,
    NumberViewModel,
    BooleanViewModel,
    UrlViewModel,
    NonLocalizedStringViewModel,
    DomainValueViewModel,
    DomainValueListViewModel,
    DEFAULT_LANGUAGE,
    CUSTOM_DATATYPES,
    ViewContext,
    view_context,
)


def create_test_node(
    nodeid: str,
    alias: str,
    datatype: str,
    nodegroup_id: str = "test-ng",
    config: dict = None,
) -> StaticNode:
    """Helper to create a test node."""
    return StaticNode(
        nodeid=nodeid,
        name=alias,
        datatype=datatype,
        nodegroup_id=nodegroup_id,
        alias=alias,
        graph_id="test-graph",
        is_collector=False,
        isrequired=False,
        exportable=True,
        config=config or {},
    )


def create_test_tile(
    tileid: str,
    nodegroup_id: str,
    data: dict,
) -> StaticTile:
    """Helper to create a test tile."""
    return StaticTile(
        tileid=tileid,
        nodegroup_id=nodegroup_id,
        resourceinstance_id="test-resource",
        data=data,
    )


# =============================================================================
# StringViewModel Tests
# =============================================================================

class TestStringViewModel:
    """Tests for StringViewModel."""

    @pytest.mark.asyncio
    async def test_create_with_localized_string(self):
        """Should create from localized string data."""
        node = create_test_node("node-1", "name", "string")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = {"en": "English Name", "es": "Nombre en Español"}

        vm = await StringViewModel._create(tile, node, data)

        assert vm is not None
        assert str(vm) == "English Name"  # Default language is 'en'

    @pytest.mark.asyncio
    async def test_create_with_plain_string(self):
        """Should handle plain string data."""
        node = create_test_node("node-1", "name", "string")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = "Plain String"

        vm = await StringViewModel._create(tile, node, data)

        assert vm is not None
        # Plain strings should be wrapped

    @pytest.mark.asyncio
    async def test_for_json(self):
        """Should serialize to JSON format."""
        node = create_test_node("node-1", "name", "string")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = {"en": "Test Value"}

        vm = await StringViewModel._create(tile, node, data)
        result = await vm.for_json()

        assert result is not None

    @pytest.mark.asyncio
    async def test_null_data_returns_empty_string(self):
        """Should return empty string for null data."""
        node = create_test_node("node-1", "name", "string")
        tile = create_test_tile("tile-1", "test-ng", {})

        vm = await StringViewModel._create(tile, node, None)

        # StringViewModel returns empty string for null, not None
        assert vm is not None
        assert str(vm) == ""


# =============================================================================
# DateViewModel Tests
# =============================================================================

class TestDateViewModel:
    """Tests for DateViewModel."""

    @pytest.mark.asyncio
    async def test_create_with_date_string(self):
        """Should create from date string."""
        node = create_test_node("node-1", "date_field", "date")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = "2024-01-15"

        vm = await DateViewModel._create(tile, node, data)

        assert vm is not None
        # DateViewModel extends datetime, check year/month/day
        assert vm.year == 2024
        assert vm.month == 1
        assert vm.day == 15

    @pytest.mark.asyncio
    async def test_for_json(self):
        """Should serialize to ISO format string."""
        node = create_test_node("node-1", "date_field", "date")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = "2024-06-30"

        vm = await DateViewModel._create(tile, node, data)
        result = await vm.for_json()

        # for_json returns ISO format string
        assert "2024-06-30" in result


# =============================================================================
# NumberViewModel Tests
# =============================================================================

class TestNumberViewModel:
    """Tests for NumberViewModel."""

    @pytest.mark.asyncio
    async def test_create_with_integer(self):
        """Should create from integer."""
        node = create_test_node("node-1", "count", "number")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = 42

        vm = await NumberViewModel._create(tile, node, data)

        assert vm is not None
        # NumberViewModel extends float
        assert float(vm) == 42.0

    @pytest.mark.asyncio
    async def test_create_with_float(self):
        """Should create from float."""
        node = create_test_node("node-1", "amount", "number")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = 3.14159

        vm = await NumberViewModel._create(tile, node, data)

        assert vm is not None
        # NumberViewModel extends float
        assert float(vm) == 3.14159

    @pytest.mark.asyncio
    async def test_for_json(self):
        """Should serialize to JSON format."""
        node = create_test_node("node-1", "value", "number")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = 100

        vm = await NumberViewModel._create(tile, node, data)
        result = await vm.for_json()

        assert result == 100


# =============================================================================
# BooleanViewModel Tests
# =============================================================================

class TestBooleanViewModel:
    """Tests for BooleanViewModel."""

    @pytest.mark.asyncio
    async def test_create_with_true(self):
        """Should create from True."""
        node = create_test_node("node-1", "is_active", "boolean")
        tile = create_test_tile("tile-1", "test-ng", {})

        vm = await BooleanViewModel._create(tile, node, True)

        assert vm is not None
        # BooleanViewModel uses _value and __bool__
        assert bool(vm) == True

    @pytest.mark.asyncio
    async def test_create_with_false(self):
        """Should create from False."""
        node = create_test_node("node-1", "is_active", "boolean")
        tile = create_test_tile("tile-1", "test-ng", {})

        vm = await BooleanViewModel._create(tile, node, False)

        assert vm is not None
        # BooleanViewModel uses _value and __bool__
        assert bool(vm) == False

    @pytest.mark.asyncio
    async def test_for_json(self):
        """Should serialize to JSON format."""
        node = create_test_node("node-1", "flag", "boolean")
        tile = create_test_tile("tile-1", "test-ng", {})

        vm = await BooleanViewModel._create(tile, node, True)
        result = await vm.for_json()

        assert result == True


# =============================================================================
# UrlViewModel Tests
# =============================================================================

class TestUrlViewModel:
    """Tests for UrlViewModel."""

    @pytest.mark.asyncio
    async def test_create_with_url_object(self):
        """Should create from URL object."""
        node = create_test_node("node-1", "website", "url")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = {
            "url": "https://example.com",
            "url_label": "Example Site",
        }

        vm = await UrlViewModel._create(tile, node, data)

        assert vm is not None
        # UrlViewModel extends str, displays label or url
        assert str(vm) == "Example Site"
        # Access underlying Url via _value
        assert vm._value.url == "https://example.com"

    @pytest.mark.asyncio
    async def test_for_json(self):
        """Should serialize to JSON format."""
        node = create_test_node("node-1", "link", "url")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = {
            "url": "https://test.com",
            "url_label": "Test",
        }

        vm = await UrlViewModel._create(tile, node, data)
        result = await vm.for_json()

        assert result is not None
        # for_json returns dict with url/url_label
        assert isinstance(result, dict)
        assert result.get("url") == "https://test.com"


# =============================================================================
# NonLocalizedStringViewModel Tests
# =============================================================================

class TestNonLocalizedStringViewModel:
    """Tests for NonLocalizedStringViewModel."""

    @pytest.mark.asyncio
    async def test_create_with_string(self):
        """Should create from plain string."""
        node = create_test_node("node-1", "identifier", "non-localized-string")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = "ABC-123"

        vm = await NonLocalizedStringViewModel._create(tile, node, data)

        assert vm is not None
        # NonLocalizedStringViewModel extends str
        assert str(vm) == "ABC-123"

    @pytest.mark.asyncio
    async def test_for_json(self):
        """Should serialize to JSON format."""
        node = create_test_node("node-1", "code", "non-localized-string")
        tile = create_test_tile("tile-1", "test-ng", {})
        data = "CODE-456"

        vm = await NonLocalizedStringViewModel._create(tile, node, data)
        result = await vm.for_json()

        assert result == "CODE-456"


# =============================================================================
# DomainValueViewModel Tests
# =============================================================================

class TestDomainValueViewModel:
    """Tests for DomainValueViewModel."""

    @pytest.mark.asyncio
    async def test_create_with_domain_value_object(self):
        """Should create from StaticDomainValue object."""
        node = create_test_node("node-1", "status", "domain-value")
        tile = create_test_tile("tile-1", "test-ng", {})

        # Create a StaticDomainValue directly
        domain_value = StaticDomainValue(
            id="opt-1",
            selected=False,
            text={"en": "Option 1"},
        )

        vm = await DomainValueViewModel._create(tile, node, domain_value)

        assert vm is not None
        # DomainValueViewModel extends str, displays the domain value
        assert str(vm) == "Option 1"

    @pytest.mark.asyncio
    async def test_null_data_returns_none(self):
        """Should return None for null data."""
        node = create_test_node("node-1", "status", "domain-value")
        tile = create_test_tile("tile-1", "test-ng", {})

        vm = await DomainValueViewModel._create(tile, node, None)

        assert vm is None


# =============================================================================
# ViewContext Tests
# =============================================================================

class TestViewContext:
    """Tests for ViewContext."""

    def test_default_language(self):
        """Should have default language set."""
        assert DEFAULT_LANGUAGE == "en"

    def test_view_context_singleton(self):
        """Should provide singleton view context."""
        assert view_context is not None
        assert isinstance(view_context, ViewContext)

    def test_custom_datatypes_registry(self):
        """Should have custom datatypes registry."""
        assert CUSTOM_DATATYPES is not None
        # Can register custom datatypes
        # CUSTOM_DATATYPES.set("custom-type", CustomViewModel)


# =============================================================================
# Integration Tests
# =============================================================================

class TestViewModelIntegration:
    """Integration tests for ViewModels."""

    @pytest.mark.asyncio
    async def test_multiple_viewmodels_same_tile(self):
        """Should handle multiple ViewModels from same tile."""
        tile = create_test_tile("tile-1", "test-ng", {})

        string_node = create_test_node("node-1", "name", "string")
        date_node = create_test_node("node-2", "date", "date")
        number_node = create_test_node("node-3", "count", "number")

        string_vm = await StringViewModel._create(tile, string_node, {"en": "Test"})
        date_vm = await DateViewModel._create(tile, date_node, "2024-01-01")
        number_vm = await NumberViewModel._create(tile, number_node, 42)

        assert string_vm is not None
        assert date_vm is not None
        assert number_vm is not None

    @pytest.mark.asyncio
    async def test_all_viewmodels_support_for_json(self):
        """All ViewModels should support for_json()."""
        tile = create_test_tile("tile-1", "test-ng", {})

        viewmodels = [
            (StringViewModel, create_test_node("n1", "s", "string"), {"en": "Test"}),
            (DateViewModel, create_test_node("n2", "d", "date"), "2024-01-01"),
            (NumberViewModel, create_test_node("n3", "n", "number"), 42),
            (BooleanViewModel, create_test_node("n4", "b", "boolean"), True),
            (NonLocalizedStringViewModel, create_test_node("n5", "ns", "non-localized-string"), "ABC"),
        ]

        for vm_class, node, data in viewmodels:
            vm = await vm_class._create(tile, node, data)
            if vm is not None:
                result = await vm.for_json()
                assert result is not None, f"{vm_class.__name__}.for_json() returned None"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
