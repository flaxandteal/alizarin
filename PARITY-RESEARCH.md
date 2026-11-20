# Backend and extension parity research

A programme for finding inconsistent or redundant behaviour across the WASM,
NAPI and Python backends and the extensions, and a register of what it finds.

**Rule for this phase: record, don't fix.** Findings go in the register below.
Fixes are made after consolidation, grouped by class, each with all three
backends and a guard that stops the class recurring (see "Backend parity is
mandatory" in `CLAUDE.md`).

## Problem classes

| # | Class | Seen so far |
|---|---|---|
| 1 | Hand-mirrored logic diverging from core | Python `StaticResourceMetadata` dropped lifecycle fields; Python descriptors crashed on Arches' per-language form |
| 2 | Stubs that pass for implementations | Python `ResourceModelWrapper.all()` returned `[]` |
| 3 | Dead calls in untested dynamic branches | `graphManager.get_root_view_model` (find-and-replace overshoot in `2d3b6ef`) |
| 4 | Tests that cannot fail | swallowed `except Exception`, asserts inside `if len(...)`, impossible fixture tiles |
| 5 | Runtime/environment dependence | `datetime.replace()` on a subclass (Python 3.13); runner images losing Python 3.9 |
| 6 | Test-harness divergence between backends | NAPI vitest config lacked the extension WASM plugin; Python CI lacked `alizarin_geo` |
| 7 | Docs/comments drifting from code | matrix said WASM had no CSV; ABI page said v2 (is 5); docs used non-exported Python functions; `js/backend.ts` NAPI-handler comment |
| 8 | Extension wiring differing per binding | each extension module has its own copy of core's registries |
| 9 | Redundant implementations | TS and Python ORMs separate; descriptor rule in core and Python; 21 struct literals patched by hand |
| 10 | Packaging/version consistency | b9/beta.9 sorts above the alphas on PyPI and crates.io |

## Workstreams

Each ends with a guard (lint, test or generated artefact), not just fixes.

- **A. ORM-level surface inventory** (2, 3, 9). List public methods of `GraphManager`, `ResourceModelWrapper`, the instance wrapper and view models per backend; extend `tests/binding-parity.json` (it covers only Rust binding classes today); flag stubs (`...`, `pass`, `return []`).
- **B. Hand-mirrored logic** (1, 9). Find Python/TS code that parses, serialises or defaults what core defines; round-trip core goldens and raw Arches exports through it; Hypothesis for shapes.
- **C. Static checking** (3). pyright/mypy over `python/alizarin` with `viewContext.graphManager` typed as a Protocol; `tsc --strict` over `js/`. Would it have caught `2d3b6ef`?
- **D. Test integrity** (4). Lint tests for unre-raised `except`, conditional asserts, print-for-assert, fixture-dependent skips, guarded fixtures; TS equivalents; then mutation testing (mutmut, Stryker) on ORM modules.
- **E. ORM behavioural parity** (2, 3, 9). Scenarios-as-data (load, cross-model traversal, `all()`, edit/commit, `for_json(cascade)`, lifecycle) run by a runner per backend against one expected file.
- **F. Runtime matrix** (5). Scheduled CI on `ubuntu-latest` + newest Python/Node; audit stdlib subclasses (`DateViewModel(datetime)` etc.).
- **G. Extensions × bindings** (6, 8). What each extension registers per binding (handlers, folds, derive providers, ABI), and which CI job tests it against the main package.
- **H. Docs honesty** (7). Execute the docs' Python snippets; generate the support matrix from `binding-parity.json`; grep comments asserting facts about other backends.
- **I. Redundancy register** (9). Catalogue duplicates; decide per item: move to core, generate, or keep with a parity test.
- **J. Release consistency** (10). Post-release install smoke test from npm/PyPI/crates.io; decide the version-ordering question.

Order: D and C first (until then other tests may be hollow), then A and B, then E. F–J can run alongside.

## Resolution plan

Decided 2026-11-19:
- **Canonical names**, spelt idiomatically per language: camelCase in TS, snake_case in Python and Rust. Core's name where core has one.
- **No aliases.** Renames and behaviour changes go in `CHANGELOG.md`.
- **No yanking**; packaging and release fixes (J) are out of scope for now.
- **Python view models keep subclassing builtins**; fix copy, pickle and arithmetic in place (F-5, F-6).
- Open: whether Python gets a data layer (A-3).

1. **Manifest** (A-5, A-6, A-7, H-3, G-5). `tests/binding-parity.json` grows into the API manifest: every class, method and free function (ORM classes included), canonical name, per-backend status (present / `ecosystem:` / `gap:`). Tested in both directions; the support matrix and binding docs are generated from it.
2. **Core owns behaviour** (B-4, B-5, B-7..B-11, F-7). One core function per duplicated piece of logic, bindings as one-line wrappers; Python dataclasses built from `canonical_static`; NAPI `createStatic*` through core; value display decided in core.
3. **Two ORMs, pinned** (A-1..A-4, C-1, E). Shared scenarios-as-data run by every backend; Python ORM gaps filled as wrappers over core.
4. **Guards first, ratcheted.** Each guard lands with today's findings as an explicit allow-list (`tests/guard-allowlist.json`), so CI is green from the start; a fix deletes its entry and no entry may be added back. Guards: test-integrity lint (D), no unexpected skips (D-4), type-check baselines (C-5), Python ↔ core round trip (B-5), manifest (1), scenarios (3).
5. **Releases.** Additive changes and corrections of wrong output ship as they're ready, each with a `CHANGELOG.md` entry. Removals and renames are batched into one parity release with a migration table.

| Step | Findings |
|---|---|
| Guards | D-1..D-10, C-2, C-5, B-5/B-6 (as tests), A-5 |
| Additive/corrections | A-1, A-4, A-6/A-7 gaps, B-1..B-3, B-7..B-11, C-1, C-3, C-4, D fixes, F-5..F-7, G-1..G-4, H-1, H-2 |
| Parity release (breaking) | A-6/A-7 renames, B-4/B-5 switch to core parsing, F-7 number display, A-2/A-3 decisions |

## Register

Fields: id, class, workstream, location, finding, evidence (command/output or file:line), proposed fix, guard.

<!-- Entries are appended below, newest last. -->

### Already fixed (this session, before the programme)

- **F-1** (1) `449e246` Python descriptors parse Arches' per-language form as core's loader does.
- **F-2** (2) `b10f74c` Python `ResourceModelWrapper.all()` returns the model's registered resources.
- **F-3** (3, 4) `ea36244` Python `retrieve()` calls `getResource`; `test_cross_model_traversal` made able to fail (removed `except Exception: print`, `if len(members)` guard, impossible 3-tile fixture).
- **F-4** (5) `b336d3c` `DateViewModel.for_json` on 3.12+ (subclass `replace()`).

### D. Test integrity

Method: AST scan of `test_*.py` (core + `ext/*/python`) for unre-raised `except`, `assert` under `if`, `pytest.skip` in test bodies, tests with no assertion; each hit read by hand. Scanner false positives: fixtures named `test_*` (`test_resources.py:103 test_model`, `ext/clm/.../test_reference.py:1549 test_collection`); parametrised-key conditionals (`test_view_models_shared.py:61,63`); `test_reference.py:2065` (asserts on both branches).

- **D-1** (4) `ext/clm/python/tests/test_clm_collection.py:160-175` `test_collection_as_resource_references`: the activities check runs only `if activities_node_id in tile_data`, else prints "Note: ... may be filtered if empty" and passes. **It takes the print branch today** (`pytest -s` shows the Note; Part 2, concepts, does resolve). So nothing about CLM references-as-resource-instances is asserted. Unknown whether fixture or real bug. Fix: assert found; investigate why references don't reach the tiles. Guard: D lint (conditional assert without an `else: fail`). **Resolved: neither; the test's premise was false.** A resource-instance-list points at resource instances and has nothing to do with RDM/CLM collections; nothing resolves collection labels for it (strict conversion rejects them as invalid UUIDs, lenient drops them). The file never imported `alizarin_clm`, so it tested no CLM. Rewritten: the resource-instance-list test is gone; a controlled list's labels are now tested resolving as CLM `reference` values (directly, and after a SKOS XML round trip) and, separately, as concept ids on a `concept-list` node. D-2's three tests now assert the node's tile exists.
- **D-2** (4) Same pattern, currently taking the asserting branch: `test_clm_collection.py:284-299` (concepts), `crates/alizarin-python/tests/test_rdm_collection.py:305-313` and `:1052-1059` (`if node_id in tile_data`, no assertion that it was found). Fix: assert presence first.
- **D-3** (4) `crates/alizarin-python/tests/test_view_models.py:415-419` `test_all_viewmodels_support_for_json`: asserts only `if vm is not None`; a `_create` returning `None` for every class passes. Fix: assert non-None (or list the classes expected to return None).
- **D-4** (4) ~35 `pytest.skip` calls in test bodies conditional on fixtures/handlers (`test_rdm_collection.py` ×10; `ext/clm/python/tests/test_reference.py` ×18, `test_reference_change_collection.py` ×7, `test_clm_collection.py` ×3). None fire today (local: core 325 passed/1 skipped, clm 81 passed; CI run 44581208986: same, only skip is the documented `staticTile`). Risk: a missing fixture or unregistered handler becomes a silent skip. Guard: conftest hook failing on any skip not on an allow-list (start with the one `staticTile` skip).
- **D-5** (6, 8) `ext/geo/python` and `ext/pg/python` have no Python tests of their own; CI's extension loop builds them and runs nothing (`=== Building ext/geo/python/ ===` with no totals). geo is covered only via `crates/alizarin-python/tests/test_utm.py`; pg Python not at all. Fix/guard: per-extension smoke test per binding (workstream G).
- **D-6** (6) Locally `alizarin_clm` imports from the source tree (`ext/clm/python/python/alizarin_clm/__init__.py`), so local runs may use a stale native module; CI builds wheels. Note for G: tests should say which build they ran against.
- **D-7** (4, parity rule) `tests/dual-backend/backendConsistency.test.ts:116-158`: the `.tileId`, `.nodeAlias`, `.nodegroupId` getter tests assert only `if (x !== null && x !== undefined)`. They accept `undefined`, which the parity rule forbids, and pass if the getter is missing altogether. Each backend is checked alone, not against the other. Fix: `expect(x === null || typeof x === 'string')` plus a cross-backend equality. Guard: D lint (conditional `expect`).
- **D-8** (4) `tests/client.test.ts:15` `describe.skip('ArchesClientRemote')`, reason "require a live remote Arches server", yet `beforeEach` mocks `global.fetch` (`:12`), so the reason looks wrong (skipped since `2a5ff11`, 2025-10-14). `ArchesClientRemote` therefore has no tests on any backend. Fix: un-skip against the mocked fetch.
- **D-9** (4) `tests/wasmResourceModelWrapper.test.ts:264` `test.skip(... getRootNode should throw if no root found)`, because the test mutates a returned Map rather than the graph. The behaviour (no root → throw) is untested; build a rootless graph instead.
### A. ORM-level surface

Method: TS class methods (regex over `js/graphManager.ts`, `js/pseudos.ts`, `js/semantic.ts`, `js/viewModels/*.ts`) vs Python (AST over `python/alizarin/*.py`, Protocols excluded), names normalised to snake_case; then searched Python and the native classes (`dir()` of `RustResourceInstanceWrapperCore`, `PyResourceModelWrapper`) for differently-named equivalents. NAPI shares the TS ORM, so this is TS vs Python. Not counted as gaps: `$commit`/`$discard` vs `commit`/`discard` (documented idiom), TS fields read as methods (`graph`, `nodes`, `edges`, `nodegroups`, `nodesByAlias`, `tile`, `node`), Python lists not awaitable item-wise (see F-3).

Stub scan: no stub-shaped bodies left in Python ORM classes. The `__forJsonCache` → `None` hits match TS's `null`.

- **A-1** (2, parity) Missing from the Python ORM but present in the native core, so a thin wrapper would do: on `ResourceInstanceWrapper`, TS `getDescriptors`, `getName`, `getValuesAtPath`, `pruneResourceTiles`, `release` ↔ core `get_descriptors`, `get_name`, `get_values_at_path`, `prune_resource_tiles`, `release`. On `ResourceInstanceViewModel`, TS `getDescriptors`, `getName`, `getSlug`, `getDescription`, `getMapPopup`, `forDisplayJson` ↔ core `get_descriptors`, `to_display_json`. Python users would have to reach into `rivm._i._rust_core`.
- **A-2** (2, parity) Missing in Python at both layers: `ResourceModelWrapper.makeInstance` (create a new resource through the ORM; Python can only `register_resource` a dict), `find`/`findStatic`, `allSummaries`/`loadFullResource` (summary-first loading), `asTree`, `fromStaticResource`, `getNodeIdFromAlias`, `getNodeObjectFromId`, `getEdgeObjects`. `ResourceInstanceWrapper.keys`/`values`/`entries`, `getViewModelsAtPath`, `ensureTilesLoaded`, `get/setOrmAttribute`. `PseudoValue.updateValue`, `getParentTileId`. `PseudoList.describeField`/`describeFieldGroup`, `push`/`splice`. Each needs a decision: wrap, or record `ecosystem:`/`gap:`.
- **A-3** (2, 7; parity, docs) The Python ORM has no client or static-store layer: `GraphManager` is `register_graph`/`register_resource` over an in-memory dict (its own docstrings say "test fixture storage"), with no counterpart to TS `initialize`/`loadGraph`/`get(modelName)`/`ArchesClient*`/`StaticStore`. The support matrix's "Registry calls behind StaticStore ✓ Python" refers to the native registry, not the ORM, and `docs/bindings/python.mdx` now presents a Python ORM. The docs should say how resources get into it, or the layer should exist.
- **A-4** (2, both backends) `PseudoValue.getChildren(direct)` is half-built on both sides alike. TS `js/pseudos.ts:369-380` has an empty "tricky with AttrPromise" block and only delegates to `inner`; Python `pseudos.py:333-356` documents `direct` as "not implemented yet" and does the same. Consistent, but a stub in both. Decide: implement in core, or remove the parameter.
- **A-6** (2, 9; parity) Free functions per binding. Method: `export function` in `pkg/alizarin.d.ts` (WASM, 87), `export declare function` in `crates/alizarin-napi/index.d.ts` (NAPI, 31), callables in `alizarin.alizarin` (Python, 99), compared as snake_case. **Only 15 exist on all three.** Two problems mixed together:
  - *Same function, different names* (decide one name per function, keep aliases for a release): `coerce_geo_json`(W) / `coerce_geojson`(P); `add_collection_to_global_rdm_cache`(W) / `add_collection_to_global_cache`(P); `add_to_global_rdm_cache_from_skos_xml`(W) / `add_from_skos_xml_to_global_cache`(P); `collection(s)_to_skos_xml`(W,N) / `skos_collection(s)_to_xml`(P); `tiles_to_tree`/`tree_to_tiles`/`cards_to_tree`(W) / `tiles_to_json_tree`/`json_tree_to_tiles`/`cards_to_json_tree`(P); `export_graphs`/`export_all_graphs`/`build_prebuild_export`(W) / `export_graphs_to_dir`/`export_all_graphs_to_dir`/`export_prebuild`(P); `resolve_labels_with_lookup`(W) / `resolve_labels`(P); `register_extension_handler`/`has_extension_handler`(W,N) / `register_type_handler`/`has_type_handler`(P); `extension_coerce`/`extension_render_display`(N) / `coerce_with_extension`/`render_display_with_extension`(P); `build_graph_from_csvs`/`build_business_data_from_csv`(N) / `build_graph_from_model_csvs`/`build_resources_from_business_csv`(W,P); `get_registered_extension_mutations`(W) / `list_extension_mutations`(P); `find_needed_collections`(W,N) / `get_needed_collections`(P).
  - *Missing* (each to fill or record as `ecosystem:`/`gap:`). Not in the matrix today: NAPI lacks `merge_resources`, `batch_merge_resources`, `generate_uuid_v5`, the RDM namespace set (H-3), `register_extension_mutation`/`has_extension_mutation`/`apply_mutations_with_extensions`, `get_registered_graph_ids`, `validate_model_csvs`, `global_rdm_has_collection`/`get_parent_id`, `set/clear/has_global_rdm_cache`. WASM lacks `apply_aggregate_functions_from_file`, `import_prebuild`. Python lacks `unregister_extension_handler`/`unregister_extension_mutation`, `get_default_config_keys`, `get_default_resolvable_datatypes`, `build_alias_to_collection_map`, `remove_from_global_rdm_cache`, `global_rdm_get_collection_ids`, `summary_to_metadata`. Already matrix-documented as NAPI gaps: `coerce_*`, `batch_trees_to_tiles`/`batch_tiles_to_trees`.
- **A-7** (2, 9; all three) `tests/binding-parity.json` lists 5 classes; the bindings export 36 (WASM), 17 (NAPI), 20 (Python). Comparing the unlisted counterparts' methods (`.d.ts` for WASM/NAPI, `dir()` for Python, snake_case):
  - *RDM cache* differs most: WASM 14, NAPI 9, Python 21 methods. NAPI lacks every lookup and validation (`lookup_label`, `lookup_value`, `lookup_concept`, `get_concept_id_for_value`, `validate_concept`, `validate_value`, `get_collection_ids`); WASM lacks `lookup_by_label`, `search_all`, `validate_concept`, `get_collection`. The TS ORM itself only calls `hasCollection`/`addCollectionFromJson` (`js/static-types.ts:537,544`), so nothing in alizarin breaks, but `getRdmCache()` hands users a different API per backend.
  - *Registry*: NAPI lacks `get_all_full`, `is_empty`, `get_graph_id`, `insert_from_json`, `populate_caches`; Python lacks `contains`, `length`, `memory_stats`, `merge_from_business_data_json`, `load_from_business_data_bytes`.
  - *PseudoNode*: Python exposes 16 members to the JS bindings' 28. It lacks the node fields (`config`, `description`, `isrequired`, `issearchable`, `istopnode`, `ontologyclass`, `parentproperty`, `sortorder`, …).
  - *StaticGraph*: NAPI lacks 16 of WASM's 37 (`get_node_by_alias`, `get_node_by_id`, `push_node`/`push_edge`/`push_card`/…, `ontology_id`, `relatable_resource_model_ids`, …); Python has no native class (dataclass, see B-5).
  - *CoercionResult*: Python lacks `warnings`, so coercion warnings are lost there. NAPI has no class (no standalone `coerce*`, matrix-documented).
  - *Naming drift*: `PopulateResult` `get_value_aliases`/`get_all_values`/`all_nodegroups_map` (W,N) vs `aliases`/`get_values`/`get_nodegroups_map` (P) (written the other way round here at first, and copied wrongly into the manifest generator; fixed with A-8); `NodeConfigManager` `from_graph_json` (W) vs `build_from_graph_json` (N) vs `load_graph_json` (P), `from_graph` (W) vs `build_from_graph` (N); `RdmCache` `add_collections_from_skos_xml` (W) vs `add_from_skos_xml` (P), `length` (W) vs `collection_count` (N); `global_rdm_lookup_label` (W) vs `global_rdm_concept_pref_label` (P). `StaticDomainValue.to_dict` (P) has no JS counterpart: the JS classes have no `toJSON`.
  - Matching: `NodeConfigBoolean`/`Concept`/`Domain`.
- **A-8** (classification, done) Every `gap: unclassified` in `tests/binding-parity.json` now has a reason, from rules in `scripts/guards/binding_manifest.py` (`REASONS`), which rebuilds every class outside `CURATED` on each run: 102 `ecosystem:`, 239 `gap:`. Ecosystem covers the WASM bootstrap and timing, filesystem forms on WASM, Python iterators, `toJSON`/`len()` spellings, and the static types (plain JSON on NAPI, dataclasses in Python, each pointing at B-7/B-5). Found on the way:
  - Two dead or demo exports on WASM, recorded as gaps to remove: `ExampleEdgeWrapper`/`ExampleNodegroupWrapper` (the `wasm_wrapper!` macro demo in `crates/alizarin-wasm/src/wasm_wrappers.rs`) and `WasmNodegroupResult` (never constructed, `pseudo_value.rs:588`).
  - Under NAPI the TS layer still loads WASM: `js/csvModelLoader.ts` validates through WASM `validateModelCsvs` always, and `js/client.ts:391` probes `napi.getRdmNamespaceRaw`, which NAPI lacks, so the SKOS base URI silently falls back to `http://localhost/`.
  - Fixed since: B-1..B-3, F-5/F-6 (and the same `__new__` pattern in `UrlViewModel`, `ConceptValueViewModel`, clm `ReferenceValueViewModel`, filelist `FileItemViewModel`), C-1, and the NAPI SKOS base URI (core now keeps the raw namespace; NAPI binds set/get/get-raw/has/clear; H-3's UUID generators remain). C-1 was worse than recorded: since `d6f7ea8` the parent pseudo *is* set, so `describeField()` raised rather than returned `None`. Removing `_alizarin_rust: Optional[Any]` from `__init__.py` (the module is imported unconditionally later) took mypy from 177 to 71.
  - New (C-7): Python's `describe_field` gives the node alias, TypeScript's `describeField` gives "<model name> - <node name>". To decide in core or the scenario harness (E).
  - The export functions are three shapes of one thing: `build_prebuild_export` (W, returns it), `NapiPrebuildExporter` (N, class), `export_prebuild` (P, to a directory).
- **A-5** (guard) `tests/binding-parity.json` lists only the Rust binding classes, so none of A-1..A-4 could fail it. Extend it (or a sibling) to the ORM classes on both sides, with each difference given an `ecosystem:`/`gap:` reason.

### B. Hand-mirrored logic

Method: for every graph (`graph[]` with `nodes`) and resource (`business_data.resources[]`) in `tests/**/*.json`, compare core's parse+serialise (`alizarin.alizarin.canonical_static(kind, raw)`) with Python's `StaticX.from_dict(raw).to_dict()`. Python's `to_dict` already goes through core, so every difference comes from Python's `from_dict`. 20 objects checked; core rejects 3 fixtures that look deliberately invalid (missing `component_id`, `root`, `resourceinstance_id`). TS has no hand-written static types (it uses the WASM classes), so this class is Python-only.

- **B-1** (1) Python drops `publication.user_id` and `publication.most_recent_edit_id` (`tests/data/models/Person.json`, `tests/definitions/models/22477f01-….json`); core keeps them. Loading an Arches graph through Python and writing it back loses them. Fix: mirror the fields, or better, stop mirroring (B-5).
- **B-2** (1) Card `is_editable: null` becomes `True` in Python (`static_types.py:297` `data.get("is_editable", True)`); core keeps `null` (`tests/data/branches/Addresses.json`). A defaulting difference that changes output.
- **B-3** (1) Resource `tiles: []` → Python omits `tiles`; core writes `[]` (`tests/data/aggregate/business_data.json`, `tests/fixtures/validation/invalid-business-data.json`). An output difference of the kind the parity rule names (which empty fields are written is core's choice).
- **B-4** (1, 9; corrects F-1) Core disagrees with itself on per-language descriptors (`{"en": {...}}`): the business-data loader resolves them (`loader.rs` `FlexibleDescriptors`, "en" else lowest key), while `StaticResource`'s serde ignores them (`canonical_static("resource", …)` drops them). F-1 copied the loader's rule into Python, so Python now matches the loader but not `canonical_static`. Fix in core: one rule, on `StaticResourceDescriptors`' own `Deserialize`; then remove Python's copy (`StaticResourceDescriptors.from_dict`).
- **B-5** (9; root cause of B-1..B-4) `static_types.py` re-implements core's parsing (`from_dict` per type, with its own defaults); only `to_dict` goes through core. Fix: build the dataclasses from `canonical_static(kind, raw)` output, so core decides fields and defaults. Guard: this round-trip check as a test over all fixtures (and Hypothesis shapes), failing on any difference.
- **B-7** (1; NAPI) `js/backend.ts:467-474` `createWasmType` returns its input unchanged on NAPI, so `createStaticResource`/`createStaticNode`/`createStaticNodegroup`/`createStaticCard`/`createStaticPublication`/`createStaticGraphMeta` skip core's parse there, while on WASM the same JSON goes through core's constructor. Callers include `client.ts:192,373` (resources from file or server) and `staticStore.ts:115`. Whatever core normalises (the `resource_instance_lifecycle_state` alias, defaults, descriptor forms, `tiles_loaded`) is normalised on WASM and raw on NAPI until something downstream re-parses it. Fix: route NAPI through core (a NAPI `canonicalStatic(kind, json)` like Python's).
- **B-8** (1, parity rule; NAPI) `js/backend.ts:553-560` `createStaticResourceFromSummary` on NAPI hand-builds `{resourceinstance, metadata, tiles_loaded:false}`, with no `tiles` key, so `resource.tiles` is `undefined` where WASM's `StaticResource.fromSummary` gives `null`. Fix: a core `*_from_json` both call.
- **B-9** (1, 9; WASM vs Python) Resource-shape handling duplicated in binding crates and already diverged: `crates/alizarin-wasm/src/batch_conversion.rs:117-157` (`tilesToTree`; again in `cardsToTree` `:184-200`) and `crates/alizarin-python/src/lib.rs:249-311` (`tiles_to_json_tree`; again from `:372`). Both accept nested or flat input, pull `resourceinstanceid`, build a `StaticResource` and insert `resourceinstanceid`/`graph_id` into the tree by hand. Differences: (a) malformed `tiles` is an error on WASM (`.transpose()?`) but silently an empty tree on Python (`.ok()…unwrap_or_default()`); (b) WASM takes `graph_id` from the graph argument, Python from the resource (errors if absent); (c) both invent a random v4 id when none is given, so the output isn't deterministic. NAPI has no `tiles_to_tree`. Fix: one core function (`tiles_to_tree_from_json(resource_json, graph)`) that all three call. Guard: the output-parity golden.
- **B-11** (1, 9; WASM vs NAPI) `loadFromBusinessDataBytes` returns `{resourceinstanceid, graph_id, isPublic}` refs whose public-scope rule (`scopes` contains `"public"`) is written separately in each binding: `crates/alizarin-wasm/src/graph.rs:2955-2971` and `crates/alizarin-napi/src/lib.rs:468-478`. Identical today, but it's a business rule living outside core; Python's registry lacks the method entirely (A-7). Fix: core returns the refs.
- **B-10** (scale) Hand-built JSON in binding crates by count: `json!(` macros WASM 28, NAPI 11, Python 16; resource field-name literals WASM 38, NAPI 4, Python 31. Not all are wrong (some are argument parsing), but each is a candidate for B-9's treatment; `tests/sourceParity.test.ts` could flag new ones.
- **B-6** (coverage) Only 20 graph/resource objects across the fixtures, so the round trip covers little of Arches' real variety. Add a few real exports (with lifecycle, functions, per-language descriptors, branches) as fixtures for the B-5 guard.

### F. Runtime and stdlib subclasses

Method: AST-listed Python classes subclassing builtins (core + `ext/*/python`), then exercised copy/deepcopy/pickle (and datetime arithmetic) on the constructible ones in `.venv` (Python 3.12.5).

Subclasses with a custom `__new__`: `StringViewModel(str)`, `NonLocalizedStringViewModel(str)`, `EDTFViewModel(str)`, `UrlViewModel(str)`, `ConceptValueViewModel(str)`, `DomainValueViewModel(str)`, `NumberViewModel(float)`, `DateViewModel(datetime)`; in extensions `ReferenceValueViewModel(str)` (clm), `FileItemViewModel(str)` (filelist). List subclasses without `__new__`: `ConceptListViewModel`, `ResourceInstanceListViewModel`, `DomainValueListViewModel`, `EditableList`, `ReferenceListViewModel`, `FileListViewModel`.

- **F-5** (5) `StringViewModel`: `copy.copy`, `copy.deepcopy`, `pickle` all fail (`AttributeError: 'str' object has no attribute 'get'`): `__new__` expects the language dict, and the copy protocols pass the plain string. Anything that copies or caches with pickle (multiprocessing, `functools.lru_cache` in some uses, Django caches) breaks.
- **F-6** (5) `DateViewModel`: copy/deepcopy/pickle fail (`__new__() takes 2 positional arguments but 3 were given`); `date + timedelta` fails even on 3.12 (`… but 9 were given`). On 3.13 `replace()`/`astimezone()` fail the same way (shown with a minimal subclass; F-4 fixed only `for_json`). Fix: let `__new__` accept datetime's own field signature too, or don't subclass `datetime` (hold one), as `for_json` now does internally.
- **F-7** (parity; fixture gap) `str(NumberViewModel(3))` is `'3.0'` in Python; TS's `Number(3).toString()` is `'3'`. The shared datatype fixture (`tests/data/view-models/`) uses only `42.5`, so whole numbers are never compared. Fix: core-defined display for numbers; add an integral value to the fixture.
- **F-8** (6) A stale second install of the Python package (1,888 lines vs the source's 1,944 in `view_models.py`) lives in `ext/clm/python/.venv/lib/python3.13/site-packages/alizarin/`. Running clm tests with that venv tests old code. Local hygiene; worth a note in the contributor docs, or removal.
- **F-todo** NonLocalizedString, EDTF and Number pass copy/pickle. Url, ConceptValue, DomainValue (core) and ReferenceValue, FileItem (extensions) not yet exercised (need constructed inputs); likely the same failure as F-5 where `__new__` takes a structured value.

### G. Extensions × bindings

Method: listed `ext/<e>/<binding>` directories and tracked test files; counted `#[test]` in each `ext/<e>/core/src`; grepped `tests/` and `crates/*/tests` for each extension's packages and modules; read `vitest.napi.config.js` and `.github/workflows/extensions.yml`.

| ext | core (Rust `#[test]`) | WASM/JS | NAPI | Python |
|---|---|---|---|---|
| clm | 22 | own `ext/clm/js/tests/clm.test.ts` (WASM config); docExamples; `extensionConfig.test.ts` | `extensionConfig.test.ts` only | 8 own files |
| filelist | 10 | own `filelist.test.ts` (WASM); docExamples | **none** | 3 own files |
| geo | 41 | via `tests/dual-backend/utm.test.ts`, `aggregate.test.ts` | same two | via `crates/alizarin-python/tests/test_utm.py`, `test_aggregate.py`; **no own tests** |
| pg | 7 | (no binding) | (no binding) | **none** |

- **G-1** (6) Extension JS tests run only through the WASM config (`extensions.yml:78-91`, `npm run test:all -- ext/<e>/js/tests/`), never under `vitest.napi.config.js`, so `@alizarin/clm-napi` coercion, display and references are untested except clm's config. `@alizarin/filelist-napi` has no test anywhere. Fix: run each extension's tests under both configs (dual-backend style).
- **G-2** (6) `vitest.napi.config.js:15-21` excludes 7 core test files as "To port: they build WASM objects directly": `buildResourceDescriptors`, `graphConfig`, `graphs`, `resources`, `static-types`, `type-coercion` (+ `deriveProvider`, justified). Each is NAPI coverage that doesn't exist. Track as `gap:` entries until ported.
- **G-3** (6) geo's validate and fold handlers (documented in `docs/extension-abi.mdx`) are exercised through the bindings only for UTM derive and aggregates; there is no binding-level test of geo validation or folding on any backend.
- **G-4** (6, 7) `ext/pg` has core and Python only, and no Python tests (see D-5). Whether it is Python-only by design isn't recorded anywhere; it should be, as `ecosystem:`.
- **G-5** (7) `notes/backend-support-matrix.md` has no extension rows. Add them, generated from the same source as A-5.
- **G-6** (8) Not re-verified this round: each extension module (`.wasm`, `.node`, `.so`) links its own copy of core, so registries aren't shared and providers cross via the C ABI (ABI 5, `DeriveProviderInfo`/`HostGraph`). Worth a test per binding that a provider registered by the extension module is visible to the main module's registry. The UTM tests do this for derive; nothing does it for type handlers on NAPI beyond config.

### H. Docs and comments

Method: parsed all 21 ```python blocks in `docs/**/*.mdx` and resolved every `alizarin…` name against the installed package; checked `from 'alizarin'` imports in all 72 ts/js blocks against `js/main.ts`'s export list; grepped code comments for claims about other backends ("WASM-only", "not available in NAPI", "compiled into", …) and checked each.

- Docs code: **clean.** All Python blocks parse and every `alizarin` name resolves; every TS import from `'alizarin'` is exported. (The earlier docs pass fixed what there was.) Not checked: that the blocks run (they need data); see the H guard.
- **H-1** (7) `js/backend.ts:276-292`: the `registerExtensionHandler` doc says NAPI handlers are "compiled into the native binary (Rust crates linked at build time)"; base crates no longer link extensions, which register through their own native modules. On NAPI the function only warns if `hasExtensionHandler` is false; the comment should say that.
- **H-2** (7) `js/interfaces.ts:139` "PseudoNode creation (WASM-only; may be undefined in NAPI)": NAPI has `createPseudoNode`/`createPseudoNodeChild` (`crates/alizarin-napi/src/instance_wrapper_napi.rs:1812`, `index.d.ts:520,522`). Stale; the optional typing it justifies can go.
- **H-3** (2, parity) RDM namespace functions: WASM exports 10 (`setRdmNamespace`, `getRdmNamespace`, `getRdmNamespaceRaw`, `hasRdmNamespace`, `clearRdmNamespace`, `parseRdmNamespace`, `generateCollectionUuid`, `generateConceptUuid`, `generateValueUuid`, `labelsToDeterministicString`; `crates/alizarin-wasm/src/rdm_namespace_wasm.rs:66-190`). Python has 4 (`set/get/has/clear_rdm_namespace`). NAPI has none; it takes a namespace only as an argument (`index.d.ts:59`). The raw-string storage is commented "WASM-only" (`:44`) with no reason given. Deterministic UUID generation is exactly what must agree across backends. Guard: A-5 must cover free functions as well as classes.
- **H-guard** Extract the docs' Python blocks into a pytest that runs them against small fixtures (the TS side already has `docExamples` for `examples/*.tsx`, but not for inline ```ts blocks in `.mdx`).

### J. Release consistency (checked against registries 2026-11-19, after alpha.148)

All 8 npm packages, 5 PyPI projects and 6 crates have 2.0.0-alpha.148.

- **J-1** (10) npm `latest` dist-tags are inconsistent and some are badly stale: `alizarin` → `1.0.0`; `@alizarin/clm`, `@alizarin/filelist`, **`@alizarin/napi`** → `0.2.1-alpha.83`; `@alizarin/geo`, `@alizarin/{clm,filelist,geo}-napi` → `2.0.0-beta.3`. A plain `npm install @alizarin/napi` gets alpha.83, which also mismatches `alizarin@latest` (1.0.0). Fix: decide what `latest` means while 2.0 is in alpha (all point at the newest 2.0 alpha, or all at 1.0-era equivalents) and set it in the release workflows. Guard: post-release check that every package's `latest` matches the policy.
- **J-2** (10) PyPI: `alizarin` a148 has 5 files (4 wheels + sdist); `alizarin-clm`, `-filelist`, `-geo`, `-pg` have 4 (wheels only, no sdist), so the extensions can't be installed on platforms without a prebuilt wheel (e.g. Linux aarch64, musl) while the main package can. Confirmed: `release-python.yml`'s only `build-sdist` job (`:80-101`) builds the main package. Fix: an sdist per extension, or document the supported platforms.
- **J-3** (10) PyPI and crates.io `max`/`latest` is `2.0.0b9`/`2.0.0-beta.9` for every project, above all alphas (known; unpinned installs get beta.9). Accepted under the alpha-line policy, but the docs' pinned versions must be bumped every release; a release-time check could do it.

### I. Redundancy register (compiled from the sections above)

| Duplicate | Where | Proposed resolution |
|---|---|---|
| Static-type parsing | core serde ↔ `static_types.py` `from_dict` (B-1..B-5) | Python builds from `canonical_static` output |
| Descriptor-language rule | core `loader.rs` `FlexibleDescriptors` ↔ core `StaticResourceDescriptors` serde ↔ Python (B-4) | one rule in core's `Deserialize` |
| ORM layer | TS (`js/`) ↔ Python (`python/alizarin`) (A-1..A-3, C-1, F-5..F-7) | keep both, move behaviour into core wrappers, guard with E scenarios |
| TS interfaces | two `IModelWrapper`/`IInstanceWrapper` declarations (C-3) | one each |
| Function names | same function, different names per binding (A-6) | one name, aliases for a release |
| Struct literals | 21 sites patched by hand for the lifecycle fields | `..Default::default()` or a constructor in core |
| Number/date display | Python `str()`/`for_json` ↔ TS `toString()`/`toISOString()` (F-4, F-7) | core-defined display |

### C. Static checking

Method: mypy (installed into `.venv`) over `crates/alizarin-python/python/alizarin` (`--ignore-missing-imports --follow-imports=silent`, 304 lines: 77 union-attr, 46 no-untyped-def, 23 attr-defined, ...); `tsc --noEmit --strict` with the repo tsconfig (259 lines: 39 TS2345, 26 TS2339, 18 TS18047, 17 TS2531, 17 TS18048, 6 TS2551, ...). Outputs in the session scratchpad (`mypy.txt`, `tsc.txt`); regenerate rather than trust.

- **C-1** (3, 2; parity) Python view models' `describeField()`/`describeFieldGroup()` (~15 classes in `view_models.py`, e.g. `:295-299`) call `self.__parentPseudo.describeField()`, but `PseudoValue` defines only `describe_field` (`pseudos.py:81,316`): a dead call like `2d3b6ef`. It never raises because Python's `get_view_model` (`view_models.py:~1816`) takes `parentPseudo` but never stores it on the view model, so `describeField()` always returns `None`. TypeScript stores it (`js/viewModels/getViewModel.ts:137`, `js/pseudos.ts:335`) and returns the real description. Two bugs masking each other; a parity gap either way. Confirmed by reading; confirm at runtime when fixing. Guard: mypy `attr-defined` in CI; an ORM parity scenario for `describeField` (E).
- **C-2** (3, guard gap) The native module has no `.pyi` stubs, so mypy can't check calls across the Rust boundary: 12 `RustResourceInstanceWrapperCore` methods flagged `attr-defined` (`instance_wrapper.py:143-662`) all exist at runtime, which leaves real misses in that layer undetectable. Fix: generate stubs (pyo3 `stub_gen`/`pyo3-stubgen`) and check them in CI against `binding-parity.json`.
- **C-3** (9) Two `IModelWrapper` declarations disagree: `js/interfaces.ts:118,121` has `getNodeObjectFromAlias`/`getChildNodeAliases`, the one `renderers.ts`/`semantic.ts` see doesn't (TS2551 ×4). The methods exist (`graphManager.ts:950,958`); the duplicated interface is the defect. Likewise `IInstanceWrapper` lacks `wasmWrapper`/`getDescriptors` that `ResourceInstanceViewModel.ts:80,141,213` and `semantic.ts:115,223` use (TS2339 ×10). Fix: one interface each. Also `wasmWrapper` holds the NAPI wrapper too (`graphManager.ts:60`); misleading name.
- **C-4** (3, possible) Smaller TS2339s to read individually: `client.ts:232,374` `__source` on `StaticResource`; `renderers.ts:69` `forJson` on `String`; `utils.ts:213,216,227` `inner`/`__has` on `IPseudo`; `semantic.ts:154,288` `parentNode` on the pseudo union; `graphManager.ts:1110` `pruneTiles` on `never`; test-only `buildResourceDescriptors.test.ts:405` `map_popup` vs `mapPopup` (shape mismatch between snake and camel descriptors?).
- **C-6** (checked; not bugs) mypy's 5 `name-defined` errors (`view_models.py:1410,1716,1717,1730,1779`: `StaticNodeConfigBoolean`, `PseudoNode`, `StaticEdge`) are all quoted annotations whose types aren't imported under `TYPE_CHECKING`. Harmless at runtime; fix with `TYPE_CHECKING` imports to lower the baseline.
- **C-5** (guard) Neither checker runs in CI. The non-strict `tsc` the build uses passes; mypy has never run. Decide a baseline (count per code) and fail on increase, starting with `attr-defined`/TS2339/TS2551.

- **D-10** (scanner notes) TS hits read and judged fine: `docExamples.test.ts:65` (mock fetch 404), `outputParity.test.ts:211,227` (failures collected and reported), `graphManager.test.ts:664,690` and `wasmErrors.test.ts:25` (error captured then asserted), `bindingParity.test.ts:35` `skipIf` (documented `null` exceptions), `viewModels.test.ts:69,72` and `type-coercion.test.ts:52` (parametrised expectations).
