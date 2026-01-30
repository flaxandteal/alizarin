"""
Static type definitions matching TypeScript static-types.ts

These classes represent the core Arches data structures with pervasive type hints.
"""

from __future__ import annotations

import json
from typing import Any, Dict, List, Optional, Union
from dataclasses import dataclass, field


# =============================================================================
# Translatable String
# =============================================================================

@dataclass
class StaticTranslatableString:
    """
    Translatable string with language-keyed values.

    Matches TypeScript/Rust StaticTranslatableString.
    Can be constructed from a string (uses 'en' as default language)
    or a dict of language -> value mappings.
    """
    _value: Dict[str, str] = field(default_factory=dict)

    def __init__(self, value: Union[str, Dict[str, str]] = "") -> None:
        if isinstance(value, str):
            self._value = {"en": value}
        elif isinstance(value, dict):
            self._value = value
        else:
            self._value = {"en": str(value)}

    def __str__(self) -> str:
        return self._value.get("en", "") or next(iter(self._value.values()), "")

    def __getitem__(self, key: str) -> str:
        return self._value.get(key, "")

    def get(self, key: str, default: str = "") -> str:
        return self._value.get(key, default)

    def to_json(self) -> Dict[str, str]:
        return self._value

    @classmethod
    def from_json(cls, data: Union[str, Dict[str, str]]) -> StaticTranslatableString:
        return cls(data)


# =============================================================================
# Node Types
# =============================================================================

