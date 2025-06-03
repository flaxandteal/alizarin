import { ArchesClient, ArchesClientRemote } from './client.ts';
import { staticStore } from './staticStore.ts';
import { StaticTile, StaticGraph, StaticNode, StaticNodegroup, StaticResource, StaticGraphMeta } from './static-types';
import { CheckPermission, IRIVM, ResourceInstanceViewModelConstructor } from './interfaces';
declare class WKRM {
    modelName: string;
    modelClassName: string;
    graphId: string;
    meta: StaticGraphMeta;
    constructor(meta: StaticGraphMeta);
}
declare class ConfigurationOptions {
    graphs: Array<string> | null | boolean;
    eagerLoadGraphs: boolean;
    constructor();
}
declare class ResourceModelWrapper<RIVM extends IRIVM<RIVM>> {
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
declare class GraphManager {
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
declare const graphManager: GraphManager;
export { GraphManager, graphManager, ArchesClientRemote, staticStore, WKRM, ResourceModelWrapper };
