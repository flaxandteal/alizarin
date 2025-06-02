import { GraphResult, archesClient, ArchesClient, ArchesClientRemote } from "./client.ts";
import { staticStore } from "./staticStore.ts"
import {
  StaticCard,
  StaticEdge,
  StaticCardsXNodesXWidgets,
  StaticFunctionsXGraphs,
  StaticTile,
  StaticGraph,
  StaticNode,
  StaticNodegroup,
  StaticResource,
  StaticResourceDescriptors,
  StaticGraphMeta,
} from "./static-types";
import { makePseudoCls, PseudoList } from "./pseudos.ts";
import { DEFAULT_LANGUAGE, ResourceInstanceViewModel, ValueList, viewContext, SemanticViewModel } from "./viewModels.ts";
import { CheckPermission, GetMeta, IRIVM, IStringKeyedObject, IPseudo, IInstanceWrapper, IViewModel, ResourceInstanceViewModelConstructor } from "./interfaces";
import { AttrPromise } from "./utils";

const MAX_GRAPH_DEPTH = 100;
const DESCRIPTOR_FUNCTION_ID = "60000000-0000-0000-0000-000000000001";

class WKRM {
  modelName: string;
  modelClassName: string;
  graphId: string;
  meta: StaticGraphMeta;

  constructor(meta: StaticGraphMeta) {
    let name: {[lang: string]: string} | string | undefined;
    if (meta.name instanceof Object) {
      name = meta.name[DEFAULT_LANGUAGE].toString();
    } else {
      name = meta.name;
    }
    this.modelName = name || "Unnamed";
    this.graphId = meta.graphid;
    this.modelClassName = (meta.slug || this.modelName)
      .replace(/[_-]/g, " ")
      .replace(/\s(.)/g, (c: string) => c.toUpperCase())
      .replace(/\s/g, "");
    this.modelClassName = this.modelClassName[0].toUpperCase() + this.modelClassName.slice(1);
    this.meta = meta;
  }
}

class ConfigurationOptions {
  graphs: Array<string> | null | boolean;

  constructor() {
    this.graphs = null;
  }
}

class ResourceInstanceWrapper<RIVM extends IRIVM<RIVM>> implements IInstanceWrapper<RIVM> {
  wkri: RIVM;
  model: ResourceModelWrapper<RIVM>;

