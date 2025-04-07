import { StaticGraph, StaticResource } from "./static-types";
import { StaticCollection } from "./rdm";

class GraphResult {
  models: string[];

  constructor(jsonData: GraphResult) {
    this.models = jsonData.models;
  }
}

abstract class ArchesClient {
  abstract getGraphs(): Promise<GraphResult>;

  abstract getGraph(graphId: string): Promise<StaticGraph>;

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

  async getGraph(graphId: string): Promise<StaticGraph> {
    const response = await fetch(
      `${this.archesUrl}/graphs/${graphId}?format=arches-json&gen=`,
    );
    return await response.json();
  }

  async getResource(resourceId: string): Promise<StaticResource> {}

  async getCollection(collectionId: string): Promise<StaticCollection> {}

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

  async getGraph(graphId: string): Promise<StaticGraph> {
    const response = await fetch(
      `${this.archesUrl}/${this.graphIdToGraphFile(graphId)}`,
    );
    return (await response.json()).graph[0];
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    const response = await fetch(
      `${this.archesUrl}/${this.resourceIdToFile(resourceId)}`,
    );
    return await response.json();
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
      const response = await fetch(`${this.archesUrl}/${file}`);
      const resourceSet = (await response.json()).business_data.resources;
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
    this.allGraphFile = allGraphFile || (() => "public/models/_all.json");
    this.graphIdToGraphFile =
      graphIdToGraphFile ||
      ((graphId: string) => `public/models/${graphId}.json`);
    this.graphIdToResourcesFiles =
      graphIdToResourcesFiles ||
      ((graphId: string) => [`public/resources/_${graphId}.json`]);
    this.resourceIdToFile =
      resourceIdToFile ||
      ((resourceId: string) => `public/resources/${resourceId}.json`);
    this.collectionIdToFile =
      collectionIdToFile ||
      ((collectionId: string) => `public/collections/${collectionId}.json`);
  }

  async getGraphs(): Promise<GraphResult> {
    const fs = await this.fs;
    const response = await fs.readFile(this.allGraphFile(), "utf8");
    return await JSON.parse(response);
  }

  async getGraph(graphId: string): Promise<StaticGraph> {
    const fs = await this.fs;
    const response = await fs.readFile(
      this.graphIdToGraphFile(graphId),
      "utf8",
    );
    return await JSON.parse(response).graph[0];
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    const fs = await this.fs;
    const response = await fs.readFile(
      this.resourceIdToFile(resourceId),
      "utf8",
    );
    return await JSON.parse(response);
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
      const resourceSet = (await JSON.parse(response)).business_data.resources;
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
};
