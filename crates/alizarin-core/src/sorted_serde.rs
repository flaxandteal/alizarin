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

/// The value under the lowest key: a deterministic "any value" for fallbacks
/// (e.g. a label in some language when neither the requested one nor `en`
/// exists). `map.values().next()` would pick a different entry per process.
pub fn lowest_key_value<K: Ord, V>(map: &HashMap<K, V>) -> Option<&V> {
    map.iter().min_by(|a, b| a.0.cmp(b.0)).map(|(_, v)| v)
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
    fn fallback_value_is_the_lowest_key_however_the_map_was_built() {
        for _ in 0..16 {
            let labels: HashMap<String, &str> = ["ga", "cy", "fr", "de"]
                .iter()
                .map(|lang| (lang.to_string(), *lang))
                .collect();
            assert_eq!(lowest_key_value(&labels), Some(&"cy"));
        }
        assert_eq!(lowest_key_value(&HashMap::<String, i32>::new()), None);
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
