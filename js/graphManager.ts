import { GraphResult, archesClient, ArchesClient, ArchesClientRemote } from './client';
import { staticStore, StaticStore } from './staticStore';
import { CardComponent, Widget } from './cards';
import {
  StaticTranslatableString,
  StaticCollection,
  StaticTile,
  StaticGraph,
  StaticNode,
  StaticNodegroup,
  StaticResource,
  StaticResourceSummary,
  StaticGraphMeta
} from "./static-types";
import { PseudoValue, PseudoUnavailable, wrapRustPseudo } from "./pseudos.ts";
import {
  createInstanceWrapperForResource,
  createInstanceWrapperForModel,
  createResourceModelWrapper,
  createWKRM,
  createStaticGraphMeta,
  createStaticResourceFromSummary,
  applyGraphMutations,
  getBackend,
  loadTilesFromResource,
  resourceTilesLoaded,
  safeStringify,
} from "./backend";
import { ResourceInstanceViewModel, viewContext, SemanticViewModel, NodeViewModel } from "./viewModels.ts";
import { GetMeta, IRIVM, IStringKeyedObject, IPseudo, IInstanceWrapper, IViewModel, IModelWrapperBackend, IWKRM, ResourceInstanceViewModelConstructor, PermissionValue } from "./interfaces";
import { nodeConfigManager } from "./nodeConfig.ts";
import { AttrPromise } from "./utils";

/**
 * Normalise a model identifier so a display name, slug, alias or PascalCased class
 * name all collapse to the same key: `"NPC"`, `"npc"`, `"Monster Type"` and
 * `"monster-type"` → `"npc"` / `"monstertype"`. Used by `graphManager.get()` so a
 * lookup is not restricted to the exact class name (fix for C7).
 */
export function normalizeModelName(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]/g, '');
}

// Import and re-export timing functions from dedicated module (avoids circular imports)
import { recordNativeTiming, printNativeTimings, clearNativeTimings, getNativeTimings, recordWasmTiming, printWasmTimings, clearWasmTimings, getWasmTimings } from './wasmTiming';
export { recordNativeTiming, printNativeTimings, clearNativeTimings, getNativeTimings, recordWasmTiming, printWasmTimings, clearWasmTimings, getWasmTimings };

// Re-export permission types for external use
export type { ConditionalPermission, PermissionValue } from "./interfaces";

class ConfigurationOptions {
  graphs: Array<string> | null | boolean = null;
  eagerLoadGraphs: boolean = false;
  defaultAllowAllNodegroups: boolean = false;
}

export class ResourceInstanceWrapper<RIVM extends IRIVM<RIVM>> implements IInstanceWrapper<RIVM> {
  wkri: RIVM;
  model: ResourceModelWrapper<RIVM>;
  wasmWrapper: any; // WASMResourceInstanceWrapper or NapiResourceInstanceWrapper
  resource?: StaticResource;

