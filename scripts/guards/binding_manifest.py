#!/usr/bin/env python3
"""
Build tests/binding-parity.json from the bindings' runtime surfaces
(PARITY-RESEARCH.md, A-5..A-7).

Keeps every class entry already in the manifest (hand-curated, with reasons)
and adds: every other exported class, grouped with its counterparts in the
other bindings (CLASS_GROUPS), methods matched by snake_case name or the
renames in METHOD_RENAMES; and every free function under its canonical name
(snake_case; FUNCTION_RENAMES groups the ones spelt differently per binding).
Where a binding lacks something, the reason comes from REASONS, else the
manifest's existing one, else "gap: unclassified" (to be given a rule here).
Classes outside CURATED are rebuilt on every run, so edit them here, not in
the JSON.

Usage (run with the repo's .venv, so `alizarin` imports):
  node scripts/guards/binding_surface.mjs > /tmp/js.json
  .venv/bin/python scripts/guards/binding_manifest.py /tmp/js.json > /tmp/manifest.json
Then review the diff against tests/binding-parity.json before replacing it.
"""
import fnmatch
import inspect
import json
import pathlib
import re
import sys

import alizarin.alizarin as py

ROOT = pathlib.Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "tests" / "binding-parity.json"
UNCLASSIFIED = "gap: unclassified (PARITY-RESEARCH.md A-6/A-7)"
BINDINGS = ("wasm", "napi", "python")

# Counterpart classes across bindings: key -> (wasm, napi, python); None = absent.
CLASS_GROUPS = {
    "nodeConfigManager": ("WasmNodeConfigManager", "NapiNodeConfigManager", "RustNodeConfigManager"),
    "nodeConfigBoolean": ("WasmNodeConfigBoolean", "NapiNodeConfigBoolean", "RustNodeConfigBoolean"),
    "nodeConfigConcept": ("WasmNodeConfigConcept", "NapiNodeConfigConcept", "RustNodeConfigConcept"),
    "nodeConfigDomain": ("WasmNodeConfigDomain", "NapiNodeConfigDomain", "RustNodeConfigDomain"),
    "rdmCache": ("WasmRdmCache", "NapiRdmCache", "RustRdmCache"),
    "staticDomainValue": ("WasmStaticDomainValue", "NapiStaticDomainValue", "RustStaticDomainValue"),
    "populateResult": ("WasmPopulateResult", "NapiPopulateResult", "RustPopulateResult"),
    "resourceRegistry": ("StaticResourceRegistry", "NapiStaticResourceRegistry", "ResourceRegistry"),
    "pseudoNode": ("PseudoNode", "NapiPseudoNode", "PseudoNode"),
    "staticGraph": ("StaticGraph", "NapiStaticGraph", None),
    "coercionResult": ("WasmCoercionResult", None, "CoercionResult"),
}

# Same method, different spelling: class key -> [(canonical, {binding: name})].
METHOD_RENAMES = {
    "populateResult": [
        ("get_value_aliases", {"wasm": "getValueAliases", "napi": "getValueAliases", "python": "aliases"}),
        ("get_all_values", {"wasm": "getAllValues", "napi": "getAllValues", "python": "get_values"}),
        ("all_nodegroups_map", {"wasm": "allNodegroupsMap", "napi": "allNodegroupsMap", "python": "get_nodegroups_map"}),
    ],
    "nodeConfigManager": [
        ("from_graph", {"wasm": "fromGraph", "napi": "buildFromGraph", "python": None}),
        ("from_graph_json", {"wasm": "fromGraphJson", "napi": "buildFromGraphJson", "python": "load_graph_json"}),
    ],
    "rdmCache": [
        ("add_collections_from_skos_xml", {"wasm": "addCollectionsFromSkosXml", "python": "add_from_skos_xml"}),
        ("length", {"wasm": "length", "napi": "collectionCount"}),
    ],
}

# Classes kept exactly as written in the manifest (hand-curated, with their own
# renames); every other class is rebuilt from the surfaces on each run.
CURATED = {"instanceWrapper", "modelWrapper", "pseudoList", "pseudoValue", "staticTile"}

