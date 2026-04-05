import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { StringTranslatedLanguage } from "./types";
export declare class StringViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: Map<string, StringTranslatedLanguage>;
    __forJsonCache(): null;
    constructor(value: Map<string, StringTranslatedLanguage>, language?: string | null);
    forJson(): string;
    lang(language: string): string;
    static __create(tile: StaticTile, node: StaticNode, value: any): StringViewModel | Promise<StringViewModel | null> | null;
    __asTileData(): Map<string, StringTranslatedLanguage>;
}
