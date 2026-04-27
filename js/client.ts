import { StaticGraphMeta, StaticGraph, StaticResource, StaticResourceSummary, StaticResourceRegistry, StaticTile } from "./static-types";
import { StaticCollection } from "./rdm";

class GraphResult {
  models: {[graphId: string]: StaticGraphMeta};

  constructor(jsonData: GraphResult) {
    this.models = Object.fromEntries(
      Object.entries(jsonData.models).map(([k, v]) => {
        // Inject graphid from the key if not present in the value
        const data = { graphid: k, ...v };
        return [k, new StaticGraphMeta(data)];
      })
    );
  }
}

abstract class ArchesClient {
  registry: StaticResourceRegistry | null = null;

  abstract getGraphs(): Promise<GraphResult>;

  abstract getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null>;

  abstract getGraphByIdOnly(graphId: string): Promise<StaticGraph | null>;

  abstract getResources(
    graphId: string,
    limit: number,
    reloadIfSeen: boolean
  ): Promise<StaticResource[]>;

  abstract getResource(resourceId: string): Promise<StaticResource>;

  abstract getCollection(collectionId: string): Promise<StaticCollection>;

  // New summary-based methods for performance optimization
  abstract getResourceSummaries(
    graphId: string,
    limit: number,
  ): Promise<StaticResourceSummary[]>;

  abstract getResourceTiles(resourceId: string): Promise<StaticTile[]>;
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
    const jsonText = await response.text();
    return StaticGraph.fromJsonString(jsonText);
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
    _reloadIfSeen: boolean
  ): Promise<StaticResource[]> {
    const response = await fetch(
      `${this.archesUrl}/resources?graph_uuid=${graphId}&format=arches-json&hide_empty_nodes=false&compact=false&limit=${limit}`,
    );
    return await response.json();
  }

  async getResourceSummaries(
    graphId: string,
    limit: number,
  ): Promise<StaticResourceSummary[]> {
    // For remote client, this would ideally be a separate endpoint
    // For now, fall back to getting full resources and extracting summaries
    const resources = await this.getResources(graphId, limit, false);
    return resources.map(resource => new StaticResourceSummary({
      resourceinstanceid: resource.resourceinstance.resourceinstanceid,
      graph_id: resource.resourceinstance.graph_id,
      name: resource.resourceinstance.name,
      descriptors: resource.resourceinstance.descriptors || {},
      metadata: resource.metadata || {},
      publication_id: resource.resourceinstance.publication_id,
      principaluser_id: resource.resourceinstance.principaluser_id,
      legacyid: resource.resourceinstance.legacyid,
      graph_publication_id: resource.resourceinstance.graph_publication_id
    }));
  }

  async getResourceTiles(resourceId: string): Promise<StaticTile[]> {
    const resource = await this.getResource(resourceId);
    return resource.tiles || [];
  }
}

