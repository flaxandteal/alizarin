# Migration Plan: Hierarchy Traversal to Rust with Structured Output

## Executive Summary

**Goal:** Move hierarchy construction logic to Rust as single source of truth, while keeping ViewModels (datatype-specific formatting) in each language (JavaScript/Python).

**Current State:** Rust returns flat `PseudoRecipe` list, JavaScript builds hierarchy from recipes.

**Target State:** Rust returns structured `RustPseudoValue` trees with full hierarchy, JavaScript/Python create thin wrappers that instantiate ViewModels on demand.

**Status:** Design complete, ready for implementation.

---

## Context: Work Already Completed

### What's Working Now

1. ✅ **valuesFromResourceNodegroup** - Rust handles tile traversal
2. ✅ **ensureNodegroup** - Rust handles nodegroup loading with recursion
3. ✅ **populate** - Rust orchestrates loading all nodegroups
4. ✅ **All tests passing** - 253 tests, including 25 comprehensive ensureNodegroup edge case tests

### Current Architecture

```
Rust (WASM/FFI)                    JavaScript/Python
┌─────────────────┐               ┌──────────────────┐
│ Graph Traversal │               │ Build Hierarchy  │
│ Returns:        │──recipes──────▶│ from recipes     │
│ Vec<Recipe>     │               │ Create ViewModels│
└─────────────────┘               └──────────────────┘
```

**PseudoRecipe** (current flat output):
```rust
struct PseudoRecipe {
    node_alias: String,        // "name", "basic_info"
    tile_id: Option<String>,   // "tile-123"
    node_id: String,
    is_collector: bool,
    sentinel_undefined: bool,
}
```

### Key Files (Current State)

- `src/instance_wrapper.rs` - Contains `populate()`, `ensureNodegroup()`, `valuesFromResourceNodegroup()`
- `src/model_wrapper.rs` - Graph metadata access
- `src/graph.rs` - Core graph types (StaticNode, StaticTile, StaticNodegroup)
- `js/graphManager.ts` - ResourceInstanceWrapper with recipe → hierarchy conversion
- `js/pseudos.ts` - PseudoValue, PseudoList classes with `makePseudoCls()`
- `js/semantic.ts` - SemanticViewModel (structural container)
- `js/viewModels.ts` - Datatype-specific ViewModels (String, Date, Concept, etc.)

---

## Target Architecture

```
Rust (WASM/FFI)                    JavaScript/Python
┌─────────────────┐               ┌──────────────────┐
│ Graph Traversal │               │ Thin Wrappers    │
│ Hierarchy Build │──structure────▶│ Create ViewModels│
│ Returns:        │               │ on demand        │
│ RustPseudoValue │               │                  │
│   (nested tree) │               │                  │
└─────────────────┘               └──────────────────┘
```

**RustPseudoValue** (target structured output):
```rust
struct RustPseudoValue {
    node: Arc<StaticNode>,              // Graph node metadata
    tile: Option<Arc<StaticTile>>,      // Tile reference
    tile_data: Option<serde_json::Value>, // Raw JSON (unparsed)
    children: Vec<RustPseudoValue>,     // Nested hierarchy!
    is_collector: bool,
    parent_index: Option<usize>,
}
```

### Benefits

1. **Single Source of Truth** - Hierarchy logic only in Rust
2. **Language Parity** - JS and Python get identical structures
3. **Clean Separation** - Structure (Rust) vs Types (JS/Python ViewModels)
4. **JSON I/O** - Natural in Rust with full structure
5. **Multi-Language Support** - Same Rust core for JS (WASM) and Python (PyO3)

### What Stays in Each Language

**Rust owns:**
- Graph traversal algorithms
- Hierarchy construction (parent/child relationships)
- Tile filtering and permissions
- State management (sentinels, implied nodegroups)
- Structure: PseudoList vs single PseudoValue

**JavaScript/Python own:**
- ViewModel creation (String, Date, Concept, etc.)
- Datatype-specific formatting (i18n, localization)
- Language idioms (Proxy in JS, descriptors in Python)
- Lazy property access patterns
- Update/save logic

---

## Phase 1: Design RustPseudoValue Structure

### 1.1 Define Core Rust Types

**Create:** `src/pseudo_value.rs` (new file)

```rust
use crate::graph::{StaticNode, StaticTile, StaticNodegroup};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Represents a value in the resource hierarchy
/// Analogous to JS PseudoValue but language-agnostic
#[derive(Clone, Serialize, Deserialize)]
pub struct RustPseudoValue {
    /// The graph node this represents
    pub node: Arc<StaticNode>,

    /// The tile containing data (if any)
    pub tile: Option<Arc<StaticTile>>,

    /// Raw tile data for this node (unparsed JSON)
    pub tile_data: Option<serde_json::Value>,

    /// Child nodes in the hierarchy
    pub children: Vec<RustPseudoValue>,

    /// Whether this is a collector node (array vs single)
    pub is_collector: bool,

    /// Index of parent in flat array (for efficient lookup)
    pub parent_index: Option<usize>,
}

/// Container for multiple PseudoValues sharing the same parent
/// Analogous to JS PseudoList
#[derive(Clone, Serialize, Deserialize)]
pub struct RustPseudoList {
    /// The collector node
    pub node: Arc<StaticNode>,

    /// Multiple items (one per tile)
    pub items: Vec<RustPseudoValue>,
}

/// Result from ensureNodegroup with structured output
#[derive(Clone)]
pub struct NodegroupResult {
    /// Root values keyed by alias
    pub values: HashMap<String, RustPseudoList>,

    /// Implied nodegroups discovered
    pub implied_nodegroups: Vec<String>,

    /// Updated nodegroup states
    pub nodegroup_states: HashMap<String, bool>,
}
```

**Design rationale:**
- `Arc<StaticNode>` - Shared ownership, no duplication of node metadata
- `Option<Arc<StaticTile>>` - Tiles can be shared across nodes
- `tile_data: Option<serde_json::Value>` - Raw JSON, Rust doesn't parse datatypes
- `Vec<RustPseudoValue>` - Owned children, builds the tree
- `parent_index` - Efficient parent lookup without lifetimes

### 1.2 Add WASM Bindings

