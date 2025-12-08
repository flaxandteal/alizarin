import { StaticTile, StaticNode } from "./static-types";
import { ISemantic, IViewModel, IPseudo, IRIVM, IModelWrapper } from "./interfaces";
import { getViewModel } from "./viewModels";
import { AttrPromise } from "./utils";
import { PseudoNode, WasmPseudoValue, WasmPseudoList } from '../pkg/alizarin';

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

/**
 * PseudoValue - Thin JS wrapper around WasmPseudoValue from Rust
 *
 * Most state and logic now lives in Rust's RustPseudoValue/WasmPseudoValue.
 * This wrapper provides:
 * 1. JS-specific getViewModel integration
 * 2. Promise-based value loading with AttrPromise
 * 3. Inner/outer pattern coordination
 */
class PseudoValue<VM extends IViewModel> implements IPseudo {
  // The Rust backing - all state lives here
  private _wasm: WasmPseudoValue;

  // JS-specific: parent IRIVM reference (needed for getViewModel)
  parent: IRIVM<any> | null;

  // JS-specific: parent PseudoValue for traversal
  _parentValue: PseudoValue<any> | null = null;

  // JS-specific: cached inner wrapper (lazily created)
  private _inner: PseudoValue<any> | null = null;

  // JS-specific: cached loaded value (AttrPromise wrapping VM)
  private _cachedValue: AttrPromise<VM> | null = null;

  // Proxy getters to Rust
  get datatype(): string {
    return this._wasm.datatype;
  }

  get isInner(): boolean {
    return this._wasm.isInner;
  }

  get isOuter(): boolean {
    return this._wasm.isOuter;
  }

  get tile(): StaticTile | null {
    return this._wasm.tile;
  }

  set tile(t: StaticTile | null) {
    this._wasm.tile = t;
  }

  get value(): any {
    return this._wasm.value;
  }

  get valueLoaded(): boolean | undefined {
    const v = this._wasm.valueLoaded;
    if (v === undefined) return undefined;
    return v;
  }

  get accessed(): boolean {
    return this._wasm.accessed;
  }

  get independent(): boolean {
    return this._wasm.independent;
  }

  get originalTile(): StaticTile | null {
    // Original tile is same as tile at construction - Rust tracks this
    return this._wasm.tile;
  }

  // Node property accessors (proxy to wasm which has PseudoNode-compatible API)
  get node(): WasmPseudoValue {
    // WasmPseudoValue now has all PseudoNode getters, so it can act as node
    return this._wasm;
  }

  get nodeid(): string {
    return this._wasm.nodeId;
  }

  // Inner/outer pattern
  get inner(): PseudoValue<any> | null {
    if (this._wasm.isOuter && !this._inner) {
      const wasmInner = this._wasm.inner;
      if (wasmInner) {
        this._inner = PseudoValue.fromWasm(wasmInner, this.parent!);
        this._inner._parentValue = this;
      }
    }
    return this._inner;
  }

  get parentValue(): PseudoValue<any> | null {
    return this._parentValue;
  }

  set parentValue(newParentValue: PseudoValue<any> | null) {
    this._parentValue = newParentValue;
  }

  // Convenience methods
  isIterable(): boolean {
    return this._wasm.isIterable();
  }

  describeField(): string {
    let fieldName = this._wasm.name;
    if (this.parent && this.parent.__) {
      fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
    }
    return fieldName;
  }

