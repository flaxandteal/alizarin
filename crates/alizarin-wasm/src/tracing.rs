//! Unified tracing infrastructure for Alizarin
//!
//! This module provides performance tracing that works in both WASM (browser)
//! and native Rust contexts.
//!
//! In WASM: Integrates with the browser's Performance API (performance.now())
//! In Native: Uses std::time::Instant for high-resolution timing
//!
//! For hot loops, use `record_timing_sampled` which only measures every Nth call
//! to reduce the overhead of crossing the JS boundary in WASM.

use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ============================================================================
// Platform-agnostic time source
// ============================================================================

/// Get current time in milliseconds (platform-agnostic)
#[cfg(target_arch = "wasm32")]
pub fn now_ms() -> f64 {
    now()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_ms() -> f64 {
    use std::time::Instant;
    thread_local! {
        static START: Instant = Instant::now();
    }
    START.with(|start| start.elapsed().as_secs_f64() * 1000.0)
}

// ============================================================================
// Performance API bindings (WASM only)
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn mark(name: &str);

    #[wasm_bindgen(js_namespace = performance, js_name = mark)]
    fn mark_with_options(name: &str, options: &JsValue);

    #[wasm_bindgen(js_namespace = performance)]
    fn measure(name: &str, start_mark: &str, end_mark: &str);

    #[wasm_bindgen(js_namespace = performance, js_name = measure)]
    fn measure_with_options(name: &str, options: &JsValue);

    #[wasm_bindgen(js_namespace = performance, js_name = clearMarks)]
    fn clear_marks(name: &str);

    #[wasm_bindgen(js_namespace = performance, js_name = clearMeasures)]
    fn clear_measures(name: &str);

    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

// Stub implementations for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
fn mark(_name: &str) {}

#[cfg(not(target_arch = "wasm32"))]
fn clear_marks(_name: &str) {}

// ============================================================================
// Span implementation
// ============================================================================

/// A tracing span that records timing via the Performance API
///
/// When dropped, the span automatically records a performance.measure()
/// that can be collected by the JS tracing infrastructure.
pub struct Span {
    #[allow(dead_code)] // Used in WASM builds only
    name: String,
    start_mark: String,
    attributes: HashMap<String, SpanValue>,
    ended: bool,
}

#[derive(Clone)]
pub enum SpanValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Span {
    /// Create a new span and mark its start time
    pub fn new(name: &str) -> Self {
        let start_mark = format!("alizarin:{}:start:{}", name, now_ms() as u64);
        mark(&start_mark);

        Self {
            name: name.to_string(),
            start_mark,
            attributes: HashMap::new(),
            ended: false,
        }
    }

    /// Add a string attribute to the span
    pub fn set_attribute_str(&mut self, key: &str, value: &str) -> &mut Self {
        self.attributes
            .insert(key.to_string(), SpanValue::String(value.to_string()));
        self
    }

    /// Add an integer attribute to the span
    pub fn set_attribute_int(&mut self, key: &str, value: i64) -> &mut Self {
        self.attributes
            .insert(key.to_string(), SpanValue::Int(value));
        self
    }

    /// Add a float attribute to the span
    pub fn set_attribute_float(&mut self, key: &str, value: f64) -> &mut Self {
        self.attributes
            .insert(key.to_string(), SpanValue::Float(value));
        self
    }

    /// Add a boolean attribute to the span
    pub fn set_attribute_bool(&mut self, key: &str, value: bool) -> &mut Self {
        self.attributes
            .insert(key.to_string(), SpanValue::Bool(value));
        self
    }

    /// End the span and record the measurement (native version - just clears marks)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn end(&mut self) {
        if self.ended {
            return;
        }
        self.ended = true;
        // In native mode, we don't have the Performance API, so just clean up
        clear_marks(&self.start_mark);
    }

    /// End the span and record the measurement (WASM version - uses Performance API)
    #[cfg(target_arch = "wasm32")]
    pub fn end(&mut self) {
        if self.ended {
            return;
        }
        self.ended = true;

        let end_mark = format!("alizarin:{}:end:{}", self.name, now_ms() as u64);
        mark(&end_mark);

        // Create measure with detail containing attributes
        let measure_name = format!("alizarin:{}", self.name);

        // Build options object with start, end, and detail
        let options = js_sys::Object::new();

        js_sys::Reflect::set(&options, &"start".into(), &self.start_mark.clone().into()).ok();
        js_sys::Reflect::set(&options, &"end".into(), &end_mark.clone().into()).ok();

        // Add attributes as detail
        if !self.attributes.is_empty() {
            let detail = js_sys::Object::new();
            for (key, value) in &self.attributes {
                let js_value: JsValue = match value {
                    SpanValue::String(s) => s.clone().into(),
                    SpanValue::Int(i) => (*i as f64).into(),
                    SpanValue::Float(f) => (*f).into(),
                    SpanValue::Bool(b) => (*b).into(),
                };
                js_sys::Reflect::set(&detail, &key.clone().into(), &js_value).ok();
            }
            js_sys::Reflect::set(&options, &"detail".into(), &detail).ok();
        }

        measure_with_options(&measure_name, &options);

        // Clean up marks
        clear_marks(&self.start_mark);
        clear_marks(&end_mark);
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        self.end();
    }
}

