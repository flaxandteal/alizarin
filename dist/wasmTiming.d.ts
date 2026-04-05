interface TimingStats {
    count: number;
    totalMs: number;
    minMs: number;
    maxMs: number;
}
export declare function recordWasmTiming(label: string, ms: number): void;
export declare function printWasmTimings(): void;
export declare function clearWasmTimings(): void;
export declare function getWasmTimings(): Map<string, TimingStats>;
export {};
