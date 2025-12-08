/**
 * Unified tracing infrastructure for Alizarin
 *
 * This module provides a consistent tracing API that works across:
 * - Browser (via Performance API)
 * - Node.js (via perf_hooks)
 * - Rust/WASM (via web-sys Performance API, bridged through here)
 *
 * The API is designed to be compatible with OpenTelemetry semantics,
 * making it easy to upgrade to full OTel later if needed.
 */

// Types
export interface SpanAttributes {
  [key: string]: string | number | boolean | undefined;
}

export interface SpanContext {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
}

export interface SpanData {
  name: string;
  context: SpanContext;
  startTime: number;
  endTime?: number;
  duration?: number;
  attributes: SpanAttributes;
  status: 'ok' | 'error' | 'unset';
  events: SpanEvent[];
}

export interface SpanEvent {
  name: string;
  timestamp: number;
  attributes?: SpanAttributes;
}

export type SpanExporter = (spans: SpanData[]) => void;

// Simple ID generation (not cryptographically secure, but fine for tracing)
function generateId(length: number): string {
  const chars = '0123456789abcdef';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars[Math.floor(Math.random() * chars.length)];
  }
  return result;
}

function generateTraceId(): string {
  return generateId(32);
}

function generateSpanId(): string {
  return generateId(16);
}

// Get high-resolution time
function now(): number {
  if (typeof performance !== 'undefined' && performance.now) {
    return performance.now();
  }
  // Fallback for environments without Performance API
  return Date.now();
}

/**
 * Active span tracking using a stack (for nested spans)
 */
class SpanStack {
  private stack: Span[] = [];

  push(span: Span): void {
    this.stack.push(span);
  }

  pop(): Span | undefined {
    return this.stack.pop();
  }

  current(): Span | undefined {
    return this.stack[this.stack.length - 1];
  }
}

const spanStack = new SpanStack();

/**
 * Span class - represents a unit of work
 */
export class Span {
  private data: SpanData;
  private ended: boolean = false;
  private tracer: Tracer;

  constructor(
    name: string,
    tracer: Tracer,
    parentContext?: SpanContext,
    attributes?: SpanAttributes
  ) {
    this.tracer = tracer;
    this.data = {
      name,
      context: {
        traceId: parentContext?.traceId ?? generateTraceId(),
        spanId: generateSpanId(),
        parentSpanId: parentContext?.spanId,
      },
      startTime: now(),
      attributes: attributes ?? {},
      status: 'unset',
      events: [],
    };
  }

  get context(): SpanContext {
    return this.data.context;
  }

  get name(): string {
    return this.data.name;
  }

  setAttribute(key: string, value: string | number | boolean): this {
    if (!this.ended) {
      this.data.attributes[key] = value;
    }
    return this;
  }

  setAttributes(attributes: SpanAttributes): this {
    if (!this.ended) {
      Object.assign(this.data.attributes, attributes);
    }
    return this;
  }

  addEvent(name: string, attributes?: SpanAttributes): this {
    if (!this.ended) {
      this.data.events.push({
        name,
        timestamp: now(),
        attributes,
      });
    }
    return this;
  }

  setStatus(status: 'ok' | 'error', message?: string): this {
    if (!this.ended) {
      this.data.status = status;
      if (message) {
        this.data.attributes['status.message'] = message;
      }
    }
    return this;
  }

  recordException(error: Error): this {
    if (!this.ended) {
      this.addEvent('exception', {
        'exception.type': error.name,
        'exception.message': error.message,
        'exception.stacktrace': error.stack,
      });
      this.setStatus('error', error.message);
    }
    return this;
  }

  end(): void {
    if (this.ended) return;

    this.ended = true;
    this.data.endTime = now();
    this.data.duration = this.data.endTime - this.data.startTime;

    // Remove from stack if it's the current span
    if (spanStack.current() === this) {
      spanStack.pop();
    }

    // Report to tracer
    this.tracer.onSpanEnd(this.data);
  }

  getData(): SpanData {
    return { ...this.data };
  }
}

/**
 * Tracer class - creates spans and manages export
 */
export class Tracer {
  private name: string;
  private version?: string;
  private exporters: SpanExporter[] = [];
  private pendingSpans: SpanData[] = [];
  private batchSize: number = 100;
  private flushIntervalMs: number = 5000;
  private flushTimer?: ReturnType<typeof setTimeout>;

  constructor(name: string, version?: string) {
    this.name = name;
    this.version = version;
  }