**Add to:** `src/pseudo_value.rs`

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl RustPseudoValue {
    /// Get node alias
    #[wasm_bindgen(getter = nodeAlias)]
    pub fn node_alias(&self) -> Option<String> {
        self.node.alias.clone()
    }

    /// Get node ID
    #[wasm_bindgen(getter = nodeId)]
    pub fn node_id(&self) -> String {
        self.node.nodeid.clone()
    }

    /// Get node datatype
    #[wasm_bindgen(getter = datatype)]
    pub fn datatype(&self) -> String {
        self.node.datatype.clone()
    }

    /// Get tile data as JsValue
    #[wasm_bindgen(getter = tileData)]
    pub fn tile_data(&self) -> JsValue {
        match &self.tile_data {
            Some(data) => serde_wasm_bindgen::to_value(data).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Get tile reference
    #[wasm_bindgen(getter = tile)]
    pub fn tile(&self) -> JsValue {
        match &self.tile {
            Some(tile) => {
                // Return StaticTile as JsValue
                serde_wasm_bindgen::to_value(tile.as_ref()).unwrap_or(JsValue::NULL)
            },
            None => JsValue::NULL,
        }
    }

    /// Get children count
    #[wasm_bindgen(getter = childrenCount)]
    pub fn children_count(&self) -> usize {
        self.children.len()
    }

    /// Get child by index
    #[wasm_bindgen(js_name = getChild)]
    pub fn get_child(&self, index: usize) -> Option<RustPseudoValue> {
        self.children.get(index).cloned()
    }

    /// Get child by alias
    #[wasm_bindgen(js_name = getChildByAlias)]
    pub fn get_child_by_alias(&self, alias: &str) -> Option<RustPseudoValue> {
        self.children.iter()
            .find(|c| c.node.alias.as_deref() == Some(alias))
            .cloned()
    }

    /// Get all children as array
    #[wasm_bindgen(getter = children)]
    pub fn children(&self) -> Vec<RustPseudoValue> {
        self.children.clone()
    }

    /// Check if this is a collector (array)
    #[wasm_bindgen(getter = isCollector)]
    pub fn is_collector(&self) -> bool {
        self.is_collector
    }
}

#[wasm_bindgen]
impl RustPseudoList {
    /// Get the node this list represents
    #[wasm_bindgen(getter = node)]
    pub fn node(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.node.as_ref()).unwrap_or(JsValue::NULL)
    }

    /// Get items array
    #[wasm_bindgen(getter = items)]
    pub fn items(&self) -> Vec<RustPseudoValue> {
        self.items.clone()
    }

    /// Get item count
    #[wasm_bindgen(getter = length)]
    pub fn length(&self) -> usize {
        self.items.len()
    }
}
```

**Update:** `src/lib.rs`

Add new module:
```rust
pub mod pseudo_value;
```

---

## Phase 2: Port Hierarchy Construction Logic

### 2.1 Analysis: Current JS Logic to Port

**Key source files:**
- `js/pseudos.ts::makePseudoCls()` (lines 435-484) - Decides PseudoList vs PseudoValue
- `js/graphManager.ts::ensureNodegroup()` (lines 349-405) - Converts recipes to hierarchy

**Critical decisions made in JS:**
1. **Is it a list?** Check `is_collector && cardinality == "n"`
2. **Parent/child relationships** - Built from `edges` map (node_id → [child_node_ids])
3. **Tile grouping** - Multiple tiles for same node → array items
4. **Inner/outer nodes** - Semantic nodes can have nested structure

**Example hierarchy to build:**
```
resource
  └─ basic_info: PseudoList (cardinality="n")
      ├─ [0]: SemanticViewModel
      │   ├─ name: StringViewModel
      │   ├─ image: FileViewModel
      │   └─ source: StringViewModel
      └─ [1]: SemanticViewModel
          └─ name: StringViewModel
```

### 2.2 Implement Hierarchy Builder in Rust

**Add to:** `src/pseudo_value.rs`

```rust
impl RustPseudoValue {
    /// Build a PseudoValue from a node and tile
    /// PORT: js/pseudos.ts makePseudoCls (lines 435-484)
    pub fn from_node_and_tile(
        node: Arc<StaticNode>,
        tile: Option<Arc<StaticTile>>,
        node_objs: &HashMap<String, Arc<StaticNode>>,
        nodegroup_objs: &HashMap<String, Arc<StaticNodegroup>>,
        edges: &HashMap<String, Vec<String>>,
    ) -> Self {
        // Get tile data for this node (if tile exists)
        let tile_data = tile.as_ref()
            .and_then(|t| t.data.get(&node.nodeid))
            .cloned();

        // Determine if this is a collector
        let is_collector = node.is_collector;

        // Build children from edges
        let children = Self::build_children(
            &node,
            tile.as_ref(),
            node_objs,
            nodegroup_objs,
            edges,
        );

        RustPseudoValue {
            node,
            tile,
            tile_data,
            children,
            is_collector,
            parent_index: None,
        }
    }

    /// Build child nodes from edge relationships
    /// PORT: Logic from ensureNodegroup and valuesFromResourceNodegroup
    fn build_children(
        parent_node: &StaticNode,
        tile: Option<&Arc<StaticTile>>,
        node_objs: &HashMap<String, Arc<StaticNode>>,
        nodegroup_objs: &HashMap<String, Arc<StaticNodegroup>>,
        edges: &HashMap<String, Vec<String>>,
    ) -> Vec<RustPseudoValue> {
        let mut children = Vec::new();

        // Find child node IDs from edges
        if let Some(child_ids) = edges.get(&parent_node.nodeid) {
            for child_id in child_ids {
                if let Some(child_node) = node_objs.get(child_id) {
                    // Skip if different nodegroup (will be loaded separately via implied nodegroups)
                    if child_node.nodegroup_id != parent_node.nodegroup_id {
                        continue;
                    }

                    // Recursively build child
                    let child = Self::from_node_and_tile(
                        Arc::clone(child_node),
                        tile.cloned(),
                        node_objs,
                        nodegroup_objs,
                        edges,
                    );

                    children.push(child);
                }
            }
        }

        children
    }

    /// Group multiple tiles into a PseudoList
    /// PORT: Logic from ensureNodegroup where recipes are grouped by tile
    pub fn group_by_tiles(
        node: Arc<StaticNode>,
        tiles: Vec<Arc<StaticTile>>,
        node_objs: &HashMap<String, Arc<StaticNode>>,
        nodegroup_objs: &HashMap<String, Arc<StaticNodegroup>>,
        edges: &HashMap<String, Vec<String>>,
    ) -> RustPseudoList {
        let items = tiles.into_iter()
            .map(|tile| {
                Self::from_node_and_tile(
                    Arc::clone(&node),
                    Some(tile),
                    node_objs,
                    nodegroup_objs,
                    edges,
                )
            })
            .collect();

        RustPseudoList {
            node,
            items,
        }
    }
}
```

### 2.3 Update valuesFromResourceNodegroup

**Modify:** `src/instance_wrapper.rs`

**Current:** Returns `Vec<PseudoRecipe>`

**Change to:** Returns structured `HashMap<String, RustPseudoList>`

```rust
// Update the result struct
pub struct ValuesFromNodegroupResult {
    // OLD: recipes: Vec<PseudoRecipe>,
    // NEW:
    values: HashMap<String, RustPseudoList>,
    implied_nodegroups: Vec<String>,
}

// Update the implementation
pub fn values_from_resource_nodegroup(
    &self,
    existing_values_js: JsValue,
    nodegroup_tile_ids: Vec<String>,
    nodegroup_id: &str,
    node_objs_js: JsValue,
    edges_js: JsValue,
) -> Result<ValuesFromNodegroupResult, JsValue> {
    // ... existing tile filtering logic (keep this) ...

    // NEW: Build structured output instead of recipes
    let mut values: HashMap<String, RustPseudoList> = HashMap::new();

    // Group tiles by node alias
    let mut tiles_by_node: HashMap<String, Vec<Arc<StaticTile>>> = HashMap::new();

    for tile_opt in nodegroup_tiles {
        if let Some(tile) = tile_opt {
            // For each node in the tile data
            for (nodeid, node_value) in &tile.data {
                if let Some(node) = node_objs.get(nodeid) {
                    if let Some(alias) = &node.alias {
                        // Skip null values
                        if !node_value.is_null() {
                            tiles_by_node.entry(alias.clone())
                                .or_insert_with(Vec::new)
                                .push(Arc::clone(tile));
                        }
                    }
                }
            }
        }
    }

    // Build PseudoValues with hierarchy for each alias
    for (alias, tiles) in tiles_by_node {
        // Find the node by alias
        if let Some(node) = nodes_by_alias.get(&alias) {
            let pseudo_list = RustPseudoValue::group_by_tiles(
                Arc::clone(node),
                tiles,
                &node_objs,
                &nodegroup_objs,
                &edges,
            );
            values.insert(alias, pseudo_list);
        }
    }

    Ok(ValuesFromNodegroupResult {
        values,
        implied_nodegroups: implied_nodegroups_set.into_iter().collect(),
    })
}
```

---

## Phase 3: Update Return Types Throughout

### 3.1 Update ensureNodegroup

**Modify:** `src/instance_wrapper.rs`

```rust
// Update result struct
pub struct EnsureNodegroupResult {
    // OLD: recipes: Vec<PseudoRecipe>,
    // NEW:
    values: HashMap<String, RustPseudoList>,
    implied_nodegroups: Vec<String>,
    all_nodegroups_map: HashMap<String, serde_json::Value>,
}

#[wasm_bindgen]
impl EnsureNodegroupResult {
    #[wasm_bindgen(getter = values)]
    pub fn values(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.values).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = impliedNodegroups)]
    pub fn implied_nodegroups(&self) -> Vec<String> {
        self.implied_nodegroups.clone()
    }

    #[wasm_bindgen(getter = allNodegroupsMap)]
    pub fn all_nodegroups_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.all_nodegroups_map).unwrap_or(JsValue::NULL)
    }
}

