import { IViewModel, IPseudo, GetMeta } from "../interfaces";
import { StaticTile, StaticNode } from "../static-types";
import { ConceptListCacheEntry } from "./cacheEntries";
import { ConceptValueViewModel } from "./ConceptValueViewModel";
export declare class ConceptListViewModel extends Array implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: IPseudo | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: Promise<(ConceptValueViewModel | null)[]> | null;
    forJson(): Promise<Promise<import("../static-types").StaticValue>[]>;
    __forJsonCache(getMeta: GetMeta): Promise<ConceptListCacheEntry>;
    static __create(tile: StaticTile, node: StaticNode, value: any, cacheEntry?: ConceptListCacheEntry | null): Promise<ConceptListViewModel>;
    __asTileData(): Promise<ConceptValueViewModel[]>;
}
