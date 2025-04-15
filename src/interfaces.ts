import { StaticCollection, StaticValue, StaticConcept, StaticTile, StaticGraph, StaticNode, StaticNodegroup, StaticResource } from "./static-types";

interface IRIVM {
  _: IInstanceWrapper,
  __: IModelWrapper,
}

interface IStringKeyedObject {
  [key: string]: any
}

interface IInstanceWrapper {
  resource;
  model;
  ensureNodegroup;
  setOrmAttribute;
  getOrmAttribute;
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
}

interface IViewModel {
  __parentPseudo: IPseudo | undefined;
  forJson(): {[key: string]: any} | {[key: string]: any}[];
}

export type { IInstanceWrapper, IModelWrapper, IRIVM, IStringKeyedObject, IReferenceDataManager, IViewModel, IPseudo, INodeConfig };