// ============================================================================
// Convenience macros and functions
// ============================================================================

/// Start a new span
pub fn start_span(name: &str) -> Span {
    Span::new(name)
}

/// Time a closure and return its result
pub fn timed<T, F: FnOnce() -> T>(name: &str, f: F) -> T {
    let _span = Span::new(name);
    f()
}

/// Time a closure with attributes
pub fn timed_with<T, F: FnOnce(&mut Span) -> T>(name: &str, f: F) -> T {
    let mut span = Span::new(name);
    let result = f(&mut span);
    span.end();
    result
}

// ============================================================================
// Legacy compatibility layer
// ============================================================================
// This provides backward compatibility with the existing TimingStats system
// while transitioning to the new Performance API-based approach.

#[derive(Default, Clone)]
struct LegacyTimingStats {
    count: u32,
    total_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

impl LegacyTimingStats {
    fn record(&mut self, ms: f64) {
        self.count += 1;
        self.total_ms += ms;
        if self.count == 1 {
            self.min_ms = ms;
            self.max_ms = ms;
        } else {
            self.min_ms = self.min_ms.min(ms);
            self.max_ms = self.max_ms.max(ms);
        }
    }

    fn avg_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ms / self.count as f64
        }
    }
}

thread_local! {
    static LEGACY_TIMINGS: RefCell<HashMap<&'static str, LegacyTimingStats>> = RefCell::new(HashMap::new());
}

/// Record a timing measurement (legacy compatibility)
///
/// Note: This only stores in the internal LEGACY_TIMINGS map, which can be
/// retrieved via getRscvTimings(). We intentionally don't emit Performance API
/// measures here because record_timing is called thousands of times during ETL,
/// and the Performance API overhead would cause significant slowdown.
pub fn record_timing(label: &'static str, ms: f64) {
    LEGACY_TIMINGS.with(|timings| {
        timings.borrow_mut().entry(label).or_default().record(ms);
    });
}

// ============================================================================
// Sampled timing for hot loops
// ============================================================================
// In hot loops, calling now_ms() on every iteration is expensive because it
// crosses the JS/WASM boundary. Instead, we sample every Nth call and
// extrapolate the timing.

/// Default sample rate: measure every 100th call
pub const DEFAULT_SAMPLE_RATE: u32 = 100;

thread_local! {
    static SAMPLE_COUNTERS: RefCell<HashMap<&'static str, u32>> = RefCell::new(HashMap::new());
}

/// Record timing with sampling - only measures every Nth call to reduce overhead.
///
/// The timing is extrapolated: if we measure 1ms on a sampled call, we record
/// sample_rate * 1ms as the total for those calls.
///
/// Returns true if this call was sampled (caller should measure), false otherwise.
#[inline]
pub fn should_sample(label: &'static str, sample_rate: u32) -> bool {
    SAMPLE_COUNTERS.with(|counters| {
        let mut counters = counters.borrow_mut();
        let counter = counters.entry(label).or_insert(0);
        *counter += 1;
        if *counter >= sample_rate {
            *counter = 0;
            true
        } else {
            false
        }
    })
}

/// Record a sampled timing measurement.
///
/// Use with `should_sample()`:
/// ```ignore
/// let do_sample = should_sample("my_label", 100);
/// let t0 = if do_sample { now_ms() } else { 0.0 };
/// // ... do work ...
/// if do_sample {
///     record_timing_sampled("my_label", now_ms() - t0, 100);
/// }
/// ```
///
/// This records count=sample_rate and total_ms=ms*sample_rate to extrapolate
/// the timing across all calls.
pub fn record_timing_sampled(label: &'static str, ms: f64, sample_rate: u32) {
    LEGACY_TIMINGS.with(|timings| {
        let mut timings = timings.borrow_mut();
        let stats = timings.entry(label).or_default();
        // Record the extrapolated count and total
        stats.count += sample_rate;
        stats.total_ms += ms * sample_rate as f64;
        // For min/max, we use the actual sampled value (not extrapolated)
        if stats.count == sample_rate {
            stats.min_ms = ms;
            stats.max_ms = ms;
        } else {
            stats.min_ms = stats.min_ms.min(ms);
            stats.max_ms = stats.max_ms.max(ms);
        }
    });
}

