"""
Static types for CLM (Controlled List Manager) references.

These types match the TypeScript StaticReference and StaticReferenceLabel types.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional


@dataclass
class StaticReferenceLabel:
    """
    A label for a reference item.

    Matches TypeScript StaticReferenceLabel.
    """
    id: str
    language_id: str
    list_item_id: str
    value: str
    valuetype_id: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticReferenceLabel:
        """Create from dictionary."""
        return cls(
            id=data.get("id", ""),
            language_id=data.get("language_id", ""),
            list_item_id=data.get("list_item_id", ""),
            value=data.get("value", ""),
            valuetype_id=data.get("valuetype_id", ""),
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "language_id": self.language_id,
            "list_item_id": self.list_item_id,
            "value": self.value,
            "valuetype_id": self.valuetype_id,
        }


@dataclass
class StaticReference:
    """
    A reference to an item in a controlled list.

    Matches TypeScript StaticReference.
    """
    list_id: str
    uri: str
    labels: List[StaticReferenceLabel] = field(default_factory=list)

    def __str__(self) -> str:
        """Get display string for the reference."""
        return self.to_display_string()

    def to_display_string(self, lang: Optional[str] = None) -> str:
        """
        Get the display string for this reference.

        Args:
            lang: Language code (defaults to "en")

        Returns:
            The preferred label value
        """
        if len(self.labels) == 1:
            return self.labels[0].value

        target_lang = lang or "en"
        pref_label: Optional[str] = None

        for label in self.labels:
            if label.valuetype_id == "prefLabel":
                pref_label = label.value
                if label.language_id == target_lang:
                    return label.value

        return pref_label or "(undefined)"

    def lang(self, language: str) -> Optional[str]:
        """Get the label in a specific language."""
        for label in self.labels:
            if label.language_id == language and label.valuetype_id == "prefLabel":
                return label.value
        return None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticReference:
        """Create from dictionary."""
        labels_data = data.get("labels", [])
        labels = []
        for label_data in labels_data:
            if isinstance(label_data, StaticReferenceLabel):
                labels.append(label_data)
            elif isinstance(label_data, dict):
                labels.append(StaticReferenceLabel.from_dict(label_data))

        return cls(
            list_id=data.get("list_id", ""),
            uri=data.get("uri", ""),
            labels=labels,
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "list_id": self.list_id,
            "uri": self.uri,
            "labels": [label.to_dict() for label in self.labels],
        }


__all__ = [
    "StaticReferenceLabel",
    "StaticReference",
]
