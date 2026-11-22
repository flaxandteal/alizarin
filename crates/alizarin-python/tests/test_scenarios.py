"""
Scenarios: sequences of ORM steps (read, assign, commit), as data in
tests/data/scenarios/*.json, each checked as it runs. TypeScript plays the
same files on WASM and NAPI in tests/dual-backend/scenarios.test.ts, so the
ORMs agree step by step. The vocabulary is in tests/data/scenarios/README.md.
"""

import inspect
import json
import os

import pytest

import alizarin
from alizarin.graph_manager import GraphManager
from alizarin.view_models import viewContext

DATA = os.path.join(os.path.dirname(__file__), "../../../tests/data")
SCENARIOS = os.path.join(DATA, "scenarios")
FILES = sorted(f for f in os.listdir(SCENARIOS) if f.endswith(".json"))


def load(*parts):
    with open(os.path.join(DATA, *parts), encoding="utf-8") as f:
        return json.load(f)


def plain(value):
    """As JSON would write it."""
    if hasattr(value, "to_dict"):
        value = value.to_dict()
    return json.loads(json.dumps(value, default=str))


async def resolved(value):
    return await value if inspect.isawaitable(value) else value


async def open_fixture(fixture):
    """A fixture's resource, opened through the ORM, and its nodes by alias."""
    if os.path.exists(os.path.join(DATA, fixture, "collection.json")):
        collection = load(fixture, "collection.json")
        cache = alizarin.RustRdmCache()
        cache.add_collection_from_json(collection["id"], json.dumps(list(collection["concepts"].values())))
        alizarin.set_global_rdm_cache(cache)
    graph = load(fixture, "graph.json")
    gm = GraphManager()
    gm.register_graph(graph)
    data = load(fixture, "resource.json")
    gm.register_resource(data)
    viewContext.graphManager = gm
    resource = await gm.getResource(data["resourceinstance"]["resourceinstanceid"], lazy=False)
    return resource, {node["alias"]: node["nodeid"] for node in graph["nodes"]}


def upserted_by_alias(upserted, node_ids, aliases):
    """Each upserted tile's value for an alias, as JSON."""
    tiles = [plain(tile) for tile in upserted]
    out = {}
    for alias in aliases:
        tile = next((t for t in tiles if node_ids[alias] in (t.get("data") or {})), None)
        out[alias] = tile["data"][node_ids[alias]] if tile else "(no upserted tile)"
    return out


async def _for_json(value):
    return plain(await resolved(value.for_json()))


async def _str(value):
    return str(value)


# A read's checks, by the key a step names them with.
READS = {"forJson": _for_json, "str": _str}


async def read(step, resource, node_ids, at):
    value = await getattr(resource, step["read"])
    checks = [key for key in READS if key in step]
    actual = {key: await READS[key](value) for key in checks}
    assert actual == {key: step[key] for key in checks}, at


async def pending(step, resource, node_ids, at):
    # Assigned and not yet committed: the value as given.
    assert plain(await getattr(resource, step["pending"])) == step["value"], at


async def assign(step, resource, node_ids, at):
    setattr(resource, step["assign"], step["value"])


async def assign_fails(step, resource, node_ids, at):
    with pytest.raises(Exception, match=step["error"]):
        setattr(resource, step["assign_fails"], step["value"])


async def commit(step, resource, node_ids, at):
    delta = resource.commit()
    aliases = list(step["commit"]["upserted"])
    assert upserted_by_alias(delta["upserted"], node_ids, aliases) == step["commit"]["upserted"], at
    assert len(delta["upserted"]) == len(aliases), f"{at} (upserted tiles)"
    assert list(delta["removed"]) == step["commit"]["removed"], at


async def commit_fails(step, resource, node_ids, at):
    with pytest.raises(Exception, match=step["commit_fails"]):
        resource.commit()


async def describe(step, resource, node_ids, at):
    value = await getattr(resource, step["describe"])
    actual = {"field": value.describeField(), "group": value.describeFieldGroup()}
    assert actual == {"field": step["field"], "group": step["group"]}, at


# One handler per step, keyed by the step's kind (its first key).
STEPS = {f.__name__: f for f in (read, pending, assign, assign_fails, commit, commit_fails, describe)}


@pytest.mark.asyncio
@pytest.mark.parametrize("file", FILES)
async def test_scenario(file):
    scenario = load("scenarios", file)
    resource, node_ids = await open_fixture(scenario["fixture"])
    try:
        for i, step in enumerate(scenario["steps"]):
            at = f"{file} step {i}: {json.dumps(step, ensure_ascii=False)}"
            kind = next(iter(step))
            assert kind in STEPS, f"{at}: a known step"
            await STEPS[kind](step, resource, node_ids, at)
    finally:
        viewContext.graphManager = None
        alizarin.clear_global_rdm_cache()
