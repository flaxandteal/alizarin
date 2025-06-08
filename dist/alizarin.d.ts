export declare const AlizarinModel: typeof viewModels.ResourceInstanceViewModel;

declare abstract class ArchesClient {
    abstract getGraphs(): Promise<GraphResult>;
    abstract getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;
    abstract getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;
    abstract getResources(graphId: string, limit: number): Promise<StaticResource[]>;
    abstract getResource(resourceId: string): Promise<StaticResource>;
    abstract getCollection(collectionId: string): Promise<StaticCollection>;
}

declare const archesClient: ArchesClientRemote;

declare class ArchesClientLocal extends ArchesClient {
    fs: any;
    allGraphFile: () => string;
    graphToGraphFile?: (graph: StaticGraphMeta) => string;
    graphIdToGraphFile: (graphId: string) => string;
    graphIdToResourcesFiles: (graphId: string) => string[];
    resourceIdToFile: (resourceId: string) => string;
    collectionIdToFile: (collectionId: string) => string;
    constructor({ allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile, }?: {
        allGraphFile?: () => string;
        graphToGraphFile?: (graph: StaticGraphMeta) => string;
        graphIdToGraphFile?: (graphId: string) => string;
        graphIdToResourcesFiles?: (graphId: string) => string[];
        resourceIdToFile?: (resourceId: string) => string;
        collectionIdToFile?: (collectionId: string) => string;
    });
    getGraphs(): Promise<GraphResult>;
    getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;
    getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number | null): Promise<StaticResource[]>;
}

declare class ArchesClientRemote extends ArchesClient {
    archesUrl: string;
    constructor(archesUrl: string);
    getGraphs(): Promise<GraphResult>;
    getGraph(graph: StaticGraphMeta): Promise<StaticGraph>;
    getGraphByIdOnly(graphId: string): Promise<StaticGraph>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number): Promise<StaticResource[]>;
}

declare class ArchesClientRemoteStatic extends ArchesClient {
    archesUrl: string;
    allGraphFile: () => string;
    graphToGraphFile?: (graph: StaticGraphMeta) => string;
    graphIdToGraphFile: (graphId: string) => string;
    graphIdToResourcesFiles: (graphId: string) => string[];
    resourceIdToFile: (resourceId: string) => string;
    collectionIdToFile: (collectionId: string) => string;
    constructor(archesUrl: string, { allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile, }?: {
        allGraphFile?: () => string;
        graphToGraphFile?: (graph: StaticGraphMeta) => string;
        graphIdToGraphFile?: (graphId: string) => string;
        graphIdToResourcesFiles?: (graphId: string) => string[];
        resourceIdToFile?: (resourceId: string) => string;
        collectionIdToFile?: (collectionId: string) => string;
    });
    getGraphs(): Promise<GraphResult>;
    getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;
    getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number): Promise<StaticResource[]>;
}

declare class AttrPromise<T> extends Promise<T> implements IStringKeyedObject {
    [key: string | symbol]: any;
    [Symbol.toPrimitive]: undefined;
    constructor(executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason: any) => void) => void);
}

declare abstract class BaseRenderer {
    render(asset: ResourceInstanceViewModel<any>): Promise<any>;
    abstract renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any>;
    abstract renderDate(value: DateViewModel, _depth: number): Promise<any>;
    abstract renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any>;
    abstract renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any>;
    abstract renderSemantic(value: SemanticViewModel, depth: number): Promise<any>;
    abstract renderBlock(block: {
        [key: string]: string;
    } | {
        [key: string]: string;
    }[], depth: number): any;
    abstract renderArray(value: any[], depth: number): Promise<any>;
    abstract renderString(value: string | StringViewModel | NonLocalizedStringViewModel, _depth: number): Promise<any>;
    abstract renderBoolean(value: boolean | BooleanViewModel, _depth: number): Promise<any>;
    abstract renderNumber(value: number | NumberViewModel, _depth: number): Promise<any>;
    abstract renderUrl(value: UrlViewModel, _depth: number): Promise<any>;
    renderValue(value: any, depth: number): Promise<any>;
}

