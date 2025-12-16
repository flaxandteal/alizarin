"""Tests for type coercion functions, especially domain values."""

import pytest
from alizarin.alizarin import (
    coerce_value,
    coerce_domain_value,
    coerce_domain_value_list,
    CoercionResult,
)


class TestDomainValueCoercion:
    """Tests for domain-value coercion."""

    def test_coerce_domain_value_uuid_no_config(self):
        """Should coerce a valid UUID without config."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        result = coerce_domain_value(uuid, None)

        assert not result.is_error
        assert result.tile_data == uuid

    def test_coerce_domain_value_with_config(self):
        """Should coerce UUID and resolve display value from config."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        config = {
            "options": [
                {"id": uuid, "text": {"en": "Option 1"}, "selected": False}
            ]
        }
        result = coerce_domain_value(uuid, config)

        assert not result.is_error
        assert result.tile_data == uuid
        assert result.display_value["id"] == uuid
        assert result.display_value["text"]["en"] == "Option 1"

    def test_coerce_domain_value_not_found_in_config(self):
        """Should error when UUID not found in config options."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        other_uuid = "660e8400-e29b-41d4-a716-446655440001"
        config = {
            "options": [
                {"id": other_uuid, "text": {"en": "Other"}, "selected": False}
            ]
        }
        result = coerce_domain_value(uuid, config)

        assert result.is_error
        assert "not found" in result.error.lower()

    def test_coerce_domain_value_object_input(self):
        """Should extract id from domain value object."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        input_obj = {"id": uuid, "text": {"en": "Already resolved"}}
        result = coerce_domain_value(input_obj, None)

        assert not result.is_error
        assert result.tile_data == uuid

    def test_coerce_domain_value_null(self):
        """Should handle null input."""
        result = coerce_domain_value(None, None)

        assert result.is_null
        assert not result.is_error

    def test_coerce_domain_value_empty_string(self):
        """Should treat empty string as null."""
        result = coerce_domain_value("", None)

        assert result.is_null
        assert not result.is_error

    def test_coerce_domain_value_invalid_uuid(self):
        """Should error on invalid UUID format."""
        result = coerce_domain_value("not-a-uuid", None)

        assert result.is_error
        assert "uuid" in result.error.lower()

    def test_coerce_domain_value_number_input(self):
        """Should error on number input."""
        result = coerce_domain_value(12345, None)

        assert result.is_error