// Update the ensure_nodegroup function to use values_from_resource_nodegroup's new output
pub fn ensure_nodegroup(
    &self,
    // ... parameters ...
) -> Result<EnsureNodegroupResult, JsValue> {
    // ... existing logic ...

    // Call values_from_resource_nodegroup (now returns structured data)
    let values_result = self.values_from_resource_nodegroup(
        all_values_js.clone(),
        nodegroup_tile_ids,
        nodegroup_id,
        node_objs_js.clone(),
        edges_js.clone(),
    )?;

    // Accumulate values (already structured)
    for (alias, pseudo_list) in values_result.values {
        all_values.insert(alias, pseudo_list);
    }

    // ... rest of logic ...

    Ok(EnsureNodegroupResult {
        values: all_values,
        implied_nodegroups: implied_nodegroups_set.into_iter().collect(),
        all_nodegroups_map: all_nodegroups,
    })
}
```

### 3.2 Update populate

**Modify:** `src/instance_wrapper.rs`

```rust
pub struct PopulateResult {
    // OLD: recipes: Vec<PseudoRecipe>,
    // NEW:
    values: HashMap<String, RustPseudoList>,
    all_values_map: HashMap<String, serde_json::Value>,
    all_nodegroups_map: HashMap<String, serde_json::Value>,
}

#[wasm_bindgen]
impl PopulateResult {
    #[wasm_bindgen(getter = values)]
    pub fn values(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.values).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = allValuesMap)]
    pub fn all_values_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.all_values_map).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = allNodegroupsMap)]
    pub fn all_nodegroups_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.all_nodegroups_map).unwrap_or(JsValue::NULL)
    }
}

// Update populate to aggregate structured results
pub fn populate(
    &self,
    // ... parameters ...
) -> Result<PopulateResult, JsValue> {
    // ... existing initialization ...

    let mut all_values: HashMap<String, RustPseudoList> = HashMap::new();

    // Phase 1 & 2 remain the same, but now accumulate structured values
    for nodegroup_id in nodegroup_ids.iter() {
        let result = self.ensure_nodegroup(/* ... */)?;

        // Merge structured values
        for (alias, pseudo_list) in result.values().iter() {
            all_values.insert(alias.clone(), pseudo_list.clone());
        }

        // ... rest of logic ...
    }

    Ok(PopulateResult {
        values: all_values,
        all_values_map: all_values.clone(), // For compatibility
        all_nodegroups_map: all_nodegroups,
    })
}
```

---

## Phase 4: Create JS Wrapper Layer

### 4.1 Create PseudoValue Wrapper

**Add to:** `js/pseudos.ts`

```typescript
/**
 * Wraps RustPseudoValue, creating ViewModels on demand
 * This replaces the recipe → PseudoValue conversion
 */
export class PseudoValueFromRust implements IPseudo {
    private rust: RustPseudoValue;
    private wkri: IRIVM<any>;
    private model: IModelWrapper<any>;
    private _value: IViewModel | null = null;
    private _accessed: boolean = false;

    constructor(
        rust: RustPseudoValue,
        wkri: IRIVM<any>,
        model: IModelWrapper<any>
    ) {
        this.rust = rust;
        this.wkri = wkri;
        this.model = model;
    }

    get node(): PseudoNode {
        // Return node info (could create PseudoNode or just return data)
        return {
            alias: this.rust.nodeAlias,
            datatype: this.rust.datatype,
            nodeid: this.rust.nodeId,
            name: this.rust.node?.name || '',
            // ... other node properties
        } as any; // TODO: Type properly
    }

    get tile(): StaticTile | null {
        return this.rust.tile;
    }

    get accessed(): boolean {
        return this._accessed;
    }

    set accessed(value: boolean) {
        this._accessed = value;
    }

    async getValue(): Promise<IViewModel> {
        if (this._value) {
            return this._value;
        }

        this._accessed = true;
        const datatype = this.rust.datatype;
        const tileData = this.rust.tileData;

        if (datatype === "semantic") {
            // Create semantic wrapper that exposes children
            this._value = new SemanticViewModelFromRust(
                this.rust,
                this.wkri,
                this.model
            );
        } else {
            // Create datatype-specific ViewModel using existing getViewModel
            this._value = await getViewModel(
                this,  // parentPseudo
                this.rust.tile,
                this.node as any, // TODO: Type properly
                tileData,
                this.wkri,
                new Map(), // childNodes - not needed for leaf nodes
                false // isInner
            );
        }

        return this._value;
    }

    async forJson() {
        const value = await this.getValue();
        if (value && typeof value.forJson === 'function') {
            return value.forJson();
        }
        return null;
    }

    // Implement other IPseudo methods as needed
    describeField(): string {
        return this.node.name || this.node.alias || '';
    }

    describeFieldGroup(): string {
        return this.describeField();
    }
}
```

### 4.2 Create SemanticViewModel Wrapper

**Add to:** `js/semantic.ts`

```typescript
/**
 * Wraps RustPseudoValue for semantic nodes
 * Provides dynamic property access to children
 */
class SemanticViewModelFromRust implements IStringKeyedObject, IViewModel {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined = undefined;
    then: undefined;
    [Symbol.toPrimitive]: undefined;

    __parentPseudo: PseudoValue<any> | undefined;

    private rust: RustPseudoValue;
    private wkri: IRIVM<any>;
    private model: IModelWrapper<any>;
    private _childCache: Map<string, PseudoValueFromRust> = new Map();

    __forJsonCache(): null {
        return null;
    }

    constructor(
        rust: RustPseudoValue,
        wkri: IRIVM<any>,
        model: IModelWrapper<any>
    ) {
        this.rust = rust;
        this.wkri = wkri;
        this.model = model;

        // Return Proxy for dynamic property access
        return new Proxy(this, {
            get: (target, key: string | symbol) => {
                // Handle Symbol.toStringTag
                if (key.toString() === "Symbol.toStringTag") {
                    return () => "SemanticViewModelFromRust";
                }

                // If property exists on target, return it
                if (key in target) {
                    return target[key];
                }

                const k = typeof key === 'symbol' ? key.description || '' : key;

                // Handle special properties
                if (k.startsWith('__') || k === 'length') {
                    return target[k];
                }

                // Look up child by alias
                return new AttrPromise((resolve) => {
                    target._getChild(k).then(resolve);
                });
            },

            set: (target, key: string | symbol, value) => {
                const k = typeof key === 'symbol' ? key.description || '' : key;

                if (key in target || k.startsWith('__')) {
                    target[k] = value;
                } else {
                    target._setChild(k, value);
                }
                return true;
            }
        });
    }

