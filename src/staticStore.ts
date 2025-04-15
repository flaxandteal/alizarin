import { archesClient, ArchesClient, ArchesClientRemote } from "./client.ts";
import {
  StaticValue,
  StaticConcept,
  StaticTile,
  StaticGraph,
  StaticNode,
  StaticNodegroup,
  StaticResource,
  StaticResourceMetadata,
} from "./static-types";

// TODO: this does not currently cache, to avoid
//  memory leaks.
class StaticStore {
  archesClient: ArchesClient;
  cache: Map<string, StaticResource | StaticResourceMetadata>;
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
      }
      return resource;
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
        if (this.cacheMetadataOnly) {
          this.cache.set(
            resource.resourceinstance.resourceinstanceid,
            this.cacheMetadataOnly ? resource.resourceinstance : resource
          );
        }
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
}

const staticStore = new StaticStore(archesClient);

export { staticStore };