  // Local cache for wrapped pseudo values (replaces ValueList)
  private _pseudoCache: Map<string, any> = new Map();
  cache: {[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined;
  scopes?: string[];
  metadata?: {[key: string]: string};
  private tilesLoaded: boolean = false;
  private pruneTiles: boolean = true;

  constructor(
    wkri: RIVM,
    model: ResourceModelWrapper<RIVM>,
    resource: StaticResource | null | false, // False to disable dynamic resource-loading
    pruneTiles: boolean = true,
    lazy: boolean = false,
    assumeTilesComprehensiveForNodegroup: boolean = true
  ) {
    const constructorStart = performance.now();
    this.wkri = wkri;
    this.model = model;
    this.pruneTiles = pruneTiles;

    // Initialize wrapper for tile management (WASM or NAPI depending on backend)
    let t0 = performance.now();
    if (resource) {
      this.wasmWrapper = createInstanceWrapperForResource(resource, this.model.staticStore.registry);
      this.resource = resource;
      recordNativeTiming("createInstanceWrapperForResource", performance.now() - t0);
    } else {
      this.wasmWrapper = createInstanceWrapperForModel(model.wkrm.graphId, wkri.id);
      recordNativeTiming("createInstanceWrapperForModel", performance.now() - t0);
    }

    this._pseudoCache = new Map();
    this.cache = resource ? resource.__cache : undefined;
    this.scopes = resource ? resource.__scopes : undefined;
    this.metadata = resource ? resource.metadata : undefined;

    // Set lazy mode (NAPI instance wrapper manages tiles internally)
    if (typeof this.wasmWrapper.setLazy === 'function') {
      t0 = performance.now();
      this.wasmWrapper.setLazy(lazy);
      recordNativeTiming("setLazy", performance.now() - t0);
    }

    // Set up tile loader callback
    if (resource && typeof this.wasmWrapper.setTileLoader === 'function') {
      // Create a tile loader that loads from staticStore API
      const resourceId = resource.resourceinstance.resourceinstanceid;
      t0 = performance.now();
      this.wasmWrapper.setTileLoader((nodegroupId: string) => {
        // If nodegroupId is null/undefined, load all tiles
        // Otherwise load tiles for specific nodegroup
        const tiles = this.model.staticStore.loadTiles(resourceId, nodegroupId);
        return tiles;
      });
      recordNativeTiming("setTileLoader", performance.now() - t0);
    }

    // Load tiles into Rust if we have any - regardless of lazy mode,
    // if tiles are already present we should use them
    let tilesLoaded = false;
    if (typeof this.wasmWrapper.tilesLoaded === 'function') {
      t0 = performance.now();
      tilesLoaded = this.wasmWrapper.tilesLoaded();
      recordNativeTiming("tilesLoaded (constructor)", performance.now() - t0);
    }

    // Use loadTilesFromResource to avoid expensive tiles getter (which creates N WASM wrappers)
    if (!tilesLoaded && resource && resourceTilesLoaded(resource) && typeof this.wasmWrapper.loadTilesFromResource === 'function') {
      t0 = performance.now();
      loadTilesFromResource(this.wasmWrapper, resource, assumeTilesComprehensiveForNodegroup);
      recordNativeTiming("loadTilesFromResource", performance.now() - t0);
    }

    if (pruneTiles && resource) {
      t0 = performance.now();
      this.pruneResourceTiles();
      recordNativeTiming("pruneResourceTiles", performance.now() - t0);
    }
    recordNativeTiming("constructor total", performance.now() - constructorStart);
  }

  async ensureTilesLoaded(): Promise<void> {
    if (!this.wasmWrapper.tilesLoaded()) {
      // Use in-memory resource if available (avoids registry/disk lookup)
      if (this.resource && this.resource.tiles && this.resource.tiles.length > 0) {
        loadTilesFromResource(this.wasmWrapper, this.resource, true);
        return;
      }

      // If a tile loader callback is set, use ONLY that — no HTTP fallback.
      // This prevents silent degradation from tile source to archesClient.
      const tileLoader = typeof this.wasmWrapper.getTileLoader === 'function'
        ? this.wasmWrapper.getTileLoader()
        : null;
      if (tileLoader) {
        const tiles = await tileLoader(null);
        if (tiles && tiles.length > 0) {
          this.wasmWrapper.appendTiles(tiles);
        }
        return;
      }

      // No tile loader — fetch from staticStore registry via archesClient
      const resourceId = this.wasmWrapper.getResourceId();
      const fullResource = await this.model.staticStore.ensureFullResource(resourceId);
      if (fullResource && resourceTilesLoaded(fullResource)) {
        loadTilesFromResource(this.wasmWrapper, fullResource, true);
      }
    }
  }

  /**
   * Fetch one nodegroup's tiles (all of them when null) and merge them,
   * marking exactly that nodegroup loaded, even if it has none: so lazy
   * lookups fetch each nodegroup once and never read an unfetched one as
   * empty. Uses the tile loader when one is set, else the static store, on
   * either backend.
   */
  async fetchNodegroupTiles(nodegroupId: string | null): Promise<void> {
    const tileLoader = typeof this.wasmWrapper.getTileLoader === 'function'
      ? this.wasmWrapper.getTileLoader()
      : null;
    let tiles;
    if (tileLoader) {
      tiles = await tileLoader(nodegroupId);
    } else {
      const resourceId = this.wasmWrapper.getResourceId();
      if (!resourceId) {
        return;
      }
      tiles = await this.model.staticStore.loadTiles(resourceId, nodegroupId);
    }
    this.wasmWrapper.appendFetchedTiles(tiles ?? [], nodegroupId);
  }

  /**
   * Fetch every nodegroup a lazy lookup listed (`nodegroupsToFetch`,
   * `getMissingNodegroupsForChildren`, or a `TILES_NOT_LOADED:a,b` error):
   * from the compiled-in tile source first, where the backend has one, and the
   * rest through fetchNodegroupTiles. The same on either backend, as Python's
   * `fetch_nodegroups` is.
   */
  async fetchNodegroups(nodegroupIds: string[]): Promise<void> {
    if (nodegroupIds.length === 0) {
      return;
    }
    const remaining: string[] = this.wasmWrapper.fetchFromTileSource(nodegroupIds);
    for (const nodegroupId of remaining) {
      await this.fetchNodegroupTiles(nodegroupId);
    }
  }

  // =========================================================================
  // Editing: assignment records edits; $commit() applies them through core
  // =========================================================================

  /** Edits recorded since the last commit, as core's `applyEdits` takes them. */
  private _pendingEdits: any[] = [];
  /** What reads see until the commit: assigned values and edited lists. */
  private _pendingValues: Map<string, any> = new Map();

  private static _pendingKey(tileId: string | null | undefined, alias: string): string {
    return `${tileId ?? ''}/${alias}`;
  }

  /**
   * Record `value` for child `key` of semantic `node` (null: the root) within
   * `tile` (null at the root) as a pending edit. The checks that need only the
   * graph (an unknown or derived node, a semantic node, a multi-valued one, an
   * unloaded nodegroup) throw now; coercion happens at commit. As Python's
   * `ResourceInstanceWrapper.assign`.
   */
  assign(tile: StaticTile | null, node: StaticNode | null, key: string, value: any): void {
    node = node ?? this.model.getRootNode();
    const child = this.model.getChildNodes(node.nodeid).get(key);
    if (!child) {
      throw Error(`${node.alias} has no child ${key}`);
    }
    const nodegroupId = child.nodegroup_id;
    if (tile && tile.nodegroup_id === nodegroupId) {
      this.recordEdit({ op: 'set', tile: tile.tileid, alias: key, value });
    } else {
      const nodegroup = nodegroupId ? this.model.getNodegroupObjects().get(nodegroupId) : undefined;
      if (nodegroup?.cardinality === 'n') {
        throw Error(`${key} is multi-valued: push to its list, or set values on one of its items`);
      }
      this.recordEdit({ op: 'set', parent: tile?.tileid ?? null, alias: key, value });
    }
    this._pendingValues.set(ResourceInstanceWrapper._pendingKey(tile?.tileid, key), value);
  }

  /** The value `key` was given under `tileId` since the last commit, if any. */
  pendingValue(tileId: string | null | undefined, key: string): { value: any } | undefined {
    const pendingKey = ResourceInstanceWrapper._pendingKey(tileId, key);
    return this._pendingValues.has(pendingKey)
      ? { value: this._pendingValues.get(pendingKey) }
      : undefined;
  }

  /** Make reads of `key` under `tileId` see `value` (an edited list) until the commit. */
  setPendingValue(tileId: string | null | undefined, key: string, value: any): void {
    this._pendingValues.set(ResourceInstanceWrapper._pendingKey(tileId, key), value);
  }

  /**
   * Record one edit (`{op: "set" | "add" | "remove", ...}`), checking each
   * node it gives a value against the graph first. Returns its index, which
   * later edits can name as `{ref: index}`.
   */
  recordEdit(op: any): number {
    const aliases = op.op === 'set' ? [op.alias] : op.op === 'add' ? Object.keys(op.values ?? {}) : [];
    for (const alias of aliases) {
      this.wasmWrapper.checkEditable(alias);
    }
    this._pendingEdits.push(op);
    return this._pendingEdits.length - 1;
  }

  /** Whether there are edits not yet committed. */
  hasPendingEdits(): boolean {
    return this._pendingEdits.length > 0;
  }

  /**
   * Apply every pending edit through core, all or nothing: values coerced,
   * stored Derive functions rerun for what changed. Returns the tiles to write
   * and the ids to delete; persisting them is the caller's. Pending edits are
   * dropped either way, so a failure leaves the resource as it was.
   */
  commitEdits(): { upserted: StaticTile[]; removed: string[] } {
    const ops = this._pendingEdits;
    this.discardEdits();
    if (ops.length === 0) {
      return { upserted: [], removed: [] };
    }
    return this.wasmWrapper.applyEdits(ops);
  }

  /** Drop every pending edit; reads see the resource as committed. */
  discardEdits(): void {
    this._pendingEdits = [];
    this._pendingValues = new Map();
    this._pseudoCache = new Map();
  }

  pruneResourceTiles(): undefined {
    this.wasmWrapper.pruneResourceTiles();
  }

  // Direct Rust delegation methods (replaces ValueList)

  /**
   * Retrieve pseudo value by alias - queries Rust's pseudo_cache
   * Replaces ValueList.retrieve
   */
  async retrievePseudo(key: string, dflt: any = null, raiseError: boolean = false): Promise<Array<IPseudo> | null> {
    // Check local cache first (for wrapped values)
    let result: any = this._pseudoCache.get(key);
    if (result instanceof Promise) {
      result = await result;
    }
    if (Array.isArray(result) && result.length > 0) {
      return result;
    }

    // Query Rust's pseudo_cache
    const rustValue = this.wasmWrapper.getCachedPseudo(key);

    if (rustValue) {
      // Wrap the Rust value and cache locally
      const wrappedValue = wrapRustPseudo(rustValue, this.wkri, this.model);
      // Store as array for consistency
      const wrappedArray = [wrappedValue];
      this._pseudoCache.set(key, wrappedArray);
      return wrappedArray;
    }

    // Not found in cache
    if (raiseError) {
      throw Error(`Unset key ${key}`);
    }
    return dflt;
  }

  async hasPseudo(key: string): Promise<boolean> {
    const value = await this.retrievePseudo(key, null, false);
    return value !== null && value !== undefined;
  }

  setPseudo(key: string, value: any): void {
    this._pseudoCache.set(key, value);
  }

  async setDefaultPseudo(key: string, value: any): Promise<any> {
    const existingValue = await this.retrievePseudo(key, null, false);
    if (existingValue !== null) {
      return existingValue;
    }
    // Value not found, set the default
    this._pseudoCache.set(key, value);
    return value;
  }

  async loadNodes(aliases: Array<string>): Promise<void> {
    for (const key of aliases) {
      await this.retrievePseudo(key);
    }
  }

  getName(update: boolean = false): string {
    // Always ensure descriptors are computed (getDescriptors(false) computes if cache is empty)
    this.getDescriptors(update);
    return this.wasmWrapper.getName();
  }

  getDescriptors(update: boolean = false) {
    return this.wasmWrapper.getDescriptors(update);
  }

  getValuesAtPath(path: string, filterTileId?: string) {
    const rustValue = this.wasmWrapper.getValuesAtPath(path, filterTileId);
    return wrapRustPseudo(rustValue, this.wkri, this.model);
  }

  /**
   * Resolve a dot-separated path and return ViewModels, loading tiles lazily if needed.
   *
   * Unlike getValuesAtPath (sync, requires tiles pre-loaded), this is async and
   * will load only the target nodegroup's tiles on demand via staticStore.
   *
   * Returns a single IViewModel for single-cardinality nodes, or a PseudoList
   * (iterable of AttrPromise<IViewModel>) for multi-cardinality nodes.
   */
  async getViewModelsAtPath(path: string, filterTileId?: string): Promise<IViewModel | IPseudo | null> {
    // Step 1: Resolve path to nodegroup (no tiles needed — only uses graph model)
    const pathInfo = this.wasmWrapper.resolvePath(path);
    const { nodegroupId } = pathInfo;

    // Step 2: Ensure tiles for target nodegroup are loaded, with the sources
    // of an on-load derived target
    if (!this.wasmWrapper.isNodegroupLoaded(nodegroupId)) {
      if (!this.wasmWrapper.getResourceId()) {
        return null;
      }
      const toFetch: string[] = this.wasmWrapper.nodegroupsToFetch(nodegroupId);
      await this.fetchNodegroups(toFetch.length ? toFetch : [nodegroupId]);
    }

    // Step 3: getValuesAtPath will find tiles for this nodegroup
    const pseudo = this.getValuesAtPath(path, filterTileId);
    if (!pseudo) return null;

    // Step 4: Resolve to ViewModels
    return pseudo.getValue();
  }

  addPseudo(childNode: StaticNode, tile: StaticTile | null): IPseudo {
    const key = childNode.alias;
    if (!key) {
      throw Error(`Cannot add a pseudo node with no alias ${childNode.nodeid}`);
    }

    // Phase 4e: Call Rust to create the pseudo value, then wrap in TS class
    try {
      // Calculate permissions
      const isPermitted = this.model.isNodegroupPermitted(
        childNode.nodegroup_id || '',
        tile
      );

      // Call Rust makePseudoValue
      const rustValue = this.wasmWrapper.makePseudoValue(
        key,
        tile?.tileid || null,
        isPermitted,
        false // is_single
      );

      // Handle unavailable case (Rust returns null for unpermitted)
      if (rustValue === null || rustValue === undefined) {
        const child = new PseudoUnavailable(childNode);
        this.setDefaultPseudo(key, []).then((val: Array<any>) => val.push(child));
        return child;
      }

      // Wrap the Rust value in TS PseudoValue/PseudoList
      const child = wrapRustPseudo(rustValue, this.wkri, this.model);

      // Phase 4k: Don't use setDefault - it triggers retrieve which may already have
      // created values from Rust. Instead, directly get/set the value.
      this.setDefaultPseudo(key, []).then((val: Array<any>) => val.push(child));
      return child;

      // if (existingValue === undefined || existingValue === false) {
      //   // No existing value - create a new array
      //   valueList.values.set(key, [child]);
      // } else if (Array.isArray(existingValue)) {
      //   // Already an array (PseudoList or plain) - push to it
      //   // But check if child is already present (by checking tile ID to avoid duplicates)
      //   const childTileId = child.tile?.tileid;
      //   const alreadyPresent = childTileId && existingValue.some((v: any) => v.tile?.tileid === childTileId);
      //   if (!alreadyPresent) {
      //     existingValue.push(child);
      //   }
      // } else if (existingValue instanceof Promise) {
      //   // Value is still loading - wait and then push
      //   existingValue.then((val: any) => {
      //     if (Array.isArray(val)) {
      //       const childTileId = child.tile?.tileid;
      //       const alreadyPresent = childTileId && val.some((v: any) => v.tile?.tileid === childTileId);
      //       if (!alreadyPresent) {
      //         val.push(child);
      //       }
      //     }
      //   });
      // }
      // return child;
    } catch (e) {
      console.error("Rust makePseudoValue failed:", e);
      throw new Error(`Rust makePseudoValue failed: ${e}. This should not happen - check Rust implementation.`);
    }
  }

  allEntries(): MapIterator<[string, Array<IPseudo> | false | null]> {
    return this._pseudoCache.entries()
  }

  async keys() {
    return (await this.getRootViewModel()).keys();
  }

  async values() {
    return (await this.getRootViewModel()).values();
  }

  async entries() {
    return (await this.getRootViewModel()).entries();
  }

  async getRootViewModel(): Promise<SemanticViewModel> {
    const root = await this.getRoot();
    let value: IViewModel | null = null;
    if (root) {
      const rootValue = await root.getValue();
      if (!Array.isArray(rootValue)) {
        value = rootValue;
      }
    }
    if (!value || !(value instanceof SemanticViewModel)) {
      throw Error(`Tried to get root on ${this.model.wkrm.modelClassName}, which has no root`);
    }
    return value;
  }

  getOrmAttribute(key: string): AttrPromise<IViewModel> {
    let promise: Promise<void>;
    if (this.resource === null) {
      //promise = this.model.findStatic(this.wkri.id).then(resource => {
      //  this.resource = resource;
      //}).then(() => this.populate(true));
    } else {
      promise = new Promise((resolve) => { resolve(); });
    }

    // TODO remapping
    return new AttrPromise(resolve => {
      return promise.then(() => this.getRootViewModel()).then(root => resolve(root[key]));
    });
  }

  async getRoot(): Promise<IPseudo | undefined> {
    const node = this.model.getRootNode();
    if (!node) {
      return undefined;
    }

    const alias = node.alias;
    if (!(typeof alias == 'string')) {
      throw Error(`Alias missing for node ${node.nodeid}`);
    }

    // Try to get root pseudo directly from cache (returns WasmPseudoValue, not list)
    const rootPseudo = this.wasmWrapper.getRootPseudo();
    if (rootPseudo) {
      // Wrap as PseudoValue directly (not via wrapRustPseudo which creates lists)
      return PseudoValue.fromWasm(rootPseudo, this.wkri);
    }

    // Not in cache - create a new pseudo value for empty resource
    const rustValue = this.wasmWrapper.makePseudoValue(
      alias,
      null,  // tile_id
      true,  // is_permitted
      false  // is_single
    );
    const value = wrapRustPseudo(rustValue, this.wkri, this.model);
    return value;
  }

  setOrmAttribute(key: string, value: any) {
    // TODO remapping
    return this.getRootViewModel().then((root) => {
      if (root) {
        root[key] = value;
      } else {
        throw Error(`Tried to set ${key} on ${self}, which has no root`);
      }
    });
  }

  /**
   * Populate all nodegroups for a resource
   * Uses Rust implementation via WASM - Rust caches all values internally
   */
  async populate(lazy: boolean): Promise<void> {
    const populateStart = performance.now();
    const nodegroupObjs = this.model.getNodegroupObjects();
    const rootNode = this.model.getRootNode();

    if (rootNode.alias === null) {
      throw Error("Cannot populate a model with no proper root node");
    }

    try {
      if (!lazy) {
        // Ensure tiles are loaded in Rust if we need them for non-lazy population.
        // Use tilesLoaded() to check Rust state, not this.resource.tiles (JS state).
        let t0 = performance.now();
        const loaded = this.wasmWrapper.tilesLoaded();
        recordNativeTiming("tilesLoaded (populate)", performance.now() - t0);
        if (!loaded) {
          t0 = performance.now();
          await this.ensureTilesLoaded();
          recordNativeTiming("ensureTilesLoaded", performance.now() - t0);
        }
      }

      // Get all nodegroup IDs
      const nodegroupIds = [...nodegroupObjs.keys()];

      // Call Rust implementation - Rust stores values in its pseudo_cache
      // Permissions are read directly from the model in Rust (set via setPermittedNodegroups)
      const t0 = performance.now();
      const result = this.wasmWrapper.populate(
        lazy,
        nodegroupIds,
        rootNode.alias
      );
      recordNativeTiming("populate", performance.now() - t0);

      // Initialize allNodegroups from Rust result
      const allNodegroups: Map<string, boolean> = new Map();
      const updatedNodegroups = result.allNodegroupsMap;
      if (updatedNodegroups instanceof Map) {
        for (const [key, value] of updatedNodegroups.entries()) {
          if (typeof value === 'boolean') {
            allNodegroups.set(key, value);
          }
        }
      } else {
        for (const [key, value] of Object.entries(updatedNodegroups)) {
          if (typeof value === 'boolean') {
            allNodegroups.set(key, value as boolean);
          }
        }
      }

      // Clear local cache - Rust is now the source of truth
      this._pseudoCache = new Map();

      recordNativeTiming("populate total", performance.now() - populateStart);
    } catch (error) {
      const resourceId = this.wasmWrapper.getResourceId?.() || 'unknown';
      console.error(`[populate] Rust implementation failed for resource ${resourceId}:`, error);
      throw new Error(`populate failed for resource ${resourceId}: ${error}`);
    }
  }

  async getValueCache(build: boolean = true, getMeta: GetMeta = undefined): Promise<{[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined> {
    if (build) {
      this.cache = await this.buildValueCache(getMeta);
    }
    return this.cache;
  }

  async buildValueCache(getMeta: GetMeta): Promise<{[tileId: string]: {[nodeId: string]: IStringKeyedObject}}> {
    const cacheByTile: {[tileId: string]: {[nodeId: string]: IStringKeyedObject}} = {};
    for (let pseudos of this._pseudoCache.values()) {
      pseudos = await pseudos;
      if (pseudos) {
        await Promise.all(pseudos.map(async (pseudo: IPseudo) => {
          const value = await pseudo.getValue();
          // We do not try to cache pseudolists
          if (pseudo.tile && value && (!Array.isArray(pseudo))) {
            // @ts-expect-error this can be IViewModel[] while using a pseudolist
            const cacheJson = await value.__forJsonCache(getMeta); // caching JSON
            if (cacheJson) {
              const tileId = pseudo.tile.ensureId();
              const nodeId = pseudo.node.nodeid;
              if (!(tileId in cacheByTile)) {
                cacheByTile[tileId] = {};
              }
              if (!(nodeId in cacheByTile[tileId])) {
                cacheByTile[tileId][nodeId] = {};
              }
              cacheByTile[tileId][nodeId] = cacheJson;
            }
          }
        }));
      }
    }
    return cacheByTile;
  }

  /**
   * Release WASM memory held by this wrapper.
   * Call this after extracting data from a resource to free memory when processing many resources.
   * This clears tiles, pseudo cache, and other internal state.
   */
  release(): void {
    // Clear JS-side caches
    this._pseudoCache.clear();
    this.cache = undefined;
    this.scopes = undefined;
    this.metadata = undefined;

    // Release WASM-side memory
    this.wasmWrapper.release();

    // Clear resource reference to allow GC
    this.resource = undefined;
  }
}

/** A core graph mutation, e.g. `{ AddNode: {...} }` (core's `GraphMutation`). */
type GraphMutation = { [kind: string]: unknown };

/** Per-node flags, as core's `NodeOptions` takes them. */
type NodeOptions = {
  exportable?: boolean,
  fieldname?: string,
  hascustomalias?: boolean;
  is_collector?: boolean;
  isrequired?: boolean;
  issearchable?: boolean;
  istopnode?: boolean;
  sortorder?: number;
};

/** A core mutation of `kind` (`AddNode`, `AddCard`, …) with its params. */
function mutation(kind: string, params: object): GraphMutation {
  return { [kind]: params };
}

/** A plain or translatable string, as core reads a translatable string. */
function translatable(value: string | StaticTranslatableString): unknown {
  return typeof value === 'string' ? value : (value as any).toJSON();
}

/**
 * Fluent builder for graph schema changes. It records core graph mutations
 * and `apply()` runs them in core, so the same mutations give the same graph,
 * ids included, on WASM, NAPI and Python (whose GraphMutator is the same).
 * Cards and widgets for new nodes are created by core when `autocreateCard`.
 */
class GraphMutator {
  baseGraph: StaticGraph;
  mutations: GraphMutation[];

  autocreateCard: boolean;

  constructor(baseGraph: StaticGraph, options: {
    autocreateCard?: boolean
  } = {}) {
    this.baseGraph = baseGraph;
    this.mutations = [];
    this.autocreateCard = options.autocreateCard === undefined || options.autocreateCard;
  }

  addSemanticNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options: NodeOptions = {}, config?: {[key: string]: any}) {
    return this._addGenericNode(parentAlias, alias, name, cardinality, "semantic", ontologyClass, parentProperty, description, options, config);
  }

  addConceptNode(parentAlias: string | null, alias: string, name: string, collection: StaticCollection, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options: NodeOptions & {
    is_list?: boolean,
  } = {}, config?: {[key: string]: any}) {
    config = config || {};
    if (collection?.id) {
      config['rdmCollection'] = collection.id
    }
    const { is_list, ...nodeOptions } = options;
    return this._addGenericNode(parentAlias, alias, name, cardinality, is_list ? "concept-list" : "concept", ontologyClass, parentProperty, description, nodeOptions, config);
  }

  addStringNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options: NodeOptions = {}, config?: {[key: string]: any}) {
    return this._addGenericNode(parentAlias, alias, name, cardinality, "string", ontologyClass, parentProperty, description, options, config);
  }

  addCard(nodegroup: string | StaticNodegroup, name: string | StaticTranslatableString, component?: CardComponent, options: {
    active?: boolean,
    cssclass?: string | null,
    helpenabled?: boolean,
    helptext?: string | null | StaticTranslatableString,
    helptitle?: string | null | StaticTranslatableString,
    instructions?: string | null | StaticTranslatableString,
    is_editable?: boolean,
    description?: string | null | StaticTranslatableString,
    sortorder?: number | null,
    visible?: boolean
  } = {}, config?: {[key: string]: any}) {
    const optional = (value?: string | null | StaticTranslatableString) => value ? translatable(value) : null;
    this.mutations.push(mutation('AddCard', {
      nodegroup_id: typeof nodegroup === 'string' ? nodegroup : nodegroup.nodegroupid,
      name: translatable(name),
      component_id: component?.id ?? null,
      options: {
        active: options.active ?? null,
        cssclass: options.cssclass ?? null,
        helpenabled: options.helpenabled ?? null,
        helptext: optional(options.helptext),
        helptitle: optional(options.helptitle),
        instructions: optional(options.instructions),
        is_editable: options.is_editable ?? null,
        description: optional(options.description),
        sortorder: options.sortorder ?? null,
        visible: options.visible ?? null,
      },
      config: config ?? null,
    }));
    return this;
  }

  _addGenericNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', datatype: string, ontologyClass: string | string[], parentProperty: string, description?: string, options: NodeOptions = {}, config?: {[key: string]: any}) {
    this.mutations.push(mutation('AddNode', {
      parent_alias: parentAlias,
      alias,
      name,
      cardinality,
      datatype,
      ontology_class: ontologyClass,
      parent_property: parentProperty,
      description: description ?? null,
      config: config ?? null,
      options,
    }));
    return this;
  }

  addWidgetToCard(
    nodeId: string,
    widget: Widget,
    name: string,
    config: {[key: string]: any},
    options: {
      sortorder?: number | null,
      visible?: boolean
    } = {}
  ): GraphMutator {
    this.mutations.push(mutation('AddWidgetToCard', {
      node_id: nodeId,
      widget_id: widget.id,
      label: name,
      config,
      sortorder: options.sortorder ?? null,
      visible: options.visible ?? null,
    }));
    return this;
  }

  /** A new graph with the mutations applied, by core; the base graph is untouched. */
  apply(): StaticGraph {
    return applyGraphMutations(this.baseGraph, JSON.stringify({
      mutations: this.mutations,
      options: {
        autocreate_card: this.autocreateCard,
        autocreate_widget: this.autocreateCard,
      },
    }));
  }
}

/**
 * ResourceModelWrapper — graph schema access with backend delegation.
 *
 * Stores a backend wrapper (_backend) that is either a WASMResourceModelWrapper
 * or NapiResourceModelWrapper, and delegates node/edge/nodegroup/permission
 * operations to it. WKRM metadata is stored locally.
 */
class ResourceModelWrapper<RIVM extends IRIVM<RIVM>> {
  private _backend: IModelWrapperBackend;
  wkrm: IWKRM;
  viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>;
  // Supports both boolean and conditional permission rules
  permittedNodegroups?: Map<string, PermissionValue>;
  pruneTiles: boolean = true;
  /** The resource store this model reads from. Injectable so independent layer
   *  stacks each get their own store; defaults to the shared `staticStore`. */
  staticStore: StaticStore = staticStore;

