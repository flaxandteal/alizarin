import { ArchesClient, ArchesClientRemote } from './client';
import { staticStore } from './staticStore';
import { CardComponent, Widget } from './cards';
import { StaticTranslatableString, StaticCollection, StaticConstraint, StaticEdge, StaticTile, StaticGraph, StaticNode, StaticNodegroup, StaticResource, StaticResourceSummary } from "./static-types";
import { PseudoValue, PseudoUnavailable } from "./pseudos.ts";
import { WKRM, WASMResourceModelWrapper, WASMResourceInstanceWrapper } from "../pkg/alizarin";
import { SemanticViewModel, NodeViewModel } from "./viewModels.ts";
import { GetMeta, IRIVM, IStringKeyedObject, IPseudo, IInstanceWrapper, IViewModel, ResourceInstanceViewModelConstructor, PermissionValue } from "./interfaces";
import { AttrPromise } from "./utils";
import { recordWasmTiming, printWasmTimings, clearWasmTimings, getWasmTimings } from './wasmTiming';
export { recordWasmTiming, printWasmTimings, clearWasmTimings, getWasmTimings };
export type { ConditionalPermission, PermissionValue } from "./interfaces";
declare class ConfigurationOptions {
    graphs: Array<string> | null | boolean;
    eagerLoadGraphs: boolean;
    defaultAllowAllNodegroups: boolean;
}
export declare class ResourceInstanceWrapper<RIVM extends IRIVM<RIVM>> implements IInstanceWrapper<RIVM> {
    wkri: RIVM;
    model: ResourceModelWrapper<RIVM>;
    wasmWrapper: WASMResourceInstanceWrapper;
    resource?: StaticResource;
    private _pseudoCache;
    cache: {
        [tileId: string]: {
            [nodeId: string]: IStringKeyedObject;
        };
    } | undefined;
    scopes?: string[];
    metadata?: {
        [key: string]: string;
    };
    private tilesLoaded;
    private pruneTiles;
    constructor(wkri: RIVM, model: ResourceModelWrapper<RIVM>, resource: StaticResource | null | false, // False to disable dynamic resource-loading
    pruneTiles?: boolean, lazy?: boolean, assumeTilesComprehensiveForNodegroup?: boolean);
    ensureTilesLoaded(): Promise<void>;
    pruneResourceTiles(): undefined;
    /**
     * Retrieve pseudo value by alias - queries Rust's pseudo_cache
     * Replaces ValueList.retrieve
     */
    retrievePseudo(key: string, dflt?: any, raiseError?: boolean): Promise<Array<IPseudo> | null>;
    hasPseudo(key: string): Promise<boolean>;
    setPseudo(key: string, value: any): void;
    setDefaultPseudo(key: string, value: any): Promise<any>;
    loadNodes(aliases: Array<string>): Promise<void>;
    getName(update?: boolean): string;
    getDescriptors(update?: boolean): import("./static-types").StaticResourceDescriptors;
    getValuesAtPath(path: string, filterTileId?: string): PseudoValue<any> | PseudoUnavailable | import("./pseudos.ts").PseudoList;
    addPseudo(childNode: StaticNode, tile: StaticTile | null): IPseudo;
    allEntries(): MapIterator<[string, Array<IPseudo> | false | null]>;
    keys(): Promise<any>;
    values(): Promise<any>;
    entries(): Promise<any>;
    getRootViewModel(): Promise<SemanticViewModel>;
    getOrmAttribute(key: string): AttrPromise<IViewModel>;
    getRoot(): Promise<IPseudo | undefined>;
    setOrmAttribute(key: string, value: any): Promise<void>;
    /**
     * Populate all nodegroups for a resource
     * Uses Rust implementation via WASM - Rust caches all values internally
     */
    populate(lazy: boolean): Promise<void>;
    getValueCache(build?: boolean, getMeta?: GetMeta): Promise<{
        [tileId: string]: {
            [nodeId: string]: IStringKeyedObject;
        };
    } | undefined>;
    buildValueCache(getMeta: GetMeta): Promise<{
        [tileId: string]: {
            [nodeId: string]: IStringKeyedObject;
        };
    }>;
    /**
     * Release WASM memory held by this wrapper.
     * Call this after extracting data from a resource to free memory when processing many resources.
     * This clears tiles, pseudo cache, and other internal state.
     */
    release(): void;
}
type GraphMutation = (baseGraph: StaticGraph) => StaticGraph;
declare class GraphMutator {
    baseGraph: StaticGraph;
    mutations: GraphMutation[];
    autocreateCard: boolean;
    constructor(baseGraph: StaticGraph, options?: {
        autocreateCard?: boolean;
    });
    _generateUuidv5(key: string): string;
    _generateEdge(fromNode: string, toNode: string, ontologyProperty: string, name?: string, description?: string): StaticEdge;
    addSemanticNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options?: {
        exportable?: boolean;
        fieldname?: string;
        hascustomalias?: boolean;
        is_collector?: boolean;
        isrequired?: boolean;
        issearchable?: boolean;
        istopnode?: boolean;
        sortorder?: number;
    }, config?: {
        [key: string]: any;
    }): this;
    addConceptNode(parentAlias: string | null, alias: string, name: string, collection: StaticCollection, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options?: {
        is_list?: boolean;
        exportable?: boolean;
        fieldname?: string;
        hascustomalias?: boolean;
        is_collector?: boolean;
        isrequired?: boolean;
        issearchable?: boolean;
        istopnode?: boolean;
        sortorder?: number;
    }, config?: {
        [key: string]: any;
    }): this;
    addCard(nodegroup: string | StaticNodegroup, name: string | StaticTranslatableString, component?: CardComponent, options?: {
        active?: boolean;
        constraints?: Array<StaticConstraint>;
        cssclass?: string | null;
        helpenabled?: boolean;
        helptext?: string | null | StaticTranslatableString;
        helptitle?: string | null | StaticTranslatableString;
        instructions?: string | null | StaticTranslatableString;
        is_editable?: boolean;
        description?: string | null;
        sortorder?: number | null;
        visible?: boolean;
    }, config?: {
        [key: string]: any;
    }): void;
    addStringNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options?: {
        exportable?: boolean;
        fieldname?: string;
        hascustomalias?: boolean;
        is_collector?: boolean;
        isrequired?: boolean;
        issearchable?: boolean;
        istopnode?: boolean;
        sortorder?: number;
    }, config?: {
        [key: string]: any;
    }): this;
    _addNodegroup(parentAlias: string | null, nodegroupId: string, cardinality: 'n' | '1', name?: StaticTranslatableString): this;
    _addGenericNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', datatype: string, ontologyClass: string | string[], parentProperty: string, description?: string, options?: {
        exportable?: boolean;
        fieldname?: string;
        hascustomalias?: boolean;
        is_collector?: boolean;
        isrequired?: boolean;
        issearchable?: boolean;
        istopnode?: boolean;
        sortorder?: number;
    }, config?: {
        [key: string]: any;
    }): this;
    addWidgetToCard(nodeId: string, widget: Widget, name: string, config: {
        [key: string]: any;
    }, options?: {
        sortorder?: number | null;
        silentSkip?: boolean;
        visible?: boolean;
    }): GraphMutator;
    apply(): StaticGraph;
}
declare class ResourceModelWrapper<RIVM extends IRIVM<RIVM>> extends WASMResourceModelWrapper {
    viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>;
    permittedNodegroups?: Map<string, PermissionValue>;
    pruneTiles: boolean;
    constructor(wkrm: WKRM, graph: StaticGraph, viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>, defaultAllow: boolean);
    getRoot(): NodeViewModel;
    buildNodes(): void;
    getNodeObjects(): Map<string, StaticNode>;
    getNodeObjectsByAlias(): Map<string, StaticNode>;
    getEdges(): Map<string, string[]>;
    getNodegroupObjects(): Map<string, StaticNodegroup>;
    getBranchPublicationIds(accessible?: boolean): string[];
    getCollections(accessible?: boolean): string[];
    pruneGraph(keepFunctions?: string[]): undefined;
    getPruneTiles(pruneTiles?: boolean): boolean;
    all(params?: {
        limit?: number;
        lazy?: boolean;
        pruneTiles?: boolean;
    } | undefined): Promise<Array<RIVM>>;
    resourceGenerator(staticResources: AsyncIterable<StaticResource, RIVM, unknown>, lazy?: boolean, pruneTiles?: boolean): AsyncGenerator<RIVM, void, unknown>;
    iterAll(params: {
        limit?: number;
        lazy?: boolean;
        pruneTiles?: boolean;
    }): AsyncGenerator<RIVM>;
    summaryGenerator(staticSummaries: AsyncIterable<StaticResourceSummary>, lazy?: boolean): AsyncGenerator<RIVM>;
    iterAllSummaries(params: {
        limit?: number;
    }): AsyncGenerator<RIVM>;
    allSummaries(params?: {
        limit?: number;
    } | undefined): Promise<Array<RIVM>>;
    loadFullResource(id: string): Promise<RIVM>;
    findStatic(id: string): Promise<StaticResource>;
    find(id: string, lazy?: boolean, pruneTiles?: boolean): Promise<RIVM>;
    setPermittedNodegroups(permissions: Map<string, PermissionValue>): void;
    getPermittedNodegroups(): Map<string, PermissionValue>;
    isNodegroupPermitted(nodegroupId: string, _tile: StaticTile | null): boolean;
    makeInstance(id: string, resource: StaticResource | null, pruneTiles?: boolean, lazy?: boolean): RIVM;
    fromStaticResource(resource: StaticResource, lazy?: boolean, pruneTiles?: boolean): Promise<RIVM>;
    asTree(): {
        [key: string]: any;
    };
}
declare class GraphManager {
    _initialized: boolean;
    archesClient: ArchesClient;
    graphs: Map<string, ResourceModelWrapper<any>>;
    wkrms: Map<string, WKRM>;
    defaultAllow: boolean;
    constructor(archesClient: ArchesClient);
    getPruneTiles(pruneTiles?: boolean): boolean;
    initialize(configurationOptions?: ConfigurationOptions | undefined): Promise<void>;
    loadGraph<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string, defaultAllow?: boolean): Promise<ResourceModelWrapper<RIVM>>;
    get<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string, defaultAllow?: boolean): Promise<ResourceModelWrapper<RIVM>>;
    getResource<T extends IRIVM<T>>(resourceId: string, lazy?: boolean, pruneTiles?: boolean): Promise<T>;
    getGraph(graphId: string): StaticGraph;
}
declare const graphManager: GraphManager;
export { GraphManager, graphManager, ArchesClientRemote, staticStore, WKRM, WASMResourceModelWrapper, ResourceModelWrapper, GraphMutator };
