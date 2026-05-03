import { StaticNode as WasmStaticNode, StaticGraphMeta as WasmStaticGraphMeta, WasmRdmCache } from "../pkg/alizarin";
export declare function setWasmURL(url: string): void;
/**
 * Get the global WASM RDM cache singleton.
 * Creates the cache on first access (after WASM initialization).
 * @throws Error if WASM is not yet initialized
 */
export declare function getGlobalWasmRdmCache(): WasmRdmCache;
/**
 * Check if the global WASM RDM cache is available.
 */
export declare function hasGlobalWasmRdmCache(): boolean;
/**
 * Ensure the WASM RDM cache is initialized and ready.
 * This can be awaited early in the application lifecycle to ensure
 * the cache is available before any hierarchy methods are called.
 * @returns The initialized WasmRdmCache
 */
export declare function ensureWasmRdmCache(): Promise<WasmRdmCache>;
export declare function initWasm(): Promise<void>;
export { WasmStaticNode, WasmStaticGraphMeta, WasmRdmCache };
declare function _parseSkosXml(xmlContent: string, baseUri?: string): any;
declare function _parseSkosXmlToCollection(xmlContent: string, baseUri?: string): any;
declare function _collectionToSkosXml(collection: any): string;
declare function _collectionsToSkosXml(collections: any[]): string;
export { _parseSkosXml as parseSkosXml, _parseSkosXmlToCollection as parseSkosXmlToCollection, _collectionToSkosXml as collectionToSkosXml, _collectionsToSkosXml as collectionsToSkosXml, };
export { registerExtensionHandler } from "./backend";
