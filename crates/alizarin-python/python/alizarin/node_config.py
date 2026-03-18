"""
Node configuration classes for typed node configs.

Matches TypeScript nodeConfig.ts
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, TYPE_CHECKING

if TYPE_CHECKING:
    from .static_types import StaticNode, StaticDomainValue


# =============================================================================
# Interfaces / Protocols
# =============================================================================


class INodeConfig:
    """Base interface for node configs."""
    pass


# =============================================================================
# Static Node Configs
# =============================================================================


@dataclass
class StaticNodeConfigBoolean(INodeConfig):
    """
    Configuration for boolean nodes.

    Matches TypeScript StaticNodeConfigBoolean.
    """
    i18n_properties: List[str] = field(default_factory=list)
    falseLabel: Dict[str, str] = field(default_factory=dict)
    trueLabel: Dict[str, str] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticNodeConfigBoolean:
        """Create from dictionary."""
        return cls(
            i18n_properties=data.get("i18n_properties", []),
            falseLabel=data.get("falseLabel", {}),
            trueLabel=data.get("trueLabel", {}),
        )


@dataclass
class StaticNodeConfigConcept(INodeConfig):
    """
    Configuration for concept nodes.

    Matches TypeScript StaticNodeConfigConcept.
    """
    rdmCollection: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticNodeConfigConcept:
        """Create from dictionary."""
        return cls(
            rdmCollection=data.get("rdmCollection", ""),
        )


@dataclass
class StaticNodeConfigReference(INodeConfig):
    """
    Configuration for reference nodes (CLM extension).

    Matches TypeScript node config for reference datatype.
    """
    controlledList: str = ""
    rdmCollection: str = ""
    multi_value: bool = False

    def get_collection_id(self) -> Optional[str]:
        """Get the collection ID (controlledList or rdmCollection)."""
        return self.controlledList or self.rdmCollection or None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticNodeConfigReference:
        """Create from dictionary."""
        return cls(
            controlledList=data.get("controlledList", ""),
            rdmCollection=data.get("rdmCollection", ""),
            multi_value=data.get("multiValue", False),
        )


@dataclass
class StaticNodeConfigDomain(INodeConfig):
    """
    Configuration for domain-value nodes.

    Matches TypeScript StaticNodeConfigDomain.
    """
    i18n_config: Dict[str, str] = field(default_factory=dict)
    options: List[StaticDomainValue] = field(default_factory=list)

    def get_selected(self) -> Optional[StaticDomainValue]:
        """Get the selected option."""
        for option in self.options:
            if option.selected:
                return option
        return None

    def value_from_id(self, id: str) -> Optional[StaticDomainValue]:
        """Find option by ID."""
        for option in self.options:
            if option.id == id:
                return option
        return None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticNodeConfigDomain:
        """Create from dictionary."""
        from .static_types import StaticDomainValue

        options_data = data.get("options", [])
        options = []
        for opt in options_data:
            if isinstance(opt, StaticDomainValue):
                options.append(opt)
            elif isinstance(opt, dict):
                options.append(StaticDomainValue.from_dict(opt))

        return cls(
            i18n_config=data.get("i18n_config", {}),
            options=options,
        )


# =============================================================================
# Node Config Manager
# =============================================================================


class NodeConfigManager:
    """
    Manager for caching and retrieving node configurations.

    Matches TypeScript NodeConfigManager.
    """
    _cache: Dict[str, Optional[INodeConfig]] = {}

    def __init__(self, cache: Optional[Dict[str, Optional[INodeConfig]]] = None):
        if cache is None:
            cache = NodeConfigManager._cache
        self.cache = cache

    def retrieve(self, node: StaticNode) -> Optional[INodeConfig]:
        """
        Retrieve configuration for a node.

        Args:
            node: The StaticNode to get config for

        Returns:
            Node configuration or None
        """
        if node.nodeid in self.cache:
            return self.cache[node.nodeid]

        node_config: Optional[INodeConfig] = None
        config_data = node.config or {}

        if node.datatype == "boolean":
            node_config = StaticNodeConfigBoolean.from_dict(config_data)
        elif node.datatype in ("domain-value-list", "domain-value"):
            node_config = StaticNodeConfigDomain.from_dict(config_data)
        elif node.datatype == "reference":
            node_config = StaticNodeConfigReference.from_dict(config_data)
        elif node.datatype in ("concept", "concept-list"):
            node_config = StaticNodeConfigConcept.from_dict(config_data)

        self.cache[node.nodeid] = node_config
        return node_config

    def clear_cache(self) -> None:
        """Clear the configuration cache."""
        self.cache.clear()


# =============================================================================
# Singleton Instance
# =============================================================================

nodeConfigManager = NodeConfigManager()


# =============================================================================
# Exports
# =============================================================================

__all__ = [
    "INodeConfig",
    "StaticNodeConfigBoolean",
    "StaticNodeConfigConcept",
    "StaticNodeConfigDomain",
    "StaticNodeConfigReference",
    "NodeConfigManager",
    "nodeConfigManager",
]