  // Cached copies of backend data (to avoid repeated cross-boundary calls)
  private _nodes: Map<string, StaticNode> | null = null;
  private _nodesByAlias: Map<string, StaticNode> | null = null;
  private _edges: Map<string, string[]> | null = null;
  private _nodegroups: Map<string, StaticNodegroup> | null = null;
  // The backend builds a graph (and its edge list) afresh on every read
  private _graph: StaticGraph | null = null;
  private _edgeObjects: any[] | null = null;

  constructor(wkrm: IWKRM, graph: StaticGraph, viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>, defaultAllow: boolean = false, store: StaticStore = staticStore) {
    this.wkrm = wkrm;
    this._backend = createResourceModelWrapper(wkrm, graph, defaultAllow);
    this.pruneTiles = !defaultAllow;
    this.viewModelClass = viewModelClass;
    this.staticStore = store;
  }

  // =========================================================================
  // Backend-delegated properties
  // =========================================================================

  get graph(): StaticGraph {
    if (!this._graph) {
      this._graph = this._backend.graph;
    }
    return this._graph!;
  }

  set graph(g: StaticGraph) {
    this._backend.graph = typeof g === 'string' ? g : safeStringify(g);
    this._clearCaches();
  }

  get nodes(): Map<string, StaticNode> | null {
    if (!this._nodes) {
      const raw = this._backend.nodes;
      if (raw) this._nodes = raw instanceof Map ? raw : new Map(Object.entries(raw));
    }
    return this._nodes;
  }

