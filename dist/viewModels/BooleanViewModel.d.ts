import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { StaticNodeConfigBoolean } from '../nodeConfig';
export declare class BooleanViewModel extends Boolean implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    __config: StaticNodeConfigBoolean;
    describeField: () => string;
    describeFieldGroup: () => string;
    constructor(value: boolean, config: StaticNodeConfigBoolean);
    toString(lang?: string | undefined): string;
    __forJsonCache(): null;
    forJson(): boolean;
    static __create(tile: StaticTile, node: StaticNode, value: any): BooleanViewModel | Promise<BooleanViewModel | null> | null;
    __asTileData(): boolean;
}
