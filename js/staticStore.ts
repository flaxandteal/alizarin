import { ArchesClient } from "./client.ts";
import {
  StaticResource,
  StaticResourceSummary,
  StaticResourceMetadata,
  StaticResourceRegistry,
  StaticTile,
} from "./static-types";

/**
 * StaticStore - Thin wrapper around StaticResourceRegistry with optional archesClient fallback
 *
 * Primary mode: Resources are pre-loaded into the registry before use.
 * Fallback mode: If archesClient is set and resource not in registry, loads on-demand.
 *
 * Methods return sync results wrapped in Promises for API compatibility.
 */
class StaticStore {
  private _registry: StaticResourceRegistry | null = null;
  archesClient: ArchesClient | null = null;

  constructor(registry?: StaticResourceRegistry) {
    if (registry) {
      this._registry = registry;
    }
  }

  /**
   * Get or create the registry (lazy initialization for WASM timing)
   */
  get registry(): StaticResourceRegistry {
    if (!this._registry) {
      this._registry = new StaticResourceRegistry();
    }
    return this._registry;
  }

  /**
   * Set the registry (allows replacing with a pre-populated registry)
   */
  set registry(registry: StaticResourceRegistry) {
    this._registry = registry;
  }

  /**
   * Get metadata for a resource from the registry (or load via archesClient if available).
   */
  async getMeta(id: string, onlyIfCached: boolean = true): Promise<StaticResourceMetadata | null> {
    const summary = this.registry.getSummary(id);
    if (summary) {
      return summary.toMetadata();
    }

    // Fallback: load via archesClient if not in registry
    if (!onlyIfCached && this.archesClient) {
      const resource = await this.loadOne(id);
      return resource.resourceinstance;
    }
    return null;
  }

  /**
   * Get all full resources for a graph.
   * First yields from registry, then loads remaining via archesClient if available.
   */
  async* loadAll(
    graphId: string,
    limit: number | undefined = undefined,
    useCache: boolean = true
  ): AsyncIterable<StaticResource> {
    let count = 0;
    const yielded = new Set<string>();

    // First: yield from registry
    if (useCache) {
      const resources = this.registry.getAllFullForGraph(graphId);
      for (const resource of resources) {
        if (limit && count >= limit) return;
        yielded.add(resource.resourceinstance.resourceinstanceid);
        yield resource;
        count++;
      }
    }

    // Second: load remaining via archesClient if available
    if (this.archesClient && (!limit || count < limit)) {
      const remaining = limit ? limit - count : undefined;
      const resourcesJSON = await this.archesClient.getResources(graphId, remaining || 0, !useCache);

      for (let resourceJSON of resourcesJSON) {
        if (limit && count >= limit) return;

        // Skip if already yielded from registry
        const id = resourceJSON.resourceinstanceid ||
          (resourceJSON as any).resourceinstance?.resourceinstanceid;
        if (id && yielded.has(id)) continue;

        // Load full resource if we got a summary
        let resource: StaticResource;
        if (resourceJSON instanceof StaticResource) {
          resource = resourceJSON;
        } else if (resourceJSON.resourceinstanceid) {
          // It's a summary, load the full resource
          resource = await this.archesClient.getResource(resourceJSON.resourceinstanceid);
        } else {
          // Plain JSON object, wrap in StaticResource
          resource = new StaticResource(resourceJSON);
        }

        if (resource.resourceinstance.graph_id !== graphId) continue;

        // Get the resource ID before merging (merge consumes the resource)
        const resourceId = resource.resourceinstance.resourceinstanceid;

        // Add to registry first (this consumes the resource)
        this.registry.mergeFromResources([resource], true, false);

        // Get fresh copy from registry to yield (the original is now consumed)
        const fresh = this.registry.getFull(resourceId);
        if (fresh) {
          yield fresh;
          count++;
        }
      }
    }
  }

