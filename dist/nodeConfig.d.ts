/**
 * Node configuration management.
 *
 * TypeScript wrappers around the Rust/WASM NodeConfigManager.
 * Config parsing and storage is done in Rust, shared with Python bindings.
 */
import { INodeConfig } from './interfaces';
import { StaticNode, StaticDomainValue, StaticGraph } from './static-types';
import { WasmNodeConfigBoolean, WasmNodeConfigConcept, WasmNodeConfigDomain, WasmNodeConfigReference } from '../pkg/alizarin';
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
declare class StaticNodeConfigReference implements INodeConfig {
    private _wasm;
    constructor(wasmConfig: WasmNodeConfigReference);
    get controlledList(): string;
    get rdmCollection(): string;
    get multiValue(): boolean;
    getCollectionId(): string | undefined;
}
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
type NodeConfigType = StaticNodeConfigBoolean | StaticNodeConfigConcept | StaticNodeConfigDomain | StaticNodeConfigReference;
declare class NodeConfigManager {
    private _wasmManager;
    private _cache;
    private _graphsLoaded;
    private getWasmManager;
    loadFromGraph(graph: StaticGraph): void;
    retrieve(node: StaticNode): NodeConfigType | null;
    private getConfigFromWasm;
    clear(): void;
    hasGraph(graphId: string): boolean;
}
declare const nodeConfigManager: NodeConfigManager;
export { nodeConfigManager, StaticNodeConfigDomain, StaticNodeConfigBoolean, StaticNodeConfigConcept, StaticNodeConfigReference, NodeConfigManager, };
