import { RDM, nodeConfig, utils, viewModels, registerDisplaySerializer, wasmReady } from "alizarin";
import type { interfaces, staticTypes } from "alizarin";
type IPseudo = interfaces.IPseudo;
type IViewModel = interfaces.IViewModel;
type StaticTile = staticTypes.StaticTile;
type StaticNode = staticTypes.StaticNode;

// =============================================================================
// CLM Reference Helpers
// =============================================================================
// These functions convert SKOS concepts to CLM StaticReference format.
// They work with StaticCollection without modifying the core type.

interface StaticReferenceData {
  labels: Array<{
    id: string;
    language_id: string;
    list_item_id: string;
    value: string;
    valuetype_id: string;
  }>;
  list_id: string;
  uri: string;
  id: string;
}

/**
 * Get a CLM-compatible reference value by concept ID from a collection.
 */
function getReferenceValueFromCollection(collection: any, conceptId: string): StaticReferenceData | null {
  const concept = collection.__allConcepts?.[conceptId];
  if (!concept) {
    return null;
  }

  // Convert SKOS concept to CLM StaticReference format
  const labels = [];
  for (const [langId, prefLabel] of Object.entries(concept.prefLabels || {})) {
    const pLabel = prefLabel as any;
    labels.push({
      id: pLabel.id,
      language_id: langId,
      list_item_id: concept.id,
      value: pLabel.value,
      valuetype_id: 'prefLabel'
    });
  }

  return {
    labels,
    list_id: collection.id,
    uri: concept.source || `http://localhost:8000/plugins/controlled-list-manager/item/${concept.id}`,
    id: concept.id
  };
}

/**
 * Get a CLM-compatible reference value by label string from a collection.
 * Performs case-insensitive label matching with whitespace trimming.
 */
function getReferenceValueByLabelFromCollection(collection: any, label: string): StaticReferenceData | null {
  // Find concept by matching any prefLabel value (case-insensitive, trimmed)
  const trimmedLabel = label.trim().toLowerCase();
  const values = collection.__values || {};
  const matchingValue = Object.values(values).find(
    (value: any) => value.value?.trim().toLowerCase() === trimmedLabel
  ) as any;

  if (!matchingValue || !matchingValue.__concept) {
    return null;
  }

  return getReferenceValueFromCollection(collection, matchingValue.__concept.id);
}

// =============================================================================

// WASM initialization is deferred - registrations that need it use wasmReady.then()

// Helper to unwrap nested arrays/view models and find a reference object
function unwrapToReference(value: any): any {
  if (!value) return null;

  // If it's a ReferenceValueViewModel (extends String with _ref property)
  if (value._ref) {
    const ref = value._ref;
    // _ref might have toJSON method
    if (typeof ref.toJSON === 'function') {
      return ref.toJSON();
    }
    return ref;
  }

  // If it's a ReferenceListViewModel (extends Array), get first item
  if (Array.isArray(value)) {
    if (value.length === 0) return null;
    return unwrapToReference(value[0]);
  }

  // If it has labels, it's already a reference object
  if (value.labels) return value;

  // If it's a string, it might be JSON - try parsing
  if (typeof value === 'string') {
    try {
      return unwrapToReference(JSON.parse(value));
    } catch {
      return null;
    }
  }

  return value;
}

