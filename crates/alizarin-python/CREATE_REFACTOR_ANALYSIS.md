# Analysis: Refactoring `_create` Methods

## Revised Architecture (NodeConfig First)

Moving nodeConfig management to Rust first enables more centralized logic:

```
                    Python/TypeScript                          Rust
                    ─────────────────                          ────
                                                         NodeConfigManager
                                                         (graph domain values,
                                                          boolean labels)
                                                               │
                                                               ▼
Input Value ──┬──▶ (a) Recognize ViewModel ──▶ Copy/Reference
              │
              ├──▶ (b) Recognize Tile Data ──────────────────▶ json_to_tile_data()
              │       (dict/list passthrough)                      │
              │                                                    ▼
              └──▶ (c) Coerce to JSON ───────────────────────▶ json_to_tile_data()
                     - resolve promises                            │
                     - convert rich types                          │
                                                                   ▼
                                                         ┌─────────────────┐
                                                         │ Domain Lookup   │
                                                         │ (UUID → Value)  │
                                                         └─────────────────┘
                                                                   │
                                                                   ▼
                                                         Validated tile data +
                                                         Resolved domain values
```

## Phase 0: NodeConfig in Rust (PREREQUISITE)

### What NodeConfig Provides

Currently in Python/TypeScript:
- `StaticNodeConfigDomain` - List of domain value options from graph
- `StaticNodeConfigBoolean` - True/false labels (i18n)
- `StaticNodeConfigConcept` - RDM collection ID reference

### Rust Implementation

```rust
/// Domain value configuration (from graph)
pub struct StaticNodeConfigDomain {
    pub options: Vec<StaticDomainValue>,
    pub i18n_config: HashMap<String, String>,
}

impl StaticNodeConfigDomain {
    /// Lookup domain value by ID
    pub fn value_from_id(&self, id: &str) -> Option<&StaticDomainValue> {
        self.options.iter().find(|opt| opt.id == id)
    }

    /// Get selected value (if any)
    pub fn get_selected(&self) -> Option<&StaticDomainValue> {
        self.options.iter().find(|opt| opt.selected)
    }
}

/// Boolean configuration
pub struct StaticNodeConfigBoolean {
    pub true_label: HashMap<String, String>,
    pub false_label: HashMap<String, String>,
}

/// Concept configuration (just references RDM)
pub struct StaticNodeConfigConcept {
    pub collection_id: String,
}

/// Factory: build node config from graph
pub fn build_node_config(node: &StaticNode, graph: &StaticGraph) -> Option<NodeConfig> {
    match node.datatype.as_str() {
        "domain-value" | "domain-value-list" => {
            // Extract options from node.config or graph
            Some(NodeConfig::Domain(StaticNodeConfigDomain { ... }))
        }
        "boolean" => {
            Some(NodeConfig::Boolean(StaticNodeConfigBoolean { ... }))
        }
        "concept" | "concept-list" => {
            Some(NodeConfig::Concept(StaticNodeConfigConcept { ... }))
        }
        _ => None
    }
}
```

### What This Unlocks

With nodeConfig in Rust:

| Type | Before | After |
|------|--------|-------|
| **DomainValue** | Python does UUID → StaticDomainValue lookup | Rust does lookup, returns full object |
| **DomainValueList** | Python iterates, looks up each | Rust handles full list |
| **Boolean** | Python looks up labels | Rust provides labels |

ConceptValue **can also move to Rust** with a "fetch-then-lookup" pattern (see below).

---

## RDM Collection Handling (Fetch-Then-Lookup Pattern)

The same pattern used for tile data can apply to RDM collections:

```
Python/TypeScript                              Rust
────────────────                               ────
                                          RdmCollectionCache
                                          (HashMap<collection_id, StaticCollection>)
                                               │
is_collection_loaded(id) ◄────────────────────┤
         │                                     │
         ▼                                     │
[false] → await RDM.retrieveCollection(id)     │
         │         (async HTTP)                │
         ▼                                     │
load_collection(id, data) ────────────────────►│
                                               │
[true] ────────────────────────────────────────┤
                                               │
UUID string ──────────────────────────────────► coerce_concept_value(uuid, collection_id)
                                               │
                                               ▼
                                          StaticValue (resolved)
```

### Rust API for RDM