declare class BooleanViewModel extends Boolean implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    __config: StaticNodeConfigBoolean;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    constructor(value: boolean, config: StaticNodeConfigBoolean);
    toString(lang?: string | undefined): string;
    __forJsonCache(): null;
    forJson(): boolean;
    static __create(tile: StaticTile, node: StaticNode, value: any): BooleanViewModel | Promise<BooleanViewModel | null> | null;
    __asTileData(): boolean;
}

declare type CheckPermission = ((nodegroupId: string, tile: StaticTile | null, node: Map<string, StaticNode>) => boolean);

declare class Cleanable extends String {
    __clean: string | undefined;
}

declare namespace client {
    export {
        archesClient,
        ArchesClient,
        ArchesClientRemoteStatic,
        ArchesClientRemote,
        ArchesClientLocal,
        GraphResult
    }
}
export { client }

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

declare class ConfigurationOptions {
    graphs: Array<string> | null | boolean;
    eagerLoadGraphs: boolean;
    constructor();
}

declare const CUSTOM_DATATYPES: Map<string, string | IViewModel>;

declare class DateViewModel extends Date implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    __original: string;
    then: undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    __forJsonCache(): null;
    constructor(val: string);
    static __create(tile: StaticTile, node: StaticNode, value: any): DateViewModel | Promise<DateViewModel | null> | null;
    forJson(): Promise<string>;
    __asTileData(): string;
}

declare const DEFAULT_LANGUAGE = "en";

declare class DomainValueViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    _value: StaticDomainValue | Promise<StaticDomainValue>;
    constructor(value: StaticDomainValue);
    forJson(): Promise<StaticDomainValue>;
    __forJsonCache(): null;
    getValue(): StaticDomainValue | Promise<StaticDomainValue>;
    lang(lang: string): Promise<string | undefined>;
    static __create(tile: StaticTile, node: StaticNode, value: any): Promise<DomainValueViewModel | null>;
    __asTileData(): Promise<string | null>;
}

declare class FlatMarkdownRenderer extends MarkdownRenderer {
    renderSemantic(vm: SemanticViewModel, depth: number): Promise<any>;
    renderArray(value: any, depth: number): Promise<any>;
    renderString(value: string | StringViewModel | NonLocalizedStringViewModel, _depth: number): Promise<any>;
}

declare class GeoJSONViewModel implements IViewModel, IStringKeyedObject {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
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

declare function getCurrentLanguage(): string;

declare type GetMeta = ((vm: IViewModel) => IStringKeyedObject) | undefined;

declare function getViewModel<RIVM extends IRIVM<RIVM>>(parentPseudo: PseudoValue<any>, tile: StaticTile, node: StaticNode, data: any, parent: IRIVM<RIVM> | null, childNodes: Map<string, StaticNode>, isInner?: boolean): Promise<IViewModel | null>;

export declare class GraphManager {
    _initialized: boolean;
    archesClient: ArchesClient;
    graphs: Map<string, ResourceModelWrapper<any>>;
    wkrms: Map<string, WKRM>;
    constructor(archesClient: ArchesClient);
    initialize(configurationOptions?: ConfigurationOptions | undefined): Promise<void>;
    loadGraph<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string): Promise<ResourceModelWrapper<RIVM>>;
    get<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string): Promise<ResourceModelWrapper<RIVM>>;
    getResource<T extends IRIVM<T>>(resourceId: string, lazy?: boolean, pruneTiles?: boolean): Promise<T>;
    getGraph(graphId: string): StaticGraph;
}

export declare const graphManager: GraphManager;

declare class GraphResult {
    models: {
        [graphId: string]: StaticGraphMeta;
    };
    constructor(jsonData: GraphResult);
}

declare interface IGraphManager {
    getResource<T extends IRIVM<T>>(resourceId: string, lazy: boolean): Promise<T>;
}