    async _getChild(alias: string): Promise<IViewModel | null> {
        // Check cache
        if (this._childCache.has(alias)) {
            return this._childCache.get(alias)!.getValue();
        }

        // Get child from Rust
        const rustChild = this.rust.getChildByAlias(alias);
        if (!rustChild) {
            // Child not found - may need to load nodegroup
            // For now, return null. TODO: Trigger loading if needed
            return null;
        }

        // Wrap and cache
        const wrapped = new PseudoValueFromRust(
            rustChild,
            this.wkri,
            this.model
        );
        this._childCache.set(alias, wrapped);

        return wrapped.getValue();
    }

    async _setChild(alias: string, value: any) {
        // Handle updates - need to update Rust tree
        // TODO: Implement update logic
        console.warn('Setting semantic properties not yet implemented:', alias, value);
    }

    async toString(): Promise<string> {
        const entries: string[] = [];
        for (let i = 0; i < this.rust.childrenCount; i++) {
            const child = this.rust.getChild(i);
            if (child && child.nodeAlias) {
                const wrapped = new PseudoValueFromRust(child, this.wkri, this.model);
                const value = await wrapped.getValue();
                entries.push(`${child.nodeAlias}: ${value}`);
            }
        }
        return `[[${entries.join(",")}]]`;
    }

    async toObject() {
        const result: any = {};
        for (let i = 0; i < this.rust.childrenCount; i++) {
            const child = this.rust.getChild(i);
            if (child && child.nodeAlias) {
                const wrapped = new PseudoValueFromRust(child, this.wkri, this.model);
                result[child.nodeAlias] = await wrapped.getValue();
            }
        }
        return result;
    }

    async forJson() {
        const result: any = {};

        async function _forJson(v: IPseudo | IViewModel | null) {
            v = await v;
            if (!v) {
                return null;
            }
            if (typeof v.forJson === 'function') {
                return await v.forJson();
            }
            return v;
        }

        for (let i = 0; i < this.rust.childrenCount; i++) {
            const child = this.rust.getChild(i);
            if (child && child.nodeAlias) {
                const wrapped = new PseudoValueFromRust(child, this.wkri, this.model);
                result[child.nodeAlias] = await _forJson(wrapped);
            }
        }

        return result;
    }
}

export { SemanticViewModelFromRust };
```

### 4.3 Update graphManager.ts

**Modify:** `js/graphManager.ts::ensureNodegroup()`

Replace recipe conversion logic with wrapper creation:

```typescript
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

    const enableParallelTesting = process.env.ALIZARIN_PARALLEL_TEST === "true";

    try {
        // ... existing tile permissions logic (keep) ...

        // Call Rust implementation
        const result = this.wasmWrapper.ensureNodegroup(
            allValues,
            allNodegroups,
            nodegroupId,
            nodeObjs,
            nodegroupObjs,
            edges,
            addIfMissing,
            allTiles,
            tilePermissions,
            doImpliedNodegroups
        );

        // NEW: Wrap structured output instead of converting recipes
        const newValues = new Map<string, any>();

        // result.values is HashMap<String, RustPseudoList>
        for (const [alias, rustPseudoList] of Object.entries(result.values)) {
            // Wrap each item in the list
            const wrappedItems = rustPseudoList.items.map((rustPseudo: RustPseudoValue) =>
                new PseudoValueFromRust(rustPseudo, this.wkri, this.model)
            );

            newValues.set(alias, wrappedItems);
        }

        // Update allValues with newValues
        for (const [key, value] of newValues.entries()) {
            if (value !== undefined) {
                allValues.set(key, value);
            }
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

        const impliedNodegroups = new Set(result.impliedNodegroups);

        // ... parallel testing logic (update to compare structures) ...

        return [newValues, impliedNodegroups];

    } catch (error) {
        console.error('[ensureNodegroup] Rust implementation failed, falling back to JS:', error);
        return this.ensureNodegroup_JS(
            allValues,
            allNodegroups,
            nodegroupId,
            nodeObjs,
            nodegroupObjs,
            edges,
            addIfMissing,
            tiles,
            doImpliedNodegroups
        );
    }
}
```

**Similarly update:** `populate()` method

```typescript
async populate(lazy: boolean): Promise<void> {
    // ... existing setup ...

    try {
        // ... tile preparation ...

        const result = this.wasmWrapper.populate(
            lazy,
            nodegroupIds,
            rootNode.alias,
            nodeObjs,
            nodegroupObjs,
            edges,
            allTiles,
            tilePermissions
        );

        // NEW: Wrap structured output
        const allValues: Map<string, any> = new Map();
        const allNodegroups: Map<string, any> = new Map();

        // Initialize allNodegroups
        for (const nodegroupId of nodegroupIds) {
            allNodegroups.set(nodegroupId, false);
        }

        // Set root
        allValues.set(rootNode.alias, false);

        // Wrap all values from Rust
        for (const [alias, rustPseudoList] of Object.entries(result.values)) {
            const wrappedItems = rustPseudoList.items.map((rustPseudo: RustPseudoValue) =>
                new PseudoValueFromRust(rustPseudo, this.wkri, this.model)
            );
            allValues.set(alias, wrappedItems);
        }

        // Update nodegroup states
        const updatedNodegroups = result.allNodegroupsMap;
        // ... (same as ensureNodegroup) ...

        // Create ValueList
        this.valueList = new ValueList(
            allValues,
            allNodegroups,
            this,
            this.resource ? this.resource.tiles : null,
        );

    } catch (error) {
        console.error('[populate] Rust implementation failed, falling back to JS:', error);
        return this.populate_JS(lazy);
    }
}
```

---

## Phase 5: Python Bindings (PyO3)

### 5.1 Create PyO3 Module

**Create:** `src/python_bindings.rs` (new file)

```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use crate::pseudo_value::{RustPseudoValue, RustPseudoList};
use crate::instance_wrapper::ResourceInstanceWrapper;
use std::sync::Arc;
use std::collections::HashMap;

/// Python wrapper for RustPseudoValue
/// Uses Arc for shared ownership with Python GC
#[pyclass(name = "PseudoValue")]
pub struct PyPseudoValue {
    inner: Arc<RustPseudoValue>,
}

#[pymethods]
impl PyPseudoValue {
    #[getter]
    fn node_alias(&self) -> Option<String> {
        self.inner.node.alias.clone()
    }

    #[getter]
    fn node_id(&self) -> String {
        self.inner.node.nodeid.clone()
    }

    #[getter]
    fn datatype(&self) -> String {
        self.inner.node.datatype.clone()
    }

    #[getter]
    fn tile_data(&self, py: Python) -> PyResult<PyObject> {
        match &self.inner.tile_data {
            Some(data) => {
                // Convert serde_json::Value to Python object
                Ok(pythonize::pythonize(py, data)?)
            },
            None => Ok(py.None()),
        }
    }

    #[getter]
    fn children(&self) -> Vec<PyPseudoValue> {
        self.inner.children.iter()
            .map(|c| PyPseudoValue {
                inner: Arc::new(c.clone())
            })
            .collect()
    }

