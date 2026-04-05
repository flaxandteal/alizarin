import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
export declare class DateViewModel extends Date implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    __original: string;
    then: undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    __forJsonCache(): null;
    constructor(val: string);
    static __create(tile: StaticTile, node: StaticNode, value: any): DateViewModel | Promise<DateViewModel | null> | null;
    forJson(): Promise<string>;
    __asTileData(): string;
}
