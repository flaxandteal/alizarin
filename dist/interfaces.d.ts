import { StaticResource, StaticTile, StaticGraphMeta, StaticCollection, StaticNode, StaticNodegroup } from "./static-types";
import { AttrPromise } from "./utils";
type ResourceInstanceViewModelConstructor<T extends IRIVM<T>> = new (id: string, modelWrapper: IModelWrapper<T> | null, instanceWrapperFactory: ((rivm: T) => IInstanceWrapper<T>) | null, cacheEntry: object | null) => T;
interface IRIVM<T extends IRIVM<T>> {
    [key: string]: any;
    id: string;
    then: undefined;
    $: IInstanceWrapper<T> | null;
    __: IModelWrapper<T> | null;
    __parentPseudo: IPseudo | undefined;
}
interface IStringKeyedObject {
    [key: string | symbol]: any;
}
type GetMeta = ((vm: IViewModel) => IStringKeyedObject) | undefined;
/**
 * Conditional permission rule for filtering tiles by data values.
 * Tiles are permitted if the value at `path` is in the `allowed` set.
 */
interface ConditionalPermission {
    /** JSON path to evaluate (e.g., ".data.uuid.field.name") */
    path: string;
    /** Set of allowed values - tile is permitted if value at path is in this set */
    allowed: string[];
}
/** Permission value: boolean for simple allow/deny, or conditional rule */
type PermissionValue = boolean | ConditionalPermission;
interface IViewModel {
    _: IViewModel | undefined | Promise<IViewModel | null>;
    __parentPseudo: IPseudo | undefined;
    forJson(): {
        [key: string]: any;
    } | {
        [key: string]: any;
    }[] | string | number | boolean | null;
    __forJsonCache(getMeta: GetMeta): IStringKeyedObject | null;
}
interface IInstanceWrapper<T extends IRIVM<T>> {
    resource: StaticResource | null | false;
    model: IModelWrapper<T>;
    loadNodes(aliases: Array<string>): Promise<void>;
    allEntries(): MapIterator<[string, Array<IPseudo> | false | null]>;
    addPseudo(childNode: StaticNode, tile: StaticTile | null, node: StaticNode): IPseudo;
    setOrmAttribute(key: string, value: any): Promise<void>;
    getOrmAttribute(key: string): Promise<any>;
    getValueCache(build: boolean, getMeta: GetMeta): Promise<{
        [tileId: string]: {
            [nodeId: string]: IStringKeyedObject;
        };
    } | undefined>;
    getRoot(): Promise<IPseudo | undefined>;
    getRootViewModel(): Promise<IStringKeyedObject>;
    populate(lazy: boolean): Promise<void>;
    retrievePseudo(key: string, dflt?: any, raiseError?: boolean): Promise<Array<IPseudo> | null>;
    hasPseudo(key: string): Promise<boolean>;
    setPseudo(key: string, value: any): void;
    setDefaultPseudo(key: string, value: any): Promise<any>;
}
declare class INodeConfig {
}
interface IWKRM {
    modelName: string;
    modelClassName: string;
    graphId: string;
    meta: StaticGraphMeta;
}
/**
 * Backend interface for graph schema operations.
 * Implemented by both WASMResourceModelWrapper and NapiResourceModelWrapper.
 *
 * This is the contract that the TS ResourceModelWrapper delegates to.
 * WASM returns JS Maps from getters; NAPI returns plain objects.
 * The TS layer normalizes to Maps via the ResourceModelWrapper property getters.
 */
interface IModelWrapperBackend {
    graph: any;
    readonly nodes: Map<string, StaticNode> | Record<string, StaticNode> | null;
    readonly nodesByAlias: Map<string, StaticNode> | Record<string, StaticNode> | null;
    readonly edges: Map<string, string[]> | Record<string, string[]> | null;
    readonly nodegroups: Map<string, StaticNodegroup> | Record<string, StaticNodegroup> | null;
    buildNodes(): void;
    getGraphId(): string;
    getRootNode(): StaticNode;
    getNodeObjects(): Map<string, StaticNode> | Record<string, StaticNode>;
    getNodeObjectsByAlias(): Map<string, StaticNode> | Record<string, StaticNode>;
    getNodeObjectFromAlias(alias: string): StaticNode;
    getNodeObjectFromId(id: string): StaticNode;
    getChildNodes(nodeId: string): Map<string, StaticNode> | Record<string, StaticNode>;
    getChildNodeAliases(nodeId: string): string[];
    getNodeIdFromAlias(alias: string): string;
    getEdges(): Map<string, string[]> | Record<string, string[]>;
    getNodegroupObjects(): Map<string, StaticNodegroup> | Record<string, StaticNodegroup>;
    getNodegroupIds(): string[];
    getNodegroupName(nodegroupId: string): string;
    setPermittedNodegroups(permissions: any): void;
    getPermittedNodegroups(): Map<string, boolean> | Record<string, boolean>;
    isNodegroupPermitted(nodegroupId: string, tile?: StaticTile | null): boolean;
    setDefaultAllowAllNodegroups?(defaultAllow: boolean): void;
    pruneGraph(keepFunctions?: string[]): void;
    createPseudoNode?(alias?: string | null): any;
    createPseudoNodeChild?(childNode: string, parent: any): any;
    createPseudoValue?(alias: string | null | undefined, tile: any, parent: any): any;
}
interface IModelWrapper<T extends IRIVM<T>> {
    all(params: {
        limit?: number;
        lazy?: boolean;
    } | undefined): Promise<Array<T>>;
    getPermittedNodegroups(): Map<string | null, PermissionValue>;
    isNodegroupPermitted(nodegroupId: string, tile: StaticTile | null): boolean;
    getChildNodes(nodeId: string): Map<string, StaticNode>;
    getNodeObjectsByAlias(): Map<string, StaticNode>;
    getNodeObjects(): Map<string, StaticNode>;
    getNodegroupObjects(): Map<string, StaticNodegroup>;
    getEdges(): Map<string, string[]>;
    wkrm: IWKRM;
    getNodegroupName(nodegroupId: string): string;
    createPseudoValue(alias: string | null | undefined, tile: any, parent: any): any;
}
interface IReferenceDataManager {
    retrieveCollection(id: string): Promise<StaticCollection>;
}
interface IPseudo {
    parentValue: IPseudo | null;
    getValue(): AttrPromise<IViewModel | null | Array<IViewModel>>;
    forJson(): {
        [key: string]: any;
    } | {
        [key: string]: any;
    }[] | string | number | boolean | null;
    isIterable(): boolean;
    tile: any;
    node: any;
    describeField: () => string;
    describeFieldGroup: () => string;
}
interface ISemantic extends IViewModel {
    update(mapLike: Map<string, any> | {
        [key: string]: any;
    }): void;
}
interface IGraphManager {
    getResource<T extends IRIVM<T>>(resourceId: string, lazy: boolean): Promise<T>;
}
export type { ConditionalPermission, PermissionValue, ISemantic, ResourceInstanceViewModelConstructor, GetMeta, IInstanceWrapper, IModelWrapper, IModelWrapperBackend, IRIVM, IWKRM, IStringKeyedObject, IReferenceDataManager, IViewModel, IPseudo, INodeConfig, IGraphManager };
