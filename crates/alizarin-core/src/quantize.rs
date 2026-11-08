// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ordered-value quantization: the shared encoding for `IndexClass::Ordered`.
//!
//! Lives here, not in the emitter, because BOTH sides need the SAME function:
//! the emitter quantizes a tile's value into the head's `value_tags` column, and
//! a query-builder quantizes the endpoints of a range filter into the same
//! integer space. If they disagreed by a day, a range query would silently miss
//! its own boundary. One function, one truth.
//!
//! The keys are SIGNED integers. SQLite's `INTEGER` is i64 and orders signed, so
//! a pre-1970 date is simply a negative day count that sorts before 1970 — no
//! bias needed. (v1 RM stored a u32 and added 2³¹ to keep the order under
//! unsigned comparison; the head's signed column makes that unnecessary.)

/// A date string (`YYYY`, `YYYY-MM`, or `YYYY-MM-DD`, optionally with a time
/// suffix that is ignored — day precision) → signed days since 1970-01-01.
///
/// `None` if the leading `YYYY[-MM[-DD]]` will not parse. Missing month/day
/// default to 1 (so `"2020"` is 2020-01-01), matching how a bare year is a point
/// at the start of its range.
pub fn quantize_date(date_str: &str) -> Option<i64> {
    let (year, month, day) = date_parts(date_str)?;
    let (month, day) = (month.unwrap_or(1), day.unwrap_or(1));
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

/// Like [`quantize_date`], but the LAST day the string covers, in the same
/// integer space: `"1860"` is 1860-12-31, `"1860-02"` is 1860-02-29. Use it for
/// the upper end of a range, where a bare year means "up to the end of it".
pub fn quantize_date_upper(date_str: &str) -> Option<i64> {
    let (year, month, day) = date_parts(date_str)?;
    let month = month.unwrap_or(12);
    if !(1..=12).contains(&month) {
        return None;
    }
    let day = day.unwrap_or_else(|| days_in_month(year, month));
    if !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

/// `YYYY[-MM[-DD]]` → (year, month, day), each of month/day `None` if absent or
/// unparseable. Ignores a time suffix and allows a leading `-` for BCE years.
fn date_parts(date_str: &str) -> Option<(i64, Option<i64>, Option<i64>)> {
    // Take the date portion before any 'T' time separator or whitespace.
    let date_part = date_str.split(['T', ' ']).next().unwrap_or(date_str).trim();
    // Allow a leading '-' for BCE years (e.g. "-0044-03-15"): split the sign off,
    // then split the rest on '-', and reattach the sign to the year.
    let (neg, rest) = match date_part.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, date_part),
    };
    let mut parts = rest.split('-');
    let year_abs: i64 = parts.next()?.parse().ok()?;
    let year = if neg { -year_abs } else { year_abs };
    let month = parts.next().and_then(|s| s.parse().ok());
    let day = parts.next().and_then(|s| s.parse().ok());
    Some((year, month, day))
}

/// Days in a month of the proleptic Gregorian calendar.
fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days from 1970-01-01 to the given civil (proleptic Gregorian) date.
///
/// Howard Hinnant's `days_from_civil` — the canonical algorithm; exact for all
/// dates, handles negative years, returns a signed count. `m` in `[1,12]`, `d`
/// in `[1,31]` (validated by the caller).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_zero() {
        assert_eq!(quantize_date("1970-01-01"), Some(0));
    }

    #[test]
    fn upper_bound_is_the_last_day_covered() {
        assert_eq!(quantize_date_upper("1970"), quantize_date("1970-12-31"));
        assert_eq!(quantize_date_upper("1970-04"), quantize_date("1970-04-30"));
        assert_eq!(quantize_date_upper("2000-02"), quantize_date("2000-02-29"));
        assert_eq!(quantize_date_upper("1900-02"), quantize_date("1900-02-28"));
        assert_eq!(
            quantize_date_upper("1970-04-15"),
            quantize_date("1970-04-15")
        );
        assert_eq!(quantize_date_upper("-0044"), quantize_date("-0044-12-31"));
        assert_eq!(
            quantize_date_upper("1850-21"),
            None,
            "seasons aren't months"
        );
        assert_eq!(quantize_date_upper("not a date"), None);
    }

    #[test]
    fn ordering_is_monotonic_including_pre_1970() {
        // A strictly increasing sequence of dates must map to strictly
        // increasing keys — the whole point, and the property a range query
        // relies on. Pre-1970 must sort BELOW the epoch (negative).
        let dates = [
            "-0044-03-15", // Ides of March, 44 BCE
            "1215-06-15",  // Magna Carta
            "1969-12-31",
            "1970-01-01",
            "1970-01-02",
            "2000-02-29", // leap day
            "2020-05-01",
            "2024-12-31",
        ];
        let keys: Vec<i64> = dates.iter().map(|d| quantize_date(d).unwrap()).collect();
        assert_eq!(keys[3], 0, "epoch");
        assert!(keys[2] < 0, "1969 sorts below the epoch");
        for w in keys.windows(2) {
            assert!(
                w[0] < w[1],
                "dates map to strictly increasing keys: {keys:?}"
            );
        }
    }

    #[test]
    fn precision_defaults_and_time_suffix() {
        assert_eq!(quantize_date("2020"), quantize_date("2020-01-01"));
        assert_eq!(quantize_date("2020-05"), quantize_date("2020-05-01"));
        // A time suffix is dropped to day precision.
        assert_eq!(
            quantize_date("2020-05-01T13:45:00Z"),
            quantize_date("2020-05-01")
        );
    }

    #[test]
    fn known_offsets() {
        assert_eq!(quantize_date("1971-01-01"), Some(365));
        assert_eq!(quantize_date("1972-01-01"), Some(730)); // 1971 not a leap year
        assert_eq!(quantize_date("1973-01-01"), Some(1096)); // 1972 WAS a leap year
    }

    #[test]
    fn garbage_is_none() {
        assert_eq!(quantize_date("not-a-date"), None);
        assert_eq!(quantize_date(""), None);
        assert_eq!(quantize_date("2020-13-01"), None); // month 13
        assert_eq!(quantize_date("2020-01-32"), None); // day 32
    }
}
