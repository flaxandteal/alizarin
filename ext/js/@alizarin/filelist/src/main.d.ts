import type { interfaces, staticTypes } from "alizarin";
type IPseudo = interfaces.IPseudo;
type IViewModel = interfaces.IViewModel;
type StaticTile = staticTypes.StaticTile;
type StaticNode = staticTypes.StaticNode;
interface LocalizedStringValue {
    direction: string;
    value: string;
}
type LocalizedString = {
    [lang: string]: LocalizedStringValue;
};
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
declare class FileListItem {
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
    constructor(data: FileListItemData);
    /**
     * Get the display string for this file.
     * Uses title if available (in specified language), otherwise falls back to filename.
     */
    toDisplayString(lang?: string): string;
    /**
     * Get the alt text in a specific language.
     */
    getAltText(lang?: string): string | null;
    /**
     * Check if this is an image file based on MIME type.
     */
    isImage(): boolean;
    /**
     * Convert to plain object for JSON serialization.
     */
    toJson(): FileListItemData;
}
declare class FileItemViewModel extends String implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: IPseudo | undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    _file: FileListItem;
    constructor(file: FileListItem);
    forJson(): Promise<FileListItemData>;
    getValue(): FileListItem;
    get name(): string;
    get url(): string | undefined;
    get file_id(): string | undefined;
    get fileType(): string | undefined;
    get size(): number | undefined;
    isImage(): boolean;
    getAltText(lang?: string): string | null;
    __asTileData(): Promise<FileListItemData | null>;
}
declare class FileListViewModel extends Array<FileItemViewModel | Promise<FileItemViewModel | null> | null> implements IViewModel {
    _: IViewModel | Promise<IViewModel> | undefined;
    __parentPseudo: IPseudo | undefined;
    describeField: () => any;
    describeFieldGroup: () => any;
    _value: Promise<(FileItemViewModel | null)[]> | null;
    forJson(): Promise<(FileListItemData | null)[] | null>;
    /**
     * Get only image files from the list.
     */
    getImages(): Promise<FileItemViewModel[]>;
    /**
     * Find a file by name.
     */
    getByName(name: string): Promise<FileItemViewModel | null>;
    /**
     * Find a file by ID.
     */
    getById(fileId: string): Promise<FileItemViewModel | null>;
    static __create(tile: StaticTile, node: StaticNode, value: unknown, _cacheEntry?: object | null): Promise<FileListViewModel>;
    __asTileData(): Promise<(FileListItemData | null)[] | null>;
}
declare class FileListDataType {
    static __create(tile: StaticTile, node: StaticNode, value: unknown, _cacheEntry: object): Promise<FileListViewModel>;
}
export { FileListItem, FileItemViewModel, FileListViewModel, FileListDataType, };
export type { LocalizedStringValue, LocalizedString, FileListItemData, };
