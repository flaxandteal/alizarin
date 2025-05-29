import { StaticGraphMeta, StaticGraph, StaticResource } from './static-types';
import { StaticCollection } from './rdm';
declare class GraphResult {
    models: {
        [graphId: string]: StaticGraphMeta;
    };
    constructor(jsonData: GraphResult);
}
declare abstract class ArchesClient {
    abstract getGraphs(): Promise<GraphResult>;
    abstract getGraph(graphId: string): Promise<StaticGraph | null>;
    abstract getResources(graphId: string, limit: number): Promise<StaticResource[]>;
    abstract getResource(resourceId: string): Promise<StaticResource>;
    abstract getCollection(collectionId: string): Promise<StaticCollection>;
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
declare const archesClient: ArchesClientRemote;
export { archesClient, ArchesClient, ArchesClientRemoteStatic, ArchesClientRemote, ArchesClientLocal, GraphResult, };
