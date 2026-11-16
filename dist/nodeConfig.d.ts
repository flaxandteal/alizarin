/**
 * Node configuration management.
 *
 * TypeScript wrappers around the Rust/WASM NodeConfigManager.
 * Config parsing and storage is done in Rust, shared with Python bindings.
 */
import { INodeConfig } from './interfaces';
import { StaticNode, StaticDomainValue, StaticGraph } from './static-types';
import { WasmNodeConfigBoolean, WasmNodeConfigConcept, WasmNodeConfigDomain } from '../pkg/alizarin';
declare class StaticNodeConfigBoolean implements INodeConfig {
    private _wasm;
    constructor(wasmConfig: WasmNodeConfigBoolean);
    get i18n_properties(): string[];
    get falseLabel(): {
        [key: string]: string;
    };
    get trueLabel(): {
        [key: string]: string;
    };
    getLabel(value: boolean, language: string): string | undefined;
}
declare class StaticNodeConfigConcept implements INodeConfig {
    private _wasm;
    constructor(wasmConfig: WasmNodeConfigConcept);
    get rdmCollection(): string;
}
/**
 * Builds an extension datatype's config from the config its registered handler
 * supplies (a plain object whose shape the extension defines).
 */
type NodeConfigAdapter = (config: any) => INodeConfig;
/** Register how an extension datatype's node config is presented. */
declare function registerNodeConfigAdapter(datatype: string, adapter: NodeConfigAdapter): void;
declare class StaticNodeConfigDomain implements INodeConfig {
    private _wasm;
    private _optionsCache;
    constructor(wasmConfig: WasmNodeConfigDomain);
    get i18n_config(): {
        [key: string]: string;
    };
    get options(): StaticDomainValue[];
    getSelected(): StaticDomainValue | undefined;
    valueFromId(id: string): StaticDomainValue | undefined;
}
type NodeConfigType = StaticNodeConfigBoolean | StaticNodeConfigConcept | StaticNodeConfigDomain | INodeConfig;
declare class NodeConfigManager {
    private _backendManager;
    private _cache;
    private _graphsLoaded;
    private getBackendManager;
    /** The backend-native manager (WASM or NAPI). Used by forDisplayJson. */
    get wasmManager(): any;
    loadFromGraph(graph: StaticGraph): void;
    retrieve(node: StaticNode): NodeConfigType | null;
    private getConfigFromWasm;
    clear(): void;
    hasGraph(graphId: string): boolean;
}
declare const nodeConfigManager: NodeConfigManager;
export { nodeConfigManager, StaticNodeConfigDomain, StaticNodeConfigBoolean, StaticNodeConfigConcept, NodeConfigManager, registerNodeConfigAdapter, };
export type { NodeConfigAdapter };