    fn get_child_by_alias(&self, alias: &str) -> Option<PyPseudoValue> {
        self.inner.children.iter()
            .find(|c| c.node.alias.as_deref() == Some(alias))
            .map(|c| PyPseudoValue {
                inner: Arc::new(c.clone())
            })
    }

    #[getter]
    fn is_collector(&self) -> bool {
        self.inner.is_collector
    }

    #[getter]
    fn children_count(&self) -> usize {
        self.inner.children.len()
    }
}

/// Python wrapper for RustPseudoList
#[pyclass(name = "PseudoList")]
pub struct PyPseudoList {
    inner: Arc<RustPseudoList>,
}

#[pymethods]
impl PyPseudoList {
    #[getter]
    fn items(&self) -> Vec<PyPseudoValue> {
        self.inner.items.iter()
            .map(|item| PyPseudoValue {
                inner: Arc::new(item.clone())
            })
            .collect()
    }

    fn __len__(&self) -> usize {
        self.inner.items.len()
    }

    fn __getitem__(&self, index: usize) -> PyResult<PyPseudoValue> {
        self.inner.items.get(index)
            .map(|item| PyPseudoValue {
                inner: Arc::new(item.clone())
            })
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("Index out of range"))
    }
}

/// Python wrapper for resource instance
#[pyclass(name = "ResourceInstance")]
pub struct PyResourceInstance {
    wrapper: ResourceInstanceWrapper,
}

#[pymethods]
impl PyResourceInstance {
    #[new]
    fn new(
        graph_json: &str,
        tiles_json: &str,
        resource_id: &str,
    ) -> PyResult<Self> {
        // TODO: Initialize ResourceInstanceWrapper from JSON
        // This requires deserializing graph and tiles
        todo!("Initialize from JSON")
    }

    fn ensure_nodegroup(
        &mut self,
        py: Python,
        nodegroup_id: &str,
    ) -> PyResult<PyObject> {
        // TODO: Call Rust ensureNodegroup and wrap results
        // Returns dict of { alias: [PyPseudoValue] }
        todo!("Call ensure_nodegroup")
    }

    fn populate(&mut self, lazy: bool) -> PyResult<PyObject> {
        // TODO: Call Rust populate
        todo!("Call populate")
    }

    fn to_json(&self) -> PyResult<String> {
        // TODO: Serialize to JSON
        todo!("Serialize to JSON")
    }

    #[staticmethod]
    fn from_json(json_str: &str) -> PyResult<Self> {
        // TODO: Deserialize from JSON
        todo!("Deserialize from JSON")
    }
}

/// The Python module
#[pymodule]
fn alizarin_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyPseudoValue>()?;
    m.add_class::<PyPseudoList>()?;
    m.add_class::<PyResourceInstance>()?;
    Ok(())
}
```

**Update:** `Cargo.toml`

Add PyO3 feature:
```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Existing dependencies...
pyo3 = { version = "0.20", features = ["extension-module"], optional = true }
pythonize = { version = "0.20", optional = true }

[features]
default = []
python = ["pyo3", "pythonize"]
```

### 5.2 Create Python Wrapper Classes

**Create:** `python/alizarin/__init__.py`

```python
"""
Alizarin - Arches ORM Python bindings
Wraps Rust core with Python-friendly interface
"""

from typing import Any, Dict, List, Optional, Union
try:
    import alizarin_core  # Rust module built with PyO3
except ImportError:
    raise ImportError(
        "alizarin_core Rust module not found. "
        "Build with: maturin develop --features python"
    )


class PseudoValue:
    """Python wrapper for Rust PseudoValue

    Provides lazy loading of ViewModels from Rust structure.
    """

    def __init__(self, rust_pseudo: alizarin_core.PseudoValue, wkri: 'ResourceInstance'):
        self._rust = rust_pseudo
        self._wkri = wkri
        self._value = None

    @property
    def node_alias(self) -> Optional[str]:
        return self._rust.node_alias

    @property
    def datatype(self) -> str:
        return self._rust.datatype

    @property
    def tile_data(self) -> Any:
        return self._rust.tile_data

    @property
    def value(self):
        """Get or create ViewModel for this PseudoValue"""
        if self._value is None:
            if self._rust.datatype == "semantic":
                self._value = SemanticViewModel(self._rust, self._wkri)
            else:
                # Create Python ViewModel based on datatype
                self._value = create_view_model(
                    self._rust.datatype,
                    self._rust.tile_data
                )
        return self._value

    def __repr__(self):
        return f"PseudoValue(alias={self.node_alias}, datatype={self.datatype})"


class SemanticViewModel:
    """Python equivalent of SemanticViewModel

    Provides dynamic attribute access to child nodes.
    """

    def __init__(self, rust_pseudo: alizarin_core.PseudoValue, wkri: 'ResourceInstance'):
        self._rust = rust_pseudo
        self._wkri = wkri
        self._child_cache: Dict[str, Any] = {}

    def __getattr__(self, name: str):
        # Avoid infinite recursion for internal attributes
        if name.startswith('_'):
            return object.__getattribute__(self, name)

        # Check cache
        if name in self._child_cache:
            return self._child_cache[name]

        # Look up child in Rust
        rust_child = self._rust.get_child_by_alias(name)
        if rust_child:
            wrapped = PseudoValue(rust_child, self._wkri)
            self._child_cache[name] = wrapped.value
            return self._child_cache[name]

        raise AttributeError(f"SemanticViewModel has no attribute '{name}'")

    def __setattr__(self, name: str, value: Any):
        if name.startswith('_'):
            object.__setattr__(self, name, value)
        else:
            # TODO: Implement setting values
            raise NotImplementedError("Setting semantic properties not yet implemented")

    def to_dict(self) -> Dict[str, Any]:
        """Convert to Python dictionary"""
        result = {}
        for child in self._rust.children:
            alias = child.node_alias
            if alias:
                wrapped = PseudoValue(child, self._wkri)
                result[alias] = wrapped.value
        return result

    def __repr__(self):
        children = [c.node_alias for c in self._rust.children if c.node_alias]
        return f"SemanticViewModel(children={children})"


class ResourceInstance:
    """Python wrapper for Rust ResourceInstance

    Main entry point for loading and accessing Arches resources.
    """

    def __init__(self, graph_id: str, resource_id: str):
        # TODO: Load graph and tiles from database/API
        # For now, placeholder
        graph_json = "{}"  # load_graph(graph_id)
        tiles_json = "[]"  # load_tiles(resource_id)

        self._rust = alizarin_core.ResourceInstance(
            graph_json,
            tiles_json,
            resource_id
        )
        self._values: Dict[str, Any] = {}

    def __getattr__(self, name: str):
        if name.startswith('_'):
            return object.__getattribute__(self, name)

        # Lazy load nodegroup
        if name not in self._values:
            result = self._rust.ensure_nodegroup(name)

            # Wrap results
            wrapped = {}
            for alias, pseudo_list in result.items():
                wrapped[alias] = [
                    PseudoValue(pv, self)
                    for pv in pseudo_list.items
                ]

            self._values.update(wrapped)

        return self._values.get(name)

    def to_json(self) -> str:
        """Export resource to JSON"""
        return self._rust.to_json()

    @classmethod
    def from_json(cls, json_str: str) -> 'ResourceInstance':
        """Create resource from JSON"""
        rust_instance = alizarin_core.ResourceInstance.from_json(json_str)
        # TODO: Wrap properly
        instance = cls.__new__(cls)
        instance._rust = rust_instance
        instance._values = {}
        return instance

    def __repr__(self):
        return f"ResourceInstance(id={getattr(self._rust, 'id', 'unknown')})"