  /**
   * Add an exporter to receive completed spans
   */
  addExporter(exporter: SpanExporter): this {
    this.exporters.push(exporter);
    return this;
  }

  /**
   * Start a new span
   */
  startSpan(name: string, attributes?: SpanAttributes): Span {
    const parentSpan = spanStack.current();
    const span = new Span(name, this, parentSpan?.context, attributes);
    return span;
  }

  /**
   * Start a span and make it the active span
   */
  startActiveSpan<T>(
    name: string,
    fn: (span: Span) => T,
    attributes?: SpanAttributes
  ): T;
  startActiveSpan<T>(
    name: string,
    attributes: SpanAttributes,
    fn: (span: Span) => T
  ): T;
  startActiveSpan<T>(
    name: string,
    fnOrAttributes: ((span: Span) => T) | SpanAttributes,
    maybeFn?: (span: Span) => T
  ): T {
    let fn: (span: Span) => T;
    let attributes: SpanAttributes | undefined;

    if (typeof fnOrAttributes === 'function') {
      fn = fnOrAttributes;
    } else {
      attributes = fnOrAttributes;
      fn = maybeFn!;
    }

    const span = this.startSpan(name, attributes);
    spanStack.push(span);

    try {
      const result = fn(span);

      // Handle promises
      if (result instanceof Promise) {
        return result
          .then((value) => {
            span.setStatus('ok');
            span.end();
            return value;
          })
          .catch((error) => {
            span.recordException(error);
            span.end();
            throw error;
          }) as T;
      }

      span.setStatus('ok');
      span.end();
      return result;
    } catch (error) {
      span.recordException(error as Error);
      span.end();
      throw error;
    }
  }

  /**
   * Called when a span ends
   */
  onSpanEnd(spanData: SpanData): void {
    // Add tracer info
    spanData.attributes['tracer.name'] = this.name;
    if (this.version) {
      spanData.attributes['tracer.version'] = this.version;
    }

    this.pendingSpans.push(spanData);

    // Batch export
    if (this.pendingSpans.length >= this.batchSize) {
      this.flush();
    } else if (!this.flushTimer) {
      this.flushTimer = setTimeout(() => this.flush(), this.flushIntervalMs);
    }
  }

  /**
   * Flush pending spans to exporters
   */
  flush(): void {
    if (this.flushTimer) {
      clearTimeout(this.flushTimer);
      this.flushTimer = undefined;
    }

    if (this.pendingSpans.length === 0) return;

    const spans = this.pendingSpans;
    this.pendingSpans = [];

    for (const exporter of this.exporters) {
      try {
        exporter(spans);
      } catch (e) {
        console.error('Exporter error:', e);
      }
    }
  }

  /**
   * Get the current active span
   */
  getCurrentSpan(): Span | undefined {
    return spanStack.current();
  }
}

/**
 * TracerProvider - manages tracers (singleton pattern)
 */
class TracerProvider {
  private tracers: Map<string, Tracer> = new Map();
  private globalExporters: SpanExporter[] = [];

  getTracer(name: string, version?: string): Tracer {
    const key = `${name}@${version ?? 'unknown'}`;

    if (!this.tracers.has(key)) {
      const tracer = new Tracer(name, version);
      // Add global exporters to new tracer
      for (const exporter of this.globalExporters) {
        tracer.addExporter(exporter);
      }
      this.tracers.set(key, tracer);
    }

    return this.tracers.get(key)!;
  }

  addGlobalExporter(exporter: SpanExporter): this {
    this.globalExporters.push(exporter);
    // Add to existing tracers
    for (const tracer of this.tracers.values()) {
      tracer.addExporter(exporter);
    }
    return this;
  }

  flushAll(): void {
    for (const tracer of this.tracers.values()) {
      tracer.flush();
    }
  }
}

// Global tracer provider instance
const globalProvider = new TracerProvider();

/**
 * Get a tracer instance
 */
export function getTracer(name: string, version?: string): Tracer {
  return globalProvider.getTracer(name, version);
}

/**
 * Add a global exporter
 */
export function addGlobalExporter(exporter: SpanExporter): void {
  globalProvider.addGlobalExporter(exporter);
}

/**
 * Flush all pending spans
 */
export function flushAll(): void {
  globalProvider.flushAll();
}

// ============================================================================
// Built-in Exporters
// ============================================================================

/**
 * Console exporter - logs spans to console
 */
