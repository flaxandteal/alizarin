#!/usr/bin/env python3
"""
Build tests/binding-parity.json from the bindings' runtime surfaces
(PARITY-RESEARCH.md, A-5..A-7).

Keeps every class entry already in the manifest (hand-curated, with reasons)
and adds: every other exported class, grouped with its counterparts in the
other bindings (CLASS_GROUPS), methods matched by snake_case name or the
renames in METHOD_RENAMES; and every free function under its canonical name
(snake_case; FUNCTION_RENAMES groups the ones spelt differently per binding).
Where a binding lacks something and the manifest has no reason yet, the
reason is "gap: unclassified", to be replaced by a real ecosystem:/gap: one.

Usage (run with the repo's .venv, so `alizarin` imports):
  node scripts/guards/binding_surface.mjs > /tmp/js.json
  .venv/bin/python scripts/guards/binding_manifest.py /tmp/js.json > /tmp/manifest.json
Then review the diff against tests/binding-parity.json before replacing it.
"""
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
        ("get_value_aliases", {"wasm": "aliases", "napi": "aliases", "python": "get_value_aliases"}),
        ("get_all_values", {"wasm": "getValues", "napi": "getValues", "python": "get_all_values"}),
        ("all_nodegroups_map", {"wasm": "getNodegroupsMap", "napi": "getNodegroupsMap", "python": "all_nodegroups_map"}),
    ],
    "staticDomainValue": [
        ("to_json", {"wasm": "toJson", "napi": "toJson", "python": "to_dict"}),
    ],
    "nodeConfigManager": [
        ("from_graph", {"wasm": "fromGraph", "napi": "buildFromGraph", "python": None}),
        ("from_graph_json", {"wasm": "fromGraphJson", "napi": "buildFromGraphJson", "python": None}),
    ],
}

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
}


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
            entry["why"] = {b: UNCLASSIFIED for b in missing}
        out.append(entry)
    return out


def main():
    js = json.loads(pathlib.Path(sys.argv[1]).read_text())
    surfaces = {"wasm": js["wasm"], "napi": js["napi"], "python": python_surface()}
    manifest = json.loads(MANIFEST.read_text())
    classes = manifest["classes"]

    # Every class some binding exports, grouped; anything ungrouped stands alone.
    grouped = {(b, n) for spec in classes.values() for b in BINDINGS if (n := spec.get(b))}
    for key, names in CLASS_GROUPS.items():
        names = dict(zip(BINDINGS, names))
        grouped |= {(b, n) for b, n in names.items() if n}
        if key in classes:
            continue
        spec = {**names, "why": {b: UNCLASSIFIED for b in BINDINGS if names[b] is None} or None}
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
                "why": {x: UNCLASSIFIED for x in BINDINGS if x != b},
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
        missing = {b: old_why.get(b, UNCLASSIFIED) for b in BINDINGS if entry[b] is None}
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