class TestDomainValueListCoercion:
    """Tests for domain-value-list coercion."""

    def test_coerce_domain_value_list_uuids(self):
        """Should coerce an array of UUIDs."""
        uuid1 = "550e8400-e29b-41d4-a716-446655440000"
        uuid2 = "660e8400-e29b-41d4-a716-446655440001"
        result = coerce_domain_value_list([uuid1, uuid2], None)

        assert not result.is_error
        assert result.tile_data == [uuid1, uuid2]

    def test_coerce_domain_value_list_with_config(self):
        """Should resolve display values from config."""
        uuid1 = "550e8400-e29b-41d4-a716-446655440000"
        uuid2 = "660e8400-e29b-41d4-a716-446655440001"
        config = {
            "options": [
                {"id": uuid1, "text": {"en": "Option A"}, "selected": False},
                {"id": uuid2, "text": {"en": "Option B"}, "selected": False},
            ]
        }
        result = coerce_domain_value_list([uuid1, uuid2], config)

        assert not result.is_error
        assert result.tile_data == [uuid1, uuid2]
        assert len(result.display_value) == 2
        assert result.display_value[0]["text"]["en"] == "Option A"
        assert result.display_value[1]["text"]["en"] == "Option B"

    def test_coerce_domain_value_list_empty(self):
        """Should handle empty array."""
        result = coerce_domain_value_list([], None)

        assert not result.is_error
        assert result.tile_data == []

    def test_coerce_domain_value_list_null(self):
        """Should handle null input."""
        result = coerce_domain_value_list(None, None)

        assert result.is_null
        assert not result.is_error

    def test_coerce_domain_value_list_invalid_uuid(self):
        """Should error on invalid UUID in array."""
        uuid1 = "550e8400-e29b-41d4-a716-446655440000"
        result = coerce_domain_value_list([uuid1, "not-a-uuid"], None)

        assert result.is_error

    def test_coerce_domain_value_list_single_uuid_error(self):
        """Should error on single UUID (requires array)."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        result = coerce_domain_value_list(uuid, None)

        assert result.is_error

    def test_coerce_domain_value_list_objects(self):
        """Should coerce array of domain value objects."""
        uuid1 = "550e8400-e29b-41d4-a716-446655440000"
        uuid2 = "660e8400-e29b-41d4-a716-446655440001"
        input_list = [
            {"id": uuid1, "text": {"en": "A"}},
            {"id": uuid2, "text": {"en": "B"}},
        ]
        result = coerce_domain_value_list(input_list, None)

        assert not result.is_error
        assert result.tile_data == [uuid1, uuid2]


class TestDomainValueDispatcher:
    """Tests using the coerce_value dispatcher."""

    def test_coerce_value_domain_value(self):
        """Should dispatch to domain-value coercion."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        result = coerce_value("domain-value", uuid, None)

        assert not result.is_error
        assert result.tile_data == uuid

    def test_coerce_value_domain_value_list(self):
        """Should dispatch to domain-value-list coercion."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        result = coerce_value("domain-value-list", [uuid], None)

        assert not result.is_error
        assert result.tile_data == [uuid]


class TestDomainValueRoundTrip:
    """Round-trip tests: coerce → tile_data → coerce again."""

    def test_domain_value_round_trip(self):
        """UUID should round-trip through coercion."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"

        # First coercion
        result1 = coerce_domain_value(uuid, None)
        assert not result1.is_error

        # Round-trip: coerce the tile_data again
        result2 = coerce_domain_value(result1.tile_data, None)
        assert not result2.is_error
        assert result2.tile_data == result1.tile_data

    def test_domain_value_round_trip_with_config(self):
        """UUID with config should round-trip with consistent display values."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        config = {
            "options": [
                {"id": uuid, "text": {"en": "Test Option"}, "selected": False}
            ]
        }

        # First coercion
        result1 = coerce_domain_value(uuid, config)
        assert not result1.is_error
        assert result1.display_value["text"]["en"] == "Test Option"

        # Round-trip
        result2 = coerce_domain_value(result1.tile_data, config)
        assert not result2.is_error
        assert result2.tile_data == result1.tile_data
        assert result2.display_value == result1.display_value

    def test_domain_value_round_trip_from_object(self):
        """Object input should round-trip as UUID."""
        uuid = "550e8400-e29b-41d4-a716-446655440000"
        input_obj = {"id": uuid, "text": {"en": "Already Resolved"}}

        # First coercion: Object → UUID
        result1 = coerce_domain_value(input_obj, None)
        assert not result1.is_error
        assert result1.tile_data == uuid

        # Round-trip: UUID → UUID
        result2 = coerce_domain_value(result1.tile_data, None)
        assert not result2.is_error
        assert result2.tile_data == uuid

    def test_domain_value_list_round_trip(self):
        """Array of UUIDs should round-trip."""
        uuid1 = "550e8400-e29b-41d4-a716-446655440000"
        uuid2 = "660e8400-e29b-41d4-a716-446655440001"

        # First coercion
        result1 = coerce_domain_value_list([uuid1, uuid2], None)
        assert not result1.is_error

        # Round-trip
        result2 = coerce_domain_value_list(result1.tile_data, None)
        assert not result2.is_error
        assert result2.tile_data == result1.tile_data

    def test_domain_value_list_round_trip_with_config(self):
        """Array with config should round-trip with display values."""
        uuid1 = "550e8400-e29b-41d4-a716-446655440000"
        uuid2 = "660e8400-e29b-41d4-a716-446655440001"
        config = {
            "options": [
                {"id": uuid1, "text": {"en": "A"}, "selected": False},
                {"id": uuid2, "text": {"en": "B"}, "selected": False},
            ]
        }

        # First coercion
        result1 = coerce_domain_value_list([uuid1, uuid2], config)
        assert not result1.is_error

        # Round-trip
        result2 = coerce_domain_value_list(result1.tile_data, config)
        assert not result2.is_error
        assert result2.tile_data == result1.tile_data
        assert result2.display_value == result1.display_value

    def test_domain_value_list_round_trip_from_objects(self):
        """Array of objects should round-trip as UUIDs."""
        uuid1 = "550e8400-e29b-41d4-a716-446655440000"
        uuid2 = "660e8400-e29b-41d4-a716-446655440001"
        input_list = [
            {"id": uuid1, "text": {"en": "A"}},
            {"id": uuid2, "text": {"en": "B"}},
        ]

        # First coercion: Objects → UUIDs
        result1 = coerce_domain_value_list(input_list, None)
        assert not result1.is_error
        assert result1.tile_data == [uuid1, uuid2]

        # Round-trip: UUIDs → UUIDs
        result2 = coerce_domain_value_list(result1.tile_data, None)
        assert not result2.is_error
        assert result2.tile_data == [uuid1, uuid2]

    def test_domain_value_empty_round_trip(self):
        """Empty string should round-trip as null."""
        result1 = coerce_domain_value("", None)
        assert result1.is_null

        # Null stays null
        result2 = coerce_domain_value(result1.tile_data, None)
        assert result2.is_null

    def test_domain_value_list_empty_round_trip(self):
        """Empty array should round-trip as empty array."""
        result1 = coerce_domain_value_list([], None)
        assert not result1.is_error
        assert result1.tile_data == []

        result2 = coerce_domain_value_list(result1.tile_data, None)
        assert not result2.is_error
        assert result2.tile_data == []
