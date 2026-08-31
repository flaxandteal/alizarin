"""
Alizarin PG Extension

PostgreSQL COPY format output for bulk-loading Alizarin resources.

Usage:
    >>> from alizarin_pg import batch_trees_to_pg_copy
    >>> result = batch_trees_to_pg_copy(trees_json, graph_id)
    >>> # result["tiles_copy"] and result["resources_copy"] are COPY-ready text
"""

from alizarin_pg._rust import batch_trees_to_pg_copy

__all__ = ["batch_trees_to_pg_copy"]
