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
/**
 * Span class - represents a unit of work
 */
export declare class Span {
    private data;
    private ended;
    private tracer;
    constructor(name: string, tracer: Tracer, parentContext?: SpanContext, attributes?: SpanAttributes);
    get context(): SpanContext;
    get name(): string;
    setAttribute(key: string, value: string | number | boolean): this;
    setAttributes(attributes: SpanAttributes): this;
    addEvent(name: string, attributes?: SpanAttributes): this;
    setStatus(status: 'ok' | 'error', message?: string): this;
    recordException(error: Error): this;
    end(): void;
    getData(): SpanData;
}
/**
 * Tracer class - creates spans and manages export
 */
export declare class Tracer {
    private name;
    private version?;
    private exporters;
    private pendingSpans;
    private batchSize;
    private flushIntervalMs;
    private flushTimer?;
    constructor(name: string, version?: string);
    /**
     * Add an exporter to receive completed spans
     */
    addExporter(exporter: SpanExporter): this;
    /**
     * Start a new span
     */
    startSpan(name: string, attributes?: SpanAttributes): Span;
    /**
     * Start a span and make it the active span
     */
    startActiveSpan<T>(name: string, fn: (span: Span) => T, attributes?: SpanAttributes): T;
    startActiveSpan<T>(name: string, attributes: SpanAttributes, fn: (span: Span) => T): T;
    /**
     * Called when a span ends
     */
    onSpanEnd(spanData: SpanData): void;
    /**
     * Flush pending spans to exporters
     */
    flush(): void;
    /**
     * Get the current active span
     */
    getCurrentSpan(): Span | undefined;
}
/**
 * Get a tracer instance
 */
export declare function getTracer(name: string, version?: string): Tracer;
/**
 * Add a global exporter
 */
export declare function addGlobalExporter(exporter: SpanExporter): void;
/**
 * Flush all pending spans
 */
export declare function flushAll(): void;
/**
 * Console exporter - logs spans to console
 */
export declare function consoleExporter(spans: SpanData[]): void;
/**
 * Summary exporter - aggregates timing stats like the old system
 */
export declare class SummaryExporter {
    private stats;
    export: (spans: SpanData[]) => void;
    getSummary(): Map<string, {
        count: number;
        totalMs: number;
        avgMs: number;
        minMs: number;
        maxMs: number;
    }>;
    printSummary(label?: string): void;
    reset(): void;
    /**
     * Merge external timing stats into this summary
     * @param stats Map or object with timing stats in the format {count, totalMs, minMs, maxMs}
     * @param prefix Optional prefix to add to all stat names (e.g., "rust:" or "alizarin:")
     */
    mergeStats(stats: Map<string, {
        count: number;
        totalMs: number;
        minMs: number;
        maxMs: number;
    }> | Record<string, {
        count: number;
        totalMs: number;
        minMs: number;
        maxMs: number;
    }>, prefix?: string): void;
}
/**
 * Bridge for collecting performance entries from WASM
 *
 * The Rust code can use web-sys to call performance.mark() and performance.measure(),
 * and this function collects those entries and converts them to spans.
 */
export declare function collectWasmPerformanceEntries(tracer: Tracer): void;
/**
 * Decorator-style helper for timing a function
 */
export declare function traced<T extends (...args: any[]) => any>(name: string, fn: T, tracer?: Tracer): T;
/**
 * Time a block of code
 */
export declare function timed<T>(name: string, fn: () => T | Promise<T>, tracer?: Tracer): Promise<T>;
/**
 * Register the Rust timing getter (called from main.ts after WASM init)
 */
export declare function registerRustTimingGetter(getter: () => Map<string, any>): void;
/**
 * Register the alizarin JS timing getter (called from main.ts)
 */
export declare function registerAlizarinTimingGetter(getter: () => any): void;
/**
 * Register the detailed WASM timing getter (called from main.ts)
 */
export declare function registerWasmTimingGetter(getter: () => Map<string, any>): void;
/**
 * Collect all timing stats from Rust/WASM and alizarin JS into a SummaryExporter
 *
 * @param exporter The SummaryExporter to merge stats into
 */
export declare function collectAllTimings(exporter: SummaryExporter): void;