  resource: StaticResource | null | false ;
  valueList: ValueList<RIVM>;
  cache: {[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined;
  scopes?: string[];
  metadata?: {[key: string]: string};

  constructor(
    wkri: RIVM,
    model: ResourceModelWrapper<RIVM>,
    resource: StaticResource | null | false, // False to disable dynamic resource-loading
  ) {
    this.wkri = wkri;
    this.model = model;
    if (resource) {
      this.model.stripTiles(resource);
    }
    this.resource = resource;
    this.valueList = new ValueList(new Map<string, any>(), new Map<string, boolean>(), this, []);
    this.cache = resource ? resource.__cache : undefined;
    this.scopes = resource ? resource.__scopes : undefined;
    this.metadata = resource ? resource.metadata : undefined;
  }

  async loadNodes(aliases: Array<string>): Promise<void> {
    for (const key of aliases) {
      await this.valueList.retrieve(key);
    }
  }

  async getName(update: boolean = false) {
    let resourceName = this.resource && this.resource.resourceinstance.name;
    if (update || !resourceName) {
      const descriptors = await this.getDescriptors(update);
      resourceName = (descriptors && descriptors.name) || resourceName || '<Unnamed>';
      if (this.resource && this.resource.resourceinstance) {
        this.resource.resourceinstance.name = resourceName;
      }
    }
    return resourceName;
  }

  async getDescriptors(update: boolean = false) {
    let descriptors = this.resource && this.resource.resourceinstance.descriptors;
    if (update || !descriptors || descriptors.isEmpty()) {
      descriptors = new StaticResourceDescriptors();
      let descriptorConfig;
      if (this.model.graph.functions_x_graphs) {
        const descriptorNode = this.model.graph.functions_x_graphs.find(node => node.function_id === DESCRIPTOR_FUNCTION_ID);
        if (descriptorNode) {
          descriptorConfig = descriptorNode.config;
        }
      }
      const nodes = this.model.getNodeObjects();
      if (descriptorConfig) {
        for (const [descriptor, config] of Object.entries(descriptorConfig.descriptor_types)) {
          const semanticNode = nodes.get(config.nodegroup_id);
          let description = config.string_template;
          if (!description) {
            continue;
          }
          const requestedNodes = [...description.match(/<[A-Za-z _-]*>/g)];
          const relevantNodes = [...nodes.values()].filter(node => node.nodegroup_id === config.nodegroup_id && requestedNodes.includes(`<${node.name}>`)).map(node => [node.name, node.alias || '']);
          let relevantValues;
          // First try and see if we can find all of these on one tile, for consistency.
          if (semanticNode) {
            let semanticValue = await (await this.valueList.retrieve(semanticNode.alias || ''))[0];
            if (semanticValue instanceof PseudoList) {
              semanticValue = await semanticValue[0];
            } else {
              semanticValue = await semanticValue.getValue();
            }
            if (semanticValue) {
              if (semanticValue) {
                relevantValues = await Promise.all(relevantNodes.filter(([_, alias]) => semanticValue.__has(alias)).map(([name, alias]) => semanticValue[alias].then((value: IViewModel) => [name, value])));
              }
            }
          }
          if (relevantValues) {
            description = relevantValues.reduce((desc, [name, value]) => value ? desc.replace(`<${name}>`, value) : desc, description);
          }
          relevantValues = await Promise.all(relevantNodes.map(([name, alias]) => this.valueList.retrieve(alias).then(values => [name, values ? values[0] : undefined])));
          if (relevantValues) {
            description = relevantValues.reduce((desc, [name, value]) => value ? desc.replace(`<${name}>`, value) : desc, description);
          }
          descriptors[descriptor] = description;
        }
      }
    }
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
    const child = makePseudoCls(
      this.model,
      key,
      false,
      (!childNode.is_collector && childNode.nodegroup_id === node.nodegroup_id) ? tile : null, // Does it share a tile
      this.wkri,
    );

    const valueList: ValueList<any> = this.valueList;
    valueList.setDefault(key, []).then((val: Array<any>) => val.push(child));
    return child;
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
      throw Error(`Tried to get root on ${this}, which has no root`);
    }
    return value;
  }

  getOrmAttribute(key: string): AttrPromise<IViewModel> {
    let promise: Promise<void>;
    if (this.resource === null) {
      promise = this.model.findStatic(this.wkri.id).then(resource => {
        this.resource = resource;
      }).then(() => this.populate(true));
    } else {
      promise = new Promise((resolve) => { resolve(); });
    }

    // TODO remapping
    return promise.then(() => this.getRootViewModel()).then(root => root[key]);
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
        value = makePseudoCls(this.model, alias, false, null, this.wkri);
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

  async ensureNodegroup(
    allValues: Map<string, any>,
    allNodegroups: Map<string, boolean | Promise<any>>,
    nodegroupId: string,
    nodeObjs: Map<string, StaticNode>,
    nodegroupObjs: Map<string, StaticNodegroup>,
    edges: Map<string, string[]>,
    addIfMissing: boolean,
    tiles: StaticTile[] | null,
    doImpliedNodegroups: boolean = true
  ): Promise<[Map<string, any>, Set<string>]> {
    const impliedNodegroups: Set<string> = new Set();
    const sentinel = allNodegroups.get(nodegroupId); // no action required if pending
    let newValues = new Map();

    if (sentinel === false || (addIfMissing && sentinel === undefined)) {
      [...nodeObjs.values()].filter((node: StaticNode) => {
        return node.nodegroup_id === nodegroupId;
      }).forEach((node: StaticNode) => allValues.delete(node.alias || ''));
      let nodegroupTiles: (StaticTile | null)[];
      if (tiles === null) {
        nodegroupTiles = [];
        console.error("Tiles must be provided and cannot be lazy-loaded yet");
      } else {
        nodegroupTiles = tiles.filter(
          (tile) => tile.nodegroup_id == nodegroupId && this.model.isNodegroupPermitted(nodegroupId, tile)
        );
        if (nodegroupTiles.length == 0 && addIfMissing) {
          nodegroupTiles = [null];
        }
        const rgValues = await this.valuesFromResourceNodegroup(
          allValues,
          nodegroupTiles,
          nodegroupId,
          nodeObjs,
          edges,
        );
        newValues = rgValues[0];
        const newImpliedNodegroups: Set<string> = rgValues[1];

        [...newValues.entries()].forEach((entry) => {
          if (entry[1] !== undefined) {
            allValues.set(entry[0], entry[1]);
          }
        });
        [...newImpliedNodegroups].forEach((v) => {
          impliedNodegroups.add(v);
        });
        allNodegroups.set(nodegroupId, true);
      }
    }

    // RMV double-check against Python logic
    if (doImpliedNodegroups) {
      for (const nodegroupId of [...impliedNodegroups]) {
        // TODO: why are we not keeping implied nodegroups?
        const [impliedValues] = await this.ensureNodegroup(
          allValues,
          allNodegroups,
          nodegroupId,
          nodeObjs,
          nodegroupObjs,
          edges,
          true,
          tiles, // RMV different from Python
          true
        );
        for (const [key, value] of impliedValues) {
          newValues.set(key, value);
        }
      }
      impliedNodegroups.clear();
    }

    return [newValues, impliedNodegroups];
  }

  async populate(lazy: boolean): Promise<void> {
    const nodeObjs = this.model.getNodeObjects();
    const nodegroupObjs = this.model.getNodegroupObjects();
    const edges = this.model.getEdges();
    // FIXME: this needs to be nodeObjs to ensure tiles
    // whose nodegroup node is in a different nodegroup
    // (e.g. children of designation_and_protection_timespan)
    // get loaded - however, just doing that drops performance
    // by half or two-thirds, so a less wasteful approach is needed.
    const allValues: Map<string, any> = new Map();
    const allNodegroups: Map<string, any> = new Map([...nodegroupObjs.keys()].map((id: string) => {
      return [id || "", false];
    }));
    //[...nodegroupObjs.keys()].map((id: string) => {
    //  const node = nodeObjs.get(id);
    //  if (!node) {
    //    throw Error(`Could not find node for nodegroup ${id}`);
    //  }
    //  allValues.set(node.alias || "", false);
    //});
    const rootNode = this.model.getRootNode();

    if (rootNode.alias === null) {
      throw Error("Cannot populate a model with no proper root node");
    }

    allValues.set(rootNode.alias, false);

    let tiles = null;
    if (!lazy && this.resource) {
      tiles = this.resource.tiles;
      let impliedNodegroups = new Set<string>();
      for (const [ng] of nodegroupObjs) {
        const [_, newImpliedNodegroups] = await this.ensureNodegroup(
          allValues,
          allNodegroups,
          ng,
          nodeObjs,
          nodegroupObjs,
          edges,
          true, // RMV: check vs python
          tiles,
          false
        );

        for (const impliedNodegroup of [...newImpliedNodegroups]) {
          impliedNodegroups.add(impliedNodegroup);
        }
        impliedNodegroups.delete(ng);
      }

      while (impliedNodegroups.size) {
        const newImpliedNodegroups = new Set<string>();
        for (const nodegroupId of [...impliedNodegroups]) {
          const currentValue = allNodegroups.get(nodegroupId);
          if (currentValue === false || currentValue === undefined) {
            const [_, newImpliedNodegroups] = await this.ensureNodegroup(
              allValues,
              allNodegroups,
              nodegroupId,
              nodeObjs,
              nodegroupObjs,
              edges,
              true,
              tiles, // RMV different from Python
              true
            );
            for (const impliedNodegroup of [...newImpliedNodegroups]) {
              newImpliedNodegroups.add(impliedNodegroup);
            }
          }
        }
        impliedNodegroups = newImpliedNodegroups;
      }
    } else if (this.resource) {
      this.model.stripTiles(this.resource);
    }

    this.valueList = new ValueList(
      allValues,
      allNodegroups,
      this,
      this.resource ? this.resource.tiles : null,
    );
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

  async valuesFromResourceNodegroup(
    existingValues: Map<string, any>,
    nodegroupTiles: (StaticTile | null)[],
    nodegroupId: string,
    nodeObjs: Map<string, StaticNode>,
    edges: Map<string, string[]>,
  ): Promise<[Map<string, any>, Set<string>]> {
    const allValues = new Map<string, any>();

    const impliedNodegroups = new Set<string>();
    const impliedNodes: Map<string, [StaticNode, StaticTile]> = new Map();

    const nodesUnseen = new Set(
      [...nodeObjs.values()]
        .filter((node) => node.nodegroup_id == nodegroupId)
        .map((node) => node.alias),
    );
    const tileNodesSeen: Set<[string, string]> = new Set();
    const _addPseudo = async (node: StaticNode, tile: StaticTile | null) => {
      const key = node.alias || "";
      nodesUnseen.delete(node.alias);
      const tileid = tile && tile.tileid;
      if (tileid) {
        tileNodesSeen.add([node.nodeid, tileid]);
      }
      let existing = existingValues.get(key);
      if (existing instanceof Promise) {
        existing = await existing;
      }
      if (existing !== false && existing !== undefined) {
        // This might be correct - confirm.
        // console.warn(`Tried to load node twice: ${key} (${node.nodeid}<${node.nodegroup_id})`, nodegroupId);
        allValues.set(key, existing);
      }
      if (!allValues.has(key)) {
        allValues.set(key, []);
      }
      const pseudoNode = makePseudoCls(this.model, key, false, tile, this.wkri);
      // We shouldn't have to take care of this case, as it should already
      // be included below.
      // if tile.parenttile_id:
      for (const [domain, ranges] of edges) {
        if (ranges.includes(node.nodeid)) {
          const domainNode = nodeObjs.get(domain);
          if (!domainNode) {
            throw Error("Edge error in graph");
          }
          const toAdd = domainNode.nodegroup_id
            ? domainNode.nodegroup_id
            : '';
          if (toAdd && toAdd !== nodegroupId) {
            impliedNodegroups.add(toAdd);
          }
          if (domainNode.nodegroup_id && tile && domainNode.nodegroup_id === tile.nodegroup_id && domainNode.nodegroup_id !== domainNode.nodeid && tileid && !impliedNodes.has(domainNode.nodeid + tileid)) {
            impliedNodes.set(domainNode.nodeid + tileid, [domainNode, tile]);
          }
          break;
        }
      }
      if (Array.isArray(pseudoNode)) {
        const value = allValues.get(key);
        if (value !== undefined && value !== false) {
          for (const pseudoList of allValues.get(key)) {
            if (!(pseudoList instanceof PseudoList) || !(pseudoNode instanceof PseudoList)) {
              throw Error(`Should be all lists not ${typeof pseudoList} and ${typeof pseudoNode}`);
            }

            if (pseudoList.parentNode == pseudoNode.parentNode) {
              for (const ps of pseudoNode) {
                pseudoList.push(ps);
              }
              return;
            }
          }
        }
      }
      allValues.get(key).push(pseudoNode);
    };

    for (const tile of nodegroupTiles) {
      const parentNode = nodeObjs.get(nodegroupId);
      if (parentNode === undefined) {
        continue;
      }
      if (!parentNode.nodegroup_id || parentNode.nodegroup_id == nodegroupId) {
        await _addPseudo(parentNode, tile);
      }

      if (tile) {
        const tileNodes = new Map();
        for (const [key, value] of [...tile.data.entries()]) {
          tileNodes.set(key, value);
        }

        // Semantic nodes in this tile should always have a pseudo-node
        [...nodeObjs.values()].filter((node: StaticNode) => {
          return node.nodegroup_id === nodegroupId && !tileNodes.get(node.nodeid) && node.datatype === 'semantic';
        }).forEach((node: StaticNode) => tileNodes.set(node.nodeid, {}));

        if (!tileNodes.has(tile.nodegroup_id)) {
          tileNodes.set(tile.nodegroup_id, {});
        }
        for (const [nodeid, nodeValue] of [...tileNodes.entries()]) {
          if (nodeid == nodegroupId) {
            // RMV is this correct?
            continue;
          }
          const node = nodeObjs.get(nodeid);
          if (!node) {
            throw Error(`Unknown node in nodegroup: ${nodeid} in ${nodegroupId}`);
          }
          if (nodeValue !== null) {
            await _addPseudo(node, tile);
          }
        }
      }
    }
    while (impliedNodes.size > 0) {
      const value = impliedNodes.entries().next().value;
      if (value) {
        const [node, tile] = value[1];
        // If nodeid!=nodegroup_id, then it has its own tile.
        if (tile.tileid && !tileNodesSeen.has([node.nodeid, tile.tileid])) {
          await _addPseudo(node, tile);
        }
        impliedNodes.delete(value[0]);
      }
    }
    // Remove any "unloaded" sentinel values so we do not try and
    // reload this nodegroup.
    [...nodesUnseen.keys()].forEach((nodeUnseen) => {
      // if (allValues.get(nodeUnseen) === false) { // TODO: work out why this is not necessary
      if (nodeUnseen) {
        allValues.set(nodeUnseen, undefined);
      }
      // }
    });
    return [allValues, impliedNodegroups];
  }
}

class ResourceModelWrapper<RIVM extends IRIVM<RIVM>> {
  wkrm: WKRM;
  graph: StaticGraph;
  viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>;
  permittedNodegroups?: Map<string | null, boolean | CheckPermission>;

  constructor(wkrm: WKRM, graph: StaticGraph, viewModelClass?: ResourceInstanceViewModelConstructor<RIVM>) {
    this.wkrm = wkrm;
    this.graph = graph;
    this.viewModelClass = viewModelClass;
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
    const allNodegroups = this.getNodegroupObjects();
    const root = this.graph.root.nodeid;
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

    this.graph.cards = (this.graph.cards || []).filter((card: StaticCard) => allowedNodegroups.get(card.nodegroup_id));
    this.graph.cards_x_nodes_x_widgets = (this.graph.cards_x_nodes_x_widgets || []).filter((card: StaticCardsXNodesXWidgets) => allowedNodes.has(card.node_id));
    this.graph.edges = (this.graph.edges || []).filter((edge: StaticEdge) => (edge.domainnode_id === root || allowedNodes.has(edge.domainnode_id)) && allowedNodes.has(edge.rangenode_id));
    this.graph.nodegroups = (this.graph.nodegroups || []).filter((ng: StaticNodegroup) => allowedNodegroups.has(ng.nodegroupid));
    this.graph.nodes = (this.graph.nodes || []).filter((node: StaticNode) => allowedNodes.has(node.nodeid));

    // At this point, every originally-allowed nodegroup has an allowed parent, up to the root.
    if (Array.isArray(keepFunctions) && this.graph.functions_x_graphs) {
      this.graph.functions_x_graphs = this.graph.functions_x_graphs.filter((fxg: StaticFunctionsXGraphs) => keepFunctions.includes(fxg.function_id));
    } else {
      this.graph.functions_x_graphs = [];
    }
  }

  exportGraph(): StaticGraph {
    const graph = this.graph;
    return new StaticGraph({
      author: graph.author,
      cards: graph.cards,
      cards_x_nodes_x_widgets: graph.cards_x_nodes_x_widgets,
      color: graph.color,
      config: graph.config,
      deploymentdate: graph.deploymentdate,
      deploymentfile: graph.deploymentfile,
      description: graph.description,
      edges: graph.edges,
      functions_x_graphs: graph.functions_x_graphs,
      graphid: graph.graphid,
      iconclass: graph.iconclass,
      is_editable: graph.is_editable,
      isresource: graph.isresource,
      jsonldcontext: graph.jsonldcontext,
      name: graph.name,
      nodegroups: graph.nodegroups,
      nodes: graph.nodes,
      ontology_id: graph.ontology_id,
      publication: graph.publication,
      relatable_resource_model_ids: graph.relatable_resource_model_ids,
      resource_2_resource_constraints: graph.resource_2_resource_constraints,
      root: graph.root,
      slug: graph.slug,
      subtitle: graph.subtitle,
      template_id: graph.template_id,
      version: graph.version,
    });
  }

  async all(params: { limit?: number; lazy?: boolean } | undefined = undefined): Promise<Array<RIVM>> {
    const paramObj = params || { limit: undefined, lazy: undefined };
    const promises = [];
    for await (const resource of this.iterAll(paramObj)) {
      promises.push(resource);
    }
    return Promise.all(promises);
  }

  stripTiles(resource: StaticResource) {
    if (resource.tiles) {
      const nodes = this.getNodeObjects();
      resource.tiles = resource.tiles.filter(tile => {
        const node = nodes.get(tile.nodegroup_id);
        if (!node) {
          throw Error(`Tile ${tile.tileid} has nodegroup ${tile.nodegroup_id} that is not on the model ${this.graph.graphid}`);
        }
        return this.isNodegroupPermitted(tile.nodegroup_id || '', tile);
      });
    }
  }

  async* resourceGenerator(staticResources: AsyncIterable<StaticResource, RIVM, unknown>, lazy: boolean=false) {
    for await (const staticResource of staticResources) {
      yield this.fromStaticResource(staticResource, lazy);
    }
  }

  async* iterAll(params: { limit?: number; lazy?: boolean }): AsyncGenerator<RIVM> {
    yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy);
  }

  async findStatic(id: string): Promise<StaticResource> {
    return await staticStore.loadOne(id);
  }

  async find(id: string, lazy: boolean = true): Promise<RIVM> {
    const rivm = await this.findStatic(id);
    return this.fromStaticResource(rivm, lazy);
  }

  setPermittedNodegroups(permissions: Map<string | null, boolean | CheckPermission>) {
    const nodegroups = this.getNodegroupObjects();
    const nodes = this.getNodeObjectsByAlias();
    const nodesById = this.getNodeObjects();
    this.permittedNodegroups = new Map([...permissions].map(([key, value]): [key: string | null, value: boolean | CheckPermission] => {
      const k = key || '';
      if (nodegroups.has(k) || k === '') {
        return [key, value];
      } else {
        const node = nodes.get(k);
        if (node) {
          // The nodeid is the nodegroup ID of the children, but may not be the nodegroup ID of
          // the semantic node itself.
          return [node.nodeid, value];
        } else {
          throw Error(`Could not find ${key} in nodegroups for permissions`);
        }
      }
    }));
  }

  // Defaults to visible, which helps reduce the risk of false sense of security
  // from front-end filtering masking the presence of data transferred to it.
  getPermittedNodegroups(): Map<string | null, boolean | CheckPermission> {
    if (!this.permittedNodegroups) {
      const permissions = new Map([...this.getNodegroupObjects()].map(
        ([k, _]: [k: string, _: StaticNodegroup]) => [k, true]
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

  isNodegroupPermitted(nodegroupId: string, tile: StaticTile | null): boolean {
    let permitted: boolean | CheckPermission | undefined = this.getPermittedNodegroups().get(nodegroupId);
    if (permitted && typeof permitted == 'function') {
      const nodes = this.getNodeObjectsByAlias();
      permitted = permitted(nodegroupId, tile, nodes);
    }
    if (!permitted) {
      return false;
    }
    if (permitted === true) {
      return true;
    }
    throw Error(`Ambiguous permission state: ${permitted} for nodegroup ${nodegroupId}`);
  }

  makeInstance(id: string, resource: StaticResource | null): RIVM {
    if (!this.viewModelClass) {
      throw Error(`Cannot instantiate without a viewModelClass in ${this.wkrm.modelClassName}`);
    }
    // TODO: This line needs fixed.
    const instance: RIVM = new this.viewModelClass(
      id,
      this.viewModelClass.prototype.__,
      (rivm: RIVM) =>
        new ResourceInstanceWrapper(rivm, this, resource),
      null
    );
    return instance;
  }

  edges: Map<string, string[]> | undefined;
  nodes: Map<string, StaticNode> | undefined;
  nodegroups: Map<string, StaticNodegroup> | undefined;
  nodesByAlias: Map<string, StaticNode> | undefined;

  getChildNodes(nodeId: string): Map<string, StaticNode> {
    const childNodes = new Map<string, StaticNode>();
    const edges = this.getEdges().get(nodeId);
    if (edges) {
      for (const [, n] of this.getNodeObjects()) {
        if (edges.includes(n.nodeid)) {
          if (n.alias) {
            childNodes.set(n.alias, n);
          }
        }
      }
    }
    return childNodes;
  }

  buildNodes() {
    if (this.nodes || this.nodegroups) {
      throw Error("Cache should never try and rebuild nodes when non-empty");
    }

    this.edges = new Map<string, string[]>();
    this.nodes = new Map<string, StaticNode>();
    this.nodegroups = new Map<string, StaticNodegroup>();

    const graph = this.graph ?? graphManager.getGraph(this.wkrm.graphId);
    if (!graph) {
      throw Error(`Could not find graph ${this.wkrm.graphId} for resource`);
    }
    const nodes = new Map(graph.nodes.map((node) => [node.nodeid, node]));
    const nodegroups = new Map(
      graph.nodes
        .filter((node) => node.nodegroup_id)
        .map((node) => [
          node.nodegroup_id || "",
          new StaticNodegroup({
            cardinality: "n",
            legacygroupid: null,
            nodegroupid: node.nodegroup_id || "",
            parentnodegroup_id: null,
          }),
        ]),
    );
    for (const nodegroup of graph.nodegroups) {
      nodegroups.set(nodegroup.nodegroupid, nodegroup);
    }

    const edgePairs = graph.edges.map((edge) => [
      edge.domainnode_id,
      edge.rangenode_id,
    ]);
    const edges: Map<string, string[]> = edgePairs.reduce((edges, dr) => {
      const range = edges.get(dr[0]) || [];
      range.push(dr[1]);
      edges.set(dr[0], range);
      return edges;
    }, new Map<string, string[]>());

    this.nodes = nodes;
    this.nodegroups = nodegroups;
    this.edges = edges;
    this.nodesByAlias = new Map(
      [...nodes.values()].map((node) => [node.alias || "", node]),
    );
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

  getNodeObjects(): Map<string, StaticNode> {
    if (!this.nodes) {
      this.buildNodes();
    }
    if (!this.nodes) {
      throw Error("Could not build nodes");
    }
    return this.nodes;
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

  getRootNode(): StaticNode {
    const nodes = this.getNodeObjects();
    const rootNode = [...nodes.values()].find((n) => !n.nodegroup_id);
    if (!rootNode) {
      throw Error(
        `COULD NOT FIND ROOT NODE FOR ${this.wkrm.modelClassName}. Does the graph ${this.graph.graphid} still exist?`,
      );
    }
    rootNode.alias = rootNode.alias || '';
    return rootNode;
  }

  fromStaticResource(
    resource: StaticResource,
    lazy: boolean = false,
  ): Promise<RIVM> {
    // TODO: implement lazy
    const wkri: RIVM = this.makeInstance(
      resource.resourceinstance.resourceinstanceid,
      resource,
    );

    if (!wkri.$) {
      throw Error("Could not load resource from static definition");
    }

    return wkri.$.populate(lazy).then(() => wkri);
  }
}

function makeResourceModelWrapper<T extends IRIVM<T>>(
  viewModelClass: ResourceInstanceViewModelConstructor<T> | undefined,
  wkrm: WKRM,
  graph: StaticGraph,
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

  const wrapper = new ResourceModelWrapper<T>(wkrm, graph, vmc);
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

    Object.entries(graphJsons["models"]).forEach(([graphId, meta]: [string, StaticGraphMeta]) => {
      meta.graphid = meta.graphid || graphId;
      const wkrm = new WKRM(meta);
      this.wkrms.set(wkrm.modelClassName, wkrm);
    });
    let graphs: Array<string> = Object.keys(graphJsons["models"]);
    const allowedGraphs = configurationOptions.graphs;
    if (allowedGraphs !== null) {
      if (allowedGraphs === false) {
        throw Error("No current meaning of allowedGraphs === false");
      } else if (allowedGraphs !== true) {
        graphs = graphs.filter(
          (graphId: string) => allowedGraphs.includes(graphId),
        );
      }
      await Promise.all(graphs.map(g => this.loadGraph(g)));
    }

    this._initialized = true;
  }

  async loadGraph<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string): Promise<ResourceModelWrapper<RIVM>> {
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
      model = makeResourceModelWrapper<RIVM>(undefined, wkrm, graph);
    } else {
      modelClassName = modelClass.name;
      model = makeResourceModelWrapper<RIVM>(modelClass, wkrm, graph);
    }

    this.graphs.set(graph.graphid, model.prototype.__);
    return model.prototype.__;
  }

  async get<RIVM extends IRIVM<RIVM>>(modelClass: ResourceInstanceViewModelConstructor<RIVM> | string): Promise<ResourceModelWrapper<RIVM>> {
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
      return this.loadGraph(modelClass);
    }
    return wrapper;
  }

  async getResource<T extends IRIVM<T>>(resourceId: string, lazy: boolean = true): Promise<T> {
    const rivm = await staticStore.loadOne(resourceId);
    let graph = this.graphs.get(rivm.resourceinstance.graph_id);
    if (!graph) {
      graph = await this.loadGraph(rivm.resourceinstance.graph_id);
      if (!graph) {
        throw Error(`Graph not found for resource ${resourceId}`);
      }
    }
    return graph.fromStaticResource(rivm, lazy);
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

export { GraphManager, graphManager, ArchesClientRemote, staticStore, WKRM, ResourceModelWrapper };
