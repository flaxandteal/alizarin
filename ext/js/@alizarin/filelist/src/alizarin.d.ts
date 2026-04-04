// Type declarations for alizarin package
// The main alizarin package doesn't currently emit .d.ts files

declare module 'alizarin' {
  export const utils: {
    getCurrentLanguage(): string;
    setCurrentLanguage(lang: string): void;
    slugify(s: string): string;
  };

  export namespace interfaces {
    export interface IPseudo {
      describeField(): unknown;
      describeFieldGroup(): unknown;
    }
    export interface IViewModel {
      _: IViewModel | Promise<IViewModel> | undefined;
      __parentPseudo: IPseudo | undefined;
      describeField(): unknown;
      describeFieldGroup(): unknown;
    }
  }

  export namespace staticTypes {
    export interface StaticTile {
      data: Map<string, unknown>;
    }
    export interface StaticNode {
      nodeid: string;
      alias?: string;
      config?: {
        maxFiles?: number;
        maxFileSize?: number;
        activateMax?: boolean;
        imagesOnly?: boolean;
        [key: string]: unknown;
      };
    }
  }

  export const viewModels: {
    CUSTOM_DATATYPES: Map<string, unknown>;
    ResourceInstanceViewModel: unknown;
  };

  // WASM initialization
  export function initWasm(): Promise<void>;
  export const wasmReady: Promise<void>;
}
