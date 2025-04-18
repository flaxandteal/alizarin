declare abstract class ArchesClient {
    abstract getGraphs(): Promise<GraphResult>;
    abstract getGraph(graphId: string): Promise<StaticGraph | null>;
    abstract getResources(graphId: string, limit: number): Promise<StaticResource[]>;
    abstract getResource(resourceId: string): Promise<StaticResource>;
    abstract getCollection(collectionId: string): Promise<StaticCollection>;
}

declare const archesClient: ArchesClientRemote;

declare class ArchesClientLocal extends ArchesClient {
    fs: any;
    allGraphFile: Function;
    graphIdToGraphFile: Function;
    graphIdToResourcesFiles: Function;
    resourceIdToFile: Function;
    collectionIdToFile: Function;
    constructor({ allGraphFile, graphIdToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, }?: {
        [k: string]: Function;
    });
    getGraphs(): Promise<GraphResult>;
    getGraph(graphId: string): Promise<StaticGraph | null>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number | null): Promise<StaticResource[]>;
}

declare class ArchesClientRemote extends ArchesClient {
    archesUrl: string;
    constructor(archesUrl: string);
    getGraphs(): Promise<GraphResult>;
    getGraph(graphId: string | null): Promise<StaticGraph>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number): Promise<StaticResource[]>;
}

declare class ArchesClientRemoteStatic extends ArchesClient {
    archesUrl: string;
    allGraphFile: Function;
    graphIdToGraphFile: Function;
    graphIdToResourcesFiles: Function;
    resourceIdToFile: Function;
    collectionIdToFile: Function;
    constructor(archesUrl: string, { allGraphFile, graphIdToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, }?: {
        [k: string]: Function;
    });
    getGraphs(): Promise<GraphResult>;
    getGraph(graphId: string): Promise<StaticGraph | null>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number): Promise<StaticResource[]>;
}

declare class AttrPromise<T> extends Promise<T> implements IStringKeyedObject {
    constructor(executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason: any) => void) => void);
}

declare class Cleanable extends String {
    __clean: String | undefined;
}

declare namespace client {
    export {
        archesClient,
        ArchesClient,
        ArchesClientRemoteStatic,
        ArchesClientRemote,
        ArchesClientLocal
    }
}
export { client }

declare class ConceptValueViewModel extends String implements IViewModel {
    __parentPseudo: PseudoValue | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    _value: StaticValue | Promise<StaticValue>;
    constructor(value: StaticValue);
    forJson(): Promise<StaticValue>;
    getValue(): StaticValue | Promise<StaticValue>;
    static __create(tile: StaticTile, node: StaticNode, value: any): Promise<ConceptValueViewModel | null>;
    __asTileData(): any;
}

declare class ConfigurationOptions {
    graphs: Array<string> | null;
    constructor();
}

declare class DomainValueViewModel extends String implements IViewModel {
    __parentPseudo: PseudoValue | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    _value: StaticDomainValue | Promise<StaticDomainValue>;
    constructor(value: StaticDomainValue);
    forJson(): Promise<StaticDomainValue>;
    getValue(): StaticValue | Promise<StaticValue>;
    lang(lang: string): string | undefined;
    static __create(tile: StaticTile, node: StaticNode, value: any): Promise<DomainValueViewModel | null>;
    __asTileData(): any;
}

