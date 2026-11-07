"""
tile_namespace: derived cardinality-n tile ids for tree conversion.

Without a namespace, multi-valued tiles keep random ids (layers can only add to
each other). With one, the same tree yields the same ids, and different layer
namespaces keep their new tiles apart.
"""

import json
import os

import alizarin


MODELS_DIR = os.path.join(os.path.dirname(__file__), '../../../tests/data/models')
LAYER_A = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
LAYER_B = "6ba7b811-9dad-11d1-80b4-00c04fd430c8"

# Two names: `name` is multi-valued, so these are cardinality-n tiles.
TREES = json.dumps([
    {"name": [
        {"forenames": [{"forename": "Alice"}]},
        {"forenames": [{"forename": "Alys"}]},
    ]}
])


def register_person() -> str:
    with open(os.path.join(MODELS_DIR, 'Person.json')) as f:
        data = json.load(f)
    if 'graph' in data and isinstance(data['graph'], list):
        data = data['graph'][0]
    return alizarin.register_graph(json.dumps(data))


def tile_ids(graph_id: str, tile_namespace=None) -> list:
    result = alizarin.batch_trees_to_tiles(
        trees_json=TREES,
        graph_id=graph_id,
        strict=True,
        id_keys=["person-1"],
        tile_namespace=tile_namespace,
    )
    [resource] = result['business_data']['resources']
    return [(t['tileid'], t.get('parenttile_id')) for t in resource['tiles']]


def test_namespace_makes_tile_ids_reproducible():
    graph_id = register_person()
    assert tile_ids(graph_id, LAYER_A) == tile_ids(graph_id, LAYER_A)


def test_without_namespace_multi_valued_ids_stay_random():
    graph_id = register_person()
    assert tile_ids(graph_id) != tile_ids(graph_id)


def test_layer_namespaces_do_not_share_new_tile_ids():
    graph_id = register_person()
    a = {tile_id for tile_id, _ in tile_ids(graph_id, LAYER_A)}
    b = {tile_id for tile_id, _ in tile_ids(graph_id, LAYER_B)}
    assert a != b