  get nodesByAlias(): Map<string, StaticNode> | null {
    if (!this._nodesByAlias) {
      const raw = this._backend.nodesByAlias;
      if (raw) this._nodesByAlias = raw instanceof Map ? raw : new Map(Object.entries(raw));
    }
    return this._nodesByAlias;
  }

  get edges(): Map<string, string[]> | null {
    if (!this._edges) {
      const raw = this._backend.edges;
      if (raw) this._edges = raw instanceof Map ? raw : new Map(Object.entries(raw));
    }
    return this._edges;
  }

  get nodegroups(): Map<string, StaticNodegroup> | null {
    if (!this._nodegroups) {
      const raw = this._backend.nodegroups;
      if (raw) this._nodegroups = raw instanceof Map ? raw : new Map(Object.entries(raw));
    }
    return this._nodegroups;
  }

  private _clearCaches() {
    this._nodes = null;
    this._nodesByAlias = null;
    this._edges = null;
    this._nodegroups = null;
    this._graph = null;
    this._edgeObjects = null;
  }

  // =========================================================================
  // Delegated methods
  // =========================================================================

  getRootNode(): StaticNode {
    return this._backend.getRootNode();
  }

  getChildNodes(nodeId: string): Map<string, StaticNode> {
    const raw = this._backend.getChildNodes(nodeId);
    return raw instanceof Map ? raw : new Map(Object.entries(raw));
  }