declare interface IInstanceWrapper<T extends IRIVM<T>> {
    resource: StaticResource | null | false;
    model: IModelWrapper<T>;
    loadNodes(aliases: Array<string>): Promise<void>;
    allEntries(): MapIterator<[string, Array<IPseudo> | false | null]>;
    addPseudo(childNode: StaticNode, tile: StaticTile | null, node: StaticNode): IPseudo;
    ensureNodegroup(allValues: Map<string, any>, allNodegroups: Map<string, boolean | Promise<any>>, nodegroupId: string | null, nodeObjs: Map<string, StaticNode>, nodegroupObjs: Map<string, StaticNodegroup>, edges: Map<string, string[]>, addIfMissing: boolean, tiles: StaticTile[] | null, doImpliedNodegroups: boolean): Promise<[Map<string, any>, Set<string>]>;
    setOrmAttribute(key: string, value: any): Promise<void>;
    getOrmAttribute(key: string): Promise<any>;
    getValueCache(build: boolean, getMeta: GetMeta): Promise<{
        [tileId: string]: {
            [nodeId: string]: IStringKeyedObject;
        };
    } | undefined>;
    getRoot(): Promise<IPseudo | undefined>;
    getRootViewModel(): Promise<IStringKeyedObject>;
    populate(lazy: boolean): Promise<void>;
}

declare interface IModelWrapper<T extends IRIVM<T>> {
    all(params: {
        limit?: number;
        lazy?: boolean;
    } | undefined): Promise<Array<T>>;
    getPermittedNodegroups(): Map<string | null, boolean | CheckPermission>;
    isNodegroupPermitted(nodegroupId: string, tile: StaticTile | null, nodes: Map<string, StaticNode>): boolean;
    getChildNodes(nodeId: string): Map<string, StaticNode>;
    getNodeObjectsByAlias(): Map<string, StaticNode>;
    getNodeObjects(): Map<string, StaticNode>;
    getNodegroupObjects(): Map<string, StaticNodegroup>;
    getEdges(): Map<string, string[]>;
    wkrm: IWKRM;
}

declare class INodeConfig {
}

declare namespace interfaces {
    export {
        CheckPermission,
        ISemantic,
        ResourceInstanceViewModelConstructor,
        GetMeta,
        IInstanceWrapper,
        IModelWrapper,
        IRIVM,
        IStringKeyedObject,
        IReferenceDataManager,
        IViewModel,
        IPseudo,
        INodeConfig,
        IGraphManager
    }
}
export { interfaces }

declare interface IPseudo {
    parentNode: IPseudo | null;
    getValue(): AttrPromise<IViewModel | null | Array<IViewModel>>;
    forJson(): {
        [key: string]: any;
    } | {
        [key: string]: any;
    }[] | string | number | boolean | null;
    isIterable(): boolean;
    tile: any;
    node: any;
    describeField: () => string;
    describeFieldGroup: () => string;
}

declare interface IReferenceDataManager {
    retrieveCollection(id: string): Promise<StaticCollection>;
}

declare interface IRIVM<T extends IRIVM<T>> {
    [key: string]: any;
    id: string;
    then: undefined;
    $: IInstanceWrapper<T> | null;
    __: IModelWrapper<T> | null;
    __parentPseudo: IPseudo | undefined;
}

declare interface ISemantic extends IViewModel {
    update(mapLike: Map<string, any> | {
        [key: string]: any;
    }): void;
}

declare interface IStaticDescriptorConfig {
    descriptor_types: {
        nodegroup_id: string;
        string_template: string;
    }[];
}

declare interface IStaticNodeConfigBoolean {
    i18n_properties: string[];
    falseLabel: {
        [key: string]: string;
    };
    trueLabel: {
        [key: string]: string;
    };
}

declare interface IStaticNodeConfigDomain {
    i18n_config: {
        [key: string]: string;
    };
    options: StaticDomainValue[];
}

declare interface IStringKeyedObject {
    [key: string | symbol]: any;
}

