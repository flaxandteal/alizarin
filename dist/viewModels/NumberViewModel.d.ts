import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
export declare class NumberViewModel extends Number implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    toString(): string;
    __forJsonCache(): null;
    forJson(): number;
    static __create(tile: StaticTile, node: StaticNode, value: any): NumberViewModel | Promise<NumberViewModel | null> | null;
    __asTileData(): number;
}