  getChildNodeAliases(nodeId: string): string[] {
    return this._backend.getChildNodeAliases(nodeId);
  }

  getGraphId(): string {
    return this._backend.getGraphId?.() ?? this.wkrm.graphId;
  }

  getNodeObjectFromAlias(alias: string): StaticNode {
    return this._backend.getNodeObjectFromAlias(alias);
  }

  getNodeObjectFromId(id: string): StaticNode {
    return this._backend.getNodeObjectFromId(id);
  }

  getNodeIdFromAlias(alias: string): string {
    return this._backend.getNodeIdFromAlias(alias);
  }

  getNodegroupIds(): string[] {
    return this._backend.getNodegroupIds();
  }

  getNodegroupName(nodegroupId: string): string {
    return this._backend.getNodegroupName(nodegroupId);
  }

  // PseudoNode creation — only available in WASM mode (used for NodeViewModel)
  createPseudoNode(alias?: string): any {
    if (typeof this._backend.createPseudoNode === 'function') {
      return this._backend.createPseudoNode(alias);
    }
    throw new Error('createPseudoNode not available in current backend');
  }

  createPseudoNodeChild(childNode: string, parent: any): any {
    if (typeof this._backend.createPseudoNodeChild === 'function') {
      return this._backend.createPseudoNodeChild(childNode, parent);
    }
    throw new Error('createPseudoNodeChild not available in current backend');
  }