declare class GeoJSONViewModel implements IViewModel, IStringKeyedObject {
    [key: string]: any;
    __parentPseudo: PseudoValue | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    _value: {
        [key: string]: any;
    };
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

declare function getViewModel(parentPseudo: PseudoValue, tile: StaticTile, node: StaticNode, data: any, parent: ResourceInstanceViewModel | null, childNodes: Map<string, StaticNode>): Promise<[IViewModel, Function | null, string, boolean]>;

declare class GraphManager {
    _initialized: boolean;
    archesClient: ArchesClient;
    graphs: Map<string, ResourceModelWrapper<any>>;
    wkrms: Map<string, WKRM>;
    constructor(archesClient: ArchesClient);
    initialize(configurationOptions: ConfigurationOptions | undefined): Promise<void>;
    get(modelClassName: string): typeof ResourceInstanceViewModel;
    getGraph(graphId: string): StaticGraph;
}

export declare const graphManager: GraphManager;

declare class GraphResult {
    models: string[];
    constructor(jsonData: GraphResult);
}

declare interface IInstanceWrapper {
    resource: any;
    model: any;
    ensureNodegroup: any;
    setOrmAttribute: any;
    getOrmAttribute: any;
}

declare interface IModelWrapper {
    getPermittedNodegroups(): Map<string, StaticNodegroup>;
    getChildNodes(nodeId: string): Map<string, StaticNode>;
    getNodeObjectsByAlias(): Map<string, StaticNode>;
    getNodegroupObjects(): Map<string, StaticNodegroup>;
    wkrm: any;
}

declare interface IPseudo {
    parentNode: IPseudo | null;
}

declare interface IRIVM {
    _: IInstanceWrapper;
    __: IModelWrapper;
}

declare interface IStringKeyedObject {
    [key: string]: any;
}

declare interface IViewModel {
    __parentPseudo: IPseudo | undefined;
    forJson(): {
        [key: string]: any;
    } | {
        [key: string]: any;
    }[];
}

declare class JsonRenderer extends Renderer {
    renderConceptValue(value: ConceptValueViewModel): Promise<any>;
    renderDomainValue(value: DomainValueViewModel): Promise<any>;
    renderResourceReference(value: ResourceInstanceViewModel): Promise<any>;
}

declare class MarkdownRenderer extends Renderer {
    conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined;
    domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined;
    resourceReferenceToUrl: ((value: ResourceInstanceViewModel) => string) | undefined;
    constructor(callbacks: {
        conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined;
        domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined;
        resourceReferenceToUrl: ((value: ResourceInstanceViewModel) => string) | undefined;
    });
    renderDomainValue(domainValue: DomainValueViewModel): Promise<any>;
    renderConceptValue(conceptValue: ConceptValueViewModel): Promise<any>;
    renderResourceReference(rivm: ResourceInstanceViewModel): Promise<any>;
}

declare class PseudoValue implements IPseudo {
    node: StaticNode;
    tile: StaticTile | null;
    value: any;
    parent: IRIVM | null;
    parentNode: PseudoValue | null;
    valueLoaded: boolean | undefined;
    datatype: string | null;
    originalTile: StaticTile | null;
    accessed: boolean;
    childNodes: Map<string, StaticNode>;
    multiple: boolean;
    asTileData: Function | null;
    describeField(): string;
    describeFieldGroup(): string;
    constructor(node: StaticNode, tile: StaticTile | null, value: any, parent: IRIVM | null, childNodes: Map<string, StaticNode>);
    getParentTileId(): string | null;
    getTile(): Promise<(any[] | StaticTile | null)[]>;
    clear(): void;
    updateValue(): AttrPromise<IViewModel>;
    getValue(): AttrPromise<IViewModel | null>;
    getLength(): any;
    getType(): Promise<(string | boolean | null)[]>;
    getChildTypes(): Promise<any>;
    getChildren(direct?: null): any;
}

export declare const RDM: ReferenceDataManager;

declare class ReferenceDataManager {
    archesClient: ArchesClient;
    collections: Map<string, Promise<StaticCollection>>;
    constructor(archesClient: ArchesClient);
    retrieveCollection(id: string): Promise<StaticCollection>;
}

declare class Renderer {
    render(asset: ResourceInstanceViewModel): Promise<any>;
    renderDomainValue(value: DomainValueViewModel): Promise<any>;
    renderConceptValue(value: ConceptValueViewModel): Promise<any>;
    renderResourceReference(value: ResourceInstanceViewModel): Promise<any>;
    renderBlock(block: {
        [key: string]: string;
    } | {
        [key: string]: string;
    }[]): Promise<{
        [key: string]: any;
    }>;
    renderValue(value: any): Promise<any>;
}

declare namespace renderers {
    export {
        MarkdownRenderer,
        JsonRenderer,
        Cleanable
    }
}
export { renderers }

declare class ResourceInstanceViewModel implements IStringKeyedObject {
    [key: string]: any;
    _: IInstanceWrapper;
    __: IModelWrapper;
    id: string;
    then: null;
    toString(): string;
    forJson(cascade?: boolean): Promise<StaticResourceReference>;
    constructor(id: string, modelWrapper: IModelWrapper, instanceWrapperFactory: (rivm: ResourceInstanceViewModel) => IInstanceWrapper);
}

declare class ResourceModelWrapper<RIVM extends ResourceInstanceViewModel> {
    wkrm: WKRM;
    graph: StaticGraph;
    viewModelClass: typeof ResourceInstanceViewModel;
    makePseudoCls: Function;
    constructor(wkrm: WKRM, graph: StaticGraph);
    all(params: {
        limit: number;
        lazy: boolean;
    }): Promise<Array<RIVM>>;
    resourceGenerator(staticResources: AsyncIterable<StaticResource, RIVM, unknown>, lazy?: boolean): AsyncGenerator<RIVM, void, unknown>;
    iterAll(params: {
        limit: number;
        lazy: boolean;
    }): AsyncGenerator<RIVM>;
    find(id: string, lazy?: boolean): Promise<RIVM>;
    getPermittedNodegroups(): Map<string, StaticNodegroup>;
    makeInstance(id: string, resource: StaticResource | null): IRIVM;
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
    fromStaticResource(resource: StaticResource, lazy?: boolean): Promise<RIVM>;
}

declare class SemanticViewModel implements IStringKeyedObject, IViewModel {
    [key: string]: any;
    then: undefined;
    __parentPseudo: PseudoValue | undefined;
    __childValues: Map<string, any>;
    __parentWkri: ResourceInstanceViewModel | null;
    __childNodes: Map<string, StaticNode>;
    __tile: StaticTile | null;
    __node: StaticNode;
    constructor(parentWkri: ResourceInstanceViewModel | null, childNodes: Map<string, StaticNode>, tile: StaticTile | null, node: StaticNode);
    toString(): Promise<string>;
    toObject(): Promise<any>;
    forJson(): Promise<any>;
    __update(map: Map<string, any>): Promise<void[]>;
    __get(key: string): Promise<any>;
    __set(key: string, value: any): Promise<void>;
    __getChildTypes(): Promise<Map<string, any>>;
    __getChildren(direct?: null | boolean): Promise<any[]>;
    __getChildValue(key: string): Promise<any>;
    __makePseudo(key: string): any;
    static __create(tile: StaticTile, node: StaticNode, value: any, parent: ResourceInstanceViewModel | null, childNodes: Map<string, StaticNode>): Promise<SemanticViewModel>;
    __asTileData(): Promise<(any[] | null)[]>;
    __getChildValues(targetKey?: string | null): Promise<any>;
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
        [lang: string]: object;
    };
    constructor(jsonData: StaticDomainValue);
    toString(): object;
    lang(lang: string): object;
    forJson(): Promise<{
        id: string;
        selected: boolean;
        text: {
            [lang: string]: object;
        };
    }>;
}

