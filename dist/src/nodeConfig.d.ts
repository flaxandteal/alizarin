import { INodeConfig } from './interfaces';
import { StaticNode, StaticDomainValue } from './static-types';
interface IStaticNodeConfigDomain {
    i18n_config: {
        [key: string]: string;
    };
    options: StaticDomainValue[];
}
interface IStaticNodeConfigBoolean {
    i18n_properties: string[];
    falseLabel: {
        [key: string]: string;
    };
    trueLabel: {
        [key: string]: string;
    };
}
declare class StaticNodeConfigBoolean implements IStaticNodeConfigBoolean, INodeConfig {
    i18n_properties: string[];
    falseLabel: {
        [key: string]: string;
    };
    trueLabel: {
        [key: string]: string;
    };
    constructor(jsonData: IStaticNodeConfigBoolean);
}
declare class StaticNodeConfigDomain implements IStaticNodeConfigDomain, INodeConfig {
    i18n_config: {
        [key: string]: string;
    };
    options: StaticDomainValue[];
    getSelected(): StaticDomainValue | undefined;
    valueFromId(id: string): StaticDomainValue | undefined;
    constructor(jsonData: IStaticNodeConfigDomain);
}
declare class NodeConfigManager {
    static _cache: Map<string, INodeConfig | null>;
    cache: Map<string, INodeConfig | null>;
    constructor(cache?: Map<string, INodeConfig | null> | undefined);
    retrieve(node: StaticNode): INodeConfig | StaticNodeConfigBoolean | StaticNodeConfigDomain | null | undefined;
}
declare const nodeConfigManager: NodeConfigManager;
export { nodeConfigManager, StaticNodeConfigDomain, StaticNodeConfigBoolean };