  createPseudoValue(alias: string | undefined, tile: any, parent: any): any {
    if (typeof this._backend.createPseudoValue === 'function') {
      return this._backend.createPseudoValue(alias, tile, parent);
    }
    throw new Error('createPseudoValue not available in current backend');
  }

  // =========================================================================
  // Methods with TS-level logic
  // =========================================================================

  getRoot(): NodeViewModel {
    const node = this.getRootNode();
    const pseudoNode = this.createPseudoNode(node.alias);
    return new NodeViewModel(pseudoNode, this);
  }

  buildNodes() {
    // Core keeps every backend's model built, so there is nothing to build:
    // drop the TS-side caches and read them afresh from the backend.
    this._clearCaches();
  }

  getNodeObjects(): Map<string, StaticNode> {
    if (!this.nodes) {
      this.buildNodes();
    }
    if (!this.nodes) {
      throw Error("Could not build nodes");
    }
    return this.nodes;
  }

  getNodeObjectsByAlias(): Map<string, StaticNode> {
    if (!this.nodesByAlias) {
      this.buildNodes();
    }
    if (!this.nodesByAlias) {
      throw Error("Could not build nodes");
    }
    return this.nodesByAlias;
  }

  getEdges(): Map<string, string[]> {
    if (!this.edges) {
      this.buildNodes();
    }
    if (!this.edges) {
      throw Error("Could not build edges");
    }
    return this.edges;
  }

  getNodegroupObjects(): Map<string, StaticNodegroup> {
    if (!this.nodegroups) {
      this.buildNodes();
    }
    if (!this.nodegroups) {
      throw Error("Could not build nodegroups");
    }
    return this.nodegroups;
  }

  // TODO: Switch to getBranches
  getBranchPublicationIds(accessible?: boolean): string[] {
    const accessibleOnly = accessible || false;
    const nodes = [...this.getNodeObjects().values()];
    return [...nodes.reduce(
      (acc: Set<string>, node: StaticNode): Set<string> => {
        if (node.sourcebranchpublication_id) {
          if (accessibleOnly) {
            if (this.isNodegroupPermitted(node.nodegroup_id || '', null)) {
              acc.add(node.sourcebranchpublication_id);
            }
          } else {
            acc.add(node.sourcebranchpublication_id);
          }
        }
        return acc;
      }, new Set()
    )];
  }

  /** The graph's edges, cached (unlike `graph.edges`, built on every read). */
  getEdgeObjects(): any[] {
    if (!this._edgeObjects) {
      this._edgeObjects = [...this.graph.edges];
    }
    return this._edgeObjects;
  }

  getCollections(accessible?: boolean): string[] {
    const accessibleOnly = accessible || false;
    const nodes = [...this.getNodeObjects().values()];
    return [...nodes.reduce(
      (acc: Set<string>, node: StaticNode): Set<string> => {
        if (['concept', 'concept-list'].includes(node.datatype) && node.config?.rdmCollection) {
          if (accessibleOnly) {
            if (this.isNodegroupPermitted(node.nodegroup_id || '', null)) {
              acc.add(node.config.rdmCollection);
            }
          } else {
            acc.add(node.config.rdmCollection);
          }
        }
        return acc;
      }, new Set()
    )];
  }

  pruneGraph(keepFunctions?: string[]): undefined {
    // Delegate to backend (WASM or NAPI) for graph filtering
    this._backend.pruneGraph(keepFunctions);
    this._clearCaches();
  }

  getPruneTiles(pruneTiles?: boolean) {
    if (pruneTiles === undefined) {
      pruneTiles = this.pruneTiles;
    }
    return pruneTiles;
  }

  async all(params: { limit?: number; lazy?: boolean; pruneTiles?: boolean } | undefined = undefined): Promise<Array<RIVM>> {
    const paramObj = params || { limit: undefined, lazy: undefined, pruneTiles: this.getPruneTiles(params?.pruneTiles) };
    const promises = [];
    for await (const resource of this.iterAll(paramObj)) {
      promises.push(resource);
    }
    return Promise.all(promises);
  }

  async* resourceGenerator(staticResources: AsyncIterable<StaticResource, RIVM, unknown>, lazy: boolean=false, pruneTiles?: boolean) {
    for await (const staticResource of staticResources) {
      yield this.fromStaticResource(staticResource, lazy, pruneTiles);
    }
  }

  async* iterAll(params: { limit?: number; lazy?: boolean; pruneTiles?: boolean }): AsyncGenerator<RIVM> {
    yield* this.resourceGenerator(this.staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy, params.pruneTiles);
  }

  // New summary-based methods for performance optimization
  async* summaryGenerator(staticSummaries: AsyncIterable<StaticResourceSummary>, lazy: boolean = true): AsyncGenerator<RIVM> {
    for await (const summary of staticSummaries) {
      // Create lightweight resource from summary
      const summaryResource = createStaticResourceFromSummary(summary);
      yield this.fromStaticResource(summaryResource, lazy, false); // lazy=true, pruneTiles=false
    }
  }

  async* iterAllSummaries(params: { limit?: number }): AsyncGenerator<RIVM> {
    yield* this.summaryGenerator(this.staticStore.loadAllSummaries(this.wkrm.graphId, params.limit), true);
  }

  async allSummaries(params: { limit?: number } | undefined = undefined): Promise<Array<RIVM>> {
    const paramObj = params || { limit: undefined };
    const promises = [];
    for await (const resource of this.iterAllSummaries(paramObj)) {
      promises.push(resource);
    }
    return Promise.all(promises);
  }

  async loadFullResource(id: string): Promise<RIVM> {
    // Check if we have full resource or just summary, load full resource on-demand
    const fullResource = await this.staticStore.ensureFullResource(id);
    return this.fromStaticResource(fullResource, false, true); // non-lazy, prune tiles
  }

  async findStatic(id: string): Promise<StaticResource> {
    return await this.staticStore.loadOne(id);
  }