```rust
/// RDM collection cache (populated on demand from Python/TS)
pub struct RdmCache {
    collections: HashMap<String, StaticCollection>,
}

impl RdmCache {
    /// Check if collection is loaded
    pub fn is_loaded(&self, collection_id: &str) -> bool {
        self.collections.contains_key(collection_id)
    }

    /// Load collection data (called after Python/TS fetches from RDM)
    pub fn load_collection(&mut self, collection_id: String, data: StaticCollection) {
        self.collections.insert(collection_id, data);
    }

    /// Lookup concept value by UUID
    pub fn get_concept_value(
        &self,
        collection_id: &str,
        value_id: &str,
    ) -> Option<&StaticValue> {
        self.collections
            .get(collection_id)?
            .get_concept_value(value_id)
    }
}
```

### Python/TS Usage Pattern

```python
# In ConceptValueViewModel._create or coercion layer:
async def ensure_collection_loaded(collection_id: str) -> None:
    if not rust_rdm_cache.is_loaded(collection_id):
        # Async fetch stays in Python/TS
        collection = await RDM.retrieveCollection(collection_id)
        # Pass to Rust for storage
        rust_rdm_cache.load_collection(collection_id, collection.to_json())

# Then Rust can do the lookup
value = rust_coerce_concept_value(uuid, collection_id)
```

### What This Achieves

| Type | Async Fetch | Sync Lookup | Constructor |
|------|-------------|-------------|-------------|
| DomainValue | N/A (graph data) | ✅ Rust | Takes resolved value |
| ConceptValue | Python/TS → RDM | ✅ Rust | Takes resolved value |
| Boolean | N/A (graph data) | ✅ Rust | Takes bool + labels |

**All lookup logic centralizes in Rust**, only async I/O stays in Python/TS.

---

## ViewModel Constructor Consistency Analysis

### Current Constructor Patterns

| ViewModel | Constructor Takes | Is This Tile Data? |
|-----------|------------------|-------------------|
| `DateViewModel(str)` | ISO date string | ✅ Yes |
| `EDTFViewModel(str)` | EDTF string | ✅ Yes |
| `NonLocalizedStringViewModel(str)` | Plain string | ✅ Yes |
| `NumberViewModel(Union[int, float])` | Number | ✅ Yes |
| `StringViewModel(Dict[str, str])` | i18n dict | ✅ Yes |
| `GeoJSONViewModel(Dict)` | GeoJSON dict | ✅ Yes |
| `UrlViewModel(Url)` | Url dataclass | ⚠️ Intermediate (not raw dict) |
| `BooleanViewModel(bool, config)` | bool + config | ⚠️ Requires extra config |
| `DomainValueViewModel(StaticDomainValue)` | Resolved object | ❌ Tile has UUID |
| `ConceptValueViewModel(StaticValue)` | Resolved object | ❌ Tile has UUID |
| `ResourceInstanceViewModel(id, ...)` | ID + extras | ⚠️ ID is tile data, extras are not |

### Obstacles to Consistent "Tile Value" Constructors

#### 1. **DomainValueViewModel** - SOLVABLE with Rust nodeConfig

**Current:**
```python
# Constructor
def __new__(cls, value: StaticDomainValue):  # Takes resolved object

# _create does lookup
if isinstance(value, str) and is_uuid(value):
    config = nodeConfigManager.retrieve(node)
    val = config.value_from_id(value)  # Lookup here
    return DomainValueViewModel(val)
```

**With Rust nodeConfig:**
```python
# Option A: Constructor takes UUID, does lookup internally
def __new__(cls, uuid: str, node_config: StaticNodeConfigDomain):
    value = node_config.value_from_id(uuid)
    return str.__new__(cls, str(value))

# Option B: Rust returns resolved value, constructor stays same
# Rust: coerce_domain_value(uuid, node_config) -> StaticDomainValue
```

**Recommendation:** Option B - Rust does the lookup, Python constructor stays clean.

#### 2. **ConceptValueViewModel** - SOLVABLE with fetch-then-lookup pattern

**Problem:** Tile stores UUID, but need `StaticValue` from RDM (external async service).

**Solution:** Separate async fetch from sync lookup:

```python
# Step 1: Ensure collection loaded (async, stays in Python/TS)
async def ensure_collection_loaded(collection_id: str):
    if not rust_rdm_cache.is_loaded(collection_id):
        collection = await RDM.retrieveCollection(collection_id)
        rust_rdm_cache.load_collection(collection_id, collection)

# Step 2: Lookup (sync, in Rust)
val = rust_rdm_cache.get_concept_value(collection_id, uuid)
return ConceptValueViewModel(val)
```

**What moves where:**
- **Python/TS:** `ensure_collection_loaded()` - async fetch + cache population
- **Rust:** `RdmCache.get_concept_value()` - sync lookup from cached collection

**Recommendation:** Constructor keeps taking `StaticValue`. Rust does the lookup after Python/TS ensures data is available.

