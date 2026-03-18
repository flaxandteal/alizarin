//! Simple scalar type coercion: Number, NonLocalizedString, EDTF, Date.

use super::helpers::value_type_name;
use super::result::CoercionResult;
use serde_json::Value;

/// Coerce a value to a number.
///
/// Accepts: number, string (parseable as number), null
/// Returns: the numeric value or null
pub fn coerce_number(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::Number(n) => CoercionResult::success_same(Value::Number(n.clone())),
        Value::String(s) if s.is_empty() => CoercionResult::success_same(Value::Null),
        Value::String(s) => {
            // Try to parse as number
            if let Ok(n) = s.parse::<i64>() {
                CoercionResult::success_same(Value::Number(n.into()))
            } else if let Ok(n) = s.parse::<f64>() {
                match serde_json::Number::from_f64(n) {
                    Some(num) => CoercionResult::success_same(Value::Number(num)),
                    None => {
                        CoercionResult::error(format!("Invalid number: {} (NaN or Infinity)", s))
                    }
                }
            } else {
                CoercionResult::error(format!("Cannot parse '{}' as number", s))
            }
        }
        _ => CoercionResult::error(format!(
            "Expected number or numeric string, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Coerce a value to a non-localized string.
///
/// Accepts: string, null
/// Returns: the string value or null
pub fn coerce_non_localized_string(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => CoercionResult::success_same(Value::Null),
        Value::String(_) => CoercionResult::success_same(value.clone()),
        Value::Number(n) => {
            // Convert number to string
            CoercionResult::success_same(Value::String(n.to_string()))
        }
        Value::Bool(b) => {
            // Convert bool to string
            CoercionResult::success_same(Value::String(b.to_string()))
        }
        _ => CoercionResult::error(format!("Expected string, got {:?}", value_type_name(value))),
    }
}

/// Coerce a value to an EDTF string.
///
/// EDTF is a string format, so this is similar to non-localized string
/// but could include validation in the future.
///
/// Accepts: string (EDTF format), null
/// Returns: the EDTF string or null
pub fn coerce_edtf(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => CoercionResult::success_same(Value::Null),
        Value::String(s) => {
            // Basic EDTF validation could be added here
            // For now, accept any non-empty string
            // EDTF examples: "2023", "2023-05", "2023-05-15", "2023?", "2023~", etc.
            if is_valid_edtf(s) {
                CoercionResult::success_same(value.clone())
            } else {
                // Be permissive - accept the string but log warning
                // In strict mode this could return an error
                CoercionResult::success_same(value.clone())
            }
        }
        _ => CoercionResult::error(format!(
            "Expected EDTF string, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Basic EDTF validation.
///
/// This is a simplified check - full EDTF validation would be more complex.
/// Returns true for strings that look like EDTF dates.
fn is_valid_edtf(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // EDTF can be quite complex. Basic patterns:
    // - Year: "2023", "2023?", "2023~", "-0500"
    // - Year-Month: "2023-05"
    // - Full date: "2023-05-15"
    // - Intervals: "2023/2024", "2023-05/2023-06"
    // - Uncertain: "2023?", "2023~", "2023%"
    // - Approximate: "2023~"
    // - Unknown: "XXXX", "2023-XX"
    // - Season: "2023-21" (spring), "2023-22" (summer), etc.

    // Simple heuristic: must start with digit, negative sign, or X
    let first = s.chars().next().unwrap();
    first.is_ascii_digit() || first == '-' || first == 'X' || first == 'x'
}

/// Coerce a value to a date.
///
/// Accepts: ISO date string, null, object with 'en' key (legacy format)
/// Returns: ISO date string or null
pub fn coerce_date(value: &Value) -> CoercionResult {
    match value {
        Value::Null => CoercionResult::success_same(Value::Null),
        Value::String(s) if s.is_empty() => CoercionResult::success_same(Value::Null),
        Value::String(s) => {
            // Validate and normalize the date
            match normalize_date_string(s) {
                Ok(normalized) => {
                    let tile_val = Value::String(normalized.clone());
                    CoercionResult::success(tile_val.clone(), tile_val)
                }
                Err(_) => {
                    // Be permissive - accept the string but note the issue
                    // The JS Date constructor will handle parsing
                    CoercionResult::success_same(value.clone())
                }
            }
        }
        Value::Object(obj) => {
            // Handle legacy format: {"en": "2023-05-15"}
            if let Some(Value::String(s)) = obj.get("en") {
                if s.is_empty() {
                    CoercionResult::success_same(Value::Null)
                } else {
                    // Extract the 'en' value
                    match normalize_date_string(s) {
                        Ok(normalized) => {
                            let tile_val = Value::String(normalized.clone());
                            CoercionResult::success(tile_val.clone(), tile_val)
                        }
                        Err(_) => CoercionResult::success_same(Value::String(s.clone())),
                    }
                }
            } else {
                CoercionResult::error("Date object must have 'en' key with string value")
            }
        }
        _ => CoercionResult::error(format!(
            "Expected date string, got {:?}",
            value_type_name(value)
        )),
    }
}

/// Validate and pass through an ISO 8601 date string.
///
/// Accepts date-only, datetime, datetime with timezone, and datetime with
/// milliseconds. Returns the original string if valid (Arches stores as-is).
fn normalize_date_string(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty date string".to_string());
    }

    // Try parsing in order of specificity:
    // 1. Full datetime with timezone offset (2023-05-15T10:30:00+00:00, ...Z)
    if chrono::DateTime::parse_from_rfc3339(s).is_ok() {
        return Ok(s.to_string());
    }

    // 2. Datetime without timezone (2023-05-15T10:30:00, 2023-05-15T10:30:00.000)
    if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").is_ok() {
        return Ok(s.to_string());
    }

    // 3. Date only (2023-05-15)
    if chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
        return Ok(s.to_string());
    }

    Err(format!("Invalid date format: {}", s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Number tests
    #[test]
    fn test_coerce_number_from_number() {
        let result = coerce_number(&json!(42));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(42));
    }

    #[test]
    fn test_coerce_number_from_float() {
        let result = coerce_number(&json!(4.2));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(4.2));
    }

    #[test]
    fn test_coerce_number_from_string() {
        let result = coerce_number(&json!("42"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(42));
    }

    #[test]
    fn test_coerce_number_from_float_string() {
        let result = coerce_number(&json!("4.2"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!(4.2));
    }

    #[test]
    fn test_coerce_number_from_null() {
        let result = coerce_number(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_number_from_empty_string() {
        let result = coerce_number(&json!(""));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_number_invalid_string() {
        let result = coerce_number(&json!("not a number"));
        assert!(result.is_error());
    }

    // NonLocalizedString tests
    #[test]
    fn test_coerce_non_localized_string() {
        let result = coerce_non_localized_string(&json!("hello"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("hello"));
    }

    #[test]
    fn test_coerce_non_localized_string_null() {
        let result = coerce_non_localized_string(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_non_localized_string_empty() {
        let result = coerce_non_localized_string(&json!(""));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_non_localized_string_from_number() {
        let result = coerce_non_localized_string(&json!(42));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("42"));
    }

    // EDTF tests
    #[test]
    fn test_coerce_edtf_year() {
        let result = coerce_edtf(&json!("2023"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023"));
    }

    #[test]
    fn test_coerce_edtf_year_month() {
        let result = coerce_edtf(&json!("2023-05"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05"));
    }

    #[test]
    fn test_coerce_edtf_uncertain() {
        let result = coerce_edtf(&json!("2023?"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023?"));
    }

    #[test]
    fn test_coerce_edtf_null() {
        let result = coerce_edtf(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    // Date tests
    #[test]
    fn test_coerce_date_iso() {
        let result = coerce_date(&json!("2023-05-15"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05-15"));
    }

    #[test]
    fn test_coerce_date_iso_datetime() {
        let result = coerce_date(&json!("2023-05-15T10:30:00Z"));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05-15T10:30:00Z"));
    }

    #[test]
    fn test_coerce_date_legacy_object() {
        let result = coerce_date(&json!({"en": "2023-05-15"}));
        assert!(!result.is_error());
        assert_eq!(result.tile_data, json!("2023-05-15"));
    }

    #[test]
    fn test_coerce_date_null() {
        let result = coerce_date(&json!(null));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }

    #[test]
    fn test_coerce_date_empty_string() {
        let result = coerce_date(&json!(""));
        assert!(!result.is_error());
        assert!(result.tile_data.is_null());
    }
}