# Extra keys for rebuilt classes. stringLike: js/_wasm.ts copies String.prototype
# onto the WASM class, so the TS test ignores those names.
CLASS_FLAGS = {"staticTranslatableString": {"stringLike": True}}

# Same function, different spelling: canonical -> {binding: name}. Canonical is
# core's name where core has one, else the majority spelling.
FUNCTION_RENAMES = {
    "tiles_to_tree": {"wasm": "tilesToTree", "python": "tiles_to_json_tree"},
    "tree_to_tiles": {"wasm": "treeToTiles", "python": "json_tree_to_tiles"},
    "cards_to_tree": {"wasm": "cardsToTree", "python": "cards_to_json_tree"},
    "coerce_geojson": {"wasm": "coerceGeoJson", "python": "coerce_geojson"},
    "collection_to_skos_xml": {"wasm": "collectionToSkosXml", "napi": "collectionToSkosXml", "python": "skos_collection_to_xml"},
    "collections_to_skos_xml": {"wasm": "collectionsToSkosXml", "napi": "collectionsToSkosXml", "python": "skos_collections_to_xml"},
    "find_needed_collections": {"wasm": "findNeededCollections", "napi": "findNeededCollections", "python": "get_needed_collections"},
    "build_graph_from_model_csvs": {"wasm": "buildGraphFromModelCsvs", "napi": "buildGraphFromCsvs", "python": "build_graph_from_model_csvs"},
    "build_resources_from_business_csv": {"wasm": "buildResourcesFromBusinessCsv", "napi": "buildBusinessDataFromCsv", "python": "build_resources_from_business_csv"},
    "register_extension_handler": {"wasm": "registerExtensionHandler", "napi": "registerExtensionHandler", "python": "register_type_handler"},
    "has_extension_handler": {"wasm": "hasExtensionHandler", "napi": "hasExtensionHandler", "python": "has_type_handler"},
    "coerce_with_extension": {"napi": "extensionCoerce", "python": "coerce_with_extension"},
    "render_display_with_extension": {"napi": "extensionRenderDisplay", "python": "render_display_with_extension"},
    "add_collection_to_global_rdm_cache": {"wasm": "addCollectionToGlobalRdmCache", "python": "add_collection_to_global_cache"},
    "add_to_global_rdm_cache_from_skos_xml": {"wasm": "addToGlobalRdmCacheFromSkosXml", "python": "add_from_skos_xml_to_global_cache"},
    "get_registered_extension_mutations": {"wasm": "getRegisteredExtensionMutations", "python": "list_extension_mutations"},
    "global_rdm_lookup_label": {"wasm": "globalRdmLookupLabel", "python": "global_rdm_concept_pref_label"},
}

ECO_FS = "ecosystem: reads or writes the filesystem, which WASM has not"
GAP_FS = "gap: Node has a filesystem; NAPI lacks this file-based form"
ECO_PLAIN_JSON = "ecosystem: core's JSON as plain objects, built by js/backend.ts createStatic* (constructStatic)"
ECO_DATACLASS = "ecosystem: Python dataclasses in python/alizarin/static_types.py, read and written through core"
GAP_RDM_GLOBAL = "gap: the global RDM cache functions differ per binding (A-6)"
GAP_NAMESPACE = "gap: H-3, RDM namespaces and deterministic UUIDs must agree across bindings"
GAP_EXTENSIONS = "gap: the extension registry functions differ per binding (A-6, G)"
GAP_TREES = "gap: B-9 follow-up, trees to tiles and resource merging are core functions not yet bound on NAPI (they need its extension registry)"