class ArchesClientRemoteStatic extends ArchesClient {
  archesUrl: string;
  allGraphFile: () => string;
  graphToGraphFile?: (graph: StaticGraphMeta) => string;
  graphIdToGraphFile: (graphId: string) => string;
  graphIdToResourcesFiles: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
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
      graphIdToResourcesFiles?: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
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
    const jsonText = await response.text();
    return StaticGraph.fromJsonString(jsonText);
  }

  async getGraphByIdOnly(graphId: string): Promise<StaticGraph | null> {
    const response = await fetch(
      `${this.archesUrl}/${this.graphIdToGraphFile(graphId)}`,
    );
    const jsonText = await response.text();
    return StaticGraph.fromJsonString(jsonText);
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    const source = `${this.archesUrl}/${this.resourceIdToFile(resourceId)}`;
    const response = await fetch(source);
    const jsonText = await response.text();
    const resource = StaticResource.fromJsonString(jsonText);
    resource.__source = source;
    return resource;
  }

  async getCollection(collectionId: string): Promise<StaticCollection> {
    const jsonUrl = `${this.archesUrl}/${this.collectionIdToFile(collectionId)}`;
    const response = await fetch(jsonUrl);
    if (response.ok) {
      return await response.json();
    }
    // JSON not available — try SKOS XML fallback
    const xmlUrl = jsonUrl.replace(/\.json$/, '.xml');
    const xmlResponse = await fetch(xmlUrl);
    if (!xmlResponse.ok) {
      throw new Error(`Collection ${collectionId} not found (tried JSON and XML)`);
    }
    const xmlText = await xmlResponse.text();
    const { parseSkosXmlToCollection, getRdmNamespaceRaw } = await import("../pkg/alizarin");
    const baseUri = getRdmNamespaceRaw() || xmlResponse.url;
    return parseSkosXmlToCollection(xmlText, baseUri);
  }

  async getResources(
    graphId: string,
    limit: number,
    _reloadIfSeen: boolean
  ): Promise<StaticResource[]> {
    const resources: StaticResource[] = [];
    const result = this.graphIdToResourcesFiles(graphId);
    const files = (typeof result[Symbol.asyncIterator] === 'function' || typeof result[Symbol.iterator] === 'function')
      ? result
      : await result;
    for await (const file of files) {
      const source = `${this.archesUrl}/${file}`;
      const response = await fetch(source);
      // Use bulk parsing in Rust - single JSON string copy, parses all resources at once
      const jsonText = await response.text();
      const resourceSet: StaticResource[] = StaticResource.fromBusinessDataJsonString(jsonText);
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

  async getResourceSummaries(
    graphId: string,
    limit: number,
  ): Promise<StaticResourceSummary[]> {
    const summaries: StaticResourceSummary[] = [];
    const result = this.graphIdToResourcesFiles(graphId);
    const files = (typeof result[Symbol.asyncIterator] === 'function' || typeof result[Symbol.iterator] === 'function')
      ? result
      : await result;
    for await (const file of files) {
      const source = `${this.archesUrl}/${file}`;
      const response = await fetch(source);
      // Use bulk parsing in Rust - parses entire file, extracts only summary fields
      const jsonText = await response.text();
      const summarySet: StaticResourceSummary[] = StaticResourceSummary.summariesFromBusinessDataJsonString(jsonText);
      summaries.push(...summarySet);

      if (limit && summaries.length >= limit) {
        return summaries.slice(0, limit);
      }
    }
    return limit ? summaries.slice(0, limit) : summaries;
  }

  async getResourceTiles(resourceId: string): Promise<StaticTile[]> {
    // For static client, we need to load the full resource to get tiles
    const resource = await this.getResource(resourceId);
    return resource.tiles || [];
  }
}

class ArchesClientLocal extends ArchesClient {
  allGraphFile: () => string;
  graphToGraphFile?: (graph: StaticGraphMeta) => string;
  graphIdToGraphFile: (graphId: string) => string;
  graphIdToResourcesFiles: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
  resourceIdToFile: (resourceId: string) => string;
  collectionIdToFile: (collectionId: string) => string;
  // This allows reloading of files not searched for a given graphId
  // i.e. only searching each file ONCE for a graphId.
  __loadedFileCache: {[graphId: string]: string[]};

  private async ensureFs() {
    // Only import fs when actually needed, and only in Node.js environment
    if (typeof process === 'undefined' || !process.versions?.node) {
      throw new Error('ArchesClientLocal requires Node.js filesystem access. Use ArchesClientRemoteStatic for browser environments.');
    }
    return import('fs');
  }

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
      graphIdToResourcesFiles?: ((graphId: string) => string[]) | ((graphId: string) => Promise<string[]>) | ((graphId: string) => AsyncGenerator<string>);
      resourceIdToFile?: (resourceId: string) => string;
      collectionIdToFile?: (collectionId: string) => string;
    } = {}) {
    super();
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
    this.__loadedFileCache = {};
  }

  async getGraphs(): Promise<GraphResult> {
    const fs = await this.ensureFs();
    const response = await fs.promises.readFile(this.allGraphFile(), "utf8");
    return new GraphResult(await JSON.parse(response));
  }

  async getGraph(graph: StaticGraphMeta): Promise<StaticGraph | null> {
    const fs = await this.ensureFs();
    const graphFile = this.graphToGraphFile ? this.graphToGraphFile(graph) : this.graphIdToGraphFile(graph.graphid);
    if (!graphFile) {
      return null;
    }
    const jsonText = await fs.promises.readFile(
      graphFile,
      "utf8",
    );
    return StaticGraph.fromJsonString(jsonText);
  }

  async getGraphByIdOnly(graphId: string): Promise<StaticGraph | null> {
    const fs = await this.ensureFs();
    const graphFile = this.graphIdToGraphFile(graphId);
    if (!graphFile) {
      return null;
    }
    const jsonText = await fs.promises.readFile(
      graphFile,
      "utf8",
    );
    return StaticGraph.fromJsonString(jsonText);
  }

  async getResource(resourceId: string): Promise<StaticResource> {
    const fs = await this.ensureFs();
    const source = this.resourceIdToFile(resourceId);
    const response = await fs.promises.readFile(
      source,
      "utf8",
    );
    const resource: StaticResource[] = JSON.parse(response)
      .business_data
      .resources
      .filter(resource => resource.resourceinstance.resourceinstanceid === resourceId)
      .map(resource => new StaticResource(resource))[0];
    resource.__source = source;
    return resource;
  }

  async getCollection(collectionId: string): Promise<StaticCollection> {
    const fs = await this.ensureFs();
    const jsonPath = this.collectionIdToFile(collectionId);
    try {
      const response = await fs.promises.readFile(jsonPath, "utf8");
      return JSON.parse(response);
    } catch {
      // JSON not available — try SKOS XML fallback
      const xmlPath = jsonPath.replace(/\.json$/, '.xml');
      const xmlContent = await fs.promises.readFile(xmlPath, "utf8");
      const { parseSkosXmlToCollection, getRdmNamespaceRaw } = await import("../pkg/alizarin");
      const baseUri = getRdmNamespaceRaw() || "http://localhost/";
      return parseSkosXmlToCollection(xmlContent, baseUri);
    }
  }

  async getResources(
    graphId: string,
    limit: number | null,
    reloadIfSeen: boolean
  ): Promise<StaticResource[]> {
    if (!this.registry) {
      throw new Error('ArchesClientLocal requires a StaticResourceRegistry — set via staticStore');
    }
    const fs = await this.ensureFs();
    const resources: StaticResource[] = [];
    const result = this.graphIdToResourcesFiles(graphId);
    const files = (typeof result[Symbol.asyncIterator] === 'function' || typeof result[Symbol.iterator] === 'function')
      ? result
      : await result;
    for await (const file of files) {
      if (this.__loadedFileCache[graphId] && this.__loadedFileCache[graphId].includes(file)) {
        if (!reloadIfSeen) {
          continue;
        }
      } else {
        if (!this.__loadedFileCache[graphId]) {
          this.__loadedFileCache[graphId] = [];
        }
        this.__loadedFileCache[graphId].push(file);
      }
      // Read as raw bytes — no UTF-8 string, no V8 string length limit.
      // Parsing happens entirely in Rust via loadFromBusinessDataBytes.
      const buffer = await fs.promises.readFile(file);
      const bytes = new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
      const refs: Array<{ resourceinstanceid: string; graph_id: string; isPublic: boolean }> =
        this.registry.loadFromBusinessDataBytes(bytes, true, true);

      for (const ref of refs) {
        if (ref.graph_id !== graphId) continue;
        const sr = this.registry.getFull(ref.resourceinstanceid);
        if (sr) {
          resources.push(sr);
          if (limit && resources.length >= limit) {
            break;
          }
        }
      }
      if (limit && resources.length >= limit) {
        break;
      }
    }
    return resources;
  }

  async getResourceSummaries(
    graphId: string,
    limit: number,
  ): Promise<StaticResourceSummary[]> {
    if (!this.registry) {
      throw new Error('ArchesClientLocal requires a StaticResourceRegistry — set via staticStore');
    }
    const fs = await this.ensureFs();
    const summaries: StaticResourceSummary[] = [];
    const result = this.graphIdToResourcesFiles(graphId);
    const files = (typeof result[Symbol.asyncIterator] === 'function' || typeof result[Symbol.iterator] === 'function')
      ? result
      : await result;
    for await (const file of files) {
      const buffer = await fs.promises.readFile(file);
      const bytes = new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
      const refs: Array<{ resourceinstanceid: string; graph_id: string; isPublic: boolean }> =
        this.registry.loadFromBusinessDataBytes(bytes, false, true);

      for (const ref of refs) {
        if (ref.graph_id !== graphId) continue;
        const summary = this.registry.getSummary(ref.resourceinstanceid);
        if (summary) {
          summaries.push(summary);
          if (limit && summaries.length >= limit) {
            return summaries;
          }
        }
      }
    }
    return limit ? summaries.slice(0, limit) : summaries;
  }

  async getResourceTiles(resourceId: string): Promise<StaticTile[]> {
    const resource = await this.getResource(resourceId);
    return resource.tiles || [];
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
