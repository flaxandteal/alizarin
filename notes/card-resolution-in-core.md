# Card resolution in core — design sketch

## Background: the problem this solves

Profiling Catalina's single-asset report page showed ~92% of time in
`FinalizationRegistryCleanup::DoCleanup` → `__wbg_staticitle_free` →
`StaticTile drop_slow` → `serde_json` map teardown. The cause is **not** batch
conversion or the geo/validation work — it is the card renderer walking
alizarin's viewmodel Proxy per field.

Root cause: `getRootViewModel()` (`js/graphManager.ts`) is uncached. Every call
re-runs `getRoot()` → `getRootPseudo()` → `root.getValue()` and re-wraps a fresh
`SemanticViewModel`, each crossing the FFI boundary with exported pseudo/tile
objects. It is invoked on **every** `__has` (`ResourceInstanceViewModel.ts`) and
**every** Proxy attribute access (`getOrmAttribute`). Catalina's
`CardRenderer.renderCard` does `await parentVm.__has(alias)` then
`await parentVm[alias]` for every card and every `parentAccessPath` step, so a
single rich asset triggers hundreds–thousands of full root re-resolutions, each
exporting `StaticTile` wasm objects that JS immediately drops → the
`FinalizationRegistry` storm.

Origin: the "run alizarin mostly via rust" migration (`54d2c39` / `e3e0187`)
made value navigation re-materialize the root VM (and tiles) per access.

There are two fixes:

- **Tactical:** memoize `getRootViewModel()`. Small; unblocks Catalina; leaves
  the per-field JS walk in place.
- **Strategic (this doc):** resolve the card tree **in core** in one pass, so
  the per-field boundary crossing becomes structurally impossible.

## What a card is, abstractly

A card is not a SQL view. A card-*tree* is close, but the precise statement is:

> A card-tree is a **schema-level, hierarchical read-projection over the
> nodegroup aggregate, annotated with presentation semantics** — closer to a
> **GraphQL selection set** (or an XSLT/template over a nested document) than a
> flat SQL view.

Where the view analogy **holds**:

- Schema-level, instance-independent, computed at read time — no data of its own.
- A projection/selection over the underlying structure.
  `render(card_tree, resource_id)` ≈ `SELECT * FROM view WHERE key = resource_id`
  — parameterised by the resource, like a view with a bound key.
- Composable and hierarchical — cards nest, mirroring nodegroup nesting.

Where it **breaks**:

1. **Shape.** A SQL view yields homogeneous tuples; a card-tree yields a
   heterogeneous nested document — typed values with per-field metadata.
2. **Presentation contract.** Cards carry widgets, labels, i18n, sort order,
   visibility, descriptions, cardinality rendering. A view is pure data; a card
   is data **+ a rendering contract**.
3. **Bidirectionality.** Cards are also the edit surface — they drive forms. So
   a card is closer to an **updatable view bound to a form**. The report path
   only exercises the read direction.
4. **Graph-walk semantics.** Collector/semantic nodes (empty parent, children
   carry data), edges crossing nodegroup boundaries — a tree traversal, not
   relational algebra.

In CQRS terms: a card-tree is a **read model / projection** over the resource
aggregate. That framing is what justifies moving it to core: we are not "moving
UI code into core," we are moving the **read-model resolution** into core and
leaving **formatting** in the host.

## The contract

One function, one boundary crossing:

```rust
// Schema-level, built once per graph, cached. Instance-independent.
pub fn build_card_spec(graph: &StaticGraph) -> CardSpecTree;

// Instance-level. Walks the resource's tiles ONCE, resolves every widget
// value through the datatype handlers, emits a presentation-neutral tree.
pub fn resolve_card_tree(
    resource: &StaticResource,
    spec: &CardSpecTree,
    graph: &StaticGraph,
    registry: Option<&ExtensionTypeRegistry>,
    opts: &RenderOptions,          // language, public_view, permitted nodegroups
) -> RenderTree;
```

The host calls `renderCardTree(resource)` once, gets a `RenderTree` back, and
formats it. Boundary crossings drop from **O(fields × navigation-steps)** to
**1**.

