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
//!
//! # Removal and looseness
//!
//! [`fold_remove`] takes values back out. Bounds can't be inverted, so removing a
//! value that touches one keeps the bound and marks the summary `"loose": true`:
//! it may be a superset of the true summary until a re-fold. Core owns the flag
//! (handlers never see it): folding into or merging with a loose summary gives a
//! loose summary, and removing every value gives the exact empty summary. A
//! re-fold from scratch is exact.

use serde_json::{json, Value};
use std::cmp::Ordering;

use crate::extension_type_registry::ExtensionTypeRegistry;
pub use crate::extension_type_registry::RemoveStep;
use crate::quantize::{quantize_date, quantize_date_upper};

/// The summary key core uses to mark a summary loose. Reserved: a handler's
/// summary must not use it.
pub const LOOSE_KEY: &str = "loose";

/// Whether a summary is marked loose: a value touching one of its bounds was
/// removed, so it may be a superset of the true summary until a re-fold.
pub fn is_loose(summary: &Value) -> bool {
    summary.get(LOOSE_KEY) == Some(&Value::Bool(true))
}

/// Remove the loose flag from `summary`, returning whether it was set, so a
/// handler only ever sees its own summary shape.
fn take_loose(summary: &mut Value) -> bool {
    summary
        .as_object_mut()
        .and_then(|o| o.remove(LOOSE_KEY))
        .is_some_and(|flag| flag == Value::Bool(true))
}

