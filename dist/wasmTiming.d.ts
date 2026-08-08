interface TimingStats {
    count: number;
    totalMs: number;
    minMs: number;
    maxMs: number;
}
export declare function recordNativeTiming(label: string, ms: number): void;
export declare function printNativeTimings(): void;
export declare function clearNativeTimings(): void;
export declare function getNativeTimings(): Map<string, TimingStats>;
/** @deprecated Use recordNativeTiming */
export declare const recordWasmTiming: typeof recordNativeTiming;
/** @deprecated Use printNativeTimings */
export declare const printWasmTimings: typeof printNativeTimings;
/** @deprecated Use clearNativeTimings */
export declare const clearWasmTimings: typeof clearNativeTimings;
/** @deprecated Use getNativeTimings */
export declare const getWasmTimings: typeof getNativeTimings;
export {};
