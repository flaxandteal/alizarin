"""GraphManager with a prebuild directory: graphs and business data loaded by
core, models found by name, and references followed through the registry, as
TypeScript's local recipe does (tests/dual-backend/docsLocalRecipe.test.ts,
on the same data: docs/example, here laid out as a prebuild)."""

import os
import shutil

import pytest

from alizarin.graph_manager import GraphManager
from alizarin.view_models import viewContext

EXAMPLE = os.path.join(os.path.dirname(__file__), "../../../docs/example")
PERSON_ID = "193750d0-e4b8-527d-8dc2-cbfb4cd31e72"
SITE_ID = "bb8ec53b-0d84-5d36-95a9-e02fdd046f24"


@pytest.fixture
def prebuild(tmp_path):
    """docs/example's graphs and business data in the prebuild layout."""
    models = tmp_path / "graphs" / "resource_models"
    models.mkdir(parents=True)
    for name in os.listdir(os.path.join(EXAMPLE, "resource_models")):
        if not name.startswith("_"):
            shutil.copy(os.path.join(EXAMPLE, "resource_models", name), models / name)
    shutil.copytree(os.path.join(EXAMPLE, "business_data"), tmp_path / "business_data")
    return str(tmp_path)


@pytest.fixture
def gm(prebuild):
    gm = GraphManager()
    loaded = gm.load_prebuild(prebuild)
    assert len(loaded["graph_ids"]) == 6
    assert PERSON_ID in loaded["resource_ids"]
    viewContext.graphManager = gm
    yield gm
    viewContext.graphManager = None


def test_import_prebuild_gives_cores_summary(prebuild):
    # As NAPI's importPrebuild (tests/dual-backend/prebuildImport.test.ts),
    # with Python's validator alongside.
    import alizarin

    result = alizarin.import_prebuild(prebuild, "http://localhost/")
    assert sorted(result) == ["collectionIds", "collections", "graphIds", "ontologyConfigs", "ontologyValidator"]
    assert len(result["graphIds"]) == 6


def test_load_graph_takes_a_class_name_a_display_name_or_an_id(gm):
    person = gm.load_graph("Person")
    assert gm.load_graph("person") is person
    assert gm.load_graph(person.graph.graphid) is person


def test_an_unknown_model_name_says_which_it_knows(gm):
    with pytest.raises(KeyError, match="Person"):
        gm.load_graph("Nope")


@pytest.mark.asyncio
async def test_references_resolve_through_the_registry(gm):
    person = await gm.getResource(PERSON_ID)
    site = await person.primary_site
    assert site.id == SITE_ID
    assert str(await site.name)
    org = await site.maintained_by
    assert str(await org.name) == "City Council -- Planning & GIS Division"


@pytest.mark.asyncio
async def test_a_models_resources_are_its_business_data(gm):
    people = await gm.load_graph("Person").all()
    assert PERSON_ID in [p.id for p in people]
