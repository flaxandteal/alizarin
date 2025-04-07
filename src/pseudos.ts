import { StaticTile, StaticNode, StaticNodegroup } from "./static-types";
import { IViewModel, IPseudo, IRIVM, IModelWrapper } from "./interfaces";
import { getViewModel } from "./viewModels";
import { AttrPromise } from "./utils";

class PseudoUnavailable implements IPseudo {
  parentNode: PseudoValue | null = null;

  async getValue() {
    console.warn("Tried to get value of unavailable node");
    return null;
  }

  getLength() {
    return 0;
  }

  getChildren(_: boolean = false) {
    return [];
  }
}

class PseudoValue implements IPseudo {
  node: StaticNode;
  tile: StaticTile | null;
  value: any;
  parent: IRIVM | null;
  parentNode: PseudoValue | null;
  valueLoaded: boolean | undefined = false;
  datatype: string | null = null;
  originalTile: StaticTile | null;
  accessed: boolean;
  childNodes: Map<string, StaticNode>;
  multiple: boolean = false;
  asTileData: Function | null = null;

  constructor(
    node: StaticNode,
    tile: StaticTile | null,
    value: any,
    parent: IRIVM | null,
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
    if (this.asTileData && this.value !== null) {
      // It may be better to make this fully async if there's a performance benefit.
      tileValue = await this.asTileData(this.value);
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
    if (this.tile && this.tile.data && this.node.nodeid in this.tile.data) {
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
        let value;
        [value, this.asTileData, this.datatype, this.multiple] = vm;
        if (value !== null && this.value instanceof Object) {
          value.__parentPseudo = this;
        }
        if (value !== null) {
          this.valueLoaded = true;
        }
        return value;
      };
      this.value = new AttrPromise((resolve) => {
        vm.then((vm) => resolve(resolveAttr(vm)));
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

  async getType() {
    await this.updateValue();
    return [this.datatype, this.multiple];
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
}

class PseudoList extends Array implements IPseudo {
  node: StaticNode;
  parent: IRIVM | null;
  parentNode: PseudoValue | null = null;
  tile: StaticTile | undefined;
  parenttileId: string | undefined;
  ghostChildren: Set<PseudoValue> | null = null;

  // Otherwise interferes with Array methods;
  initialize(node: StaticNode, parent: IRIVM | null) {
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

  getValue() {
    return this;
  }

  toString() {
    return `<PL: ${this.length}>`;
  }
}

// Fix wkri type.
function makePseudoCls(
  this: IModelWrapper,
  key: string,
  single: boolean,
  tile: StaticTile | null = null,
  wkri: any | null = null,
): PseudoList | PseudoValue | PseudoUnavailable {
  const nodeObj = this.getNodeObjectsByAlias().get(key);
  if (!nodeObj) {
    throw Error("Could not find node by alias");
  }

  const nodegroups = this.getNodegroupObjects();
  const nodegroup = nodegroups.get(nodeObj.nodegroup_id || "");

  const permitted = this.getPermittedNodegroups();
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
    if (nodeObj.nodegroup_id && !permitted.get(nodeObj.nodegroup_id)) {
      nodeValue = new PseudoUnavailable();
    } else {
      const childNodes = this.getChildNodes(nodeObj.nodeid);
      nodeValue = new PseudoValue(nodeObj, tile, null, wkri, childNodes);
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
