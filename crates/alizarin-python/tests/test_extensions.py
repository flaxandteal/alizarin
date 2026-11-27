"""
clm's `reference` and filelist's `file-list` through the ORM, as
tests/dual-backend/extensions.test.ts checks on WASM and NAPI: importing each
extension registers its handler, then a resource's values are read, rendered
for display by core through the handlers, and an edit is coerced by the
handler before it is stored. Same fixture, same expected values.
"""

import json
import os

import pytest

import alizarin
import alizarin_clm  # noqa: F401  (registers the reference handler)
import alizarin_filelist  # noqa: F401  (registers the file-list handler)
from alizarin import GraphMutator
from alizarin.graph_manager import GraphManager
from alizarin.static_types import StaticGraph
from alizarin.view_models import viewContext

CRM = "http://www.cidoc-crm.org/cidoc-crm/"
RESOURCE_ID = "eeeeeeee-1111-2222-3333-444444444444"
LIST_ID = "2730d609-3a8d-49dc-bf51-6ac34e80294a"

# clm's fixture reference (ext/clm/js/tests/data).
REFERENCE = [{
    "labels": [{
        "id": "0ea39e2e-6663-467c-8707-ab492896d23e", "language_id": "en",
        "list_item_id": "6672d187-dfc3-4424-8c63-7a3b377b4159", "value": "Item 1>1",
        "valuetype_id": "prefLabel",
    }],
    "list_id": LIST_ID,
    "uri": "http://localhost:8000/plugins/controlled-list-manager/item/6672d187-dfc3-4424-8c63-7a3b377b4159",
}]
FILES = [
    {"name": "file1.pdf", "url": "/files/1", "file_id": "id-1", "type": "application/pdf"},
    {"name": "file2.jpg", "url": "/files/2", "file_id": "id-2", "type": "image/jpeg"},
]


@pytest.fixture
def record():
    """A record with a file-list and a reference node, and a value in each."""
    yield _record(REFERENCE)
    viewContext.graphManager = None


@pytest.fixture
def marker_record():
    """The record, its reference still an unresolved label marker."""
    yield _record([LABEL_MARKER])
    viewContext.graphManager = None


# A reference value not yet resolved against its list: what a tree carries
# when the concept wasn't in the RDM cache at conversion.
LABEL_MARKER = {"__needs_rdm_label_lookup": True, "label": "Bowl", "controlledList": LIST_ID}


def _record(kind):
    path = os.path.join(os.path.dirname(__file__), "../../../tests/data/models/Group.json")
    with open(path) as f:
        base = StaticGraph.from_dict(json.load(f)["graph"][0])
    mutator = GraphMutator(base)
    # Added as strings, then given their datatypes, as the JS test must.
    mutator._add_node(None, "attachments", "Attachments", "1", "string",
                      f"{CRM}E31_Document", f"{CRM}P70i_is_documented_in", None, None, None)
    mutator._add_node(None, "kind", "Kind", "1", "string",
                      f"{CRM}E55_Type", f"{CRM}P2_has_type", None, None, None)
    graph = mutator.apply().to_dict()
    nodes = {n["alias"]: n for n in graph["nodes"] if n.get("alias")}
    nodes["attachments"]["datatype"] = "file-list"
    nodes["kind"]["datatype"] = "reference"
    nodes["kind"]["config"] = {"controlledList": LIST_ID, "multiValue": True}

    def tile(alias, value):
        return {
            "tileid": f"{alias}-tile", "nodegroup_id": nodes[alias]["nodegroup_id"],
            "resourceinstance_id": RESOURCE_ID, "parenttile_id": None,
            "data": {nodes[alias]["nodeid"]: value}, "sortorder": 0, "provisionaledits": None,
        }

    gm = GraphManager(default_allow_all_nodegroups=True)
    gm.register_graph(graph)
    gm.register_resource({
        "resourceinstance": {"resourceinstanceid": RESOURCE_ID, "graph_id": graph["graphid"], "name": "Record"},
        "tiles": [tile("attachments", FILES), tile("kind", kind)],
    })
    viewContext.graphManager = gm
    return gm, graph


def test_both_handlers_are_registered():
    assert alizarin.has_type_handler("reference")
    assert alizarin.has_type_handler("file-list")


REGISTRATIONS = os.path.join(
    os.path.dirname(__file__), "../../../tests/data/extension-registrations.json"
)


@pytest.mark.parametrize("extension", ["clm", "filelist", "geo"])
def test_importing_an_extension_registers_what_it_does_on_every_binding(extension):
    # The same fixture tests/dual-backend/extensions.test.ts reads on WASM and NAPI.
    import importlib

    importlib.import_module(f"alizarin_{extension}")
    with open(REGISTRATIONS) as f:
        expected = json.load(f)[extension]
    handlers = alizarin.get_registered_extension_handlers()
    mutations = alizarin.list_extension_mutations()
    assert [h for h in expected["handlers"] if h not in handlers] == []
    assert [m for m in expected["mutations"] if m not in mutations] == []
    assert [d for d in expected["derive"] if not alizarin.has_derive_provider(d)] == []


@pytest.mark.asyncio
async def test_values_read_through_the_orm(record):
    gm, _ = record
    asset = await gm.getResource(RESOURCE_ID, lazy=False)
    assert len(await asset.attachments) == 2
    # Item by item: how a list view model stringifies differs per binding
    # (PARITY-RESEARCH.md, G-6).
    assert [str(v) for v in await asset.kind] == ["Item 1>1"]


@pytest.mark.asyncio
async def test_core_renders_both_for_display_through_their_handlers(record):
    gm, graph = record
    asset = await gm.getResource(RESOURCE_ID, lazy=False)
    # The graph's node configs, as the JS ORM passes its node config manager.
    ncm = alizarin._alizarin_rust.RustNodeConfigManager()
    ncm.load_graph_json(json.dumps(graph))
    display = asset._i._rust_core.to_display_json(ncm, "en")
    # A multi-value reference displays as a list (clm's display_as_list).
    assert {k: display[k] for k in ("attachments", "kind")} == {
        "attachments": "file1.pdf, file2.jpg",
        "kind": ["Item 1>1"],
    }


@pytest.mark.asyncio
async def test_an_unresolved_reference_displays_as_on_every_binding(marker_record):
    # clm renders an unresolved label marker by its label, as on WASM and
    # NAPI (tests/dual-backend/extensions.test.ts).
    gm, graph = marker_record
    asset = await gm.getResource(RESOURCE_ID, lazy=False)
    ncm = alizarin._alizarin_rust.RustNodeConfigManager()
    ncm.load_graph_json(json.dumps(graph))
    display = asset._i._rust_core.to_display_json(ncm, "en")
    assert display["kind"] == ["Bowl"]


@pytest.mark.asyncio
async def test_an_edit_is_coerced_by_the_handler_before_it_is_stored(record):
    gm, _ = record
    asset = await gm.getResource(RESOURCE_ID, lazy=False)
    asset.attachments = [FILES[1]]
    delta = asset.commit()
    assert [list(t["data"].values())[0] for t in delta["upserted"]] == [[{
        "accepted": False, "content": None, "file_id": "id-2", "index": 0, "last_modified": None,
        "name": "file2.jpg", "path": None, "renderer": None, "selected": False, "size": None,
        "status": None, "type": "image/jpeg", "url": "/files/2",
    }]]
