import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
export declare class NonLocalizedStringViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    __forJsonCache(): null;
    forJson(): string;
    static __create(tile: StaticTile, node: StaticNode, value: any): NonLocalizedStringViewModel | Promise<NonLocalizedStringViewModel | null> | null;
    __asTileData(): string;
}
