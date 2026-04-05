import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { DomainValueViewModel } from "./DomainValueViewModel";
export declare class DomainValueListViewModel extends Array implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: Promise<(DomainValueViewModel | null)[]> | null;
    forJson(): Promise<Promise<import("../static-types").StaticDomainValue>[]>;
    __forJsonCache(): null;
    static __create(tile: StaticTile, node: StaticNode, value: any): Promise<DomainValueListViewModel>;
    __asTileData(): Promise<DomainValueViewModel[]>;
}
