//! Unified tracing infrastructure for Alizarin WASM
//!
//! This module provides performance tracing that integrates with the browser's
//! Performance API (performance.mark/measure), which can then be collected
//! by the JS-side tracing infrastructure.
//!
//! The approach uses the W3C Performance API as common ground between
//! Rust/WASM and JavaScript, avoiding the complexity of full OpenTelemetry
//! in WASM while maintaining semantic compatibility.

use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// ============================================================================
// Performance API bindings
// ============================================================================

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

// ============================================================================
// Span implementation
// ============================================================================

/// A tracing span that records timing via the Performance API
///
/// When dropped, the span automatically records a performance.measure()
/// that can be collected by the JS tracing infrastructure.
pub struct Span {
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
        let start_mark = format!("alizarin:{}:start:{}", name, now() as u64);
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
        self.attributes.insert(key.to_string(), SpanValue::String(value.to_string()));
        self
    }

    /// Add an integer attribute to the span
    pub fn set_attribute_int(&mut self, key: &str, value: i64) -> &mut Self {
        self.attributes.insert(key.to_string(), SpanValue::Int(value));
        self
    }

    /// Add a float attribute to the span
    pub fn set_attribute_float(&mut self, key: &str, value: f64) -> &mut Self {
        self.attributes.insert(key.to_string(), SpanValue::Float(value));
        self
    }

    /// Add a boolean attribute to the span
    pub fn set_attribute_bool(&mut self, key: &str, value: bool) -> &mut Self {
        self.attributes.insert(key.to_string(), SpanValue::Bool(value));
        self
    }

    /// End the span and record the measurement
    pub fn end(&mut self) {
        if self.ended {
            return;
        }
        self.ended = true;

        let end_mark = format!("alizarin:{}:end:{}", self.name, now() as u64);
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
        if self.count == 0 { 0.0 } else { self.total_ms / self.count as f64 }
    }
}

thread_local! {
    static LEGACY_TIMINGS: RefCell<HashMap<&'static str, LegacyTimingStats>> = RefCell::new(HashMap::new());
}

/// Record a timing measurement (legacy compatibility)
pub fn record_timing(label: &'static str, ms: f64) {
    LEGACY_TIMINGS.with(|timings| {
        timings.borrow_mut().entry(label).or_default().record(ms);
    });

    // Also emit as a Performance measure for the new system
    // Note: This creates a measure without marks, using duration directly
    let measure_name = format!("alizarin:{}", label);
    let options = js_sys::Object::new();

    // Use current time minus duration as start
    let end_time = now();
    let start_time = end_time - ms;

    js_sys::Reflect::set(&options, &"start".into(), &start_time.into()).ok();
    js_sys::Reflect::set(&options, &"end".into(), &end_time.into()).ok();

    let detail = js_sys::Object::new();
    js_sys::Reflect::set(&detail, &"source".into(), &"legacy".into()).ok();
    js_sys::Reflect::set(&options, &"detail".into(), &detail).ok();

    measure_with_options(&measure_name, &options);
}

/// Print timing summary (legacy compatibility)
#[wasm_bindgen(js_name = printRscvTimings)]
pub fn print_rscv_timings() {
    LEGACY_TIMINGS.with(|timings| {
        let timings = timings.borrow();
        web_sys::console::log_1(&"=== RSCV Timing Summary ===".into());
        let mut entries: Vec<_> = timings.iter().collect();
        entries.sort_by(|a, b| b.1.total_ms.partial_cmp(&a.1.total_ms).unwrap());
        for (label, stats) in entries {
            web_sys::console::log_1(&format!(
                "{}: count={}, total={:.2}ms, avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                label, stats.count, stats.total_ms, stats.avg_ms(), stats.min_ms, stats.max_ms
            ).into());
        }
    });
}

/// Clear timing stats (legacy compatibility)
#[wasm_bindgen(js_name = clearRscvTimings)]
pub fn clear_rscv_timings() {
    LEGACY_TIMINGS.with(|timings| {
        timings.borrow_mut().clear();
    });
}

/// Get timing stats as a JS Map for integration with the JS tracing system
/// Returns Map<string, {count: number, totalMs: number, minMs: number, maxMs: number, avgMs: number}>
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

// ============================================================================
// WASM-exposed tracing API
// ============================================================================

/// WASM-exposed span handle for use from JavaScript
#[wasm_bindgen]
pub struct WasmSpan {
    inner: Option<Span>,
}

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
