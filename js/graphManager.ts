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
  StaticFunctionsXGraphs,
  StaticTile,
  StaticGraph,
  StaticNode,
  StaticNodegroup,
  StaticResource,
  StaticResourceSummary,
  StaticResourceDescriptors,
  StaticGraphMeta,
  IStaticDescriptorConfig
} from "./static-types";
import { PseudoList, PseudoUnavailable, wrapRustPseudo, makePseudoCls_JS } from "./pseudos.ts";
import { WKRM, WASMResourceModelWrapper, WASMResourceInstanceWrapper, newWASMResourceInstanceWrapperForResource, newWASMResourceInstanceWrapperForModel } from "../pkg/alizarin";
import { DEFAULT_LANGUAGE, ResourceInstanceViewModel, ValueList, viewContext, SemanticViewModel, NodeViewModel } from "./viewModels.ts";
import { CheckPermission, GetMeta, IRIVM, IStringKeyedObject, IPseudo, IInstanceWrapper, IViewModel, ResourceInstanceViewModelConstructor } from "./interfaces";
import { } from "./nodeConfig.ts";
import { generateUuidv5, AttrPromise, buildResourceDescriptors, serializeValuesMap } from "./utils";

const MAX_GRAPH_DEPTH = 100;

class ConfigurationOptions {
  graphs: Array<string> | null | boolean = null;
  eagerLoadGraphs: boolean = false;
  defaultAllowAllNodegroups: boolean = false;
}

export class ResourceInstanceWrapper<RIVM extends IRIVM<RIVM>> implements IInstanceWrapper<RIVM> {
  wkri: RIVM;
  model: ResourceModelWrapper<RIVM>;
  wasmWrapper: WASMResourceInstanceWrapper;
  resource?: StaticResource;

