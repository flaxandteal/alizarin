"""
Python's static types round-trip as core does (PARITY-RESEARCH.md, B-5).

For every graph and business-data resource in the repo's test fixtures,
core's parse-and-serialise (canonical_static) and Python's
StaticX.from_dict(raw).to_dict() must agree. to_dict already goes through
core, so a difference is something Python's from_dict dropped, added or
changed. Today's differences are listed, with their finding ids, under
"coreRoundTrip" in tests/guard-allowlist.json; the test fails on a new one
and on a listed one that has gone (delete its entry).
"""

import json
import pathlib

from alizarin.alizarin import canonical_static
from alizarin.static_types import StaticGraph, StaticResource

ROOT = pathlib.Path(__file__).resolve().parents[3]
ALLOWLIST = ROOT / "tests" / "guard-allowlist.json"


def _diff(core, mine, path=""):
    if type(core) is not type(mine):
        yield path, f"{type(core).__name__} vs {type(mine).__name__}"
    elif isinstance(core, dict):
        for key in sorted(set(core) | set(mine)):
            if key not in mine:
                yield f"{path}.{key}", "dropped by Python"
            elif key not in core:
                yield f"{path}.{key}", "added by Python"
            else:
                yield from _diff(core[key], mine[key], f"{path}.{key}")
    elif isinstance(core, list):
        if len(core) != len(mine):
            yield path, f"length {len(core)} vs {len(mine)}"
        else:
            for a, b in zip(core, mine):
                yield from _diff(a, b, f"{path}[]")
    elif core != mine:
        yield path, f"{core!r} vs {mine!r}"


def _objects():
    for path in sorted((ROOT / "tests").rglob("*.json")):
        if "node_modules" in path.parts:
            continue
        try:
            data = json.loads(path.read_text())
        except (ValueError, UnicodeDecodeError):
            continue
        if not isinstance(data, dict):
            continue
        rel = path.relative_to(ROOT).as_posix()
        for graph in data.get("graph") or []:
            if isinstance(graph, dict) and "nodes" in graph:
                yield rel, "graph", StaticGraph, graph
        business_data = data.get("business_data")
        if isinstance(business_data, dict):
            for resource in business_data.get("resources") or []:
                yield rel, "resource", StaticResource, resource


def current_differences():
    found = set()
    for rel, kind, cls, raw in _objects():
        try:
            core = canonical_static(kind, raw)
        except ValueError:
            continue  # core rejects it: not a round-trip question
        try:
            mine = cls.from_dict(raw).to_dict()
        except Exception as e:  # noqa: BLE001 - any failure is the finding
            found.add(f"{rel}::{kind}::Python rejects::{type(e).__name__}")
            continue
        for path, what in _diff(core, mine):
            found.add(f"{rel}::{kind}{path}::{what}")
    return found


def test_python_static_types_round_trip_as_core_does():
    allowed = set(json.loads(ALLOWLIST.read_text()).get("coreRoundTrip", {}))
    found = current_differences()
    new = sorted(found - allowed)
    gone = sorted(allowed - found)
    assert not new and not gone, (
        "\n".join([f"NEW   {k}" for k in new] + [f"FIXED {k}  (remove it from tests/guard-allowlist.json)" for k in gone])
    )
