"""
Node config for the ``reference`` datatype.

Core alizarin models only its own datatypes' configs; CLM supplies this one and
registers it with ``alizarin.register_node_config_adapter`` on import.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Optional

from alizarin.node_config import INodeConfig


@dataclass
class StaticNodeConfigReference(INodeConfig):
    """Configuration for reference nodes (matches the TypeScript adapter)."""

    controlledList: str = ""
    rdmCollection: str = ""
    multi_value: bool = False

    def get_collection_id(self) -> Optional[str]:
        """The collection id a lookup should use (controlledList, else rdmCollection)."""
        return self.controlledList or self.rdmCollection or None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticNodeConfigReference:
        """Create from a node's raw config."""
        return cls(
            controlledList=data.get("controlledList") or "",
            rdmCollection=data.get("rdmCollection") or "",
            multi_value=bool(data.get("multiValue", False)),
        )
