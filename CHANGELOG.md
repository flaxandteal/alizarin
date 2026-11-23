# Changelog

Changes that consumers can notice: new API, behaviour changes, removals and
renames. Breaking changes are marked **Breaking**, with what to do instead.
Renames get no aliases; they are listed here. The parity work behind most
entries is tracked in `PARITY-RESEARCH.md`.

## Unreleased

### Added
- NAPI: the RDM cache's lookups WASM and Python have (`getCollectionIds`,
  `lookupLabel`, `lookupConcept`, `lookupValue`, `getConceptIdForValue`,
  `validateValue`, `addCollectionsFromSkosXml`), and the global RDM cache
  functions (`setGlobalRdmCache`, `hasGlobalRdmCache`, `clearGlobalRdmCache`,
  `addToGlobalRdmCacheFromSkosXml`, `addCollectionToGlobalRdmCache`,
  `removeFromGlobalRdmCache`, `globalRdmLookupLabel`, `globalRdmHasCollection`,
  `globalRdmGetParentId`, `globalRdmGetCollectionIds`). Tree-to-tiles label
  resolution against the global cache now works from NAPI.
- Python: `RustRdmCache.lookup_label`, `remove_collection` (returning whether
  it was there) and `resolve_labels(tree_json, alias_to_collection, strict)`;
  `global_rdm_lookup_label` (a label string, as WASM's and NAPI's),
  `global_rdm_get_collection_ids` and `remove_from_global_rdm_cache`. The
  package now also exports the `global_rdm_*` lookups it already had
  (`global_rdm_has_collection`, `global_rdm_lookup_value`,
  `global_rdm_concept_pref_label`, `global_rdm_get_parent_id`).
- All three bindings' RDM lookups and global-cache functions are core's, held
  to one set of expected results (golden routes `rdm`, `rdm:global`).
- NAPI and Python: the deterministic id functions WASM had,
  `generateCollectionUuid` / `generate_collection_uuid`,
  `generateConceptUuid`, `generateValueUuid`, `parseRdmNamespace`,
  `labelsToDeterministicString`; NAPI also `generateUuidV5` (Python had
  `generate_uuid_v5`). Core's on every binding, held to one set of expected
  ids.
- Python: `GraphManager.load_prebuild(dir, base_uri)` loads a prebuild
  directory's graphs, collections and business data, and
  `GraphManager.load_graph(name_or_id)` finds a model by class name (case
  and spaces aside) or graph id, as TypeScript's `loadGraph`. The manager's
  resources are held by core's registry (`gm.registry`), which references
  resolve against. Core: `StaticResourceRegistry::load_prebuild_business_data`;
  NAPI: `registry.loadPrebuildBusinessData(dir)`; Python:
  `ResourceRegistry.load_prebuild_business_data(dir)`.
- NAPI: `treeToTiles`, `batchTreesToTiles`, `mergeResources`,
  `batchMergeResources`, with WASM's arguments. TypeScript: the same four
  exported from `alizarin`, on either backend. Core now does all of each
  (`tree_to_tiles_from_json`, `batch_trees_to_tiles_from_json`,
  `merge_resources_from_json`, `batch_merge_resources_from_json`); WASM and
  Python wrap them.
- NAPI: `compareNodes(a, b)`; Python: `compare_nodes(a, b)`; TypeScript:
  `compareNodes` from `alizarin`, on either backend. Whether two nodes are
  the same apart from their ids, as WASM's `StaticNode.compare`.
- NAPI: `setRdmNamespace`, `getRdmNamespace`, `getRdmNamespaceRaw`,
  `hasRdmNamespace`, `clearRdmNamespace`, as on WASM. Python:
  `get_rdm_namespace_raw`. TypeScript: `setRdmNamespace` and
  `getRdmNamespaceRaw` exported from `alizarin`, on either backend. Core now
  keeps the string the namespace was set from (it was a WASM-only global).
- NAPI: `tilesToTree`, `cardsToTree`, `batchTilesToTrees`, against the
  registered graph the resource names. TypeScript: the same three exported
  from `alizarin`, on either backend. Python: `cards_to_json_tree` exported
  from `alizarin` (it was only in the compiled submodule).
- NAPI: `constructStatic(kind, value)` and `resourcesFromBusinessDataJson`.
  Python: `construct_static` and `resources_from_business_data_json`. A
  static type, or a business data file's resources, built by core as WASM's
  constructors build them. Python's `canonical_static` is now core's, and
  takes `"graph_meta"`.
