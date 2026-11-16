import { StaticTile, StaticNode } from "./static-types";
import { IViewModel, IPseudo, IRIVM } from "./interfaces";
import { getViewModel } from "./viewModels";
import { AttrPromise } from "./utils";
import { createStaticTile } from "./backend";
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

  // JS-cached node (see `node`)
  private _node: StaticNode | null = null;

  // JS-cached snapshot of commonly-used properties (avoids repeated WASM boundary crossings)
  private _snapshot: {
    datatype: string;
    nodeId: string;
    name: string;
    alias?: string;
    nodegroupId?: string;
    sortorder?: number;
    isOuter: boolean;
    isInner: boolean;
    isCollector: boolean;
    accessed: boolean;
    independent: boolean;
    hasTile: boolean;
    tileData?: any;
    tileId?: string;
    valueLoaded?: boolean;
  } | null = null;

  // Get snapshot (cached after first access)
  private getSnapshot() {
    if (!this._snapshot) {
      this._snapshot = this._wasm.toSnapshot();
    }
    return this._snapshot;
  }

  // Proxy getters - use cached snapshot where possible
  get datatype(): string {
    return this.getSnapshot().datatype;
  }

  get isInner(): boolean {
    return this.getSnapshot().isInner;
  }

  get isOuter(): boolean {
    return this.getSnapshot().isOuter;
  }

  get tile(): StaticTile | null {
    return this._wasm.tile;
  }

  set tile(t: StaticTile | null) {
    this._wasm.tile = t;
    // Invalidate snapshot when tile changes
    this._snapshot = null;
  }

  get valueLoaded(): boolean | undefined {
    const v = this.getSnapshot().valueLoaded;
    if (v === undefined) return undefined;
    return v;
  }

  get accessed(): boolean {
    return this.getSnapshot().accessed;
  }

  get independent(): boolean {
    return this.getSnapshot().independent;
  }

  get originalTile(): StaticTile | null {
    // Original tile is same as tile at construction - Rust tracks this
    return this._wasm.tile;
  }

  // The node itself (StaticNode fields: nodeid, alias, config...), on every
  // backend. Fetched once: a node never changes, and each fetch is a copy.
  get node(): StaticNode {
    if (this._node === null) {
      this._node = this._wasm.node;
    }
    return this._node!;
  }

  get nodeid(): string {
    return this.getSnapshot().nodeId;
  }

  // Inner/outer pattern
  get inner(): PseudoValue<any> | null {
    if (this.getSnapshot().isOuter && !this._inner) {
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

    // A semantic value gives [null, its children's relationships]; any other
    // value's tile data is core's already (edits go through core), so it
    // needn't be computed here.
    if (this._cachedValue !== null && this.getSnapshot().datatype === 'semantic') {
      const value = await this._cachedValue;
      if (value !== null) {
        const [, ownRelationships] = await (value as any).__asTileData();
        relationships = [...relationships, ...ownRelationships];
      }
    }

    if (!this.getSnapshot().hasTile) {
      throw Error("No tile available");
    }
    const returnTile = this.independent ? this._wasm.tile : null;

    return [returnTile, relationships];
  }

  clear(): void {
    // Core's: the value lives there, not in a copy of the tile.
    this._wasm.clear();
    this._cachedValue = null;
    this._snapshot = null;
  }

  updateValue(newTile?: StaticTile | null): AttrPromise<VM> {
    if (newTile) {
      this.tile = newTile;
    }

    // If already cached, return it
    if (this._cachedValue !== null) {
      return this._cachedValue;
    }

    // Handle case where tile is null but has inner - get tile first, then load
    if (!this.getSnapshot().hasTile && this.inner) {
      this._cachedValue = new AttrPromise(async (resolve) => {
        const [innerTile] = await this.inner!.getTile();
        this.tile = innerTile;
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
    // Get snapshot once - avoids multiple boundary crossings
    const snapshot = this.getSnapshot();

    // This node's value: core keeps it in step with the tile (set_tile), on
    // every backend, so the tile's data map needn't be copied out to read it.
    // ViewModels expect null for "no data", not undefined.
    const data: any = snapshot.tileData ?? null;

    // Create tile if needed
    if (!snapshot.hasTile) {
      this.tile = createStaticTile({
        nodegroup_id: snapshot.nodegroupId || "",
        tileid: null,
        data: new Map<string, any>(),
        sortorder: snapshot.sortorder,
        resourceinstance_id: "",
        parenttile_id: null,
        provisionaledits: null,
        ensureId: () => ""
      });
    }
    const currentTile = this._wasm.tile;

    // No outer/inner "_" split here: `data` is core's value for this node
    // alone (an outer node's children have their own), never a "_" tree.
    // Trees are split where they are read, in core (tree-to-tiles), as on
    // Python; the inner's children are reached through vm._ (below).

    // Call getViewModel
    const vmPromise = getViewModel(
      this,
      currentTile!,
      this.node,
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
      }
      return vm as VM | null;
    };

    return new AttrPromise((resolve) => {
      vmPromise.then((vm) => resolve(resolveAttr(vm)));
    }) as AttrPromise<VM>;
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
  /** Set once built from core's values: from then on push/splice record edits. */
  __editable: boolean = false;
  /** The node the list's items are values of. */
  __alias: string | undefined;
  /** Each item's tile id, or, for an item pushed since the last commit, its edit's index. */
  __tileIds: Array<string | number | null> = [];

  /**
   * Once built: record each item (a tree fragment of values by alias) as a new
   * tile, applied by the resource's $commit(). As Python's PseudoList.append.
   */
  push(...items: any[]): number {
    if (!this.__editable) {
      return super.push(...items);
    }
    const wrapper = this.parent?.$;
    if (!wrapper || !this.__alias) {
      throw Error("Cannot add to this list: it has no resource to edit");
    }
    for (const item of items) {
      const ref = wrapper.recordEdit({
        op: 'add',
        alias: this.__alias,
        parent: this.parenttileId ?? null,
        values: item ?? {},
      });
      this.__tileIds.push(ref);
      super.push(new AttrPromise((resolve) => resolve(item)));
    }
    wrapper.setPendingValue(this.parenttileId ?? null, this.__alias, this);
    return this.length;
  }

  /**
   * Once built: record removing each item's tile (and all under it), applied
   * by the resource's $commit(). Inserting isn't supported: push adds, and
   * tile order follows sortorder.
   */
  splice(start: number, deleteCount?: number, ...items: any[]): any[] {
    if (!this.__editable) {
      return deleteCount === undefined
        ? super.splice(start)
        : super.splice(start, deleteCount, ...items);
    }
    if (items.length > 0) {
      throw Error("Add items with push: tile order follows sortorder");
    }
    const wrapper = this.parent?.$;
    if (!wrapper || !this.__alias) {
      throw Error("Cannot remove from this list: it has no resource to edit");
    }
    const from = start < 0 ? Math.max(this.length + start, 0) : Math.min(start, this.length);
    const count = Math.max(0, Math.min(deleteCount ?? this.length - from, this.length - from));
    const tileIds = this.__tileIds.slice(from, from + count);
    if (tileIds.some((id) => typeof id !== 'string')) {
      throw Error("Cannot remove an item added since the last $commit(): $discard() instead");
    }
    for (const tile of tileIds) {
      wrapper.recordEdit({ op: 'remove', tile });
    }
    this.__tileIds.splice(from, count);
    const removed = Array.from(this).slice(from, from + count);
    // Array.prototype.splice would make a PseudoList for its result; this
    // list's own items shift down instead.
    for (let i = from; i + count < this.length; i++) {
      this[i] = this[i + count];
    }
    this.length -= count;
    wrapper.setPendingValue(this.parenttileId ?? null, this.__alias, this);
    return removed;
  }

  isIterable(): boolean {
    return true;
  }

  // Values in tile sortorder: tiles without one after the numbered ones, ties
  // and unnumbered tiles in load order (as core orders semantic children).
  async sorted() {
    const resolved = await Promise.all(this.map(async (pn) => await pn));
    const sortorder = (val: { __parentPseudo?: { tile?: { sortorder?: number | null } } | null }) =>
      val?.__parentPseudo?.tile?.sortorder;
    // Array.prototype.sort is stable, so equal keys keep load order.
    return resolved.sort((a, b) => {
      const [x, y] = [sortorder(a), sortorder(b)];
      if (x == null || y == null) {
        return Number(x == null) - Number(y == null);
      }
      return x - y;
    });
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

  getValue(): AttrPromise<PseudoList | IViewModel | null> {
    // For cardinality-1 nodes (isSingle=true), unwrap to single value
    // This matches the behavior of the async retrieve path which returns
    // WasmPseudoValue (not WasmPseudoList) for is_single=true nodes
    if (this.isSingle) {
      if (this.length > 0) {
        // The list contains AttrPromise<IViewModel> items, return the first one
        return this[0] as AttrPromise<IViewModel>;
      } else {
        return new AttrPromise(resolve => resolve(null));
      }
    }
    return new AttrPromise(resolve => resolve(this));
  }

  /**
   * Resolve every per-tile entry and flatten one level into a single list.
   *
   * A cardinality-N nodegroup yields one entry per tile. When the node is itself
   * list-valued (`resource-instance-list`, `concept-list`, `domain-value-list`),
   * each per-tile entry is ALSO a list — so awaiting the field directly gives a
   * list of per-tile lists (`[[a, b], [c]]`), which readers would otherwise have
   * to flatten by hand (C6). `flattened()` does that, awaiting any nested
   * promises: `[[a, b], [c]]` becomes `[a, b, c]`. For a scalar cardinality-N node
   * (one value per tile) it is equivalent to awaiting the field, so it is safe to
   * use uniformly. `null`/`undefined` entries are dropped.
   */
  async flattened(): Promise<any[]> {
    const resolved = await Promise.all(Array.from(this).map(async (item) => await item));
    const out: any[] = [];
    for (const entry of resolved) {
      if (Array.isArray(entry)) {
        const inner = await Promise.all(Array.from(entry).map(async (e: any) => await e));
        out.push(...inner.filter((e) => e !== null && e !== undefined));
      } else if (entry !== null && entry !== undefined) {
        out.push(entry);
      }
    }
    return out;
  }

  toString() {
    return `<PL: ${this.length}>`;
  }
}

/**
 * Thin wrapper to convert Rust WasmPseudoValue/WasmPseudoList to TS PseudoValue/PseudoList
 * This is the ONLY place where Rust values should be wrapped in TS classes
 *
 * @param rustValue - WasmPseudoValue or WasmPseudoList from Rust
 * @param wkri - The WKRI wrapper (parent IRIVM)
 * @param model - The model wrapper (used to look up node objects by id for lists)
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
      list.__tileIds.push(wasmValue.tileId ?? null);
    }

    // What push/splice need to record edits: from now on they do.
    list.parent = list.parent ?? wkri;
    list.__alias = wasmList.nodeAlias;
    list.parenttileId = wasmValues.length > 0
      ? PseudoValue.fromWasm(wasmValues[0], wkri).getParentTileId() ?? undefined
      : undefined;
    list.__editable = true;

    return list;
  } else {
    // It's a single WasmPseudoValue - wrap using static factory
    return PseudoValue.fromWasm(rustValue as WasmPseudoValue, wkri);
  }
}

export { PseudoNode, PseudoValue, PseudoList, PseudoUnavailable, wrapRustPseudo };
