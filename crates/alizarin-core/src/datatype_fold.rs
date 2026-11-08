//! Datatype → fold dispatch: reducing many values of one datatype into a summary.
//!
//! Registry-first, like [`crate::datatype_index`]: a registered
//! [`ExtensionTypeHandler`](crate::extension_type_registry::ExtensionTypeHandler)
//! whose `fold_identity` returns `Some` owns its datatype's summary. Otherwise
//! the built-in fallback covers only core-owned datatypes:
//!
//! * `number` → `{count, min, max}`
//! * `date`, `edtf` → `{count, min, max, min_day, max_day}`: `min`/`max` are the
//!   raw values with the earliest start / latest end, and the `_day` fields are
//!   those ends in the index's integer space ([`crate::quantize`]), so a summary
//!   can be compared directly with index keys.
//!
//! Every other datatype has no summary unless a handler claims it (geometry is
//! `alizarin-geo`'s). Summaries are JSON with sorted keys, so they serialise
//! identically on every backend, and folds are order-independent: equal keys
//! are broken by the value's JSON text, never by arrival order.

use serde_json::{json, Value};
use std::cmp::Ordering;

use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::quantize::{quantize_date, quantize_date_upper};

/// A fold that couldn't complete, e.g. a value its datatype can't summarise.
#[derive(Debug, Clone, PartialEq)]
pub struct FoldError(pub String);

impl std::fmt::Display for FoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FoldError {}

/// One fold step: the new summary, and how many values were skipped.
///
/// Only the built-in `edtf` fold skips (values its date parser can't order
/// yet); everything else either folds a value or fails.
#[derive(Debug, Clone, PartialEq)]
pub struct FoldStep {
    pub summary: Value,
    pub skipped: usize,
}