# Why a binding lacks something (PARITY-RESEARCH.md A-6/A-7). Keys are
# "fn:<canonical>", "class:<key>" or "op:<class key>.<canonical>", with fnmatch
# wildcards; the first rule with a reason for the binding wins.
REASONS = [
    # Runtime and language differences.
    ("fn:init_sync", {"napi": "ecosystem: wasm-bindgen bootstrap; native modules initialise on import",
                      "python": "ecosystem: wasm-bindgen bootstrap; native modules initialise on import"}),
    ("fn:main", {"napi": "ecosystem: wasm-bindgen start function", "python": "ecosystem: wasm-bindgen start function"}),
    ("fn:*rscv_timings", {"napi": "ecosystem: WASM-build timing instrumentation (js/tracing)",
                          "python": "ecosystem: WASM-build timing instrumentation (js/tracing)"}),
    ("class:wasmSpan", {"napi": "ecosystem: WASM-build timing instrumentation (js/tracing)",
                        "python": "ecosystem: WASM-build timing instrumentation (js/tracing)"}),
    ("fn:new_wasmresource_instance_wrapper_for_*", {
        "napi": "ecosystem: WASM factory used by js/backend.ts; NAPI constructs NapiResourceInstanceWrapper directly",
        "python": "ecosystem: WASM factory used by js/backend.ts; Python constructs its wrapper class directly"}),
    ("class:pyPseudoListIterator", {"wasm": "ecosystem: Python iterator protocol object", "napi": "ecosystem: Python iterator protocol object"}),
    ("class:treeToTilesIterator", {"wasm": "ecosystem: Python iterator protocol object", "napi": "ecosystem: Python iterator protocol object"}),
    ("class:napiTileData", {"wasm": "ecosystem: NAPI's live view of StaticTile.data; WASM returns the map itself",
                            "python": "ecosystem: NAPI's live view of StaticTile.data; Python returns the dict itself"}),
    ("op:pseudoNode.to_json", {"wasm": "ecosystem: spelt toJSON, the JSON.stringify protocol (filtered as plumbing)",
                               "napi": "ecosystem: spelt toJSON, the JSON.stringify protocol (filtered as plumbing)"}),
    ("op:rdmCache.length", {"python": "ecosystem: len(cache)"}),
    ("op:nodeConfigManager.length", {"python": "ecosystem: len(manager)"}),
    ("fn:*_to_dir", {"wasm": ECO_FS, "napi": GAP_FS}),
    ("fn:streamed_merge_from_files", {"wasm": ECO_FS, "napi": GAP_FS}),
    ("fn:export_prebuild", {"wasm": ECO_FS + " (build_prebuild_export returns it instead)", "napi": GAP_FS + " (NapiPrebuildExporter)"}),
    ("fn:import_prebuild", {"wasm": ECO_FS}),
    ("fn:apply_aggregate_functions_from_file", {"wasm": ECO_FS}),
    # Static types: core classes on WASM, plain JSON on NAPI, dataclasses in Python.
    ("class:static*", {"napi": ECO_PLAIN_JSON, "python": ECO_DATACLASS}),
    ("class:wKRM", {"napi": ECO_PLAIN_JSON, "python": ECO_DATACLASS}),
    ("op:staticGraph.*", {"wasm": "gap: NapiStaticGraph and StaticGraph expose different fields and builders (A-7)",
                          "napi": "gap: NapiStaticGraph and StaticGraph expose different fields and builders (A-7)"}),
    ("fn:canonical_static", {"wasm": "ecosystem: WASM's static classes already serialise through core",
                             "napi": "ecosystem: NAPI's static types are core's JSON already, built by constructStatic"}),
    ("fn:construct_static", {"wasm": "ecosystem: WASM's static classes' constructors (new StaticNode(...)) build through core"}),
    ("fn:resources_from_business_data_json",
     {"wasm": "ecosystem: a static method on WASM's resource class, StaticResource.fromBusinessDataJsonString"}),
    ("fn:summary_to_metadata", {"wasm": "ecosystem: a method on WASM's summary class, StaticResourceSummary.toMetadata"}),
    ("fn:static_resource_from_summary", {"wasm": "ecosystem: a static method on WASM's resource class, StaticResource.fromSummary"}),
    ("class:rustRdmCollection", {"wasm": "ecosystem: JS gets RDM collections as plain JSON", "napi": "ecosystem: JS gets RDM collections as plain JSON"}),
    ("class:rustRdmConcept", {"wasm": "ecosystem: JS gets RDM concepts as plain JSON", "napi": "ecosystem: JS gets RDM concepts as plain JSON"}),
    # To remove rather than add elsewhere.
    ("class:example*Wrapper", {"napi": "gap: macro demo (crates/alizarin-wasm/src/wasm_wrappers.rs) leaked into the WASM API; remove it",
                               "python": "gap: macro demo (crates/alizarin-wasm/src/wasm_wrappers.rs) leaked into the WASM API; remove it"}),
    ("class:wasmNodegroupResult", {"napi": "gap: exported but never constructed (crates/alizarin-wasm/src/pseudo_value.rs); remove it",
                                   "python": "gap: exported but never constructed (crates/alizarin-wasm/src/pseudo_value.rs); remove it"}),
    # Function families bound unevenly.
    ("fn:coerce_with_extension", {"wasm": GAP_EXTENSIONS}),
    ("fn:coerce_*", {"napi": "gap: NAPI has no standalone coercion functions; its ORM coerces inside the instance wrapper (A-6)"}),
    ("class:coercionResult", {"napi": "gap: NAPI has no standalone coercion functions (A-6)"}),
    ("fn:*rdm_namespace*", {"wasm": GAP_NAMESPACE, "napi": GAP_NAMESPACE, "python": GAP_NAMESPACE}),
    ("fn:generate_*uuid*", {"wasm": GAP_NAMESPACE, "napi": GAP_NAMESPACE, "python": GAP_NAMESPACE}),
    ("fn:labels_to_deterministic_string", {"napi": GAP_NAMESPACE, "python": GAP_NAMESPACE}),
    ("fn:*global_rdm*", {"wasm": GAP_RDM_GLOBAL, "napi": GAP_RDM_GLOBAL, "python": GAP_RDM_GLOBAL}),
    ("fn:*global_cache*", {"wasm": GAP_RDM_GLOBAL, "napi": GAP_RDM_GLOBAL, "python": GAP_RDM_GLOBAL}),
    ("fn:*extension*", {"wasm": GAP_EXTENSIONS, "napi": GAP_EXTENSIONS, "python": GAP_EXTENSIONS}),
    ("fn:has_display_renderer", {"wasm": GAP_EXTENSIONS, "napi": GAP_EXTENSIONS}),
    ("fn:*widget*", {"wasm": "gap: core's widget registry (registry.rs) is bound only in Python",
                     "napi": "gap: core's widget registry (registry.rs) is bound only in Python"}),
    ("fn:*datatype*", {"wasm": "gap: core's datatype registry (registry.rs) is bound only in Python",
                       "napi": "gap: core's datatype registry (registry.rs) is bound only in Python",
                       "python": "gap: label-resolution defaults not bound in Python (A-6)"}),
    ("fn:build_tree_from_tiles", {"wasm": "gap: bare tiles to a tree (core's, against its graph registry) is bound only in Python",
                                  "napi": "gap: bare tiles to a tree (core's, against its graph registry) is bound only in Python"}),
    ("fn:resolve_labels_in_tree", {"wasm": "gap: resolving a tree's labels before conversion (core's) is bound only in Python",
                                   "napi": "gap: resolving a tree's labels before conversion (core's) is bound only in Python"}),
    ("fn:*_tree*", {"wasm": GAP_TREES, "napi": GAP_TREES, "python": GAP_TREES}),
    ("fn:*tiles*", {"wasm": GAP_TREES, "napi": GAP_TREES, "python": GAP_TREES}),
    ("fn:*merge_resources", {"napi": GAP_TREES}),
    ("fn:matches_semantic_child", {"wasm": "gap: core's semantic-child match is bound only in Python",
                                   "napi": "gap: core's semantic-child match is bound only in Python"}),
    ("class:rustSemanticChildResult", {"wasm": "gap: core's semantic-child match is bound only in Python",
                                       "napi": "gap: core's semantic-child match is bound only in Python"}),
    ("fn:resolve_labels*", {"wasm": "gap: label resolution is bound with a different signature per binding (A-6)",
                            "napi": "gap: label resolution is bound with a different signature per binding (A-6)",
                            "python": "gap: label resolution is bound with a different signature per binding (A-6)"}),
    ("fn:build_alias_to_collection_map", {"python": "gap: label-resolution helpers not bound in Python (A-6)"}),
    ("fn:get_default_config_keys", {"python": "gap: label-resolution defaults not bound in Python (A-6)"}),
    ("fn:*graph*", {"wasm": "gap: graph registry, builders and exports are bound unevenly (A-6)",
                    "napi": "gap: graph registry, builders and exports are bound unevenly (A-6); js/csvModelLoader.ts falls back to WASM",
                    "python": "gap: graph registry, builders and exports are bound unevenly (A-6)"}),
    ("fn:set_descriptor_template", {"wasm": "gap: graph registry, builders and exports are bound unevenly (A-6)",
                                    "napi": "gap: graph registry, builders and exports are bound unevenly (A-6)"}),
    ("fn:validate_model_csvs", {"napi": "gap: missing on NAPI; js/csvModelLoader.ts validates through WASM even under NAPI"}),
    ("fn:build_prebuild_export", {"napi": "gap: NAPI exposes this as NapiPrebuildExporter", "python": "gap: Python only exports to a directory (export_prebuild)"}),
    ("class:napiPrebuildExporter", {"wasm": "gap: WASM exposes this as build_prebuild_export",
                                    "python": "gap: Python exposes this as export_prebuild"}),
    ("class:ontologyValidator", {"wasm": "gap: core's OntologyValidator is bound only in Python",
                                 "napi": "gap: core's OntologyValidator is bound only in Python"}),
    # Methods per class.
    ("op:coercionResult.warnings", {"python": "gap: Python's CoercionResult has no warnings (A-7)"}),
    ("op:nodeConfigManager.from_graph", {"python": "gap: Python builds only from graph JSON (load_graph_json) (A-7)"}),
    ("op:populateResult.all_values_map", {"python": "gap: not bound in Python (A-7)"}),
    ("op:staticDomainValue.display", {"python": "gap: not bound in Python (A-7)"}),
    ("op:staticDomainValue.to_dict", {"wasm": "gap: JS StaticDomainValue has no toJSON, so no core serialisation (A-7)",
                                      "napi": "gap: JS StaticDomainValue has no toJSON, so no core serialisation (A-7)"}),
    ("op:pseudoNode.*", {"python": "gap: Python's PseudoNode exposes the other node fields only through `node` (A-7)"}),
    ("op:rdmCache.*", {"wasm": "gap: RdmCache's methods differ per binding (A-7)", "napi": "gap: RdmCache's methods differ per binding (A-7)",
                       "python": "gap: RdmCache's methods differ per binding (A-7)"}),
    ("op:resourceRegistry.*", {"wasm": "gap: the resource registry's methods differ per binding (A-7)",
                               "napi": "gap: the resource registry's methods differ per binding (A-7)",
                               "python": "gap: the resource registry's methods differ per binding (A-7)"}),
]


