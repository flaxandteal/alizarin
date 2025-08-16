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

  abstract getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;

  abstract getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;

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

  async getGraph(graph: StaticGraphMeta): Promise<StaticGraph> {
    return this.getGraphByIdOnly(graph.graphid);
  }

  async getGraphByIdOnly(graphId: string): Promise<StaticGraph> {
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
  allGraphFile: () => string;
  graphToGraphFile?: (graph: StaticGraphMeta) => string;
  graphIdToGraphFile: (graphId: string) => string;
  graphIdToResourcesFiles: (graphId: string) => string[];
  resourceIdToFile: (resourceId: string) => string;
  collectionIdToFile: (collectionId: string) => string;

  constructor(
    archesUrl: string,
    {
      allGraphFile,
      graphToGraphFile,
      graphIdToResourcesFiles,
      resourceIdToFile,
      collectionIdToFile,
      graphIdToGraphFile,
    }: {
      allGraphFile?: () => string,
      graphToGraphFile?: (graph: StaticGraphMeta) => string,
      graphIdToGraphFile?: (graphId: string) => string;
      graphIdToResourcesFiles?: (graphId: string) => string[];
      resourceIdToFile?: (resourceId: string) => string;
      collectionIdToFile?: (collectionId: string) => string;
    } = {},
  ) {
    super();
    this.archesUrl = archesUrl;
    this.allGraphFile = allGraphFile || (() => "resource_models/_all.json");
    this.graphToGraphFile = graphToGraphFile;
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

  async getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null> {
    if (!this.graphToGraphFile) {
      return this.getGraphByIdOnly(graph.graphid);
    }
    const response = await fetch(
      `${this.archesUrl}/${this.graphToGraphFile(graph)}`,
    );
    return (await response.json()).graph[0];
  }

  async getGraphByIdOnly(graphId: string): Promise<StaticGraph | null> {
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
  allGraphFile: () => string;
  graphToGraphFile?: (graph: StaticGraphMeta) => string;
  graphIdToGraphFile: (graphId: string) => string;
  graphIdToResourcesFiles: (graphId: string) => string[];
  resourceIdToFile: (resourceId: string) => string;
  collectionIdToFile: (collectionId: string) => string;

  constructor({
      allGraphFile,
      graphToGraphFile,
      graphIdToResourcesFiles,
      resourceIdToFile,
      collectionIdToFile,
      graphIdToGraphFile,
    }: {
      allGraphFile?: () => string,
      graphToGraphFile?: (graph: StaticGraphMeta) => string,
      graphIdToGraphFile?: (graphId: string) => string;
      graphIdToResourcesFiles?: (graphId: string) => string[];
      resourceIdToFile?: (resourceId: string) => string;
      collectionIdToFile?: (collectionId: string) => string;
    } = {}) {
    super();
    this.fs = import("fs");
    this.allGraphFile = allGraphFile || (() => "tests/definitions/models/_all.json");
    this.graphToGraphFile =
      graphToGraphFile ||
      ((graph: StaticGraphMeta) => `tests/definitions/models/${graph.graphid}.json`);
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
    const response = await fs.promises.readFile(this.allGraphFile(), "utf8");
    return new GraphResult(await JSON.parse(response));
  }

  async getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null> {
    const fs = await this.fs;
    const graphFile = this.graphToGraphFile ? this.graphToGraphFile(graph) : this.graphIdToGraphFile(graph.graphid);
    if (!graphFile) {
      return null;
    }
    const response = await fs.promises.readFile(
      graphFile,
      "utf8",
    );
    return await JSON.parse(response).graph[0];
  }

  async getGraphByIdOnly(graphId: string): Promise<StaticGraph | null> {
    const fs = await this.fs;
    const graphFile = this.graphIdToGraphFile(graphId);
    if (!graphFile) {
      return null;
    }
    const response = await fs.promises.readFile(
      graphFile,
      "utf8",
    );
    return await JSON.parse(response).graph[0];
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    const fs = await this.fs;
    const source = this.resourceIdToFile(resourceId);
    const response = await fs.promises.readFile(
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
    const response = await fs.promises.readFile(
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
      const response = JSON.parse(await fs.promises.readFile(file, "utf8"));
      const source = file;
      // const read = fs.createReadStream(file, { encoding: "utf8" });
      // let buffer = '';
      // let bufferLength = 0;
      // const response: IStringKeyedObject = await (new Promise(resolve => {
      //   read.pipe(bfj.unpipe((error: string, data: string) => {
      //     if (error) {
      //       throw Error(error);
      //     }
      //     return data;
      //   })).on('data', (data: string) => {
      //     const bl = Math.floor(buffer.length / 1000);
      //     bufferLength = bl;
      //     buffer += data;
      //   }).on('end', () => {
      //     resolve(JSON.parse(buffer));
      //   });
      // }));

      const resourceSet: StaticResource[] = response.business_data.resources.filter(
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
