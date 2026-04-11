//! Shared serde helpers for graph types.
//!
//! These utilities allow fields to accept either a single string or an array
//! of strings on deserialize, while always exposing a `Vec<String>` internally.
//! On serialize, single-element vectors are emitted as a plain string for
//! round-trip compatibility with upstream Arches (which only supports a single
//! value for these fields).

/// Accepts `null`, a single string, or an array of strings on deserialize.
/// Normalises empty lists to `None`. Serialises `Some(vec![x])` as a plain
/// string and `Some(vec![x, y, ...])` as an array.
pub mod optional_string_or_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Form {
            Single(String),
            Multi(Vec<String>),
        }

        let parsed = Option::<Form>::deserialize(deserializer)?;
        Ok(match parsed {
            None => None,
            Some(Form::Single(s)) => Some(vec![s]),
            Some(Form::Multi(v)) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            }
        })
    }

    pub fn serialize<S>(value: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(list) if list.is_empty() => serializer.serialize_none(),
            Some(list) if list.len() == 1 => list[0].serialize(serializer),
            Some(list) => list.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrapper {
        #[serde(default, with = "super::optional_string_or_vec")]
        value: Option<Vec<String>>,
    }

    #[test]
    fn deserialize_null() {
        let v: Wrapper = serde_json::from_str(r#"{"value": null}"#).unwrap();
        assert_eq!(v.value, None);
    }

    #[test]
    fn deserialize_missing() {
        let v: Wrapper = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(v.value, None);
    }

    #[test]
    fn deserialize_single_string() {
        let v: Wrapper = serde_json::from_str(r#"{"value": "foo"}"#).unwrap();
        assert_eq!(v.value, Some(vec!["foo".to_string()]));
    }

    #[test]
    fn deserialize_array() {
        let v: Wrapper = serde_json::from_str(r#"{"value": ["foo", "bar"]}"#).unwrap();
        assert_eq!(v.value, Some(vec!["foo".to_string(), "bar".to_string()]));
    }

    #[test]
    fn deserialize_empty_array_becomes_none() {
        let v: Wrapper = serde_json::from_str(r#"{"value": []}"#).unwrap();
        assert_eq!(v.value, None);
    }

    #[test]
    fn serialize_none_is_null() {
        let w = Wrapper { value: None };
        assert_eq!(serde_json::to_string(&w).unwrap(), r#"{"value":null}"#);
    }

    #[test]
    fn serialize_single_as_string() {
        let w = Wrapper {
            value: Some(vec!["foo".to_string()]),
        };
        assert_eq!(serde_json::to_string(&w).unwrap(), r#"{"value":"foo"}"#);
    }

    #[test]
    fn serialize_multi_as_array() {
        let w = Wrapper {
            value: Some(vec!["foo".to_string(), "bar".to_string()]),
        };
        assert_eq!(
            serde_json::to_string(&w).unwrap(),
            r#"{"value":["foo","bar"]}"#
        );
    }
}