export function consoleExporter(spans: SpanData[]): void {
  console.log('=== Trace Spans ===');
  for (const span of spans) {
    const indent = span.context.parentSpanId ? '  ' : '';
    console.log(
      `${indent}[${span.name}] ${span.duration?.toFixed(2)}ms`,
      span.status !== 'unset' ? `(${span.status})` : '',
      Object.keys(span.attributes).length > 2 ? span.attributes : ''
    );
  }
}

/**
 * Summary exporter - aggregates timing stats like the old system
 */
export class SummaryExporter {
  private stats: Map<string, {
    count: number;
    totalMs: number;
    minMs: number;
    maxMs: number;
  }> = new Map();

  export = (spans: SpanData[]): void => {
    for (const span of spans) {
      const duration = span.duration ?? 0;
      const existing = this.stats.get(span.name);

      if (existing) {
        existing.count++;
        existing.totalMs += duration;
        existing.minMs = Math.min(existing.minMs, duration);
        existing.maxMs = Math.max(existing.maxMs, duration);
      } else {
        this.stats.set(span.name, {
          count: 1,
          totalMs: duration,
          minMs: duration,
          maxMs: duration,
        });
      }
    }
  };

  getSummary(): Map<string, {
    count: number;
    totalMs: number;
    avgMs: number;
    minMs: number;
    maxMs: number;
  }> {
    const result = new Map();
    for (const [name, stats] of this.stats) {
      result.set(name, {
        ...stats,
        avgMs: stats.count > 0 ? stats.totalMs / stats.count : 0,
      });
    }
    return result;
  }

  printSummary(label: string = ''): void {
    console.log(`=== Timing Summary ${label} ===`);
    const entries = [...this.stats.entries()]
      .sort((a, b) => b[1].totalMs - a[1].totalMs);

    for (const [name, stats] of entries) {
      const avg = stats.count > 0 ? stats.totalMs / stats.count : 0;
      console.log(
        `${name}: count=${stats.count}, total=${stats.totalMs.toFixed(2)}ms, ` +
        `avg=${avg.toFixed(2)}ms, min=${stats.minMs.toFixed(2)}ms, max=${stats.maxMs.toFixed(2)}ms`
      );
    }
  }

  reset(): void {
    this.stats.clear();
  }

  /**
   * Merge external timing stats into this summary
   * @param stats Map or object with timing stats in the format {count, totalMs, minMs, maxMs}
   * @param prefix Optional prefix to add to all stat names (e.g., "rust:" or "alizarin:")
   */
  mergeStats(
    stats: Map<string, { count: number; totalMs: number; minMs: number; maxMs: number }> | Record<string, { count: number; totalMs: number; minMs: number; maxMs: number }>,
    prefix: string = ''
  ): void {
    const entries = stats instanceof Map ? stats.entries() : Object.entries(stats);

    for (const [name, stat] of entries) {
      const fullName = prefix + name;
      const existing = this.stats.get(fullName);

      if (existing) {
        existing.count += stat.count;
        existing.totalMs += stat.totalMs;
        existing.minMs = Math.min(existing.minMs, stat.minMs);
        existing.maxMs = Math.max(existing.maxMs, stat.maxMs);
      } else {
        this.stats.set(fullName, {
          count: stat.count,
          totalMs: stat.totalMs,
          minMs: stat.minMs,
          maxMs: stat.maxMs,
        });
      }
    }
  }
}

// ============================================================================
// WASM Bridge
// ============================================================================

/**
 * Bridge for collecting performance entries from WASM
 *
 * The Rust code can use web-sys to call performance.mark() and performance.measure(),
 * and this function collects those entries and converts them to spans.
 */
export function collectWasmPerformanceEntries(tracer: Tracer): void {
  if (typeof performance === 'undefined' || !performance.getEntriesByType) {
    return;
  }

  const measures = performance.getEntriesByType('measure') as PerformanceMeasure[];

  for (const measure of measures) {
    // Only collect alizarin-prefixed measures
    if (!measure.name.startsWith('alizarin:')) {
      continue;
    }

    const spanName = measure.name.replace('alizarin:', '');
    const span = tracer.startSpan(spanName);

    // Parse detail if present (for attributes)
    if (measure.detail && typeof measure.detail === 'object') {
      span.setAttributes(measure.detail as SpanAttributes);
    }

    // Manually set timing (since this is retrospective)
    const spanData = span.getData();
    spanData.startTime = measure.startTime;
    spanData.endTime = measure.startTime + measure.duration;
    spanData.duration = measure.duration;
    spanData.attributes['source'] = 'wasm';

    span.end();
  }

  // Clear the collected measures
  for (const measure of measures) {
    if (measure.name.startsWith('alizarin:')) {
      performance.clearMeasures(measure.name);
    }
  }
}

