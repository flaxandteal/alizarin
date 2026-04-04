import { utils, viewModels, registerExtensionHandler, wasmReady } from "alizarin";
import type { interfaces, staticTypes } from "alizarin";
type IPseudo = interfaces.IPseudo;
type IViewModel = interfaces.IViewModel;
type StaticTile = staticTypes.StaticTile;
type StaticNode = staticTypes.StaticNode;

// WASM initialization is deferred - registrations that need it use wasmReady.then()

// =============================================================================
// Localized String Types
// =============================================================================

interface LocalizedStringValue {
  direction: string;
  value: string;
}

type LocalizedString = { [lang: string]: LocalizedStringValue };

// =============================================================================
// FileListItem Type
// =============================================================================

interface FileListItemData {
  accepted?: boolean;
  alt_text?: LocalizedString;
  attribution?: LocalizedString;
  content?: string;
  description?: LocalizedString;
  file_id?: string;
  index?: number;
  last_modified?: number;
  name: string;
  path?: string;
  selected?: boolean;
  size?: number;
  status?: string;
  title?: LocalizedString;
  type?: string;
  url?: string;
  renderer?: string;
}

class FileListItem {
  accepted: boolean;
  alt_text?: LocalizedString;
  attribution?: LocalizedString;
  content?: string;
  description?: LocalizedString;
  file_id?: string;
  index?: number;
  last_modified?: number;
  name: string;
  path?: string;
  selected: boolean;
  size?: number;
  status?: string;
  title?: LocalizedString;
  type?: string;
  url?: string;
  renderer?: string;

  constructor(data: FileListItemData) {
    this.accepted = data.accepted ?? false;
    this.alt_text = data.alt_text;
    this.attribution = data.attribution;
    this.content = data.content;
    this.description = data.description;
    this.file_id = data.file_id;
    this.index = data.index;
    this.last_modified = data.last_modified;
    this.name = data.name || "";
    this.path = data.path;
    this.selected = data.selected ?? false;
    this.size = data.size;
    this.status = data.status;
    this.title = data.title;
    this.type = data.type;
    this.url = data.url;
    this.renderer = data.renderer;
  }

  /**
   * Get the display string for this file.
   * Uses title if available (in specified language), otherwise falls back to filename.
   */
  toDisplayString(lang?: string): string {
    const targetLang = lang || utils.getCurrentLanguage() || "en";

    // Try title first
    if (this.title) {
      if (this.title[targetLang]?.value) {
        return this.title[targetLang].value;
      }
      // Try any language
      for (const localized of Object.values(this.title)) {
        if (localized.value) {
          return localized.value;
        }
      }
    }

    // Fall back to filename
    if (this.name) {
      return this.name;
    }

    // Last resort
    return this.file_id || "(unnamed file)";
  }

  /**
   * Get the alt text in a specific language.
   */
  getAltText(lang?: string): string | null {
    const targetLang = lang || utils.getCurrentLanguage() || "en";

    if (this.alt_text) {
      if (this.alt_text[targetLang]?.value) {
        return this.alt_text[targetLang].value;
      }
    }
    return null;
  }

  /**
   * Check if this is an image file based on MIME type.
   */
  isImage(): boolean {
    return this.type?.startsWith("image/") ?? false;
  }

  /**
   * Convert to plain object for JSON serialization.
   */
  toJson(): FileListItemData {
    const result: FileListItemData = {
      name: this.name,
      accepted: this.accepted,
      selected: this.selected,
    };

    if (this.alt_text) result.alt_text = this.alt_text;
    if (this.attribution) result.attribution = this.attribution;
    if (this.content) result.content = this.content;
    if (this.description) result.description = this.description;
    if (this.file_id) result.file_id = this.file_id;
    if (this.index !== undefined) result.index = this.index;
    if (this.last_modified) result.last_modified = this.last_modified;
    if (this.path) result.path = this.path;
    if (this.size !== undefined) result.size = this.size;
    if (this.status) result.status = this.status;
    if (this.title) result.title = this.title;
    if (this.type) result.type = this.type;
    if (this.url) result.url = this.url;
    if (this.renderer) result.renderer = this.renderer;

    return result;
  }
}

// =============================================================================
// Display Serializers
// =============================================================================

// Register the file-list extension handler (after WASM is ready)
wasmReady.then(() => {
  registerExtensionHandler('file-list', {
    renderDisplay: (tileData: unknown, language: string) => {
      if (!tileData) return null;

      if (Array.isArray(tileData)) {
        if (tileData.length === 0) return null;

        const displayStrings = tileData.map((item: unknown) => {
          if (!item || typeof item !== 'object') return null;
          const file = new FileListItem(item as FileListItemData);
          return file.toDisplayString(language);
        }).filter((s): s is string => s !== null);

        return displayStrings.join(', ');
      }

      if (typeof tileData === 'object' && tileData !== null) {
        const file = new FileListItem(tileData as FileListItemData);
        return file.toDisplayString(language);
      }

      return null;
    },
  });
});

