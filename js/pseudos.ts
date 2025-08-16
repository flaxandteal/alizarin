import { StaticTile, StaticNode } from "./static-types";
import { ISemantic, IViewModel, IPseudo, IRIVM, IModelWrapper } from "./interfaces";
import { getViewModel } from "./viewModels";
import { AttrPromise } from "./utils";

class PseudoUnavailable implements IPseudo {
  parentNode: PseudoValue<any> | null = null;
  tile: null = null;
  node: StaticNode;
  isOuter: boolean = false;

  constructor(node: StaticNode) {
    this.node = node;
  }

  async forJson(): Promise<{[key: string]: any}[] | null> {
    return null;
  }

  describeField() {
    return "Unavailable field";
  }

  describeFieldGroup() {
    return "Unavailable field";
  }

  public getValue(): AttrPromise<null> {
    return new AttrPromise(resolve => resolve(null));
  }

  getLength() {
    return 0;
  }

  getChildren(_: boolean = false) {
    return [];
  }

  isIterable(): boolean {
    return false
  }
}

const ITERABLE_DATATYPES = [
  "concept-list",
  "resource-instance-list",
  "domain-value-list"
];

class PseudoValue<VM extends IViewModel> implements IPseudo {
  node: StaticNode;
  tile: StaticTile | null;
  value: any;
  parent: IRIVM<any> | null;
  parentNode: PseudoValue<any> | null;
  valueLoaded: boolean | undefined = false;
  datatype: string | null = null;
  originalTile: StaticTile | null;
  accessed: boolean;
  childNodes: Map<string, StaticNode>;
  isOuter: boolean = false;
  isInner: boolean = false;
  inner: PseudoValue<ISemantic> | null = null;
  independent: boolean;

  isIterable(): boolean {
    return this.datatype !== null && ITERABLE_DATATYPES.includes(this.datatype);
  }

  describeField() {
    let fieldName = this.node.name;
    if (this.parent && this.parent.__) {
      fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
    }
    return fieldName;
  }

  describeFieldGroup() {
    let fieldName = this.node.name;
    if (this.parent && this.node.nodegroup_id && this.parent.$) {
      const nodegroup = this.parent.$.model.getNodeObjects().get(this.node.nodegroup_id);
      if (nodegroup && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
      }
    }
    return fieldName;
  }

  constructor(
    node: StaticNode,
    tile: StaticTile | null,
    value: any,
    parent: IRIVM<any> | null,
    childNodes: Map<string, StaticNode>,
    inner: boolean | PseudoValue<ISemantic>,
  ) {
    this.node = node;
    this.tile = tile;
    this.independent = tile === null;
    if (!parent) {
      throw Error("Must have a parent or parent class for a pseudo-node");
    }
    this.parent = parent;
    this.parentNode = null;
    this.childNodes = childNodes;
    this.value = value;
    this.accessed = false;
    this.originalTile = tile;
    this.datatype = node.datatype;
    if (inner instanceof PseudoValue) {
      this.isOuter = true;
      this.inner = inner;
    }
    if (inner === true) {
      this.isInner = true;
      this.datatype = 'semantic';
    }
  }

  // TODO deepcopy
  //

  getParentTileId() {
    return this.tile ? this.tile.parenttile_id : null;
  }

  async getTile(): Promise<[StaticTile | null, any[]]> {
    await this.updateValue();

    let relationships: Array<any> = [];

    if (this.inner) {
      [this.tile, relationships] = await this.inner.getTile();
    }

    let tileValue;
    if (this.value !== null) {
      // It may be better to make this fully async if there's a performance benefit.
      const [newTileValue, ownRelationships] = await (await this.value).__asTileData();
      tileValue = newTileValue;
      relationships = [...relationships, ...ownRelationships];
    } else {
      tileValue = null;
    }
    // if isinstance(tile_value, tuple):
    //     relationships = [
    //         relationship
    //         if isinstance(relationship, tuple)
    //         else (self.tile.nodegroup_id, self.node.nodeid, relationship)
    //         for relationship in tile_value[1]
    //     ]
    //     tile_value = tile_value[0]
    if (!this.tile) {
      throw Error();
    }
    this.tile.data = this.tile.data || {};
    if (tileValue === null) {
      if (this.tile.data.get(this.node.nodeid)) {
        this.tile.data.delete(this.node.nodeid);
      }
    } else {
      this.tile.data.set(this.node.nodeid, tileValue);
    }
    const tile = this.independent ? this.tile : null;

    // TODO relationships
    return [tile, relationships];
  }

