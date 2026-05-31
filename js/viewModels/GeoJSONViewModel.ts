import { IStringKeyedObject, IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";

export class GeoJSONViewModel implements IViewModel, IStringKeyedObject {
  [key: string | symbol]: any;
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: { [key: string]: any };

  __forJsonCache(): null {
    return null;
  }

  constructor(jsonData: { [key: string]: any }) {
    this._value = jsonData;
    return new Proxy(this, {
      get: (object: GeoJSONViewModel, key) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          return object[key];
        } else if (k in object) {
          return object[k];
        }
        return this._value[k];
      },
      set: (object: GeoJSONViewModel, key, value) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          object[key] = value;
        } else if (k in object) {
          object[k] = value;
        } else {
          this._value[k] = value;
        }
        return true;
      },
    });
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): GeoJSONViewModel | Promise<GeoJSONViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) =>
        GeoJSONViewModel.__create(tile, node, value),
      );
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    if (!(val instanceof Object)) {
      throw new Error("GeoJSON should be a JSON object");
    }
    const str = new GeoJSONViewModel(val);
    return str;
  }

  toString(): string {
    const val = this._value;
    const type = val?.type || 'GeoJSON';
    if (type === 'FeatureCollection' && Array.isArray(val.features)) {
      const geomTypes = new Set(
        val.features
          .map((f: any) => f?.geometry?.type)
          .filter(Boolean)
      );
      if (geomTypes.size > 0) {
        return `${type} (${[...geomTypes].join(', ')})`;
      }
    } else if (type === 'Feature' && val.geometry?.type) {
      return `${type} (${val.geometry.type})`;
    }
    return type;
  }

  async forJson() {
    return await this._value;
  }

  __asTileData() {
    // Validate on write path (producing tile data for save)
    const warnings = validateGeoJSONCoordinates(this._value);
    for (const w of warnings) {
      console.warn(`GeoJSON: ${w}`);
    }
    return this._value;
  }
}

/** Check coordinate positions for out-of-range lat/lng. */
function checkCoordinateRanges(coords: any, warnings: string[]): void {
  if (!Array.isArray(coords)) return;
  // A position is [lng, lat, ...] — array of numbers
  if (coords.length >= 2 && typeof coords[0] === 'number' && typeof coords[1] === 'number') {
    const lng = coords[0];
    const lat = coords[1];
    if (lng < -180 || lng > 180) {
      warnings.push(`GeoJSON coordinate longitude ${lng} is outside valid range (-180..180)`);
    }
    if (lat < -90 || lat > 90) {
      warnings.push(`GeoJSON coordinate latitude ${lat} is outside valid range (-90..90)`);
    }
  } else {
    // Nested array — recurse (LineString, Polygon rings, etc.)
    for (const item of coords) {
      checkCoordinateRanges(item, warnings);
    }
  }
}

/** Recursively validate coordinates in a GeoJSON object. */
function validateGeoJSONCoordinates(value: any): string[] {
  if (!value || typeof value !== 'object') return [];
  const warnings: string[] = [];
  const geoType = value.type;
  if (geoType === 'FeatureCollection' && Array.isArray(value.features)) {
    for (const f of value.features) {
      warnings.push(...validateGeoJSONCoordinates(f));
    }
  } else if (geoType === 'Feature' && value.geometry) {
    warnings.push(...validateGeoJSONCoordinates(value.geometry));
  } else if (geoType === 'GeometryCollection' && Array.isArray(value.geometries)) {
    for (const g of value.geometries) {
      warnings.push(...validateGeoJSONCoordinates(g));
    }
  } else if (geoType && value.coordinates) {
    checkCoordinateRanges(value.coordinates, warnings);
  }
  return warnings;
}