// Register display serializers after WASM is ready
wasmReady.then(() => {
  // Register the reference display serializer
  // This extracts display strings from StaticReference format for ETL templates
  registerDisplaySerializer('reference', (tileData: any, language: string) => {
    // Handle null/undefined
    if (!tileData) return null;

    // Unwrap nested arrays to find the actual reference object
    const data = unwrapToReference(tileData);
    if (!data) return null;

    // Handle __needs_rdm_label_lookup format (unresolved marker with label property)
    if (data.__needs_rdm_label_lookup && data.label) {
      return data.label;
    }

    // Extract display string from StaticReference format
    if (data.labels && data.labels.length > 0) {
      // First try to find a label matching the requested language with prefLabel
      const langPrefLabel = data.labels.find(
        (l: any) => l.language_id === language && l.valuetype_id === 'prefLabel'
      );
      if (langPrefLabel) return langPrefLabel.value;

      // Then try any label matching the language
      const langLabel = data.labels.find((l: any) => l.language_id === language);
      if (langLabel) return langLabel.value;

      // Then try any prefLabel
      const prefLabel = data.labels.find((l: any) => l.valuetype_id === 'prefLabel');
      if (prefLabel) return prefLabel.value;

      // Fall back to first label
      return data.labels[0].value;
    }

    return null;
  });

  // Also register for reference-list which may be serialized differently
  registerDisplaySerializer('reference-list', (tileData: any, language: string) => {
    if (!tileData) return null;

    // Handle potentially nested arrays/view models - flatten to get all reference items
    let items: any[] = [];
    const flatten = (val: any) => {
      if (!val) return;
      // Handle ReferenceValueViewModel (extends String with _ref property)
      if (val._ref) {
        const ref = val._ref;
        const refObj = typeof ref.toJSON === 'function' ? ref.toJSON() : ref;
        if (refObj && (refObj.labels || refObj.__needs_rdm_label_lookup)) {
          items.push(refObj);
        }
        return;
      }
      // Handle arrays (including ReferenceListViewModel)
      if (Array.isArray(val)) {
        val.forEach(flatten);
        return;
      }
      // Handle __needs_rdm_label_lookup format (unresolved marker)
      if (val.__needs_rdm_label_lookup && val.label) {
        items.push(val);
        return;
      }
      // Handle plain reference objects
      if (val.labels) {
        items.push(val);
        return;
      }
      // Handle JSON strings
      if (typeof val === 'string') {
        try {
          flatten(JSON.parse(val));
        } catch {}
      }
    };
    flatten(tileData);

    if (items.length === 0) return null;

    // Map each reference to its display string
    const displayStrings = items.map((ref: any) => {
      if (!ref) return null;

      // Handle __needs_rdm_label_lookup format
      if (ref.__needs_rdm_label_lookup && ref.label) {
        return ref.label;
      }

      // Handle resolved reference format
      if (!ref.labels || ref.labels.length === 0) return null;

      const langPrefLabel = ref.labels.find(
        (l: any) => l.language_id === language && l.valuetype_id === 'prefLabel'
      );
      if (langPrefLabel) return langPrefLabel.value;

      const langLabel = ref.labels.find((l: any) => l.language_id === language);
      if (langLabel) return langLabel.value;

      const prefLabel = ref.labels.find((l: any) => l.valuetype_id === 'prefLabel');
      if (prefLabel) return prefLabel.value;

      return ref.labels[0].value;
    }).filter((s: any) => s !== null);

    // Return as comma-separated string
    return displayStrings.join(', ');
  });
});

class StaticReferenceLabel {
  id: string
  language_id: string
  list_item_id: string
  value: string
  valuetype_id: string

  constructor(label: StaticReferenceLabel) {
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
};

class StaticReference {
  labels: StaticReferenceLabel[]
  list_id: string
  uri: string

  constructor(reference: StaticReference) {
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
      labels: this.labels.map(l => l.toJSON()),
      list_id: this.list_id,
      uri: this.uri
    };
  }
};

