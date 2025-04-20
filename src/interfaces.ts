import { StaticCollection, StaticValue, StaticConcept, StaticTile, StaticGraph, StaticNode, StaticNodegroup, StaticResource } from "./static-types";

interface IRIVM {
  id: string
  then: undefined;
  _: IInstanceWrapper,
  __: IModelWrapper,
  __parentPseudo: IPseudo | undefined;
  forJson(): {[key: string]: any} | {[key: string]: any}[];
}

interface IStringKeyedObject {
  [key: string]: any
}

type GetMeta = ((vm: IViewModel) => IStringKeyedObject) | undefined;

interface IViewModel {
  __parentPseudo: IPseudo | undefined;
  forJson(): {[key: string]: any} | {[key: string]: any}[];
  __forJsonCache(GetMeta): IStringKeyedObject | null;
}

interface IInstanceWrapper {
  resource;
  model;
  ensureNodegroup;
  setOrmAttribute;
  getOrmAttribute;
  getValueCache(build: boolean, getMeta: GetMeta): Promise<{[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined>;
  getRoot;
};

class INodeConfig {
};

interface IModelWrapper {
  getPermittedNodegroups(): Map<string, StaticNodegroup>;
  getChildNodes(nodeId: string): Map<string, StaticNode>;
  getNodeObjectsByAlias(): Map<string, StaticNode>;
  getNodegroupObjects(): Map<string, StaticNodegroup>;
  wkrm;
};

interface IReferenceDataManager {
  retrieveCollection(id: string): Promise<StaticCollection>;
};

interface IPseudo {
  parentNode: IPseudo | null;
  getValue(): IViewModel | null;
  tile: any;
  node: any;
  describeField: () => string;
  describeFieldGroup: () => string;
}

interface IGraphManager {
  getResource(slug: string, lazy: boolean): IRIVM;
}

export type { GetMeta, IInstanceWrapper, IModelWrapper, IRIVM, IStringKeyedObject, IReferenceDataManager, IViewModel, IPseudo, INodeConfig, IGraphManager };
