use wasm_bindgen::{JsCast, JsValue};

#[allow(dead_code)]
pub fn set_panic_hook() {
    // Call this function at least once during initialization to get
    // better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    console_error_panic_hook::set_once();
}

/// Convert an `Option<Vec<String>>` of ontology class URIs into a JS value
/// using the Arches wire shape: `None`/empty → `null`, single-element → plain
/// string, multiple → JS array of strings. Mirrors `optional_string_or_vec`
/// on the serde side.
pub fn classes_to_js_value(classes: Option<&Vec<String>>) -> JsValue {
    match classes {
        None => JsValue::NULL,
        Some(list) if list.is_empty() => JsValue::NULL,
        Some(list) if list.len() == 1 => JsValue::from_str(&list[0]),
        Some(list) => {
            let arr = js_sys::Array::new();
            for class in list {
                arr.push(&JsValue::from_str(class));
            }
            arr.into()
        }
    }
}

/// Parse a JS value (string, array of strings, null, or undefined) into a
/// cleaned `Option<Vec<String>>`. Blank/empty entries are stripped and empty
/// results collapse to `None`. Non-string/array inputs also yield `None`.
pub fn parse_js_class_list(value: JsValue) -> Option<Vec<String>> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    if let Some(s) = value.as_string() {
        let trimmed = s.trim();
        return if trimmed.is_empty() {
            None
        } else {
            Some(vec![trimmed.to_string()])
        };
    }
    if let Ok(arr) = value.dyn_into::<js_sys::Array>() {
        let mut classes: Vec<String> = Vec::with_capacity(arr.length() as usize);
        for i in 0..arr.length() {
            if let Some(s) = arr.get(i).as_string() {
                let t = s.trim();
                if !t.is_empty() {
                    classes.push(t.to_string());
                }
            }
        }
        return if classes.is_empty() {
            None
        } else {
            Some(classes)
        };
    }
    None
}

/// Insert an ontology class list into a `serde_json::Map` using the Arches
/// wire shape. Empty lists are skipped so callers don't have to pre-filter.
pub fn insert_classes_json(
    obj: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    classes: Option<&Vec<String>>,
) {
    match classes {
        None => {}
        Some(list) if list.is_empty() => {}
        Some(list) if list.len() == 1 => {
            obj.insert(key.to_string(), serde_json::Value::String(list[0].clone()));
        }
        Some(list) => {
            obj.insert(
                key.to_string(),
                serde_json::Value::Array(
                    list.iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
    }
}
