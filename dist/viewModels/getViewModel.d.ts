import { IViewModel, IRIVM } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
export declare function getViewModel<RIVM extends IRIVM<RIVM>>(parentPseudo: PseudoValue<any>, tile: StaticTile, node: StaticNode, data: any, parent: IRIVM<RIVM> | null, isInner?: boolean): Promise<IViewModel | null>;
