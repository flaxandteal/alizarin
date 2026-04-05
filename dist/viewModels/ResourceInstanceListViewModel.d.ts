import { IViewModel, IPseudo, GetMeta } from "../interfaces";
import { StaticTile, StaticNode } from "../static-types";
import { ResourceInstanceListCacheEntry } from "./cacheEntries";
import { ResourceInstanceViewModel } from "./ResourceInstanceViewModel";
export declare class ResourceInstanceListViewModel extends Array implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: IPseudo | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: Promise<(ResourceInstanceViewModel<any> | null)[]> | null;
    forJson(): Promise<Promise<{
        type: string;
        graphId: string;
        id: string;
        title: string;
        descriptors: import("./cacheEntries").ResourceDescriptors;
        meta: {
            [key: string]: any;
        };
        root: any;
    }>[]>;
    __forJsonCache(getMeta: GetMeta): Promise<ResourceInstanceListCacheEntry>;
    static __create(tile: StaticTile, node: StaticNode, value: any, cacheEntry?: ResourceInstanceListCacheEntry | null): Promise<ResourceInstanceListViewModel | null>;
    __asTileData(): Promise<ResourceInstanceViewModel<any>[]>;
}
