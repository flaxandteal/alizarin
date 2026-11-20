"""
Binding parity: the native module exports exactly the classes and free
functions tests/binding-parity.json lists for Python, and each class has
exactly the listed methods, so anything added to one binding only fails
until it is listed (with its reason, if another binding lacks it). The
TypeScript side checks WASM and NAPI against the same manifest in
tests/dual-backend/bindingParity.test.ts.
"""

import inspect
import json
import os

import pytest

import alizarin.alizarin as rust

MANIFEST = os.path.join(os.path.dirname(__file__), "../../../tests/binding-parity.json")

with open(MANIFEST) as f:
    _MANIFEST = json.load(f)
CLASSES = _MANIFEST["classes"]
FUNCTIONS = _MANIFEST.get("functions", {})


def public_methods(cls) -> list:
    return sorted(name for name in dir(cls) if not name.startswith("_"))


# Classes Python lacks are checked for a reason by the TypeScript side's
# "every missing method is explained", not skipped one by one here.
@pytest.mark.parametrize("name", sorted(n for n, spec in CLASSES.items() if spec["python"] is not None))
def test_exactly_the_listed_methods(name):
    spec = CLASSES[name]
    cls = getattr(rust, spec["python"])
    expected = sorted(op["python"] for op in spec["operations"] if op["python"] is not None)
    assert public_methods(cls) == expected


def test_every_listed_function_is_exported_and_every_missing_one_explained():
    problems = []
    for canonical, spec in sorted(FUNCTIONS.items()):
        name = spec["python"]
        if name is None:
            if not (spec.get("why") or {}).get("python"):
                problems.append(f"{canonical}: missing on python, with no why")
        elif not callable(getattr(rust, name, None)):
            problems.append(f"{canonical}: {name} is not exported")
    assert problems == []


def test_nothing_is_exported_that_the_manifest_does_not_list():
    listed = {spec["python"] for spec in CLASSES.values()} | {spec["python"] for spec in FUNCTIONS.values()}
    exported = {
        name
        for name in dir(rust)
        if not name.startswith("_")
        and callable(getattr(rust, name))
        and not (inspect.isclass(getattr(rust, name)) and issubclass(getattr(rust, name), BaseException))
    }
    assert sorted(exported - listed) == []
