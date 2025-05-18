import { staticStore } from "./staticStore.ts"
import { PseudoList } from "./pseudos.ts"
import { DateViewModel, ResourceInstanceViewModel, DomainValueViewModel, ConceptValueViewModel, NonLocalizedStringViewModel, StringViewModel, SemanticViewModel, GeoJSONViewModel, BooleanViewModel, NumberViewModel } from './viewModels';

class Cleanable extends String {
  __clean: string | undefined
}

abstract class BaseRenderer {
  async render(asset: ResourceInstanceViewModel<any>) {
    if (!asset._) {
      throw Error("Cannot render unloaded asset - do you want to await asset.retrieve()?");
    }
    const root = await (await asset._.getRootViewModel());
    return this.renderValue(root, 0);
  }

  abstract renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any>;
  abstract renderDate(value: DateViewModel, _depth: number): Promise<any>;
  abstract renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any>;
  abstract renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any>;
  abstract renderSemantic(value: SemanticViewModel, depth: number): Promise<any>;
  abstract renderBlock(block: {[key: string]: string} | {[key: string]: string}[], depth: number): any;
  abstract renderArray(value: any, depth: number): Promise<any>;
  abstract renderString(value: String, _depth: number): Promise<any>;
  abstract renderBoolean(value: Boolean, _depth: number): Promise<any>;
  abstract renderNumber(value: Number, _depth: number): Promise<any>;

  async renderValue(value: any, depth: number): Promise<any> {
    let newValue;
    if (value instanceof Promise) {
      value = await value;
    }
    if (value instanceof DomainValueViewModel) {
      newValue = this.renderDomainValue(value, depth);
    } else if (value instanceof DateViewModel) {
      newValue = this.renderDate(value, depth);
    } else if (value instanceof ConceptValueViewModel) {
      newValue = this.renderConceptValue(value, depth);
    } else if (value instanceof ResourceInstanceViewModel) {
      newValue = this.renderResourceReference(value, depth);
    } else if (value instanceof SemanticViewModel) {
      newValue = this.renderSemantic(value, depth);
    } else if (value instanceof Array) {
      newValue = this.renderArray(value, depth);
    } else if (value instanceof StringViewModel || value instanceof NonLocalizedStringViewModel) {
      newValue = this.renderString(value, depth);
    } else if (value instanceof BooleanViewModel) {
      newValue = this.renderBoolean(value, depth);
    } else if (value instanceof NumberViewModel) {
      newValue = this.renderNumber(value, depth);
    } else if (value instanceof GeoJSONViewModel) {
      newValue = this.renderBlock(await value.forJson(), depth);
    } else if (value instanceof Object) {
      newValue = this.renderBlock(value, depth);
    } else {
      newValue = value;
    }
    return newValue;
  }
}

class Renderer extends BaseRenderer {
  async renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any> {
    return value;
  }

  async renderString(value: String, _depth: number): Promise<any> {
    return `${value}`;
  }

  async renderNumber(value: Number, _depth: number): Promise<any> {
    return value.toString();
  }

  async renderBoolean(value: Boolean, _depth: number): Promise<any> {
    return value.toString();
  }

  async renderDate(value: DateViewModel, _depth: number): Promise<any> {
    return value;
  }

  async renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any> {
    return value;
  }

  async renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any> {
    return value;
  }

  async renderSemantic(value: SemanticViewModel, depth: number): Promise<any> {
    return this.renderBlock(await value.toObject(), depth);
  }

  renderBlock(block: {[key: string]: string} | {[key: string]: string}[], depth: number): any {
    const renderedBlock: {[key: string]: any} = {};
    const promises: Promise<void>[] = [];
    for (const [key, value] of Object.entries(block)) {
      promises.push(
        this.renderValue(value, depth + 1).then((val: any) => { renderedBlock[key] = val; })
      );
    }
    return Promise.all(promises).then(() => renderedBlock);
  }

  async renderArray(value: any, depth: number): Promise<any> {
      return Promise.all(value.map((val: any) => this.renderValue(val, depth + 1)));
  }

}

class MarkdownRenderer extends Renderer {
  conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined
  dateToText: ((value: DateViewModel) => string) | undefined
  domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined
  resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined
  nodeToUrl: ((value: string) => string) | undefined

  constructor(callbacks: {
    conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined,
    dateToText: ((value: DateViewModel) => string) | undefined,
    domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined,
    resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined,
    nodeToUrl: ((value: string) => string) | undefined,
  }) {
    super();
    this.conceptValueToUrl = callbacks.conceptValueToUrl;
    this.dateToText = callbacks.dateToText;
    this.domainValueToUrl = callbacks.domainValueToUrl;
    this.resourceReferenceToUrl = callbacks.resourceReferenceToUrl;
    this.nodeToUrl = callbacks.nodeToUrl;
  }


