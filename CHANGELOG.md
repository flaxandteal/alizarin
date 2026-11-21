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

### Changed
- Malformed resource descriptors (`{"name": 5}`, or the same under a
  language) are an error when a resource is read. The business-data loader
  used to drop them silently to empty.

### Fixed
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