def create_view_model(datatype: str, data: Any) -> Any:
    """Create Python ViewModel based on datatype

    This is where language-specific type handling happens.
    """
    # TODO: Implement Python ViewModels for each datatype
    # For now, return raw data
    if datatype == "string":
        # StringViewModel - handle i18n, etc.
        return data  # Placeholder
    elif datatype == "date":
        # DateViewModel - parse to datetime
        return data  # Placeholder
    elif datatype == "number":
        return data
    elif datatype == "boolean":
        return data
    else:
        # Generic - return raw
        return data


__all__ = ['PseudoValue', 'SemanticViewModel', 'ResourceInstance']
```

**Create:** `python/setup.py` or `pyproject.toml`

```toml
# pyproject.toml for maturin
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "alizarin"
version = "0.1.0"
description = "Arches ORM with Rust core"
requires-python = ">=3.8"

[tool.maturin]
features = ["python"]
```

---

## Phase 6: JSON Serialization

### 6.1 Implement to_json in Rust

**Add to:** `src/pseudo_value.rs`

```rust
impl RustPseudoValue {
    /// Convert to nested JSON representation
    /// Used for: API output, database storage, data export
    pub fn to_json(&self) -> serde_json::Value {
        if self.children.is_empty() {
            // Leaf node - return raw tile data
            self.tile_data.clone().unwrap_or(serde_json::Value::Null)
        } else {
            // Branch node (semantic) - build object from children
            let mut obj = serde_json::Map::new();

            for child in &self.children {
                if let Some(alias) = &child.node.alias {
                    obj.insert(alias.clone(), child.to_json());
                }
            }

            serde_json::Value::Object(obj)
        }
    }
}

impl RustPseudoList {
    /// Convert list to JSON array
    pub fn to_json(&self) -> serde_json::Value {
        let array: Vec<serde_json::Value> = self.items.iter()
            .map(|item| item.to_json())
            .collect();

        serde_json::Value::Array(array)
    }
}

impl NodegroupResult {
    /// Convert all values to JSON object
    pub fn to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        for (alias, pseudo_list) in &self.values {
            obj.insert(alias.clone(), pseudo_list.to_json());
        }

        serde_json::Value::Object(obj)
    }
}
```

**Add WASM binding:**

```rust
#[wasm_bindgen]
impl RustPseudoValue {
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self.to_json()).unwrap_or_default()
    }
}

#[wasm_bindgen]
impl RustPseudoList {
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self.to_json()).unwrap_or_default()
    }
}
```

### 6.2 Implement from_json in Rust

**Add to:** `src/pseudo_value.rs`

```rust
impl RustPseudoValue {
    /// Create PseudoValue hierarchy from nested JSON
    /// Used for: importing data, hydrating from API, testing
    pub fn from_json(
        json: &serde_json::Value,
        node: Arc<StaticNode>,
        node_objs: &HashMap<String, Arc<StaticNode>>,
        edges: &HashMap<String, Vec<String>>,
    ) -> Result<Self, String> {
        // Determine if this is a leaf or branch node
        let has_children = edges.contains_key(&node.nodeid);

        let (tile_data, children) = if has_children {
            // Branch node - recurse into children
            let child_ids = edges.get(&node.nodeid).unwrap();

            let mut children = Vec::new();

            if let serde_json::Value::Object(obj) = json {
                for child_id in child_ids {
                    if let Some(child_node) = node_objs.get(child_id) {
                        if let Some(alias) = &child_node.alias {
                            if let Some(child_json) = obj.get(alias) {
                                let child = Self::from_json(
                                    child_json,
                                    Arc::clone(child_node),
                                    node_objs,
                                    edges,
                                )?;
                                children.push(child);
                            }
                        }
                    }
                }
            }

            (None, children)
        } else {
            // Leaf node - store data
            (Some(json.clone()), Vec::new())
        };

        Ok(RustPseudoValue {
            node,
            tile: None,  // No tile when importing from JSON
            tile_data,
            children,
            is_collector: false,  // Will be set based on graph metadata
            parent_index: None,
        })
    }
}