  override async renderDomainValue(domainValue: DomainValueViewModel, _: number): Promise<any> {
    const value = await domainValue.getValue();
    const url = this.domainValueToUrl ? await this.domainValueToUrl(domainValue): null;
    const text = url ? `[${value.toString()}](${url.trim()})` : value.toString();
    const wrapper = new Cleanable(`
    <span
      class='alizarin-domain-value' data-id='${value.id}'
    >
      ${text}
    </span>`.replace(/\n/g, ' ').trim());
    wrapper.__clean = domainValue.toString();
    return wrapper;
  }

  override async renderDate(date: DateViewModel, _: number): Promise<any> {
    const value = await date;
    const text = this.dateToText ? await this.dateToText(value): value.toISOString();
    const wrapper = new Cleanable(`
    <time datetime='${text}'>
      ${text}
    </time>`.replace(/\n/g, ' ').trim());
    wrapper.__clean = text;
    return wrapper;
  }

  override async renderConceptValue(conceptValue: ConceptValueViewModel, _: number): Promise<any> {
    const value = await conceptValue.getValue();
    const url = this.conceptValueToUrl ? await this.conceptValueToUrl(conceptValue): null;
    const text = url ? `[${value.value}](${url.trim()})` : value.value;
    const wrapper = new Cleanable(`
    <span
      class='alizarin-concept-value' data-id='${value.id}'
      data-concept-id='${value.__concept ? value.__concept.id : ""}'
      data-concept-ref='$${value.__concept ? value.__concept.source : ""}'
    >
      ${text}
    </span>`.replace(/\n/g, ' ').trim());
    wrapper.__clean = conceptValue.toString();
    return wrapper;
  }

  override async renderResourceReference(rivm: ResourceInstanceViewModel<any>, _: number): Promise<any> {
    const value = await rivm.forJson(false);
    const url = this.resourceReferenceToUrl ? await this.resourceReferenceToUrl(rivm): null;
    let title = value.title || value.type || 'Resource';
    const text = url ? `[${title}](${url.trim()})` : title;
    const resourceMetadata = await staticStore.getMeta(value.id);
    if (resourceMetadata) {
      title = resourceMetadata.name;
    }
    const wrapper = new Cleanable(`
    <span
      class='alizarin-resource-instance alizarin-related-resource' data-id='${value.id}'
      data-graph-id='${value.graphId}'
    >
      ${text}
    </span>`.replace(/\n/g, ' ').trim());
    wrapper.__clean = rivm.toString();
    return wrapper;
  }
}

class FlatMarkdownRenderer extends MarkdownRenderer {
  override async renderSemantic(vm: SemanticViewModel, depth: number): Promise<any> {
    const children = [...(await vm.__getChildValues()).entries()].map(([_, v]) => [v.node.alias, v.node]);
    const nodes = Object.fromEntries(await Promise.all(children));
    return super.renderSemantic(vm, depth).then(async block => {
      const text = [
        `* <span class='node-type'>${vm.__node.name}</span> &rarr;`,
        ...Object.entries(await block).map(([key, value]) => {
          const node = nodes[key];
          let nodeName = node.name;
          if (this.nodeToUrl) {
            nodeName = `[${node.name}](${this.nodeToUrl(node)})`;
          }
          if ((typeof value == 'string' || value instanceof String) && value.indexOf('\n') != -1) {
            return `  * <span class='node-name'>${nodeName}</span> <span class='node-alias'>[*${node.alias}*]</span>:<span class='node-value'>\n${value.split('\n').map(x => `    ${x}`).join('\n')}\n    </span>`;
          } else {
            return `  * <span class='node-name'>${nodeName}</span> <span class='node-alias'>[*${node.alias}*]</span>: <span class='node-value'>${value}</span>`;
          }
        }).join('\n').split('\n')
      ];
      if (text[1] == '') {
        text[0] += `<span class='node-empty'>&lt;empty&gt;</span>`;
        text.pop();
      }
      return text.map(line => `  ${line}`).join('\n');
    });
  }

  override async renderArray(value: any, depth: number): Promise<any> {
      const rows = await super.renderArray(value, depth);
      if (value instanceof PseudoList || value.indexOf('\n') != -1) {
        return rows.map(x => `${x}`).join('\n');
      } else {
        return rows.join(", ");
      }
  }

  async renderString(value: String, _depth: number): Promise<any> {
    if (value.indexOf('\n') != -1) {
      return '\n    ' + value.split('\n').join('\n    ');
    }
  }
}

class JsonRenderer extends Renderer {
  async renderDate(value: DateViewModel, _depth: number): Promise<any> {
    return value.forJson();
  }

  async renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any> {
    return value.forJson();
  }

  async renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any> {
    return value.forJson();
  }

  async renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any> {
    return value.forJson();
  }
}

export { MarkdownRenderer, JsonRenderer, Cleanable, FlatMarkdownRenderer };
