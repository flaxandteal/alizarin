//! Serialize `HashMap` fields with their keys in sorted order.
//!
//! Public structs keep `HashMap` fields so their Rust API doesn't change, but
//! anything serialized from them (JSON files, NAPI/Python objects, WASM values)
//! must not depend on hash iteration order, which differs between processes on
//! native targets. Use with `#[serde(serialize_with = "...")]`; bindings that
//! return key lists use [`sorted_keys`].

use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

pub fn sorted_map<S, K, V>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Ord + Serialize,
    V: Serialize,
{
    map.iter().collect::<BTreeMap<_, _>>().serialize(serializer)
}

pub fn sorted_opt_map<S, K, V>(
    map: &Option<HashMap<K, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Ord + Serialize,
    V: Serialize,
{
    map.as_ref()
        .map(|m| m.iter().collect::<BTreeMap<_, _>>())
        .serialize(serializer)
}

/// A map's keys as a sorted list, for bindings that hand key lists to callers.
pub fn sorted_keys<'a>(keys: impl Iterator<Item = &'a String>) -> Vec<String> {
    let mut keys: Vec<String> = keys.cloned().collect();
    keys.sort();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Holder {
        #[serde(serialize_with = "sorted_map")]
        map: HashMap<String, i32>,
        #[serde(serialize_with = "sorted_opt_map")]
        opt: Option<HashMap<String, i32>>,
    }

    #[test]
    fn keys_serialize_sorted() {
        let map: HashMap<String, i32> = (0..50).map(|i| (format!("k{i:02}"), i)).collect();
        let json = serde_json::to_string(&Holder {
            map: map.clone(),
            opt: Some(map),
        })
        .unwrap();
        let expected: String = (0..50)
            .map(|i| format!("\"k{i:02}\":{i}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            json,
            format!("{{\"map\":{{{expected}}},\"opt\":{{{expected}}}}}")
        );
    }

    #[test]
    fn none_serializes_as_null() {
        let json = serde_json::to_string(&Holder {
            map: HashMap::new(),
            opt: None,
        })
        .unwrap();
        assert_eq!(json, "{\"map\":{},\"opt\":null}");
    }
}
