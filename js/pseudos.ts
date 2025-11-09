import { StaticTile, StaticNode } from "./static-types";
import { ISemantic, IViewModel, IPseudo, IRIVM, IModelWrapper } from "./interfaces";
import { getViewModel } from "./viewModels";
import { AttrPromise } from "./utils";
import { PseudoNode } from "../pkg/wasm";

class PseudoUnavailable implements IPseudo {
  parentValue: PseudoValue<any> | null = null;
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

// PseudoNode is now implemented in Rust and imported from ../pkg/wasm
// ITERABLE_DATATYPES constant is also defined in Rust (src/pseudos.rs)

class PseudoValue<VM extends IViewModel> implements IPseudo {
  // Note: node, parentNode, datatype, childNodes, isOuter, isInner, inner are now in Rust PseudoNode base class

  node: PseudoNode;
  tile: StaticTile | null;
  value: any;
  parent: IRIVM<any> | null;
  _parentValue: PseudoValue<any> | null = null;
  valueLoaded: boolean | undefined = false;
  originalTile: StaticTile | null;
  accessed: boolean;
  independent: boolean;
  inner: PseudoValue<any> | null = null;

  // Proxy getters to node properties
  // PORT: Line 64-66 (src/pseudo_value.rs:307-311 - datatype getter)
  get datatype() {
    if (this.rustValue) {
      return this.rustValue.datatype; // Phase 4b: Use Rust when available
    }
    return this.node.datatype;
  }

  // PORT: Line 68-70
  get isInner() {
    return this.node.isInner;
  }

  // PORT: Line 72-74
  get isOuter() {
    return this.node.isOuter;
  }

  // PORT: Line 76-78 (src/pseudo_value.rs:373-380 - inner getter)
  get inner() {
    if (this.rustValue?.inner) {
      // Phase 4b: Lazily wrap Rust inner if not yet wrapped
      if (!this.inner) {
        // Create JS PseudoValue wrapper for Rust inner
        const innerNode = this.node.inner;
        if (innerNode) {
          this.inner = new PseudoValue(
            innerNode,
            this.tile,
            null,
            this.parent!,
            new Map(),
            this.rustValue.inner,
            true
          );
        }
      }
    }
    return this.inner;
  }

  get parentValue() {
    return this._parentValue;
  }

  set parentValue(newParentValue: PseudoValue) {
    this.node.parentNode = newParentValue.node;
    this._parentValue = newParentValue;
  }

