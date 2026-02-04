"""
Static types for FileList datatype.

These types match the Arches file-list datatype structure.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional


@dataclass
class LocalizedStringValue:
    """
    A localized string with direction and value.

    Matches the Arches i18n string format.
    """
    direction: str = "ltr"
    value: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> LocalizedStringValue:
        """Create from dictionary."""
        if data is None:
            return cls()
        return cls(
            direction=data.get("direction", "ltr"),
            value=data.get("value", ""),
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "direction": self.direction,
            "value": self.value,
        }


# Type alias for localized strings (language code -> LocalizedStringValue)
LocalizedString = Dict[str, LocalizedStringValue]


def localized_string_from_dict(data: Optional[Dict[str, Any]]) -> Optional[LocalizedString]:
    """Convert a dict to LocalizedString."""
    if data is None:
        return None
    result = {}
    for lang, val in data.items():
        if isinstance(val, dict):
            result[lang] = LocalizedStringValue.from_dict(val)
        elif isinstance(val, LocalizedStringValue):
            result[lang] = val
    return result if result else None


def localized_string_to_dict(ls: Optional[LocalizedString]) -> Optional[Dict[str, Any]]:
    """Convert LocalizedString to dict."""
    if ls is None:
        return None
    return {lang: val.to_dict() for lang, val in ls.items()}


@dataclass
class FileListItem:
    """
    A single file in a file-list.

    Matches the Arches file-list datatype structure.
    """
    # Required fields
    name: str = ""

    # Common optional fields
    accepted: bool = False
    alt_text: Optional[LocalizedString] = None
    attribution: Optional[LocalizedString] = None
    content: Optional[str] = None
    description: Optional[LocalizedString] = None
    file_id: Optional[str] = None
    index: Optional[int] = None
    last_modified: Optional[int] = None
    path: Optional[str] = None
    selected: bool = False
    size: Optional[int] = None
    status: Optional[str] = None
    title: Optional[LocalizedString] = None
    file_type: Optional[str] = None
    url: Optional[str] = None
    renderer: Optional[str] = None

    # Extra fields we might not know about
    extra: Dict[str, Any] = field(default_factory=dict)

    def __str__(self) -> str:
        """Get display string for the file."""
        return self.to_display_string()

    def to_display_string(self, lang: Optional[str] = None) -> str:
        """
        Get the display string for this file.

        Uses title if available (in specified language), otherwise falls back to filename.

        Args:
            lang: Language code (defaults to "en")

        Returns:
            The display string for this file
        """
        target_lang = lang or "en"

        # Try title first
        if self.title:
            if target_lang in self.title:
                val = self.title[target_lang]
                if val.value:
                    return val.value
            # Try any language
            for localized in self.title.values():
                if localized.value:
                    return localized.value

        # Fall back to filename
        if self.name:
            return self.name

        # Last resort
        return self.file_id or "(unnamed file)"

    def get_alt_text(self, lang: Optional[str] = None) -> Optional[str]:
        """Get the alt text in a specific language."""
        target_lang = lang or "en"

        if self.alt_text:
            if target_lang in self.alt_text:
                val = self.alt_text[target_lang]
                if val.value:
                    return val.value
        return None

    def is_image(self) -> bool:
        """Check if this is an image file based on MIME type."""
        if self.file_type:
            return self.file_type.startswith("image/")
        return False

    def lang(self, language: str) -> Optional[str]:
        """Get the title in a specific language."""
        if self.title and language in self.title:
            return self.title[language].value
        return None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> FileListItem:
        """Create from dictionary."""
        # Known fields to extract
        known_fields = {
            'accepted', 'alt_text', 'attribution', 'content',
            'description', 'file_id', 'index', 'last_modified', 'last_modified',
            'name', 'path', 'selected', 'size', 'status', 'title', 'type',
            'file_type', 'url', 'renderer'
        }

        # Collect extra fields
        extra = {k: v for k, v in data.items() if k not in known_fields}

        return cls(
            name=data.get("name", ""),
            accepted=data.get("accepted", False),
            alt_text=localized_string_from_dict(data.get("alt_text")),
            attribution=localized_string_from_dict(data.get("attribution")),
            content=data.get("content"),
            description=localized_string_from_dict(data.get("description")),
            file_id=data.get("file_id"),
            index=data.get("index"),
            last_modified=data.get("last_modified"),
            path=data.get("path"),
            selected=data.get("selected", False),
            size=data.get("size"),
            status=data.get("status"),
            title=localized_string_from_dict(data.get("title")),
            file_type=data.get("type") or data.get("file_type"),
            url=data.get("url"),
            renderer=data.get("renderer"),
            extra=extra,
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary (Arches tile data format)."""
        result: Dict[str, Any] = {
            "name": self.name,
            "accepted": self.accepted,
            "selected": self.selected,
        }

        # Add optional fields if present
        if self.alt_text is not None:
            result["alt_text"] = localized_string_to_dict(self.alt_text)
        if self.attribution is not None:
            result["attribution"] = localized_string_to_dict(self.attribution)
        if self.content is not None:
            result["content"] = self.content
        if self.description is not None:
            result["description"] = localized_string_to_dict(self.description)
        if self.file_id is not None:
            result["file_id"] = self.file_id
        if self.index is not None:
            result["index"] = self.index
        if self.last_modified is not None:
            result["last_modified"] = self.last_modified
        if self.path is not None:
            result["path"] = self.path
        if self.size is not None:
            result["size"] = self.size
        if self.status is not None:
            result["status"] = self.status
        if self.title is not None:
            result["title"] = localized_string_to_dict(self.title)
        if self.file_type is not None:
            result["type"] = self.file_type
        if self.url is not None:
            result["url"] = self.url
        if self.renderer is not None:
            result["renderer"] = self.renderer

        # Add extra fields
        result.update(self.extra)

        return result


__all__ = [
    "LocalizedStringValue",
    "LocalizedString",
    "localized_string_from_dict",
    "localized_string_to_dict",
    "FileListItem",
]