function referenceToString(reference: StaticReference): string {
  if (reference.labels.length == 1) {
    return reference.labels[0].value;
  }
  let prefLabel: string | undefined;
  const lang = utils.getCurrentLanguage();
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

// Types for pending lookups (lazy loading)
interface PendingUuidLookup {
  type: 'uuid';
  uuid: string;
  collectionId: string;
}

interface PendingLabelLookup {
  type: 'label';
  label: string;
  collectionId: string;
}

type PendingLookup = PendingUuidLookup | PendingLabelLookup;

class ReferenceValueViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _ref: StaticReference | null;
  _pendingLookup: PendingLookup | null = null;
  _resolvedRef: StaticReference | null = null;
  _resolutionPromise: Promise<StaticReference | null> | null = null;
  _tile: StaticTile | null = null;
  _nodeid: string | null = null;
  _collectionId: string | null = null;

  get [Symbol.toStringTag]() {
    return 'ReferenceValue';
  }

  constructor(reference: StaticReference | null, pendingLookup?: PendingLookup, tile?: StaticTile, nodeid?: string, collectionId?: string) {
    // If we have a resolved reference, use its display string; otherwise use placeholder
    super(reference ? referenceToString(reference) : (pendingLookup?.type === 'label' ? pendingLookup.label : '(pending)'));
    this._ref = reference;
    this._pendingLookup = pendingLookup || null;
    this._tile = tile || null;
    this._nodeid = nodeid || null;
    this._collectionId = collectionId ?? (pendingLookup?.collectionId ?? null);
  }

  async getParent(): Promise<ReferenceValueViewModel | null> {
      const ref = await this._resolvePending();
      if (!ref || !this._collectionId) {
        return null;
      }

      // Get the concept ID from one of the labels
      const conceptId = ref.labels[0]?.list_item_id;
      if (!conceptId) {
        return null;
      }

      const collection = await RDM.retrieveCollection(this._collectionId);
      if (!collection.getParentId) {
        throw new Error(
          `Collection ${this._collectionId} does not support hierarchy lookups. ` +
          'Ensure WASM is initialized and the collection is a StaticCollection.'
        );
      }

      const parentId = collection.getParentId(conceptId);
      if (!parentId) {
        return null; // Top-level concept
      }

      const parentRef = getReferenceValueFromCollection(collection, parentId);
      if (!parentRef) {
        return null;
      }

      return new ReferenceValueViewModel(new StaticReference(parentRef as any), undefined, undefined, undefined, this._collectionId);
  }

  /**
   * Get the parent reference value, if this reference has a parent in the hierarchy.
   * @returns A new ReferenceValueViewModel for the parent, or null if no parent
   * @throws Error if the collection doesn't support hierarchy lookups
   */
  get parent(): Promise<ReferenceValueViewModel | null> {
    return this.getParent();
  }

  /**
   * Get all ancestor reference values, from immediate parent to root.
   * @returns Array of ReferenceValueViewModels for ancestors
   */
  async getAncestors(): Promise<ReferenceValueViewModel[]> {
    const result: ReferenceValueViewModel[] = [];
    let current: ReferenceValueViewModel | null = this;

    while ((current = await current.parent) !== null) {
      result.push(current);
    }

    return result;
  }

  /**
   * Get all ancestor reference values, from immediate parent to root.
   * @returns Array of ReferenceValueViewModels for ancestors
   */
  get ancestors(): Promise<ReferenceValueViewModel[]> {
    return this.getAncestors();
  }

  /**
   * Resolve any pending lookup. Called lazily when the resolved value is needed.
   */
  private async _resolvePending(): Promise<StaticReference | null> {
    // Already resolved
    if (this._resolvedRef) {
      return this._resolvedRef;
    }

    // Already have a reference (no pending lookup)
    if (this._ref) {
      this._resolvedRef = this._ref;
      return this._ref;
    }

    // No pending lookup
    if (!this._pendingLookup) {
      return null;
    }

    // Avoid duplicate resolution
    if (this._resolutionPromise) {
      return this._resolutionPromise;
    }

    this._resolutionPromise = (async () => {
      const lookup = this._pendingLookup!;
      const collection = await RDM.retrieveCollection(lookup.collectionId);

      let val: StaticReferenceData | null = null;
      if (lookup.type === 'uuid') {
        val = getReferenceValueFromCollection(collection, lookup.uuid);
        if (!val) {
          console.error("Could not find reference for UUID", lookup.uuid, "in collection", lookup.collectionId);
        }
      } else if (lookup.type === 'label') {
        val = getReferenceValueByLabelFromCollection(collection, lookup.label);
        if (!val) {
          console.error("Could not find reference for label", lookup.label, "in collection", lookup.collectionId);
        }
      }

      if (val) {
        this._resolvedRef = new StaticReference(val as any);
        // Update tile data if we have tile reference
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

  async forJson(): Promise<any> {
    const ref = await this._resolvePending();
    // Return plain object for WASM compatibility
    return ref && typeof ref.toJSON === 'function' ? ref.toJSON() : ref;
  }

  async getValue(): Promise<StaticReference | null> {
    return this._resolvePending();
  }

  /**
   * Get the display string, resolving lazily if needed.
   * For sync contexts, returns the placeholder until resolved.
   */
  async getDisplay(): Promise<string> {
    const ref = await this._resolvePending();
    return ref ? referenceToString(ref) : '(unresolved)';
  }

  // For JSON serialization, return the plain reference object (sync version)
  // IMPORTANT: This must return the reference object, NOT the string value,
  // even though ReferenceValueViewModel extends String.
  toJSON(): any {
    // If we have a resolved ref, use it
    if (this._resolvedRef) {
      return this._resolvedRef.toJSON();
    }
    // If we have an original ref, use it
    if (this._ref) {
      return this._ref.toJSON();
    }
    // If pending lookup, return the marker so it can be resolved later
    if (this._pendingLookup) {
      if (this._pendingLookup.type === 'uuid') {
        return { __needs_rdm_lookup: true, uuid: this._pendingLookup.uuid };
      } else {
        return { __needs_rdm_label_lookup: true, label: this._pendingLookup.label, controlledList: this._pendingLookup.collectionId };
      }
    }
    return null;
  }

  // Convert to plain object for WASM tile data serialization
  async __asTileData(): Promise<any> {
    const ref = await this._resolvePending();
    return ref && typeof ref.toJSON === 'function' ? ref.toJSON() : ref;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    _cacheEntry: object
  ): Promise<ReferenceValueViewModel | null> {
    const nodeid = node.nodeid;
    const collectionId = node.config?.controlledList || node.config?.rdmCollection;
    if (!collectionId) {
      throw Error(`Node ${node.alias} (${node.nodeid}) missing controlledList or rdmCollection in config`);
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (value instanceof Promise) {
          return value.then((value) => {
            return ReferenceValueViewModel.__create(tile, node, value, _cacheEntry);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            // UUID string - create lazy lookup (no fetch yet)
            const pendingLookup: PendingUuidLookup = { type: 'uuid', uuid: value, collectionId };
            // Store the marker in tile data for now
            tile.data.set(nodeid, { __needs_rdm_lookup: true, uuid: value });
            return new ReferenceValueViewModel(null, pendingLookup, tile, nodeid, collectionId);
          } else {
            throw Error(
              `Set references using values from collections, not strings: ${value}`,
            );
          }
        } else if (typeof value === "object" && value !== null && value.__needs_rdm_lookup && value.uuid) {
          // UUID marked for RDM lookup by the coercion layer - create lazy lookup (no fetch yet)
          const pendingLookup: PendingUuidLookup = { type: 'uuid', uuid: value.uuid, collectionId };
          // Keep the marker in tile data
          tile.data.set(nodeid, value);
          return new ReferenceValueViewModel(null, pendingLookup, tile, nodeid, collectionId);
        } else if (typeof value === "object" && value !== null && value.__needs_rdm_label_lookup && value.label) {
          // Label string marked for RDM lookup - create lazy lookup (no fetch yet)
          const lookupCollectionId = value.controlledList || collectionId;
          const pendingLookup: PendingLabelLookup = { type: 'label', label: value.label, collectionId: lookupCollectionId };
          // Keep the marker in tile data
          tile.data.set(nodeid, value);
          return new ReferenceValueViewModel(null, pendingLookup, tile, nodeid, lookupCollectionId);
        } else if (Array.isArray(value) && value.length > 0 && "labels" in value[0]) {
          // Handle array of pre-formatted reference values from business data
          // For now, just use the first value
          const ref = new StaticReference(value[0]);
          tile.data.set(nodeid, ref.toJSON());
          return new ReferenceValueViewModel(ref, undefined, undefined, undefined, collectionId);
        } else if (typeof value === "object" && value !== null && "labels" in value) {
          // Handle single pre-formatted reference value from business data
          const ref = new StaticReference(value);
          tile.data.set(nodeid, ref.toJSON());
          return new ReferenceValueViewModel(ref, undefined, undefined, undefined, collectionId);
        } else {
          throw Error("Could not set reference from this data: " + JSON.stringify(value));
        }
      }
    }

    if (!tile || !value) {
      return null;
    }
    const str = new ReferenceValueViewModel(value, undefined, undefined, undefined, collectionId);
    return str;
  }
}

class ReferenceListViewModel extends Array<ReferenceValueViewModel> implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ReferenceValueViewModel | null)[]> | null = null;

  get [Symbol.toStringTag]() {
    return 'ReferenceList';
  }

  // Return comma-separated labels for string coercion (template rendering)
  toString(): string {
    return this.map(v => v?.toString() ?? '').filter(s => s).join(', ');
  }

  // Allow string comparison to reference type
  includes(value: ReferenceValueViewModel | string): boolean {
    return this.some(item => item == value);
  }

  // For JSON serialization, return the array of plain reference objects
  toJSON(): any[] {
    // Use Array.from to ensure we iterate properly over this Array subclass
    const result: any[] = [];
    for (let i = 0; i < this.length; i++) {
      const v = this[i] as ReferenceValueViewModel;
      if (v && v._ref) {
        // Access _ref directly - it's a StaticReference with toJSON method
        const ref = v._ref;
        if (typeof (ref as any).toJSON === 'function') {
          result.push((ref as any).toJSON());
        } else {
          result.push(ref);
        }
      } else if (v) {
        // Fallback: if it's a ReferenceValueViewModel (extends String),
        // avoid returning the string primitive
        result.push(null);
      }
    }
    return result;
  }

  async forJson(): Promise<any[] | null> {
    const value = await this._value;
    if (!value) return null;
    // Await each forJson call since they're async
    const results = await Promise.all(value.map((v) => (v ? v.forJson() : null)));
    // Filter out nulls and return flat array of reference objects
    return results.filter(r => r !== null);
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    _cacheEntry: object | null = null
  ): Promise<ReferenceListViewModel> {
    const nodeid = node.nodeid;
    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }

    if (value === null) {
      const str = new ReferenceListViewModel();
      str._value = Promise.resolve([]);
      return str;
    }

    if (!Array.isArray(value)) {
      throw Error(
        `Cannot set an (entire) reference list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`,
      );
    }

    // Create all view models (may be promises)
    const viewModelPromises = value.map((c, _i) => {
      if (c instanceof ReferenceValueViewModel) {
        return Promise.resolve(c);
      }
      return ReferenceValueViewModel.__create(tile, node, c, {});
    });

    // Await all view models to resolve
    const resolvedViewModels = await Promise.all(viewModelPromises);
    const validViewModels = resolvedViewModels.filter((v): v is ReferenceValueViewModel => v !== null);

    // Convert to plain objects for tile data
    const tileDataRefs = await Promise.all(
      validViewModels.map(async (vm) => {
        const ref = await vm.getValue();
        return ref && typeof ref.toJSON === 'function' ? ref.toJSON() : ref;
      })
    );

    // Set tile data with resolved plain objects
    tile.data.set(nodeid, tileDataRefs);

    // Create the list view model with resolved values
    const str = new ReferenceListViewModel();
    str.push(...validViewModels);
    str._value = Promise.resolve(validViewModels);
    return str;
  }

  async __asTileData() {
    if (!this._value) return null;
    const values = await this._value;
    // Convert each ReferenceValueViewModel to plain object for WASM compatibility
    return Promise.all(values.map(async (v) => {
      if (!v) return null;
      return v.__asTileData ? await v.__asTileData() : v;
    }));
  }
}

class ReferenceMergedDataType {
  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    _cacheEntry: object
  ): Promise<ReferenceValueViewModel | ReferenceListViewModel | null> {
    const config = nodeConfig.nodeConfigManager.retrieve(node);
    if (config && config.multiValue) {
      return ReferenceListViewModel.__create(tile, node, value, _cacheEntry);
    }
    return ReferenceValueViewModel.__create(tile, node, value, _cacheEntry);
  }
}

viewModels.CUSTOM_DATATYPES.set("reference", ReferenceMergedDataType);
