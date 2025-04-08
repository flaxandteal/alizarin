declare abstract class ArchesClient {
    abstract getGraphs(): Promise<GraphResult>;
    abstract getGraph(graphId: string): Promise<StaticGraph>;
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
    getGraph(graphId: string): Promise<StaticGraph>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number | null): Promise<StaticResource[]>;
}

declare class ArchesClientRemote extends ArchesClient {
    archesUrl: string;
    constructor(archesUrl: string);
    getGraphs(): Promise<GraphResult>;
    getGraph(graphId: string): Promise<StaticGraph>;
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
    getGraph(graphId: string): Promise<StaticGraph>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number): Promise<StaticResource[]>;
}

declare class AttrPromise<T> extends Promise<T> implements IStringKeyedObject {
    constructor(executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason: any) => void) => void);
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

declare class ConfigurationOptions {
    graphs: Array<string> | null;
    constructor();
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

declare class ResourceInstanceViewModel implements IStringKeyedObject {
    [key: string]: any;
    _: IInstanceWrapper;
    __: IModelWrapper;
    id: string;
    then: null;
    toString(): string;
    forJson(cascade?: boolean): Promise<{
        type: any;
        graphId: any;
        id: string;
    }>;
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

declare class StaticStore {
    archesClient: ArchesClient;
    constructor(archesClient: ArchesClient);
    loadAll(graphId: string, limit?: number | undefined): Promise<Array<StaticResource>>;
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
        StaticNode,
        StaticNodegroup,
        StaticEdge,
        StaticCollection,
        StaticConcept,
        StaticDomainValue
    }
}
export { staticTypes }

declare class StaticValue {
    id: string;
    value: string;
    __concept: StaticConcept | null;
    constructor(jsonData: StaticValue, concept?: StaticConcept | null);
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
        getViewModel
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
