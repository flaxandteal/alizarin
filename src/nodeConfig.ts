import { INodeConfig } from './interfaces';
import { StaticNode, StaticDomainValue } from './static-types';

interface IStaticNodeConfigDomain {
  i18n_config: {[key: string]: string}
  options: StaticDomainValue[];
};

class StaticNodeConfigDomain implements IStaticNodeConfigDomain, INodeConfig {
  i18n_config: {[key: string]: string}
  options: StaticDomainValue[];

  getSelected() {
    return this.options.find(option => option.selected);
  }

  valueFromId(id: string) {
    return this.options.find(option => option.id == id);
  }

  constructor(jsonData: IStaticNodeConfigDomain) {
    this.i18n_config = jsonData.i18n_config;
    this.options = jsonData.options;
    if (this.options) {
      this.options = this.options.map(sdv => {
        if (!(sdv instanceof StaticDomainValue)) {
          return new StaticDomainValue(sdv);
        }
        return sdv;
      });
    }
  }
}

class NodeConfigManager {
  static _cache: Map<string, INodeConfig | null>
  cache: Map<string, INodeConfig | null>

  constructor(cache: Map<string, INodeConfig | null> | undefined = undefined) {
    if (!cache) {
      cache = NodeConfigManager._cache || new Map();
    }
    this.cache = cache;
  }

  retrieve(node: StaticNode) {
    if (this.cache.has(node.nodeid)) {
      return this.cache.get(node.nodeid);
    }
    let nodeConfig = null;
    switch (node.datatype) {
      case "domain-value-list":
      case "domain-value":
        // @ts-expect-error node.config is not typed
        nodeConfig = new StaticNodeConfigDomain(node.config);
        break;
      default:
    };
    this.cache.set(node.nodeid, nodeConfig);
    return nodeConfig;
  }
};

const nodeConfigManager = new NodeConfigManager();

export { nodeConfigManager, StaticNodeConfigDomain };
