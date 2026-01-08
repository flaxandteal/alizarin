"""
Alizarin FileList Extension

This extension provides the "file-list" datatype for file attachments in Arches.

Usage:
    Simply import this module to register the file-list datatype:

    >>> import alizarin_filelist

    The file-list datatype will be automatically registered with alizarin's
    CUSTOM_DATATYPES registry.

Working with Files:
    >>> from alizarin_filelist import FileListItem, FileListViewModel
    >>>
    >>> # Create a file item
    >>> file = FileListItem(
    ...     name="photo.jpg",
    ...     file_type="image/jpeg",
    ...     url="/files/abc123",
    ...     size=12345,
    ... )
    >>> print(file.to_display_string())  # "photo.jpg"
    >>> print(file.is_image())  # True
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

# Import static types
from .static_types import (
    LocalizedStringValue,
    LocalizedString,
    localized_string_from_dict,
    localized_string_to_dict,
    FileListItem,
)

# Import view models
from .view_models import (
    FileItemViewModel,
    FileListViewModel,
    FileListDataType,
)

__version__ = "0.1.0"


def _register_rust_handler() -> bool:
    """
    Register the Rust coercion handler with alizarin.

    Returns True if successful, False if Rust extension not available.
    """
    try:
        # Import the Rust extension
        from . import _rust as rust_ext

        # Import alizarin's registration function
        import alizarin
        if not hasattr(alizarin, 'register_type_handler') or alizarin.register_type_handler is None:
            # Rust extension functions not available
            return False

        # Get the capsule and register it
        capsule = rust_ext.get_filelist_handler_capsule()
        alizarin.register_type_handler(capsule)
        return True
    except ImportError:
        # Rust extension not built yet - this is fine
        return False
    except Exception as e:
        print(f"Warning: Failed to register FileList Rust handler: {e}")
        return False


def _register_python_handler() -> None:
    """
    Register the Python ViewModel with alizarin's CUSTOM_DATATYPES.

    This is always done, regardless of whether Rust handler is available.
    """
    try:
        from alizarin.view_models import CUSTOM_DATATYPES
        CUSTOM_DATATYPES["file-list"] = FileListDataType
    except ImportError as e:
        print(f"Warning: Could not register FileList Python handler: {e}")


# Auto-register on import
_rust_available = _register_rust_handler()
_register_python_handler()


__all__ = [
    # Version
    "__version__",
    # Static types
    "LocalizedStringValue",
    "LocalizedString",
    "localized_string_from_dict",
    "localized_string_to_dict",
    "FileListItem",
    # View models
    "FileItemViewModel",
    "FileListViewModel",
    "FileListDataType",
]