declare interface IViewModel {
    _: IViewModel | undefined | Promise<IViewModel | null>;
    __parentPseudo: IPseudo | undefined;
    forJson(): {
        [key: string]: any;
    } | {
        [key: string]: any;
    }[] | string | number | boolean | null;
    __forJsonCache(getMeta: GetMeta): IStringKeyedObject | null;
}

declare interface IWKRM {
    modelName: string;
    modelClassName: string;
    graphId: string;
    meta: StaticGraphMeta;
}

declare class JsonRenderer extends Renderer {
    renderDate(value: DateViewModel, _depth: number): Promise<any>;
    renderBoolean(value: boolean | BooleanViewModel, _depth: number): Promise<any>;
    renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any>;
    renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any>;
    renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any>;
}

declare class MarkdownRenderer extends Renderer {
    conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined;
    dateToText: ((value: DateViewModel) => string) | undefined;
    domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined;
    resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined;
    nodeToUrl: ((value: string) => string) | undefined;
    constructor(callbacks: {
        conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined;
        dateToText: ((value: DateViewModel) => string) | undefined;
        domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined;
        resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined;
        nodeToUrl: ((value: string) => string) | undefined;
    });
    renderUrl(value: UrlViewModel, _depth: number): Promise<any>;
    renderDomainValue(domainValue: DomainValueViewModel, _: number): Promise<any>;
    renderDate(date: DateViewModel, _: number): Promise<any>;
    renderConceptValue(conceptValue: ConceptValueViewModel, _: number): Promise<any>;
    renderResourceReference(rivm: ResourceInstanceViewModel<any>, _: number): Promise<any>;
}

declare namespace nodeConfig {
    export {
        nodeConfigManager,
        StaticNodeConfigDomain,
        StaticNodeConfigBoolean
    }
}
export { nodeConfig }

declare class NodeConfigManager {
    static _cache: Map<string, INodeConfig | null>;
    cache: Map<string, INodeConfig | null>;
    constructor(cache?: Map<string, INodeConfig | null> | undefined);
    retrieve(node: StaticNode): INodeConfig | StaticNodeConfigBoolean | StaticNodeConfigDomain | null | undefined;
}

declare const nodeConfigManager: NodeConfigManager;

declare class NonLocalizedStringViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    __forJsonCache(): null;
    forJson(): string;
    static __create(tile: StaticTile, node: StaticNode, value: any): NonLocalizedStringViewModel | Promise<NonLocalizedStringViewModel | null> | null;
    __asTileData(): string;
}

declare class NumberViewModel extends Number implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    toString(): string;
    __forJsonCache(): null;
    forJson(): number;
    static __create(tile: StaticTile, node: StaticNode, value: any): NumberViewModel | Promise<NumberViewModel | null> | null;
    __asTileData(): number;
}

declare class PseudoValue<VM extends IViewModel> implements IPseudo {
    node: StaticNode;
    tile: StaticTile | null;
    value: any;
    parent: IRIVM<any> | null;
    parentNode: PseudoValue<any> | null;
    valueLoaded: boolean | undefined;
    datatype: string | null;
    originalTile: StaticTile | null;
    accessed: boolean;
    childNodes: Map<string, StaticNode>;
    isOuter: boolean;
    isInner: boolean;
    inner: PseudoValue<ISemantic> | null;
    independent: boolean;
    isIterable(): boolean;
    describeField(): string;
    describeFieldGroup(): string;
    constructor(node: StaticNode, tile: StaticTile | null, value: any, parent: IRIVM<any> | null, childNodes: Map<string, StaticNode>, inner: boolean | PseudoValue<ISemantic>);
    getParentTileId(): string | null;
    getTile(): Promise<[StaticTile | null, any[]]>;
    clear(): void;
    updateValue(tile?: StaticTile | null): AttrPromise<VM>;
    getValue(): AttrPromise<VM | null>;
    getLength(): number;
    getChildTypes(): Promise<{}>;
    getChildren(direct?: null): IPseudo[];
    forJson(): Promise<{
        [key: string]: any;
    } | {
        [key: string]: any;
    }[] | string | number | boolean | null>;
}