  async find(id: string, lazy: boolean = true, pruneTiles?: boolean): Promise<RIVM> {
    const rivm = await this.findStatic(id);
    const x = this.fromStaticResource(rivm, lazy, pruneTiles);
    return x;
  }

  setDefaultAllowAllNodegroups(defaultAllow: boolean) {
    this.pruneTiles = !defaultAllow;
    if (typeof this._backend.setDefaultAllowAllNodegroups === 'function') {
      this._backend.setDefaultAllowAllNodegroups(defaultAllow);
    }
  }

  // Supports both boolean and conditional permission rules
  setPermittedNodegroups(permissions: Map<string, PermissionValue>) {
    const nodegroups = this.getNodegroupObjects();
    const nodes = this.getNodeObjectsByAlias();

    this.permittedNodegroups = new Map([...permissions].map(([key, value]): [string, PermissionValue] => {
      const k = key ?? '';  // Convert null/undefined to empty string

      // Resolve key to nodegroup ID if it's an alias
      let resolvedKey: string;
      if (nodegroups.has(k) || k === '') {
        resolvedKey = k;  // Use normalized key (not original)
      } else {
        const node = nodes.get(k);
        if (node) {
          // The nodeid is the nodegroup ID of the children, but may not be the nodegroup ID of
          // the semantic node itself.
          resolvedKey = node.nodeid ?? '';  // Ensure nodeid is not null
        } else {
          throw Error(`Could not find ${key} in nodegroups for permissions`);
        }
      }

      return [resolvedKey, value];
    }));

    // Propagate to backend for pruneGraph to work correctly
    // WASM backend accepts Map (js_sys::Map); NAPI backend accepts plain object (serde)
    const permsForBackend = getBackend() === 'napi'
      ? Object.fromEntries(this.permittedNodegroups)
      : this.permittedNodegroups;
    this._backend.setPermittedNodegroups(permsForBackend);
  }

  // Defaults to visible, which helps reduce the risk of false sense of security
  // from front-end filtering masking the presence of data transferred to it.
  getPermittedNodegroups(): Map<string, PermissionValue> {
    if (!this.permittedNodegroups) {
      // Auto-initializing with all-true permissions - this will overwrite any WASM-side permissions
      const permissions: Map<string, PermissionValue> = new Map([...this.getNodegroupObjects()].map(
        ([k, _]: [k: string, _: StaticNodegroup]) => [k ?? '', true]  // Ensure key is not null
      ));
      permissions.set("", true); // Have to have access to root node.
      this.setPermittedNodegroups(permissions);
    }
    const permittedNodegroups = this.permittedNodegroups;
    if (permittedNodegroups === undefined) {
      throw Error("Could not set permitted nodegroups");
    }
    // TODO allow reducing
    return permittedNodegroups;
  }

  // Check if a nodegroup is permitted (for JS-side checks)
  // Note: Conditional rules mean the nodegroup is permitted, but individual tiles may be filtered in Rust
  isNodegroupPermitted(nodegroupId: string, _tile: StaticTile | null): boolean {
    const permitted = this.getPermittedNodegroups().get(nodegroupId ?? '');
    if (permitted === undefined || permitted === false) {
      return false;
    }
    if (permitted === true) {
      return true;
    }
    // Conditional permission - nodegroup is permitted, tiles are filtered in Rust
    if (typeof permitted === 'object' && 'path' in permitted && 'allowed' in permitted) {
      return true;  // Nodegroup permitted; tile filtering happens in Rust
    }
    throw Error(`Ambiguous permission state: ${JSON.stringify(permitted)} for nodegroup ${nodegroupId}`);
  }

  makeInstance(id: string, resource: StaticResource | null, pruneTiles?: boolean, lazy: boolean = false): RIVM {
    pruneTiles = this.getPruneTiles(pruneTiles);
    if (!this.viewModelClass) {
      throw Error(`Cannot instantiate without a viewModelClass in ${this.wkrm.modelClassName}`);
    }
    // TODO: This line needs fixed.
    const instance: RIVM = new this.viewModelClass(
      id,
      this.viewModelClass.prototype.__,
      (rivm: RIVM) =>
        new ResourceInstanceWrapper(rivm, this, resource, pruneTiles, lazy),
      null
    );
    return instance;
  }

  fromStaticResource(
    resource: StaticResource,
    lazy: boolean = false,
    pruneTiles?: boolean
  ): Promise<RIVM> {
    const start = performance.now();
    const wkri: RIVM = this.makeInstance(
      resource.resourceinstance.resourceinstanceid,
      resource,
      pruneTiles,
      lazy
    );
    recordNativeTiming("makeInstance", performance.now() - start);

    if (!wkri.$) {
      throw Error("Could not load resource from static definition");
    }

    const pop = wkri.$.populate(lazy).then(() => {
      recordNativeTiming("fromStaticResource total", performance.now() - start);
      return wkri;
    });
    return pop;
  }

  asTree(): {[key: string]: any} {
    const root = this.getRootNode();
    const nodegroups = this.getNodegroupObjects();
    const addChildren = (node: StaticNode) => {
      const branch: {[key: string]: any} = {};
      const children = this.getChildNodes(node.nodeid);
      if (!children.size) {
        return false;
      }
      for (const child of children.values()) {
        const nodegroup = nodegroups.get(child.nodegroup_id || '');
        const multiple = (
          child.nodegroup_id &&
          child.is_collector &&
          nodegroup &&
          nodegroup.cardinality == 'n' &&
          node.nodegroup_id !== child.nodegroup_id
        ) || child.datatype.endsWith('-list');
        const childBranch = addChildren(child);
        const alias = child.alias || '';
        if (childBranch === false) {
          branch[alias] = child.datatype;
        } else {
          branch[alias] = childBranch;
          if (child.datatype !== 'semantic') {
             branch[alias]['_'] = child.datatype;
          }
        }
        if (multiple) {
          branch[alias] = [branch[alias]];
        }
      }
      return branch;
    }
    return addChildren(root) || {};
  }
}

function makeResourceModelWrapper<T extends IRIVM<T>>(
  viewModelClass: ResourceInstanceViewModelConstructor<T> | undefined,
  wkrm: IWKRM,
  graph: StaticGraph,
  defaultAllow: boolean,
  store: StaticStore = staticStore
): ResourceInstanceViewModelConstructor<T> {
  let vmc: ResourceInstanceViewModelConstructor<T>;
  if (!viewModelClass) {
    // @ts-expect-error It may be possible to correct this, but TS does not know that
    // the dynamically-defined class meets the IRIVM interface.
    const viewModelClassObj: {[name: string]: ResourceInstanceViewModelConstructor<T>} = {
      [wkrm.modelClassName]: class extends ResourceInstanceViewModel<T> {
        static _: ResourceInstanceWrapper<T> | null;
        static __: ResourceModelWrapper<T> | null;
      },
    };
    vmc = viewModelClassObj[wkrm.modelClassName];
  } else {
    vmc = viewModelClass;
  }

  const wrapper = new ResourceModelWrapper<T>(wkrm, graph, vmc, defaultAllow, store);
  vmc.prototype.__ = wrapper;
  return vmc;
}

class GraphManager {
  _initialized: boolean = false;
  archesClient: ArchesClient;
  // These are hydrated graphs - for metadata-only versions
  // of graphs, using wkrms[*].meta
  graphs: Map<string, ResourceModelWrapper<any>>;
  wkrms: Map<string, IWKRM>;
  defaultAllow: boolean = false;
  /** The resource store this manager reads through. Injectable so two managers
   *  (e.g. two layer stacks) stay independent; defaults to the shared singleton. */
  staticStore: StaticStore = staticStore;

