"""
Tests for FileList extension.

Tests static types, view models, and extension registration.
"""

import pytest
import json


# =============================================================================
# Static Types Tests
# =============================================================================

class TestFileListItem:
    """Tests for FileListItem dataclass."""

    def test_create_minimal(self):
        """Should create FileListItem with minimal required fields."""
        from alizarin_filelist import FileListItem

        item = FileListItem(
            name="test.pdf",
            url="/files/abc123",
        )

        assert item.name == "test.pdf"
        assert item.url == "/files/abc123"
        assert item.file_id is None
        assert item.file_type is None
        assert item.size is None

    def test_create_full(self):
        """Should create FileListItem with all fields."""
        from alizarin_filelist import FileListItem
        from alizarin_filelist.static_types import LocalizedStringValue

        item = FileListItem(
            name="photo.jpg",
            url="/files/xyz789",
            file_id="file-uuid-123",
            file_type="image/jpeg",
            size=12345,
            title={"en": LocalizedStringValue(value="My Photo"), "es": LocalizedStringValue(value="Mi Foto")},
            alt_text={"en": LocalizedStringValue(value="A beautiful sunset")},
        )

        assert item.name == "photo.jpg"
        assert item.url == "/files/xyz789"
        assert item.file_id == "file-uuid-123"
        assert item.file_type == "image/jpeg"
        assert item.size == 12345
        assert item.title["en"].value == "My Photo"
        assert item.alt_text["en"].value == "A beautiful sunset"

    def test_from_dict(self):
        """Should create FileListItem from dict."""
        from alizarin_filelist import FileListItem

        data = {
            "name": "document.pdf",
            "url": "/files/doc123",
            "file_id": "doc-uuid",
            "type": "application/pdf",
            "size": 54321,
        }

        item = FileListItem.from_dict(data)

        assert item.name == "document.pdf"
        assert item.url == "/files/doc123"
        assert item.file_id == "doc-uuid"
        assert item.file_type == "application/pdf"
        assert item.size == 54321

    def test_from_dict_with_file_type_key(self):
        """Should handle both 'type' and 'file_type' keys."""
        from alizarin_filelist import FileListItem

        # Using 'file_type' key
        data = {
            "name": "test.png",
            "url": "/files/test",
            "file_type": "image/png",
        }

        item = FileListItem.from_dict(data)
        assert item.file_type == "image/png"

    def test_to_dict(self):
        """Should serialize FileListItem to dict."""
        from alizarin_filelist import FileListItem

        item = FileListItem(
            name="video.mp4",
            url="/files/video",
            file_id="vid-123",
            file_type="video/mp4",
            size=999999,
        )

        data = item.to_dict()

        assert data["name"] == "video.mp4"
        assert data["url"] == "/files/video"
        assert data["file_id"] == "vid-123"
        assert data["type"] == "video/mp4"
        assert data["size"] == 999999

    def test_to_display_string(self):
        """Should return name as display string."""
        from alizarin_filelist import FileListItem

        item = FileListItem(name="report.xlsx", url="/files/report")

        assert item.to_display_string() == "report.xlsx"

    def test_is_image_true(self):
        """Should identify image file types."""
        from alizarin_filelist import FileListItem

        image_types = [
            "image/jpeg",
            "image/png",
            "image/gif",
            "image/webp",
            "image/svg+xml",
            "image/bmp",
        ]

        for file_type in image_types:
            item = FileListItem(name="test", url="/test", file_type=file_type)
            assert item.is_image() is True, f"{file_type} should be identified as image"

    def test_is_image_false(self):
        """Should return False for non-image file types."""
        from alizarin_filelist import FileListItem

        non_image_types = [
            "application/pdf",
            "text/plain",
            "video/mp4",
            "audio/mpeg",
            None,
        ]

        for file_type in non_image_types:
            item = FileListItem(name="test", url="/test", file_type=file_type)
            assert item.is_image() is False, f"{file_type} should not be identified as image"

    def test_get_alt_text_with_language(self):
        """Should get alt text for specific language."""
        from alizarin_filelist import FileListItem
        from alizarin_filelist.static_types import LocalizedStringValue

        item = FileListItem(
            name="photo.jpg",
            url="/files/photo",
            alt_text={
                "en": LocalizedStringValue(value="English description"),
                "es": LocalizedStringValue(value="Spanish description"),
            },
        )

        assert item.get_alt_text("en") == "English description"
        assert item.get_alt_text("es") == "Spanish description"

    def test_get_alt_text_fallback(self):
        """Should return None when language not found (no fallback in Python impl)."""
        from alizarin_filelist import FileListItem
        from alizarin_filelist.static_types import LocalizedStringValue

        item = FileListItem(
            name="photo.jpg",
            url="/files/photo",
            alt_text={
                "en": LocalizedStringValue(value="English description"),
                "de": LocalizedStringValue(value="German description"),
            },
        )

        # Python impl returns None when exact language not found
        assert item.get_alt_text("fr") is None
        assert item.get_alt_text("en") == "English description"

    def test_get_alt_text_no_alt(self):
        """Should return None when no alt text."""
        from alizarin_filelist import FileListItem

        item = FileListItem(name="photo.jpg", url="/files/photo")

        assert item.get_alt_text() is None


