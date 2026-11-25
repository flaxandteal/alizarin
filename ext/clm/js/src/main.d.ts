import type { interfaces } from "alizarin";
/**
 * A reference node's config, as `nodeConfigManager.retrieve` returns it. Built
 * from what the reference handler supplies:
 * `{controlledList, rdmCollection, multiValue, collectionId}`.
 */
declare class StaticNodeConfigReference implements interfaces.INodeConfig {
    private _config;
    constructor(config: StaticNodeConfigReference['_config']);
    get controlledList(): string;
    get rdmCollection(): string;
    get multiValue(): boolean;
    getCollectionId(): string | undefined;
}
/**
 * Set the base URI reference item URIs are built on, e.g. Arches'
 * `PUBLIC_SERVER_ADDRESS` + `plugins/controlled-list-manager/item/`: for the
 * view models here and, on the NAPI backend, for core's marker resolution. As
 * Python's `alizarin_clm.set_clm_base_uri`.
 */
declare function setClmBaseUri(uri: string): void;
/** The base URI set with {@link setClmBaseUri}, or null. */
declare function getClmBaseUri(): string | null;
/** Clear the base URI, back to clm's default. */
declare function clearClmBaseUri(): void;
export { StaticNodeConfigReference, setClmBaseUri, getClmBaseUri, clearClmBaseUri };