def reason(key, binding, fallback=UNCLASSIFIED):
    for pattern, per in REASONS:
        if fnmatch.fnmatchcase(key, pattern) and binding in per:
            return per[binding]
    return fallback


def snake(name):
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name).lower()


def python_surface():
    classes, functions = {}, []
    for name in dir(py):
        if name.startswith("_"):
            continue
        obj = getattr(py, name)
        if inspect.isclass(obj):
            if not issubclass(obj, BaseException):
                classes[name] = sorted(m for m in dir(obj) if not m.startswith("_"))
        elif callable(obj):
            functions.append(name)
    return {"classes": classes, "functions": sorted(functions)}


def build_operations(key, names, surfaces):
    """Operations for a class group, matched by snake name, then renames."""
    methods = {b: (surfaces[b]["classes"].get(names[b], []) if names[b] else None) for b in BINDINGS}
    taken = {b: set() for b in BINDINGS}
    ops = []
    for canonical, per in METHOD_RENAMES.get(key, []):
        op = {b: (per.get(b) if methods[b] is not None and per.get(b) in (methods[b] or []) else None) for b in BINDINGS}
        for b in BINDINGS:
            if op[b]:
                taken[b].add(op[b])
        ops.append((canonical, op))
    by_snake = {}
    for b in BINDINGS:
        for m in methods[b] or []:
            if m not in taken[b]:
                by_snake.setdefault(snake(m), {})[b] = m
    for canonical, per in by_snake.items():
        ops.append((canonical, {b: per.get(b) for b in BINDINGS}))
    out = []
    for canonical, op in sorted(ops, key=lambda x: x[0]):
        entry = dict(op)
        missing = [b for b in BINDINGS if op[b] is None and names[b] is not None]
        if missing:
            entry["why"] = {b: reason(f"op:{key}.{canonical}", b) for b in missing}
        out.append(entry)
    return out