#### 3. **BooleanViewModel** - SOLVABLE with Rust nodeConfig

**Current:**
```python
def __init__(self, value: bool, config: StaticNodeConfigBoolean):
    self._value = bool(value)
    self.__config = config  # Needed for toString()
```

**Options:**
- **A:** Constructor takes just `bool`, lookup config lazily in `toString()`
- **B:** Rust provides config with coerced value
- **C:** Keep current (it's not that bad)

**Recommendation:** Option B - When Rust has nodeConfig, it can return `(bool, labels)` together.

#### 4. **ResourceInstanceViewModel** - ACCEPTABLE as-is

**Current:**
```python
def __init__(self, id: str, modelWrapper, instanceWrapperFactory, cacheEntry):
```

The `id` IS the tile data. The extras support lazy loading of cross-resource relationships.

**Recommendation:** Keep as-is. The constructor effectively takes tile data (UUID), extras are optional.

#### 5. **UrlViewModel** - MINOR inconsistency

**Current:** Takes `Url` dataclass, not raw dict.

```python
def __new__(cls, value: Url):  # Intermediate type

# _create converts dict to Url
url = Url(url=val['url'], url_label=val.get('url_label'))
return UrlViewModel(url)
```

**Recommendation:** Could take raw dict, but `Url` dataclass adds type safety. Keep as-is.

---

## Revised Per-ViewModel Analysis

### 1. Simple Scalar Types (EASY - No Changes)

These already take tile data directly:
- DateViewModel, EDTFViewModel, NonLocalizedStringViewModel, NumberViewModel

### 2. Dict-Based Types (EASY - No Changes)

These already take tile-format data:
- StringViewModel, GeoJSONViewModel

UrlViewModel takes intermediate `Url` type - acceptable.

### 3. Config-Dependent Types (NOW EASY with Rust nodeConfig)

#### BooleanViewModel
| Step | Current | With Rust nodeConfig |
|------|---------|---------------------|
| Tile data | bool | ✅ Same |
| Config lookup | Python | ✅ Rust |
| Constructor | `(bool, config)` | Could be `(bool)` if Rust provides labels |

#### DomainValueViewModel
| Step | Current | With Rust nodeConfig |
|------|---------|---------------------|
| Tile data | UUID string | ✅ Same |
| Config lookup | Python | ✅ Rust |
| Constructor | `(StaticDomainValue)` | ✅ Same (Rust provides resolved value) |

### 4. Reference Types (IMPROVED)

#### ConceptValueViewModel - Fetch-then-lookup pattern
| Step | Current | With Rust RdmCache |
|------|---------|---------------------|
| Tile data | UUID string | Same |
| Collection fetch | Python (async) | ✅ Python ensures loaded |
| Value lookup | Python | ✅ Rust (sync from cache) |
| Constructor | `(StaticValue)` | Same |

#### ResourceInstanceViewModel - No change needed
Already takes UUID (tile data) + optional extras for lazy loading.

### 5. List Types (IMPROVED with Rust caches)

#### DomainValueListViewModel
With Rust nodeConfig, Rust can resolve all UUIDs → StaticDomainValues in a single call.

#### ConceptListViewModel
With fetch-then-lookup: Python/TS ensures collection loaded, then Rust does batch lookup.

---

## Revised Rust Function Signatures

```rust
/// NodeConfig management
pub struct NodeConfigManager {
    configs: HashMap<String, NodeConfig>,  // Keyed by nodeid
}

impl NodeConfigManager {
    /// Build config from graph (call once when loading graph)
    pub fn from_graph(graph: &StaticGraph) -> Self;

    /// Get config for a node
    pub fn get(&self, nodeid: &str) -> Option<&NodeConfig>;
}

/// Coerce value to tile data with config lookup
pub fn coerce_to_tile_data(
    value: Value,
    datatype: &str,
    node_config: Option<&NodeConfig>,
) -> Result<CoercedValue, CoercionError> {
    match datatype {
        // Simple types - no config needed
        "string" => Ok(coerce_string(value)?),
        "number" => Ok(coerce_number(value)?),
        "date" => Ok(coerce_date(value)?),
        // ...

        // Config-dependent types
        "domain-value" => {
            let config = node_config.ok_or(CoercionError::MissingConfig)?;
            coerce_domain_value(value, config.as_domain()?)
        }
        "boolean" => {
            let config = node_config.ok_or(CoercionError::MissingConfig)?;
            coerce_boolean(value, config.as_boolean()?)
        }

        // External lookup types - return tile data only
        "concept" => Ok(coerce_concept_value_tile_only(value)?),

        _ => Err(CoercionError::UnknownDatatype),
    }
}

/// Result of coercion - may include resolved reference
pub enum CoercedValue {
    /// Simple tile data (string, number, dict, etc.)
    TileData(Value),

    /// Domain value with resolved details
    DomainValue {
        tile_data: String,  // UUID for tile
        resolved: StaticDomainValue,  // For ViewModel constructor
    },

    /// Boolean with labels
    Boolean {
        tile_data: bool,
        true_label: HashMap<String, String>,
        false_label: HashMap<String, String>,
    },
}
```

---

## Summary: What Changes with Rust NodeConfig

### Now Moves to Rust:
1. **Domain value lookup** - UUID → StaticDomainValue
2. **Boolean labels** - true/false label retrieval
3. **Domain value list handling** - Batch UUID resolution
4. **Concept value lookup** - UUID → StaticValue (after collection loaded)
5. **Concept list handling** - Batch UUID resolution from cached collections

### Still Stays in Python/TypeScript:
1. **RDM collection fetching** - Async HTTP to external service (but lookup moves to Rust)
2. **ViewModel instance recognition** - `isinstance()` checks
3. **Promise/async resolution** - `await` handling
4. **ResourceInstance lazy loading** - Cross-resource relationships

---

## Revised Difficulty Summary

| ViewModel | Without Rust Caches | With Rust Caches |
|-----------|---------------------|------------------|
| DateViewModel | ⭐ Easy | ⭐ Easy |
| EDTFViewModel | ⭐ Easy | ⭐ Easy |
| NonLocalizedStringViewModel | ⭐ Easy | ⭐ Easy |
| NumberViewModel | ⭐ Easy | ⭐ Easy |
| StringViewModel | ⭐ Easy | ⭐ Easy |
| UrlViewModel | ⭐ Easy | ⭐ Easy |
| GeoJSONViewModel | ⭐ Easy | ⭐ Easy |
| BooleanViewModel | ⭐⭐ Medium | ⭐ Easy |
| ResourceInstanceViewModel | ⭐⭐ Medium | ⭐⭐ Medium |
| **DomainValueViewModel** | ⭐⭐⭐ Hard | ⭐ Easy |
| **DomainValueListViewModel** | ⭐⭐⭐ Hard | ⭐⭐ Medium |
| **ConceptValueViewModel** | ⭐⭐⭐ Hard | ⭐⭐ Medium (fetch-then-lookup) |
| **ConceptListViewModel** | ⭐⭐⭐ Hard | ⭐⭐ Medium (fetch-then-lookup) |

---

## Revised Implementation Order

### Phase 0: Rust Caches (PREREQUISITE)
1. **NodeConfigManager** - Domain, Boolean configs from graph
   - Define `NodeConfig` enum (Domain, Boolean, Concept)
   - Implement `NodeConfigManager::from_graph()`
   - Expose to Python/TS via FFI

2. **RdmCache** - Concept collections (populated on demand)
   - Define `RdmCache` with collection storage
   - Implement `is_loaded()`, `load_collection()`, `get_concept_value()`
   - Expose to Python/TS via FFI

### Phase 1: Simple scalars
- Date, EDTF, NonLocalizedString, Number

### Phase 2: Dict types
- String, Url, GeoJSON

### Phase 3: Config-dependent (NOW EASY)
- Boolean (uses Rust nodeConfig)
- DomainValue (uses Rust nodeConfig for lookup)
- DomainValueList (batch lookup in Rust)

### Phase 4: RDM-dependent (NOW MEDIUM)
- ConceptValue (Python ensures collection loaded, Rust does lookup)
- ConceptList (batch lookup from cached collections)

### Phase 5: Format normalization
- ResourceInstance, ResourceInstanceList

---

## Appendix: Constructor Consistency Summary

**Can take tile data directly (or already do):**
- ✅ DateViewModel, EDTFViewModel, NonLocalizedStringViewModel, NumberViewModel
- ✅ StringViewModel, GeoJSONViewModel
- ✅ ResourceInstanceViewModel (ID is tile data)

**Need resolved object (lookup in Rust):**
- ✅ DomainValueViewModel → Rust nodeConfig lookup
- ✅ ConceptValueViewModel → Rust RdmCache lookup (after Python ensures collection loaded)

**Take intermediate types (acceptable):**
- ⚠️ UrlViewModel (Url dataclass vs raw dict)
- ⚠️ BooleanViewModel (needs config for labels)

**Conclusion:** With Rust caches (NodeConfigManager + RdmCache), **all ViewModels** can have constructors that effectively take tile data, with Rust doing any necessary resolution. Python/TS only handles async I/O (RDM HTTP fetch) and ViewModel instance recognition.
