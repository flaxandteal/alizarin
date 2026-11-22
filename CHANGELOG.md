# Changelog

Changes that consumers can notice: new API, behaviour changes, removals and
renames. Breaking changes are marked **Breaking**, with what to do instead.
Renames get no aliases; they are listed here. The parity work behind most
entries is tracked in `PARITY-RESEARCH.md`.

## Unreleased

### Added
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

### Fixed
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
