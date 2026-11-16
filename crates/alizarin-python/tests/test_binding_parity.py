"""
Binding parity: each Python wrapper class has exactly the methods
tests/binding-parity.json lists for Python, so a method added to one binding
only fails until it is listed (with its reason, if another binding lacks it).
The TypeScript side checks WASM and NAPI against the same manifest in
tests/dual-backend/bindingParity.test.ts.
"""

import json
import os

import pytest

import alizarin.alizarin as rust

MANIFEST = os.path.join(os.path.dirname(__file__), "../../../tests/binding-parity.json")

with open(MANIFEST) as f:
    CLASSES = json.load(f)["classes"]


def public_methods(cls) -> list:
    return sorted(name for name in dir(cls) if not name.startswith("_"))


@pytest.mark.parametrize("name", sorted(CLASSES))
def test_exactly_the_listed_methods(name):
    spec = CLASSES[name]
    if spec["python"] is None:
        pytest.skip(f"no native Python {name}: {spec['why']['python']}")
    cls = getattr(rust, spec["python"])
    expected = sorted(op["python"] for op in spec["operations"] if op["python"] is not None)
    assert public_methods(cls) == expected
