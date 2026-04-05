import { IStringKeyedObject, IViewModel } from "../interfaces";
import { PseudoNode } from "../pseudos";
export declare class NodeViewModel implements IStringKeyedObject, IViewModel {
    [key: string | symbol]: any;
    then: undefined;
    [Symbol.toPrimitive]: undefined;
    __parentPseudo: PseudoNode;
    __parentWkrm: any | null;
    __forJsonCache(): null;
    constructor(parentPseudo: PseudoNode, parentWkrm: any | null);
    toString(): Promise<string>;
    __getEdgeTo(key: string): Promise<any>;
    __get(key: string): Promise<NodeViewModel>;
    static __create(pseudo: PseudoNode, parent: any | null): Promise<NodeViewModel>;
}
