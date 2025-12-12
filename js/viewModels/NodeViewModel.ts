import { IStringKeyedObject, IViewModel } from "../interfaces";
import { StaticEdge } from "../static-types";
import { AttrPromise } from "../utils";
import { PseudoNode } from "../pseudos";

export class NodeViewModel implements IStringKeyedObject, IViewModel {
  [key: string | symbol]: any;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  __parentPseudo: PseudoNode;
  __parentWkrm: any | null;

  __forJsonCache(): null {
    return null;
  }

  constructor(
    parentPseudo: PseudoNode,
    parentWkrm: any | null,
  ) {
    this.__parentPseudo = parentPseudo;
    this.__parentWkrm = parentWkrm;
    return new Proxy(this, {
      set: (object, key, value) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          object[key] = value;
        } else if (k.startsWith("__") || k in object) {
          object[k] = value;
        } else {
          throw new Error("Cannot set values on a node");
        }
        return true;
      },
      get: (object, key) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;

        if (key.toString() === "Symbol.toStringTag") {
          return () => this.constructor.name;
        }

        if (key in object) {
          return object[key];
        } else if (k.startsWith("__") || k in object) {
          return object[k];
        } else if (k == "_") {
          return this.__parentPseudo.node;
        } else if (k.endsWith("$edge")) {
          return this.__getEdgeTo(k.substring(0, k.length - 5));
        }
        if (k == "length") {
          return object.__parentPseudo.size;
        }
        return new AttrPromise((resolve) => {
          object.__get(k).then(resolve);
        });
      },
    });
  }

  async toString(): Promise<string> {
    if (!this.__parentPseudo) {
      return "[NodeViewModel]";
    }
    const alias = this.__parentPseudo.alias;
    return alias || "[unnamed]";
  }

  async __getEdgeTo(key: string) {
    const childNode = this.__parentPseudo.childNodes.get(key);

    if (!childNode) {
      throw new Error(`Child node key ${key} missing`);
    }

    const domainId = this.__parentPseudo.nodeid;
    const rangeId = childNode.nodeid;
    const edges = this.__parentWkrm.graph.edges.filter(
      (edge: StaticEdge) => edge.domainnode_id === domainId && edge.rangenode_id === rangeId
    );
    if (edges.length !== 1) {
      throw new Error(`Number of edges from ${domainId}->${rangeId} != 1`);
    }
    return edges[0];
  }

  async __get(key: string) {
    const pseudo = this.__parentWkrm.createPseudoNodeChild(key, this.__parentPseudo);
    return NodeViewModel.__create(pseudo, this.__parentWkrm);
  }

  static async __create(
    pseudo: PseudoNode,
    parent: any | null,
  ): Promise<NodeViewModel> {
    const node = new NodeViewModel(pseudo, parent);
    return node;
  }
}
