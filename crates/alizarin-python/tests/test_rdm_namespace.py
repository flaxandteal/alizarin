"""
RDM namespace: core keeps the namespace and the string it was set from, as on
WASM and NAPI (tests/dual-backend/rdmNamespace.test.ts).
"""

import pytest

import alizarin.alizarin as rust

URL = "http://example.org/rdm/"
# uuid5(NAMESPACE_URL, URL)
URL_UUID = "81adf959-2b61-5009-8011-c017660f4f1f"


@pytest.fixture(autouse=True)
def cleared():
    yield
    rust.clear_rdm_namespace()


def test_keeps_the_url_it_was_set_from_and_its_uuid():
    rust.set_rdm_namespace(URL)
    assert rust.get_rdm_namespace_raw() == URL
    assert rust.get_rdm_namespace() == URL_UUID
    assert rust.has_rdm_namespace()


def test_clearing_forgets_both():
    rust.set_rdm_namespace(URL)
    rust.clear_rdm_namespace()
    assert rust.get_rdm_namespace_raw() is None
    assert rust.get_rdm_namespace() is None
    assert not rust.has_rdm_namespace()
