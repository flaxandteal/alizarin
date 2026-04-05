import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { Url } from "./types";
export declare class UrlViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: Url;
    __forJsonCache(): null;
    constructor(value: Url);
    forJson(): {
        [key: string]: string;
    };
    label(): string;
    href(): string;
    static __create(tile: StaticTile, node: StaticNode, value: any): UrlViewModel | Promise<UrlViewModel | null> | null;
    __asTileData(): {
        [key: string]: string;
    };
}