  describeFieldGroup(): string {
    let fieldName = this._wasm.name;
    const nodegroupId = this._wasm.nodegroupId;
    if (this.parent && nodegroupId && this.parent.$) {
      const nodegroupName = this.parent.$.model.getNodegroupName(nodegroupId);
      if (nodegroupName && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroupName}`;
      }
    }
    return fieldName;
  }

  /**
   * Create PseudoValue from WasmPseudoValue (preferred constructor)
   */
  static fromWasm<VM extends IViewModel>(
    wasm: WasmPseudoValue,
    parent: IRIVM<any>,
  ): PseudoValue<VM> {
    const pv = new PseudoValue<VM>(wasm, parent);
    return pv;
  }

  /**
   * Private constructor - use static factories
   */
  private constructor(
    wasm: WasmPseudoValue,
    parent: IRIVM<any>,
  ) {
    if (!parent) {
      throw Error("Must have a parent or parent class for a pseudo-node");
    }
    this._wasm = wasm;
    this.parent = parent;
    this._wasm.parent = parent; // Store in Rust too for callbacks
  }

  // Legacy constructor support - create via model wrapper
  static create<VM extends IViewModel>(
    nodeOrAlias: StaticNode | string,
    tile: StaticTile | null,
    value: any,
    parent: IRIVM<any>,
  ): PseudoValue<VM> {
    if (!parent || !parent.__) {
      throw new Error('Must have a parent or parent class for a pseudo-node');
    }
    // Use model wrapper to create WasmPseudoValue
    const alias = typeof nodeOrAlias === 'string' ? nodeOrAlias : nodeOrAlias.alias;
    const wasm = parent.__.createPseudoValue(alias, tile, parent);
    return PseudoValue.fromWasm(wasm, parent);
  }

  getParentTileId(): string | null {
    const tile = this._wasm.tile;
    return tile ? tile.parenttile_id : null;
  }

  async getTile(): Promise<[StaticTile | null, any[]]> {
    await this.updateValue();

    let relationships: Array<any> = [];

    if (this.inner) {
      [this.tile, relationships] = await this.inner.getTile();
    }

    let tileValue;
    if (this._cachedValue !== null) {
      const value = await this._cachedValue;
      if (value !== null) {
        const [newTileValue, ownRelationships] = await (value as any).__asTileData();
        tileValue = newTileValue;
        relationships = [...relationships, ...ownRelationships];
      } else {
        tileValue = null;
      }
    } else {
      tileValue = null;
    }

    const tile = this._wasm.tile;
    if (!tile) {
      throw Error("No tile available");
    }
    tile.data = tile.data || new Map();
    const nodeid = this._wasm.nodeId;
    if (tileValue === null) {
      if (tile.data.has(nodeid)) {
        tile.data.delete(nodeid);
      }
    } else {
      tile.data.set(nodeid, tileValue);
    }
    const returnTile = this.independent ? tile : null;

    return [returnTile, relationships];
  }

  clear(): void {
    this._wasm.clear();
    this._cachedValue = null;
    const tile = this._wasm.tile;
    if (tile && tile.data && tile.data.has(this._wasm.nodeId)) {
      tile.data.delete(this._wasm.nodeId);
    }
  }

  updateValue(newTile?: StaticTile | null): AttrPromise<VM> {
    if (newTile) {
      this._wasm.tile = newTile;
    }

    // If already cached, return it
    if (this._cachedValue !== null) {
      return this._cachedValue;
    }

    // Handle case where tile is null but has inner - get tile first, then load
    if (!this._wasm.tile && this.inner) {
      this._cachedValue = new AttrPromise(async (resolve) => {
        const [innerTile] = await this.inner!.getTile();
        this._wasm.tile = innerTile;
        // Call the real loading logic directly, bypassing the cache wrapper
        const result = await this._updateValueReal();
        resolve(result);
      }) as AttrPromise<VM>;
      return this._cachedValue;
    }

    // Normal path - do the real work and cache it
    this._cachedValue = this._updateValueReal();
    return this._cachedValue;
  }

  private _updateValueReal(): AttrPromise<VM> {
    // Create tile if needed
    if (!this._wasm.tile) {
      const nodegroupId = this._wasm.nodegroupId || "";
      const sortorder = this._wasm.sortorder;
      this._wasm.tile = new StaticTile({
        nodegroup_id: nodegroupId,
        tileid: null,
        data: new Map<string, any>(),
        sortorder: sortorder,
        resourceinstance_id: "",
        parenttile_id: null,
        provisionaledits: null,
        ensureId: () => ""
      });
    }

    // Get data - check tile first, then tileData from Rust
    let data: any;
    const currentTile = this._wasm.tile;
    const nodeid = this._wasm.nodeId;

    if (
      currentTile &&
      currentTile.data !== null &&
      currentTile.data.has(nodeid) &&
      this.datatype !== 'semantic'
    ) {
      data = currentTile.data.get(nodeid);
    } else {
      data = this._wasm.tileData;
    }

    // Handle outer/inner data splitting
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

    // Call getViewModel
    const vmPromise = getViewModel(
      this,
      currentTile!,
      this._wasm.node, // Use node object from wasm
      data,
      this.parent,
      this.isInner
    );

    const resolveAttr = (vm: IViewModel | null): VM | null => {
      if (vm !== null && vm instanceof Object) {
        vm.__parentPseudo = this;
        if (this.isOuter && this.inner) {
          (vm as any)._ = this.inner.getValue();
        }
        // Sync tile data to Rust for toTiles()/toResource()
        this.syncTileData(vm);
      }
      return vm as VM | null;
    };

    return new AttrPromise((resolve) => {
      vmPromise.then((vm) => resolve(resolveAttr(vm)));
    }) as AttrPromise<VM>;
  }

  /**
   * Sync the ViewModel's tile data to Rust.
   * Called after VM creation/resolution so toTiles()/toResource() can serialize.
   */
  private syncTileData(vm: IViewModel | null): void {
    if (vm === null) {
      this._wasm.setTileData(null);
      return;
    }

    // Check if the VM has __asTileData method
    if (typeof (vm as any).__asTileData === 'function') {
      const tileData = (vm as any).__asTileData();
      if (tileData instanceof Promise) {
        // Handle async __asTileData
        tileData.then((data: any) => {
          this._wasm.setTileData(data);
        }).catch((e: Error) => {
          console.warn("syncTileData: failed to get tile data", e);
        });
      } else {
        this._wasm.setTileData(tileData);
      }
    }
  }

  public getValue(): AttrPromise<VM | null> {
    return this.updateValue();
  }

  getLength(): number {
    return this.getChildren().length;
  }

  async getChildTypes(): Promise<object> {
    await this.updateValue();
    let childTypes = {};
    const value = await this._cachedValue;
    if (value && value instanceof Object && 'getChildTypes' in value && typeof (value as any).getChildTypes === 'function') {
      childTypes = (value as any).getChildTypes();
    }
    if (this.inner) {
      Object.assign(childTypes, await this.inner.getChildTypes());
    }
    return childTypes;
  }

  getChildren(direct: boolean | null = null): IPseudo[] {
    let children: IPseudo[] = [];
    // Note: This accesses cached value synchronously. May need async version.
    if (this._cachedValue) {
      // Peek at resolved value if available
      // This is tricky with AttrPromise - might need to refactor
    }
    if (this.inner) {
      children = [...children, ...this.inner.getChildren(direct)];
    }
    return children;
  }

  async forJson(): Promise<{[key: string]: any} | {[key: string]: any}[] | string | number | boolean | null> {
    const value = await this.getValue();
    return value instanceof Object ? (value as any).forJson() : value;
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
  /** Whether this list represents a cardinality-1 node (should unwrap to single value in forJson) */
  isSingle: boolean = false;

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
      const nodegroupName = this.parent.$.model.getNodegroupName(this.node.nodegroup_id);
      if (nodegroupName && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroupName}`;
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

  async forJson(): Promise<{[key: string]: any}[] | {[key: string]: any} | null> {
    const array: {[key: string]: any}[] = Array.from(
      (await this.sorted()).map(
        async (entry: Promise<IViewModel> | IViewModel) => {
          const value = await entry;
          return (value && value instanceof Object && value.forJson) ? value.forJson() : value;
        }
      )
    );
    const resolved = await Promise.all(array);

    // If this is a cardinality-1 node, unwrap to single value
    if (this.isSingle) {
      return resolved.length > 0 ? resolved[0] : null;
    }
    return resolved;
  }

  getValue(): AttrPromise<PseudoList> {
    return new AttrPromise(resolve => resolve(this));
  }

  toString() {
    return `<PL: ${this.length}>`;
  }
}

/**
 * Thin wrapper to convert Rust WasmPseudoValue/WasmPseudoList to TS PseudoValue/PseudoList
 * This is the ONLY place where Rust values should be wrapped in TS classes (besides populate/ensureNodegroup)
 *
 * @param rustValue - WasmPseudoValue or WasmPseudoList from Rust
 * @param wkri - The WKRI wrapper (parent IRIVM)
 * @param model - The model wrapper (for getting nodes - currently unused with new pattern)
 * @returns PseudoValue, PseudoList, or PseudoUnavailable
 */
function wrapRustPseudo(
  rustValue: WasmPseudoValue | WasmPseudoList | null,
  wkri: IRIVM<any>,
  model: any,
): PseudoValue<any> | PseudoList | PseudoUnavailable | null | undefined {
  // Handle null/unavailable case
  if (rustValue === null || rustValue === undefined) {
    return rustValue === null ? null : undefined;
  }

  // Check if it's a WasmPseudoList (has getAllValues method)
  if ('getAllValues' in rustValue && typeof rustValue.getAllValues === 'function') {
    const wasmList = rustValue as WasmPseudoList;
    const wasmValues = wasmList.getAllValues();

    const list = new PseudoList();
    // Initialize with first value's node info
    if (wasmValues.length > 0) {
      const firstValue = wasmValues[0];
      const nodeId = firstValue.nodeId;
      const node = model.getNodeObjectFromId(nodeId);
      list.initialize(node, wkri);
    }

    // Copy isSingle flag from WASM list (cardinality-1 nodes should unwrap to single value)
    list.isSingle = wasmList.isSingle;

    // Wrap each WasmPseudoValue using static factory
    // Push getValue() result (AttrPromise) - PseudoList contract expects
    // IViewModel or Promise<IViewModel>, and AttrPromise allows property chaining
    for (const wasmValue of wasmValues) {
      const pseudoValue = PseudoValue.fromWasm(wasmValue, wkri);
      list.push(pseudoValue.getValue());
    }

    return list;
  } else {
    // It's a single WasmPseudoValue - wrap using static factory
    return PseudoValue.fromWasm(rustValue as WasmPseudoValue, wkri);
  }
}

export { PseudoNode, PseudoValue, PseudoList, PseudoUnavailable, wrapRustPseudo };
