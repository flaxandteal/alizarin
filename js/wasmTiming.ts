// ============================================================================
// JS-side timing for Rust boundary crossings (WASM or NAPI)
// Extracted to separate module to avoid circular imports
// ============================================================================

interface TimingStats {
  count: number;
  totalMs: number;
  minMs: number;
  maxMs: number;
}

const nativeTimings: Map<string, TimingStats> = new Map();

export function recordNativeTiming(label: string, ms: number) {
  let stats = nativeTimings.get(label);
  if (!stats) {
    stats = { count: 0, totalMs: 0, minMs: Infinity, maxMs: -Infinity };
    nativeTimings.set(label, stats);
  }
  stats.count++;
  stats.totalMs += ms;
  stats.minMs = Math.min(stats.minMs, ms);
  stats.maxMs = Math.max(stats.maxMs, ms);
}

export function printNativeTimings() {
  console.log("=== JS-side Native Timing Summary ===");
  const entries = [...nativeTimings.entries()].sort((a, b) => b[1].totalMs - a[1].totalMs);
  for (const [label, stats] of entries) {
    const avgMs = stats.count > 0 ? stats.totalMs / stats.count : 0;
    console.log(
      `${label}: count=${stats.count}, total=${stats.totalMs.toFixed(2)}ms, avg=${avgMs.toFixed(2)}ms, min=${stats.minMs.toFixed(2)}ms, max=${stats.maxMs.toFixed(2)}ms`
    );
  }
}

export function clearNativeTimings() {
  nativeTimings.clear();
}

export function getNativeTimings(): Map<string, TimingStats> {
  return nativeTimings;
}

/** @deprecated Use recordNativeTiming */
export const recordWasmTiming = recordNativeTiming;
/** @deprecated Use printNativeTimings */
export const printWasmTimings = printNativeTimings;
/** @deprecated Use clearNativeTimings */
export const clearWasmTimings = clearNativeTimings;
/** @deprecated Use getNativeTimings */
export const getWasmTimings = getNativeTimings;