// =============================================================================
// View Models
// =============================================================================

class FileItemViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _file: FileListItem;

  constructor(file: FileListItem) {
    super(file.toDisplayString());
    this._file = file;
  }

  async forJson(): Promise<FileListItemData> {
    return this._file.toJson();
  }

  getValue(): FileListItem {
    return this._file;
  }

  get name(): string {
    return this._file.name;
  }

  get url(): string | undefined {
    return this._file.url;
  }

  get file_id(): string | undefined {
    return this._file.file_id;
  }

  get fileType(): string | undefined {
    return this._file.type;
  }

  get size(): number | undefined {
    return this._file.size;
  }

  isImage(): boolean {
    return this._file.isImage();
  }

  getAltText(lang?: string): string | null {
    return this._file.getAltText(lang);
  }

  async __asTileData(): Promise<FileListItemData | null> {
    return this._file ? this._file.toJson() : null;
  }
}

class FileListViewModel extends Array<FileItemViewModel | Promise<FileItemViewModel | null> | null> implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(FileItemViewModel | null)[]> | null = null;

  async forJson(): Promise<(FileListItemData | null)[] | null> {
    const value = await this._value;
    if (!value) return null;
    return Promise.all(value.map(async (v) => (v ? v.forJson() : null)));
  }

  /**
   * Get only image files from the list.
   */
  async getImages(): Promise<FileItemViewModel[]> {
    const items = Array.from(this);
    const resolved = await Promise.all(items);
    return resolved.filter((item): item is FileItemViewModel =>
      item !== null && item.isImage()
    );
  }

  /**
   * Find a file by name.
   */
  async getByName(name: string): Promise<FileItemViewModel | null> {
    const items = Array.from(this);
    const resolved = await Promise.all(items);
    return resolved.find((item): item is FileItemViewModel =>
      item !== null && item.name === name
    ) ?? null;
  }

  /**
   * Find a file by ID.
   */
  async getById(fileId: string): Promise<FileItemViewModel | null> {
    const items = Array.from(this);
    const resolved = await Promise.all(items);
    return resolved.find((item): item is FileItemViewModel =>
      item !== null && item.file_id === fileId
    ) ?? null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: unknown,
    _cacheEntry: object | null = null
  ): Promise<FileListViewModel> {
    const nodeid = node.nodeid;
    let val: (FileItemViewModel | Promise<FileItemViewModel | null> | null)[] = [];

    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }

    if (value !== null && value !== undefined) {
      tile.data.set(nodeid, []);

      // Handle awaitable value
      if (value instanceof Promise) {
        const resolved = await value;
        return FileListViewModel.__create(tile, node, resolved, _cacheEntry);
      }

      // Ensure we have an array
      let items: unknown[];
      if (Array.isArray(value)) {
        items = value;
      } else if (typeof value === 'object') {
        // Single file object - wrap in array
        items = [value];
      } else {
        throw new Error(
          `Cannot set file-list value on node ${nodeid} except via array or object: ${JSON.stringify(value)}`
        );
      }

      // Create FileItemViewModel for each item
      val = items.map((item, idx) => {
        if (item instanceof FileItemViewModel) {
          return item;
        }
        if (typeof item === 'object' && item !== null) {
          const fileData = item as FileListItemData;
          if (fileData.index === undefined) {
            fileData.index = idx;
          }
          const file = new FileListItem(fileData);
          return new FileItemViewModel(file);
        }
        return null;
      });

      // Update tile data with resolved values
      Promise.all(val).then((vals) => {
        const tileData = vals
          .filter((v): v is FileItemViewModel => v !== null)
          .map((v) => v.getValue().toJson());
        tile.data.set(nodeid, tileData);
      });
    } else {
      val = [];
    }

    const viewModel = new FileListViewModel(...val);
    viewModel._value = Promise.all(val) as Promise<(FileItemViewModel | null)[]>;
    return viewModel;
  }

  async __asTileData(): Promise<(FileListItemData | null)[] | null> {
    return this.forJson();
  }
}

// =============================================================================
// Factory / DataType Registration
// =============================================================================

class FileListDataType {
  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: unknown,
    _cacheEntry: object
  ): Promise<FileListViewModel> {
    return FileListViewModel.__create(tile, node, value, _cacheEntry);
  }
}

// Register with alizarin's custom datatypes
viewModels.CUSTOM_DATATYPES.set("file-list", FileListDataType);

// =============================================================================
// Exports
// =============================================================================

export {
  FileListItem,
  FileItemViewModel,
  FileListViewModel,
  FileListDataType,
};

export type {
  LocalizedStringValue,
  LocalizedString,
  FileListItemData,
};
