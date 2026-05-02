import { GraphResult, archesClient, ArchesClient, ArchesClientRemote } from './client';
import { staticStore } from './staticStore';
import { CardComponent, DEFAULT_CARD_COMPONENT, Widget, getDefaultWidgetForNode } from './cards';
import {
  StaticTranslatableString,
  StaticCollection,
  StaticConstraint,
  StaticCard,
  StaticEdge,
  StaticCardsXNodesXWidgets,
  StaticTile,
  StaticGraph,
  StaticNode,
  StaticNodegroup,
  StaticResource,
  StaticResourceSummary,
  StaticGraphMeta
} from "./static-types";
import { PseudoValue, PseudoUnavailable, wrapRustPseudo } from "./pseudos.ts";
import { createInstanceWrapperForResource, createInstanceWrapperForModel, createResourceModelWrapper, createWKRM, getBackend } from "./backend";
import { ResourceInstanceViewModel, viewContext, SemanticViewModel, NodeViewModel } from "./viewModels.ts";
import { GetMeta, IRIVM, IStringKeyedObject, IPseudo, IInstanceWrapper, IViewModel, IModelWrapperBackend, IWKRM, ResourceInstanceViewModelConstructor, PermissionValue } from "./interfaces";
import { nodeConfigManager } from "./nodeConfig.ts";
import { generateUuidv5, AttrPromise } from "./utils";

