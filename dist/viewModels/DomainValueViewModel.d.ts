import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode, StaticDomainValue } from "../static-types";
export declare class DomainValueViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: StaticDomainValue | Promise<StaticDomainValue>;
    constructor(value: StaticDomainValue);
    forJson(): Promise<StaticDomainValue>;
    __forJsonCache(): null;
    getValue(): StaticDomainValue | Promise<StaticDomainValue>;
    lang(lang: string): Promise<string | undefined>;
    static __create(tile: StaticTile, node: StaticNode, value: any): Promise<DomainValueViewModel | null>;
    __asTileData(): Promise<string>;
}