impl RustPseudoList {
    /// Create PseudoList from JSON array
    pub fn from_json(
        json: &serde_json::Value,
        node: Arc<StaticNode>,
        node_objs: &HashMap<String, Arc<StaticNode>>,
        edges: &HashMap<String, Vec<String>>,
    ) -> Result<Self, String> {
        let items = if let serde_json::Value::Array(arr) = json {
            arr.iter()
                .map(|item_json| {
                    RustPseudoValue::from_json(
                        item_json,
                        Arc::clone(&node),
                        node_objs,
                        edges,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            return Err("Expected JSON array for PseudoList".to_string());
        };

        Ok(RustPseudoList {
            node,
            items,
        })
    }
}
```

**Add WASM binding:**

```rust
#[wasm_bindgen]
impl RustPseudoValue {
    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json_string(
        json_str: &str,
        node_js: JsValue,
        node_objs_js: JsValue,
        edges_js: JsValue,
    ) -> Result<RustPseudoValue, JsValue> {
        let json: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let node: StaticNode = serde_wasm_bindgen::from_value(node_js)?;
        let node_objs: HashMap<String, StaticNode> =
            serde_wasm_bindgen::from_value(node_objs_js)?;
        let edges: HashMap<String, Vec<String>> =
            serde_wasm_bindgen::from_value(edges_js)?;

        // Convert to Arc
        let node_objs_arc: HashMap<String, Arc<StaticNode>> =
            node_objs.into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect();

        Self::from_json(&json, Arc::new(node), &node_objs_arc, &edges)
            .map_err(|e| JsValue::from_str(&e))
    }
}
```

---

## Testing Strategy

### Step 1: Unit Tests in Rust

**Create:** `src/pseudo_value.rs` (at end of file)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_node(id: &str, alias: &str, datatype: &str) -> Arc<StaticNode> {
        Arc::new(StaticNode {
            nodeid: id.to_string(),
            alias: Some(alias.to_string()),
            datatype: datatype.to_string(),
            is_collector: false,
            graph_id: "test-graph".to_string(),
            name: alias.to_string(),
            exportable: true,
            hascustomalias: false,
            isrequired: false,
            issearchable: false,
            istopnode: false,
            nodegroup_id: Some("test-ng".to_string()),
            config: HashMap::new(),
            description: None,
            fieldname: None,
            ontologyclass: None,
            parentproperty: None,
            sortorder: 0,
            sourcebranchpublication_id: None,
        })
    }

    #[test]
    fn test_build_simple_hierarchy() {
        // Create nodes
        let root_node = create_test_node("root", "basic_info", "semantic");
        let child_node = create_test_node("child", "name", "string");

        // Create edges
        let mut edges = HashMap::new();
        edges.insert("root".to_string(), vec!["child".to_string()]);

        let mut node_objs = HashMap::new();
        node_objs.insert("root".to_string(), Arc::clone(&root_node));
        node_objs.insert("child".to_string(), Arc::clone(&child_node));

        let nodegroup_objs = HashMap::new();

        // Build hierarchy
        let pseudo = RustPseudoValue::from_node_and_tile(
            root_node,
            None,
            &node_objs,
            &nodegroup_objs,
            &edges,
        );

        // Verify structure
        assert_eq!(pseudo.children.len(), 1);
        assert_eq!(pseudo.children[0].node.alias, Some("name".to_string()));
    }

    #[test]
    fn test_json_roundtrip() {
        // Create test structure
        let root_node = create_test_node("root", "basic_info", "semantic");
        let child_node = create_test_node("child", "name", "string");

        let mut edges = HashMap::new();
        edges.insert("root".to_string(), vec!["child".to_string()]);

        let mut node_objs = HashMap::new();
        node_objs.insert("root".to_string(), Arc::clone(&root_node));
        node_objs.insert("child".to_string(), Arc::clone(&child_node));

        let nodegroup_objs = HashMap::new();

        // Create with data
        let mut child = RustPseudoValue::from_node_and_tile(
            child_node,
            None,
            &node_objs,
            &nodegroup_objs,
            &edges,
        );
        child.tile_data = Some(serde_json::json!({
            "en": {"value": "Test Name", "direction": "ltr"}
        }));

        let mut pseudo = RustPseudoValue::from_node_and_tile(
            root_node,
            None,
            &node_objs,
            &nodegroup_objs,
            &edges,
        );
        pseudo.children = vec![child];

        // Convert to JSON
        let json = pseudo.to_json();

        // Verify structure
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("name"));

        // Convert back
        let restored = RustPseudoValue::from_json(
            &json,
            pseudo.node.clone(),
            &node_objs,
            &edges,
        ).unwrap();

        // Verify matches
        assert_eq!(pseudo.children.len(), restored.children.len());
        assert_eq!(
            pseudo.children[0].node.alias,
            restored.children[0].node.alias
        );
    }

    #[test]
    fn test_group_by_tiles() {
        // Create test nodes
        let node = create_test_node("basic_info", "basic_info", "semantic");

        // Create test tiles
        let tile1 = Arc::new(StaticTile {
            tileid: Some("tile-1".to_string()),
            nodegroup_id: "test-ng".to_string(),
            data: HashMap::new(),
            sortorder: 0,
            resourceinstance_id: "res-1".to_string(),
            parenttile_id: None,
            provisionaledits: None,
        });

        let tile2 = Arc::new(StaticTile {
            tileid: Some("tile-2".to_string()),
            nodegroup_id: "test-ng".to_string(),
            data: HashMap::new(),
            sortorder: 1,
            resourceinstance_id: "res-1".to_string(),
            parenttile_id: None,
            provisionaledits: None,
        });

        let tiles = vec![tile1, tile2];

        let node_objs = HashMap::new();
        let nodegroup_objs = HashMap::new();
        let edges = HashMap::new();

        // Group tiles
        let pseudo_list = RustPseudoValue::group_by_tiles(
            node,
            tiles,
            &node_objs,
            &nodegroup_objs,
            &edges,
        );

        // Verify
        assert_eq!(pseudo_list.items.len(), 2);
    }
}
```

Run tests:
```bash
cargo test --lib
```

### Step 2: Integration Tests (JavaScript)

**Create:** `tests/rust-structured-output.test.ts`

```typescript
import { describe, it, expect, beforeAll } from 'vitest';
import { ResourceModelWrapper, ResourceInstanceWrapper } from '../js/graphManager';
import { PseudoValueFromRust } from '../js/pseudos';
import { GraphMutator } from './graphMutator';
import { initWasmForTests } from './wasm-init';

describe("Rust Structured Output Integration", () => {
    beforeAll(async () => {
        await initWasmForTests();
    });

    it("should return structured hierarchy from ensureNodegroup", async () => {
        // Create test graph
        const mutator = new GraphMutator();
        const graph = mutator.createGraph("Test Graph");
        const ng = mutator.addNodegroup(graph, "basic_info", "n");
        const nameNode = mutator.addNode(ng, "name", "string");

        const model = new ResourceModelWrapper(
            mutator.createWKRM(graph),
            graph
        );

        // Create resource with tile
        const resource = {
            resourceinstanceid: "test-resource",
            graph_id: graph.graphid,
            tiles: [{
                tileid: "tile-1",
                nodegroup_id: ng.nodegroupid,
                data: new Map([[nameNode.nodeid, { en: { value: "Test", direction: "ltr" }}]]),
                sortorder: 0,
                resourceinstance_id: "test-resource",
                parenttile_id: null,
                provisionaledits: null,
            }],
        };

        const mockRIVM: any = {};
        const wrapper = new ResourceInstanceWrapper(mockRIVM, model, resource as any, false);
        mockRIVM.$ = wrapper;
        mockRIVM.__ = model;

        // Call ensureNodegroup
        const allValues = new Map();
        const allNodegroups = new Map([[ng.nodegroupid, false]]);

        const [newValues, impliedNodegroups] = await wrapper.ensureNodegroup(
            allValues,
            allNodegroups,
            ng.nodegroupid,
            model.getNodeObjects(),
            model.getNodegroupObjects(),
            model.getEdges(),
            true,
            resource.tiles,
            false
        );

        // Verify structured output
        expect(newValues.has("basic_info")).toBe(true);
        const basicInfoList = newValues.get("basic_info");
        expect(Array.isArray(basicInfoList)).toBe(true);
        expect(basicInfoList.length).toBe(1);

        // Verify it's a PseudoValueFromRust wrapper
        const firstItem = basicInfoList[0];
        expect(firstItem).toBeInstanceOf(PseudoValueFromRust);

        // Access nested property
        const value = await firstItem.getValue();
        expect(value).toBeDefined();
    });

    it("should match JS implementation output", async () => {
        // TODO: Enable parallel testing
        // Compare structured output from Rust vs JS
    });

    it("should handle nested semantic nodes", async () => {
        // Create graph with nested structure
        const mutator = new GraphMutator();
        const graph = mutator.createGraph("Test Graph");
        const parentNg = mutator.addNodegroup(graph, "parent", "n");
        const childNg = mutator.addNodegroup(graph, "child", "n");

        const parentNode = mutator.addNode(parentNg, "parent_semantic", "semantic");
        const childNode = mutator.addNode(childNg, "child_value", "string");

        // Add edge parent -> child
        mutator.addEdge(graph, parentNode.nodeid, childNode.nodeid);

        // Create resource with tiles
        // ... test nested access
    });
});
```

Run tests:
```bash
npm test -- tests/rust-structured-output.test.ts
```

### Step 3: Python Tests

**Create:** `python/tests/test_python_bindings.py`

```python
import pytest
from alizarin import ResourceInstance, PseudoValue, SemanticViewModel


def test_pseudo_value_wrapper():
    """Test PseudoValue wraps Rust correctly"""
    # TODO: Create test resource
    resource = create_test_resource()

    # Access property
    basic_info = resource.basic_info
    assert basic_info is not None
    assert isinstance(basic_info, list)
    assert len(basic_info) > 0


def test_semantic_view_model():
    """Test SemanticViewModel dynamic attributes"""
    resource = create_test_resource()

    # Access nested property
    basic_info_item = resource.basic_info[0]
    assert isinstance(basic_info_item, SemanticViewModel)

    # Access child
    name = basic_info_item.name
    assert name is not None


def test_json_roundtrip():
    """Test JSON export/import"""
    resource = create_test_resource()

    # Export to JSON
    json_str = resource.to_json()
    assert json_str

    import json
    json_data = json.loads(json_str)

    # Verify structure
    assert "basic_info" in json_data
    assert isinstance(json_data["basic_info"], list)

    # Import back
    restored = ResourceInstance.from_json(json_str)
    assert restored is not None


def test_lazy_loading():
    """Test that nodegroups are loaded on demand"""
    resource = create_test_resource()

    # First access should trigger loading
    basic_info = resource.basic_info

    # Second access should use cache
    basic_info2 = resource.basic_info

    assert basic_info is basic_info2  # Same object


def create_test_resource():
    """Helper to create test resource"""
    # TODO: Implement
    graph_id = "test-graph"
    resource_id = "test-resource"
    return ResourceInstance(graph_id, resource_id)
```

Run tests:
```bash
pytest python/tests/
```

### Step 4: Validation Against Current Implementation

**Enable parallel testing:**

Set environment variable:
```bash
export ALIZARIN_PARALLEL_TEST=true
npm test
```

This runs both Rust (new structured) and JS (old recipe-based) implementations in parallel and compares results.

---

## Migration Checklist

### Phase 1: Design ✓
- [ ] Create `src/pseudo_value.rs`
- [ ] Define `RustPseudoValue` struct
- [ ] Define `RustPseudoList` struct
- [ ] Define `NodegroupResult` struct
- [ ] Add WASM bindings for all structs
- [ ] Add to `src/lib.rs` module exports
- [ ] Write Rust unit tests for structure creation

### Phase 2: Port Hierarchy Logic
- [ ] Implement `RustPseudoValue::from_node_and_tile()`
- [ ] Implement `RustPseudoValue::build_children()`
- [ ] Implement `RustPseudoValue::group_by_tiles()`
- [ ] Add unit tests for hierarchy construction
- [ ] Test with simple graph (parent + child)
- [ ] Test with complex graph (multiple levels)

### Phase 3: Update Rust Functions
- [ ] Update `ValuesFromNodegroupResult` struct
- [ ] Modify `values_from_resource_nodegroup()` to return structured data
- [ ] Update `EnsureNodegroupResult` struct
- [ ] Modify `ensureNodegroup()` to use new result type
- [ ] Update `PopulateResult` struct
- [ ] Modify `populate()` to aggregate structured results
- [ ] Add WASM bindings for all result types
- [ ] Test Rust functions in isolation

### Phase 4: JavaScript Wrappers
- [ ] Create `PseudoValueFromRust` class in `js/pseudos.ts`
- [ ] Create `SemanticViewModelFromRust` class in `js/semantic.ts`
- [ ] Update `graphManager.ts::ensureNodegroup()` to wrap Rust output
- [ ] Update `graphManager.ts::populate()` to wrap Rust output
- [ ] Keep `ensureNodegroup_JS()` and `populate_JS()` for parallel testing
- [ ] Create integration tests
- [ ] Test property access works (resource.basic_info[0].name)
- [ ] Test lazy loading still works

### Phase 5: Python Bindings
- [ ] Create `src/python_bindings.rs`
- [ ] Implement `PyPseudoValue` class
- [ ] Implement `PyPseudoList` class
- [ ] Implement `PyResourceInstance` class
- [ ] Add `python` feature to `Cargo.toml`
- [ ] Create `python/alizarin/__init__.py`
- [ ] Implement Python wrapper classes
- [ ] Create `pyproject.toml` for maturin
- [ ] Build Python module: `maturin develop --features python`
- [ ] Write Python tests
- [ ] Test dynamic attribute access works

### Phase 6: JSON Serialization
- [ ] Implement `RustPseudoValue::to_json()`
- [ ] Implement `RustPseudoList::to_json()`
- [ ] Add WASM binding `toJSON()`
- [ ] Implement `RustPseudoValue::from_json()`
- [ ] Implement `RustPseudoList::from_json()`
- [ ] Add WASM binding `fromJSON()`
- [ ] Add roundtrip tests (Rust)
- [ ] Add roundtrip tests (JS)
- [ ] Add roundtrip tests (Python)

### Validation & Cleanup
- [ ] Run full test suite: `npm test` (all 253+ tests should pass)
- [ ] Enable parallel testing: `ALIZARIN_PARALLEL_TEST=true npm test`
- [ ] Verify Rust and JS produce identical structures
- [ ] Check memory usage is acceptable
- [ ] Remove debug logging
- [ ] Update documentation
- [ ] Consider removing old recipe-based code (after validation)
- [ ] Benchmark performance (optional)

---

## Key Files Reference

| File | Purpose | Status |
|------|---------|--------|
| `src/pseudo_value.rs` | Core Rust types and hierarchy logic | Create |
| `src/instance_wrapper.rs` | Update return types | Modify |
| `src/python_bindings.rs` | PyO3 bindings for Python | Create |
| `src/lib.rs` | Module exports | Modify |
| `js/pseudos.ts` | Add `PseudoValueFromRust` wrapper | Modify |
| `js/semantic.ts` | Add `SemanticViewModelFromRust` | Modify |
| `js/graphManager.ts` | Update to use structured output | Modify |
| `python/alizarin/__init__.py` | Python wrapper classes | Create |
| `pyproject.toml` | Python package config | Create |
| `tests/rust-structured-output.test.ts` | Integration tests | Create |
| `python/tests/test_python_bindings.py` | Python tests | Create |

---

## Success Criteria

1. ✅ **Single Source of Truth** - All hierarchy construction logic in Rust
2. ✅ **Language Parity** - JS and Python get identical structures from Rust
3. ✅ **No Regressions** - All existing 253+ tests pass
4. ✅ **Clean Separation** - Structure (Rust) vs Types (ViewModels in each language)
5. ✅ **Incremental Loading** - Still works via lazy `ensureNodegroup` calls
6. ✅ **JSON I/O** - Can export/import resources as nested JSON
7. ✅ **Performance** - No significant slowdown vs current implementation

---

## Notes for Fresh Session

### Quick Context

We've already completed a successful migration of graph traversal logic to Rust using a "recipe pattern" where Rust returns flat lists of node+tile pointers, and JavaScript builds the hierarchy. This plan migrates to a better architecture where Rust returns the full structured hierarchy and languages just wrap it.

### Where We Are

- **Current**: Rust returns `Vec<PseudoRecipe>`, JS builds hierarchy
- **Target**: Rust returns `RustPseudoValue` tree, JS/Python wrap it
- **Reason**: Single source of truth for traversal, enable Python support, cleaner separation

### Start Here

1. **Phase 1**: Create `src/pseudo_value.rs` with core structs
2. **Test as you go**: Each phase has unit tests - write them!
3. **Keep old code**: Maintain `_JS` versions for parallel testing
4. **Don't break tests**: All 253 tests should keep passing

### Key Decisions Made

- Rust owns hierarchy construction but NOT ViewModels
- Tile data stays as `serde_json::Value` (unparsed in Rust)
- Use `Arc<>` for shared ownership, avoid lifetime complexity
- WASM for JS, PyO3 for Python
- Keep incremental loading (no pre-traversing everything)

### Build Commands

```bash
# Rust tests
cargo test --lib

# Build WASM
wasm-pack build --target web --out-dir pkg

# JS tests
npm test

# Python (after Phase 5)
maturin develop --features python
pytest python/tests/
```

### Useful Grep Patterns

```bash
# Find current recipe usage
rg "PseudoRecipe" --type rust
rg "recipe" --type typescript

# Find hierarchy construction
rg "makePseudoCls" js/
rg "build.*child" src/
```

---

## End of Migration Plan

This document provides a complete roadmap for migrating from recipe-based to structured hierarchy output. Follow the phases sequentially, test at each step, and maintain parallel testing until confident the new system works correctly.
