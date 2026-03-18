// Type declarations for alizarin package
// The main alizarin package doesn't currently emit .d.ts files

declare module 'alizarin' {
  export const RDM: {
    retrieveCollection(collectionId: string): Promise<any>;
  };

  export const nodeConfig: {
    nodeConfigManager: {
      retrieve(node: any): any;
    };
  };

  export const utils: {
    getCurrentLanguage(): string;
    setCurrentLanguage(lang: string): void;
    slugify(s: string): string;
  };

  export namespace interfaces {
    export interface IPseudo {
      describeField(): any;
      describeFieldGroup(): any;
    }
    export interface IViewModel {
      _: IViewModel | Promise<IViewModel> | undefined;
      __parentPseudo: IPseudo | undefined;
      describeField(): any;
      describeFieldGroup(): any;
    }
  }

  export namespace staticTypes {
    export interface StaticTile {
      data: Map<string, any>;
    }
    export interface StaticNode {
      nodeid: string;
      alias?: string;
      config?: {
        controlledList?: string;
        rdmCollection?: string;
        multiValue?: boolean;
      };
    }
    export interface StaticCollection {
      id: string;
      // Internal concept/value lookups used by CLM helper functions
      __allConcepts?: { [conceptId: string]: any };
      __values?: { [valueId: string]: any };
    }
  }

  export const viewModels: {
    CUSTOM_DATATYPES: Map<string, any>;
    ResourceInstanceViewModel: any;
  };

  // Extension registry functions
  export function registerDisplaySerializer(
    datatype: string,
    callback: (tileData: any, language: string) => string | null
  ): void;

  export function hasDisplaySerializer(datatype: string): boolean;

  export function unregisterDisplaySerializer(datatype: string): void;

  export function getRegisteredDisplaySerializers(): string[];

  // WASM initialization
  export function initWasm(): Promise<void>;
  export const wasmReady: Promise<void>;
}