export declare const RDM: ReferenceDataManager;

declare class ReferenceDataManager {
    archesClient: ArchesClient;
    collections: Map<string, Promise<StaticCollection>>;
    constructor(archesClient: ArchesClient);
    retrieveCollection(id: string): Promise<StaticCollection>;
}

declare class Renderer extends BaseRenderer {
    renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any>;
    renderString(value: string | StringViewModel | NonLocalizedStringViewModel, _depth: number): Promise<any>;
    renderNumber(value: number | NumberViewModel, _depth: number): Promise<any>;
    renderBoolean(value: boolean | BooleanViewModel, _depth: number): Promise<any>;
    renderDate(value: DateViewModel, _depth: number): Promise<any>;
    renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any>;
    renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any>;
    renderSemantic(value: SemanticViewModel, depth: number): Promise<any>;
    renderUrl(value: UrlViewModel, _depth: number): Promise<any>;
    renderBlock(block: {
        [key: string]: string;
    } | {
        [key: string]: string;
    }[], depth: number): any;
    renderArray(value: any, depth: number): Promise<any>;
}

declare namespace renderers {
    export {
        MarkdownRenderer,
        JsonRenderer,
        Cleanable,
        FlatMarkdownRenderer
    }
}
export { renderers }

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
    _: IViewModel | Promise<IViewModel> | undefined;
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

declare type ResourceInstanceViewModelConstructor<T extends IRIVM<T>> = new (id: string, modelWrapper: IModelWrapper<T> | null, instanceWrapperFactory: ((rivm: T) => IInstanceWrapper<T>) | null, cacheEntry: object | null) => T;

export declare class ResourceModelWrapper<RIVM extends IRIVM<RIVM>> {
    wkrm: WKRM;
    graph: StaticGraph;
    viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>;
    permittedNodegroups?: Map<string | null, boolean | CheckPermission>;
    constructor(wkrm: WKRM, graph: StaticGraph, viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>);
    getBranchPublicationIds(accessible?: boolean): string[];
    getCollections(accessible?: boolean): string[];
    pruneGraph(keepFunctions?: string[]): undefined;
    exportGraph(): StaticGraph;
    all(params?: {
        limit?: number;
        lazy?: boolean;
    } | undefined): Promise<Array<RIVM>>;
    stripTiles(resource: StaticResource): void;
    resourceGenerator(staticResources: AsyncIterable<StaticResource, RIVM, unknown>, lazy?: boolean, pruneTiles?: boolean): AsyncGenerator<RIVM, void, unknown>;
    iterAll(params: {
        limit?: number;
        lazy?: boolean;
    }): AsyncGenerator<RIVM>;
    findStatic(id: string): Promise<StaticResource>;
    find(id: string, lazy?: boolean, pruneTiles?: boolean): Promise<RIVM>;
    setPermittedNodegroups(permissions: Map<string | null, boolean | CheckPermission>): void;
    getPermittedNodegroups(): Map<string | null, boolean | CheckPermission>;
    isNodegroupPermitted(nodegroupId: string, tile: StaticTile | null): boolean;
    makeInstance(id: string, resource: StaticResource | null, pruneTiles?: boolean): RIVM;
    edges: Map<string, string[]> | undefined;
    nodes: Map<string, StaticNode> | undefined;
    nodegroups: Map<string, StaticNodegroup> | undefined;
    nodesByAlias: Map<string, StaticNode> | undefined;
    getChildNodes(nodeId: string): Map<string, StaticNode>;
    buildNodes(): void;
    getNodeObjectsByAlias(): Map<string, StaticNode>;
    getEdges(): Map<string, string[]>;
    getNodeObjects(): Map<string, StaticNode>;
    getNodegroupObjects(): Map<string, StaticNodegroup>;
    getRootNode(): StaticNode;
    fromStaticResource(resource: StaticResource, lazy?: boolean, pruneTiles?: boolean): Promise<RIVM>;
}

