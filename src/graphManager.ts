import { archesClient, ArchesClient, ArchesClientRemote } from "./client.ts";
import { staticStore } from "./staticStore.ts"
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
import { ReferenceDataManager } from "./rdm";
import { makePseudoCls } from "./pseudos.ts";
import { ResourceInstanceViewModel, ValueList } from "./viewModels.ts";
import { IRIVM, IStringKeyedObject, IInstanceWrapper, IViewModel } from "./interfaces";
import { AttrPromise } from "./utils";

class WKRM {
  modelName: string;
  modelClassName: string;
  graphId: string;

  constructor(modelName: string, graphId: string) {
    this.modelName = modelName;
    this.graphId = graphId;
    this.modelClassName = modelName
      .replace(/\s(.)/g, (c) => c.toUpperCase())
      .replace(/\s/g, "");
  }
}

class ConfigurationOptions {
  graphs: Array<string> | null;

  constructor() {
    this.graphs = null;
  }
}

class ResourceInstanceWrapper<RIVM extends ResourceInstanceViewModel> implements IInstanceWrapper {
  wkri: RIVM;
  model: ResourceModelWrapper<RIVM>;

  resource: StaticResource | null | false ;
  valueList: ValueList;

  constructor(
    wkri: RIVM,
    model: ResourceModelWrapper<RIVM>,
    resource: StaticResource | null | false, // False to disable dynamic resource-loading
  ) {
    this.wkri = wkri;
    this.model = model;
    this.resource = resource;
    this.valueList = new ValueList<RIVM>(new Map<string, any>(), this);
  }

  async keys() {
    return (await this.getRoot()).keys();
  }

  async values() {
    return (await this.getRoot()).values();
  }

  async entries() {
    return (await this.getRoot()).entries();
  }

  async getOrmAttribute(key: string): AttrPromise<IViewModel> | IViewModel {
    if (this.resource === null) {
      this.resource = await this.model.find(this.wkri.id);
      await this.populate(true);
    }

    // TODO remapping
    const root = await this.getRoot();
    if (root) {
      const value = root.getValue();
      return value[key];
    } else {
      throw Error(`Tried to get ${key} on ${this}, which has no root`);
    }
  }

  async getRoot() {
    const values = this.valueList;
    const node = this.model.getRootNode();
    if (node) {
      let value;
      const alias = node.alias;
      if (!alias) {
        throw Error(`Alias missing for node ${node.nodeid}`);
      }
      await values.setDefault(alias, []);
      const nodeValues = await values.get(alias);

      if (nodeValues.length > 1) {
        throw Error("Cannot have multiple root tiles");
      } else if (nodeValues.length == 1) {
        value = nodeValues[0];
      } else {
        value = this.model.makePseudoCls(alias, false, null, this.wkri);
        values.set(alias, [value]);
      }
      return value;
    }
  }

  setOrmAttribute(key: string, value: any) {
    // TODO remapping
    const root = this.getRoot();
    if (root) {
      root.value[key] = value;
    } else {
      throw Error(`Tried to set ${key} on ${self}, which has no root`);
    }
  }

  async ensureNodegroup(
    allValues: Map<string, any>,
    node: StaticNode,
    nodegroupId: string,
    nodeObjs: Map<string, StaticNode>,
    nodegroupObjs: Map<string, StaticNodegroup>,
    edges: Map<string, string[]>,
    addIfMissing: boolean,
    tiles: StaticTile[] | null,
    doImpliedNodegroups: boolean = true
  ): Promise<[Map<string, any>, Map<string, StaticNode>]> {
    if (!node) {
      return allValues;
    }
    const alias: string = node.alias || "";
    const impliedNodegroups: Map<string, StaticNode> = new Map();
    const value = node && (await allValues.get(alias));

    let newAllValues = allValues;

    if (value === false || (addIfMissing && value === undefined)) {
      if (newAllValues.has(alias)) {
        newAllValues.delete(alias);
      }
      let nodegroupTiles: (StaticTile | null)[];
      if (tiles === null) {
        nodegroupTiles = [];
        console.error("Tiles must be provided and cannot be lazy-loaded yet");
      } else {
        nodegroupTiles = tiles.filter(
          (tile) => tile.nodegroup_id == nodegroupId,
        );
        if (nodegroupTiles.length == 0 && addIfMissing) {
          nodegroupTiles = [null];
        }
        const rgValues = await this.valuesFromResourceNodegroup(
          newAllValues,
          nodegroupTiles,
          nodegroupId,
          nodeObjs,
          edges,
        );
        const newValues = rgValues[0];
        const newImpliedNodegroups: Map<string, any> = rgValues[1];

        [...newValues.entries()].forEach((entry) => {
          newAllValues.set(entry[0], entry[1]);
        });
        [...newImpliedNodegroups.entries()].forEach(([k, v]) => {
          impliedNodegroups.set(k, v);
        });
      }
    }

    // RMV double-check against Python logic
    if (doImpliedNodegroups) {
      for (const [nodegroupId, node] of [...impliedNodegroups.entries()]) {
        const [impliedValues] = await this.ensureNodegroup(
          newAllValues,
          node,
          nodegroupId,
          nodeObjs,
          nodegroupObjs,
          edges,
          true,
          tiles, // RMV different from Python
          true
        );
        newAllValues = impliedValues;
      }
      impliedNodegroups.clear();
    }

    return [newAllValues, impliedNodegroups];
  }

