declare class StaticGraphMeta {
    [key: string]: any;
    author: string | undefined;
    cards: number | undefined;
    cards_x_nodes_x_widgets: number | undefined;
    color: string | undefined;
    description: {
        [lang: string]: string;
    } | undefined;
    edges: number | undefined;
    graphid: string;
    iconclass: string | undefined;
    is_editable: boolean | undefined;
    isresource: boolean | undefined;
    jsonldcontext: {
        [key: string]: any;
    } | undefined;
    name: {
        [lang: string]: string;
    } | undefined;
    nodegroups: number | undefined;
    nodes: number | undefined;
    ontology_id: string | undefined;
    publication: {
        [key: string]: string | null;
    } | undefined;
    relatable_resource_model_ids: string[];
    resource_2_resource_constraints: any[];
    root: StaticNode | undefined;
    slug: string | undefined;
    subtitle: {
        [lang: string]: string;
    } | undefined;
    version: string | undefined;
    constructor(jsondata: StaticGraphMeta);
}
declare class StaticTranslatableString extends String {
    translations: Map<string, string>;
    lang: string;
    constructor(s: string | StaticTranslatableString, lang?: undefined | string);
}
declare class StaticNodegroup {
    legacygroupid: null;
    nodegroupid: string;
    parentnodegroup_id: string | null;
    cardinality: "1" | "n" | null;
    constructor(jsonData: StaticNodegroup);
}
declare class StaticNode {
    alias: string | null;
    config: {
        [key: string]: any;
    };
    datatype: string;
    description: string | null;
    exportable: boolean;
    fieldname: null | string;
    graph_id: string;
    hascustomalias: boolean;
    is_collector: boolean;
    isrequired: boolean;
    issearchable: boolean;
    istopnode: boolean;
    name: string;
    nodegroup_id: string | null;
    nodeid: string;
    parentproperty: string | null;
    sortorder: number;
    ontologyclass: string | null;
    sourcebranchpublication_id: null | string;
    constructor(jsonData: StaticNode);
}
declare class StaticConstraint {
    card_id: string;
    constraintid: string;
    nodes: Array<string>;
    uniquetoallinstances: boolean;
    constructor(jsonData: StaticConstraint);
}
declare class StaticCard {
    active: boolean;
    cardid: string;
    component_id: string;
    config: null | object;
    constraints: Array<StaticConstraint>;
    cssclass: null | string;
    description: string | null | StaticTranslatableString;
    graph_id: string;
    helpenabled: boolean;
    helptext: StaticTranslatableString;
    helptitle: StaticTranslatableString;
    instructions: StaticTranslatableString;
    is_editable: boolean;
    name: StaticTranslatableString;
    nodegroup_id: string;
    sortorder: number | null;
    visible: boolean;
    constructor(jsonData: StaticCard);
}
declare class StaticCardsXNodesXWidgets {
    card_id: string;
    config: object;
    id: string;
    label: StaticTranslatableString;
    node_id: string;
    sortorder: number;
    visible: boolean;
    widget_id: string;
    constructor(jsonData: StaticCardsXNodesXWidgets);
}
declare class StaticEdge {
    description: null;
    domainnode_id: string;
    edgeid: string;
    graph_id: string;
    name: null | string;
    rangenode_id: string;
    ontologyproperty: null | string;
    constructor(jsonData: StaticEdge);
}
declare class StaticFunctionsXGraphs {
    config: object;
    function_id: string;
    graph_id: string;
    id: string;
    constructor(jsonData: StaticFunctionsXGraphs);
}
declare class StaticPublication {
    graph_id: string;
    notes: null | string;
    publicationid: string;
    published_time: string;
    constructor(jsonData: StaticPublication);
}
declare class StaticGraph {
    author: string;
    cards: Array<StaticCard> | null;
    cards_x_nodes_x_widgets: Array<StaticCardsXNodesXWidgets> | null;
    color: string | null;
    config: object;
    deploymentdate: null | string;
    deploymentfile: null | string;
    description: StaticTranslatableString;
    edges: Array<StaticEdge>;
    functions_x_graphs: Array<StaticFunctionsXGraphs> | null;
    graphid: string;
    iconclass: string;
    is_editable: boolean | null;
    isresource: boolean;
    jsonldcontext: string | null;
    name: StaticTranslatableString;
    nodegroups: Array<StaticNodegroup>;
    nodes: Array<StaticNode>;
    ontology_id: string | null;
    publication: StaticPublication | null;
    relatable_resource_model_ids: Array<string>;
    resource_2_resource_constraints: Array<any> | null;
    root: StaticNode;
    slug: string | null;
    subtitle: StaticTranslatableString;
    template_id: string;
    version: string;
    constructor(jsonData: StaticGraph);
}
type StaticProvisionalEdit = any;
declare class StaticValue {
    id: string;
    value: string;
    __concept: StaticConcept | null;
    __conceptId: string | null;
    constructor(jsonData: StaticValue, concept?: StaticConcept | string | null);
    toString(): string;
}
declare class StaticConcept {
    id: string;
    prefLabels: {
        [lang: string]: StaticValue;
    };
    source: string | null;
    sortOrder: number | null;
    children: StaticConcept[] | null;
    constructor(jsonData: StaticConcept);
    getPrefLabel(): StaticValue;
    toString(): string;
}
declare class StaticCollection {
    id: string;
    prefLabels: {
        [lang: string]: StaticValue;
    };
    concepts: {
        [conceptId: string]: StaticConcept;
    };
    __allConcepts: {
        [conceptId: string]: StaticConcept;
    };
    __values: {
        [valueId: string]: StaticValue;
    };
    constructor(jsonData: StaticCollection);
    getConceptValue(valueId: string): StaticValue;
    toString(): StaticValue;
}
declare class StaticTile {
    data: Map<string, object | Map<string, any> | Array<any> | null | number | boolean | string>;
    nodegroup_id: string;
    resourceinstance_id: string;
    tileid: string | null;
    parenttile_id: string | null;
    provisionaledits: null | Array<StaticProvisionalEdit>;
    sortorder: number | null;
    constructor(jsonData: StaticTile);
    ensureId(): string;
}
declare class StaticResourceDescriptors {
    [key: string]: (string | undefined | Function);
    name?: string;
    map_popup?: string;
    description?: string;
    constructor(jsonData?: StaticResourceDescriptors);
    isEmpty(): boolean;
}
declare class StaticResourceMetadata {
    descriptors: StaticResourceDescriptors;
    graph_id: string;
    name: string;
    resourceinstanceid: string;
    publication_id: string | null;
    principaluser_id: number | null;
    legacyid: null | string;
    graph_publication_id: string | null;
    constructor(jsonData: StaticResourceMetadata);
}
declare class StaticDomainValue {
    id: string;
    selected: boolean;
    text: {
        [lang: string]: string;
    };
    constructor(jsonData: StaticDomainValue);
    toString(): string;
    lang(lang: string): string | undefined;
    forJson(): Promise<{
        id: string;
        selected: boolean;
        text: {
            [lang: string]: string;
        };
    }>;
}
declare class StaticResourceReference {
    id: string;
    type: string | undefined;
    graphId: string;
    title: string | undefined;
    root: any | undefined;
    meta?: {
        [key: string]: any;
    };
    constructor(jsonData: StaticResourceReference);
}
declare class StaticResource {
    resourceinstance: StaticResourceMetadata;
    tiles: Array<StaticTile> | null;
    metadata: {
        [key: string]: string;
    };
    __cache: {
        [tileId: string]: {
            [nodeId: string]: {
                [key: string]: string;
            };
        };
    } | undefined;
    __source: string | undefined;
    __scopes: string[] | undefined;
    constructor(jsonData: StaticResource);
}
export { StaticValue, StaticTile, StaticGraph, StaticResource, StaticResourceMetadata, StaticNode, StaticNodegroup, StaticEdge, StaticCard, StaticCardsXNodesXWidgets, StaticCollection, StaticConcept, StaticDomainValue, StaticResourceReference, StaticGraphMeta, StaticFunctionsXGraphs, StaticResourceDescriptors };