declare class SemanticViewModel implements IStringKeyedObject, IViewModel {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    __parentPseudo: PseudoValue<any> | undefined;
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

declare class StaticCard {
    active: boolean;
    cardid: string;
    component_id: string;
    config: null | object;
    constraints: Array<StaticConstraint>;
    cssclass: null | string;
    description: string | null | StaticTranslatableString;
    graph_id: string;
    helpenabled: boolean;
    helptext: StaticTranslatableString;
    helptitle: StaticTranslatableString;
    instructions: StaticTranslatableString;
    is_editable: boolean;
    name: StaticTranslatableString;
    nodegroup_id: string;
    sortorder: number | null;
    visible: boolean;
    constructor(jsonData: StaticCard);
}

declare class StaticCardsXNodesXWidgets {
    card_id: string;
    config: object;
    id: string;
    label: StaticTranslatableString;
    node_id: string;
    sortorder: number;
    visible: boolean;
    widget_id: string;
    constructor(jsonData: StaticCardsXNodesXWidgets);
}

declare class StaticCollection {
    id: string;
    prefLabels: {
        [lang: string]: StaticValue;
    };
    concepts: {
        [conceptId: string]: StaticConcept;
    };
    __allConcepts: {
        [conceptId: string]: StaticConcept;
    };
    __values: {
        [valueId: string]: StaticValue;
    };
    constructor(jsonData: StaticCollection);
    getConceptValue(valueId: string): StaticValue;
    getConceptByValue(label: string): StaticConcept | null | undefined;
    toString(): StaticValue;
}

declare class StaticConcept {
    id: string;
    prefLabels: {
        [lang: string]: StaticValue;
    };
    source: string | null;
    sortOrder: number | null;
    children: StaticConcept[] | null;
    constructor(jsonData: StaticConcept);
    getPrefLabel(): StaticValue;
    toString(): string;
}

declare class StaticConstraint {
    card_id: string;
    constraintid: string;
    nodes: Array<string>;
    uniquetoallinstances: boolean;
    constructor(jsonData: StaticConstraint);
}

declare class StaticDomainValue {
    id: string;
    selected: boolean;
    text: {
        [lang: string]: string;
    };
    constructor(jsonData: StaticDomainValue);
    toString(): string;
    lang(lang: string): string | undefined;
    forJson(): Promise<{
        id: string;
        selected: boolean;
        text: {
            [lang: string]: string;
        };
    }>;
}

declare class StaticEdge {
    description: string | null;
    domainnode_id: string;
    edgeid: string;
    graph_id: string;
    name: null | string;
    rangenode_id: string;
    ontologyproperty: null | string;
    constructor(jsonData: StaticEdge);
    copy?(): StaticEdge;
}

declare class StaticFunctionsXGraphs {
    config: IStaticDescriptorConfig;
    function_id: string;
    graph_id: string;
    id: string;
    constructor(jsonData: StaticFunctionsXGraphs);
    copy(): StaticFunctionsXGraphs;
}

declare class StaticGraph {
    author: string;
    cards: Array<StaticCard> | null;
    cards_x_nodes_x_widgets: Array<StaticCardsXNodesXWidgets> | null;
    color: string | null;
    config: object;
    deploymentdate: null | string;
    deploymentfile: null | string;
    description: StaticTranslatableString;
    edges: Array<StaticEdge>;
    functions_x_graphs: Array<StaticFunctionsXGraphs> | null;
    graphid: string;
    iconclass: string;
    is_editable: boolean | null;
    isresource: boolean;
    jsonldcontext: string | null;
    name: StaticTranslatableString;
    nodegroups: Array<StaticNodegroup>;
    nodes: Array<StaticNode>;
    ontology_id: string | null;
    publication: StaticPublication | null;
    relatable_resource_model_ids: Array<string>;
    resource_2_resource_constraints: Array<any> | null;
    root: StaticNode;
    slug: string | null;
    subtitle: StaticTranslatableString;
    template_id: string;
    version: string;
    constructor(jsonData: StaticGraph);
    copy?(): StaticGraph;
}

declare class StaticGraphMeta {
    [key: string]: any;
    author: string | undefined;
    cards: number | undefined;
    cards_x_nodes_x_widgets: number | undefined;
    color: string | undefined;
    description: {
        [lang: string]: string;
    } | undefined;
    edges: number | undefined;
    graphid: string;
    iconclass: string | undefined;
    is_editable: boolean | undefined;
    isresource: boolean | undefined;
    jsonldcontext: {
        [key: string]: any;
    } | undefined;
    name: {
        [lang: string]: string;
    } | undefined;
    nodegroups: number | undefined;
    nodes: number | undefined;
    ontology_id: string | undefined;
    publication: {
        [key: string]: string | null;
    } | undefined;
    relatable_resource_model_ids: string[];
    resource_2_resource_constraints: any[];
    root: StaticNode | undefined;
    slug: string | undefined;
    subtitle: {
        [lang: string]: string;
    } | undefined;
    version: string | undefined;
    constructor(jsondata: StaticGraphMeta);
}

declare class StaticNode {
    alias: string | null;
    config: {
        [key: string]: any;
    } | null;
    datatype: string;
    description: string | null;
    exportable: boolean;
    fieldname: null | string;
    graph_id: string;
    hascustomalias: boolean;
    is_collector: boolean;
    isrequired: boolean;
    issearchable: boolean;
    istopnode: boolean;
    name: string;
    nodegroup_id: string | null;
    nodeid: string;
    parentproperty: string | null;
    sortorder: number;
    ontologyclass: string | null;
    sourcebranchpublication_id: null | string;
    constructor(jsonData: StaticNode);
    copy?(): StaticNode;
}

declare class StaticNodeConfigBoolean implements IStaticNodeConfigBoolean, INodeConfig {
    i18n_properties: string[];
    falseLabel: {
        [key: string]: string;
    };
    trueLabel: {
        [key: string]: string;
    };
    constructor(jsonData: IStaticNodeConfigBoolean);
}

declare class StaticNodeConfigDomain implements IStaticNodeConfigDomain, INodeConfig {
    i18n_config: {
        [key: string]: string;
    };
    options: StaticDomainValue[];
    getSelected(): StaticDomainValue | undefined;
    valueFromId(id: string): StaticDomainValue | undefined;
    constructor(jsonData: IStaticNodeConfigDomain);
}

declare class StaticNodegroup {
    legacygroupid: null;
    nodegroupid: string;
    parentnodegroup_id: string | null;
    cardinality: "1" | "n" | null;
    constructor(jsonData: StaticNodegroup);
    copy(): StaticNodegroup;
}

declare type StaticProvisionalEdit = any;

declare class StaticPublication {
    graph_id: string;
    notes: null | string;
    publicationid: string;
    published_time: string;
    constructor(jsonData: StaticPublication);
    copy(): StaticPublication;
}

declare class StaticResource {
    resourceinstance: StaticResourceMetadata;
    tiles: Array<StaticTile> | null;
    metadata: {
        [key: string]: string;
    };
    __cache: {
        [tileId: string]: {
            [nodeId: string]: {
                [key: string]: string;
            };
        };
    } | undefined;
    __source: string | undefined;
    __scopes: string[] | undefined;
    constructor(jsonData: StaticResource);
}

declare class StaticResourceDescriptors {
    [key: string]: (string | undefined | (() => boolean));
    name?: string;
    map_popup?: string;
    description?: string;
    constructor(jsonData?: StaticResourceDescriptors);
    isEmpty(): boolean;
}

declare class StaticResourceMetadata {
    descriptors: StaticResourceDescriptors;
    graph_id: string;
    name: string;
    resourceinstanceid: string;
    publication_id: string | null;
    principaluser_id: number | null;
    legacyid: null | string;
    graph_publication_id: string | null;
    constructor(jsonData: StaticResourceMetadata);
}

declare class StaticResourceReference {
    id: string;
    type: string | undefined;
    graphId: string;
    title: string | undefined;
    root: any | undefined;
    meta?: {
        [key: string]: any;
    };
    constructor(jsonData: StaticResourceReference);
}

declare class StaticStore {
    archesClient: ArchesClient;
    cache: Map<string, StaticResource | StaticResourceMetadata>;
    cacheMetadataOnly: boolean;
    constructor(archesClient: ArchesClient, cacheMetadataOnly?: boolean);
    getMeta(id: string, onlyIfCached?: boolean): Promise<StaticResourceMetadata | null>;
    loadAll(graphId: string, limit?: number | undefined): AsyncIterable<StaticResource>;
    loadOne(id: string): Promise<StaticResource>;
}

export declare const staticStore: StaticStore;

declare class StaticTile {
    data: Map<string, object | Map<string, any> | Array<any> | null | number | boolean | string>;
    nodegroup_id: string;
    resourceinstance_id: string;
    tileid: string | null;
    parenttile_id: string | null;
    provisionaledits: null | Array<StaticProvisionalEdit>;
    sortorder: number | null;
    constructor(jsonData: StaticTile);
    ensureId(): string;
}

declare class StaticTranslatableString extends String {
    translations: Map<string, string>;
    lang: string;
    constructor(s: string | StaticTranslatableString, lang?: undefined | string);
    copy(): StaticTranslatableString;
}

declare namespace staticTypes {
    export {
        StaticValue,
        StaticTile,
        StaticGraph,
        StaticResource,
        StaticResourceMetadata,
        StaticNode,
        StaticNodegroup,
        StaticEdge,
        StaticCard,
        StaticCardsXNodesXWidgets,
        StaticCollection,
        StaticConcept,
        StaticDomainValue,
        StaticResourceReference,
        StaticGraphMeta,
        StaticFunctionsXGraphs,
        StaticResourceDescriptors,
        IStaticDescriptorConfig
    }
}
export { staticTypes }

declare class StaticValue {
    id: string;
    value: string;
    __concept: StaticConcept | null;
    __conceptId: string | null;
    constructor(jsonData: StaticValue, concept?: StaticConcept | string | null);
    toString(): string;
}

declare class StringTranslatedLanguage {
    value: string;
}

declare class StringViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    _value: Map<string, StringTranslatedLanguage>;
    __forJsonCache(): null;
    constructor(value: Map<string, StringTranslatedLanguage>, language?: string | null);
    forJson(): string;
    lang(language: string): string | undefined;
    static __create(tile: StaticTile, node: StaticNode, value: any): StringViewModel | Promise<StringViewModel | null> | null;
    __asTileData(): Map<string, StringTranslatedLanguage>;
}

declare class Url {
    url: string;
    url_label?: string;
    constructor(url: string, url_label?: string);
}

declare class UrlViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
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

declare namespace utils {
    export {
        AttrPromise,
        getCurrentLanguage
    }
}
export { utils }

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

declare class ViewContext {
    graphManager: IGraphManager | undefined;
}

declare const viewContext: ViewContext;

declare namespace viewModels {
    export {
        ResourceInstanceCacheEntry,
        DEFAULT_LANGUAGE,
        ResourceInstanceViewModel,
        ValueList,
        getViewModel,
        DomainValueViewModel,
        SemanticViewModel,
        StringViewModel,
        DateViewModel,
        GeoJSONViewModel,
        ConceptValueViewModel,
        viewContext,
        NonLocalizedStringViewModel,
        CUSTOM_DATATYPES,
        BooleanViewModel,
        NumberViewModel,
        UrlViewModel
    }
}
export { viewModels }

export declare class WKRM {
    modelName: string;
    modelClassName: string;
    graphId: string;
    meta: StaticGraphMeta;
    constructor(meta: StaticGraphMeta);
}

export { }