  async populate(lazy: boolean) {
    const nodeObjs = this.model.getNodeObjects();
    const nodegroupObjs = this.model.getNodegroupObjects();
    const edges = this.model.getEdges();
    const allValues: Map<string, any> = new Map(
      [...nodegroupObjs.keys()].map((id: string) => {
        const node = nodeObjs.get(id);
        if (!node) {
          throw Error(`Could not find node for nodegroup ${id}`);
        }
        return [node.alias || "", false];
      }),
    );
    const rootNode = this.model.getRootNode();

    if (rootNode.alias === null) {
      throw Error("Cannot populate a model with no proper root node");
    }

    allValues.set(rootNode.alias, false);

    if (!lazy && this.resource) {
      const tiles = this.resource.tiles;
      let impliedNodegroups = new Map<string, StaticNode>();
      for (const [ng, _] of nodegroupObjs) {
        const [values, newImpliedNodegroups] = await this.ensureNodegroup(
          allValues,
          nodeObjs.get(ng) || rootNode, // should be the only missing ID
          ng,
          nodeObjs,
          nodegroupObjs,
          edges,
          true, // RMV: check vs python
          tiles,
          false
        );
        for (const [key, value] of [...values.entries()]) {
          allValues.set(key, value);
        }

        for (const [impliedNodegroup, impliedNode] of [...newImpliedNodegroups.entries()]) {
          impliedNodegroups.set(impliedNodegroup, impliedNode);
        }
        impliedNodegroups.delete(ng);
      }

      while (impliedNodegroups.size) {
        const newImpliedNodegroups = new Map<string, StaticNode>();
        for (const [nodegroupId, impliedNode] of [...impliedNodegroups.entries()]) {
          const currentValue = allValues.get(impliedNode.nodeid);
          if (currentValue === false || currentValue === undefined) {
            const [impliedValues, newImpliedNodegroups] = await this.ensureNodegroup(
              allValues,
              impliedNode,
              nodegroupId,
              nodeObjs,
              nodegroupObjs,
              edges,
              true,
              tiles, // RMV different from Python
              true
            );
            for (const [impliedNodegroup, impliedNode] of [...newImpliedNodegroups.entries()]) {
              newImpliedNodegroups.set(impliedNodegroup, impliedNode);
            }
            for (const [key, value] of [...impliedValues.entries()]) {
              allValues.set(key, value);
            }
          }
        }
        impliedNodegroups = newImpliedNodegroups;
      }
    }

    this.valueList = new ValueList(
      allValues,
      this,
      this.resource ? this.resource.tiles : null,
    );
  }

