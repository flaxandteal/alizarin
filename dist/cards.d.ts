import { StaticNode } from "./static-types";
declare class CardComponent {
    id: string;
    name: string;
    constructor(id: string, name: string);
}
declare class Widget {
    id: string;
    name: string;
    datatype: string;
    defaultConfig: string;
    constructor(id: string, name: string, datatype: string, defaultConfig: string);
    getDefaultConfig(): {
        [key: string]: any;
    };
}
declare const DEFAULT_CARD_COMPONENT: CardComponent;
declare function getDefaultWidgetForNode(node: StaticNode, preferences?: {
    [key: string]: string;
}): Widget;
export { DEFAULT_CARD_COMPONENT, CardComponent, getDefaultWidgetForNode, Widget };