  clear() {
    this.value = null;
    if (this.tile && this.tile.data && this.tile.data.has(this.node.nodeid)) {
      this.tile.data.delete(this.node.nodeid);
    }
  }

  updateValue(tile?: StaticTile | null): AttrPromise<VM> {
    if (tile) {
      this.tile = tile;
    }
    this.accessed = true;
    if (this.inner) {
      this.inner.accessed = true;
    }
    if (!this.tile) {
      if (!this.node) {
        throw Error("Empty tile");
      }
      if (this.inner) {
        return new AttrPromise(async (resolve) => {
          const tile = await this.inner?.getTile();
          resolve(this.updateValue(tile ? tile[0] : undefined));
        });
      }
      if (!this.tile) {
        // NB: You may see issues where the nodegroup is null because it is the root node,
        // and a node below is not marked as a collector, so tries to fill its tile in
        // A cardinality n node below the root should be a collector.
        this.tile = new StaticTile({
          nodegroup_id: this.node.nodegroup_id || "",
          tileid: null,
          data: new Map<string, any>(),
          sortorder: this.node.sortorder,
          resourceinstance_id: "",
          parenttile_id: null,
          provisionaledits: null,
          ensureId: () => ""
        });
        // this.relationships = [];
      }
    }
    if (this.valueLoaded === false) {
      this.valueLoaded = undefined;
      let data: any;
      if (
        this.value === null &&
        this.tile.data !== null &&
        this.tile.data.has(this.node.nodeid) &&
        this.datatype !== 'semantic' // Semantic nodes only have placeholder data
      ) {
        data = this.tile.data.get(this.node.nodeid);
      } else {
        data = this.value;
      }

      if (this.isOuter && typeof data === 'object' && this.inner && data) {
        let outerData = undefined;
        if ("_" in data && !data.constructor) {
          outerData = data["_"];
          delete data["_"];
          this.inner.getValue().then((v: ISemantic | null) => v && v.update(data));
          data = outerData;
        } else if (data instanceof Map && data.has("_")) {
          outerData = data.get("_");
          data.delete("_");
          this.inner.getValue().then((v: ISemantic | null) => v && v.update(data));
          data = outerData;
        }
      }
      const vm = getViewModel(
        this,
        this.tile,
        this.node,
        data,
        this.parent,
        this.childNodes,
        this.isInner
      );

      const resolveAttr = (vm: IViewModel) => {
        if (vm !== null && vm instanceof Object) {
          vm.__parentPseudo = this;
          if (this.isOuter && this.inner) {
            vm._ = this.inner.getValue();
          }

          this.valueLoaded = true;
        }
        return vm;
      };
      this.value = new AttrPromise((resolve) => {
        vm.then((vm) => resolve(vm ? resolveAttr(vm) : vm));
      });
    }

    return this.value;
  }

  public getValue(): AttrPromise<VM | null> {
    return this.updateValue();
  }

  // @value.setter
  // def value(this, value):

  getLength() {
    return this.getChildren().length;
  }

  async getChildTypes() {
    await this.updateValue();
    let childTypes = {};
    if (this.value && this.value instanceof Object && 'getChildTypes' in this.value && typeof this.value.getChildTypes === 'function') {
      childTypes = this.value.getChildTypes();
    }
    if (this.inner) {
      Object.assign(childTypes, this.inner.getChildTypes());
    }
    return childTypes;
  }

  getChildren(direct = null): IPseudo[] {
    let children = [];
    if (this.value && this.value instanceof Object && 'getChildren' in this.value && typeof this.value.getChildren === 'function') {
      children = this.value.getChildren(direct);
    }
    if (this.inner) {
      children = [...children, ...this.inner.getChildren(direct)];
    }
    return children;
  }

  async forJson(): Promise<{[key: string]: any} | {[key: string]: any}[] | string | number | boolean | null> {
    const value = (await this.getValue());
    return value instanceof Object ? value.forJson() : value;
  }
}

