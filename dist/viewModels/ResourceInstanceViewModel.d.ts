import { IStringKeyedObject, IInstanceWrapper, IModelWrapper, IViewModel, IPseudo, IRIVM, GetMeta } from "../interfaces";
import { StaticTile, StaticNode } from "../static-types";
import { viewContext } from "./types";
import { ResourceInstanceCacheEntry, ResourceDescriptors } from "./cacheEntries";
export declare class ResourceInstanceViewModel<RIVM extends IRIVM<RIVM>> implements IStringKeyedObject {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined;
    $: IInstanceWrapper<RIVM> | null;
    __: IModelWrapper<RIVM> | null;
    __parentPseudo: IPseudo | undefined;
    __cacheEntry: ResourceInstanceCacheEntry | null;
    id: string;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    gm: typeof viewContext.graphManager;
    toString(): string;
    getName(retrieveIfNeeded?: boolean): Promise<string>;
    getSlug(retrieveIfNeeded?: boolean): Promise<string>;
    getDescription(retrieveIfNeeded?: boolean): Promise<string>;
    getMapPopup(retrieveIfNeeded?: boolean): Promise<string>;
    getDescriptors(retrieveIfNeeded?: boolean): Promise<ResourceDescriptors | undefined>;
    __has(key: string): Promise<boolean | undefined>;
    __asTileData(): Promise<IStringKeyedObject>;
    __forJsonCache(getMeta: GetMeta): Promise<ResourceInstanceCacheEntry>;
    forJson(cascade?: boolean): Promise<{
        type: string;
        graphId: string;
        id: string;
        title: string;
        descriptors: ResourceDescriptors;
        meta: {
            [key: string]: any;
        };
        root: any;
    }>;
    /**
     * Get JSON representation with display-friendly values.
     *
     * Unlike forJson() which returns tile data format (language maps, StaticReference objects),
     * this returns human-readable strings using registered display serializers.
     *
     * Use this for ETL/export/indexing where you want display strings instead of structured data.
     *
     * @param cascade - If true, ensures the resource is fully populated before serializing
     * @param language - Language code for display strings (defaults to current language)
     * @returns JSON with display-friendly values
     */
    forDisplayJson(cascade?: boolean, language?: string): Promise<{
        type: string;
        graphId: string;
        id: string;
        title: string;
        descriptors: ResourceDescriptors;
        meta: {
            [key: string]: any;
        };
        root: any;
    }>;
    retrieve(): Promise<[IInstanceWrapper<RIVM>, IModelWrapper<RIVM>]>;
    constructor(id: string, modelWrapper: IModelWrapper<RIVM> | null, instanceWrapperFactory: ((rivm: RIVM) => IInstanceWrapper<RIVM>) | null, cacheEntry: object | null);
    static __create(tile: StaticTile, node: StaticNode, value: any, cacheEntry: ResourceInstanceCacheEntry | null): Promise<ResourceInstanceViewModel<any> | null>;
}