  isIterable() {
    return this.node.isIterable();
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

  // Phase 4b: Optional Rust WASM backing
  // PORT: Equivalent to WasmPseudoValue in src/pseudo_value.rs:300-381
  rustValue?: any; // WasmPseudoValue from Rust

  constructor(
    node: StaticNode | PseudoNode,
    tile: StaticTile | null,
    value: any,
    parent: IRIVM<any>,
    childNodes: Map<string, StaticNode>,
    rustValue?: any, // Phase 4b: Optional WasmPseudoValue from Rust
  ) {
    // Phase 4b: Store Rust backing if provided
    // PORT: RustPseudoValue in src/pseudo_value.rs:5-37
    this.rustValue = rustValue;

    // PORT: Line 120 - tile assignment (src/pseudo_value.rs:11 - tile field)
    // Phase 4d: Extract tile from Rust if not provided in JS
    // PORT: src/pseudo_value.rs:306-312 - tile getter exposes Rust tile
    if (tile === null && rustValue && rustValue.tile) {
      this.tile = rustValue.tile;
    } else {
      this.tile = tile;
    }
    // PORT: Line 121 - independent flag
    this.independent = this.tile === null;
    // PORT: Line 122-124 - parent validation
    if (!parent) {
      throw Error("Must have a parent or parent class for a pseudo-node");
    }
    // PORT: Line 125-129 - node initialization
    if (node.constructor.name === 'StaticNode') {
      this.node = parent.__.createPseudoNode(node.alias);
    } else {
      this.node = node;
    }
    // PORT: Line 130-133 - basic field initialization
    this.parent = parent;
    // Phase 4d: Extract tile_data from Rust if value not provided
    // PORT: src/pseudo_value.rs:313-319 - tileData getter exposes Rust tile_data
    if (value === null && rustValue && rustValue.tileData) {
      this.value = rustValue.tileData;
    } else {
      this.value = value;
    }
    this.accessed = false;
    this.originalTile = this.tile; // Use extracted tile
    // PORT: Line 134-143 - inner/outer pattern (src/pseudo_value.rs:15 - inner field)
    if (node.isOuter) {
      // Phase 4d: Extract inner tile from Rust if available
      // PORT: src/pseudo_value.rs:373-380 - inner getter exposes Rust inner
      const innerTile = (rustValue?.inner?.tile) ?? this.tile;
      this.inner = new PseudoValue(
        node.inner,
        innerTile, // Phase 4d: Use Rust inner tile
        null,
        parent,
        childNodes,
        rustValue?.inner, // Phase 4b: Pass Rust inner if available
        true
      )
    }
  }

  // TODO deepcopy
  //

  // PORT: Line 189-191
  getParentTileId() {
    // Phase 4b: Get tile from Rust if available
    // PORT: src/pseudo_value.rs:331-338 - tile getter
    const tile = this.rustValue?.tile ?? this.tile;
    return tile ? tile.parenttile_id : null;
  }

  // PORT: Line 193-224
  async getTile(): Promise<[StaticTile | null, any[]]> {
    await this.updateValue();

    let relationships: Array<any> = [];

    // PORT: Line 198-200
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
      const childNodes: Map<string, StaticNode> = this.parent.__.getChildNodes(this.node.nodeid);
      const vm = getViewModel(
        this,
        this.tile,
        this.node,
        data,
        this.parent,
        childNodes,
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
  parentValue: PseudoValue<any> | null = null;
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

// makePseudoClsForNode is now imported from WASM

// PORT: Lines 481-528 (with Phase 4b enhancements)
// Fix wkri type.
/**
 * Legacy JavaScript-only implementation of makePseudoCls.
 * Creates PseudoValue/PseudoList WITHOUT Rust backing.
 *
 * For Rust-backed values, use wrapRustPseudo() instead.
 *
 * @deprecated Use wrapRustPseudo for Rust-backed values
 */
function makePseudoCls_JS(
  model: IModelWrapper<any>,
  key: string,
  single: boolean,
  tile: StaticTile | null = null,
  wkri: any,
): PseudoList | PseudoValue<any> | PseudoUnavailable {
  // PORT: Line 487-491
  const nodeObjs = model.getNodeObjectsByAlias();
  const nodeObj = nodeObjs.get(key);
  if (!nodeObj) {
    throw Error("Could not find node by alias");
  }

  // PORT: Line 493-494
  const nodegroups = model.getNodegroupObjects();
  const nodegroup = nodegroups.get(nodeObj.nodegroup_id || "");

  // PORT: Line 496-506
  let value = null;
  if (
    nodeObj.nodegroup_id &&
    nodeObj.is_collector &&
    nodegroup &&
    nodegroup.cardinality == "n" &&
    !single
  ) {
    // PORT: Line 504-505 - create PseudoList
    value = new PseudoList();
    value.initialize(nodeObj, wkri);
  }

  // PORT: Line 507-525
  // JS-only path: create single value (no Rust backing)
  if (value === null || tile) {
    let nodeValue;
    const isPermitted = model.isNodegroupPermitted(nodeObj.nodegroup_id || '', tile, nodeObjs);
    if (isPermitted) {
      const childNodes: Map<string, StaticNode> = model.getChildNodes(nodeObj.nodeid);
      let inner: boolean | PseudoValue<any> = false;
      if (childNodes && childNodes.size && nodeObj.datatype !== 'semantic') {
        inner = new PseudoValue(nodeObj, tile, null, wkri, childNodes, true);
      }
      nodeValue = new PseudoValue(
        nodeObj,
        tile,
        null,
        wkri,
        inner !== false ? new Map() : childNodes,
        inner
      );
    } else {
      nodeValue = new PseudoUnavailable(nodeObj);
    }
    if (value) {
      value.push(nodeValue.getValue());
    } else {
      value = nodeValue;
    }
  }

  // PORT: Line 528
  return value;
}

/**
 * Thin wrapper to convert Rust WasmPseudoValue/WasmPseudoList to TS PseudoValue/PseudoList
 * This is the ONLY place where Rust values should be wrapped in TS classes (besides populate/ensureNodegroup)
 *
 * @param rustValue - WasmPseudoValue or WasmPseudoList from Rust
 * @param wkri - The WKRI wrapper
 * @param model - The model wrapper (for getting nodes and child relationships)
 * @returns PseudoValue, PseudoList, or PseudoUnavailable
 */
function wrapRustPseudo(
  rustValue: any | null,
  wkri: any,
  model: any,
): PseudoValue<any> | PseudoList | PseudoUnavailable {
  // Handle null/unavailable case
  if (rustValue === null || rustValue === undefined) {
    // We don't have the node here, so return a generic unavailable
    // The caller should handle this case
    throw new Error("Cannot wrap null rustValue - caller should handle permissions");
  }

  // Check if it's a WasmPseudoList (has getAllValues method)
  if (typeof rustValue.getAllValues === 'function') {
    // It's a list - create PseudoList and populate with wrapped values
    const nodeAlias = rustValue.nodeAlias;
    const wasmValues = rustValue.getAllValues();

    const list = new PseudoList();
    // We need a dummy node to initialize - get it from the first value
    if (wasmValues.length > 0) {
      const firstValue = wasmValues[0];
      const nodeId = firstValue.nodeId;
      const nodeObjs = model.getNodeObjectsByAlias();
      // Find node by nodeId
      for (const [, node] of nodeObjs.entries()) {
        if (node.nodeid === nodeId) {
          list.initialize(node, wkri);
          break;
        }
      }
    }

    // Wrap each WasmPseudoValue in a TS PseudoValue and add to list
    for (const wasmValue of wasmValues) {
      const nodeId = wasmValue.nodeId;
      const nodeObjs = model.getNodeObjectsByAlias();

      // Find the node object
      let nodeObj = null;
      for (const [, node] of nodeObjs.entries()) {
        if (node.nodeid === nodeId) {
          nodeObj = node;
          break;
        }
      }

      if (!nodeObj) {
        throw new Error(`Node not found for nodeId: ${nodeId}`);
      }

      const childNodes = model.getChildNodes(nodeObj.nodeid);

      // Handle inner/outer pattern
      let inner: boolean | PseudoValue<any> = false;
      if (childNodes && childNodes.size && nodeObj.datatype !== 'semantic') {
        const innerWasm = wasmValue.inner;
        if (innerWasm) {
          inner = new PseudoValue(nodeObj, null, null, wkri, childNodes, innerWasm, true);
        }
      }

      const pseudoValue = new PseudoValue(
        nodeObj,
        null, // tile extracted from wasmValue in constructor
        null,
        wkri,
        inner !== false ? new Map() : childNodes,
        wasmValue,
        inner
      );

      list.push(pseudoValue.getValue());
    }

    return list;
  } else {
    // It's a single value - wrap in PseudoValue
    const nodeId = rustValue.nodeId;
    const nodeObjs = model.getNodeObjectsByAlias();

    // Find the node object
    let nodeObj = null;
    for (const [, node] of nodeObjs.entries()) {
      if (node.nodeid === nodeId) {
        nodeObj = node;
        break;
      }
    }

    if (!nodeObj) {
      throw new Error(`Node not found for nodeId: ${nodeId}`);
    }

    const childNodes = model.getChildNodes(nodeObj.nodeid);

    // Handle inner/outer pattern
    let inner: boolean | PseudoValue<any> = false;
    if (childNodes && childNodes.size && nodeObj.datatype !== 'semantic') {
      const innerWasm = rustValue.inner;
      if (innerWasm) {
        inner = new PseudoValue(nodeObj, null, null, wkri, childNodes, innerWasm, true);
      }
    }

    return new PseudoValue(
      nodeObj,
      null, // tile extracted from rustValue in constructor
      null,
      wkri,
      inner !== false ? new Map() : childNodes,
      rustValue,
      inner
    );
  }
}

export { PseudoNode, PseudoValue, PseudoList, PseudoUnavailable, makePseudoCls_JS, wrapRustPseudo };
