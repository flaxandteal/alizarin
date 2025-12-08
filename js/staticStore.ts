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
    useCache: boolean = true
  ): AsyncIterable<StaticResource> {
    // IMPORTANT: When cacheMetadataOnly is true, this cache only stores metadata (StaticResourceMetadata),
    // not full StaticResource objects. This means:
    // 1. The cache check below (instanceof StaticResource) will never find cached entries
    // 2. We rely entirely on the ArchesClient's file-level caching (via reloadIfSeen parameter)
    // 3. We always pass reloadIfSeen=false so files aren't re-read, but StaticResources are reconstructed each time
    // This is intentional for memory efficiency - we don't keep full resources in memory.

    let toFind: number = limit | -1;

    // Only check in-memory cache if we're storing full StaticResource objects
    if (useCache && !this.cacheMetadataOnly) {
      for (const entry of this.cache.values()) {
        if (entry instanceof StaticResource && entry.resourceinstance.graph_id === graphId) {
          toFind -= 1;
          yield entry;
          if (toFind === 0) return;
        }
      }
    }
    toFind = toFind > 0 ? toFind : 0;

    // When cacheMetadataOnly is true, we don't keep full StaticResources in memory,
    // so we need to reload files to reconstruct them (reloadIfSeen=true).
    // Otherwise respect the useCache parameter for file-level deduplication.
    const reloadIfSeen = this.cacheMetadataOnly ? true : !useCache;
    const resourcesJSON: (StaticResource | StaticResourceSummary)[] =
      await this.archesClient.getResources(graphId, toFind, reloadIfSeen);
    for (let resourceJSON of resourcesJSON.values()) {
      if (!(resourceJSON instanceof StaticResource) && resourceJSON.resourceinstanceid) {
        resourceJSON = await this.archesClient.getResource(resourceJSON.resourceinstanceid);
      }
      const resource = new StaticResource(resourceJSON);
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
    // Cache based on cacheMetadataOnly setting
    this.cache.set(id, this.cacheMetadataOnly ? resource.resourceinstance : resource);
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

  async loadTiles(id: string, nodegroupId?: string | null): Promise<StaticTile[]> {
    // Check if we already have full resource with tiles in cache
    const cached = this.cache.get(id);
    if (cached instanceof StaticResource && cached.tiles) {
      const tiles = cached.tiles;
      // Filter by nodegroup if specified
      if (nodegroupId) {
        return tiles.filter(tile => tile.nodegroup_id === nodegroupId);
      }
      return tiles;
    }

    // Load tiles on-demand
    const tiles = await this.archesClient.getResourceTiles(id);
    // Filter by nodegroup if specified
    if (nodegroupId) {
      return tiles.filter(tile => tile.nodegroup_id === nodegroupId);
    }
    return tiles;
  }

  async ensureFullResource(id: string): Promise<StaticResource> {
    const cached = this.cache.get(id);

    if (cached instanceof StaticResource && cached.tiles) {
      return cached;
    }

    if (cached instanceof StaticResourceSummary) {
      // We have summary, need to load full resource
      const fullResource = await this.loadOne(id);
      this.cache.set(id, fullResource);
      return fullResource;
    }

    // Load full resource
    return await this.loadOne(id);
  }
}

const staticStore = new StaticStore(archesClient);

export { staticStore };
