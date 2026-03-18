// ============================================================================
// JS-side timing for WASM boundary crossings
// Extracted to separate module to avoid circular imports
// ============================================================================

interface TimingStats {
  count: number;
  totalMs: number;
  minMs: number;
  maxMs: number;
}

const wasmTimings: Map<string, TimingStats> = new Map();

export function recordWasmTiming(label: string, ms: number) {
  let stats = wasmTimings.get(label);
  if (!stats) {
    stats = { count: 0, totalMs: 0, minMs: Infinity, maxMs: -Infinity };
    wasmTimings.set(label, stats);
  }
  stats.count++;
  stats.totalMs += ms;
  stats.minMs = Math.min(stats.minMs, ms);
  stats.maxMs = Math.max(stats.maxMs, ms);
}

export function printWasmTimings() {
  console.log("=== JS-side WASM Timing Summary ===");
  const entries = [...wasmTimings.entries()].sort((a, b) => b[1].totalMs - a[1].totalMs);
  for (const [label, stats] of entries) {
    const avgMs = stats.count > 0 ? stats.totalMs / stats.count : 0;
    console.log(
      `${label}: count=${stats.count}, total=${stats.totalMs.toFixed(2)}ms, avg=${avgMs.toFixed(2)}ms, min=${stats.minMs.toFixed(2)}ms, max=${stats.maxMs.toFixed(2)}ms`
    );
  }
}

export function clearWasmTimings() {
  wasmTimings.clear();
}

// Get raw wasmTimings Map for integration with tracing infrastructure
export function getWasmTimings(): Map<string, TimingStats> {
  return wasmTimings;
}
