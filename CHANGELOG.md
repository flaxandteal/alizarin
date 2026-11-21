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
- Malformed resource descriptors (`{"name": 5}`, or the same under a
  language) are an error when a resource is read. The business-data loader
  used to drop them silently to empty.

### Fixed
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
