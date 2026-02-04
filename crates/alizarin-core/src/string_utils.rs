//! String utility functions.
//!
//! Platform-agnostic string transformation utilities used by both
//! WASM and Python bindings.

use serde_json::Value;

/// Convert a camelCase string to snake_case.
///
/// Converts uppercase letters to lowercase with underscore prefix.
/// Note: Consecutive uppercase letters (acronyms) are converted individually,
/// e.g., "XMLParser" becomes "x_m_l_parser".
///
/// # Examples
///
/// ```
/// use alizarin_core::string_utils::camel_to_snake;
///
/// assert_eq!(camel_to_snake("firstName"), "first_name");
/// assert_eq!(camel_to_snake("already_snake"), "already_snake");
/// ```
pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 10);
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            // Don't add underscore at the start
            if !result.is_empty() {
                // Check if we're in an acronym (current is upper, next is upper or end)
                let next_is_upper_or_end = chars.peek().map(|n| n.is_uppercase()).unwrap_or(true);
                let prev_was_upper = result.chars().last().map(|p| p.is_uppercase()).unwrap_or(false);

                // Add underscore if:
                // - Previous was lowercase, or
                // - We're at the end of an acronym (next is lowercase)
                if !prev_was_upper || !next_is_upper_or_end {
                    result.push('_');
                }
            }
            // SAFETY: to_lowercase() always yields at least one char for any valid Unicode char
            result.push(c.to_lowercase().next().expect("to_lowercase() always yields at least one char"));
        } else {
            result.push(c);
        }
    }

    result
}

/// Recursively transform all keys in a JSON value from camelCase to snake_case.
///
/// This is useful for converting JavaScript-style JSON to Python-style JSON
/// during ETL operations.
///
/// # Examples
///
/// ```
/// use alizarin_core::string_utils::transform_keys_to_snake;
/// use serde_json::json;
///
/// let input = json!({
///     "firstName": "John",
///     "contactInfo": {
///         "emailAddress": "john@example.com"
///     }
/// });
///
/// let output = transform_keys_to_snake(input);
///
/// assert_eq!(output["first_name"], "John");
/// assert_eq!(output["contact_info"]["email_address"], "john@example.com");
/// ```
pub fn transform_keys_to_snake(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                let new_key = camel_to_snake(&key);
                let new_val = transform_keys_to_snake(val);
                new_map.insert(new_key, new_val);
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(transform_keys_to_snake).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_camel_to_snake_simple() {
        assert_eq!(camel_to_snake("firstName"), "first_name");
        assert_eq!(camel_to_snake("lastName"), "last_name");
        assert_eq!(camel_to_snake("emailAddress"), "email_address");
    }

    #[test]
    fn test_camel_to_snake_single_word() {
        assert_eq!(camel_to_snake("name"), "name");
        assert_eq!(camel_to_snake("id"), "id");
    }

    #[test]
    fn test_camel_to_snake_already_snake() {
        assert_eq!(camel_to_snake("first_name"), "first_name");
        assert_eq!(camel_to_snake("email_address"), "email_address");
    }

    #[test]
    fn test_camel_to_snake_acronyms() {
        // Note: consecutive uppercase letters are treated individually
        // This matches the behavior used in both WASM and Python
        assert_eq!(camel_to_snake("XMLParser"), "x_m_l_parser");
        assert_eq!(camel_to_snake("getHTTPResponse"), "get_h_t_t_p_response");
        assert_eq!(camel_to_snake("parseJSON"), "parse_j_s_o_n");
    }

    #[test]
    fn test_camel_to_snake_mixed() {
        // Consecutive uppercase letters are treated individually
        assert_eq!(camel_to_snake("getURLForID"), "get_u_r_l_for_i_d");
    }

    #[test]
    fn test_transform_keys_nested() {
        let input = json!({
            "firstName": "John",
            "lastName": "Doe",
            "contactInfo": {
                "emailAddress": "john@example.com",
                "phoneNumber": "123-456-7890"
            },
            "addresses": [
                {
                    "streetName": "Main St",
                    "zipCode": "12345"
                }
            ]
        });

        let result = transform_keys_to_snake(input);

        assert_eq!(result["first_name"], "John");
        assert_eq!(result["last_name"], "Doe");
        assert_eq!(result["contact_info"]["email_address"], "john@example.com");
        assert_eq!(result["contact_info"]["phone_number"], "123-456-7890");
        assert_eq!(result["addresses"][0]["street_name"], "Main St");
        assert_eq!(result["addresses"][0]["zip_code"], "12345");
    }

    #[test]
    fn test_transform_keys_preserves_values() {
        let input = json!({
            "camelKey": "camelValue",
            "numberKey": 42,
            "boolKey": true,
            "nullKey": null
        });

        let result = transform_keys_to_snake(input);

        assert_eq!(result["camel_key"], "camelValue"); // Value unchanged
        assert_eq!(result["number_key"], 42);
        assert_eq!(result["bool_key"], true);
        assert!(result["null_key"].is_null());
    }
}