  /**
   * Get a full resource from the registry (or load via archesClient if available).
   */
  async loadOne(id: string): Promise<StaticResource> {
    // Check registry first
    const cached = this.registry.getFull(id);
    if (cached) {
      return cached;
    }

    // Fallback: load via archesClient
    if (this.archesClient) {
      const resource = await this.archesClient.getResource(id);
      // Get the actual resourceinstanceid before merging (merge consumes the resource)
      // The registry stores by resourceinstanceid, not the lookup ID (which may be a slug)
      const resourceId = resource.resourceinstance.resourceinstanceid;
      // mergeFromResources CONSUMES the resource (transfers ownership to Rust),
      // so we must get a fresh copy from the registry after merging
      this.registry.mergeFromResources([resource], true, false);
      // Get fresh copy from registry using the actual resourceinstanceid
      const fresh = this.registry.getFull(resourceId);
      if (!fresh) {
        throw new Error(`Resource ${id} (${resourceId}) was merged but not found in registry`);
      }
      return fresh;
    }

    throw new Error(`Resource ${id} not in registry and no archesClient available.`);
  }

  /**
   * Get all summaries for a graph.
   */
  async* loadAllSummaries(
    graphId: string,
    limit: number | undefined = undefined,
  ): AsyncIterable<StaticResourceSummary> {
    let count = 0;
    const yielded = new Set<string>();

    // First: yield from registry (convert full resources to summaries)
    const resources = this.registry.getAllFullForGraph(graphId);
    for (const resource of resources) {
      if (limit && count >= limit) return;
      yielded.add(resource.resourceinstance.resourceinstanceid);
      yield StaticResourceSummary.fromResource(resource);
      count++;
    }

    // Second: load summaries via archesClient if available
    if (this.archesClient && (!limit || count < limit)) {
      const remaining = limit ? limit - count : 0;
      const summariesJSON = await this.archesClient.getResourceSummaries(graphId, remaining);

      for (const summaryJSON of summariesJSON) {
        if (limit && count >= limit) return;

        const summary = new StaticResourceSummary(summaryJSON);
        if (summary.graph_id !== graphId) continue;

        // Skip if already yielded from registry
        if (yielded.has(summary.resourceinstanceid)) continue;

        // Add to registry as summary only
        this.registry.insert(summary);

        yield summary;
        count++;
      }
    }
  }

  /**
   * Get tiles for a resource.
   */
  async loadTiles(id: string, nodegroupId?: string | null): Promise<StaticTile[]> {
    // Try registry first
    const cached = this.registry.getFull(id);
    if (cached && cached.tilesLoaded) {
      const tiles = cached.tiles ?? [];
      return nodegroupId ? tiles.filter(tile => tile.nodegroup_id === nodegroupId) : tiles;
    }

    // Fallback: load tiles via archesClient
    if (this.archesClient) {
      const tiles = await this.archesClient.getResourceTiles(id);
      return nodegroupId ? tiles.filter(tile => tile.nodegroup_id === nodegroupId) : tiles;
    }

    throw new Error(`Resource ${id} tiles not in registry and no archesClient available.`);
  }

  /**
   * Get a full resource with tiles loaded.
   */
  async ensureFullResource(id: string): Promise<StaticResource> {
    // Check registry for full resource with tiles
    const cached = this.registry.getFull(id);
    if (cached && cached.tilesLoaded) {
      return cached;
    }

    // Fallback: load via archesClient
    if (this.archesClient) {
      const resource = await this.archesClient.getResource(id);
      // Get the actual resourceinstanceid before merging (merge consumes the resource)
      // The registry stores by resourceinstanceid, not the lookup ID (which may be a slug)
      const resourceId = resource.resourceinstance.resourceinstanceid;
      // mergeFromResources CONSUMES the resource (transfers ownership to Rust),
      // so we must get a fresh copy from the registry after merging
      this.registry.mergeFromResources([resource], true, false);
      // Get fresh copy from registry using the actual resourceinstanceid
      const fresh = this.registry.getFull(resourceId);
      if (!fresh) {
        throw new Error(`Resource ${id} (${resourceId}) was merged but not found in registry`);
      }
      return fresh;
    }

    if (cached) {
      throw new Error(`Resource ${id} in registry but tiles not loaded.`);
    }
    throw new Error(`Resource ${id} not in registry and no archesClient available.`);
  }

  /**
   * Check if a resource exists in the registry.
   */
  contains(id: string): boolean {
    return this.registry.contains(id);
  }

  /**
   * Check if a full resource (with tiles) exists in the registry.
   */
  hasFull(id: string): boolean {
    return this.registry.hasFull(id);
  }
}

// Default instance - registry is lazily initialized when first accessed
const staticStore = new StaticStore();

export { staticStore, StaticStore };