declare class StaticEdge {
    description: null;
    domainnode_id: string;
    edgeid: string;
    graph_id: string;
    name: null | string;
    rangenode_id: string;
    ontologyproperty: null | string;
    constructor(jsonData: StaticEdge);
}

declare class StaticFunctionsXGraphs {
    config: object;
    function_id: string;
    graph_id: string;
    id: string;
    constructor(jsonData: StaticFunctionsXGraphs);
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
    resource_2_resource_constringaints: Array<any> | null;
    slug: string | null;
    subtitle: StaticTranslatableString;
    template_id: string;
    version: string;
    constructor(jsonData: StaticGraph);
}

declare class StaticNode {
    alias: string | null;
    config: {
        [key: string]: any;
    };
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
}

declare class StaticNodegroup {
    legacygroupid: null;
    nodegroupid: string;
    parentnodegroup_id: string | null;
    cardinality: "1" | "n" | null;
    constructor(jsonData: StaticNodegroup);
}

declare type StaticProvisionalEdit = any;

declare class StaticPublication {
    graph_id: string;
    notes: null | string;
    publicationid: string;
    published_time: string;
    constructor(jsonData: StaticPublication);
}

declare class StaticResource {
    resourceinstance: StaticResourceMetadata;
    tiles: Array<StaticTile> | null;
    __source: string | undefined;
    constructor(jsonData: StaticResource);
}

declare class StaticResourceMetadata {
    descriptors: Map<string, any>;
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
    root: any | undefined;
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
}

declare class StaticTranslatableString extends String {
    translations: Map<string, string>;
    lang: string;
    constructor(s: string | StaticTranslatableString, lang?: undefined | string);
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
        StaticCollection,
        StaticConcept,
        StaticDomainValue,
        StaticResourceReference
    }
}
export { staticTypes }

declare class StaticValue {
    id: string;
    value: string;
    __concept: StaticConcept | null;
    constructor(jsonData: StaticValue, concept?: StaticConcept | null);
    toString(): string;
}

declare class StringViewModel extends String implements IViewModel {
    __parentPseudo: PseudoValue | undefined;
    describeField: () => string | null;
    describeFieldGroup: () => string | null;
    _value: Map<string, object>;
    constructor(value: Map<string, object>, language?: string | null);
    forJson(): string;
    lang(language: string): any;
    static __create(tile: StaticTile, node: StaticNode, value: any): StringViewModel | Promise<StringViewModel> | null;
    __asTileData(): Map<string, object>;
}

declare namespace utils {
    export {
        AttrPromise,
        getCurrentLanguage
    }
}
export { utils }

declare class ValueList {
    values: Map<string, any>;
    wrapper: IInstanceWrapper;
    tiles: StaticTile[] | null;
    constructor(values: Map<string, any>, wrapper: IInstanceWrapper, tiles: StaticTile[] | null);
    get(key: string): Promise<any>;
    set(key: string, value: any): void;
    has(key: string): Promise<boolean>;
    retrieve(key: string, dflt?: any, raiseError?: boolean): Promise<any>;
    setDefault(key: string, value: any): Promise<any>;
}

declare namespace viewModels {
    export {
        ResourceInstanceViewModel,
        ValueList,
        getViewModel,
        DomainValueViewModel,
        SemanticViewModel,
        StringViewModel,
        GeoJSONViewModel,
        ConceptValueViewModel
    }
}
export { viewModels }

declare class WKRM {
    modelName: string;
    modelClassName: string;
    graphId: string;
    constructor(modelName: string, graphId: string);
}

export { }