  valueList: ValueList<RIVM>;
  cache: {[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined;
  scopes?: string[];
  metadata?: {[key: string]: string};
  private tilesLoaded: boolean = false;

  constructor(
    wkri: RIVM,
    model: ResourceModelWrapper<RIVM>,
    resource: StaticResource | null | false, // False to disable dynamic resource-loading
    pruneTiles: boolean = true,
    lazy: boolean = false
  ) {
    this.wkri = wkri;
    this.model = model;

    // Initialize WASM wrapper for tile management
    if (resource) {
      this.wasmWrapper = newWASMResourceInstanceWrapperForResource(resource);
      this.resource = resource;
    } else {
      this.wasmWrapper = newWASMResourceInstanceWrapperForModel(model.wkrm.graphId);
    }

    this.valueList = new ValueList(new Map<string, any>(), new Map<string, boolean>(), this);
    this.cache = resource ? resource.__cache : undefined;
    this.scopes = resource ? resource.__scopes : undefined;
    this.metadata = resource ? resource.metadata : undefined;

    // Set lazy mode
    this.wasmWrapper.setLazy(lazy);

    // Set up tile loader callback
    if (resource) {
      // Create a tile loader that loads from staticStore API
      const resourceId = resource.resourceinstance.resourceinstanceid;
      this.wasmWrapper.setTileLoader((nodegroupId) => {
        // If nodegroupId is null/undefined, load all tiles
        // Otherwise load tiles for specific nodegroup
        return staticStore.loadTiles(resourceId, nodegroupId);
      });
    }

    // Load tiles into Rust if we have any (non-lazy mode)
    if (!lazy && !this.wasmWrapper.tilesLoaded() && resource && resource.tiles && resource.tiles.length > 0) {
      try {
        this.wasmWrapper.loadTiles(resource.tiles);
      } catch (e) {
        console.error("Failed to load tiles into WASM:", e);
      }
    }

    if (pruneTiles && resource) {
      this.pruneResourceTiles()
    }
  }

  async ensureTilesLoaded(): Promise<void> {
    if (!this.wasmWrapper.tilesLoaded()) {
      // Load tiles on-demand
      const tiles = await staticStore.loadTiles(this.wasmWrapper.getResourceId());

      // Load tiles into Rust
      try {
        this.wasmWrapper.loadTiles(tiles);
      } catch (e) {
        console.error("Failed to load tiles into WASM:", e);
      }

      // Re-populate with full tile data
      await this.populate(false); // non-lazy to process all tiles
    }
  }

  pruneResourceTiles(): undefined {
    this.wasmWrapper.pruneResourceTiles();
  }

  async loadNodes(aliases: Array<string>): Promise<void> {
    for (const key of aliases) {
      await this.valueList.retrieve(key);
    }
  }

  async getName(update: boolean = false) {
    // If just reading cached data, use Rust implementation
    if (!update) {
      return this.wasmWrapper.getName();
    }

    // Otherwise build/update name using TypeScript logic
    const descriptors = await this.getDescriptors(update);
    const resourceName = (descriptors && descriptors.name) || '<Unnamed>';
    if (this.resource && this.resource.resourceinstance) {
      this.resource.resourceinstance.name = resourceName;
    }
    return resourceName;
  }

  async getDescriptors(update: boolean = false) {
    // If just reading cached data, use Rust implementation
    if (!update) {
      const cachedDescriptors = this.wasmWrapper.getDescriptors();
      if (cachedDescriptors && !cachedDescriptors.isEmpty()) {
        return cachedDescriptors;
      }
    }

    // Otherwise build descriptors using utility function
    const descriptors = await buildResourceDescriptors(
      this.model.graph,
      this.model.getNodeObjects(),
      this.valueList
    );

    // Cache the built descriptors
    if (this.resource && this.resource.resourceinstance) {
      this.resource.resourceinstance.descriptors = descriptors;
      if (descriptors.name) {
        this.resource.resourceinstance.descriptors.name = descriptors.name;
      }
    }

    return descriptors;
  }

  addPseudo(childNode: StaticNode, tile: StaticTile | null, node: StaticNode): IPseudo {
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
        const valueList: ValueList<any> = this.valueList;
        valueList.setDefault(key, []).then((val: Array<any>) => val.push(child));
        return child;
      }

      // Wrap the Rust value in TS PseudoValue/PseudoList
      const child = wrapRustPseudo(rustValue, this.wkri, this.model);

      const valueList: ValueList<any> = this.valueList;
      valueList.setDefault(key, []).then((val: Array<any>) => val.push(child));
      return child;
    } catch (e) {
      console.error("Rust makePseudoValue failed:", e);
      throw new Error(`Rust makePseudoValue failed: ${e}. This should not happen - check Rust implementation.`);
    }
  }

