import { IStringKeyedObject, IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
export declare class GeoJSONViewModel implements IViewModel, IStringKeyedObject {
    [key: string | symbol]: any;
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: PseudoValue<any> | undefined;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    describeField: () => string;
    describeFieldGroup: () => string;
    _value: {
        [key: string]: any;
    };
    __forJsonCache(): null;
    constructor(jsonData: {
        [key: string]: any;
    });
    static __create(tile: StaticTile, node: StaticNode, value: any): GeoJSONViewModel | Promise<GeoJSONViewModel | null> | null;
    toString(): string;
    forJson(): Promise<{
        [key: string]: any;
    }>;
    __asTileData(): {
        [key: string]: any;
    };
}
