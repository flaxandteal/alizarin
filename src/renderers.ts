import { staticStore } from "./staticStore.ts"
import { DateViewModel, ResourceInstanceViewModel, DomainValueViewModel, ConceptValueViewModel, StringViewModel, SemanticViewModel, GeoJSONViewModel } from './viewModels';

class Cleanable extends String {
  __clean: string | undefined
}

class Renderer {
  async render(asset: ResourceInstanceViewModel<any>) {
    if (!asset._) {
      throw Error("Cannot render unloaded asset - do you want to await asset.retrieve()?");
    }
    const root = await (await asset._.getRootViewModel());
    return this.renderValue(root);
  }

  async renderDomainValue(value: DomainValueViewModel): Promise<any> {
    return value;
  }

  async renderDate(value: DateViewModel): Promise<any> {
    return value;
  }

  async renderConceptValue(value: ConceptValueViewModel): Promise<any> {
    return value;
  }

  async renderResourceReference(value: ResourceInstanceViewModel<any>): Promise<any> {
    return value;
  }

  renderBlock(block: {[key: string]: string} | {[key: string]: string}[]) {
    const renderedBlock: {[key: string]: any} = {};
    const promises: Promise<void>[] = [];
    for (const [key, value] of Object.entries(block)) {
      promises.push(
        this.renderValue(value).then((val: any) => { renderedBlock[key] = val; })
      );
    }
    return Promise.all(promises).then(() => renderedBlock);
  }

  async renderValue(value: any): Promise<any> {
    let newValue;
    if (value instanceof Promise) {
      value = await value;
    }
    if (value instanceof DomainValueViewModel) {
      newValue = this.renderDomainValue(value);
    } else if (value instanceof DateViewModel) {
      newValue = this.renderDate(value);
    } else if (value instanceof ConceptValueViewModel) {
      newValue = this.renderConceptValue(value);
    } else if (value instanceof ResourceInstanceViewModel) {
      newValue = this.renderResourceReference(value);
    } else if (value instanceof SemanticViewModel) {
      newValue = this.renderBlock(await value.toObject());
    } else if (value instanceof Array) {
      newValue = Promise.all(value.map(val => this.renderValue(val)));
    } else if (value instanceof StringViewModel) {
      newValue = `${value}`;
    } else if (value instanceof GeoJSONViewModel) {
      newValue = this.renderBlock(await value.forJson());
    } else if (value instanceof Object) {
      newValue = this.renderBlock(value);
    } else {
      newValue = value;
    }
    return newValue;
  }
}

class MarkdownRenderer extends Renderer {
  conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined
  dateToText: ((value: DateViewModel) => string) | undefined
  domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined
  resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined

  constructor(callbacks: {
    conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined,
    dateToUrl: ((value: DateViewModel) => string) | undefined,
    domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined,
    resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined,
  }) {
    super();
    this.conceptValueToUrl = callbacks.conceptValueToUrl;
    this.dateToUrl = callbacks.dateToUrl;
    this.domainValueToUrl = callbacks.domainValueToUrl;
    this.resourceReferenceToUrl = callbacks.resourceReferenceToUrl;
  }


  async renderDomainValue(domainValue: DomainValueViewModel): Promise<any> {
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

  async renderDate(date: DateViewModel): Promise<any> {
    const value = await date;
    const text = this.dateToText ? await this.dateToText(value): value.toISOString();
    const wrapper = new Cleanable(`
    <time datetime='${text}'>
      ${text}
    </time>`.replace(/\n/g, ' ').trim());
    wrapper.__clean = text;
    return wrapper;
  }

  async renderConceptValue(conceptValue: ConceptValueViewModel): Promise<any> {
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

  async renderResourceReference(rivm: ResourceInstanceViewModel<any>): Promise<any> {
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

class JsonRenderer extends Renderer {
  async renderDate(value: DateViewModel): Promise<any> {
    return value.forJson();
  }

  async renderConceptValue(value: ConceptValueViewModel): Promise<any> {
    return value.forJson();
  }

  async renderDomainValue(value: DomainValueViewModel): Promise<any> {
    return value.forJson();
  }

  async renderResourceReference(value: ResourceInstanceViewModel<any>): Promise<any> {
    return value.forJson();
  }
}

export { MarkdownRenderer, JsonRenderer, Cleanable };