  allEntries(): MapIterator<[string, Array<IPseudo> | false | null]> {
    return this.valueList.values.entries()
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
    const values = this.valueList;
    const node = this.model.getRootNode();
    if (node) {
      let value;
      const alias = node.alias;
      if (!(typeof alias == 'string')) {
        throw Error(`Alias missing for node ${node.nodeid}`);
      }
      await values.setDefault(alias, []);
      const nodeValues = await values.get(alias);

      if (nodeValues.length > 1) {
        throw Error("Cannot have multiple root tiles");
      } else if (nodeValues.length == 1) {
        value = nodeValues[0];
      } else {
        // Fallback for empty resources - use Rust to create value with null tile
        // This happens when ValueList.retrieve() skips ensureNodegroup (no wrapper.resource)
        const rustValue = this.wasmWrapper.makePseudoValue(
          alias,
          null,  // tile_id
          true,  // is_permitted
          false  // is_single
        );
        value = wrapRustPseudo(rustValue, this.wkri, this.model);
        values.set(alias, [value]);
      }
      return value;
    }
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
   * Ensure nodegroup is loaded with all its data
   * Uses Rust implementation via WASM
   */
  async ensureNodegroup(
    allValues: Map<string, any>,
    allNodegroups: Map<string, boolean>,
    nodegroupId: string,
    addIfMissing: boolean,
    doImpliedNodegroups: boolean = true
  ): Promise<[Map<string, any>, Set<string>]> {

    // Ensure tiles are loaded for this nodegroup if using lazy loading
    if (this.wasmWrapper.getTileLoader() && !this.wasmWrapper.hasTilesForNodegroup(nodegroupId)) {
      const loader = this.wasmWrapper.getTileLoader();
      if (loader) {
        try {
          const tiles = await loader(nodegroupId);
          this.wasmWrapper.loadTiles(tiles);
          this.wasmWrapper.markNodegroupTilesLoaded(nodegroupId);
        } catch (e) {
          console.error(`Failed to load tiles for nodegroup ${nodegroupId}:`, e);
          // Continue anyway - will work with empty tiles
        }
      }
    }

    try {
      // Phase 4h: Pass nodegroup permissions to Rust - Rust will compute tile permissions
      const nodegroupPermissions = this.model.getPermittedNodegroups();

      // Serialize maps for efficient Rust deserialization
      const serializedValues = serializeValuesMap(allValues);
      const serializedNodegroups = Object.fromEntries(allNodegroups);
      const serializedPermissions = Object.fromEntries(nodegroupPermissions);

      // Call Rust implementation
      const result = this.wasmWrapper.ensureNodegroup(
        serializedValues,
        serializedNodegroups,
        nodegroupId,
        addIfMissing,
        serializedPermissions,  // Phase 4h: Pass nodegroup permissions instead of tile permissions
        doImpliedNodegroups
      );

      // Phase 4i: Return raw Rust values instead of wrapping them
      // Caller will handle wrapping when needed
      const newValues = new Map<string, any>();

      // Get all values in a single boundary crossing instead of N+1 calls
      const allRustValues = result.getAllValues();

      for (const [alias, rustValue] of allRustValues.entries()) {

        if (!rustValue) {
          // Sentinel undefined case
          newValues.set(alias, undefined);
          continue;
        }

        // Check if value already exists in allValues
        let existing = allValues.get(alias);
        if (existing instanceof Promise) {
          existing = await existing;
        }
        if (existing !== false && existing !== undefined) {
          newValues.set(alias, existing);
          continue;
        }

        // Phase 4i: Store raw Rust value instead of wrapping
        // The caller (ValueList or populate) will wrap when needed
        if (!newValues.has(alias)) {
          newValues.set(alias, []);
        }

        newValues.get(alias).push(rustValue);
      }

      // Update allValues with newValues (filtering undefined - line 343)
      for (const [key, value] of newValues.entries()) {
        if (value !== undefined) {
          allValues.set(key, value);
        }
      }

      // Update allNodegroups from Rust result
      const updatedNodegroups = result.allNodegroupsMap;

      // Handle both Map and plain object
      if (updatedNodegroups instanceof Map) {
        for (const [key, value] of updatedNodegroups.entries()) {
          if (typeof value === 'boolean') {
            allNodegroups.set(key, value);
          }
        }
      } else {
        for (const [key, value] of Object.entries(updatedNodegroups)) {
          if (typeof value === 'boolean') {
            allNodegroups.set(key, value);
          }
        }
      }

      const impliedNodegroups = new Set(result.impliedNodegroups);

      // Parallel testing mode: compare Rust vs JS
      return [newValues, impliedNodegroups];

    } catch (e) {
      // PORT: Phase 4e-3 - Fail explicitly instead of silent fallback
      console.error("ensureNodegroup failed:", e);
      throw new Error(`ensureNodegroup failed: ${e}. `);
    }
  }

  /**
   * Populate all nodegroups for a resource
   * Uses Rust implementation via WASM
   */
  async populate(lazy: boolean): Promise<void> {
    const nodegroupObjs = this.model.getNodegroupObjects();
    const rootNode = this.model.getRootNode();

    if (rootNode.alias === null) {
      throw Error("Cannot populate a model with no proper root node");
    }


    try {
      if (!lazy && this.resource) {
        // Ensure tiles are loaded if we need them for non-lazy population
        if (!this.resource.__tilesLoaded) {
          await this.ensureTilesLoaded();
        }
      }

      // Phase 4h: Pass nodegroup permissions to Rust - Rust will compute tile permissions
      const nodegroupPermissions = this.model.getPermittedNodegroups();

      // Get all tile IDs and nodegroup IDs
      const nodegroupIds = [...nodegroupObjs.keys()];

      // Call Rust implementation
      const result = this.wasmWrapper.populate(
        lazy,
        nodegroupIds,
        rootNode.alias,
        nodegroupPermissions  // Phase 4h: Pass nodegroup permissions instead of tile permissions
      );

      // Convert all recipes to PseudoValues
      const allValues: Map<string, any> = new Map();
      const allNodegroups: Map<string, any> = new Map();

      // Initialize allNodegroups (needed for ValueList)
      for (const nodegroupId of nodegroupIds) {
        allNodegroups.set(nodegroupId, false);
      }

      // Set root node alias
      allValues.set(rootNode.alias, false);

      // PORT: Phase 4c - Use structured RustPseudoList values directly instead of recipes
      // PORT: src/instance_wrapper.rs:563-569 - WasmPopulateResult with values HashMap
      const newValues = new Map<string, any>();
      // Get all values in a single boundary crossing instead of N+1 calls
      const allRustValues = result.getAllValues();

      for (const [alias, rustValue] of allRustValues.entries()) {
        // PORT: Phase 4c - Get WasmPseudoList directly from Rust
        // PORT: src/instance_wrapper.rs:156 - getValue returns Option<WasmPseudoList>

        if (!rustValue) {
          // Sentinel undefined case
          // PORT: src/instance_wrapper.rs:732 - sentinel_undefined flag
          newValues.set(alias, undefined);
          continue;
        }

        // Check if value already exists in allValues
        let existing = allValues.get(alias);
        if (existing instanceof Promise) {
          existing = await existing;
        }
        if (existing !== false && existing !== undefined) {
          newValues.set(alias, existing);
          continue;
        }

        // Wrap Rust value in JavaScript PseudoList/PseudoValue
        // wrapRustPseudo returns a complete PseudoList with all values already inside
        const wrappedValue = wrapRustPseudo(rustValue, this.wkri, this.model);
        newValues.set(alias, wrappedValue);
      }

      // Update allValues with newValues
      for (const [key, value] of newValues.entries()) {
        if (value !== undefined) {
          allValues.set(key, value);
        }
      }

      // Root node doesn't have nodegroup data - set to empty array if still false
      if (allValues.get(rootNode.alias) === false) {
        allValues.set(rootNode.alias, []);
      }

      // Update allNodegroups from Rust result
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
            allNodegroups.set(key, value);
          }
        }
      }

