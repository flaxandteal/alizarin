import { StaticGraphMeta, StaticGraph, StaticResource } from "./static-types";
import { StaticCollection } from "./rdm";

class GraphResult {
  models: {[graphId: string]: StaticGraphMeta};

  constructor(jsonData: GraphResult) {
    this.models = Object.fromEntries(
      Object.entries(jsonData.models).map(([k, v]) => [k, new StaticGraphMeta(v)])
    );
  }
}

abstract class ArchesClient {
  abstract getGraphs(): Promise<GraphResult>;

  abstract getGraph(graphId: string): Promise<StaticGraph | null>;

  abstract getResources(
    graphId: string,
    limit: number,
  ): Promise<StaticResource[]>;

  abstract getResource(resourceId: string): Promise<StaticResource>;

  abstract getCollection(collectionId: string): Promise<StaticCollection>;
}

class ArchesClientRemote extends ArchesClient {
  archesUrl: string;

  constructor(archesUrl: string) {
    super();
    this.archesUrl = archesUrl;
  }

  async getGraphs(): Promise<GraphResult> {
    const response = await fetch(
      `${this.archesUrl}/api/arches/graphs?format=arches-json&hide_empty_nodes=false&compact=false`,
    );
    return await response.json();
  }

  async getGraph(graphId: string | null): Promise<StaticGraph> {
    const response = await fetch(
      `${this.archesUrl}/graphs/${graphId}?format=arches-json&gen=`,
    );
    return await response.json();
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    throw Error(`Not implemented yet: getResource(${resourceId}`);
  }

  async getCollection(collectionId: string): Promise<StaticCollection> {
    throw Error(`Not implemented yet: getCollection(${collectionId}`);
  }

  async getResources(
    graphId: string,
    limit: number,
  ): Promise<StaticResource[]> {
    const response = await fetch(
      `${this.archesUrl}/resources?graph_uuid=${graphId}&format=arches-json&hide_empty_nodes=false&compact=false&limit=${limit}`,
    );
    return await response.json();
  }
}

class ArchesClientRemoteStatic extends ArchesClient {
  archesUrl: string;
  allGraphFile: Function;
  graphIdToGraphFile: Function;
  graphIdToResourcesFiles: Function;
  resourceIdToFile: Function;
  collectionIdToFile: Function;

  constructor(
    archesUrl: string,
    {
      allGraphFile,
      graphIdToGraphFile,
      graphIdToResourcesFiles,
      resourceIdToFile,
      collectionIdToFile,
    }: { [k: string]: Function } = {},
  ) {
    super();
    this.archesUrl = archesUrl;
    this.allGraphFile = allGraphFile || (() => "resource_models/_all.json");
    this.graphIdToGraphFile =
      graphIdToGraphFile ||
      ((graphId: string) => `resource_models/${graphId}.json`);
    this.graphIdToResourcesFiles =
      graphIdToResourcesFiles ||
      ((graphId: string) => [`business_data/_${graphId}.json`]);
    this.resourceIdToFile =
      resourceIdToFile ||
      ((resourceId: string) => `business_data/${resourceId}.json`);
    this.collectionIdToFile =
      collectionIdToFile ||
      ((collectionId: string) => `collections/${collectionId}.json`);
  }

  async getGraphs(): Promise<GraphResult> {
    const response = await fetch(`${this.archesUrl}/${this.allGraphFile()}`);
    return await response.json();
  }

  async getGraph(graphId: string): Promise<StaticGraph | null> {
    const response = await fetch(
      `${this.archesUrl}/${this.graphIdToGraphFile(graphId)}`,
    );
    return (await response.json()).graph[0];
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    const source = `${this.archesUrl}/${this.resourceIdToFile(resourceId)}`;
    const response = await fetch(source);
    return response.json().then((response: StaticResource) => {
      response.__source = source;
      return response;
    });
  }

  async getCollection(collectionId: string): Promise<StaticCollection> {
    const response = await fetch(
      `${this.archesUrl}/${this.collectionIdToFile(collectionId)}`,
    );
    return await response.json();
  }