- Python: `ResourceRegistry.load_from_business_data_bytes`, as WASM's and
  NAPI's `loadFromBusinessDataBytes`. Core now decides each ref's `isPublic`
  (its `__scopes` include `"public"`); both bindings had their own copy.
- NAPI: `staticResourceFromSummary`. Python: `static_resource_from_summary`
  and `summary_to_metadata`. A lightweight resource (no tiles,
  `tiles_loaded: false`) or the metadata from a registry summary, as WASM's
  `StaticResource.fromSummary` and `summary.toMetadata()` give; core builds
  all of them. `createStaticResourceFromSummary` on NAPI uses it (it built the
  object by hand).

### Changed
- Tiles to trees (`tilesToTree`, `cardsToTree`, `batchTilesToTrees`; Python
  `tiles_to_json_tree`, `cards_to_json_tree`, `batch_tiles_to_trees`) read a
  resource by one rule in core. **Breaking**, where it differed:
  - a resource with no `resourceinstanceid` takes its tiles'
    `resourceinstance_id`; with neither it is an error (a random id was
    invented). Give the resource its id.
  - a resource naming another graph than the one converted against is an
    error (WASM relabelled it with the graph's id).
  - malformed `tiles` are an error in Python too (it gave an empty tree).
  - Python's `batch_tiles_to_trees` result always has `errors` and
    `error_count`, as WASM's did.
- NAPI: `createStaticResource`, `createStaticNode`, `createStaticNodegroup`,
  `createStaticEdge`, `createStaticCard`, `createStaticPublication`,
  `createStaticCardsXNodesXWidgets`, `createStaticGraphMeta` (and `new
  StaticResource(…)` etc.) build through core, as on WASM; they returned
  their input unchanged. `parseStaticResources` parses through core too.
  **Breaking**, on NAPI only, where it differed from WASM:
  - malformed input is an error (it was passed through).
  - a resource gets `tiles_loaded` (as given, else whether it has tiles).
  - `parseStaticResources` on JSON with no `business_data.resources` is an
    error (it gave `[]`).
  - `new StaticTile(…)` gives a `NapiStaticTile`, as `createStaticTile` does
    (it gave the input object).
- Business data is read by one reader in core on every path (the prebuild
  loader, `loadFromBusinessDataBytes`, `parseStaticResources` /
  `StaticResource.fromBusinessDataJsonString`, Python's
  `resources_from_business_data_json`). Where they differed:
  - **Breaking:** `parseStaticResources` gives a resource with no tiles
    `tiles_loaded: true` (it was `false`): the file holds all its tiles. A
    declared `tiles_loaded` is kept (the loader overrode it with `true`).
  - `parseStaticResources` accepts a resource without `descriptors`, with
    `metadata: null`, or a bare resource rather than the wrapper, as the
    loader did. Any `StaticResource` read from JSON takes absent
    `descriptors` as empty.
  - A malformed `business_data` wrapper is reported as such (the error was
    about a bare resource's missing `resourceinstance`).
  - The loader reads about 30% faster.
- Trees to tiles on WASM, where it differed from Python (now one rule in
  core):
  - concept labels in a tree resolve to ids against the global RDM cache
    when one is set (`setGlobalRdmCache`), as in Python; they were left as
    given.
  - `batchTreesToTiles(…, fromCamel: true)` matches camelCase aliases, as
    `treeToTiles` and Python do, instead of rewriting every key in the trees
    to snake_case first.
  - `batchTreesToTiles` always returns `warnings` (it was dropped when
    empty), and now `warning_count`, as Python.
  - a malformed batch in `batchMergeResources` gives core's message
    (`Batch 2: …`), as Python.
- **Breaking:** `StaticNode.compare(a, b)` (WASM) returns a name, not a
  number: `"same"` (was `2`, and `true` for one object twice),
  `"same_ids_partial"` (`1`), `"nodeid_conflict"` (`-1`),
  `"nodegroup_id_conflict"` (`-2`), `"graph_id_conflict"` (`-3`),
  `"different"` (`false`). The rule is now core's, the per-node equality a
  graph diff needs, and a bad argument throws (it logged and gave `null`).
  An id missing on one side and `null` on the other now counts as unset on
  both.
- Business data from CSV (`buildResourcesFromBusinessCsv`, NAPI
  `buildBusinessDataFromCsv`, Python `build_resources_from_business_csv`) is
  one core function on every binding: extension types always go through the
  extension registry (WASM and Python used none; NAPI only with an
  `rdmCache`). Concept labels resolve against the collections, or against
  NAPI's `rdmCache` when given, as before. An invalid graph or collections
  argument is `Invalid graph JSON: …` / `Invalid collections JSON: …` on
  every binding.
- WASM: `buildGraphFromModelCsvs` returns core's serialisation (sorted keys),
  as NAPI does.
- Python: `build_graph_from_model_csvs` and `validate_model_csvs` exported
  from `alizarin` (they were only in the compiled submodule).
- **Breaking:** extension coercion returns core's `{tileData, displayValue,
  warnings}` (`warnings` always present) on NAPI (`extensionCoerce`, which
  dropped `warnings` when empty) and Python (`coerce_with_extension`, which
  gave a tuple of two JSON strings and dropped warnings). Without a handler
  that can coerce, both give `null` / `None` (Python raised `KeyError`).
- **Breaking:** a populate result's `getAllValues()` is a plain object of
  pseudo lists by alias on WASM and NAPI (WASM gave a `Map`; NAPI gave
  summaries, `{nodeAlias, isSingle, isLoaded, totalValues}`, which each list
  has).
- Python: `TreeToTilesIterator` converts each tree as `json_tree_to_tiles` and
  `batch_trees_to_tiles` do, resolving concept labels against the global RDM
  cache (it left them unresolved).
- **Breaking:** `importPrebuild` / `import_prebuild` return core's summary,
  `{graphIds, collectionIds, collections, ontologyConfigs}`, on NAPI and
  Python (Python's were `graph_ids`, `collection_ids`; it keeps its
  validator, as `ontologyValidator`). NAPI's takes the extra reference-data
  and ontology directories, as Python's did.
- **Breaking:** prebuild export data is core's on WASM and NAPI:
  `{graphFiles, referenceDataFiles, businessDataFiles}` of `{relativePath,
  content}` (NAPI's `buildExportData` gave `{files, graphFileCount,
  referenceDataFileCount}`). WASM's `buildPrebuildExport` takes `graphIds`,
  as NAPI's did. TypeScript: `buildPrebuildExport(baseUri, graphIds?)` from
  `alizarin`, on either backend.
- **Breaking (Python):** view models' `describeField()` /
  `describeFieldGroup()` (and pseudos' `describe_field` /
  `describe_field_group`) name a field as TypeScript does: `"<model> - <node
  name>"`, and `"<model> - <nodegroup name>"` for its group, `[…]` for a
  list (they gave the node's alias, and the nodegroup's id). The rule is
  core's (`describe_field`, `describe_field_group`, on every binding); a
  nodegroup's name is its grouping node's (Python's `nodegroup_name`), where
  Python took the first node it found in the group.
- **Breaking (Python):** `ResourceRegistry.populate_caches` returns
  `unknownReferences` / `hasUnknown` (were `unknown_references` /
  `has_unknown`), and `resolve_path` returns `nodegroupId` / `isSingle` /
  `targetNodeId` (were snake_case): core now builds both outputs, as WASM and
  NAPI already named them. Rename the keys you read.
- Python: the static types' `from_dict` read their input through core
  (`construct_static`), as WASM's constructors and NAPI's `createStatic*` do,
  instead of parsing it themselves. **Breaking**, where Python's own parse
  differed:
  - a dict missing a field core requires is a `ValueError` (Python filled in
    defaults, e.g. `StaticNode.from_dict({"nodeid": …})` gave `name=""`).
    Give the field.
  - `StaticResource.from_dict` settles `tiles_loaded` (as given, else whether
    it has tiles), as WASM's `new StaticResource(…)` does; `to_dict` then
    includes it.
  `alizarin.static_types.from_core(cls, data)` reads a dict that is already
  core's output (from a native call) without building it again.
- A resource's descriptors (`getDescriptors()`, `getName()`; Python
  `get_descriptors()`) take the resource's stored `name` when the graph's
  descriptor function gives none, or the graph has none (it was `null`).
  Arches keeps the computed name there. The resource's own `descriptors`
  are unchanged, so exports are too.
- Malformed resource descriptors (`{"name": 5}`, or the same under a
  language) are an error when a resource is read. The business-data loader
  used to drop them silently to empty.

### Removed
- WASM: `ExampleEdgeWrapper` and `ExampleNodegroupWrapper`, the
  `wasm_wrapper!` macro's demo, which had leaked into the API; use
  `StaticEdge` / `StaticNodegroup`. `WasmNodegroupResult`, which nothing
  returned.

### Fixed
- A graph built from CSVs lists its collections in the order they first
  appear in collections.csv; it was hash order, so it changed from run to
  run.
- A graph exports to a file named by its English name (else its
  lowest-keyed language's), as documented; it took the current language,
  so the same graph's file name changed with the UI language.
- Python: `str()` of a number view model is as TypeScript's: `"7"`, not
  `"7.0"` (and `1e+21`, `NaN`, …), by core's `number_display`
  (ECMAScript's Number::toString, from the `ryu-js` crate), exported from
  `alizarin`.
- `ResourceInstanceViewModel.getName()` / `getSlug()` / `getDescription()` /
  `getMapPopup()` give `null` when there is no value on either backend (NAPI
  gave `undefined`); typed `string | null`.
- `ArchesClientLocal.getResource` on a resource not in the file
  `resourceIdToFile` names says so (it threw a `TypeError` setting
  `__source`), and reads a one-resource file as well as a business data file.
- `graphManager.initialize()` takes a plain options object
  (`{ graphs: [...] }`); `ConfigurationOptions` is not exported, so typed
  callers could not pass any.
- `loadGraph()` / `get()` on an unknown model name list the names known (and
  say if `initialize()` has not run), in one message for both.
- Docs: new [Local testing](docs/local-testing.mdx) (`ArchesClientLocal`
  setup, the graph list's format, preloading for references) and
  [Resource references](docs/resource-references.mdx) (stubs, retrieval,
  descriptors and `__cache`) guides, run as written by
  `tests/dual-backend/docsLocalRecipe.test.ts`; `loadGraph`'s naming
  documented in the JS API and LLM reference.
- Python CLI: `alizarin to-trees` and `alizarin convert` failed on every run
  (they passed their bindings an argument those do not take).
- Per-language resource descriptors (Arches' `{"en": {"name": …}}`) are read
  wherever a resource is, as "en", else the lowest language, and written flat.
  Only the business-data loader read them; elsewhere (`StaticResource` from
  JSON, Python's `canonical_static`) they became empty.
- NAPI: `ArchesClientLocal` parsed collections from SKOS XML against
  `http://localhost/` whatever namespace was set, since NAPI had none; it uses
  the namespace's string, as WASM does.
- Python: `StaticGraph.to_dict` keeps the keys of a graph's `publication`
  that it does not model (`user_id`, `most_recent_edit_id`), as core does;
  `StaticPublication` holds them in `extra`.
- Python: a card's `is_editable: null` stays `None` (it became `True`), and a
  resource's `tiles: []` is written back as `[]` (it was dropped), as core
  does.
- Python: `StringViewModel`, `DateViewModel`, `UrlViewModel`,
  `ConceptValueViewModel`, and clm's `ReferenceValueViewModel` and filelist's
  `FileItemViewModel` survive `copy.copy`, `copy.deepcopy` and `pickle` (they
  raised). `DateViewModel` supports `datetime` arithmetic, `replace()` and
  `astimezone()` (they raised); the results are `DateViewModel`s.
- Python: `describeField()` / `describeFieldGroup()` on view models (core, clm,
  filelist) raised `AttributeError` once the view model had its parent; they
  return the field's description. (It still differs from TypeScript's, which
  prefixes the model name.)
- Python: `ResourceModelWrapper.all()` returns the model's registered
  resources (it returned `[]`). `params` takes `limit` and `lazy` (default
  `False`, as in TypeScript).
- Python: reading through a resource-instance reference into another resource
  (`await group.members[0].name`), `for_json(cascade=True)` on an unloaded
  reference, and the reference's cache entry raised `AttributeError`
  (`GraphManager` has no `get_root_view_model`). They now load the resource
  through `GraphManager.getResource`, as TypeScript does.
- Python: `StaticResource.from_dict` accepts Arches' per-language descriptors
  (`{"en": {...}}`), which crashed `GraphManager.register_resource` on
  resources straight from an Arches export. It takes `"en"`, else the first
  language, as core's business-data loader does. (Core's own
  `StaticResource` parsing still drops them; that is to be made one rule in
  core.)

## 2.0.0-alpha.148

### Added
- Resource lifecycles: `resourceLifecycleState` / `transitionResourceLifecycle`
  (Python `resource_lifecycle_state` / `transition_resource_lifecycle`) on
  every backend. Transitions are strict by default. Composing layers whose
  lifecycle states disagree gives a Conflict state, resolved by a transition
  record. See `docs/lifecycles.mdx`.

### Fixed
- Python: `DateViewModel.for_json()` returned the raw stored string on
  Python 3.12+ (where `datetime.replace` builds the subclass); it returns the
  ISO timestamp again.