/// Set the loose flag on `summary` when `loose`. Only an object summary can carry
/// it; anything else that needs the flag is an error rather than a silently exact
/// summary.
fn with_loose(mut summary: Value, loose: bool, datatype: &str) -> Result<Value, FoldError> {
    if loose {
        let object = summary.as_object_mut().ok_or_else(|| {
            FoldError(format!(
                "{datatype}: summary is not an object, so it cannot be marked loose"
            ))
        })?;
        object.insert(LOOSE_KEY.to_string(), Value::Bool(true));
    }
    Ok(summary)
}

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
/// ignored; anything else the datatype can't summarise fails the fold. A loose
/// `acc` stays loose.
pub fn fold_values(
    datatype: &str,
    mut acc: Value,
    values: &[&Value],
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<FoldStep, FoldError> {
    let loose = take_loose(&mut acc);
    let values: Vec<&Value> = values.iter().copied().filter(|v| !v.is_null()).collect();
    if let Some(handler) = handler_for(datatype, config, registry) {
        let summary = handler
            .fold(acc, &values, config)
            .map_err(|e| FoldError(format!("{datatype}: {e}")))?;
        return Ok(FoldStep {
            summary: with_loose(summary, loose, datatype)?,
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
        summary: with_loose(range.to_summary(kind), loose, datatype)?,
        skipped,
    })
}

/// Merge two summaries of `datatype` from folds over disjoint values. The result
/// is loose if either input is.
pub fn fold_merge(
    datatype: &str,
    mut a: Value,
    mut b: Value,
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<Value, FoldError> {
    let loose = take_loose(&mut a) | take_loose(&mut b);
    if let Some(handler) = handler_for(datatype, config, registry) {
        let merged = handler
            .fold_merge(a, b, config)
            .map_err(|e| FoldError(format!("{datatype}: {e}")))?;
        return with_loose(merged, loose, datatype);
    }
    let kind = builtin(datatype).ok_or_else(|| no_summary(datatype))?;
    let mut range = Range::from_summary(&a, kind)?;
    range.merge(Range::from_summary(&b, kind)?);
    with_loose(range.to_summary(kind), loose, datatype)
}

/// Remove previously folded values from `acc`, or `Ok(None)` if `datatype` can't
/// remove (the caller must re-fold).
///
/// Removing a value that touches a bound keeps the bound and marks the result
/// loose. A loose `acc` stays loose, except that removing every value gives the
/// exact empty summary. Nulls are ignored, as when folding, and so are `edtf`
/// values the date parser can't order (they were skipped, never folded).
pub fn fold_remove(
    datatype: &str,
    mut acc: Value,
    values: &[&Value],
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<Option<Value>, FoldError> {
    let was_loose = take_loose(&mut acc);
    let values: Vec<&Value> = values.iter().copied().filter(|v| !v.is_null()).collect();
    let step = if let Some(handler) = handler_for(datatype, config, registry) {
        match handler
            .fold_remove(acc, &values, config)
            .map_err(|e| FoldError(format!("{datatype}: {e}")))?
        {
            Some(step) => step,
            None => return Ok(None),
        }
    } else {
        let kind = builtin(datatype).ok_or_else(|| no_summary(datatype))?;
        let mut range = Range::from_summary(&acc, kind)?;
        let mut touched = false;
        for value in values {
            match kind.bounds(value) {
                Some((lower, upper)) => touched |= range.remove(lower, upper),
                None if kind == Builtin::Edtf => {}
                None => {
                    return Err(FoldError(format!(
                        "{datatype}: cannot remove value {value}"
                    )))
                }
            }
        }
        RemoveStep {
            summary: range.to_summary(kind),
            exact: !touched,
        }
    };
    // Nothing left is exactly nothing, however loose the input was.
    let empty = fold_identity(datatype, config, registry).as_ref() == Some(&step.summary);
    let loose = !empty && (was_loose || !step.exact);
    with_loose(step.summary, loose, datatype).map(Some)
}

/// Whether any value folded into `summary` could match `predicate`:
/// conservative, so `false` means "certainly not" and a caller may skip
/// whatever the summary covers (e.g. a layer whose extent misses a query).
///
/// - An empty summary (nothing folded, or `null`) matches nothing.
/// - A loose summary is a superset, so it is answered from its kept bounds.
/// - Built-ins: `number`, `date` and `edtf` answer `{"range": {"min", "max"}}`
///   (inclusive; either end optional; ends written as values of the datatype,
///   e.g. `"1900"` for a date, which covers the whole year).
/// - A registered handler answers for its own datatype (geo: `{"bbox": [...]}`).
/// - Any predicate, summary or datatype that can't be decided gives `true`.
pub fn fold_may_match(
    datatype: &str,
    summary: &Value,
    predicate: &Value,
    config: Option<&Value>,
    registry: Option<&ExtensionTypeRegistry>,
) -> bool {
    if summary.is_null() {
        return false;
    }
    let mut summary = summary.clone();
    take_loose(&mut summary);
    if fold_identity(datatype, config, registry).as_ref() == Some(&summary) {
        return false; // nothing folded
    }
    if let Some(handler) = handler_for(datatype, config, registry) {
        return handler.fold_may_match(&summary, predicate, config);
    }
    let Some(kind) = builtin(datatype) else {
        return true;
    };
    let Ok(range) = Range::from_summary(&summary, kind) else {
        return true;
    };
    let (Some(min), Some(max)) = (range.min, range.max) else {
        return false; // values were folded but none could be ordered
    };
    let Some(wanted) = predicate.get("range") else {
        return true;
    };
    // The predicate's ends, widened to the whole span each value covers.
    let end = |field: &str, upper: bool| -> Option<Option<Key>> {
        match wanted.get(field) {
            None | Some(Value::Null) => Some(None),
            Some(value) => {
                let (lower_key, upper_key) = kind.bounds(value)?;
                Some(Some(if upper { upper_key } else { lower_key }))
            }
        }
    };
    let (Some(from), Some(to)) = (end("min", false), end("max", true)) else {
        return true; // an end we can't order: undecidable
    };
    let after_from = from.is_none_or(|f| max.key.cmp(&f) != Ordering::Less);
    let before_to = to.is_none_or(|t| min.key.cmp(&t) != Ordering::Greater);
    after_from && before_to
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

    /// Take one value back out; returns whether it touched a bound (which is
    /// kept). The last value out empties the range. Removing from an already
    /// empty range means the caller's data is inconsistent, so it counts as
    /// touching.
    fn remove(&mut self, lower: Key, upper: Key) -> bool {
        if self.count == 0 {
            return true;
        }
        self.count -= 1;
        if self.count == 0 {
            *self = Range::default();
            return false;
        }
        let touches_min = self
            .min
            .as_ref()
            .is_none_or(|m| lower.cmp(&m.key) != Ordering::Greater);
        let touches_max = self
            .max
            .as_ref()
            .is_none_or(|m| upper.cmp(&m.key) != Ordering::Less);
        touches_min || touches_max
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

    fn remove(datatype: &str, acc: Value, values: &[Value]) -> Value {
        let refs: Vec<&Value> = values.iter().collect();
        fold_remove(datatype, acc, &refs, None, None)
            .expect("removes")
            .expect("built-ins can remove")
    }

    #[test]
    fn removing_an_interior_value_stays_exact() {
        let acc = fold_all("number", &[json!(1), json!(5), json!(9)]).summary;
        let after = remove("number", acc, &[json!(5)]);
        assert_eq!(after, json!({ "count": 2, "min": 1, "max": 9 }));
        assert!(!is_loose(&after));
    }

    #[test]
    fn removing_a_bound_keeps_it_and_marks_loose() {
        let acc = fold_all("number", &[json!(1), json!(5), json!(9)]).summary;
        let after = remove("number", acc, &[json!(9)]);
        assert_eq!(
            after,
            json!({ "count": 2, "min": 1, "max": 9, "loose": true }),
            "the bound is kept: a superset"
        );
        // A tie at the bound is still marked loose: no counts are kept.
        let tied = fold_all("number", &[json!(1), json!(9), json!(9)]).summary;
        assert!(is_loose(&remove("number", tied, &[json!(9)])));
    }

    #[test]
    fn date_removal_compares_both_ends_of_a_value() {
        let acc = fold_all("date", &[json!("1850"), json!("1855-06"), json!("1860")]).summary;
        assert!(!is_loose(&remove("date", acc.clone(), &[json!("1855-06")])));
        assert!(
            is_loose(&remove("date", acc.clone(), &[json!("1860")])),
            "1860 runs to the year's end, which is the max"
        );
        // A value inside the year 1850 starts after 1850-01-01, so it isn't the min.
        assert!(!is_loose(&remove("date", acc, &[json!("1850-03-01")])));
    }

    #[test]
    fn unorderable_edtf_is_ignored_on_removal_as_on_folding() {
        let acc = fold_all("edtf", &[json!("1850"), json!("1855"), json!("1860")]).summary;
        let after = remove("edtf", acc.clone(), &[json!("1855?")]);
        assert_eq!(after, acc, "never folded, so nothing to take out");
    }

    #[test]
    fn removing_everything_is_exactly_empty_even_from_loose() {
        let acc = fold_all("number", &[json!(1), json!(9)]).summary;
        let loose = remove("number", acc, &[json!(9)]);
        assert!(is_loose(&loose));
        let empty = remove("number", loose, &[json!(1)]);
        assert_eq!(empty, json!({ "count": 0 }));
    }

    #[test]
    fn looseness_survives_folding_and_merging() {
        let acc = fold_all("number", &[json!(1), json!(9)]).summary;
        let loose = remove("number", acc, &[json!(9)]);

        let folded = fold_values("number", loose.clone(), &[&json!(4)], None, None)
            .unwrap()
            .summary;
        assert!(is_loose(&folded));

        let exact = fold_all("number", &[json!(3)]).summary;
        let merged = fold_merge("number", exact.clone(), loose, None, None).unwrap();
        assert!(
            is_loose(&merged),
            "either input loose makes the merge loose"
        );
        assert!(!is_loose(
            &fold_merge("number", exact.clone(), exact, None, None).unwrap()
        ));
    }

    #[test]
    fn a_handler_without_removal_asks_for_a_refold() {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register("number".to_string(), Arc::new(Concatenate));
        let result = fold_remove("number", json!("ab"), &[&json!("a")], None, Some(&registry));
        assert_eq!(result, Ok(None));
    }

    fn may(datatype: &str, summary: &Value, predicate: Value) -> bool {
        fold_may_match(datatype, summary, &predicate, None, None)
    }

    #[test]
    fn may_match_never_excludes_a_folded_value() {
        let values = [json!(-4.5), json!(0), json!(3), json!(12.25)];
        let summary = fold_all("number", &values).summary;
        for v in &values {
            assert!(
                may(
                    "number",
                    &summary,
                    json!({ "range": { "min": v, "max": v } })
                ),
                "{v}"
            );
        }
        // A loose summary keeps its bounds, so a removed extreme is still "maybe".
        let loose = remove("number", summary, &[json!(12.25)]);
        assert!(is_loose(&loose));
        for v in &values {
            assert!(
                may("number", &loose, json!({ "range": { "min": v, "max": v } })),
                "{v}"
            );
        }
    }

    #[test]
    fn may_match_excludes_disjoint_ranges() {
        let summary = fold_all("number", &[json!(10), json!(20)]).summary;
        assert!(!may("number", &summary, json!({ "range": { "min": 21 } })));
        assert!(!may("number", &summary, json!({ "range": { "max": 9.5 } })));
        assert!(!may(
            "number",
            &summary,
            json!({ "range": { "min": 0, "max": 9 } })
        ));
        assert!(
            may("number", &summary, json!({ "range": { "min": 20 } })),
            "inclusive"
        );
        assert!(may(
            "number",
            &summary,
            json!({ "range": { "min": 15, "max": 99 } })
        ));
        assert!(may("number", &summary, json!({ "range": {} })), "unbounded");
    }

    #[test]
    fn date_may_match_widens_each_end_to_what_it_covers() {
        let summary = fold_all("date", &[json!("1850-03-01"), json!("1860")]).summary;
        assert!(!may(
            "date",
            &summary,
            json!({ "range": { "min": "1861" } })
        ));
        assert!(!may(
            "date",
            &summary,
            json!({ "range": { "max": "1849" } })
        ));
        // "1860" runs to the year's end; "1850" as a max covers all of 1850.
        assert!(may(
            "date",
            &summary,
            json!({ "range": { "min": "1860-06-01" } })
        ));
        assert!(may("date", &summary, json!({ "range": { "max": "1850" } })));
    }

    #[test]
    fn may_match_is_false_only_when_certain() {
        let summary = fold_all("number", &[json!(1), json!(2)]).summary;
        assert!(
            may("number", &summary, json!({ "bbox": [0, 0, 1, 1] })),
            "unknown predicate"
        );
        assert!(
            may("number", &summary, json!({ "range": { "min": "x" } })),
            "unorderable end"
        );
        assert!(
            may("string", &json!({ "count": 1 }), json!({ "range": {} })),
            "no fold"
        );
        assert!(!may(
            "number",
            &fold_all("number", &[]).summary,
            json!({ "range": {} })
        ));
        assert!(
            !may("number", &Value::Null, json!({ "range": {} })),
            "null: nothing folded"
        );
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
