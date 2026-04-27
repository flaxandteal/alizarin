import { StaticGraphMeta, StaticGraph, StaticResource, StaticResourceSummary, StaticResourceRegistry, StaticTile } from "./static-types";
import { StaticCollection } from "./rdm";
declare class GraphResult {
    models: {
        [graphId: string]: StaticGraphMeta;
    };
    constructor(jsonData: GraphResult);
}
declare abstract class ArchesClient {
    registry: StaticResourceRegistry | null;
    abstract getGraphs(): Promise<GraphResult>;
    abstract getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;
    abstract getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;
    abstract getResources(graphId: string, limit: number, reloadIfSeen: boolean): Promise<StaticResource[]>;
    abstract getResource(resourceId: string): Promise<StaticResource>;
    abstract getCollection(collectionId: string): Promise<StaticCollection>;
    abstract getResourceSummaries(graphId: string, limit: number): Promise<StaticResourceSummary[]>;
    abstract getResourceTiles(resourceId: string): Promise<StaticTile[]>;
}
declare class ArchesClientRemote extends ArchesClient {
    archesUrl: string;
    constructor(archesUrl: string);
    getGraphs(): Promise<GraphResult>;
    getGraph(graph: StaticGraphMeta): Promise<StaticGraph>;
    getGraphByIdOnly(graphId: string): Promise<StaticGraph>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number, _reloadIfSeen: boolean): Promise<StaticResource[]>;
    getResourceSummaries(graphId: string, limit: number): Promise<StaticResourceSummary[]>;
    getResourceTiles(resourceId: string): Promise<StaticTile[]>;
}
declare class ArchesClientRemoteStatic extends ArchesClient {
    archesUrl: string;
    allGraphFile: () => string;
    graphToGraphFile?: (graph: StaticGraphMeta) => string;
    graphIdToGraphFile: (graphId: string) => string;
    graphIdToResourcesFiles: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
    resourceIdToFile: (resourceId: string) => string;
    collectionIdToFile: (collectionId: string) => string;
    constructor(archesUrl: string, { allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile, }?: {
        allGraphFile?: () => string;
        graphToGraphFile?: (graph: StaticGraphMeta) => string;
        graphIdToGraphFile?: (graphId: string) => string;
        graphIdToResourcesFiles?: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
        resourceIdToFile?: (resourceId: string) => string;
        collectionIdToFile?: (collectionId: string) => string;
    });
    getGraphs(): Promise<GraphResult>;
    getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;
    getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number, _reloadIfSeen: boolean): Promise<StaticResource[]>;
    getResourceSummaries(graphId: string, limit: number): Promise<StaticResourceSummary[]>;
    getResourceTiles(resourceId: string): Promise<StaticTile[]>;
}
declare class ArchesClientLocal extends ArchesClient {
    allGraphFile: () => string;
    graphToGraphFile?: (graph: StaticGraphMeta) => string;
    graphIdToGraphFile: (graphId: string) => string;
    graphIdToResourcesFiles: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
    resourceIdToFile: (resourceId: string) => string;
    collectionIdToFile: (collectionId: string) => string;
    __loadedFileCache: {
        [graphId: string]: string[];
    };
    private ensureFs;
    constructor({ allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile, }?: {
        allGraphFile?: () => string;
        graphToGraphFile?: (graph: StaticGraphMeta) => string;
        graphIdToGraphFile?: (graphId: string) => string;
        graphIdToResourcesFiles?: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
        resourceIdToFile?: (resourceId: string) => string;
        collectionIdToFile?: (collectionId: string) => string;
    });
    getGraphs(): Promise<GraphResult>;
    getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;
    getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;
    getResource(resourceId: string): Promise<StaticResource>;
    getCollection(collectionId: string): Promise<StaticCollection>;
    getResources(graphId: string, limit: number | null, reloadIfSeen: boolean): Promise<StaticResource[]>;
    getResourceSummaries(graphId: string, limit: number): Promise<StaticResourceSummary[]>;
    getResourceTiles(resourceId: string): Promise<StaticTile[]>;
}
declare const archesClient: ArchesClientRemote;
export { archesClient, ArchesClient, ArchesClientRemoteStatic, ArchesClientRemote, ArchesClientLocal, GraphResult, };