      // Create ValueList with results
      this.valueList = new ValueList(
        allValues,
        allNodegroups,
        this,
      );

    } catch (error) {
      // PORT: Phase 4e-3 - Fail explicitly instead of silent fallback
      console.error('[populate] Rust implementation failed:', error);
      throw new Error(`populate failed: ${error}`);
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
    for (let pseudos of this.valueList.values.values()) {
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
   * Graph traversal helper that creates PseudoValues from nodegroup tiles
   * Uses Rust implementation via WASM
   */
  async valuesFromResourceNodegroup(
    existingValues: Map<string, any>,
    nodegroupTiles: (StaticTile | null)[],
    nodegroupId: string,
    nodeObjs: Map<string, StaticNode>,
    edges: Map<string, string[]>,
  ): Promise<[Map<string, any>, Set<string>]> {


    try {
      // Convert tiles to tile IDs for Rust
      const tileIds = nodegroupTiles.map(tile => tile?.tileid || "");

      // Call Rust implementation
      const result = this.wasmWrapper.valuesFromResourceNodegroup(
        existingValues,
        tileIds,
        nodegroupId,
        nodeObjs,
        edges
      );

      // PORT: Phase 4c - Use structured RustPseudoList values directly instead of recipes
      const allValues = new Map<string, any>();

      // Get all values in a single boundary crossing instead of N+1 calls
      const allRustValues = result.getAllValues();

      for (const [alias, rustValue] of allRustValues.entries()) {
        // PORT: Phase 4c - Get WasmPseudoList directly from Rust

        if (!rustValue) {
          // Sentinel undefined case
          allValues.set(alias, undefined);
          continue;
        }

        // Check if value already exists in existingValues
        let existing = existingValues.get(alias);
        if (existing instanceof Promise) {
          existing = await existing;
        }
        if (existing !== false && existing !== undefined) {
          allValues.set(alias, existing);
          continue;
        }

        // Create TS PseudoValue/PseudoList with Rust backing
        // PORT: Phase 4e - Use wrapRustPseudo for cleaner wrapping of Rust values
        const pseudoNode = wrapRustPseudo(rustValue, this.wkri, this.model);

        if (!allValues.has(alias)) {
          allValues.set(alias, []);
        }

        // Handle PseudoList merging
        if (Array.isArray(pseudoNode)) {
          const value = allValues.get(alias);
          if (value !== undefined && value !== false) {
            let merged = false;
            for (const pseudoList of allValues.get(alias)) {
              if (!(pseudoList instanceof PseudoList) || !(pseudoNode instanceof PseudoList)) {
                throw Error(`Should be all lists not ${typeof pseudoList} and ${typeof pseudoNode}`);
              }

              if (pseudoList.parentNode == pseudoNode.parentNode) {
                for (const ps of pseudoNode) {
                  pseudoList.push(ps);
                }
                merged = true;
                break;
              }
            }
            if (merged) {
              continue;
            }
          }
        }

        allValues.get(alias).push(pseudoNode);
      }

      const impliedNodegroups = new Set(result.impliedNodegroups);

      return [allValues, impliedNodegroups];

    } catch (e) {
      // PORT: Phase 4e-3 - Fail explicitly instead of silent fallback
      console.error("valuesFromResourceNodegroup failed:", e);
      throw new Error(`valuesFromResourceNodegroup failed: ${e}`);
    }
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

  addSemanticNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string, parentProperty: string, description?: string, options: {
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

  addConceptNode(parentAlias: string | null, alias: string, name: string, collection: StaticCollection, cardinality: 'n' | '1', ontologyClass: string, parentProperty: string, description?: string, options: {
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

  addStringNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', ontologyClass: string, parentProperty: string, description?: string, options: {
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

  _addGenericNode(parentAlias: string | null, alias: string, name: string, cardinality: 'n' | '1', datatype: string, ontologyClass: string, parentProperty: string, description?: string, options: {
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

// WASMResourceModelWrapper is now imported from WASM

class ResourceModelWrapper<RIVM extends IRIVM<RIVM>> extends WASMResourceModelWrapper {
  viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>;
  // Phase 4h: Simplified to boolean-only (removed CheckPermission callback)
  permittedNodegroups?: Map<string, boolean>;

  constructor(wkrm: WKRM, graph: StaticGraph, viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>, defaultAllow: boolean) {
    super(wkrm, graph, defaultAllow);
    this.viewModelClass = viewModelClass;
  }

  getRoot(): NodeViewModel {
    const node = this.getRootNode();
    const pseudoNode = this.createPseudoNode(node.alias);
    return new NodeViewModel(pseudoNode, this);
  }

  buildNodes() {
    const graph = this.graph ?? graphManager.getGraph(this.wkrm.graphId);
    return this.buildNodesForGraph(graph);
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
    // RMV DOES THESE NEED MOVED TO RUST?
    // Get the graph once to avoid multiple clones
    const graph = this.graph;
    const allNodegroups = this.getNodegroupObjects();
    const root = graph.root.nodeid;
    // Strictly, this ultimately also contains nodes, but not all allowed nodes - the key point is that
    // it has only and all nodegroups that we will keep.
    const allowedNodegroups = new Map([...allNodegroups.values()].filter((nodegroup: StaticNodegroup) => {
      return this.isNodegroupPermitted(nodegroup.nodegroupid || '', null);
    }).map((nodegroup: StaticNodegroup) => [nodegroup.nodegroupid, nodegroup.nodegroupid === null || nodegroup.nodegroupid === '' || nodegroup.nodegroupid === root]));
    const backedges: Map<string, string> = new Map();
    for (const [d, rs] of this.getEdges()) {
      for (const r of rs) {
        if (backedges.has(r)) {
          throw Error(`Graph is malformed, node ${r} has multiple parents, ${backedges.get(r)} and ${d} at least`);
        }
        backedges.set(r, d);
      }
    }

    let loops = 0;
    // This is not a fast approach, but it's simple enough. Optimize if needed.
    allowedNodegroups.set(root, true);
    while (loops < MAX_GRAPH_DEPTH) {
      const unrooted = [...allowedNodegroups.entries()].filter(([_, rooted]: [string, boolean]) => !rooted);
      if (unrooted.length === 0) {
        break;
      }
      for (const [ng] of unrooted) {
        if (ng === root) {
          continue;
        }
        const next = backedges.get(ng);
        if (!next) {
          throw Error(`Graph does not have a parent for ${ng}`);
        }
        allowedNodegroups.set(ng, true);
        if (!allowedNodegroups.has(next)) {
          allowedNodegroups.set(next, false);
        }
      }
      loops += 1;
    }

    if (loops >= MAX_GRAPH_DEPTH) {
      throw Error("Hit edge traversal limit when pruning, is the graph well-formed without cycles?")
    }

    const allowedNodes = new Set([...this.getNodeObjects().values()].filter((node: StaticNode) => {
      return (node.nodegroup_id && allowedNodegroups.get(node.nodegroup_id)) || node.nodeid === root;
    }).map((node: StaticNode) => node.nodeid));

    graph.cards = (graph.cards || []).filter((card: StaticCard) => allowedNodegroups.get(card.nodegroup_id));
    graph.cards_x_nodes_x_widgets = (graph.cards_x_nodes_x_widgets || []).filter((card: StaticCardsXNodesXWidgets) => allowedNodes.has(card.node_id));
    graph.edges = (graph.edges || []).filter((edge: StaticEdge) => (edge.domainnode_id === root || allowedNodes.has(edge.domainnode_id)) && allowedNodes.has(edge.rangenode_id));
    graph.nodegroups = (graph.nodegroups || []).filter((ng: StaticNodegroup) => allowedNodegroups.has(ng.nodegroupid));
    graph.nodes = (graph.nodes || []).filter((node: StaticNode) => allowedNodes.has(node.nodeid));

    // At this point, every originally-allowed nodegroup has an allowed parent, up to the root.
    if (Array.isArray(keepFunctions) && graph.functions_x_graphs) {
      graph.functions_x_graphs = graph.functions_x_graphs.filter((fxg: StaticFunctionsXGraphs) => keepFunctions.includes(fxg.function_id));
    } else {
      graph.functions_x_graphs = [];
    }

    // Persist the modified graph back to the WASM wrapper
    this.graph = graph;
  }

  async all(params: { limit?: number; lazy?: boolean; pruneTiles?: boolean } | undefined = undefined): Promise<Array<RIVM>> {
    const paramObj = params || { limit: undefined, lazy: undefined, pruneTiles: undefined };
    const promises = [];
    for await (const resource of this.iterAll(paramObj)) {
      promises.push(resource);
    }
    return Promise.all(promises);
  }

  async* resourceGenerator(staticResources: AsyncIterable<StaticResource, RIVM, unknown>, lazy: boolean=false, pruneTiles: boolean = true) {
    for await (const staticResource of staticResources) {
      yield this.fromStaticResource(staticResource, lazy, pruneTiles);
    }
  }

  async* iterAll(params: { limit?: number; lazy?: boolean; pruneTiles?: boolean }): AsyncGenerator<RIVM> {
    const pruneTiles = params.pruneTiles !== undefined ? params.pruneTiles : true;
    yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy, pruneTiles);
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

  async find(id: string, lazy: boolean = true, pruneTiles: boolean = true): Promise<RIVM> {
    const rivm = await this.findStatic(id);
    const x = this.fromStaticResource(rivm, lazy, pruneTiles);
    return x;
  }

  // Phase 4h: Simplified to boolean-only (removed CheckPermission callback)
  setPermittedNodegroups(permissions: Map<string, boolean>) {
    const nodegroups = this.getNodegroupObjects();
    const nodes = this.getNodeObjectsByAlias();
    this.permittedNodegroups = new Map([...permissions].map(([key, value]): [key: string, value: boolean] => {
      const k = key ?? '';  // Convert null/undefined to empty string
      if (!(typeof value === "boolean")) {
        console.error("For now, Rust cannot handle JS callbacks for permissions - setting to false for", key);
        value = false;
      }
      if (nodegroups.has(k) || k === '') {
        return [k, value];  // Use normalized key (not original)
      } else {
        const node = nodes.get(k);
        if (node) {
          // The nodeid is the nodegroup ID of the children, but may not be the nodegroup ID of
          // the semantic node itself.
          return [node.nodeid ?? '', value];  // Ensure nodeid is not null
        } else {
          throw Error(`Could not find ${key} in nodegroups for permissions`);
        }
      }
    }));
  }

  // Defaults to visible, which helps reduce the risk of false sense of security
  // from front-end filtering masking the presence of data transferred to it.
  // Phase 4h: Simplified to boolean-only (removed CheckPermission callback)
  getPermittedNodegroups(): Map<string, boolean> {
    if (!this.permittedNodegroups) {
      const permissions = new Map([...this.getNodegroupObjects()].map(
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

  // Phase 4h: Simplified - no callback, just boolean lookup
  isNodegroupPermitted(nodegroupId: string, tile: StaticTile | null): boolean {
    const permitted = this.getPermittedNodegroups().get(nodegroupId ?? '');
    if (!permitted) {
      return false;
    }
    if (permitted === true) {
      return true;
    }
    throw Error(`Ambiguous permission state: ${permitted} for nodegroup ${nodegroupId}`);
  }

  makeInstance(id: string, resource: StaticResource | null, pruneTiles: boolean = true, lazy: boolean = false): RIVM {
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
    pruneTiles: boolean = true
  ): Promise<RIVM> {
    const wkri: RIVM = this.makeInstance(
      resource.resourceinstance.resourceinstanceid,
      resource,
      pruneTiles,
      lazy
    );

    if (!wkri.$) {
      throw Error("Could not load resource from static definition");
    }

    const pop = wkri.$.populate(lazy).then(() => wkri);
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
  wkrm: WKRM,
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
  graphs: Map<string, ResourceModelWrapper<any>>;
  wkrms: Map<string, WKRM>;

  constructor(archesClient: ArchesClient) {
    this.archesClient = archesClient;
    this.graphs = new Map<string, ResourceModelWrapper<any>>();
    this.wkrms = new Map<string, WKRM>();
  }

  async initialize(configurationOptions: ConfigurationOptions | undefined = undefined) {
    if (this._initialized) {
      return;
    }
    if (configurationOptions === undefined) {
      configurationOptions = new ConfigurationOptions();
    }
    const graphJsons: GraphResult = await this.archesClient.getGraphs();

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
      const wkrm = new WKRM(meta);
      this.wkrms.set(wkrm.modelClassName, wkrm);
    });
    if (configurationOptions.eagerLoadGraphs) {
      await Promise.all(graphs.map(([g]) => this.loadGraph(g, configurationOptions.defaultAllowAllNodegroups)));
    }

    this._initialized = true;
  }

  async loadGraph<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string, defaultAllow: boolean=false): Promise<ResourceModelWrapper<RIVM>> {
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

    const bodyJson = await this.archesClient.getGraph(wkrm.meta);
    if (!bodyJson) {
      throw Error(`Could not load graph ${wkrm.graphId}`);
    }

    const graph = new StaticGraph(bodyJson);

    let model: ResourceInstanceViewModelConstructor<RIVM>;
    if (typeof modelClass == 'string') {
      modelClassName = modelClass;
      model = makeResourceModelWrapper<RIVM>(undefined, wkrm, graph, defaultAllow);
    } else {
      modelClassName = modelClass.name;
      model = makeResourceModelWrapper<RIVM>(modelClass, wkrm, graph, defaultAllow);
    }
    console.log("Made resource model wrapper for", graph.graphid);

    this.graphs.set(graph.graphid, model.prototype.__);
    return model.prototype.__;
  }

  async get<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string, defaultAllow: boolean=false): Promise<ResourceModelWrapper<RIVM>> {
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

  async getResource<T extends IRIVM<T>>(resourceId: string, lazy: boolean = true, pruneTiles: boolean = true): Promise<T> {
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

export { GraphManager, graphManager, ArchesClientRemote, staticStore, WKRM, WASMResourceModelWrapper, ResourceModelWrapper, GraphMutator };