@dataclass
class StaticNode:
    """
    Represents a node in an Arches graph model.

    Matches TypeScript/Rust StaticNode with all fields properly typed.
    """
    nodeid: str
    name: str
    datatype: str
    nodegroup_id: Optional[str]
    alias: Optional[str]
    graph_id: str
    is_collector: bool
    isrequired: bool
    exportable: bool
    sortorder: Optional[int] = None
    config: Dict[str, Any] = field(default_factory=dict)
    parentproperty: Optional[str] = None
    ontologyclass: Optional[str] = None
    description: Optional[str] = None
    fieldname: Optional[str] = None
    hascustomalias: bool = False
    issearchable: bool = False
    istopnode: bool = False
    sourcebranchpublication_id: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization."""
        return {
            "nodeid": self.nodeid,
            "name": self.name,
            "datatype": self.datatype,
            "nodegroup_id": self.nodegroup_id,
            "alias": self.alias,
            "graph_id": self.graph_id,
            "is_collector": self.is_collector,
            "isrequired": self.isrequired,
            "exportable": self.exportable,
            "sortorder": self.sortorder,
            "config": self.config,
            "parentproperty": self.parentproperty,
            "ontologyclass": self.ontologyclass,
            "description": self.description,
            "fieldname": self.fieldname,
            "hascustomalias": self.hascustomalias,
            "issearchable": self.issearchable,
            "istopnode": self.istopnode,
            "sourcebranchpublication_id": self.sourcebranchpublication_id,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticNode:
        """Create from dictionary."""
        return cls(
            nodeid=data["nodeid"],
            name=data.get("name", ""),
            datatype=data.get("datatype", ""),
            nodegroup_id=data.get("nodegroup_id"),
            alias=data.get("alias"),
            graph_id=data.get("graph_id", ""),
            is_collector=data.get("is_collector", False),
            isrequired=data.get("isrequired", False),
            exportable=data.get("exportable", False),
            sortorder=data.get("sortorder"),
            config=data.get("config", {}),
            parentproperty=data.get("parentproperty"),
            ontologyclass=data.get("ontologyclass"),
            description=data.get("description"),
            fieldname=data.get("fieldname"),
            hascustomalias=data.get("hascustomalias", False),
            issearchable=data.get("issearchable", False),
            istopnode=data.get("istopnode", False),
            sourcebranchpublication_id=data.get("sourcebranchpublication_id"),
        )


@dataclass
class StaticEdge:
    """
    Represents an edge between nodes in an Arches graph.

    Matches TypeScript/Rust StaticEdge.
    """
    edgeid: str
    domainnode_id: str
    rangenode_id: str
    graph_id: str
    name: Optional[str] = None
    ontologyproperty: Optional[str] = None
    description: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "edgeid": self.edgeid,
            "domainnode_id": self.domainnode_id,
            "rangenode_id": self.rangenode_id,
            "graph_id": self.graph_id,
            "name": self.name,
            "ontologyproperty": self.ontologyproperty,
            "description": self.description,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticEdge:
        return cls(
            edgeid=data["edgeid"],
            domainnode_id=data["domainnode_id"],
            rangenode_id=data["rangenode_id"],
            graph_id=data["graph_id"],
            name=data.get("name"),
            ontologyproperty=data.get("ontologyproperty"),
            description=data.get("description"),
        )


@dataclass
class StaticNodegroup:
    """
    Represents a nodegroup in an Arches graph.

    Matches TypeScript/Rust StaticNodegroup.
    """
    nodegroupid: str
    cardinality: Optional[str] = None
    parentnodegroup_id: Optional[str] = None
    legacygroupid: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "nodegroupid": self.nodegroupid,
            "cardinality": self.cardinality,
            "parentnodegroup_id": self.parentnodegroup_id,
            "legacygroupid": self.legacygroupid,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticNodegroup:
        return cls(
            nodegroupid=data["nodegroupid"],
            cardinality=data.get("cardinality"),
            parentnodegroup_id=data.get("parentnodegroup_id"),
            legacygroupid=data.get("legacygroupid"),
        )


@dataclass
class StaticConstraint:
    """
    Represents a constraint on a card.

    Matches TypeScript/Rust StaticConstraint.
    """
    constraintid: str
    card_id: str
    uniquetoallinstances: bool = False
    nodes: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "constraintid": self.constraintid,
            "card_id": self.card_id,
            "uniquetoallinstances": self.uniquetoallinstances,
            "nodes": self.nodes,
        }


@dataclass
class StaticCard:
    """
    Represents a card in an Arches graph.

    Matches TypeScript/Rust StaticCard.
    """
    cardid: str
    nodegroup_id: str
    graph_id: str
    active: bool
    component_id: str
    name: Union[str, Dict[str, str]]
    visible: bool
    is_editable: Optional[bool] = None
    config: Optional[Dict[str, Any]] = None
    constraints: List[StaticConstraint] = field(default_factory=list)
    cssclass: Optional[str] = None
    description: Optional[Union[str, Dict[str, str]]] = None
    helpenabled: bool = False
    helptext: Dict[str, str] = field(default_factory=dict)
    helptitle: Dict[str, str] = field(default_factory=dict)
    instructions: Dict[str, str] = field(default_factory=dict)
    sortorder: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "cardid": self.cardid,
            "nodegroup_id": self.nodegroup_id,
            "graph_id": self.graph_id,
            "active": self.active,
            "component_id": self.component_id,
            "name": self.name,
            "visible": self.visible,
            "is_editable": self.is_editable,
            "config": self.config,
            "constraints": [c.to_dict() if isinstance(c, StaticConstraint) else c for c in self.constraints],
            "cssclass": self.cssclass,
            "description": self.description,
            "helpenabled": self.helpenabled,
            "helptext": self.helptext,
            "helptitle": self.helptitle,
            "instructions": self.instructions,
            "sortorder": self.sortorder,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticCard:
        return cls(
            cardid=data["cardid"],
            nodegroup_id=data["nodegroup_id"],
            graph_id=data["graph_id"],
            active=data.get("active", True),
            component_id=data.get("component_id", ""),
            name=data.get("name", ""),
            visible=data.get("visible", True),
            is_editable=data.get("is_editable", True),
            config=data.get("config"),
            constraints=data.get("constraints", []),
            cssclass=data.get("cssclass"),
            description=data.get("description"),
            helpenabled=data.get("helpenabled", False),
            helptext=data.get("helptext", {}),
            helptitle=data.get("helptitle", {}),
            instructions=data.get("instructions", {}),
            sortorder=data.get("sortorder"),
        )


# =============================================================================
# Tile Types
# =============================================================================

@dataclass
class StaticTile:
    """
    Represents a tile (data container) for a nodegroup.

    Matches TypeScript/Rust StaticTile.
    """
    tileid: Optional[str]
    nodegroup_id: str
    resourceinstance_id: str
    data: Dict[str, Any]
    parenttile_id: Optional[str] = None
    sortorder: Optional[int] = None
    provisionaledits: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "tileid": self.tileid,
            "nodegroup_id": self.nodegroup_id,
            "resourceinstance_id": self.resourceinstance_id,
            "data": self.data,
            "parenttile_id": self.parenttile_id,
            "sortorder": self.sortorder,
            "provisionaledits": self.provisionaledits,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticTile:
        return cls(
            tileid=data.get("tileid"),
            nodegroup_id=data["nodegroup_id"],
            resourceinstance_id=data["resourceinstance_id"],
            data=data.get("data", {}),
            parenttile_id=data.get("parenttile_id"),
            sortorder=data.get("sortorder"),
            provisionaledits=data.get("provisionaledits"),
        )


# =============================================================================
# Publication Types
# =============================================================================

@dataclass
class StaticPublication:
    """
    Represents a publication record for a graph.

    Matches TypeScript/Rust StaticPublication.
    """
    publicationid: str
    graph_id: str
    notes: Optional[str] = None
    published_time: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "publicationid": self.publicationid,
            "graph_id": self.graph_id,
            "notes": self.notes,
            "published_time": self.published_time,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticPublication:
        return cls(
            publicationid=data["publicationid"],
            graph_id=data["graph_id"],
            notes=data.get("notes"),
            published_time=data.get("published_time"),
        )


# =============================================================================
# Graph Meta Types
# =============================================================================

@dataclass
class StaticGraphMeta:
    """
    Lightweight metadata for a graph (without full structure).

    Matches TypeScript/Rust StaticGraphMeta.
    """
    graphid: str
    name: Union[str, Dict[str, str]]
    slug: Optional[str] = None
    iconclass: Optional[str] = None
    isresource: bool = True
    relatable_resource_model_ids: List[str] = field(default_factory=list)
    ontology_id: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "graphid": self.graphid,
            "name": self.name,
            "slug": self.slug,
            "iconclass": self.iconclass,
            "isresource": self.isresource,
            "relatable_resource_model_ids": self.relatable_resource_model_ids,
            "ontology_id": self.ontology_id,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticGraphMeta:
        return cls(
            graphid=data["graphid"],
            name=data.get("name", ""),
            slug=data.get("slug"),
            iconclass=data.get("iconclass"),
            isresource=data.get("isresource", True),
            relatable_resource_model_ids=data.get("relatable_resource_model_ids", []),
            ontology_id=data.get("ontology_id"),
        )


# =============================================================================
# Graph Types
# =============================================================================

@dataclass
class StaticFunctionsXGraphs:
    """
    Represents function-graph relationships.

    Matches TypeScript/Rust StaticFunctionsXGraphs.
    """
    function_id: str
    graph_id: str
    config: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "function_id": self.function_id,
            "graph_id": self.graph_id,
            "config": self.config,
        }


@dataclass
class StaticCardsXNodesXWidgets:
    """
    Represents card-node-widget relationships.

    Matches TypeScript/Rust StaticCardsXNodesXWidgets.
    """
    card_id: str
    node_id: str
    widget_id: str
    config: Dict[str, Any] = field(default_factory=dict)
    label: Optional[Union[str, Dict[str, str]]] = None
    sortorder: Optional[int] = None
    visible: bool = True

    def to_dict(self) -> Dict[str, Any]:
        return {
            "card_id": self.card_id,
            "node_id": self.node_id,
            "widget_id": self.widget_id,
            "config": self.config,
            "label": self.label,
            "sortorder": self.sortorder,
            "visible": self.visible,
        }


@dataclass
class StaticGraph:
    """
    Represents a complete Arches resource model graph.

    Matches TypeScript/Rust StaticGraph.
    """
    graphid: str
    name: Union[str, Dict[str, str]]
    nodes: List[StaticNode]
    edges: List[StaticEdge]
    nodegroups: List[StaticNodegroup]
    root: StaticNode
    cards: Optional[List[StaticCard]] = None
    cards_x_nodes_x_widgets: Optional[List[StaticCardsXNodesXWidgets]] = None
    functions_x_graphs: List[StaticFunctionsXGraphs] = field(default_factory=list)
    author: Optional[str] = None
    color: Optional[str] = None
    description: Optional[Union[str, Dict[str, str], StaticTranslatableString]] = None
    subtitle: Optional[Union[str, Dict[str, str], StaticTranslatableString]] = None
    iconclass: Optional[str] = None
    isresource: bool = True
    ontology_id: Optional[str] = None
    config: Dict[str, Any] = field(default_factory=dict)
    deploymentdate: Optional[str] = None
    deploymentfile: Optional[str] = None
    is_editable: Optional[bool] = None
    jsonldcontext: Optional[Union[str, Dict[str, Any]]] = None
    publication: Optional[StaticPublication] = None
    relatable_resource_model_ids: List[str] = field(default_factory=list)
    resource_2_resource_constraints: Optional[List[Any]] = None
    slug: Optional[str] = None
    template_id: Optional[str] = None
    version: Optional[str] = None

    # Internal caches
    _node_map: Optional[Dict[str, StaticNode]] = field(default=None, repr=False)
    _alias_map: Optional[Dict[str, StaticNode]] = field(default=None, repr=False)
    _edge_map: Optional[Dict[str, List[StaticEdge]]] = field(default=None, repr=False)
    _nodegroup_map: Optional[Dict[str, StaticNodegroup]] = field(default=None, repr=False)

    def build_indices(self) -> None:
        """Build lookup indices for fast access."""
        self._node_map = {node.nodeid: node for node in self.nodes}
        self._alias_map = {node.alias: node for node in self.nodes if node.alias}
        self._edge_map = {}
        for edge in self.edges:
            if edge.domainnode_id not in self._edge_map:
                self._edge_map[edge.domainnode_id] = []
            self._edge_map[edge.domainnode_id].append(edge)
        self._nodegroup_map = {ng.nodegroupid: ng for ng in self.nodegroups}

    def get_node(self, node_id: str) -> Optional[StaticNode]:
        """Get node by ID."""
        if self._node_map is None:
            self.build_indices()
        return self._node_map.get(node_id) if self._node_map else None

    def get_node_by_alias(self, alias: str) -> Optional[StaticNode]:
        """Get node by alias."""
        if self._alias_map is None:
            self.build_indices()
        return self._alias_map.get(alias) if self._alias_map else None

    def get_child_edges(self, node_id: str) -> List[StaticEdge]:
        """Get edges from a node to its children."""
        if self._edge_map is None:
            self.build_indices()
        return self._edge_map.get(node_id, []) if self._edge_map else []

    def get_nodegroup(self, nodegroup_id: str) -> Optional[StaticNodegroup]:
        """Get nodegroup by ID."""
        if self._nodegroup_map is None:
            self.build_indices()
        return self._nodegroup_map.get(nodegroup_id) if self._nodegroup_map else None

    @property
    def meta(self) -> StaticGraphMeta:
        """Get graph metadata as StaticGraphMeta."""
        return StaticGraphMeta(
            graphid=self.graphid,
            name=self.name,
            slug=self.slug,
            iconclass=self.iconclass,
            isresource=self.isresource,
            relatable_resource_model_ids=self.relatable_resource_model_ids,
            ontology_id=self.ontology_id,
        )

    def to_dict(self) -> Dict[str, Any]:
        """Export graph as dict."""
        return {
            "graphid": self.graphid,
            "name": self.name,
            "nodes": [n.to_dict() for n in self.nodes],
            "edges": [e.to_dict() for e in self.edges],
            "nodegroups": [ng.to_dict() for ng in self.nodegroups],
            "root": self.root.to_dict(),
            "cards": [c.to_dict() for c in self.cards] if self.cards else None,
            "author": self.author,
            "description": self.description.to_json() if isinstance(self.description, StaticTranslatableString) else self.description,
            "subtitle": self.subtitle.to_json() if isinstance(self.subtitle, StaticTranslatableString) else self.subtitle,
            "iconclass": self.iconclass,
            "isresource": self.isresource,
            "ontology_id": self.ontology_id,
            "config": self.config,
        }

    @classmethod
    def from_json(cls, json_str: str) -> StaticGraph:
        """
        Create a StaticGraph from JSON string.

        Handles both direct graph objects and wrapped format: {"graph": [...]}
        """
        data = json.loads(json_str)

        # Handle wrapped format
        if isinstance(data, dict) and "graph" in data:
            data = data["graph"][0]

        return cls.from_dict(data)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticGraph:
        """Create a StaticGraph from dictionary."""
        nodes = [StaticNode.from_dict(n) for n in data.get("nodes", [])]
        edges = [StaticEdge.from_dict(e) for e in data.get("edges", [])]
        nodegroups = [StaticNodegroup.from_dict(ng) for ng in data.get("nodegroups", [])]

        root_data = data.get("root")
        if root_data:
            root = StaticNode.from_dict(root_data)
        elif nodes:
            # Fall back to finding the top node
            root = next((n for n in nodes if n.istopnode), nodes[0])
        else:
            raise ValueError("Graph must have a root node")

        cards = None
        if "cards" in data and data["cards"]:
            cards = [StaticCard.from_dict(c) for c in data["cards"]]

        publication = None
        if "publication" in data and data["publication"]:
            publication = StaticPublication.from_dict(data["publication"])

        graph = cls(
            graphid=data["graphid"],
            name=data.get("name", ""),
            nodes=nodes,
            edges=edges,
            nodegroups=nodegroups,
            root=root,
            cards=cards,
            author=data.get("author"),
            description=data.get("description"),
            subtitle=data.get("subtitle"),
            iconclass=data.get("iconclass"),
            isresource=data.get("isresource", True),
            ontology_id=data.get("ontology_id"),
            config=data.get("config", {}),
            publication=publication,
        )
        graph.build_indices()
        return graph


# =============================================================================
# Resource Types
# =============================================================================

@dataclass
class StaticResourceDescriptors:
    """
    Resource descriptors for display purposes.

    Matches TypeScript/Rust StaticResourceDescriptors.
    """
    name: Optional[str] = None
    description: Optional[str] = None
    map_popup: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "map_popup": self.map_popup,
        }


@dataclass
class StaticResourceMetadata:
    """
    Resource instance metadata.

    Matches TypeScript/Rust StaticResourceMetadata.
    """
    descriptors: StaticResourceDescriptors
    graph_id: str
    name: str
    resourceinstanceid: str
    publication_id: Optional[str] = None
    principaluser_id: Optional[int] = None
    legacyid: Optional[str] = None
    graph_publication_id: Optional[str] = None
    createdtime: Optional[str] = None
    lastmodified: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "descriptors": self.descriptors.to_dict() if self.descriptors else None,
            "graph_id": self.graph_id,
            "name": self.name,
            "resourceinstanceid": self.resourceinstanceid,
            "publication_id": self.publication_id,
            "principaluser_id": self.principaluser_id,
            "legacyid": self.legacyid,
            "graph_publication_id": self.graph_publication_id,
            "createdtime": self.createdtime,
            "lastmodified": self.lastmodified,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticResourceMetadata:
        descriptors = None
        if "descriptors" in data and data["descriptors"]:
            descriptors = StaticResourceDescriptors(**data["descriptors"])
        return cls(
            descriptors=descriptors or StaticResourceDescriptors(),
            graph_id=data["graph_id"],
            name=data.get("name", ""),
            resourceinstanceid=data["resourceinstanceid"],
            publication_id=data.get("publication_id"),
            principaluser_id=data.get("principaluser_id"),
            legacyid=data.get("legacyid"),
            graph_publication_id=data.get("graph_publication_id"),
            createdtime=data.get("createdtime"),
            lastmodified=data.get("lastmodified"),
        )


@dataclass
class StaticResourceSummary:
    """
    Summary of a resource for list views.

    Matches TypeScript/Rust StaticResourceSummary.
    """
    resourceinstanceid: str
    graph_id: str
    displayname: Optional[str] = None
    displaydescription: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "resourceinstanceid": self.resourceinstanceid,
            "graph_id": self.graph_id,
            "displayname": self.displayname,
            "displaydescription": self.displaydescription,
        }


@dataclass
class StaticResourceReference:
    """
    Reference to a resource from another resource.

    Matches TypeScript/Rust StaticResourceReference.
    """
    id: str
    type: str
    graphId: str
    title: Optional[str] = None
    meta: Optional[Dict[str, Any]] = None
    root: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "type": self.type,
            "graphId": self.graphId,
            "title": self.title,
            "meta": self.meta,
            "root": self.root,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticResourceReference:
        return cls(
            id=data["id"],
            type=data.get("type", "(unknown)"),
            graphId=data.get("graphId", ""),
            title=data.get("title"),
            meta=data.get("meta"),
            root=data.get("root"),
        )


@dataclass
class StaticResource:
    """
    Represents a complete resource instance.

    Matches TypeScript/Rust StaticResource.
    """
    resourceinstance: StaticResourceMetadata
    tiles: Optional[List[StaticTile]] = None
    metadata: Dict[str, str] = field(default_factory=dict)
    cache: Optional[Any] = None
    scopes: Optional[Any] = None
    tiles_loaded: Optional[bool] = None

    def get_tiles(self) -> List[StaticTile]:
        """Get tiles as list."""
        return self.tiles or []

    def to_dict(self) -> Dict[str, Any]:
        result = {
            "resourceinstance": self.resourceinstance.to_dict(),
            "metadata": self.metadata,
        }
        if self.tiles is not None:
            result["tiles"] = [t.to_dict() for t in self.tiles]
        if self.cache is not None:
            result["__cache"] = self.cache
        if self.scopes is not None:
            result["__scopes"] = self.scopes
        if self.tiles_loaded is not None:
            result["tiles_loaded"] = self.tiles_loaded
        return result

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticResource:
        # Parse tiles
        tiles = None
        if "tiles" in data and data["tiles"]:
            tiles = [StaticTile.from_dict(t) for t in data["tiles"]]

        # Parse resourceinstance metadata
        resourceinstance = StaticResourceMetadata.from_dict(data["resourceinstance"])

        return cls(
            resourceinstance=resourceinstance,
            tiles=tiles,
            metadata=data.get("metadata", {}),
            cache=data.get("__cache") or data.get("cache"),
            scopes=data.get("__scopes") or data.get("scopes"),
            tiles_loaded=data.get("tiles_loaded"),
        )


# =============================================================================
# Concept / Collection Types (RDM)
# =============================================================================

@dataclass
class StaticValue:
    """
    Represents a value in the reference data manager.

    Matches TypeScript StaticValue.
    """
    id: str
    value: str
    _concept: Optional[StaticConcept] = field(default=None, repr=False)
    _concept_id: Optional[str] = None

    def __str__(self) -> str:
        return self.value

    def to_dict(self) -> Dict[str, str]:
        return {"id": self.id, "value": self.value}

    @classmethod
    def create(
        cls,
        referent: Union[str, StaticConcept],
        value_type: str,
        value: str,
        language: str = "en"
    ) -> StaticValue:
        """Create a StaticValue with deterministic ID."""
        import uuid
        referent_id = referent.id if isinstance(referent, StaticConcept) else referent
        concept = referent if isinstance(referent, StaticConcept) else None
        # Generate deterministic UUID
        namespace = uuid.UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8")  # UUID namespace
        id_str = uuid.uuid5(namespace, f"value/{referent_id}/{value_type}/{value}/{language}")
        return cls(
            id=str(id_str),
            value=value,
            _concept=concept,
            _concept_id=concept.id if concept else None,
        )


@dataclass
class StaticConcept:
    """
    Represents a concept in the reference data manager.

    Matches TypeScript StaticConcept.
    """
    id: str
    prefLabels: Dict[str, StaticValue]
    source: Optional[str] = None
    sortOrder: Optional[int] = None
    children: Optional[List[StaticConcept]] = None

    def get_pref_label(self, language: str = "en") -> Optional[StaticValue]:
        """Get preferred label for language."""
        return self.prefLabels.get(language) or next(iter(self.prefLabels.values()), None)

    def __str__(self) -> str:
        label = self.get_pref_label()
        return label.value if label else ""

    @classmethod
    def from_value(
        cls,
        concept_scheme: Optional[StaticConcept],
        value: Union[str, StaticValue, Dict[str, StaticValue]],
        children: Optional[List[Union[str, StaticValue, StaticConcept]]] = None,
        base_language: str = "en",
        source: Optional[str] = None,
        sort_order: Optional[int] = None
    ) -> StaticConcept:
        """Create a concept from a value."""
        import uuid

        if isinstance(value, str):
            pref_labels = {base_language: StaticValue(id="", value=value)}
            tmp_value = value
        elif isinstance(value, StaticValue):
            pref_labels = {base_language: value}
            tmp_value = value.value
        elif base_language in value:
            pref_labels = value
            tmp_value = value[base_language].value
        else:
            first = sorted(value.items())[0]
            pref_labels = value
            tmp_value = first[1].value

        # Generate concept ID
        namespace = uuid.UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8")
        scheme_id = concept_scheme.id if concept_scheme else "(none)"
        concept_id = str(uuid.uuid5(namespace, f"concept/{scheme_id}/{tmp_value}"))

        child_concepts = []
        if children:
            for child in children:
                if isinstance(child, StaticConcept):
                    child_concepts.append(child)
                else:
                    child_concepts.append(cls.from_value(concept_scheme, child, base_language=base_language))

        return cls(
            id=concept_id,
            prefLabels=pref_labels,
            source=source,
            sortOrder=sort_order,
            children=child_concepts if child_concepts else None,
        )


@dataclass
class StaticCollection:
    """
    Represents a collection of concepts.

    Matches TypeScript StaticCollection.
    """
    id: str
    prefLabels: Dict[str, StaticValue]
    concepts: Dict[str, StaticConcept]
    _all_concepts: Dict[str, StaticConcept] = field(default_factory=dict, repr=False)
    _values: Dict[str, StaticValue] = field(default_factory=dict, repr=False)

    def __post_init__(self) -> None:
        """Build value lookup cache."""
        self._all_concepts = {}
        self._values = {}
        self._index_concepts(self.concepts.values())

    def _index_concepts(self, concepts: Any) -> None:
        """Recursively index concepts and their values."""
        for concept in concepts:
            self._all_concepts[concept.id] = concept
            for value in concept.prefLabels.values():
                self._values[value.id] = value
                value._concept = concept
                value._concept_id = concept.id
            if concept.children:
                self._index_concepts(concept.children)

    def get_concept_value(self, value_id: str) -> Optional[StaticValue]:
        """Get a value by its ID."""
        return self._values.get(value_id)

    def get_concept_by_value(self, label: str) -> Optional[StaticConcept]:
        """Get a concept by its label value."""
        for value in self._values.values():
            if value.value == label:
                return value._concept
        return None

    def __str__(self) -> str:
        label = self.prefLabels.get("en") or next(iter(self.prefLabels.values()), None)
        return label.value if label else ""

    @classmethod
    def create(
        cls,
        name: Union[str, StaticValue, Dict[str, StaticValue]],
        concepts: Union[List[StaticConcept], Dict[str, StaticConcept]],
        collection_id: Optional[str] = None
    ) -> StaticCollection:
        """Create a collection."""
        import uuid

        # Convert concepts list to dict
        if isinstance(concepts, list):
            concepts_dict = {c.id: c for c in concepts}
        else:
            concepts_dict = concepts

        # Handle name
        if isinstance(name, str):
            name_value = StaticValue.create("", "prefLabel", name)
            pref_labels = {"en": name_value}
            name_for_id = name
        elif isinstance(name, StaticValue):
            pref_labels = {"en": name}
            name_for_id = name.value
        else:
            pref_labels = name
            name_for_id = next(iter(name.values())).value

        # Generate collection ID if not provided
        if not collection_id:
            namespace = uuid.UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8")
            collection_id = str(uuid.uuid5(namespace, f"collection/{name_for_id}"))

        return cls(
            id=collection_id,
            prefLabels=pref_labels,
            concepts=concepts_dict,
        )

    @classmethod
    def from_concept_scheme(
        cls,
        concept_scheme: StaticConcept,
        name: Optional[Union[str, StaticValue, Dict[str, StaticValue]]] = None,
        collection_id: Optional[str] = None
    ) -> StaticCollection:
        """Create a collection from a concept scheme."""
        collection_name = name if name else str(concept_scheme)
        children = concept_scheme.children or []
        return cls.create(
            name=collection_name,
            concepts=children,
            collection_id=collection_id,
        )


@dataclass
class StaticDomainValue:
    """
    Represents a domain value from node config.

    Matches TypeScript StaticDomainValue.
    """
    id: str
    selected: bool
    text: Dict[str, str]

    def __str__(self) -> str:
        return self.text.get("en", "") or next(iter(self.text.values()), "")

    def lang(self, language: str) -> Optional[str]:
        """Get value in specific language."""
        return self.text.get(language)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "selected": self.selected,
            "text": self.text,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> StaticDomainValue:
        return cls(
            id=data["id"],
            selected=data.get("selected", False),
            text=data.get("text", {}),
        )
