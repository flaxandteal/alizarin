import { IViewModel, IPseudo, GetMeta } from "../interfaces";
import { StaticTile, StaticNode, StaticValue } from "../static-types";
import { ConceptValueCacheEntry } from "./cacheEntries";
export declare class ConceptValueViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: IPseudo | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: StaticValue | Promise<StaticValue>;
    _collectionId: string | null;
    constructor(value: StaticValue, collectionId?: string);
    /**
     * Get the parent concept value, if this concept has a parent in the hierarchy.
     * @returns A new ConceptValueViewModel for the parent, or null if no parent
     * @throws Error if the collection doesn't support hierarchy lookups
     */
    parent(): Promise<ConceptValueViewModel | null>;
    /**
     * Get all ancestor concept values, from immediate parent to root.
     * @returns Array of ConceptValueViewModels for ancestors
     */
    ancestors(): Promise<ConceptValueViewModel[]>;
    forJson(): Promise<StaticValue>;
    __forJsonCache(getMeta: GetMeta): Promise<ConceptValueCacheEntry>;
    getValue(): StaticValue | Promise<StaticValue>;
    static __create(tile: StaticTile, node: StaticNode, value: any, cacheEntry: ConceptValueCacheEntry | null): Promise<ConceptValueViewModel | null>;
    __asTileData(): Promise<string>;
}