  constructor(archesClient: ArchesClient, store: StaticStore = staticStore) {
    this.archesClient = archesClient;
    this.staticStore = store;
    this.graphs = new Map<string, ResourceModelWrapper<any>>();
    this.wkrms = new Map<string, IWKRM>();
  }

  getPruneTiles(pruneTiles?: boolean) {
    if (pruneTiles === undefined) {
      pruneTiles = !this.defaultAllow;
    }
    return pruneTiles;
  }

  async initialize(configurationOptions: ConfigurationOptions | undefined = undefined) {
    if (this._initialized) {
      return;
    }
    if (configurationOptions === undefined) {
      configurationOptions = new ConfigurationOptions();
    }
    const graphJsons: GraphResult = await this.archesClient.getGraphs();
    this.defaultAllow = configurationOptions.defaultAllowAllNodegroups;

    let graphs: Array<[string, StaticGraphMeta]> = Object.entries(graphJsons["models"]);
    const allowedGraphs = configurationOptions.graphs;
    if (allowedGraphs !== null) {
      if (allowedGraphs === false) {
        throw Error("No current meaning of allowedGraphs === false");
      } else if (allowedGraphs !== true) {
        graphs = graphs.filter(
          ([graphId, _]: [string, StaticGraphMeta]) => allowedGraphs.includes(graphId),
        );
      }
    }
    graphs.forEach(([graphId, meta]: [string, StaticGraphMeta]) => {
      if (!(meta instanceof StaticGraphMeta)) {
        meta = createStaticGraphMeta(meta);
      }
      meta.graphid = meta.graphid || graphId;
      const wkrm = createWKRM(meta);
      this.wkrms.set(wkrm.modelClassName, wkrm);
    });
    if (configurationOptions.eagerLoadGraphs) {
      await Promise.all(graphs.map(([g]) => this.loadGraph(g, configurationOptions.defaultAllowAllNodegroups)));
    }

    this._initialized = true;
  }

  /**
   * Resolve a model to its WKRM by exact class name, graph id, or a normalised
   * name ("NPC" / "npc" / "Monster Type" / "monster-type" all match the
   * "Npc" / "MonsterType" model). Shared by get() and loadGraph() so both accept
   * the same name forms. (N1: loadGraph previously matched only the exact class
   * name or graph id, so a first get("npc") that had to load the graph — i.e. the
   * cache-miss path get() delegates to — still threw despite get()'s own
   * normalised lookup.)
   */
  private resolveWkrm(modelClassName: string): IWKRM | undefined {
    const exact = this.wkrms.get(modelClassName);
    if (exact !== undefined) {
      return exact;
    }
    const target = normalizeModelName(modelClassName);
    return (
      [...this.wkrms.values()].find(w => w.graphId === modelClassName) ??
      [...this.wkrms.entries()].find(([name]) => normalizeModelName(name) === target)?.[1]
    );
  }

  async loadGraph<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string, defaultAllow: boolean=true): Promise<ResourceModelWrapper<RIVM>> {
    const modelClassName = typeof modelClass == 'string' ? modelClass : modelClass.name;

    const wkrm = this.resolveWkrm(modelClassName);
    if (wkrm === undefined) {
      throw Error(`Only loading graphs for which metadata is present, not ${modelClassName}`);
    }

    const wrapper = this.graphs.get(wkrm.graphId);
    if (wrapper !== undefined) {
      return wrapper;
    }

    const graph = await this.archesClient.getGraph(wkrm.meta);
    if (!graph) {
      throw Error(`Could not load graph ${wkrm.graphId}`);
    }

    // Load node configs (domain values, booleans, etc.) for this graph
    nodeConfigManager.loadFromGraph(graph);

    // Preserve a caller-supplied view-model class; fall back to a default class
    // built from the wkrm when the model was named by string (so a name that only
    // matched via resolveWkrm still loads, N1).
    const model: ResourceInstanceViewModelConstructor<RIVM> =
      typeof modelClass == 'string'
        ? makeResourceModelWrapper<RIVM>(undefined, wkrm, graph, defaultAllow, this.staticStore)
        : makeResourceModelWrapper<RIVM>(modelClass, wkrm, graph, defaultAllow, this.staticStore);

    this.graphs.set(graph.graphid, model.prototype.__);
    return model.prototype.__;
  }

  async get<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string, defaultAllow: boolean=true): Promise<ResourceModelWrapper<RIVM>> {
    let modelClassName: string;
    if (typeof modelClass == 'string') {
      modelClassName = modelClass;
    } else {
      modelClassName = modelClass.name;
    }

    // Initialize as a fallback — MUST be awaited, or a get() that races the first
    // initialize() sees an empty registry and throws "Cannot find model".
    await this.initialize(undefined);
    const wkrm = this.resolveWkrm(modelClassName);
    if (wkrm === undefined) {
      throw Error(`Cannot find model requested: ${modelClassName}`);
    }

    const wrapper = this.graphs.get(wkrm.graphId);
    if (wrapper === undefined) {
      // Cache miss: load the graph. loadGraph() shares resolveWkrm(), so the same
      // non-exact name that resolved here also resolves there (N1).
      return this.loadGraph(modelClass, defaultAllow);
    }
    return wrapper;
  }

  async getResource<T extends IRIVM<T>>(resourceId: string, lazy: boolean = true, pruneTiles?: boolean): Promise<T> {
    pruneTiles = this.getPruneTiles(pruneTiles);
    const rivm = await this.staticStore.loadOne(resourceId);
    let graph = this.graphs.get(rivm.resourceinstance.graph_id);
    if (!graph) {
      graph = await this.loadGraph(rivm.resourceinstance.graph_id, !pruneTiles);
      if (!graph) {
        throw Error(`Graph not found for resource ${resourceId}`);
      }
    }
    return graph.fromStaticResource(rivm, lazy, pruneTiles);
  }

  getGraph(graphId: string): StaticGraph {
    const wrapper = this.graphs.get(graphId);
    if (wrapper === undefined) {
      throw Error(`Cannot find graph requested: ${graphId}`);
    }
    return wrapper.graph;
  }
}

const graphManager = new GraphManager(archesClient);
viewContext.graphManager = graphManager;

// Re-export WKRM factory and class getter from backend
export { createWKRM, getWKRMClass } from "./backend";
export { GraphManager, graphManager, ArchesClientRemote, staticStore, StaticStore, ResourceModelWrapper, GraphMutator };