  async valuesFromResourceNodegroup(
    existingValues: Map<string, any>,
    nodegroupTiles: StaticTile[],
    nodegroupId: string,
    nodeObjs: Map<string, StaticNode>,
    edges: Map<string, string[]>,
  ) {
    const allValues = new Map<string, any>();

    const impliedNodegroups = new Map<string, StaticNode>();
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
        console.error("Existing:", existing);
        throw Error(`Tried to load node twice: ${key}`);
      }
      if (!allValues.has(key)) {
        allValues.set(key, []);
      }
      const pseudoNode = this.model.makePseudoCls(key, false, tile, this.wkri);
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
            : domainNode.nodeid;
          if (toAdd && toAdd !== nodegroupId) {
            impliedNodegroups.set(toAdd, domainNode);
          }
          if (domainNode.nodegroup_id && domainNode.nodegroup_id !== domainNode.nodeid && domainNode.nodegroup_id === node.nodegroup_id && tileid && !impliedNodes.has(domainNode.nodeid + tileid)) {
            impliedNodes.set(domainNode.nodeid + tileid, [domainNode, tile]);
          }
          break;
        }
      }
      if (Array.isArray(pseudoNode)) {
        const value = allValues.get(key);
        if (value !== undefined && value !== false) {
          for (const pseudoNodeList of allValues.get(key)) {
            if (!Array.isArray(pseudoNodeList)) {
              throw Error(`Should be all lists not ${typeof pseudoNodeList}`);
            }

            if (pseudoNodeList.parentNode == pseudoNode.parentNode) {
              for (const ps of pseudoNode) {
                pseudoNodeList.push(ps);
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
            throw Error("Unknown node in nodegroup");
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

class ResourceModelWrapper<RIVM extends ResourceInstanceViewModel> {
  wkrm: WKRM;
  graph: StaticGraph;
  viewModelClass: typeof ResourceInstanceViewModel = ResourceInstanceViewModel;
  makePseudoCls: Function;

  constructor(wkrm: WKRM, graph: StaticGraph) {
    this.wkrm = wkrm;
    this.graph = graph;
    this.makePseudoCls = makePseudoCls.bind(this);
  }

  async all(params: { limit: number; lazy: boolean }): Promise<Array<RIVM>> {
    const promises = [];
    for await (const resource of this.iterAll(params)) {
      promises.push(resource);
    }
    return Promise.all(promises);
  }

  async* resourceGenerator(staticResources: AsyncIterable<StaticResource, RIVM, unknown>, lazy: boolean=false) {
    for await (const staticResource of staticResources) {
      yield this.fromStaticResource(staticResource, lazy);
    }
  }

  async* iterAll(params: { limit: number; lazy: boolean }): AsyncGenerator<RIVM> {
    yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy);
  }

  async find(id: string, lazy: boolean = true): Promise<RIVM> {
    const rivm = await staticStore.loadOne(id);
    return this.fromStaticResource(rivm, lazy);
  }

  getPermittedNodegroups(): Map<string, StaticNodegroup> {
    // TODO allow reducing
    return this.getNodegroupObjects();
  }

  makeInstance(id: string, resource: StaticResource | null) {
    // TODO: This line needs fixed.
    const instance: IRIVM = new this.viewModelClass(
      id,
      this.viewModelClass.prototype.__,
      (rivm: ResourceInstanceViewModel) =>
        new ResourceInstanceWrapper(rivm, this, resource),
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
      for (const [_, n] of this.getNodeObjects()) {
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

    const graph = graphManager.getGraph(this.wkrm.graphId);
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
    return rootNode;
  }

  fromStaticResource(
    resource: StaticResource,
    lazy: boolean = false,
  ): Promise<RIVM> {
    // TODO: implement lazy
    const wkri = this.makeInstance(
      resource.resourceinstance.resourceinstanceid,
      resource,
    );
    return wkri._.populate(lazy).then(() => wkri);
  }
}

function makeResourceModelWrapper(
  wkrm: WKRM,
  graph: StaticGraph,
): ResourceModelWrapper<any> {
  const viewModelClass: typeof ResourceInstanceViewModel = {
    [wkrm.modelClassName]: class extends ResourceInstanceViewModel {
      static _: ResourceInstanceWrapper<any>;
      static __: ResourceModelWrapper<any>;
      static async all(
        params: { limit: number | null; lazy: boolean } = {
          limit: null,
          lazy: false,
        },
      ): Promise<Array<typeof viewModelClass>> {
        return viewModelClass.prototype.__.all({
          limit: params.limit,
          lazy: params.lazy,
        });
      }
      static async find(
        id: string,
        lazy: boolean = false,
      ): Promise<Array<typeof viewModelClass>> {
        return viewModelClass.prototype.__.find(id, lazy);
      }
      constructor(
        id: string,
        wrapper: ResourceModelWrapper<any>,
        instanceWrapperFactory: (
          rivm: ResourceInstanceViewModel,
        ) => IInstanceWrapper,
      ) {
        super(id, wrapper, instanceWrapperFactory);
      }
    },
  }[wkrm.modelClassName];
  const wrapper = new ResourceModelWrapper<any>(wkrm, graph);
  wrapper.viewModelClass = viewModelClass;
  viewModelClass.prototype.__ = wrapper;
  return wrapper;
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

  async initialize(configurationOptions: ConfigurationOptions | undefined) {
    if (this._initialized) {
      return;
    }
    if (configurationOptions === undefined) {
      configurationOptions = new ConfigurationOptions();
    }
    const graphJsons: GraphResult = await this.archesClient.getGraphs();

    let graphs = Object.keys(graphJsons["models"]);
    if (configurationOptions.graphs !== null) {
      graphs = graphs.filter(
        (graphId: string) => configurationOptions.graphs.includes(graphId),
      );
    }

    await Promise.all(
      graphs.map(async (graphId: string) => {
        const bodyJson = await this.archesClient.getGraph(graphId);
        if (bodyJson) {
          const graph = new StaticGraph(bodyJson);
          const wkrm = new WKRM(graph.name.toString(), graph.graphid);
          this.wkrms.set(wkrm.modelClassName, wkrm);
          this.graphs.set(graph.graphid, makeResourceModelWrapper(wkrm, graph));
        }
      }),
    );

    this._initialized = true;
  }

  get(modelClassName: string) {
    // Initialize as a fallback
    this.initialize(undefined);
    const wkrm = this.wkrms.get(modelClassName);
    if (wkrm === undefined) {
      throw Error(`Cannot find model requested: ${modelClassName}`);
    }
    const wrapper = this.graphs.get(wkrm.graphId);
    if (wrapper === undefined) {
      throw Error(`Cannot find graph requested: ${modelClassName}`);
    }
    return wrapper.viewModelClass;
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

export { GraphManager, graphManager, ArchesClientRemote, staticStore };