  async getResources(
    graphId: string,
    limit: number,
  ): Promise<StaticResource[]> {
    const resources = [];
    for (const file of this.graphIdToResourcesFiles(graphId)) {
      const source = `${this.archesUrl}/${file}`;
      const response = await fetch(source);
      const resourceSet: StaticResource[] = (await response.json()).business_data.resources;
      for (const resource of resourceSet) {
        resource.__source = source;
      }
      resources.push(...(limit ? resourceSet.slice(0, limit) : resourceSet));
      if (limit && resources.length > limit) {
        break;
      }
    }
    return resources;
  }
}

class ArchesClientLocal extends ArchesClient {
  fs: any;
  allGraphFile: Function;
  graphIdToGraphFile: Function;
  graphIdToResourcesFiles: Function;
  resourceIdToFile: Function;
  collectionIdToFile: Function;

  constructor({
    allGraphFile,
    graphIdToGraphFile,
    graphIdToResourcesFiles,
    resourceIdToFile,
    collectionIdToFile,
  }: { [k: string]: Function } = {}) {
    super();
    this.fs = import("fs").then((fs) => {
      return fs.promises;
    });
    this.allGraphFile = allGraphFile || (() => "tests/definitions/models/_all.json");
    this.graphIdToGraphFile =
      graphIdToGraphFile ||
      ((graphId: string) => `tests/definitions/models/${graphId}.json`);
    this.graphIdToResourcesFiles =
      graphIdToResourcesFiles ||
      ((graphId: string) => [`tests/definitions/resources/_${graphId}.json`]);
    this.resourceIdToFile =
      resourceIdToFile ||
      ((resourceId: string) => `tests/definitions/resources/${resourceId}.json`);
    this.collectionIdToFile =
      collectionIdToFile ||
      ((collectionId: string) => `tests/definitions/collections/${collectionId}.json`);
  }

  async getGraphs(): Promise<GraphResult> {
    const fs = await this.fs;
    const response = await fs.readFile(this.allGraphFile(), "utf8");
    return new GraphResult(await JSON.parse(response));
  }

  async getGraph(graphId: string): Promise<StaticGraph | null> {
    const fs = await this.fs;
    const graphFile = this.graphIdToGraphFile(graphId);
    if (!graphFile) {
      return null;
    }
    const response = await fs.readFile(
      graphFile,
      "utf8",
    );
    return await JSON.parse(response).graph[0];
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    const fs = await this.fs;
    const source = this.resourceIdToFile(resourceId);
    const response = await fs.readFile(
      source,
      "utf8",
    );
    return JSON.parse(response).then((resource: StaticResource) => {
      resource.__source = source;
      return resource;
    });
  }

  async getCollection(collectionId: string): Promise<StaticCollection> {
    const fs = await this.fs;
    const response = await fs.readFile(
      this.collectionIdToFile(collectionId),
      "utf8",
    );
    return await JSON.parse(response);
  }

  async getResources(
    graphId: string,
    limit: number | null,
  ): Promise<StaticResource[]> {
    const fs = await this.fs;
    const resources = [];
    for (const file of this.graphIdToResourcesFiles(graphId)) {
      const response = await fs.readFile(file, "utf8");
      const source = file;
      const resourceSet: StaticResource[] = (await JSON.parse(response)).business_data.resources.filter(
        (resource: StaticResource) => graphId === resource.resourceinstance.graph_id
      );
      for (const resource of resourceSet) {
        resource.__source = source;
      }
      resources.push(...(limit ? resourceSet.slice(0, limit) : resourceSet));
      if (limit && resources.length > limit) {
        break;
      }
    }
    return resources;
  }
}

const archesClient = new ArchesClientRemote("http://localhost:8000");

export {
  archesClient,
  ArchesClient,
  ArchesClientRemoteStatic,
  ArchesClientRemote,
  ArchesClientLocal,
  GraphResult,
};