// Import and re-export timing functions from dedicated module (avoids circular imports)
import { recordWasmTiming, printWasmTimings, clearWasmTimings, getWasmTimings } from './wasmTiming';
export { recordWasmTiming, printWasmTimings, clearWasmTimings, getWasmTimings };

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
      this.wasmWrapper = createInstanceWrapperForResource(resource);
      this.resource = resource;
      recordWasmTiming("createInstanceWrapperForResource", performance.now() - t0);
    } else {
      this.wasmWrapper = createInstanceWrapperForModel(model.wkrm.graphId);
      recordWasmTiming("createInstanceWrapperForModel", performance.now() - t0);
    }

    this._pseudoCache = new Map();
    this.cache = resource ? resource.__cache : undefined;
    this.scopes = resource ? resource.__scopes : undefined;
    this.metadata = resource ? resource.metadata : undefined;

    // Set lazy mode (NAPI instance wrapper manages tiles internally)
    if (typeof this.wasmWrapper.setLazy === 'function') {
      t0 = performance.now();
      this.wasmWrapper.setLazy(lazy);
      recordWasmTiming("setLazy", performance.now() - t0);
    }

    // Set up tile loader callback
    if (resource && typeof this.wasmWrapper.setTileLoader === 'function') {
      // Create a tile loader that loads from staticStore API
      const resourceId = resource.resourceinstance.resourceinstanceid;
      t0 = performance.now();
      this.wasmWrapper.setTileLoader((nodegroupId: string) => {
        // If nodegroupId is null/undefined, load all tiles
        // Otherwise load tiles for specific nodegroup
        const tiles = staticStore.loadTiles(resourceId, nodegroupId);
        return tiles;
      });
      recordWasmTiming("setTileLoader", performance.now() - t0);
    }

    // Load tiles into Rust if we have any - regardless of lazy mode,
    // if tiles are already present we should use them
    let tilesLoaded = false;
    if (typeof this.wasmWrapper.tilesLoaded === 'function') {
      t0 = performance.now();
      tilesLoaded = this.wasmWrapper.tilesLoaded();
      recordWasmTiming("tilesLoaded (constructor)", performance.now() - t0);
    }

    // Use loadTilesFromResource to avoid expensive tiles getter (which creates N WASM wrappers)
    if (!tilesLoaded && resource && resource.tilesLoaded && typeof this.wasmWrapper.loadTilesFromResource === 'function') {
      t0 = performance.now();
      this.wasmWrapper.loadTilesFromResource(resource, assumeTilesComprehensiveForNodegroup);
      recordWasmTiming("loadTilesFromResource", performance.now() - t0);
    }

    if (pruneTiles && resource) {
      t0 = performance.now();
      this.pruneResourceTiles();
      recordWasmTiming("pruneResourceTiles", performance.now() - t0);
    }
    recordWasmTiming("constructor total", performance.now() - constructorStart);
  }

  async ensureTilesLoaded(): Promise<void> {
    if (!this.wasmWrapper.tilesLoaded()) {
      // Use in-memory resource if available (avoids registry/disk lookup)
      if (this.resource && this.resource.tiles && this.resource.tiles.length > 0) {
        this.wasmWrapper.loadTilesFromResource(this.resource, true);
        return;
      }

      // Otherwise fetch from staticStore registry
      const resourceId = this.wasmWrapper.getResourceId();
      const fullResource = await staticStore.ensureFullResource(resourceId);
      if (fullResource && fullResource.tilesLoaded) {
        this.wasmWrapper.loadTilesFromResource(fullResource, true);
      }
    }
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
        recordWasmTiming("tilesLoaded (populate)", performance.now() - t0);
        if (!loaded) {
          t0 = performance.now();
          await this.ensureTilesLoaded();
          recordWasmTiming("ensureTilesLoaded", performance.now() - t0);
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
      recordWasmTiming("populate (WASM)", performance.now() - t0);

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

      recordWasmTiming("populate total", performance.now() - populateStart);
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

type GraphMutation = (baseGraph: StaticGraph) => StaticGraph;

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

  _generateUuidv5(key: string) {
    return generateUuidv5(['graph', this.baseGraph.graphid], key);
  }

  _generateEdge(fromNode: string, toNode: string, ontologyProperty: string, name?: string, description?: string) {
    const edgeId = this._generateUuidv5(`node-${fromNode}-${toNode}`);
    return new StaticEdge({
      description: description || null,
      domainnode_id: fromNode,
      edgeid: edgeId,
      graph_id: this.baseGraph.graphid,
      name: name || null,
      rangenode_id: toNode,
      ontologyproperty: ontologyProperty,
    });
  }

  addSemanticNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options: {
    exportable?: boolean,
    fieldname?: string,
    hascustomalias?: boolean;
    is_collector?: boolean;
    isrequired?: boolean;
    issearchable?: boolean;
    istopnode?: boolean;
    sortorder?: number;
  } = {}, config?: {[key: string]: any}) {
    return this._addGenericNode(
      parentAlias,
      alias,
      name,
      cardinality,
      "semantic",
      ontologyClass,
      parentProperty,
      description,
      options,
      config
    );
  }

  addConceptNode(parentAlias: string | null, alias: string, name: string, collection: StaticCollection, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options: {
    is_list?: boolean,
    exportable?: boolean,
    fieldname?: string,
    hascustomalias?: boolean;
    is_collector?: boolean;
    isrequired?: boolean;
    issearchable?: boolean;
    istopnode?: boolean;
    sortorder?: number;
  } = {}, config?: {[key: string]: any}) {
    config = config || {};
    if (collection?.id) {
      config['rdmCollection'] = collection.id
    }
    return this._addGenericNode(
      parentAlias,
      alias,
      name,
      cardinality,
      options.is_list ? "concept-list" : "concept",
      ontologyClass,
      parentProperty,
      description,
      options,
      config
    );
  }

  addCard(nodegroup: string | StaticNodegroup, name: string | StaticTranslatableString, component?: CardComponent, options: {
    active?: boolean,
    constraints?: Array<StaticConstraint>,
    cssclass?: string | null,
    helpenabled?: boolean,
    helptext?: string | null | StaticTranslatableString,
    helptitle?: string | null | StaticTranslatableString,
    instructions?: string | null | StaticTranslatableString,
    is_editable?: boolean,
    description?: string | null,
    sortorder?: number | null,
    visible?: boolean
  } = {}, config?: {[key: string]: any}) {
    const nodegroupId = typeof nodegroup === 'string' ? nodegroup : nodegroup.nodegroupid;
    const cardName = name instanceof StaticTranslatableString ? name : new StaticTranslatableString(name);
    const cardComponent = component || DEFAULT_CARD_COMPONENT;
    const helptext = options?.helptext && (
      options.helptext instanceof StaticTranslatableString ?
        options.helptext : new StaticTranslatableString(options.helptext)
    );
    const helptitle = (options?.helptitle && (
      options.helptitle instanceof StaticTranslatableString ?
        options.helptitle : new StaticTranslatableString(options.helptitle)
    ));
    const instructions = (options?.instructions && (
      options.instructions instanceof StaticTranslatableString ?
        options.instructions : new StaticTranslatableString(options.instructions)
    ));
    this.mutations.push((graph: StaticGraph) => {
      const cards = graph.cards || [];
      if (cards.filter(card => card.nodegroup_id === nodegroup).length > 0) {
        throw Error(`This nodegroup, ${nodegroupId}, already has a card`);
      }
      const cardId = this._generateUuidv5(`card-ng-${nodegroupId}`);
      const card = new StaticCard({
        active: options.active === undefined ? true : options.active,
        cardid: cardId,
        component_id: cardComponent.id,
        config: config || undefined,
        constraints: options.constraints || [],
        cssclass: options.cssclass || null,
        description: options.description || null,
        graph_id: graph.graphid,
        helpenabled: !!(options.helpenabled || (options.helpenabled === undefined && (helptext || helptitle))),
        helptext: helptext || new StaticTranslatableString(''),
        helptitle: helptitle || new StaticTranslatableString(''),
        instructions: instructions || new StaticTranslatableString(''),
        is_editable: options.is_editable === undefined ? true : options.is_editable,
        name: cardName,
        nodegroup_id: nodegroupId,
        sortorder: options.sortorder || null,
        visible: options.visible === undefined ? true : options.visible
      });
      graph.pushCard(card);
      return graph;
    });
  }

  addStringNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string | string[], parentProperty: string, description?: string, options: {
    exportable?: boolean,
    fieldname?: string,
    hascustomalias?: boolean;
    is_collector?: boolean;
    isrequired?: boolean;
    issearchable?: boolean;
    istopnode?: boolean;
    sortorder?: number;
  } = {}, config?: {[key: string]: any}) {
    return this._addGenericNode(
      parentAlias,
      alias,
      name,
      cardinality,
      "string",
      ontologyClass,
      parentProperty,
      description,
      options,
      config
    );
  }

  _addNodegroup(parentAlias: string | null, nodegroupId: string, cardinality: 'n' | '1', name?: StaticTranslatableString) {
    this.mutations.push((graph: StaticGraph) => {
      const prnt = parentAlias === null ? graph.root : graph.nodes.find(node => node.alias === parentAlias);
      if (!prnt) {
        throw Error(`Missing parent for nodegroup: ${parentAlias}`);
      }
      const nodegroup = new StaticNodegroup({
        cardinality: cardinality,
        legacygroupid: null,
        nodegroupid: nodegroupId,
        parentnodegroup_id: prnt.nodegroup_id
      });
      graph.pushNodegroup(nodegroup);
      return graph;
    });
    if (this.autocreateCard) {
      this.addCard(nodegroupId, name || '(unnamed)');
    }
    return this;
  }

  _addGenericNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', datatype: string, ontologyClass: string | string[], parentProperty: string, description?: string, options: {
    exportable?: boolean,
    fieldname?: string,
    hascustomalias?: boolean;
    is_collector?: boolean;
    isrequired?: boolean;
    issearchable?: boolean;
    istopnode?: boolean;
    sortorder?: number;
  } = {}, config?: {[key: string]: any}) {
    const nodeId = this._generateUuidv5(`node-${alias}`);
    const node = {
      alias: alias,
      config: config || {},
      datatype: datatype,
      description: description || undefined,
      exportable: options.exportable || false,
      fieldname: options.fieldname || undefined,
      graph_id: this.baseGraph.graphid,
      hascustomalias: options.hascustomalias || false,
      is_collector: options.is_collector || false,
      isrequired: options.isrequired || false,
      issearchable: options.issearchable || true, // This is the default in Arches I believe
      istopnode: options.istopnode || false,
      name: name,
      nodegroup_id: '',
      nodeid: nodeId,
      parentproperty: parentProperty,
      sortorder: options.sortorder || 0,
      ontologyclass: ontologyClass,
      sourcebranchpublication_id: undefined,
    };
    if (cardinality === 'n' || parentAlias === null) {
      node.nodegroup_id = nodeId;
      this._addNodegroup(parentAlias, node.nodegroup_id, cardinality, new StaticTranslatableString(name));
    }
    this.mutations.push((graph: StaticGraph) => {
      const prnt = parentAlias === null ? graph.root : graph.nodes.find(node => node.alias === parentAlias);
      if (!prnt) {
        throw Error(`Parent node does not exist: ${parentAlias}`);
      }
      // FIXME: we assume we are not adding a root node, but nowhere do we say this.
      node.nodegroup_id = node.nodegroup_id !== '' ? node.nodegroup_id : prnt.nodegroup_id || '';
      const newNode = new StaticNode(node);
      graph.pushNode(newNode);
      const edge = this._generateEdge(prnt.nodeid, nodeId, parentProperty);
      graph.pushEdge(edge);
      return graph;
    });

    if (this.autocreateCard && datatype !== 'semantic') {
      const widget = getDefaultWidgetForNode(node);
      const config = widget.getDefaultConfig();
      config.label = name;
      this.addWidgetToCard(
        nodeId,
        widget,
        name,
        config,
        {
          sortorder: node.sortorder,
          silentSkip: true // if, for some reason, the card is not present (i.e. was removed), we should not worry
        }
      );
    }
    return this;
  }

  addWidgetToCard(
    nodeId: string,
    widget: Widget,
    name: string,
    config: {[key: string]: any},
    options: {
      sortorder?: number | null,
      silentSkip?: boolean,
      visible?: boolean
    } = {}
  ): GraphMutator {
    this.mutations.push((graph: StaticGraph) => {
      const node = graph.nodes.find(node => node.nodeid === nodeId);
      if (!node) {
        throw Error(`Tried to add card to graph ${graph.graphid} for node ${nodeId} but it was not found.`);
      }
      const card = graph.cards?.find(card => card.nodegroup_id === node.nodegroup_id);

      if (card) {
        const cardXNodeXWidgetId = this._generateUuidv5(`cxnxw-${nodeId}-${widget.id}`);

        const cardXNodeXWidget = new StaticCardsXNodesXWidgets({
          card_id: card.cardid,
          config: config,
          id: cardXNodeXWidgetId,
          label: new StaticTranslatableString(name),
          node_id: nodeId,
          sortorder: options.sortorder || 0,
          visible: options.visible === undefined || options.visible,
          widget_id: widget.id
        });
        graph.pushCardXNodeXWidget(cardXNodeXWidget);
      } else if (!options.silentSkip) {
        throw Error(`Failed adding widget for ${nodeId} to card for ${node.nodegroup_id} on graph ${graph.graphid}, as no card for this nodegroup (yet?)`);
      }
      return graph;
    });
    return this;
  }

  apply() {
    if (!this.baseGraph.copy) {
      throw Error("Attempt to build a mutator without a proper StaticGraph base graph");
    }
    // TODO: complete deepcopies
    const graph = this.baseGraph.copy();
    return this.mutations.reduce((graph, mutation) => mutation(graph), graph);
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

  // Cached copies of backend data (to avoid repeated cross-boundary calls)
  private _nodes: Map<string, StaticNode> | null = null;
  private _nodesByAlias: Map<string, StaticNode> | null = null;
  private _edges: Map<string, string[]> | null = null;
  private _nodegroups: Map<string, StaticNodegroup> | null = null;

  constructor(wkrm: IWKRM, graph: StaticGraph, viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>, defaultAllow: boolean = false) {
    this.wkrm = wkrm;
    this._backend = createResourceModelWrapper(wkrm, graph, defaultAllow);
    this.pruneTiles = !defaultAllow;
    this.viewModelClass = viewModelClass;
  }

  // =========================================================================
  // Backend-delegated properties
  // =========================================================================

  get graph(): StaticGraph {
    return this._backend.graph;
  }

  set graph(g: StaticGraph) {
    this._backend.graph = typeof g === 'string' ? g : JSON.stringify(g);
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
    // WASM backend's buildNodes() is actually buildNodesForGraph(graph) on the
    // raw WASMResourceModelWrapper; NAPI's buildNodes() is a no-op (caches
    // built eagerly in constructor). Use backend type to dispatch correctly.
    if (getBackend() === 'napi') {
      this._backend.buildNodes();
    } else {
      // WASM: the raw class uses buildNodesForGraph(graph)
      const backend = this._backend as any;
      if (typeof backend.buildNodesForGraph === 'function') {
        backend.buildNodesForGraph(this._backend.graph);
      } else {
        this._backend.buildNodes();
      }
    }
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
    const nodes = [...this.graph.nodes.values()];
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

  getCollections(accessible?: boolean): string[] {
    const accessibleOnly = accessible || false;
    const nodes = [...this.graph.nodes.values()];
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
    yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy, params.pruneTiles);
  }

  // New summary-based methods for performance optimization
  async* summaryGenerator(staticSummaries: AsyncIterable<StaticResourceSummary>, lazy: boolean = true): AsyncGenerator<RIVM> {
    for await (const summary of staticSummaries) {
      // Create lightweight resource from summary
      const summaryResource = StaticResource.fromSummary(summary);
      yield this.fromStaticResource(summaryResource, lazy, false); // lazy=true, pruneTiles=false
    }
  }

  async* iterAllSummaries(params: { limit?: number }): AsyncGenerator<RIVM> {
    yield* this.summaryGenerator(staticStore.loadAllSummaries(this.wkrm.graphId, params.limit), true);
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
    const fullResource = await staticStore.ensureFullResource(id);
    return this.fromStaticResource(fullResource, false, true); // non-lazy, prune tiles
  }

  async findStatic(id: string): Promise<StaticResource> {
    return await staticStore.loadOne(id);
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
    recordWasmTiming("makeInstance", performance.now() - start);

    if (!wkri.$) {
      throw Error("Could not load resource from static definition");
    }

    const pop = wkri.$.populate(lazy).then(() => {
      recordWasmTiming("fromStaticResource total", performance.now() - start);
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
  defaultAllow: boolean
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

  const wrapper = new ResourceModelWrapper<T>(wkrm, graph, vmc, defaultAllow);
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

  constructor(archesClient: ArchesClient) {
    this.archesClient = archesClient;
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
        meta = new StaticGraphMeta(meta);
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

  async loadGraph<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string, defaultAllow: boolean=true): Promise<ResourceModelWrapper<RIVM>> {
    let modelClassName: string;
    if (typeof modelClass == 'string') {
      modelClassName = modelClass;
    } else {
      modelClassName = modelClass.name;
    }

    let wkrm = this.wkrms.get(modelClassName);
    if (wkrm === undefined) {
      wkrm = [...this.wkrms.values()].find(wkrm => wkrm.graphId === modelClassName);
      if (wkrm === undefined) {
        throw Error(`Only loading graphs for which metadata is present, not ${modelClassName}`);
      }
      modelClass = wkrm.modelClassName;
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

    let model: ResourceInstanceViewModelConstructor<RIVM>;
    if (typeof modelClass == 'string') {
      modelClassName = modelClass;
      model = makeResourceModelWrapper<RIVM>(undefined, wkrm, graph, defaultAllow);
    } else {
      modelClassName = modelClass.name;
      model = makeResourceModelWrapper<RIVM>(modelClass, wkrm, graph, defaultAllow);
    }

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

    // Initialize as a fallback
    this.initialize(undefined);
    let wkrm = this.wkrms.get(modelClassName);
    if (wkrm === undefined) {
      wkrm = [...this.wkrms.values()].find(w => w.graphId === modelClassName);
      if (wkrm === undefined) {
        throw Error(`Cannot find model requested: ${modelClassName}`);
      }
    }

    const wrapper = this.graphs.get(wkrm.graphId);
    if (wrapper === undefined) {
      return this.loadGraph(modelClass, defaultAllow);
    }
    return wrapper;
  }

  async getResource<T extends IRIVM<T>>(resourceId: string, lazy: boolean = true, pruneTiles?: boolean): Promise<T> {
    pruneTiles = this.getPruneTiles(pruneTiles);
    const rivm = await staticStore.loadOne(resourceId);
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
export { GraphManager, graphManager, ArchesClientRemote, staticStore, ResourceModelWrapper, GraphMutator };
