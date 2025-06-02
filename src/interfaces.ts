import { StaticResource, StaticTile, StaticGraphMeta, StaticCollection, StaticNode, StaticNodegroup } from "./static-types";
import { AttrPromise } from "./utils";

type ResourceInstanceViewModelConstructor<T extends IRIVM<T>> =
  new (
    id: string,
    modelWrapper: IModelWrapper<T> | null,
    instanceWrapperFactory: ((rivm: T) => IInstanceWrapper<T>) | null,
    cacheEntry: object | null
  ) => T;

interface IRIVM<T extends IRIVM<T>> {
  [key: string]: any;
  id: string
  then: undefined;
  $: IInstanceWrapper<T> | null;
  __: IModelWrapper<T> | null;
  __parentPseudo: IPseudo | undefined;
}

interface IStringKeyedObject {
  [key: string | symbol]: any
}

type GetMeta = ((vm: IViewModel) => IStringKeyedObject) | undefined;
type CheckPermission = ((nodegroupId: string, tile: StaticTile | null, node: Map<string, StaticNode>) => boolean);

interface IViewModel {
  _: IViewModel | undefined | Promise<IViewModel | null>;
  __parentPseudo: IPseudo | undefined;
  forJson(): {[key: string]: any} | {[key: string]: any}[] | string | number | boolean | null;
  __forJsonCache(getMeta: GetMeta): IStringKeyedObject | null;
  // static __create(
  //   tile: StaticTile,
  //   node: StaticNode,
  //   value: any,
  //   cacheEntry: IViewModel | null
  // ): Promise<IViewModel | null>;
}

interface IInstanceWrapper<T extends IRIVM<T>> {
  resource: StaticResource | null | false;
  model: IModelWrapper<T>;
  loadNodes(aliases: Array<string>): Promise<void>;
  allEntries(): MapIterator<[string, Array<IPseudo> | false | null]>;
  addPseudo(childNode: StaticNode, tile: StaticTile | null, node: StaticNode): IPseudo;
  ensureNodegroup(
    allValues: Map<string, any>,
    allNodegroups: Map<string, boolean | Promise<any>>,
    nodegroupId: string | null,
    nodeObjs: Map<string, StaticNode>,
    nodegroupObjs: Map<string, StaticNodegroup>,
    edges: Map<string, string[]>,
    addIfMissing: boolean,
    tiles: StaticTile[] | null,
    doImpliedNodegroups: boolean
  ): Promise<[Map<string, any>, Set<string>]>;
  setOrmAttribute(key: string, value: any): Promise<void>;
  getOrmAttribute(key: string): Promise<any>;
  getValueCache(build: boolean, getMeta: GetMeta): Promise<{[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined>;
  getRoot(): Promise<IPseudo | undefined>;
  getRootViewModel(): Promise<IStringKeyedObject>;
  populate(lazy: boolean): Promise<void>;
};


class INodeConfig {
};

interface IWKRM {
  modelName: string;
  modelClassName: string;
  graphId: string;
  meta: StaticGraphMeta;
};

interface IModelWrapper<T extends IRIVM<T>> {
  all(params: { limit?: number; lazy?: boolean } | undefined): Promise<Array<T>>;
  getPermittedNodegroups(): Map<string | null, boolean | CheckPermission>;
  isNodegroupPermitted(nodegroupId: string, tile: StaticTile | null, nodes: Map<string, StaticNode>): boolean;
  getChildNodes(nodeId: string): Map<string, StaticNode>;
  getNodeObjectsByAlias(): Map<string, StaticNode>;
  getNodeObjects(): Map<string, StaticNode>;
  getNodegroupObjects(): Map<string, StaticNodegroup>;
  getEdges(): Map<string, string[]>;
  wkrm: IWKRM;
};

interface IReferenceDataManager {
  retrieveCollection(id: string): Promise<StaticCollection>;
};

interface IPseudo {
  parentNode: IPseudo | null;
  getValue(): AttrPromise<IViewModel | null | Array<IViewModel>>;
  forJson(): {[key: string]: any} | {[key: string]: any}[] | string | number | boolean | null;
  isIterable(): boolean;
  tile: any;
  node: any;
  describeField: () => string;
  describeFieldGroup: () => string;
}

interface ISemantic extends IViewModel {
  update(mapLike: Map<string, any> | {[key: string]: any}): void;
}

interface IGraphManager {
  getResource<T extends IRIVM<T>>(resourceId: string, lazy: boolean): Promise<T>;
}

export type { CheckPermission, ISemantic, ResourceInstanceViewModelConstructor, GetMeta, IInstanceWrapper, IModelWrapper, IRIVM, IStringKeyedObject, IReferenceDataManager, IViewModel, IPseudo, INodeConfig, IGraphManager };
