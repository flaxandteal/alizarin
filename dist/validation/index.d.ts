/**
 * Alizarin Validation Module
 *
 * Provides validation utilities for Arches graph models and business data files.
 * Ensures compatibility with Alizarin's loading requirements.
 *
 * @module validation
 */
export { GraphLoadingValidator, validateGraphLoading, type ValidationResult, type ValidationResults, type ValidationSummary } from './validators/index.js';
export declare const schemas: {
    graphModel: {
        $schema: string;
        title: string;
        description: string;
        type: string;
        required: string[];
        properties: {
            graph: {
                type: string;
                items: {
                    $ref: string;
                };
                minItems: number;
                maxItems: number;
            };
        };
        definitions: {
            uuid: {
                type: string;
                pattern: string;
            };
            multilingualString: {
                type: string;
                patternProperties: {
                    "^[a-z]{2}(-[A-Z]{2})?$": {
                        type: string;
                    };
                };
                additionalProperties: boolean;
                minProperties: number;
            };
            graph: {
                type: string;
                required: string[];
                properties: {
                    graphid: {
                        $ref: string;
                    };
                    name: {
                        $ref: string;
                    };
                    subtitle: {
                        $ref: string;
                    };
                    description: {
                        $ref: string;
                    };
                    author: {
                        type: string;
                    };
                    deploymentdate: {
                        type: string;
                        format: string;
                    };
                    version: {
                        type: string;
                    };
                    isresource: {
                        type: string;
                    };
                    iconclass: {
                        type: string;
                    };
                    color: {
                        type: string;
                        pattern: string;
                    };
                    ontology_id: {
                        type: string;
                    };
                    template_id: {
                        $ref: string;
                    };
                    functions: {
                        type: string;
                    };
                    nodes: {
                        type: string;
                        items: {
                            $ref: string;
                        };
                        minItems: number;
                    };
                    nodegroups: {
                        type: string;
                        items: {
                            $ref: string;
                        };
                        minItems: number;
                    };
                    edges: {
                        type: string;
                        items: {
                            $ref: string;
                        };
                    };
                    cards: {
                        type: string;
                        items: {
                            $ref: string;
                        };
                    };
                    cards_x_nodes_x_widgets: {
                        type: string;
                    };
                    relatable_resource_model_ids: {
                        type: string;
                    };
                    resource_2_resource_constraints: {
                        type: string;
                    };
                    functions_x_graphs: {
                        type: string;
                    };
                    publication: {
                        type: string[];
                    };
                };
            };
            node: {
                type: string;
                required: string[];
                properties: {
                    nodeid: {
                        $ref: string;
                    };
                    name: {
                        type: string;
                    };
                    description: {
                        type: string;
                    };
                    datatype: {
                        type: string;
                        enum: string[];
                    };
                    istopnode: {
                        type: string;
                    };
                    nodegroup_id: {
                        oneOf: ({
                            $ref: string;
                            type?: undefined;
                        } | {
                            type: string;
                            $ref?: undefined;
                        })[];
                    };
                    config: {
                        type: string;
                    };
                    graph_id: {
                        $ref: string;
                    };
                    sortorder: {
                        type: string;
                    };
                    fieldname: {
                        type: string[];
                    };
                    exportable: {
                        type: string;
                    };
                    isrequired: {
                        type: string;
                    };
                    issearchable: {
                        type: string;
                    };
                    is_collector: {
                        type: string;
                    };
                    hascustomalias: {
                        type: string;
                    };
                    ontologyclass: {
                        type: string[];
                    };
                    alias: {
                        type: string[];
                    };
                    parentproperty: {
                        type: string[];
                    };
                    sourcebranchpublication_id: {
                        type: string[];
                    };
                };
                if: {
                    properties: {
                        istopnode: {
                            const: boolean;
                        };
                    };
                };
                then: {
                    properties: {
                        nodegroup_id: {
                            type: string;
                        };
                    };
                    required: string[];
                };
                else: {
                    properties: {
                        nodegroup_id: {
                            $ref: string;
                        };
                    };
                    required: string[];
                };
            };
            nodegroup: {
                type: string;
                required: string[];
                properties: {
                    nodegroupid: {
                        $ref: string;
                    };
                    cardinality: {
                        type: string[];
                        enum: string[];
                    };
                    parentnodegroup_id: {
                        oneOf: ({
                            $ref: string;
                            type?: undefined;
                        } | {
                            type: string;
                            $ref?: undefined;
                        })[];
                    };
                    legacygroupid: {
                        type: string[];
                    };
                    name: {
                        type: string;
                    };
                    sortorder: {
                        type: string;
                    };
                };
            };
            edge: {
                type: string;
                required: string[];
                properties: {
                    edgeid: {
                        $ref: string;
                    };
                    graph_id: {
                        $ref: string;
                    };
                    domainnode_id: {
                        $ref: string;
                    };
                    rangenode_id: {
                        $ref: string;
                    };
                    name: {
                        type: string[];
                    };
                    description: {
                        type: string[];
                    };
                    ontologyproperty: {
                        type: string[];
                    };
                    sortorder: {
                        type: string;
                    };
                };
            };
            card: {
                type: string;
                required: string[];
                properties: {
                    cardid: {
                        $ref: string;
                    };
                    nodegroup_id: {
                        $ref: string;
                    };
                    graph_id: {
                        $ref: string;
                    };
                    name: {
                        $ref: string;
                    };
                    description: {
                        type: string;
                    };
                    instructions: {
                        $ref: string;
                    };
                    helptext: {
                        $ref: string;
                    };
                    helptitle: {
                        $ref: string;
                    };
                    active: {
                        type: string;
                    };
                    visible: {
                        type: string;
                    };
                    sortorder: {
                        type: string;
                    };
                    helpenabled: {
                        type: string;
                    };
                    cssclass: {
                        type: string;
                    };
                    is_editable: {
                        type: string;
                    };
                    config: {
                        type: string[];
                    };
                    constraints: {
                        type: string;
                    };
                };
            };
        };
        allOf: ({
            description: string;
            if: {
                properties: {
                    graph: {
                        type: string;
                        items: {
                            properties: {
                                nodes: {
                                    type: string;
                                    contains: {
                                        properties: {
                                            istopnode: {
                                                const: boolean;
                                            };
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
            then: {
                properties: {
                    graph: {
                        items: {
                            properties: {
                                nodes: {
                                    items: {
                                        if: {
                                            properties: {
                                                istopnode: {
                                                    const: boolean;
                                                };
                                            };
                                        };
                                        then: {
                                            properties: {
                                                nodegroup_id: {
                                                    type: string;
                                                };
                                            };
                                            required: string[];
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
            properties?: undefined;
            additionalProperties?: undefined;
        } | {
            description: string;
            properties: {
                graph: {
                    items: {
                        type: string;
                        properties: {
                            nodes: {
                                type: string;
                                items: {
                                    type: string;
                                    properties: {
                                        nodegroup_id: {
                                            oneOf: ({
                                                $ref: string;
                                                type?: undefined;
                                            } | {
                                                type: string;
                                                $ref?: undefined;
                                            })[];
                                        };
                                        istopnode: {
                                            type: string;
                                        };
                                    };
                                };
                            };
                            nodegroups: {
                                type: string;
                                items: {
                                    type: string;
                                    properties: {
                                        nodegroupid: {
                                            $ref: string;
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
            additionalProperties: boolean;
            if?: undefined;
            then?: undefined;
        })[];
    };
    businessData: {
        $schema: string;
        title: string;
        description: string;
        type: string;
        required: string[];
        properties: {
            business_data: {
                $ref: string;
            };
        };
        definitions: {
            uuid: {
                type: string;
                pattern: string;
            };
            businessData: {
                type: string;
                required: string[];
                properties: {
                    resources: {
                        type: string;
                        items: {
                            $ref: string;
                        };
                        minItems: number;
                    };
                };
            };
            resource: {
                type: string;
                required: string[];
                properties: {
                    resourceid: {
                        $ref: string;
                    };
                    graph_id: {
                        $ref: string;
                    };
                    legacyid: {
                        type: string[];
                    };
                    tiles: {
                        type: string;
                        items: {
                            $ref: string;
                        };
                        minItems: number;
                    };
                    resourceinstance: {
                        $ref: string;
                    };
                };
            };
            resourceInstance: {
                type: string;
                required: string[];
                properties: {
                    resourceinstanceid: {
                        $ref: string;
                    };
                    graph_id: {
                        $ref: string;
                    };
                    legacyid: {
                        type: string;
                        minLength: number;
                        description: string;
                    };
                    createdtime: {
                        type: string;
                        format: string;
                    };
                    name: {
                        type: string;
                        minLength: number;
                        description: string;
                    };
                    displayname: {
                        type: string;
                        minLength: number;
                        description: string;
                    };
                    map_popup: {
                        type: string;
                        description: string;
                    };
                    provisional: {
                        type: string;
                        enum: string[];
                    };
                    descriptors: {
                        type: string;
                        required: string[];
                        properties: {
                            name: {
                                type: string;
                                minLength: number;
                            };
                            description: {
                                type: string;
                                minLength: number;
                            };
                            map_popup: {
                                type: string;
                                minLength: number;
                            };
                            displayname: {
                                type: string;
                                minLength: number;
                            };
                        };
                        additionalProperties: boolean;
                        description: string;
                    };
                    graph_publication_id: {
                        $ref: string;
                    };
                    publication_id: {
                        $ref: string;
                    };
                };
            };
            tile: {
                type: string;
                required: string[];
                properties: {
                    tileid: {
                        $ref: string;
                    };
                    nodegroup_id: {
                        $ref: string;
                    };
                    parenttile_id: {
                        oneOf: ({
                            $ref: string;
                            type?: undefined;
                        } | {
                            type: string;
                            $ref?: undefined;
                        })[];
                    };
                    resourceid: {
                        oneOf: ({
                            $ref: string;
                            type?: undefined;
                        } | {
                            type: string;
                            $ref?: undefined;
                        })[];
                    };
                    sortorder: {
                        type: string;
                    };
                    tiles: {
                        type: string;
                        items: {
                            $ref: string;
                        };
                    };
                    provisionaledits: {
                        type: string[];
                    };
                    data: {
                        type: string;
                        patternProperties: {
                            "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$": {
                                oneOf: ({
                                    type: string;
                                    properties?: undefined;
                                } | {
                                    type: string;
                                    properties: {
                                        en: {
                                            type: string;
                                            required: string[];
                                            properties: {
                                                value: {
                                                    type: string;
                                                };
                                                direction: {
                                                    type: string;
                                                    enum: string[];
                                                };
                                            };
                                        };
                                    };
                                })[];
                            };
                        };
                        additionalProperties: boolean;
                        minProperties: number;
                    };
                };
            };
        };
    };
    graphsRegistry: {
        $schema: string;
        title: string;
        description: string;
        type: string;
        required: string[];
        properties: {
            models: {
                type: string;
                patternProperties: {
                    "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$": {
                        $ref: string;
                    };
                };
                additionalProperties: boolean;
                minProperties: number;
            };
        };
        definitions: {
            uuid: {
                type: string;
                pattern: string;
            };
            multilingualString: {
                type: string;
                patternProperties: {
                    "^[a-z]{2}(-[A-Z]{2})?$": {
                        type: string;
                    };
                };
                additionalProperties: boolean;
                minProperties: number;
            };
            modelReference: {
                type: string;
                required: string[];
                properties: {
                    id: {
                        $ref: string;
                    };
                    name: {
                        $ref: string;
                    };
                    slug: {
                        type: string;
                        pattern: string;
                        minLength: number;
                    };
                    subtitle: {
                        $ref: string;
                    };
                    color: {
                        type: string;
                        pattern: string;
                    };
                    iconclass: {
                        type: string;
                        pattern: string;
                    };
                };
            };
        };
    };
};
/**
 * Quick validation function for common use cases
 *
 * @example
 * ```typescript
 * import { quickValidate } from 'alizarin/validation';
 *
 * const result = quickValidate('./data');
 * if (result.success) {
 *   console.log('All files are valid!');
 * }
 * ```
 */
export { validateGraphLoading as quickValidate } from './validators/index.js';
