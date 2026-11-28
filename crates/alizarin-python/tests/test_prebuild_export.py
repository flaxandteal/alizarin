"""
The prebuild export written to a directory is the in-memory export's files,
as on NAPI (tests/dual-backend/prebuildExport.test.ts): both are core's.
"""

import json
import os

import alizarin

DATA_DIR = os.path.join(os.path.dirname(__file__), "../../../tests/data/output-parity")
BASE_URI = "http://localhost/"


def written(out_dir: str) -> dict:
    files = {}
    for root, _, names in os.walk(out_dir):
        for name in names:
            path = os.path.join(root, name)
            with open(path, encoding="utf-8") as f:
                files[os.path.relpath(path, out_dir)] = f.read()
    return files


def test_graphs_written_to_a_directory_are_the_in_memory_export(tmp_path):
    with open(os.path.join(DATA_DIR, "graph.json"), encoding="utf-8") as f:
        graph = json.load(f)["graph"][0]
    alizarin.register_graph(json.dumps(graph))
    export = alizarin.build_prebuild_export(BASE_URI, [graph["graphid"]])
    expected = {f["relativePath"]: f["content"] for f in export["graphFiles"]}
    assert list(expected) == ["graphs/resource_models/Parity.json"]

    by_id = tmp_path / "by_id"
    alizarin._alizarin_rust.export_graphs_to_dir([graph["graphid"]], str(by_id))
    assert written(str(by_id)) == expected

    package = tmp_path / "package"
    alizarin.export_prebuild(str(package), BASE_URI, [graph["graphid"]])
    assert {k: v for k, v in written(str(package)).items() if k.startswith("graphs/")} == expected
