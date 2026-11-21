"""Fluent builder for graph schema changes, matching TypeScript's GraphMutator.

It records core graph mutations, and ``apply()`` runs them in core (through
``apply_mutations_from_json``), so the same mutations give the same graph, ids
included, as on WASM and NAPI. Cards and widgets for new nodes are created by
core when ``autocreate_card`` is set.
"""

import json
from typing import Any, Dict, List, Optional, Union

from .alizarin import apply_mutations_from_json
from .static_types import StaticGraph, from_core

OntologyClass = Union[str, List[str]]


def _translatable(value: Any) -> Any:
    """A plain or translatable string, as core reads a translatable string."""
    if value is None or isinstance(value, (str, dict)):
        return value
    return value.to_dict() if hasattr(value, "to_dict") else str(value)


class GraphMutator:
    """Records core graph mutations; ``apply()`` gives the mutated graph."""

    def __init__(self, base_graph: StaticGraph, autocreate_card: bool = True) -> None:
        self.base_graph = base_graph
        self.mutations: List[Dict[str, Any]] = []
        self.autocreate_card = autocreate_card

    def add_semantic_node(
        self,
        parent_alias: Optional[str],
        alias: str,
        name: str,
        cardinality: str,
        ontology_class: OntologyClass,
        parent_property: str,
        description: Optional[str] = None,
        options: Optional[Dict[str, Any]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> "GraphMutator":
        return self._add_node(parent_alias, alias, name, cardinality, "semantic",
                              ontology_class, parent_property, description, options, config)

    def add_string_node(
        self,
        parent_alias: Optional[str],
        alias: str,
        name: str,
        cardinality: str,
        ontology_class: OntologyClass,
        parent_property: str,
        description: Optional[str] = None,
        options: Optional[Dict[str, Any]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> "GraphMutator":
        return self._add_node(parent_alias, alias, name, cardinality, "string",
                              ontology_class, parent_property, description, options, config)

    def add_concept_node(
        self,
        parent_alias: Optional[str],
        alias: str,
        name: str,
        collection: Any,
        cardinality: str,
        ontology_class: OntologyClass,
        parent_property: str,
        description: Optional[str] = None,
        options: Optional[Dict[str, Any]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> "GraphMutator":
        """``collection`` is a collection (with an ``id``) or its id; set
        ``options["is_list"]`` for a concept-list node."""
        config = dict(config or {})
        collection_id = getattr(collection, "id", collection)
        if collection_id:
            config["rdmCollection"] = collection_id
        node_options = dict(options or {})
        is_list = node_options.pop("is_list", False)
        return self._add_node(parent_alias, alias, name, cardinality,
                              "concept-list" if is_list else "concept",
                              ontology_class, parent_property, description, node_options, config)

    def add_card(
        self,
        nodegroup: Any,
        name: Any,
        component: Any = None,
        options: Optional[Dict[str, Any]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> "GraphMutator":
        """``nodegroup`` is a nodegroup (with a ``nodegroupid``) or its id;
        ``component`` a card component (with an ``id``), its id, or None."""
        options = options or {}
        nodegroup_id = getattr(nodegroup, "nodegroupid", nodegroup)
        component_id = getattr(component, "id", component)
        self.mutations.append({"AddCard": {
            "nodegroup_id": nodegroup_id,
            "name": _translatable(name),
            "component_id": component_id,
            "options": {
                "active": options.get("active"),
                "cssclass": options.get("cssclass"),
                "helpenabled": options.get("helpenabled"),
                "helptext": _translatable(options.get("helptext")),
                "helptitle": _translatable(options.get("helptitle")),
                "instructions": _translatable(options.get("instructions")),
                "is_editable": options.get("is_editable"),
                "description": _translatable(options.get("description")),
                "sortorder": options.get("sortorder"),
                "visible": options.get("visible"),
            },
            "config": config,
        }})
        return self

    def add_widget_to_card(
        self,
        node_id: str,
        widget: Any,
        name: str,
        config: Dict[str, Any],
        sortorder: Optional[int] = None,
        visible: Optional[bool] = None,
    ) -> "GraphMutator":
        """``widget`` is a widget (with an ``id``) or its id."""
        self.mutations.append({"AddWidgetToCard": {
            "node_id": node_id,
            "widget_id": getattr(widget, "id", widget),
            "label": name,
            "config": config,
            "sortorder": sortorder,
            "visible": visible,
        }})
        return self

    def apply(self) -> StaticGraph:
        """A new graph with the mutations applied, by core; the base graph is
        untouched."""
        request = {
            "mutations": self.mutations,
            "options": {
                "autocreate_card": self.autocreate_card,
                "autocreate_widget": self.autocreate_card,
            },
        }
        mutated = apply_mutations_from_json(
            json.dumps(self.base_graph.to_dict()), json.dumps(request)
        )
        return from_core(StaticGraph, json.loads(mutated))

    def _add_node(
        self,
        parent_alias: Optional[str],
        alias: str,
        name: str,
        cardinality: str,
        datatype: str,
        ontology_class: OntologyClass,
        parent_property: str,
        description: Optional[str],
        options: Optional[Dict[str, Any]],
        config: Optional[Dict[str, Any]],
    ) -> "GraphMutator":
        self.mutations.append({"AddNode": {
            "parent_alias": parent_alias,
            "alias": alias,
            "name": name,
            "cardinality": cardinality,
            "datatype": datatype,
            "ontology_class": ontology_class,
            "parent_property": parent_property,
            "description": description,
            "config": config,
            "options": dict(options or {}),
        }})
        return self