class PseudoList extends Array implements IPseudo {
  node: StaticNode | undefined = undefined;
  parent: IRIVM<any> | null | undefined = undefined;
  parentNode: PseudoValue<any> | null = null;
  tile: StaticTile | undefined;
  parenttileId: string | undefined;
  ghostChildren: Set<PseudoValue<any>> | null = null;
  isOuter: boolean = false;

  isIterable(): boolean {
    return true;
  }

  async sorted() {
    const resolved = await Promise.all(this.map(async (pn) => await pn));
    const test = [];
    const sorted = resolved.sort((a, b) => {
      const vals = [a, b].map(val => {
        if (val && a.__parentPseudo && a.__parentPseudo.tile) {
          if (val.__parentPseudo.tile.sortorder > 0) {
            // RMV
            test.push(1);
          }
          return val.__parentPseudo.tile.sortorder;
        } else {
          return 0;
        }
      });
      return vals[0] - vals[1];
    });
    return sorted;
  }

  describeField() {
    if (!this.node) {
      return "[(uninitialized node)]";
    }

    let fieldName = this.node.name;
    if (this.parent && this.parent.__) {
      fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
    }
    return `[${fieldName}]`;
  }

  describeFieldGroup() {
    if (!this.node) {
      return "[(uninitialized node)]";
    }

    let fieldName = this.node.name;
    if (this.parent && this.node.nodegroup_id && this.parent.$) {
      const nodegroup = this.parent.$.model.getNodeObjects().get(this.node.nodegroup_id);
      if (nodegroup && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
      }
    }
    return `[${fieldName}]`;
  }

  // Otherwise interferes with Array methods;
  initialize(node: StaticNode, parent: IRIVM<any> | null) {
    this.node = node;
    if (Array.isArray(this.node)) {
      throw Error("Cannot make a list of lists");
    }
    if (!parent) {
      throw Error("Must have a parent or parent class for a pseudo-node");
    }
    this.parent = parent;
    this.tile = undefined;
    this.parenttileId = undefined;
    this.ghostChildren = new Set();
  }

  async forJson(): Promise<{[key: string]: any}[]> {
    const array: {[key: string]: any}[] = Array.from(
      (await this.sorted()).map(
        async (entry: Promise<IViewModel> | IViewModel) => {
          const value = await entry;
          return (value && value instanceof Object && value.forJson) ? value.forJson() : value;
        }
      )
    );
    return Promise.all(array);
  }

  getValue(): AttrPromise<PseudoList> {
    return new AttrPromise(resolve => resolve(this));
  }

  toString() {
    return `<PL: ${this.length}>`;
  }
}

// Fix wkri type.
function makePseudoCls(
  model: IModelWrapper<any>,
  key: string,
  single: boolean,
  tile: StaticTile | null = null,
  wkri: any | null = null,
): PseudoList | PseudoValue<any> | PseudoUnavailable {
  const nodeObjs = model.getNodeObjectsByAlias();
  const nodeObj = nodeObjs.get(key);
  if (!nodeObj) {
    throw Error("Could not find node by alias");
  }

  const nodegroups = model.getNodegroupObjects();
  const nodegroup = nodegroups.get(nodeObj.nodegroup_id || "");

  let value = null;
  if (
    nodeObj.nodegroup_id &&
    nodeObj.is_collector &&
    nodegroup &&
    nodegroup.cardinality == "n" &&
    !single
  ) {
    value = new PseudoList();
    value.initialize(nodeObj, wkri);
  }
  if (value === null || tile) {
    let nodeValue;
    const isPermitted = model.isNodegroupPermitted(nodeObj.nodegroup_id || '', tile, nodeObjs);
    if (isPermitted) {
      const childNodes: Map<string, StaticNode> = model.getChildNodes(nodeObj.nodeid);
      let inner: boolean | PseudoValue<any> = false;
      if (childNodes && childNodes.size && nodeObj.datatype !== 'semantic') {
        inner = new PseudoValue(nodeObj, tile, null, wkri, childNodes, true);
      }
      nodeValue = new PseudoValue(nodeObj, tile, null, wkri, inner !== false ? new Map() : childNodes, inner);
    } else {
      nodeValue = new PseudoUnavailable(nodeObj);
    }
    // If we have a tile in a list, add it
    if (value) {
      value.push(nodeValue.getValue());
    } else {
      value = nodeValue;
    }
  }

  return value;
}

export { PseudoValue, PseudoList, PseudoUnavailable, makePseudoCls };
