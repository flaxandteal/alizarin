import { StaticTile, StaticNode } from "./static-types";
import { CheckPermission, IViewModel, IPseudo, IRIVM, IModelWrapper } from "./interfaces";
import { getViewModel } from "./viewModels";
import { AttrPromise } from "./utils";

class PseudoUnavailable implements IPseudo {
  parentNode: PseudoValue | null = null;
  tile: null = null;
  node: StaticNode;

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

  async getValue() {
    return null;
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

class PseudoValue implements IPseudo {
  node: StaticNode;
  tile: StaticTile | null;
  value: any;
  parent: IRIVM<any> | null;
  parentNode: PseudoValue | null;
  valueLoaded: boolean | undefined = false;
  datatype: string | null = null;
  originalTile: StaticTile | null;
  accessed: boolean;
  childNodes: Map<string, StaticNode>;

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
    if (this.parent && this.node.nodegroup_id && this.parent._) {
      const nodegroup = this.parent._.model.getNodeObjects().get(this.node.nodegroup_id);
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
  ) {
    this.node = node;
    this.tile = tile;
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
  }

  // TODO deepcopy
  //

  getParentTileId() {
    return this.tile ? this.tile.parenttile_id : null;
  }

  async getTile() {
    await this.updateValue();

    const relationships: Array<any> = [];
    let tileValue;
    if (this.value !== null) {
      // It may be better to make this fully async if there's a performance benefit.
      tileValue = await this.value.__asTileData();
    } else {
      tileValue = this.value;
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
    const tile = this.node.is_collector ? this.tile : null;

    // TODO relationships
    return [tile, relationships];
  }

  clear() {
    this.value = null;
    if (this.tile && this.tile.data && this.tile.data.has(this.node.nodeid)) {
      this.tile.data.delete(this.node.nodeid);
    }
  }

  updateValue(): AttrPromise<IViewModel> {
    this.accessed = true;

    if (!this.tile) {
      if (!this.node) {
        throw Error("Empty tile");
      }
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
    if (this.valueLoaded === false) {
      this.valueLoaded = undefined;
      let data;
      if (
        this.value === null &&
        this.tile.data !== null &&
        this.tile.data.has(this.node.nodeid)
      ) {
        data = this.tile.data.get(this.node.nodeid);
      } else {
        data = this.value;
      }

      const vm = getViewModel(
        this,
        this.tile,
        this.node,
        data,
        this.parent,
        this.childNodes,
      );

      const resolveAttr = (vm: IViewModel) => {
        if (vm !== null && vm instanceof Object) {
          vm.__parentPseudo = this;
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

  public getValue(): AttrPromise<IViewModel | null> {
    return this.updateValue();
  }

  // @value.setter
  // def value(this, value):

  getLength() {
    return this.getChildren().length;
  }

  async getChildTypes() {
    await this.updateValue();
    if (this.value instanceof Object) {
      try {
        return this.value.get_child_types();
      } catch (AttributeError) {}
    }
    return {};
  }

  getChildren(direct = null) {
    if (this.value) {
      try {
        return this.value.getChildren(direct);
      } catch (AttributeError) {}
    }
    return [];
  }

  async forJson(): Promise<{[key: string]: any} | {[key: string]: any}[] | string | number | boolean | null> {
    const value = (await this.getValue());
    return value instanceof Object ? value.forJson() : value;
  }
}

class PseudoList extends Array implements IPseudo {
  node: StaticNode | undefined = undefined;
  parent: IRIVM<any> | null | undefined = undefined;
  parentNode: PseudoValue | null = null;
  tile: StaticTile | undefined;
  parenttileId: string | undefined;
  ghostChildren: Set<PseudoValue> | null = null;

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
    if (this.parent && this.node.nodegroup_id && this.parent._) {
      const nodegroup = this.parent._.model.getNodeObjects().get(this.node.nodegroup_id);
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
): PseudoList | PseudoValue | PseudoUnavailable {
  const nodeObj = model.getNodeObjectsByAlias().get(key);
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
    const isPermitted = model.isNodegroupPermitted(nodeObj.nodegroup_id || '', nodeObj, tile);
    if (isPermitted) {
      const childNodes = model.getChildNodes(nodeObj.nodeid);
      nodeValue = new PseudoValue(nodeObj, tile, null, wkri, childNodes);
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
