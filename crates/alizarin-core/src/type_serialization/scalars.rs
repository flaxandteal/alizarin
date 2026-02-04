//! Scalar serialization: numbers, dates, EDTF.
//!
//! These types have no display transformation - they pass through unchanged.

use super::options::{SerializationOptions, SerializationResult};
use serde_json::Value;

/// Serialize a number value (pass-through).
pub fn serialize_number(tile_data: &Value, _options: &SerializationOptions) -> SerializationResult {
    SerializationResult::success(tile_data.clone())
}

/// Serialize a date value (pass-through).
pub fn serialize_date(tile_data: &Value, _options: &SerializationOptions) -> SerializationResult {
    SerializationResult::success(tile_data.clone())
}

/// Serialize an EDTF date value (pass-through).
pub fn serialize_edtf(tile_data: &Value, _options: &SerializationOptions) -> SerializationResult {
    SerializationResult::success(tile_data.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_number() {
        let tile_data = json!(42);
        let options = SerializationOptions::display("en");

        let result = serialize_number(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(42));
    }

    #[test]
    fn test_serialize_number_float() {
        let tile_data = json!(4.2);
        let options = SerializationOptions::tile_data();

        let result = serialize_number(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!(4.2));
    }

    #[test]
    fn test_serialize_date() {
        let tile_data = json!("2023-05-15");
        let options = SerializationOptions::display("en");

        let result = serialize_date(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("2023-05-15"));
    }

    #[test]
    fn test_serialize_edtf() {
        let tile_data = json!("2023/2024");
        let options = SerializationOptions::display("en");

        let result = serialize_edtf(&tile_data, &options);
        assert!(!result.is_error());
        assert_eq!(result.value, json!("2023/2024"));
    }

    #[test]
    fn test_serialize_null() {
        let tile_data = json!(null);
        let options = SerializationOptions::tile_data();

        let result = serialize_number(&tile_data, &options);
        assert!(!result.is_error());
        assert!(result.value.is_null());
    }
}