/// Print timing summary (native Rust version)
#[cfg(not(target_arch = "wasm32"))]
pub fn print_rscv_timings() {
    LEGACY_TIMINGS.with(|timings| {
        let timings = timings.borrow();
        println!("=== RSCV Timing Summary ===");
        let mut entries: Vec<_> = timings.iter().collect();
        entries.sort_by(|a, b| b.1.total_ms.partial_cmp(&a.1.total_ms).unwrap());
        for (label, stats) in entries {
            println!(
                "{}: count={}, total={:.2}ms, avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                label,
                stats.count,
                stats.total_ms,
                stats.avg_ms(),
                stats.min_ms,
                stats.max_ms
            );
        }
    });
}

/// Clear timing stats (native Rust version)
#[cfg(not(target_arch = "wasm32"))]
pub fn clear_rscv_timings() {
    LEGACY_TIMINGS.with(|timings| {
        timings.borrow_mut().clear();
    });
}

// ============================================================================
// WASM-exposed tracing API
// ============================================================================

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Print timing summary (WASM version)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = printRscvTimings)]
pub fn print_rscv_timings() {
    LEGACY_TIMINGS.with(|timings| {
        let timings = timings.borrow();
        web_sys::console::log_1(&"=== RSCV Timing Summary ===".into());
        let mut entries: Vec<_> = timings.iter().collect();
        entries.sort_by(|a, b| b.1.total_ms.partial_cmp(&a.1.total_ms).unwrap());
        for (label, stats) in entries {
            web_sys::console::log_1(
                &format!(
                    "{}: count={}, total={:.2}ms, avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                    label,
                    stats.count,
                    stats.total_ms,
                    stats.avg_ms(),
                    stats.min_ms,
                    stats.max_ms
                )
                .into(),
            );
        }
    });
}

/// Clear timing stats (WASM version)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = clearRscvTimings)]
pub fn clear_rscv_timings() {
    LEGACY_TIMINGS.with(|timings| {
        timings.borrow_mut().clear();
    });
}

/// Get timing stats as a JS Map for integration with the JS tracing system
/// Returns Map<string, {count: number, totalMs: number, minMs: number, maxMs: number, avgMs: number}>
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = getRscvTimings)]
pub fn get_rscv_timings() -> JsValue {
    LEGACY_TIMINGS.with(|timings| {
        let timings = timings.borrow();
        let result = js_sys::Map::new();

        for (label, stats) in timings.iter() {
            let entry = js_sys::Object::new();
            js_sys::Reflect::set(&entry, &"count".into(), &(stats.count as f64).into()).ok();
            js_sys::Reflect::set(&entry, &"totalMs".into(), &stats.total_ms.into()).ok();
            js_sys::Reflect::set(&entry, &"minMs".into(), &stats.min_ms.into()).ok();
            js_sys::Reflect::set(&entry, &"maxMs".into(), &stats.max_ms.into()).ok();
            js_sys::Reflect::set(&entry, &"avgMs".into(), &stats.avg_ms().into()).ok();

            result.set(&(*label).into(), &entry);
        }

        result.into()
    })
}

/// WASM-exposed span handle for use from JavaScript
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmSpan {
    inner: Option<Span>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmSpan {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        Self {
            inner: Some(Span::new(name)),
        }
    }

    #[wasm_bindgen(js_name = setAttribute)]
    pub fn set_attribute(&mut self, key: &str, value: JsValue) {
        if let Some(span) = &mut self.inner {
            if let Some(s) = value.as_string() {
                span.set_attribute_str(key, &s);
            } else if let Some(n) = value.as_f64() {
                if n.fract() == 0.0 {
                    span.set_attribute_int(key, n as i64);
                } else {
                    span.set_attribute_float(key, n);
                }
            } else if let Some(b) = value.as_bool() {
                span.set_attribute_bool(key, b);
            }
        }
    }

    pub fn end(&mut self) {
        if let Some(mut span) = self.inner.take() {
            span.end();
        }
    }
}