/// The empty summary for `datatype`, or `None` if it has no summary.
pub fn fold_identity(
    datatype: &str,
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> Option<Value> {
    if let Some(identity) = registry
        .and_then(|r| r.get(datatype))
        .and_then(|h| h.fold_identity(config))
    {
        return Some(identity);
    }
    builtin(datatype).map(|_| json!({ "count": 0 }))
}

/// Fold a batch of values into `acc`. Null values are not values and are
/// ignored; anything else the datatype can't summarise fails the fold.
pub fn fold_values(
    datatype: &str,
    acc: Value,
    values: &[&Value],
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<FoldStep, FoldError> {
    let values: Vec<&Value> = values.iter().copied().filter(|v| !v.is_null()).collect();
    if let Some(handler) = handler_for(datatype, config, registry) {
        let summary = handler
            .fold(acc, &values, config)
            .map_err(|e| FoldError(format!("{datatype}: {e}")))?;
        return Ok(FoldStep {
            summary,
            skipped: 0,
        });
    }
    let kind = builtin(datatype).ok_or_else(|| no_summary(datatype))?;
    let mut range = Range::from_summary(&acc, kind)?;
    let mut skipped = 0;
    for value in values {
        match kind.bounds(value) {
            Some((lower, upper)) => range.add(lower, upper, value),
            None if kind == Builtin::Edtf => skipped += 1,
            None => {
                return Err(FoldError(format!(
                    "{datatype}: cannot summarise value {value}"
                )))
            }
        }
    }
    Ok(FoldStep {
        summary: range.to_summary(kind),
        skipped,
    })
}

/// Merge two summaries of `datatype` from folds over disjoint values.
pub fn fold_merge(
    datatype: &str,
    a: Value,
    b: Value,
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<Value, FoldError> {
    if let Some(handler) = handler_for(datatype, config, registry) {
        return handler
            .fold_merge(a, b, config)
            .map_err(|e| FoldError(format!("{datatype}: {e}")));
    }
    let kind = builtin(datatype).ok_or_else(|| no_summary(datatype))?;
    let mut range = Range::from_summary(&a, kind)?;
    range.merge(Range::from_summary(&b, kind)?);
    Ok(range.to_summary(kind))
}

fn handler_for<'a>(
    datatype: &str,
    config: Option<&Value>,
    registry: Option<&'a ExtensionTypeRegistry>,
) -> Option<&'a std::sync::Arc<dyn crate::extension_type_registry::ExtensionTypeHandler>> {
    registry
        .and_then(|r| r.get(datatype))
        .filter(|h| h.fold_identity(config).is_some())
}

fn no_summary(datatype: &str) -> FoldError {
    FoldError(format!("{datatype}: datatype has no summary"))
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Builtin {
    Number,
    Date,
    Edtf,
}

fn builtin(datatype: &str) -> Option<Builtin> {
    match datatype {
        "number" => Some(Builtin::Number),
        "date" => Some(Builtin::Date),
        "edtf" => Some(Builtin::Edtf),
        _ => None,
    }
}

impl Builtin {
    /// A value's (lower, upper) ordering keys, or `None` if it can't be ordered.
    fn bounds(self, value: &Value) -> Option<(Key, Key)> {
        match self {
            Builtin::Number => value.as_f64().map(|n| (Key::Number(n), Key::Number(n))),
            Builtin::Date | Builtin::Edtf => {
                let s = value.as_str()?;
                Some((
                    Key::Day(quantize_date(s)?),
                    Key::Day(quantize_date_upper(s)?),
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Key {
    Number(f64),
    Day(i64),
}

impl Key {
    fn cmp(&self, other: &Key) -> Ordering {
        match (self, other) {
            (Key::Number(a), Key::Number(b)) => a.total_cmp(b),
            (Key::Day(a), Key::Day(b)) => a.cmp(b),
            // A summary never mixes the two; order them consistently anyway.
            (Key::Number(_), Key::Day(_)) => Ordering::Less,
            (Key::Day(_), Key::Number(_)) => Ordering::Greater,
        }
    }

    fn to_json(self) -> Value {
        match self {
            Key::Number(n) => json!(n),
            Key::Day(d) => json!(d),
        }
    }
}

/// An end of a range: its ordering key and the raw value it came from.
#[derive(Debug, Clone)]
struct End {
    key: Key,
    value: Value,
}

impl End {
    /// Order by key, then by the value's JSON text, so ties don't depend on the
    /// order values arrived in.
    fn cmp(&self, other: &End) -> Ordering {
        self.key
            .cmp(&other.key)
            .then_with(|| self.value.to_string().cmp(&other.value.to_string()))
    }
}

#[derive(Debug, Clone, Default)]
struct Range {
    count: u64,
    min: Option<End>,
    max: Option<End>,
}

impl Range {
    fn add(&mut self, lower: Key, upper: Key, value: &Value) {
        self.count += 1;
        self.take_min(End {
            key: lower,
            value: value.clone(),
        });
        self.take_max(End {
            key: upper,
            value: value.clone(),
        });
    }

    fn merge(&mut self, other: Range) {
        self.count += other.count;
        if let Some(min) = other.min {
            self.take_min(min);
        }
        if let Some(max) = other.max {
            self.take_max(max);
        }
    }

    fn take_min(&mut self, end: End) {
        if self
            .min
            .as_ref()
            .is_none_or(|m| end.cmp(m) == Ordering::Less)
        {
            self.min = Some(end);
        }
    }

    fn take_max(&mut self, end: End) {
        if self
            .max
            .as_ref()
            .is_none_or(|m| end.cmp(m) == Ordering::Greater)
        {
            self.max = Some(end);
        }
    }

    fn to_summary(&self, kind: Builtin) -> Value {
        let mut summary = json!({ "count": self.count });
        if let (Some(min), Some(max)) = (&self.min, &self.max) {
            summary["min"] = min.value.clone();
            summary["max"] = max.value.clone();
            if kind != Builtin::Number {
                summary["min_day"] = min.key.to_json();
                summary["max_day"] = max.key.to_json();
            }
        }
        summary
    }

    /// Read a summary back. `min`/`max` are re-derived from their raw values,
    /// so a summary carries no state that could disagree with them.
    fn from_summary(summary: &Value, kind: Builtin) -> Result<Range, FoldError> {
        let bad = || FoldError(format!("not a {kind:?} summary: {summary}"));
        let count = summary
            .get("count")
            .and_then(Value::as_u64)
            .ok_or_else(bad)?;
        let end = |field: &str, upper: bool| -> Result<Option<End>, FoldError> {
            match summary.get(field) {
                None => Ok(None),
                Some(value) => {
                    let (lower_key, upper_key) = kind.bounds(value).ok_or_else(bad)?;
                    Ok(Some(End {
                        key: if upper { upper_key } else { lower_key },
                        value: value.clone(),
                    }))
                }
            }
        };
        Ok(Range {
            count,
            min: end("min", false)?,
            max: end("max", true)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension_type_registry::{
        ExtensionError, ExtensionTypeHandler, HandlerCapabilities,
    };
    use std::sync::Arc;

    fn fold_all(datatype: &str, values: &[Value]) -> FoldStep {
        let refs: Vec<&Value> = values.iter().collect();
        let identity = fold_identity(datatype, None, None).expect("foldable");
        fold_values(datatype, identity, &refs, None, None).expect("folds")
    }

    #[test]
    fn numbers_fold_to_count_min_max() {
        let step = fold_all("number", &[json!(3), json!(-1.5), json!(10), Value::Null]);
        assert_eq!(
            step.summary,
            json!({ "count": 3, "min": -1.5, "max": 10 }),
            "nulls are not values"
        );
        assert_eq!(step.skipped, 0);
    }

    #[test]
    fn dates_take_the_earliest_start_and_latest_end() {
        let step = fold_all("date", &[json!("1855-06-01"), json!("1850"), json!("1860")]);
        assert_eq!(step.summary["min"], json!("1850"));
        assert_eq!(step.summary["max"], json!("1860"));
        assert_eq!(
            step.summary["min_day"],
            json!(quantize_date("1850-01-01").unwrap())
        );
        assert_eq!(
            step.summary["max_day"],
            json!(quantize_date("1860-12-31").unwrap()),
            "a bare year runs to its end"
        );
    }

    #[test]
    fn an_unorderable_date_fails_the_fold() {
        let identity = fold_identity("date", None, None).unwrap();
        let err = fold_values("date", identity, &[&json!("next tuesday")], None, None);
        assert!(err.is_err());
    }

    #[test]
    fn edtf_the_date_parser_cannot_order_is_skipped_and_counted() {
        let step = fold_all("edtf", &[json!("1850"), json!("1850/1860"), json!("1855?")]);
        assert_eq!(step.skipped, 2);
        assert_eq!(step.summary["count"], json!(1));
        assert_eq!(step.summary["min"], json!("1850"));
    }

    #[test]
    fn merging_partial_folds_matches_one_fold_in_any_order() {
        let values = [
            json!("1850-03"),
            json!("1849"),
            json!("1849-01-01"),
            json!("1902-12"),
            json!("1902"),
        ];
        let whole = fold_all("date", &values).summary;

        let left = fold_all("date", &values[..2]).summary;
        let right = fold_all("date", &values[2..]).summary;
        assert_eq!(
            fold_merge("date", left.clone(), right.clone(), None, None).unwrap(),
            whole
        );
        assert_eq!(fold_merge("date", right, left, None, None).unwrap(), whole);

        let mut reversed = values.to_vec();
        reversed.reverse();
        assert_eq!(fold_all("date", &reversed).summary, whole);
    }

    #[test]
    fn empty_folds_and_unsummarised_datatypes() {
        assert_eq!(fold_all("number", &[]).summary, json!({ "count": 0 }));
        assert_eq!(fold_identity("string", None, None), None);
        let err = fold_values("string", json!({}), &[&json!("x")], None, None);
        assert!(err.is_err());
    }

    struct Concatenate;
    impl ExtensionTypeHandler for Concatenate {
        fn capabilities(&self) -> HandlerCapabilities {
            HandlerCapabilities::default()
        }
        fn fold_identity(&self, _config: Option<&Value>) -> Option<Value> {
            Some(json!(""))
        }
        fn fold(
            &self,
            acc: Value,
            values: &[&Value],
            _config: Option<&Value>,
        ) -> Result<Value, ExtensionError> {
            let mut s = acc.as_str().unwrap_or_default().to_string();
            for v in values {
                s.push_str(v.as_str().unwrap_or_default());
            }
            Ok(json!(s))
        }
        fn fold_merge(
            &self,
            a: Value,
            b: Value,
            _config: Option<&Value>,
        ) -> Result<Value, ExtensionError> {
            Ok(json!(format!(
                "{}{}",
                a.as_str().unwrap_or_default(),
                b.as_str().unwrap_or_default()
            )))
        }
    }

    #[test]
    fn a_registered_handler_owns_its_datatype_summary() {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register("number".to_string(), Arc::new(Concatenate));
        let identity = fold_identity("number", None, Some(&registry)).unwrap();
        let step = fold_values(
            "number",
            identity,
            &[&json!("a"), &json!("b")],
            None,
            Some(&registry),
        )
        .unwrap();
        assert_eq!(step.summary, json!("ab"), "the handler, not the built-in");
    }
}
