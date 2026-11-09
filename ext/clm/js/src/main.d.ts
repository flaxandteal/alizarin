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
export { StaticNodeConfigReference };
