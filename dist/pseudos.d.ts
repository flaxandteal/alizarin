import { StaticTile, StaticNode } from "./static-types";
import { IViewModel, IPseudo, IRIVM } from "./interfaces";
import { AttrPromise } from "./utils";
import { PseudoNode, WasmPseudoValue, WasmPseudoList } from '../pkg/alizarin';
declare class PseudoUnavailable implements IPseudo {
    parentValue: PseudoValue<any> | null;
    tile: null;
    node: StaticNode;
    isOuter: boolean;
    constructor(node: StaticNode);
    forJson(): Promise<{
        [key: string]: any;
    }[] | null>;
    describeField(): string;
    describeFieldGroup(): string;
    getValue(): AttrPromise<null>;
    getLength(): number;
    getChildren(_?: boolean): any[];
    isIterable(): boolean;
}
/**
 * PseudoValue - Thin JS wrapper around WasmPseudoValue from Rust
 *
 * Most state and logic now lives in Rust's RustPseudoValue/WasmPseudoValue.
 * This wrapper provides:
 * 1. JS-specific getViewModel integration
 * 2. Promise-based value loading with AttrPromise
 * 3. Inner/outer pattern coordination
 */
declare class PseudoValue<VM extends IViewModel> implements IPseudo {
    private _wasm;
    parent: IRIVM<any> | null;
    _parentValue: PseudoValue<any> | null;
    private _inner;
    private _cachedValue;
    private _snapshot;
    private getSnapshot;
    get datatype(): string;
    get isInner(): boolean;
    get isOuter(): boolean;
    get tile(): StaticTile | null;
    set tile(t: StaticTile | null);
    get value(): any;
    get valueLoaded(): boolean | undefined;
    get accessed(): boolean;
    get independent(): boolean;
    get originalTile(): StaticTile | null;
    get node(): WasmPseudoValue;
    get nodeid(): string;
    get inner(): PseudoValue<any> | null;
    get parentValue(): PseudoValue<any> | null;
    set parentValue(newParentValue: PseudoValue<any> | null);
    isIterable(): boolean;
    describeField(): string;
    describeFieldGroup(): string;
    /**
     * Create PseudoValue from WasmPseudoValue (preferred constructor)
     */
    static fromWasm<VM extends IViewModel>(wasm: WasmPseudoValue, parent: IRIVM<any>): PseudoValue<VM>;
    /**
     * Private constructor - use static factories
     */
    private constructor();
    static create<VM extends IViewModel>(nodeOrAlias: StaticNode | string, tile: StaticTile | null, value: any, parent: IRIVM<any>): PseudoValue<VM>;
    getParentTileId(): string | null;
    getTile(): Promise<[StaticTile | null, any[]]>;
    clear(): void;
    updateValue(newTile?: StaticTile | null): AttrPromise<VM>;
    private _updateValueReal;
    /**
     * Sync the ViewModel's tile data to Rust.
     * Called after VM creation/resolution so toTiles()/toResource() can serialize.
     */
    private syncTileData;
    getValue(): AttrPromise<VM | null>;
    getLength(): number;
    getChildTypes(): Promise<object>;
    getChildren(direct?: boolean | null): IPseudo[];
    forJson(): Promise<{
        [key: string]: any;
    } | {
        [key: string]: any;
    }[] | string | number | boolean | null>;
}
declare class PseudoList extends Array implements IPseudo {
    node: StaticNode | undefined;
    parent: IRIVM<any> | null | undefined;
    parentValue: PseudoValue<any> | null;
    tile: StaticTile | undefined;
    parenttileId: string | undefined;
    ghostChildren: Set<PseudoValue<any>> | null;
    isOuter: boolean;
    /** Whether this list represents a cardinality-1 node (should unwrap to single value in forJson) */
    isSingle: boolean;
    isIterable(): boolean;
    sorted(): Promise<any[]>;
    describeField(): string;
    describeFieldGroup(): string;
    initialize(node: StaticNode, parent: IRIVM<any> | null): void;
    forJson(): Promise<{
        [key: string]: any;
    }[] | {
        [key: string]: any;
    } | null>;
    getValue(): AttrPromise<PseudoList | IViewModel | null>;
    toString(): string;
}
/**
 * Thin wrapper to convert Rust WasmPseudoValue/WasmPseudoList to TS PseudoValue/PseudoList
 * This is the ONLY place where Rust values should be wrapped in TS classes
 *
 * @param rustValue - WasmPseudoValue or WasmPseudoList from Rust
 * @param wkri - The WKRI wrapper (parent IRIVM)
 * @param model - The model wrapper (for getting nodes - currently unused with new pattern)
 * @returns PseudoValue, PseudoList, or PseudoUnavailable
 */
declare function wrapRustPseudo(rustValue: WasmPseudoValue | WasmPseudoList | null, wkri: IRIVM<any>, model: any): PseudoValue<any> | PseudoList | PseudoUnavailable | null | undefined;
export { PseudoNode, PseudoValue, PseudoList, PseudoUnavailable, wrapRustPseudo };
