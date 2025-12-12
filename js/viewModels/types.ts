import { IGraphManager, IViewModel } from "../interfaces";

export const DEFAULT_LANGUAGE = "en";

export class ViewContext {
  graphManager: IGraphManager | undefined
}

export const viewContext = new ViewContext();

// Map for custom datatypes registered by extensions
export const CUSTOM_DATATYPES: Map<string, string | IViewModel> = new Map();

// Helper class for multilingual strings
export class StringTranslatedLanguage {
  value: string = ""
}

// Helper class for URLs
export class Url {
  url: string
  url_label?: string

  constructor(url: string, url_label?: string) {
    this.url = url;
    this.url_label = url_label;
  }
}