// ============================================================================
// Convenience helpers
// ============================================================================

/**
 * Decorator-style helper for timing a function
 */
export function traced<T extends (...args: any[]) => any>(
  name: string,
  fn: T,
  tracer?: Tracer
): T {
  const t = tracer ?? getTracer('alizarin');

  return ((...args: Parameters<T>): ReturnType<T> => {
    return t.startActiveSpan(name, (span) => {
      span.setAttribute('args.count', args.length);
      return fn(...args);
    });
  }) as T;
}

/**
 * Time a block of code
 */
export async function timed<T>(
  name: string,
  fn: () => T | Promise<T>,
  tracer?: Tracer
): Promise<T> {
  const t = tracer ?? getTracer('alizarin');
  return t.startActiveSpan(name, async (span) => {
    const result = await fn();
    return result;
  });
}

// ============================================================================
// Cross-platform timing collection
// ============================================================================

// These will be set by the main module when WASM is available
let _getRscvTimings: (() => Map<string, any>) | null = null;
let _getAlizarinTimingStats: (() => any) | null = null;

/**
 * Register the Rust timing getter (called from main.ts after WASM init)
 */
export function registerRustTimingGetter(getter: () => Map<string, any>): void {
  _getRscvTimings = getter;
}

/**
 * Register the alizarin JS timing getter (called from main.ts)
 */
export function registerAlizarinTimingGetter(getter: () => any): void {
  _getAlizarinTimingStats = getter;
}

/**
 * Collect all timing stats from Rust/WASM and alizarin JS into a SummaryExporter
 *
 * @param exporter The SummaryExporter to merge stats into
 */
export function collectAllTimings(exporter: SummaryExporter): void {
  // Collect Rust/WASM timings
  if (_getRscvTimings) {
    try {
      const rustTimings = _getRscvTimings();
      if (rustTimings && rustTimings.size > 0) {
        const rustStats: Record<string, { count: number; totalMs: number; minMs: number; maxMs: number }> = {};
        rustTimings.forEach((value: any, key: string) => {
          rustStats[key] = {
            count: value.count || 0,
            totalMs: value.totalMs || 0,
            minMs: value.minMs || 0,
            maxMs: value.maxMs || 0,
          };
        });
        exporter.mergeStats(rustStats, 'rust: ');
      }
    } catch (e) {
      console.warn('Failed to collect Rust timings:', e);
    }
  }

  // Collect alizarin JS timings
  if (_getAlizarinTimingStats) {
    try {
      const jsTimings = _getAlizarinTimingStats();
      if (jsTimings) {
        // Convert alizarin's flat stats format to our format
        const jsStats: Record<string, { count: number; totalMs: number; minMs: number; maxMs: number }> = {};

        if (jsTimings.wasmCalls > 0) {
          jsStats['wasm calls'] = {
            count: jsTimings.wasmCalls,
            totalMs: jsTimings.wasmTotalMs,
            minMs: jsTimings.wasmTotalMs / jsTimings.wasmCalls, // approximate
            maxMs: jsTimings.wasmTotalMs / jsTimings.wasmCalls, // approximate
          };
        }
        if (jsTimings.wrapCalls > 0) {
          jsStats['wrap calls'] = {
            count: jsTimings.wrapCalls,
            totalMs: jsTimings.wrapTotalMs,
            minMs: jsTimings.wrapTotalMs / jsTimings.wrapCalls,
            maxMs: jsTimings.wrapTotalMs / jsTimings.wrapCalls,
          };
        }
        if (jsTimings.forJsonCalls > 0) {
          jsStats['forJson calls'] = {
            count: jsTimings.forJsonCalls,
            totalMs: jsTimings.forJsonTotalMs,
            minMs: jsTimings.forJsonTotalMs / jsTimings.forJsonCalls,
            maxMs: jsTimings.forJsonTotalMs / jsTimings.forJsonCalls,
          };
        }

        if (Object.keys(jsStats).length > 0) {
          exporter.mergeStats(jsStats, 'alizarin: ');
        }
      }
    } catch (e) {
      console.warn('Failed to collect alizarin JS timings:', e);
    }
  }
}
