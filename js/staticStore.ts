import { archesClient, ArchesClient } from "./client.ts";
import {
  StaticResource,
  StaticResourceSummary,
  StaticResourceMetadata,
  StaticTile,
} from "./static-types";

// TODO: this does not currently cache, to avoid
//  memory leaks.
class StaticStore {
  archesClient: ArchesClient;
  cache: Map<string, StaticResource | StaticResourceSummary | StaticResourceMetadata>;
  cacheMetadataOnly: boolean;

  constructor(archesClient: ArchesClient, cacheMetadataOnly: boolean = true) {
    this.archesClient = archesClient;
    this.cache = new Map();
    this.cacheMetadataOnly = cacheMetadataOnly;
  }

  async getMeta(id: string, onlyIfCached: boolean = true): Promise<StaticResourceMetadata | null> {
    if (this.cache.has(id)) {
      const resource = this.cache.get(id);
      if (resource instanceof StaticResource) {
        return resource.resourceinstance;
      } else if (resource instanceof StaticResourceSummary) {
        return resource.toMetadata();
      }
      return resource || null;
    }

    if (!onlyIfCached) {
      const resource = await this.loadOne(id);
      return resource.resourceinstance;
    }
    return null;
  }

  async* loadAll(
    graphId: string,
    limit: number | undefined = undefined,
  ): AsyncIterable<StaticResource> {
    const resourcesJSON: StaticResource[] =
      await this.archesClient.getResources(graphId, limit || 0);
    for (const resourceJSON of resourcesJSON.values()) {
      const resource = new StaticResource(resourceJSON);
      console.log(resource.resourceinstance.graph_id, 1);
      if (resource.resourceinstance.graph_id !== graphId) {
        continue;
      }
      this.cache.set(
        resource.resourceinstance.resourceinstanceid,
        this.cacheMetadataOnly ? resource.resourceinstance : resource
      );
      yield resource;
    }
  }

  async loadOne(id: string): Promise<StaticResource> {
    if (this.cache.has(id)) {
      const resource = this.cache.get(id);
      if (resource instanceof StaticResource) {
        return resource;
      }
    }

    const resourceJSON: StaticResource =
      await this.archesClient.getResource(id);
    const resource = new StaticResource(resourceJSON);
    if (this.cacheMetadataOnly) {
      this.cache.set(id, this.cacheMetadataOnly ? resource.resourceinstance : resource);
    }
    return resource;
  }

  // New summary-based loading methods
  async* loadAllSummaries(
    graphId: string,
    limit: number | undefined = undefined,
  ): AsyncIterable<StaticResourceSummary> {
    const summariesJSON: StaticResourceSummary[] =
      await this.archesClient.getResourceSummaries(graphId, limit || 0);
    for (const summaryJSON of summariesJSON.values()) {
      const summary = new StaticResourceSummary(summaryJSON);
      if (summary.graph_id !== graphId) {
        continue;
      }
      // Cache summary only
      this.cache.set(summary.resourceinstanceid, summary);
      yield summary;
    }
  }

  async loadTiles(id: string): Promise<StaticTile[]> {
    // Check if we already have full resource with tiles
    const cached = this.cache.get(id);
    if (cached instanceof StaticResource && cached.__tilesLoaded) {
      return cached.tiles || [];
    }
    
    // Load tiles on-demand
    return await this.archesClient.getResourceTiles(id);
  }

  async ensureFullResource(id: string): Promise<StaticResource> {
    const cached = this.cache.get(id);
    
    if (cached instanceof StaticResource && cached.__tilesLoaded) {
      return cached;
    }
    
    if (cached instanceof StaticResourceSummary) {
      // We have summary, need to load full resource
      const fullResource = await this.loadOne(id);
      fullResource.__tilesLoaded = true;
      this.cache.set(id, fullResource);
      return fullResource;
    }
    
    // Load full resource
    return await this.loadOne(id);
  }
}

const staticStore = new StaticStore(archesClient);

export { staticStore };
