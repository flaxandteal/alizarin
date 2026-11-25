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

/// Accepts a graph-metadata count field as EITHER a number OR the collection it
/// counts. The Arches graph-list endpoint reports `cards`/`nodes`/`edges`/etc. as
/// integer counts, but a FULL graph export carries them as arrays (or objects).
/// This lets a caller hand a full `StaticGraph` straight in as its own metadata
/// entry (C10): an array/object collapses to its length, a number passes through,
/// and null/absent becomes `None` — instead of the opaque "expected u32" error.
pub mod count_or_collection {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde_json::Value;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<Value>::deserialize(deserializer)? {
            None | Some(Value::Null) => None,
            Some(Value::Number(n)) => n.as_u64().map(|u| u as u32),
            Some(Value::Array(a)) => Some(a.len() as u32),
            Some(Value::Object(o)) => Some(o.len() as u32),
            // A string or bool here is meaningless as a count; treat as absent
            // rather than failing the whole metadata parse.
            Some(_) => None,
        })
    }

    pub fn serialize<S>(value: &Option<u32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(n) => serializer.serialize_u32(*n),
            None => serializer.serialize_none(),
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
