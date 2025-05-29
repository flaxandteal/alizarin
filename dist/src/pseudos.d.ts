import { StaticTile, StaticNode } from './static-types';
import { IViewModel, IPseudo, IRIVM, IModelWrapper } from './interfaces';
import { AttrPromise } from './utils';
declare class PseudoUnavailable implements IPseudo {
    parentNode: PseudoValue | null;
    tile: null;
    node: StaticNode;
    isOuter: boolean;
    constructor(node: StaticNode);
    forJson(): Promise<{
        [key: string]: any;
    }[] | null>;
    describeField(): string;
    describeFieldGroup(): string;
    getValue(): Promise<null>;
    getLength(): number;
    getChildren(_?: boolean): never[];
    isIterable(): boolean;
}
declare class PseudoValue implements IPseudo {
    node: StaticNode;
    tile: StaticTile | null;
    value: any;
    parent: IRIVM<any> | null;
    parentNode: PseudoValue | null;
    valueLoaded: boolean | undefined;
    datatype: string | null;
    originalTile: StaticTile | null;
    accessed: boolean;
    childNodes: Map<string, StaticNode>;
    isOuter: boolean;
    isInner: boolean;
    inner: PseudoValue | null;
    independent: boolean;
    isIterable(): boolean;
    describeField(): string;
    describeFieldGroup(): string;
    constructor(node: StaticNode, tile: StaticTile | null, value: any, parent: IRIVM<any> | null, childNodes: Map<string, StaticNode>, inner: boolean | PseudoValue);
    getParentTileId(): string | null;
    getTile(): Promise<(any[] | StaticTile | null)[]>;
    clear(): void;
    updateValue(): AttrPromise<IViewModel>;
    getValue(): AttrPromise<IViewModel | null>;
    getLength(): number;
    getChildTypes(): Promise<{}>;
    getChildren(direct?: null): IPseudo[];
    forJson(): Promise<{
        [key: string]: any;
    } | {
        [key: string]: any;
    }[] | string | number | boolean | null>;
}
declare class PseudoList extends Array implements IPseudo {
    node: StaticNode | undefined;
    parent: IRIVM<any> | null | undefined;
    parentNode: PseudoValue | null;
    tile: StaticTile | undefined;
    parenttileId: string | undefined;
    ghostChildren: Set<PseudoValue> | null;
    isOuter: boolean;
    isIterable(): boolean;
    sorted(): Promise<any[]>;
    describeField(): string;
    describeFieldGroup(): string;
    initialize(node: StaticNode, parent: IRIVM<any> | null): void;
    forJson(): Promise<{
        [key: string]: any;
    }[]>;
    getValue(): AttrPromise<PseudoList>;
    toString(): string;
}
declare function makePseudoCls(model: IModelWrapper<any>, key: string, single: boolean, tile?: StaticTile | null, wkri?: any | null): PseudoList | PseudoValue | PseudoUnavailable;
export { PseudoValue, PseudoList, PseudoUnavailable, makePseudoCls };