def main():
    js = json.loads(pathlib.Path(sys.argv[1]).read_text())
    surfaces = {"wasm": js["wasm"], "napi": js["napi"], "python": python_surface()}
    manifest = json.loads(MANIFEST.read_text())
    classes = {key: spec for key, spec in manifest["classes"].items() if key in CURATED}

    # Every class some binding exports, grouped; anything ungrouped stands alone.
    grouped = {(b, n) for spec in classes.values() for b in BINDINGS if (n := spec.get(b))}
    for key, names in CLASS_GROUPS.items():
        names = dict(zip(BINDINGS, names))
        grouped |= {(b, n) for b, n in names.items() if n}
        spec = {**names, "why": {b: reason(f"class:{key}", b) for b in BINDINGS if names[b] is None} or None}
        spec["operations"] = build_operations(key, names, surfaces)
        classes[key] = spec
    for b in BINDINGS:
        for cls in sorted(surfaces[b]["classes"]):
            if (b, cls) in grouped:
                continue
            key = cls[0].lower() + cls[1:]
            while key in classes:
                key += "_" + b
            names = {x: (cls if x == b else None) for x in BINDINGS}
            classes[key] = {
                **names,
                "why": {x: reason(f"class:{key}", x) for x in BINDINGS if x != b},
                **CLASS_FLAGS.get(key, {}),
                "operations": build_operations(key, names, surfaces),
            }

    # Free functions under canonical names.
    renamed = {(b, n): canonical for canonical, per in FUNCTION_RENAMES.items() for b, n in per.items()}
    functions = {}
    for b in BINDINGS:
        for fn in surfaces[b]["functions"]:
            canonical = renamed.get((b, fn), snake(fn))
            functions.setdefault(canonical, {x: None for x in BINDINGS})[b] = fn
    previous = manifest.get("functions", {})
    for canonical, entry in functions.items():
        old_why = previous.get(canonical, {}).get("why", {})
        missing = {b: reason(f"fn:{canonical}", b, old_why.get(b, UNCLASSIFIED)) for b in BINDINGS if entry[b] is None}
        if missing:
            entry["why"] = missing

    manifest["classes"] = dict(sorted(classes.items()))
    manifest["functions"] = dict(sorted(functions.items()))
    manifest["_doc"] = (
        "Every public class, method and free function of each binding, by binding, under one canonical name "
        "(snake_case; spelt camelCase in TS). A null name must be explained in `why`: 'ecosystem: ...' for a "
        "deliberate difference, 'gap: ...' for one still to fix ('gap: unclassified' awaits a decision). "
        "A class a binding lacks is null, explained once in the class's `why`. "
        "tests/dual-backend/bindingParity.test.ts (WASM, NAPI) and crates/alizarin-python/tests/test_binding_parity.py "
        "check each binding exports exactly what is listed here, so anything added to one binding only fails until "
        "it is listed. Rebuild with scripts/guards/binding_manifest.py."
    )
    print(json.dumps(manifest, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
