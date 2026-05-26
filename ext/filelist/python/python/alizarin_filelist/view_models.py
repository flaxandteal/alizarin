"""
View Models for FileList datatype.

These ViewModels provide a Python interface for working with file-list values.
"""

from __future__ import annotations

import json
from typing import Any, Awaitable, Dict, List, Optional, TYPE_CHECKING, Union

if TYPE_CHECKING:
    from alizarin import StaticTile, StaticNode
    from alizarin.view_models import IViewModel
    from alizarin.pseudos import IPseudo

from .static_types import FileListItem


class FileItemViewModel(str):
    """
    ViewModel for a single file item.

    Extends str to display the file name/title.
    """

    def __new__(cls, file_item: FileListItem):
        return str.__new__(cls, file_item.to_display_string())

    def __init__(self, file_item: FileListItem):
        self._file: FileListItem = file_item
        self._: Optional[Union['IViewModel', Awaitable['IViewModel']]] = None
        self.__parentPseudo: Optional['IPseudo'] = None

    def describe_field(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describe_field_group(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> Dict[str, Any]:
        """Convert to JSON (FileListItem dict)."""
        return self._file.to_dict()

    def get_value(self) -> FileListItem:
        """Get the underlying FileListItem."""
        return self._file

    def lang(self, language: str) -> Optional[str]:
        """Get title in specific language."""
        return self._file.lang(language)

    @property
    def name(self) -> str:
        """Get the filename."""
        return self._file.name

    @property
    def url(self) -> Optional[str]:
        """Get the file URL."""
        return self._file.url

    @property
    def file_id(self) -> Optional[str]:
        """Get the file ID."""
        return self._file.file_id

    @property
    def file_type(self) -> Optional[str]:
        """Get the MIME type."""
        return self._file.file_type

    @property
    def size(self) -> Optional[int]:
        """Get the file size in bytes."""
        return self._file.size

    def is_image(self) -> bool:
        """Check if this is an image file."""
        return self._file.is_image()

    def get_alt_text(self, lang: Optional[str] = None) -> Optional[str]:
        """Get alt text for the file."""
        return self._file.get_alt_text(lang)


class FileListViewModel(list):
    """
    ViewModel for a list of file items.

    Extends list to hold FileItemViewModels.
    """

    def __init__(self, *items):
        super().__init__(items)
        self._: Optional[Union['IViewModel', Awaitable['IViewModel']]] = None
        self.__parentPseudo: Optional['IPseudo'] = None

    def describe_field(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describe_field_group(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> Optional[List[Dict[str, Any]]]:
        """Convert to JSON array."""
        result = []
        for item in self:
            if hasattr(item, '__await__'):
                item = await item
            if item:
                result.append(await item.for_json())
        return result if result else None

    def get_images(self) -> List[FileItemViewModel]:
        """Get only image files from the list."""
        return [item for item in self if isinstance(item, FileItemViewModel) and item.is_image()]

    def get_by_name(self, name: str) -> Optional[FileItemViewModel]:
        """Find a file by name."""
        for item in self:
            if isinstance(item, FileItemViewModel) and item.name == name:
                return item
        return None

    def get_by_id(self, file_id: str) -> Optional[FileItemViewModel]:
        """Find a file by ID."""
        for item in self:
            if isinstance(item, FileItemViewModel) and item.file_id == file_id:
                return item
        return None

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
        cache_entry: Optional[Dict[str, Any]] = None,
    ) -> 'FileListViewModel':
        """
        Create a FileListViewModel.
        """
        nodeid = node.nodeid
        val: List[FileItemViewModel] = []

        if nodeid not in tile.data:
            tile.data[nodeid] = None

        if value is not None:
            tile.data[nodeid] = []

            # Handle awaitable value
            if hasattr(value, '__await__'):
                resolved = await value
                return await FileListViewModel._create(tile, node, resolved, cache_entry)

            # Ensure we have a list
            if not isinstance(value, list):
                if isinstance(value, dict):
                    # Single file object - wrap in list
                    value = [value]
                else:
                    raise ValueError(
                        f"Cannot set file-list value on node {nodeid} except via array or dict: {json.dumps(value)}"
                    )

            # Create FileItemViewModel for each item
            for idx, item in enumerate(value):
                if isinstance(item, FileItemViewModel):
                    val.append(item)
                elif isinstance(item, FileListItem):
                    val.append(FileItemViewModel(item))
                elif isinstance(item, dict):
                    file_item = FileListItem.from_dict(item)
                    if file_item.index is None:
                        file_item.index = idx
                    val.append(FileItemViewModel(file_item))
                else:
                    raise ValueError(
                        f"Cannot create file item from: {json.dumps(item)}"
                    )

            # Update tile data with resolved values
            tile.data[nodeid] = [v.get_value().to_dict() for v in val]

        return FileListViewModel(*val)

    async def __asTileData(self) -> Optional[List[Dict[str, Any]]]:
        """Convert to tile data format."""
        return await self.for_json()


class FileListDataType:
    """
    Factory that creates FileListViewModel.

    This is registered with alizarin's CUSTOM_DATATYPES for 'file-list'.
    """

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
        cache_entry: Optional[Dict[str, Any]] = None,
    ) -> Optional[FileListViewModel]:
        """
        Create a FileListViewModel.
        """
        return await FileListViewModel._create(tile, node, value, cache_entry)


__all__ = [
    "FileItemViewModel",
    "FileListViewModel",
    "FileListDataType",
]
