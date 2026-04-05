import { IGraphManager, IViewModel } from "../interfaces";
export declare const DEFAULT_LANGUAGE = "en";
export declare class ViewContext {
    graphManager: IGraphManager | undefined;
}
export declare const viewContext: ViewContext;
export declare const CUSTOM_DATATYPES: Map<string, string | IViewModel>;
export declare class StringTranslatedLanguage {
    value: string;
}
export declare class Url {
    url: string;
    url_label?: string;
    constructor(url: string, url_label?: string);
}
