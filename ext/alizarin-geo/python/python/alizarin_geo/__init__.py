"""
Alizarin Geo Extension

Provides validation for the ``geojson-feature-collection`` datatype. Coercion is
left to alizarin's core; this extension registers a *validate-only* handler that
the host dispatches to when converting trees with strict/validation enabled.

Usage:
    Simply import this module to register the handler:

    >>> import alizarin_geo

    The handler is registered with alizarin via the C ABI capsule, guarded by
    the extension ABI handshake (so a mismatched core/ext pairing refuses to
    load rather than corrupting memory).
"""

from __future__ import annotations

__version__ = "0.1.0"


def _register_rust_handler() -> bool:
    """
    Register the Rust validation handler with alizarin.

    Returns True if successful, False if the Rust extension or alizarin's
    registration hook is unavailable.
    """
    try:
        from . import _rust as rust_ext

        import alizarin
        if not hasattr(alizarin, "register_type_handler") or alizarin.register_type_handler is None:
            return False

        capsule = rust_ext.get_geo_handler_capsule()
        alizarin.register_type_handler(capsule)
        return True
    except ImportError:
        # Rust extension not built yet - this is fine.
        return False
    except Exception as e:
        raise RuntimeError(f"Failed to register geo Rust handler: {e}") from e


def set_coord_limit(limit: "int | None") -> None:
    """
    Set the maximum coordinate count per feature collection, or ``None`` to
    disable the check. Defaults to 1500, matching Arches' Elasticsearch limit.
    """
    from . import _rust as rust_ext
    rust_ext.set_coord_limit(limit)


def get_coord_limit() -> "int | None":
    """Get the current coordinate limit, or ``None`` if the check is disabled."""
    from . import _rust as rust_ext
    return rust_ext.get_coord_limit()


def reset_coord_limit() -> None:
    """Reset the coordinate limit to the default (1500)."""
    from . import _rust as rust_ext
    rust_ext.reset_coord_limit()


# Auto-register on import.
_rust_available = _register_rust_handler()


__all__ = [
    "__version__",
    "set_coord_limit",
    "get_coord_limit",
    "reset_coord_limit",
]
