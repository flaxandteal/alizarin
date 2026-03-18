# Python Test Porting Status

## Summary

The Python Alizarin bindings are **feature-complete for core functionality** with **30/30 tests passing**. Further test porting is blocked by missing infrastructure that hasn't been implemented in Python yet.

## ✅ Completed (30/30 tests passing)

### Core Functionality Tests
- **test_conversion.py** (4 tests): tiles_to_json_tree, json_tree_to_tiles, roundtrip conversion
- **test_graph_manager.py** (9 tests): GraphManager core operations, node/edge retrieval, permissions
- **test_matching_entries.py** (5 tests): Tile filtering by parent, child tiles, root level
- **test_prune_graph.py** (2 tests): Graph pruning with permissions
- **test_semantic_children.py** (10 tests): Semantic child matching logic

### Core Features Implemented
- ✅ Rust-backed StaticGraph and StaticResource with JSON string API
- ✅ Python wrapper classes for dict ↔ JSON conversion
- ✅ ResourceInstanceWrapper with three-layer caching (Rust → Python → JSON)
- ✅ ResourceModelWrapper with graph pruning and permissions
- ✅ GraphManager and StaticStore
- ✅ Semantic child matching (matches TypeScript/Rust logic exactly)
- ✅ Bidirectional conversion: resource ↔ JSON
- ✅ Working examples (json_conversion.py)

## ⏸️ Blocked TypeScript Tests

The following TypeScript tests **cannot be ported** until additional infrastructure is implemented:

### 1. resources.test.ts
**Blocked by:**
- ArchesClient (local/remote) - not implemented in Python
- HTTP mocking infrastructure (fetchMock)
- Full resource loading pipeline
- Lazy loading support

**What it tests:**
- Loading resources from Arches API
- Lazy resource loading
- Accessing nested resource properties
- Multi-language support

**Priority:** High - core integration test, but requires ArchesClient implementation first

### 2. pseudos.test.ts
**Blocked by:**
- WASM initialization (Python doesn't use WASM directly)
- ViewModels mocking infrastructure
- Complex pseudo infrastructure not yet ported
- GraphMutator for creating test graphs

**What it tests:**
- PseudoUnavailable, PseudoValue, PseudoList classes
- Inner/outer structure for non-semantic nodes
- Iterable detection
- Field descriptions

**Priority:** Medium - tests Python pseudo classes, but requires significant infrastructure

### 3. static-types.test.ts
**Blocked by:**
- Different implementation (Python uses dataclasses, not classes with methods)
- StaticTranslatableString not implemented in Python
- Many TypeScript-specific features (Map vs Dict, copy() methods, etc.)

**What it tests:**
- Static type constructors and properties
- StaticTranslatableString multi-language support
- StaticTile data handling
- ensureId() generation

**Priority:** Low - Python uses simpler dataclass-based types, full compatibility not required

### 4. utils.test.ts
**Blocked by:**
- No utils.py module exists
- AttrPromise is JavaScript-specific (Proxy-based Promise)
- Language utilities not implemented

**What it tests:**
- slugify()
- getCurrentLanguage() / setCurrentLanguage()
- generateUuidv5()
- AttrPromise proxy

**Priority:** Low - can be implemented if needed, but not core to conversion functionality

### 5. client.test.ts
**Blocked by:**
- ArchesClient not implemented
- HTTP layer not implemented

### 6. WASM-specific tests (not applicable to Python)
- **wasmResourceModelWrapper.test.ts** - WASM bindings, Python doesn't use WASM directly
- **graph-wasm.test.ts** - WASM-specific functionality

### 7. Other specialized tests
- **buildResourceDescriptors.test.ts** - Descriptor building logic not yet ported
- **validation.test.ts** - Validation logic not yet ported
- **graphs.test.ts** - Graph manipulation not fully ported
- **graph-types.test.ts** - Type checking, likely not applicable
- **wkrm.test.ts** - WKRM class, partially implemented
- **ensureNodegroup-edge-cases.test.ts** - Edge cases for nodegroup logic

## 🎯 What Works

The Python implementation is **production-ready** for:

1. **Converting tiled Arches resources to nested JSON**
   ```python
   json_dict = alizarin.resource_to_json(resource, graph)
   ```

2. **Converting nested JSON to tiled resources**
   ```python
   resource = alizarin.json_to_resource(json_dict, graph)
   ```

3. **Working with graphs**
   ```python
   graph = alizarin.StaticGraph(json.dumps(graph_data))
   resource = alizarin.StaticResource(json.dumps(resource_data))
   ```

4. **Resource instance wrappers** (partially - core functionality works)
   ```python
   wrapper = ResourceInstanceWrapper(wkri, wasm_wrapper, model)
   await wrapper.populate(tiles)
   value_cache = await wrapper.get_value_cache()
   ```

## 📋 Next Steps

To enable porting blocked tests, the following would need to be implemented:

### High Priority
1. **ArchesClient** - HTTP client for communicating with Arches API
   - Local client (for testing with local files)
   - Remote client (for real API calls)
   - This unlocks: resources.test.ts, client.test.ts

2. **ViewModels** - Complete Python implementations
   - SemanticViewModel, StringViewModel, etc.
   - Currently stubbed but not fully functional
   - This unlocks: pseudos.test.ts

### Medium Priority
3. **Complete Pseudo Infrastructure**
   - Full PseudoValue, PseudoList implementations
   - Inner/outer structure handling
   - This unlocks: pseudos.test.ts

4. **Utility Functions** (if needed)
   - slugify, UUID generation, language handling
   - This unlocks: utils.test.ts

### Low Priority
5. **StaticTranslatableString**
   - Multi-language string class
   - Part of static-types.test.ts

6. **Descriptor Building**
   - buildResourceDescriptors logic
   - This unlocks: buildResourceDescriptors.test.ts

## 📊 Test Coverage Summary

| Test Suite | Status | Tests Passing | Notes |
|------------|--------|---------------|-------|
| Core conversion | ✅ Complete | 4/4 | Fully functional |
| GraphManager | ✅ Complete | 9/9 | Core operations work |
| Matching entries | ✅ Complete | 5/5 | Tile filtering works |
| Graph pruning | ✅ Complete | 2/2 | Permission-based pruning |
| Semantic children | ✅ Complete | 10/10 | Matches Rust/TS exactly |
| **Total** | **✅ Complete** | **30/30** | **100% of ported tests pass** |
| | | | |
| Resources | ⏸️ Blocked | N/A | Needs ArchesClient |
| Pseudos | ⏸️ Blocked | N/A | Needs ViewModels + infrastructure |
| Static types | ⏸️ Blocked | N/A | Different implementation approach |
| Utils | ⏸️ Blocked | N/A | Module doesn't exist |
| Other (9 files) | ⏸️ Blocked/N/A | N/A | Various blockers or WASM-specific |

## ✨ Achievements

The Python implementation successfully:

1. ✅ Provides a clean Python API matching the TypeScript interface
2. ✅ Uses Rust for performance-critical operations
3. ✅ Handles dict ↔ JSON conversion transparently in Python layer
4. ✅ Implements three-layer caching architecture
5. ✅ Passes all ported tests with exact semantics matching TypeScript/Rust
6. ✅ Works with real Arches data (Group.json test data)
7. ✅ Provides working examples for common use cases

The core functionality is **solid and tested**. Additional infrastructure can be added incrementally based on actual usage requirements.
