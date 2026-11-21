"""
The `alizarin` CLI: `to-trees` and `convert`, round trip, on the
output-parity fixture. Both commands called their bindings with arguments
the bindings do not take, and nothing ran them.
"""

import json
import os

from click.testing import CliRunner

from alizarin.cli import cli

DATA_DIR = os.path.join(os.path.dirname(__file__), "../../../tests/data/output-parity")


def load(name):
    with open(os.path.join(DATA_DIR, name), encoding="utf-8") as f:
        return json.load(f)


def test_to_trees_then_convert(tmp_path):
    graph = tmp_path / "graph.json"
    graph.write_text(json.dumps(load("graph.json")))
    resources = tmp_path / "resources.json"
    resources.write_text(json.dumps([load("resource.json")]))
    trees = tmp_path / "trees.json"
    tiles = tmp_path / "tiles.json"
    runner = CliRunner()

    result = runner.invoke(cli, ["to-trees", "-g", str(graph), "-i", str(resources), "-o", str(trees)])
    assert result.exit_code == 0, result.output
    [tree] = json.loads(trees.read_text())
    assert tree["resourceinstanceid"] == "0b0b0b0b-0000-4000-8000-000000000001"
    assert tree["name"]["en"]["value"] == "Fort"

    result = runner.invoke(cli, ["convert", "-g", str(graph), "-i", str(trees), "-o", str(tiles)])
    assert result.exit_code == 0, result.output
    [resource] = json.loads(tiles.read_text())["business_data"]["resources"]
    assert resource["resourceinstance"]["resourceinstanceid"] == "0b0b0b0b-0000-4000-8000-000000000001"
    names = [t["data"] for t in resource["tiles"] if "0a0a0a0a-0000-4000-8000-000000000011" in t["data"]]
    assert names[0]["0a0a0a0a-0000-4000-8000-000000000011"]["en"]["value"] == "Fort"