## Two shapes: spec (schema) vs render (instance)

**CardSpec** — a direct port of Catalina's `buildCardTree`
(`CardTreeNode`/`WidgetInfo`), moved into Rust. Schema-derived, cache per graph:

```rust
pub struct CardSpecTree { pub roots: Vec<CardNodeSpec> }

pub struct CardNodeSpec {
    pub card_id: String,
    pub name: StaticTranslatableString,
    pub description: StaticTranslatableString,
    pub nodegroup_id: String,
    pub nodegroup_alias: String,
    pub sortorder: i64,
    pub visible: bool,
    pub active: bool,
    pub cardinality: Cardinality,          // One | N
    pub widgets: Vec<WidgetSpec>,
    pub children: Vec<CardNodeSpec>,
    pub parent_access_path: Vec<String>,   // alias path through intermediate semantic nodes
}

pub struct WidgetSpec {
    pub node_id: String,
    pub node_alias: String,
    pub label: StaticTranslatableString,
    pub sortorder: i64,
    pub visible: bool,
    pub access_path: Vec<String>,          // nodegroup-root → node
    pub is_root_node: bool,
}
```

**RenderTree** — the instance-resolved projection. This is what crosses the
boundary:

```rust
pub struct RenderTree { pub cards: Vec<RenderedCard> }

pub struct RenderedCard {
    pub card_id: String,
    pub name: String,              // already localised to opts.language
    pub description: String,
    pub nodegroup_alias: String,
    pub cardinality: Cardinality,
    pub instances: Vec<RenderedInstance>,   // 0 = no data; 1 for cardinality-One; N for N
}

pub struct RenderedInstance {
    pub fields: Vec<RenderedField>,
    pub children: Vec<RenderedCard>,        // nested child cards, per instance
}

pub struct RenderedField {
    pub node_alias: String,
    pub label: String,
    pub datatype: String,
    pub value: ResolvedValue,
}

// Mirrors the renderers.ts visitor dispatch — typed leaves, NOT strings.
pub enum ResolvedValue {
    String { text: String, language: Option<String> },
    NonLocalizedString(String),
    Number(f64),
    Boolean(bool),
    Date(String),
    Concept { id: String, label: String },              // host → URL
    DomainValue { id: String, label: String },
    ResourceReference { resource_id: String, graph_id: Option<String>, display: String }, // host → URL
    Url { url: String, label: Option<String> },
    GeoJson(serde_json::Value),                          // host → map URL
    Extension { datatype: String, display: serde_json::Value }, // via registry.render_display
    Null,
}
```

## The seam — this is the whole game

| Core (Rust) | Host (JS/Catalina) |
|---|---|
| Build card spec from `graph.cards` + `cards_x_nodes_x_widgets` + edges | — |
| Walk tiles once; index by nodegroup; resolve accessPaths | — |
| Cardinality 1-vs-n, collector/empty-parent handling | — |
| Visibility/permission filtering (`opts.public_view`, permitted nodegroups) | — |
| Resolve each value → **typed `ResolvedValue`** (incl. `registry.render_display` for reference/file-list) | — |
| Localise labels/strings to `opts.language` (+ fallback) | — |
| — | `ResolvedValue` → **URL** (concept/resource/geojson callbacks) |
| — | markdown/HTML (`marked.parseInline`), section assembly |

Core resolves the **read model**; the host does **formatting + URLs**. The line
is bright: **no `marked`, no HTML, no URL construction in Rust.** Cross it and
core becomes a presentation engine; hold it and core is the view resolver it
already structurally is.

`renderers.ts`'s `MarkdownRenderer` already *is* this split, badly — its
abstract methods take `conceptValueToUrl`, `resourceReferenceToUrl`, etc. (host
concerns) but drive them by walking the VM Proxy (the expensive part). Under
this design it becomes a pure `RenderTree` walker: same formatting methods, no
boundary traffic.

## Reality check — most of this already exists

The sketch above was written before auditing core. Correction: **the resolver is
already built and already exposed in wasm.** The strategic work is far smaller
than "build a new subsystem."

What exists:

