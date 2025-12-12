import { IStringKeyedObject, IViewModel, IRIVM } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { SemanticViewModel } from "../semantic";
import { StaticTile, StaticNode } from "../static-types";
import { CUSTOM_DATATYPES } from "./types";
import {
  ConceptListCacheEntry,
  ConceptValueCacheEntry,
  ResourceInstanceCacheEntry,
  ResourceInstanceListCacheEntry,
} from "./cacheEntries";
import { ResourceInstanceViewModel } from "./ResourceInstanceViewModel";
import { ResourceInstanceListViewModel } from "./ResourceInstanceListViewModel";
import { StringViewModel } from "./StringViewModel";
import { DateViewModel } from "./DateViewModel";
import { GeoJSONViewModel } from "./GeoJSONViewModel";
import { EDTFViewModel } from "./EDTFViewModel";
import { NonLocalizedStringViewModel } from "./NonLocalizedStringViewModel";
import { NumberViewModel } from "./NumberViewModel";
import { BooleanViewModel } from "./BooleanViewModel";
import { UrlViewModel } from "./UrlViewModel";
import { DomainValueViewModel } from "./DomainValueViewModel";
import { DomainValueListViewModel } from "./DomainValueListViewModel";
import { ConceptValueViewModel } from "./ConceptValueViewModel";
import { ConceptListViewModel } from "./ConceptListViewModel";
import { FileListViewModel } from "./FileListViewModel";

export async function getViewModel<RIVM extends IRIVM<RIVM>>(
  parentPseudo: PseudoValue<any>,
  tile: StaticTile,
  node: StaticNode,
  data: any,
  parent: IRIVM<RIVM> | null,
  isInner: boolean = false
): Promise<IViewModel | null> {
  let vm;
  const cacheEntries: {[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined = parentPseudo.parent && parentPseudo.parent.$ ? await parentPseudo.parent.$.getValueCache(false, undefined) : undefined;
  let cacheEntry: IStringKeyedObject | null = null;
  if (cacheEntries) {
    cacheEntry = (tile.tileid ? (cacheEntries[tile.tileid] ?? {}) : {})[node.nodeid]
  };
  const datatype = isInner ? "semantic" : CUSTOM_DATATYPES.get(node.datatype) ?? node.datatype;

  let conceptCacheEntry: ConceptListCacheEntry | null;
  let conceptValueCacheEntry: ConceptValueCacheEntry | null;
  let resourceInstanceCacheEntry: ResourceInstanceCacheEntry | null;
  let resourceInstanceListCacheEntry: ResourceInstanceListCacheEntry | null;

  if (!(typeof datatype == "string")) {
    // @ts-expect-error Cannot make a static member part of the interface
    vm = await datatype.__create(tile, node, data, cacheEntry);
  } else {
    switch (datatype) {
      case "semantic":
        vm = await SemanticViewModel.__create(
          tile,
          node,
          data,
          parent,
        );
        break;
      case "domain-value":
        vm = await DomainValueViewModel.__create(tile, node, data);
        break;
      case "domain-value-list":
        vm = await DomainValueListViewModel.__create(tile, node, data);
        break;
      case "concept":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptValueCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          conceptValueCacheEntry = new ConceptValueCacheEntry(cacheEntry);
        } else {
          conceptValueCacheEntry = cacheEntry;
        }
        vm = await ConceptValueViewModel.__create(tile, node, data, conceptValueCacheEntry);
        break;
      case "resource-instance":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          resourceInstanceCacheEntry = new ResourceInstanceCacheEntry(cacheEntry);
        } else {
          resourceInstanceCacheEntry = cacheEntry;
        }
        vm = await ResourceInstanceViewModel.__create(tile, node, data, resourceInstanceCacheEntry);
        break;
      case "resource-instance-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceListCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          resourceInstanceListCacheEntry = new ResourceInstanceListCacheEntry(cacheEntry);
        } else {
          resourceInstanceListCacheEntry = cacheEntry;
        }
        vm = await ResourceInstanceListViewModel.__create(tile, node, data, resourceInstanceListCacheEntry);
        break;
      case "concept-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptListCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          conceptCacheEntry = new ConceptListCacheEntry(cacheEntry);
        } else {
          conceptCacheEntry = cacheEntry;
        }
        vm = await ConceptListViewModel.__create(tile, node, data, conceptCacheEntry);
        break;
      case "date":
        vm = await DateViewModel.__create(tile, node, data);
        break;
      case "geojson-feature-collection":
        vm = await GeoJSONViewModel.__create(tile, node, data);
        break;
      case "boolean":
        vm = await BooleanViewModel.__create(tile, node, data);
        break;
      case "string":
        vm = await StringViewModel.__create(tile, node, data);
        break
      case "number":
        vm = await NumberViewModel.__create(tile, node, data);
        break
      case "file-list":
        vm = await FileListViewModel.__create(tile, node, data);
        break;
      case "edtf":
        vm = await EDTFViewModel.__create(tile, node, data);
        break;
      case "url":
        vm = await UrlViewModel.__create(tile, node, data);
        break;
      case "non-localized-string":
        vm = await NonLocalizedStringViewModel.__create(tile, node, data);
        break;
      default:
        console.warn("Missing type for tile", tile.tileid, "on node", node.alias, "with type", node.datatype);
        vm = await NonLocalizedStringViewModel.__create(tile, node, data);
    }
  }

  if (vm === null) {
    return null;
  }

  vm.__parentPseudo = parentPseudo;
  if (vm instanceof Array) {
    for (const vme of vm) {
      if (vme instanceof Promise) {
        vme.then(vmep => { if (vmep !== null) vmep.__parentPseudo = parentPseudo; });
      } else {
        vme.__parentPseudo = parentPseudo;
      }
    }
  }

  return vm;
}
