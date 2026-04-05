import { IStringKeyedObject, IViewModel, IPseudo, IRIVM } from "./interfaces.ts";
import type { PseudoValue } from "./pseudos";
import { StaticTile, StaticNode } from "./static-types";
export declare function resetTimingStats(): void;
export declare function getTimingStats(): {
    wasmAvgMs: number;
    wrapAvgMs: number;
    wasmCalls: number;
    wasmTotalMs: number;
    wrapCalls: number;
    wrapTotalMs: number;
};
export declare function logTimingStats(label?: string): void;
declare class SemanticViewModel implements IStringKeyedObject, IViewModel {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    get [Symbol.toStringTag](): string;
    __parentPseudo: PseudoValue<any> | undefined;
    __childValues: Map<string, any>;
    __parentWkri: IRIVM<any> | null;
    __tile: StaticTile | null;
    __node: StaticNode;
    __forJsonCache(): null;
    constructor(parentWkri: IRIVM<any> | null, tile: StaticTile | null, node: StaticNode);
    toString(): Promise<string>;
    toObject(): Promise<Record<string, any>>;
    __get(key: string): Promise<IViewModel | IViewModel[]>;
    __has(key: string): any;
    __getChildValue(key: string): Promise<IPseudo> | null | undefined;
    static __create(tile: StaticTile, node: StaticNode, _value: any, parent: IRIVM<any> | null): Promise<SemanticViewModel>;
    __asTileData(): Promise<[null, any[]]>;
}
export { SemanticViewModel };