- **Card ingestion:** core has a `CardIndex` (`graph::card_index`) with
  `cards_by_id`, `widgets_by_card`, `card_children`; `graph.card_index()` returns
  it. (My earlier "cards are dropped at parse" claim was wrong — they're indexed.)
- **The one-pass resolver:** `card_traversal.rs` — `serialize_root_cards`,
  `serialize_card`, `cards_to_tree`. Output is exactly the `RenderTree` shape:
  `{card_id, name, component_id, cardinality, visible, active, tile_id,
  widgets:[{node_alias, node_id, widget_id, widget_name, label, sortorder,
  value}], cards:[…]}` (cardinality-n → `instances:[…]`).
- **Already exposed in wasm:** `serializeRootCards` / `serializeCardDisplay`
  (instance_wrapper.rs:1142/1174), `cardsToTree` (batch_conversion.rs:175). The
  RDM cache is **borrowed** (`rcache = rdm_cache.inner()`), concepts resolve to
  label strings — one call, one boundary crossing.
- **Python already wraps it:** `serialize_card_display` /
  `serialize_root_cards_display`.

**Catalina does not use any of it.** Its TS `CardRenderer` reimplements the
traversal and walks the viewmodel Proxy per field — the entire source of the
FFI thrash. So the strategic fix is: **adopt the resolver Catalina bypassed.**

### The actual blocker: display-mode value flattening

A clean swap is blocked by one thing — core's **display mode flattens the
structured datatypes to strings**, discarding the fields Catalina's formatting
callbacks need:

| Datatype | Catalina needs (from `RENDERER_OPTIONS`) | Core display output | Gap |
|---|---|---|---|
| concept / domain | `() => null` (no URL) | label string | none |
| string/number/bool/date | display text | display value | none |
| **geojson** | `gfc.forJson()` (raw FC) | `tile_data.clone()` (raw FC) | **none** |
| **resource-instance** | `rr.getSlug()` (id/slug) | `Value::String(display)` (resources.rs:58) | **id/slug lost** |
| **file-list** | `url`, `isImage()`, `getAltText()` | display string | **file metadata lost** |

So only **two** datatypes (resource-instance, file-list) flatten away structure
the host needs. Everything else already round-trips.

### Compatibility constraint

`serialize_card`/`serialize_root_cards` are shared by Python and napi. Changing
the display value shape by default would ripple to those consumers. The fix must
be **additive** — a structured-display option (or an extra structured field
alongside the display string), off by default — so existing flat-string callers
are untouched.

## Revised rollout (given the resolver already exists)

1. **Core (the only real code):** add an additive structured-display option to
   `card_traversal`'s value resolution so `resource-instance` emits
   `{display, resourceId, slug?}` and `file-list` emits
   `{display, url, name, isImage, altText}` instead of a bare string. geojson
   already passes through raw. Off by default; opt-in via `SerializationOptions`.
   Unit-testable in Rust against a fixture graph+resource — no bindings or
   Catalina needed to verify.
2. **wasm:** thread the option through `serializeRootCards` (already exposed).
3. **Catalina:** replace `CardRenderer`'s `getRootViewModel()` Proxy walk +
   `buildCardTree` + `renderCard`/`renderCardInstance`/`assembleMarkdown` with a
   thin formatter over `serializeRootCards(...)` output. Keep the markdown
   assembly + URL/image callbacks, now fed from the structured values. This
   deletes the duplicated traversal **and** the per-field FFI thrash in one move.
4. Feature-flag; diff the new markdown against the current renderer on a sample
   of assets before switching (I can't run Catalina here, so this gate is yours).

Secondary cleanup (independent): `serialize_root_cards` clones the whole
`pseudo_cache` once per call (instance_wrapper.rs:1149) — pass it by reference
instead. And retire the Python `get_global_rdm_cache()` deep-clone in
`serialize_*_display` (use `with_global_rdm_cache`/`_arc`).

## Decisions needed

1. **i18n boundary.** Core resolves strings to `opts.language` + fallback and
   returns resolved text (default) — or return all localisations and let the
   host pick? Resolving in core is simpler and cuts payload. **Lean:
   resolve-in-core.**
2. **RenderTree granularity.** Typed `ResolvedValue` leaves (default) vs. core
   emitting pre-rendered display strings (rejected — presentation coupling).
   **Confirm typed.**
3. **Where CardSpec lives.** Cache on the loaded graph (invalidate on graph
   reload) vs. rebuild per render. Schema is stable → **cache on graph.**
4. **Scope of first cut.** Report/read path only (Catalina's case), or also
   model the **edit** direction (cards are also the form surface)? **Lean:
   read-only first**, design `ResolvedValue` so a write-back path can be added
   later, and do not block on it.

## First implementable slice

Verifiable in isolation, in alizarin, before any Catalina change:

**Add structured-display values for `resource-instance` and `file-list` in
`card_traversal`, behind an additive `SerializationOptions` flag.**

That is: extend `get_widget_value` (card_traversal.rs) so, under the new flag,
those two datatypes emit `{display, …structured}` instead of a bare string;
leave every other datatype (and the default flag-off behaviour) unchanged. Unit
tests assert the structured shape for a fixture resource with a reference, a
file, and a geojson field. No bindings, no Catalina, no compat break.

Only once that lands and is tested does the wasm option-threading (step 2) and
the Catalina swap (step 3) follow.

---

# Cross-check: how upstream `arches-modular-reports` consumes card/tile data

Audited `arches-modular-reports` (the official Arches report app) to see whether
our seam matches how upstream does it, and whether Catalina could converge on
its Vue components instead of carrying a bespoke renderer.

## Its architecture (server-resolved, config-driven, REST)

Fundamentally different from Catalina/alizarin — because it assumes a **live
Arches + Postgres**, whereas Catalina is a **static/serverless build** (business
data JSON + wasm). That deployment difference is *why* alizarin exists: it does
client-side, in wasm, what modular-reports does server-side, in SQL.

- **Values resolve in Postgres.** `nodegroup_tile_data_utils.py` annotates via
  `__arches_get_node_display_value_v2` / `__arches_get_valueid`; `get_link()`
  builds URLs server-side (concept→`rdm`, resource-instance→`resource_report`,
  url→passthrough, reference→`uri`).
- **Report layout is a stored `ReportConfig`** (custom sections) — *not* the
  Arches card tree. Cards are fetched per-nodegroup (`api_card_from_nodegroup_id`)
  mainly for the editor; node labels/widgets come via `fetchNodePresentation`.
- **REST, paginated per nodegroup.** `NodegroupTileDataView` returns
  `{results, total_count, page}` with server-side sort/filter/query — large
  nodegroups page in, never all-in-memory. (alizarin/wasm has no pagination
  equivalent — a scaling divergence to note, orthogonal to the perf bug.)

## Baked-in assumptions in the returned data

The Vue components are coupled to Arches shapes, not a neutral envelope. Three
layers (from `ModularReport/types.ts` + `ChildTileNodeValue.vue`):

1. **Structural — Arches v2 `aliased_data`.** `ResourceData { aliased_data:
   {alias → AliasedNodeData | AliasedNodegroupData}, … }` with `TileData
   { tileid, parenttile, nodegroup, resourceinstance, sortorder,
   provisionaledits, aliased_data }` nested recursively. `AliasedNodeData` /
   `AliasedNodegroupData` come from **arches-vue-components** (Arches core).

2. **Per-value — `{display_value, node_value, details[]}` with datatype-typed
   `details`.** `ChildTileNodeValue.vue` **duck-types on the fields present in
   `details`**:
   - resource-instance → `ResourceDetails { display_value, resource_id }`
   - concept → `ConceptDetails { concept_id, valueid, valuetype_id, language_id, value }`
   - reference → `ReferenceDetails { uri, list_item_id, display_value }`
   - url → `node_value.url`, `url_label`

   So the producer must emit, per node, the **resolved display string, the raw
   value, and a datatype-specific detail object** with exact fields.

3. **Ancillary lookups it presupposes.** A separate `NodePresentation` lookup
   keyed by nodeid (labels, `card_order`, `cardinality`, `number_format`,
   `is_rich_text`) supplies layout/labels — not the card tree. Plus
   `userIsRdmAdmin` (RDM edit via `valueid`), and live permission/RDM endpoints.

## The key nuance: the main report path is *typed*, not flattened

The client does **zero** resolution, but it is handed **typed** data (`raw +
display + details`) and switches on type — it is *not* handed a pre-flattened
string. (The `{display_values, links}` endpoint from `NodeTileDataView` is a
*secondary* summary path.)

**This validates the typed-`ResolvedValue` design over "flatten to a display
string."** Upstream Arches' primary renderer wants exactly the typed-leaf shape
we proposed, and it gives us a concrete target schema (the `*Details` objects).
Conclusion for the core work: **emit typed values, not flattened strings** — for
both resource-instance and file-list (the first slice), and ideally aligned to
the Arches `*Details` field names so the two paths can converge later.

## Two paths for Catalina

- **Converge on upstream components.** Build an `aliased_data` + typed-`details`
  output mode in alizarin → reuse the modular-report Vue components, delete
  Catalina's bespoke renderer. Bigger (needs the aliased_data structure + the
  `NodePresentation` lookup + per-datatype details), but aligns with Arches and
  future-proofs. The resolvers already exist (concept→`valueid` via RDM cache,
  reference→`uri`, resource→`id`) — only the output *shape* is missing.
- **Stay bespoke.** Keep Catalina's markdown renderer; just fix it via
  `serialize_root_cards` + structured values (the plan above). Smaller; Catalina
  keeps carrying its own report UI.

Not decided here. But either path requires emitting **typed** values, so the
first slice (structured `resource-instance` + `file-list`) is on the critical
path regardless — and naming its fields after the Arches `*Details` schema keeps
the convergence option open at no extra cost.

## Worked example: the schema alizarin must emit for modular reports

Grounded in the real `arches-vue-components` types: `AliasedNodeData =
{display_value, node_value, details[]}`, `AliasedNodegroupData = AliasedTileData
| AliasedTileData[] | null`, `AliasedTileData = {aliased_data, nodegroup,
parenttile, resourceinstance, sortorder, tileid, provisionaledits}`.

A heritage asset with a name, a monument-type concept, related assets, a
designation reference, an external URL, and a repeating "descriptions" nodegroup
that nests a "sources" child nodegroup:

```jsonc
{
  "resourceinstanceid": "b1f2…-asset-uuid",
  "name": "Old Mill, Bridge Street",
  "graph": "heritage-asset",
  "graph_publication": "pub-uuid",
  "descriptors": {
    "en": { "name": "Old Mill, Bridge Street", "description": "…", "map_popup": "…" }
  },

  "aliased_data": {

    // ── scalar string, cardinality-1 → a single AliasedNodeData ──
    "name": {
      "display_value": "Old Mill, Bridge Street",
      "node_value": { "en": { "value": "Old Mill, Bridge Street", "direction": "ltr" } },
      "details": []                              // scalars carry no details
    },

    // ── concept, cardinality-1 → ConceptDetails[] ──
    "monument_type": {
      "display_value": "Watermill",
      "node_value": "9b2c…-valueid",             // the raw tile value (a valueid)
      "details": [
        {
          "concept_id":   "3a7e…-concept-uuid",  // ← alizarin: RDM cache lookup
          "valueid":      "9b2c…-valueid",        // ← RDM cache (drives admin edit)
          "value":        "Watermill",            // ← resolved pref label
          "language_id":  "en",
          "valuetype_id": "prefLabel"
        }
      ]
    },

    // ── resource-instance-list → ResourceDetails[] ──
    "related_assets": {
      "display_value": "Mill Cottage; Weir House",
      "node_value": [
        { "resourceId": "aaaa…-uuid", "ontologyProperty": "", "resourceXresourceId": "…" },
        { "resourceId": "bbbb…-uuid", "ontologyProperty": "", "resourceXresourceId": "…" }
      ],
      "details": [
        { "resource_id": "aaaa…-uuid", "display_value": "Mill Cottage" },  // ← resource registry
        { "resource_id": "bbbb…-uuid", "display_value": "Weir House"    }
      ]
    },

    // ── reference (controlled list) → ReferenceDetails[] ──
    "designation": {
      "display_value": "Listed Building — Grade II",
      "node_value": [ { "uri": "https://…/item/…", "labels": [ … ], "list_id": "…" } ],
      "details": [
        {
          "uri":          "https://…/plugins/controlled-list-manager/item/xyz",  // ← CLM
          "list_item_id": "xyz-item-uuid",
          "display_value": "Listed Building — Grade II"
        }
      ]
    },

    // ── url → read off node_value, details empty ──
    "external_record": {
      "display_value": "Historic Environment Record",
      "node_value": { "url": "https://her.example/rec/123", "url_label": "Historic Environment Record" },
      "details": []
    },

    // ── cardinality-n nodegroup → AliasedTileData[] (NOT AliasedNodeData) ──
    "descriptions": [
      {
        "tileid": "tile-desc-1",
        "nodegroup": "descriptions-ng-uuid",
        "parenttile": null,
        "resourceinstance": "b1f2…-asset-uuid",
        "sortorder": 0,
        "provisionaledits": null,
        "aliased_data": {
          "description": {
            "display_value": "A three-storey watermill…",
            "node_value": { "en": { "value": "A three-storey watermill…", "direction": "ltr" } },
            "details": []
          },
          // ── nested child nodegroup, cardinality-n ──
          "sources": [
            {
              "tileid": "tile-src-1",
              "nodegroup": "sources-ng-uuid",
              "parenttile": "tile-desc-1",        // ← links to parent tile
              "resourceinstance": "b1f2…-asset-uuid",
              "sortorder": 0,
              "provisionaledits": null,
              "aliased_data": {
                "citation": {
                  "display_value": "RCAHMS 1985, p.42",
                  "node_value": { "en": { "value": "RCAHMS 1985, p.42", "direction": "ltr" } },
                  "details": []
                }
              }
            }
          ]
        }
      }
    ]
  }
}
```

### Rules the example encodes

1. **Two node shapes, and the distinction is load-bearing.**
   - A **leaf node** value = `AliasedNodeData` = `{display_value, node_value, details[]}`.
   - A **nodegroup** value = `AliasedNodegroupData` = `AliasedTileData |
     AliasedTileData[] | null` — one object for cardinality-1, an **array** for
     cardinality-n. Each `AliasedTileData` carries the tile envelope
     (`tileid/nodegroup/parenttile/resourceinstance/sortorder/provisionaledits`)
     and recurses via its own `aliased_data`.

2. **`details` is the typed part the renderer duck-types on** — and it's exactly
   where alizarin does real work:

   | datatype | `details[]` element | alizarin source |
   |---|---|---|
   | concept | `{concept_id, valueid, value, language_id, valuetype_id}` | RDM cache (already borrowed in `serialize_concept`) |
   | resource-instance | `{resource_id, display_value}` | resource registry / `ResourceDisplayResolver` |
   | reference | `{uri, list_item_id, display_value}` | CLM (`build_item_uri`, labels) |
   | url | *(empty — read from `node_value.url/url_label`)* | passthrough |
   | scalar / geojson / string | `[]` | none |

3. **`node_value` is the raw tile value, unchanged** — alizarin already has it in
   `tile_data`. `display_value` is the resolved localized string — alizarin
   already produces it in display mode. The **only** genuinely new work is the
   typed `details[]` (reusing resolvers it already has) plus emitting the
   `AliasedTileData` envelope instead of the flat card-tree shape.

### The gap, precisely

`serialize_root_cards` today emits `{card_id, widgets:[{node_alias, label,
value}]}` with a **flattened** `value`. To feed modular reports it must instead
emit **aliased_data** (nodegroup-nested, tile-enveloped) where each leaf is
`{display_value, node_value, details[]}`. It already has `node_value` (raw) and
`display_value` (resolved); the additions are the per-datatype `details` (same
"structured value" first slice, shaped to the Arches `*Details` names) and the
`AliasedTileData` nesting rather than a card list.
