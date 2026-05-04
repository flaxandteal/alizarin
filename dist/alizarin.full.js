var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
import { w as wasmReady, r as registerExtensionHandler, C as CUSTOM_DATATYPES, g as getCurrentLanguage, a as registerResolvableDatatype, n as nodeConfigManager, R as RDM } from "./main-DW5pY2OR.js";
import { A, K, G, p, o, S, L, N, c, I, J, T, k, y, Q, t, B, l, b, _, j, D, m, F, H, i, z, O, q, P, x, d, f, s, E, h, u, M, v, e } from "./main-DW5pY2OR.js";
class FileListItem {
  constructor(data) {
    __publicField(this, "accepted");
    __publicField(this, "alt_text");
    __publicField(this, "attribution");
    __publicField(this, "content");
    __publicField(this, "description");
    __publicField(this, "file_id");
    __publicField(this, "index");
    __publicField(this, "last_modified");
    __publicField(this, "name");
    __publicField(this, "path");
    __publicField(this, "selected");
    __publicField(this, "size");
    __publicField(this, "status");
    __publicField(this, "title");
    __publicField(this, "type");
    __publicField(this, "url");
    __publicField(this, "renderer");
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
  toDisplayString(lang) {
    var _a;
    const targetLang = lang || getCurrentLanguage() || "en";
    if (this.title) {
      if ((_a = this.title[targetLang]) == null ? void 0 : _a.value) {
        return this.title[targetLang].value;
      }
      for (const localized of Object.values(this.title)) {
        if (localized.value) {
          return localized.value;
        }
      }
    }
    if (this.name) {
      return this.name;
    }
    return this.file_id || "(unnamed file)";
  }
  /**
   * Get the alt text in a specific language.
   */
  getAltText(lang) {
    var _a;
    const targetLang = lang || getCurrentLanguage() || "en";
    if (this.alt_text) {
      if ((_a = this.alt_text[targetLang]) == null ? void 0 : _a.value) {
        return this.alt_text[targetLang].value;
      }
    }
    return null;
  }
  /**
   * Check if this is an image file based on MIME type.
   */
  isImage() {
    var _a;
    return ((_a = this.type) == null ? void 0 : _a.startsWith("image/")) ?? false;
  }
  /**
   * Convert to plain object for JSON serialization.
   */
  toJson() {
    const result = {
      name: this.name,
      accepted: this.accepted,
      selected: this.selected
    };
    if (this.alt_text) result.alt_text = this.alt_text;
    if (this.attribution) result.attribution = this.attribution;
    if (this.content) result.content = this.content;
    if (this.description) result.description = this.description;
    if (this.file_id) result.file_id = this.file_id;
    if (this.index !== void 0) result.index = this.index;
    if (this.last_modified) result.last_modified = this.last_modified;
    if (this.path) result.path = this.path;
    if (this.size !== void 0) result.size = this.size;
    if (this.status) result.status = this.status;
    if (this.title) result.title = this.title;
    if (this.type) result.type = this.type;
    if (this.url) result.url = this.url;
    if (this.renderer) result.renderer = this.renderer;
    return result;
  }
}
wasmReady.then(() => {
  registerExtensionHandler("file-list", {
    renderDisplay: (tileData, language) => {
      if (!tileData) return null;
      if (Array.isArray(tileData)) {
        if (tileData.length === 0) return null;
        const displayStrings = tileData.map((item) => {
          if (!item || typeof item !== "object") return null;
          const file = new FileListItem(item);
          return file.toDisplayString(language);
        }).filter((s2) => s2 !== null);
        return displayStrings.join(", ");
      }
      if (typeof tileData === "object" && tileData !== null) {
        const file = new FileListItem(tileData);
        return file.toDisplayString(language);
      }
      return null;
    }
  });
});
class FileItemViewModel extends String {
  constructor(file) {
    super(file.toDisplayString());
    __publicField(this, "_");
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_file");
    this._file = file;
  }
  async forJson() {
    return this._file.toJson();
  }
  getValue() {
    return this._file;
  }
  get name() {
    return this._file.name;
  }
  get url() {
    return this._file.url;
  }
  get file_id() {
    return this._file.file_id;
  }
  get fileType() {
    return this._file.type;
  }
  get size() {
    return this._file.size;
  }
  isImage() {
    return this._file.isImage();
  }
  getAltText(lang) {
    return this._file.getAltText(lang);
  }
  async __asTileData() {
    return this._file ? this._file.toJson() : null;
  }
}
class FileListViewModel extends Array {
  constructor() {
    super(...arguments);
    __publicField(this, "_");
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value", null);
  }
  async forJson() {
    const value = await this._value;
    if (!value) return null;
    return Promise.all(value.map(async (v2) => v2 ? v2.forJson() : null));
  }
  /**
   * Get only image files from the list.
   */
  async getImages() {
    const items = Array.from(this);
    const resolved = await Promise.all(items);
    return resolved.filter(
      (item) => item !== null && item.isImage()
    );
  }
  /**
   * Find a file by name.
   */
  async getByName(name) {
    const items = Array.from(this);
    const resolved = await Promise.all(items);
    return resolved.find(
      (item) => item !== null && item.name === name
    ) ?? null;
  }
  /**
   * Find a file by ID.
   */
  async getById(fileId) {
    const items = Array.from(this);
    const resolved = await Promise.all(items);
    return resolved.find(
      (item) => item !== null && item.file_id === fileId
    ) ?? null;
  }
  static async __create(tile, node, value, _cacheEntry = null) {
    const nodeid = node.nodeid;
    let val = [];
    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }
    if (value !== null && value !== void 0) {
      tile.data.set(nodeid, []);
      if (value instanceof Promise) {
        const resolved = await value;
        return FileListViewModel.__create(tile, node, resolved, _cacheEntry);
      }
      let items;
      if (Array.isArray(value)) {
        items = value;
      } else if (typeof value === "object") {
        items = [value];
      } else {
        throw new Error(
          `Cannot set file-list value on node ${nodeid} except via array or object: ${JSON.stringify(value)}`
        );
      }
      val = items.map((item, idx) => {
        if (item instanceof FileItemViewModel) {
          return item;
        }
        if (typeof item === "object" && item !== null) {
          const fileData = item;
          if (fileData.index === void 0) {
            fileData.index = idx;
          }
          const file = new FileListItem(fileData);
          return new FileItemViewModel(file);
        }
        return null;
      });
      Promise.all(val).then((vals) => {
        const tileData = vals.filter((v2) => v2 !== null).map((v2) => v2.getValue().toJson());
        tile.data.set(nodeid, tileData);
      });
    } else {
      val = [];
    }
    const viewModel = new FileListViewModel(...val);
    viewModel._value = Promise.all(val);
    return viewModel;
  }
  async __asTileData() {
    return this.forJson();
  }
}
class FileListDataType {
  static async __create(tile, node, value, _cacheEntry) {
    return FileListViewModel.__create(tile, node, value, _cacheEntry);
  }
}
CUSTOM_DATATYPES.set("file-list", FileListDataType);
function getReferenceValueFromCollection(collection, conceptId) {
  var _a;
  const concept = (_a = collection.__allConcepts) == null ? void 0 : _a[conceptId];
  if (!concept) {
    return null;
  }
  const labels = [];
  for (const [langId, prefLabel] of Object.entries(concept.prefLabels || {})) {
    const pLabel = prefLabel;
    labels.push({
      id: pLabel.id,
      language_id: langId,
      list_item_id: concept.id,
      value: pLabel.value,
      valuetype_id: "prefLabel"
    });
  }
  return {
    labels,
    list_id: collection.id,
    uri: concept.source || `http://localhost:8000/plugins/controlled-list-manager/item/${concept.id}`,
    id: concept.id
  };
}
function getReferenceValueByLabelFromCollection(collection, label) {
  const trimmedLabel = label.trim().toLowerCase();
  const values = collection.__values || {};
  const matchingValue = Object.values(values).find(
    (value) => {
      var _a;
      return ((_a = value.value) == null ? void 0 : _a.trim().toLowerCase()) === trimmedLabel;
    }
  );
  if (!matchingValue || !matchingValue.__concept) {
    return null;
  }
  return getReferenceValueFromCollection(collection, matchingValue.__concept.id);
}
function flattenToReferences(value) {
  const items = [];
  const visit = (val) => {
    if (!val) return;
    if (val._ref) {
      const ref = val._ref;
      const refObj = typeof ref.toJSON === "function" ? ref.toJSON() : ref;
      if (refObj && (refObj.labels || refObj.__needs_rdm_label_lookup)) {
        items.push(refObj);
      }
      return;
    }
    if (Array.isArray(val)) {
      val.forEach(visit);
      return;
    }
    if (val.__needs_rdm_label_lookup && val.label) {
      items.push(val);
      return;
    }
    if (val.labels) {
      items.push(val);
      return;
    }
    if (typeof val === "string") {
      try {
        visit(JSON.parse(val));
      } catch {
      }
    }
  };
  visit(value);
  return items;
}
function renderReferenceDisplay(data, language) {
  if (!data) return null;
  if (data.__needs_rdm_label_lookup && data.label) {
    return data.label;
  }
  if (data.labels && data.labels.length > 0) {
    const langPrefLabel = data.labels.find(
      (l2) => l2.language_id === language && l2.valuetype_id === "prefLabel"
    );
    if (langPrefLabel) return langPrefLabel.value;
    const langLabel = data.labels.find((l2) => l2.language_id === language);
    if (langLabel) return langLabel.value;
    const prefLabel = data.labels.find((l2) => l2.valuetype_id === "prefLabel");
    if (prefLabel) return prefLabel.value;
    return data.labels[0].value;
  }
  return null;
}
wasmReady.then(() => {
  registerExtensionHandler("reference", {
    renderDisplay: (tileData, language) => {
      if (!tileData) return null;
      const items = flattenToReferences(tileData);
      if (items.length === 0) return null;
      const displayStrings = items.map((ref) => renderReferenceDisplay(ref, language)).filter((s2) => s2 !== null);
      return displayStrings.join(", ");
    }
  });
});
class StaticReferenceLabel {
  constructor(label) {
    __publicField(this, "id");
    __publicField(this, "language_id");
    __publicField(this, "list_item_id");
    __publicField(this, "value");
    __publicField(this, "valuetype_id");
    this.id = label.id;
    this.language_id = label.language_id;
    this.list_item_id = label.list_item_id;
    this.value = label.value;
    this.valuetype_id = label.valuetype_id;
  }
  toJSON() {
    return {
      id: this.id,
      language_id: this.language_id,
      list_item_id: this.list_item_id,
      value: this.value,
      valuetype_id: this.valuetype_id
    };
  }
}
class StaticReference {
  constructor(reference) {
    __publicField(this, "labels");
    __publicField(this, "list_id");
    __publicField(this, "uri");
    this.list_id = reference.list_id;
    this.uri = reference.uri;
    this.labels = [];
    for (const label of reference.labels) {
      if (label instanceof StaticReferenceLabel) {
        this.labels.push(label);
      } else {
        this.labels.push(new StaticReferenceLabel(label));
      }
    }
  }
  toJSON() {
    return {
      labels: this.labels.map((l2) => l2.toJSON()),
      list_id: this.list_id,
      uri: this.uri
    };
  }
}
function referenceToString(reference) {
  if (reference.labels.length == 1) {
    return reference.labels[0].value;
  }
  let prefLabel;
  const lang = getCurrentLanguage();
  for (const label of reference.labels) {
    if (label.valuetype_id === "prefLabel") {
      prefLabel = label.value;
      if (label.language_id === lang) {
        return prefLabel;
      }
    }
  }
  return prefLabel || "(undefined)";
}
class ReferenceValueViewModel extends String {
  constructor(reference, pendingLookup, tile, nodeid, collectionId) {
    super(reference ? referenceToString(reference) : (pendingLookup == null ? void 0 : pendingLookup.type) === "label" ? pendingLookup.label : "(pending)");
    __publicField(this, "_");
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_ref");
    __publicField(this, "_pendingLookup", null);
    __publicField(this, "_resolvedRef", null);
    __publicField(this, "_resolutionPromise", null);
    __publicField(this, "_tile", null);
    __publicField(this, "_nodeid", null);
    __publicField(this, "_collectionId", null);
    this._ref = reference;
    this._pendingLookup = pendingLookup || null;
    this._tile = tile || null;
    this._nodeid = nodeid || null;
    this._collectionId = collectionId ?? ((pendingLookup == null ? void 0 : pendingLookup.collectionId) ?? null);
  }
  get [Symbol.toStringTag]() {
    return "ReferenceValue";
  }
  async getParent() {
    var _a;
    const ref = await this._resolvePending();
    if (!ref || !this._collectionId) {
      return null;
    }
    const conceptId = (_a = ref.labels[0]) == null ? void 0 : _a.list_item_id;
    if (!conceptId) {
      return null;
    }
    const collection = await RDM.retrieveCollection(this._collectionId);
    if (!collection.getParentId) {
      throw new Error(
        `Collection ${this._collectionId} does not support hierarchy lookups. Ensure WASM is initialized and the collection is a StaticCollection.`
      );
    }
    const parentId = collection.getParentId(conceptId);
    if (!parentId) {
      return null;
    }
    const parentRef = getReferenceValueFromCollection(collection, parentId);
    if (!parentRef) {
      return null;
    }
    return new ReferenceValueViewModel(new StaticReference(parentRef), void 0, void 0, void 0, this._collectionId);
  }
  /**
   * Get the parent reference value, if this reference has a parent in the hierarchy.
   * @returns A new ReferenceValueViewModel for the parent, or null if no parent
   * @throws Error if the collection doesn't support hierarchy lookups
   */
  get parent() {
    return this.getParent();
  }
  /**
   * Get all ancestor reference values, from immediate parent to root.
   * @returns Array of ReferenceValueViewModels for ancestors
   */
  async getAncestors() {
    const result = [];
    let current = this;
    while ((current = await current.parent) !== null) {
      result.push(current);
    }
    return result;
  }
  /**
   * Get all ancestor reference values, from immediate parent to root.
   * @returns Array of ReferenceValueViewModels for ancestors
   */
  get ancestors() {
    return this.getAncestors();
  }
  /**
   * Resolve any pending lookup. Called lazily when the resolved value is needed.
   */
  async _resolvePending() {
    if (this._resolvedRef) {
      return this._resolvedRef;
    }
    if (this._ref) {
      this._resolvedRef = this._ref;
      return this._ref;
    }
    if (!this._pendingLookup) {
      return null;
    }
    if (this._resolutionPromise) {
      return this._resolutionPromise;
    }
    this._resolutionPromise = (async () => {
      const lookup = this._pendingLookup;
      const collection = await RDM.retrieveCollection(lookup.collectionId);
      let val = null;
      if (lookup.type === "uuid") {
        val = getReferenceValueFromCollection(collection, lookup.uuid);
        if (!val) {
          console.error("Could not find reference for UUID", lookup.uuid, "in collection", lookup.collectionId);
        }
      } else if (lookup.type === "label") {
        val = getReferenceValueByLabelFromCollection(collection, lookup.label);
        if (!val) {
          console.error("Could not find reference for label", lookup.label, "in collection", lookup.collectionId);
        }
      }
      if (val) {
        this._resolvedRef = new StaticReference(val);
        if (this._tile && this._nodeid) {
          const currentData = this._tile.data.get(this._nodeid);
          if (!Array.isArray(currentData)) {
            this._tile.data.set(this._nodeid, this._resolvedRef.toJSON());
          }
        }
      }
      return this._resolvedRef;
    })();
    return this._resolutionPromise;
  }
  async forJson() {
    const ref = await this._resolvePending();
    return ref && typeof ref.toJSON === "function" ? ref.toJSON() : ref;
  }
  async getValue() {
    return this._resolvePending();
  }
  /**
   * Get the display string, resolving lazily if needed.
   * For sync contexts, returns the placeholder until resolved.
   */
  async getDisplay() {
    const ref = await this._resolvePending();
    return ref ? referenceToString(ref) : "(unresolved)";
  }
  // For JSON serialization, return the plain reference object (sync version)
  // IMPORTANT: This must return the reference object, NOT the string value,
  // even though ReferenceValueViewModel extends String.
  toJSON() {
    if (this._resolvedRef) {
      return this._resolvedRef.toJSON();
    }
    if (this._ref) {
      return this._ref.toJSON();
    }
    if (this._pendingLookup) {
      if (this._pendingLookup.type === "uuid") {
        return { __needs_rdm_lookup: true, uuid: this._pendingLookup.uuid };
      } else {
        return { __needs_rdm_label_lookup: true, label: this._pendingLookup.label, controlledList: this._pendingLookup.collectionId };
      }
    }
    return null;
  }
  // Convert to plain object for WASM tile data serialization
  async __asTileData() {
    const ref = await this._resolvePending();
    return ref && typeof ref.toJSON === "function" ? ref.toJSON() : ref;
  }
  static async __create(tile, node, value, _cacheEntry) {
    var _a, _b;
    const nodeid = node.nodeid;
    const collectionId = ((_a = node.config) == null ? void 0 : _a.controlledList) || ((_b = node.config) == null ? void 0 : _b.rdmCollection);
    if (!collectionId) {
      throw Error(`Node ${node.alias} (${node.nodeid}) missing controlledList or rdmCollection in config`);
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (value instanceof Promise) {
          return value.then((value2) => {
            return ReferenceValueViewModel.__create(tile, node, value2, _cacheEntry);
          });
        } else if (typeof value == "string") {
          if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
            value
          )) {
            const pendingLookup = { type: "uuid", uuid: value, collectionId };
            tile.data.set(nodeid, { __needs_rdm_lookup: true, uuid: value });
            return new ReferenceValueViewModel(null, pendingLookup, tile, nodeid, collectionId);
          } else {
            throw Error(
              `Set references using values from collections, not strings: ${value}`
            );
          }
        } else if (typeof value === "object" && value !== null && value.__needs_rdm_lookup && value.uuid) {
          const pendingLookup = { type: "uuid", uuid: value.uuid, collectionId };
          tile.data.set(nodeid, value);
          return new ReferenceValueViewModel(null, pendingLookup, tile, nodeid, collectionId);
        } else if (typeof value === "object" && value !== null && value.__needs_rdm_label_lookup && value.label) {
          const lookupCollectionId = value.controlledList || collectionId;
          const pendingLookup = { type: "label", label: value.label, collectionId: lookupCollectionId };
          tile.data.set(nodeid, value);
          return new ReferenceValueViewModel(null, pendingLookup, tile, nodeid, lookupCollectionId);
        } else if (Array.isArray(value) && value.length > 0 && "labels" in value[0]) {
          const ref = new StaticReference(value[0]);
          tile.data.set(nodeid, ref.toJSON());
          return new ReferenceValueViewModel(ref, void 0, void 0, void 0, collectionId);
        } else if (typeof value === "object" && value !== null && "labels" in value) {
          const ref = new StaticReference(value);
          tile.data.set(nodeid, ref.toJSON());
          return new ReferenceValueViewModel(ref, void 0, void 0, void 0, collectionId);
        } else {
          throw Error("Could not set reference from this data: " + JSON.stringify(value));
        }
      }
    }
    if (!tile || !value) {
      return null;
    }
    const str = new ReferenceValueViewModel(value, void 0, void 0, void 0, collectionId);
    return str;
  }
}
class ReferenceListViewModel extends Array {
  constructor() {
    super(...arguments);
    __publicField(this, "_");
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value", null);
  }
  get [Symbol.toStringTag]() {
    return "ReferenceList";
  }
  // Return comma-separated labels for string coercion (template rendering)
  toString() {
    return this.map((v2) => (v2 == null ? void 0 : v2.toString()) ?? "").filter((s2) => s2).join(", ");
  }
  // Allow string comparison to reference type
  includes(value) {
    return this.some((item) => item == value);
  }
  // For JSON serialization, return the array of plain reference objects
  toJSON() {
    const result = [];
    for (let i2 = 0; i2 < this.length; i2++) {
      const v2 = this[i2];
      if (v2 && v2._ref) {
        const ref = v2._ref;
        if (typeof ref.toJSON === "function") {
          result.push(ref.toJSON());
        } else {
          result.push(ref);
        }
      } else if (v2) {
        result.push(null);
      }
    }
    return result;
  }
  async forJson() {
    const value = await this._value;
    if (!value) return null;
    const results = await Promise.all(value.map((v2) => v2 ? v2.forJson() : null));
    return results.filter((r) => r !== null);
  }
  static async __create(tile, node, value, _cacheEntry = null) {
    const nodeid = node.nodeid;
    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }
    if (value === null) {
      const str2 = new ReferenceListViewModel();
      str2._value = Promise.resolve([]);
      return str2;
    }
    if (!Array.isArray(value)) {
      throw Error(
        `Cannot set an (entire) reference list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`
      );
    }
    const viewModelPromises = value.map((c2, _i) => {
      if (c2 instanceof ReferenceValueViewModel) {
        return Promise.resolve(c2);
      }
      return ReferenceValueViewModel.__create(tile, node, c2, {});
    });
    const resolvedViewModels = await Promise.all(viewModelPromises);
    const validViewModels = resolvedViewModels.filter((v2) => v2 !== null);
    const tileDataRefs = await Promise.all(
      validViewModels.map(async (vm) => {
        const ref = await vm.getValue();
        return ref && typeof ref.toJSON === "function" ? ref.toJSON() : ref;
      })
    );
    tile.data.set(nodeid, tileDataRefs);
    const str = new ReferenceListViewModel();
    str.push(...validViewModels);
    str._value = Promise.resolve(validViewModels);
    return str;
  }
  async __asTileData() {
    if (!this._value) return null;
    const values = await this._value;
    return Promise.all(values.map(async (v2) => {
      if (!v2) return null;
      return v2.__asTileData ? await v2.__asTileData() : v2;
    }));
  }
}
class ReferenceMergedDataType {
  static async __create(tile, node, value, _cacheEntry) {
    const config = nodeConfigManager.retrieve(node);
    if (config && config.multiValue) {
      return ReferenceListViewModel.__create(tile, node, value, _cacheEntry);
    }
    return ReferenceValueViewModel.__create(tile, node, value, _cacheEntry);
  }
}
CUSTOM_DATATYPES.set("reference", ReferenceMergedDataType);
registerResolvableDatatype("reference");
export {
  A as AlizarinModel,
  K as CollectionMutator,
  G as GraphManager,
  p as GraphMutator,
  RDM,
  o as ResourceModelWrapper,
  S as autoDetectBackend,
  L as buildGraphFromModelCsvs,
  N as buildResourcesFromBusinessCsv,
  c as client,
  I as collectionToSkosXml,
  J as collectionsToSkosXml,
  T as createResourceRegistry,
  k as createWKRM,
  y as ensureWasmRdmCache,
  Q as getBackend,
  t as getCurrentLanguage,
  B as getTimingStats,
  l as getWKRMClass,
  b as graphManager,
  _ as initWasm,
  j as interfaces,
  D as logTimingStats,
  m as nodeConfig,
  F as parseSkosXml,
  H as parseSkosXmlToCollection,
  registerExtensionHandler,
  registerResolvableDatatype,
  i as renderers,
  z as resetTimingStats,
  O as setBackend,
  q as setCurrentLanguage,
  P as setNapiModule,
  x as setWasmURL,
  d as slugify,
  f as staticStore,
  s as staticTypes,
  E as tracing,
  h as unregisterResolvableDatatype,
  u as utils,
  M as validateModelCsvs,
  v as version,
  e as viewModels,
  wasmReady
};
