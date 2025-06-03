import { IStringKeyedObject, IInstanceWrapper, IModelWrapper, IViewModel, IPseudo, IGraphManager, IRIVM, GetMeta } from './interfaces.ts';
import { PseudoValue } from './pseudos';
import { StaticNodeConfigBoolean } from './nodeConfig';
import { StaticDomainValue, StaticTile, StaticNode, StaticValue, StaticResourceReference } from './static-types';
declare const DEFAULT_LANGUAGE = "en";
declare class ViewContext {
    graphManager: IGraphManager | undefined;
}
declare const viewContext: ViewContext;
declare class ValueList<T extends IRIVM<T>> {
    values: Map<string, any>;
    wrapper: IInstanceWrapper<T>;
    tiles: StaticTile[] | null;
    promises: Map<string, boolean | Promise<boolean | IViewModel>>;
    writeLock: null | Promise<boolean | IViewModel>;
    constructor(values: Map<string, any>, allNodegroups: Map<string, boolean>, wrapper: IInstanceWrapper<T>, tiles: StaticTile[] | null);
    get(key: string): Promise<any>;
    set(key: string, value: any): void;
    has(key: string): Promise<boolean>;
    retrieve(key: string, dflt?: any, raiseError?: boolean): Promise<any>;
    setDefault(key: string, value: any): Promise<any>;
}
declare class ConceptValueCacheEntry implements IStringKeyedObject {
    [key: string]: any;
    datatype: string;
    id: string;
    value: string;
    conceptId: string | null;
    meta: {
        [key: string]: any;
    };
    constructor({ meta, id, value, conceptId }: {
        meta: IStringKeyedObject | undefined;
        id: string;
        value: string;
        conceptId: string | null;
    });
}
declare class ResourceInstanceCacheEntry implements IStringKeyedObject {
    [key: string]: any;
    datatype: string;
    id: string;
    type: string;
    graphId: string;
    title: string | null;
    meta: {
        [key: string]: any;
    };
    constructor({ meta, id, type, graphId, title }: {
        meta: IStringKeyedObject | undefined;
        id: string;
        type: string;
        graphId: string;
        title: string | null;
    });
}
declare class ResourceInstanceViewModel<RIVM extends IRIVM<RIVM>> implements IStringKeyedObject {
    [key: string | symbol]: any;
    $: IInstanceWrapper<RIVM> | null;
    __: IModelWrapper<RIVM> | null;
    __parentPseudo: IPseudo | undefined;
    __cacheEntry: ResourceInstanceCacheEntry | null;
    id: string;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    gm: IGraphManager | undefined;
    toString(): string;
    __has(key: string): Promise<boolean | undefined>;
    __asTileData(): Promise<IStringKeyedObject>;
    __forJsonCache(getMeta: GetMeta): Promise<ResourceInstanceCacheEntry>;
    forJson(cascade?: boolean): Promise<StaticResourceReference>;
    retrieve(): Promise<[IInstanceWrapper<RIVM>, IModelWrapper<RIVM>]>;
    constructor(id: string, modelWrapper: IModelWrapper<RIVM> | null, instanceWrapperFactory: ((rivm: RIVM) => IInstanceWrapper<RIVM>) | null, cacheEntry: object | null);
    static __create(tile: StaticTile, node: StaticNode, value: any, cacheEntry: ResourceInstanceCacheEntry | null): Promise<ResourceInstanceViewModel<any> | null>;
}
declare class DomainValueViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    _value: StaticDomainValue | Promise<StaticDomainValue>;
    constructor(value: StaticDomainValue);
    forJson(): Promise<StaticDomainValue>;
    __forJsonCache(): null;
    getValue(): StaticDomainValue | Promise<StaticDomainValue>;
    lang(lang: string): Promise<string | undefined>;
    static __create(tile: StaticTile, node: StaticNode, value: any): Promise<DomainValueViewModel | null>;
    __asTileData(): Promise<string | null>;
}
declare class ConceptValueViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: IPseudo | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    _value: StaticValue | Promise<StaticValue>;
    constructor(value: StaticValue);
    forJson(): Promise<StaticValue>;
    __forJsonCache(getMeta: GetMeta): Promise<ConceptValueCacheEntry>;
    getValue(): StaticValue | Promise<StaticValue>;
    static __create(tile: StaticTile, node: StaticNode, value: any, cacheEntry: ConceptValueCacheEntry | null): Promise<ConceptValueViewModel | null>;
    __asTileData(): Promise<string | null>;
}
declare class DateViewModel extends Date implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    __original: string;
    then: undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    __forJsonCache(): null;
    constructor(val: string);
    static __create(tile: StaticTile, node: StaticNode, value: any): DateViewModel | Promise<DateViewModel | null> | null;
    forJson(): Promise<string>;
    __asTileData(): string;
}
declare class GeoJSONViewModel implements IViewModel, IStringKeyedObject {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    _value: {
        [key: string]: any;
    };
    __forJsonCache(): null;
    constructor(jsonData: {
        [key: string]: any;
    });
    static __create(tile: StaticTile, node: StaticNode, value: any): GeoJSONViewModel | Promise<GeoJSONViewModel | null> | null;
    forJson(): Promise<{
        [key: string]: any;
    }>;
    __asTileData(): {
        [key: string]: any;
    };
}
declare class StringTranslatedLanguage {
    value: string;
}
declare class NonLocalizedStringViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    __forJsonCache(): null;
    forJson(): string;
    static __create(tile: StaticTile, node: StaticNode, value: any): NonLocalizedStringViewModel | Promise<NonLocalizedStringViewModel | null> | null;
    __asTileData(): string;
}
declare class NumberViewModel extends Number implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    toString(): string;
    __forJsonCache(): null;
    forJson(): number;
    static __create(tile: StaticTile, node: StaticNode, value: any): NumberViewModel | Promise<NumberViewModel | null> | null;
    __asTileData(): boolean;
}
declare class BooleanViewModel extends Boolean implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    __config: StaticNodeConfigBoolean;
    describeField: () => any;
    describeFieldGroup: () => any;
    constructor(value: boolean, config: StaticNodeConfigBoolean);
    toString(lang?: string | undefined): string;
    __forJsonCache(): null;
    forJson(): boolean;
    static __create(tile: StaticTile, node: StaticNode, value: any): BooleanViewModel | Promise<BooleanViewModel | null> | null;
    __asTileData(): boolean;
}
declare class Url {
    url: string;
    url_label?: string;
    constructor(url: string, url_label?: string);
}
declare class UrlViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    _value: Url;
    __forJsonCache(): null;
    constructor(value: Url);
    forJson(): {
        [key: string]: string;
    };
    label(): string;
    href(): string;
    static __create(tile: StaticTile, node: StaticNode, value: any): UrlViewModel | Promise<UrlViewModel | null> | null;
    __asTileData(): {
        [key: string]: string;
    };
}
declare class StringViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue | undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    _value: Map<string, StringTranslatedLanguage>;
    __forJsonCache(): null;
    constructor(value: Map<string, StringTranslatedLanguage>, language?: string | null);
    forJson(): string;
    lang(language: string): string | undefined;
    static __create(tile: StaticTile, node: StaticNode, value: any): StringViewModel | Promise<StringViewModel | null> | null;
    __asTileData(): Map<string, StringTranslatedLanguage>;
}
declare class SemanticViewModel implements IStringKeyedObject, IViewModel {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    __parentPseudo: PseudoValue | undefined;
    __childValues: Map<string, any>;
    __parentWkri: IRIVM<any> | null;
    __childNodes: Map<string, StaticNode>;
    __tile: StaticTile | null;
    __node: StaticNode;
    __forJsonCache(): null;
    constructor(parentWkri: IRIVM<any> | null, childNodes: Map<string, StaticNode>, tile: StaticTile | null, node: StaticNode);
    toString(): Promise<string>;
    toObject(): Promise<any>;
    forJson(): Promise<any>;
    __update(map: Map<string, any>): Promise<void[]>;
    __get(key: string): Promise<IViewModel | IViewModel[] | null>;
    __set(key: string, value: any): Promise<void>;
    __has(key: string): boolean;
    __getChildTypes(): Promise<Map<string, any>>;
    __getChildren(direct?: null | boolean): Promise<any[]>;
    __getChildValue(key: string, setDefault?: boolean): Promise<IPseudo>;
    __makePseudo(key: string): IPseudo;
    static __create(tile: StaticTile, node: StaticNode, value: any, parent: IRIVM<any> | null, childNodes: Map<string, StaticNode>): Promise<SemanticViewModel>;
    __asTileData(): Promise<(any[] | null)[]>;
    __getChildValues(): Promise<Map<string, IPseudo>>;
}
declare const CUSTOM_DATATYPES: Map<string, string | IViewModel>;
declare function getViewModel<RIVM extends IRIVM<RIVM>>(parentPseudo: PseudoValue, tile: StaticTile, node: StaticNode, data: any, parent: IRIVM<RIVM> | null, childNodes: Map<string, StaticNode>, isInner?: boolean): Promise<IViewModel | null>;
export { ResourceInstanceCacheEntry, DEFAULT_LANGUAGE, ResourceInstanceViewModel, ValueList, getViewModel, DomainValueViewModel, SemanticViewModel, StringViewModel, DateViewModel, GeoJSONViewModel, ConceptValueViewModel, viewContext, NonLocalizedStringViewModel, CUSTOM_DATATYPES, BooleanViewModel, NumberViewModel, UrlViewModel };
