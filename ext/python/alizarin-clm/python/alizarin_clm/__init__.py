"""
Alizarin CLM Extension

This extension provides the "reference" datatype for Controlled List Manager integration.

Usage:
    Simply import this module to register the reference datatype:

    >>> import alizarin_clm

    The reference datatype will be automatically registered with alizarin's
    CUSTOM_DATATYPES registry.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

# Import static types
from .static_types import StaticReference, StaticReferenceLabel

# Import view models
from .view_models import (
    ReferenceValueViewModel,
    ReferenceListViewModel,
    ReferenceMergedDataType,
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
        capsule = rust_ext.get_reference_handler_capsule()
        alizarin.register_type_handler(capsule)
        return True
    except ImportError:
        # Rust extension not built yet - this is fine
        return False
    except Exception as e:
        print(f"Warning: Failed to register CLM Rust handler: {e}")
        return False


def _register_python_handler() -> None:
    """
    Register the Python ViewModel with alizarin's CUSTOM_DATATYPES.

    This is always done, regardless of whether Rust handler is available.
    """
    try:
        from alizarin.view_models import CUSTOM_DATATYPES
        CUSTOM_DATATYPES["reference"] = ReferenceMergedDataType
    except ImportError as e:
        print(f"Warning: Could not register CLM Python handler: {e}")


# Auto-register on import
_rust_available = _register_rust_handler()
_register_python_handler()


__all__ = [
    # Version
    "__version__",
    # Static types
    "StaticReference",
    "StaticReferenceLabel",
    # View models
    "ReferenceValueViewModel",
    "ReferenceListViewModel",
    "ReferenceMergedDataType",
]