# =============================================================================
# FileItemViewModel Tests
# =============================================================================

class TestFileItemViewModel:
    """Tests for FileItemViewModel."""

    def test_create_from_file_item(self):
        """Should create ViewModel from FileListItem."""
        from alizarin_filelist import FileItemViewModel, FileListItem

        item = FileListItem(
            name="document.pdf",
            url="/files/doc",
            file_id="doc-123",
            file_type="application/pdf",
            size=54321,
        )

        vm = FileItemViewModel(item)

        assert vm.name == "document.pdf"
        assert vm.url == "/files/doc"
        assert vm.file_id == "doc-123"
        assert vm.file_type == "application/pdf"
        assert vm.size == 54321

    def test_str_returns_name(self):
        """Should return name when converted to string."""
        from alizarin_filelist import FileItemViewModel, FileListItem

        item = FileListItem(name="report.xlsx", url="/files/report")
        vm = FileItemViewModel(item)

        assert str(vm) == "report.xlsx"

    def test_is_image_delegates(self):
        """Should delegate is_image to underlying item."""
        from alizarin_filelist import FileItemViewModel, FileListItem

        image_item = FileListItem(name="photo.jpg", url="/files/photo", file_type="image/jpeg")
        pdf_item = FileListItem(name="doc.pdf", url="/files/doc", file_type="application/pdf")

        assert FileItemViewModel(image_item).is_image() is True
        assert FileItemViewModel(pdf_item).is_image() is False

    def test_get_alt_text_delegates(self):
        """Should delegate get_alt_text to underlying item."""
        from alizarin_filelist import FileItemViewModel, FileListItem
        from alizarin_filelist.static_types import LocalizedStringValue

        item = FileListItem(
            name="photo.jpg",
            url="/files/photo",
            alt_text={"en": LocalizedStringValue(value="A photo")},
        )
        vm = FileItemViewModel(item)

        assert vm.get_alt_text("en") == "A photo"

    def test_get_value_returns_item(self):
        """Should return underlying FileListItem."""
        from alizarin_filelist import FileItemViewModel, FileListItem

        item = FileListItem(name="test.txt", url="/files/test")
        vm = FileItemViewModel(item)

        assert vm.get_value() is item


# =============================================================================
# FileListViewModel Tests
# =============================================================================

class TestFileListViewModel:
    """Tests for FileListViewModel."""

    @pytest.mark.asyncio
    async def test_create_from_array(self):
        """Should create ViewModel from array of file data."""
        from alizarin_filelist import FileListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        file_data = [
            {
                "name": "file1.pdf",
                "url": "/files/1",
                "file_id": "id-1",
                "type": "application/pdf",
            },
            {
                "name": "file2.jpg",
                "url": "/files/2",
                "file_id": "id-2",
                "type": "image/jpeg",
            },
        ]

        vm = await FileListViewModel._create(tile, node, file_data)

        assert vm is not None
        assert len(vm) == 2

    @pytest.mark.asyncio
    async def test_create_empty_list(self):
        """Should handle empty file list."""
        from alizarin_filelist import FileListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        vm = await FileListViewModel._create(tile, node, [])

        assert vm is not None
        assert len(vm) == 0

    @pytest.mark.asyncio
    async def test_create_null_returns_empty(self):
        """Should return empty list for null data."""
        from alizarin_filelist import FileListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        vm = await FileListViewModel._create(tile, node, None)

        assert vm is not None
        assert len(vm) == 0

    @pytest.mark.asyncio
    async def test_get_images(self):
        """Should filter to only image files."""
        from alizarin_filelist import FileListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        file_data = [
            {"name": "doc.pdf", "url": "/files/1", "type": "application/pdf"},
            {"name": "photo1.jpg", "url": "/files/2", "type": "image/jpeg"},
            {"name": "photo2.png", "url": "/files/3", "type": "image/png"},
            {"name": "text.txt", "url": "/files/4", "type": "text/plain"},
        ]

        vm = await FileListViewModel._create(tile, node, file_data)
        images = vm.get_images()

        assert len(images) == 2
        assert all(img.is_image() for img in images)

    @pytest.mark.asyncio
    async def test_get_by_name(self):
        """Should find file by name."""
        from alizarin_filelist import FileListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        file_data = [
            {"name": "report.pdf", "url": "/files/1"},
            {"name": "photo.jpg", "url": "/files/2"},
        ]

        vm = await FileListViewModel._create(tile, node, file_data)

        found = vm.get_by_name("report.pdf")
        assert found is not None
        assert found.name == "report.pdf"

        not_found = vm.get_by_name("nonexistent.txt")
        assert not_found is None

    @pytest.mark.asyncio
    async def test_get_by_id(self):
        """Should find file by ID."""
        from alizarin_filelist import FileListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        file_data = [
            {"name": "file1.pdf", "url": "/files/1", "file_id": "uuid-1"},
            {"name": "file2.pdf", "url": "/files/2", "file_id": "uuid-2"},
        ]

        vm = await FileListViewModel._create(tile, node, file_data)

        found = vm.get_by_id("uuid-2")
        assert found is not None
        assert found.file_id == "uuid-2"
        assert found.name == "file2.pdf"

        not_found = vm.get_by_id("nonexistent-uuid")
        assert not_found is None

    @pytest.mark.asyncio
    async def test_for_json(self):
        """Should serialize to JSON format."""
        from alizarin_filelist import FileListViewModel
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        file_data = [
            {"name": "file1.pdf", "url": "/files/1", "file_id": "uuid-1", "type": "application/pdf"},
        ]

        vm = await FileListViewModel._create(tile, node, file_data)
        result = await vm.for_json()

        assert result is not None
        assert isinstance(result, list)
        assert len(result) == 1
        assert result[0]["name"] == "file1.pdf"


# =============================================================================
# Extension Registration Tests
# =============================================================================

class TestExtensionRegistration:
    """Tests for FileList extension registration with alizarin."""

    def test_custom_datatypes_has_file_list(self):
        """After importing alizarin_filelist, 'file-list' should be in CUSTOM_DATATYPES."""
        # Import FileList extension (auto-registers)
        import alizarin_filelist  # noqa: F401
        from alizarin.view_models import CUSTOM_DATATYPES

        assert "file-list" in CUSTOM_DATATYPES
        assert CUSTOM_DATATYPES["file-list"]._create is not None

    def test_file_list_datatype_factory(self):
        """FileListDataType should have correct interface."""
        from alizarin_filelist import FileListDataType

        # Should have _create class method
        assert hasattr(FileListDataType, "_create")
        assert callable(FileListDataType._create)

    def test_module_exports(self):
        """Module should export expected items."""
        import alizarin_filelist

        # Static types
        assert hasattr(alizarin_filelist, "FileListItem")

        # View models
        assert hasattr(alizarin_filelist, "FileItemViewModel")
        assert hasattr(alizarin_filelist, "FileListViewModel")
        assert hasattr(alizarin_filelist, "FileListDataType")

        # Version
        assert hasattr(alizarin_filelist, "__version__")


# =============================================================================
# Integration Tests
# =============================================================================

class TestIntegration:
    """Integration tests for FileList with alizarin core."""

    @pytest.mark.asyncio
    async def test_get_view_model_returns_file_list_viewmodel(self):
        """get_view_model should return FileListViewModel for file-list datatype."""
        # Ensure extension is registered
        import alizarin_filelist  # noqa: F401
        from alizarin.view_models import CUSTOM_DATATYPES
        from alizarin import StaticTile, StaticNode

        node = StaticNode(
            nodeid="file-node-1",
            name="Attachments",
            datatype="file-list",
            nodegroup_id="ng-1",
            alias="attachments",
            graph_id="test-graph",
            is_collector=True,
            isrequired=False,
            exportable=True,
            config={},
        )

        tile = StaticTile(
            tileid="tile-1",
            nodegroup_id="ng-1",
            resourceinstance_id="resource-1",
            data={},
        )

        file_data = [
            {"name": "test.pdf", "url": "/files/test", "type": "application/pdf"},
        ]

        # Use the registered datatype directly
        datatype = CUSTOM_DATATYPES.get("file-list")
        assert datatype is not None

        vm = await datatype._create(tile, node, file_data)

        assert vm is not None
        # Should be a FileListViewModel (which is a list)
        assert isinstance(vm, list)
        assert len(vm) == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
