import { UrlViewModel, DateViewModel, ResourceInstanceViewModel, DomainValueViewModel, ConceptValueViewModel, SemanticViewModel, BooleanViewModel, NumberViewModel } from './viewModels';
declare class Cleanable extends String {
    __clean: string | undefined;
}
declare abstract class BaseRenderer {
    render(asset: ResourceInstanceViewModel<any>): Promise<any>;
    abstract renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any>;
    abstract renderDate(value: DateViewModel, _depth: number): Promise<any>;
    abstract renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any>;
    abstract renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any>;
    abstract renderSemantic(value: SemanticViewModel, depth: number): Promise<any>;
    abstract renderBlock(block: {
        [key: string]: string;
    } | {
        [key: string]: string;
    }[], depth: number): any;
    abstract renderArray(value: any[], depth: number): Promise<any>;
    abstract renderString(value: String, _depth: number): Promise<any>;
    abstract renderBoolean(value: BooleanViewModel, _depth: number): Promise<any>;
    abstract renderNumber(value: NumberViewModel, _depth: number): Promise<any>;
    abstract renderUrl(value: UrlViewModel, _depth: number): Promise<any>;
    renderValue(value: any, depth: number): Promise<any>;
}
declare class Renderer extends BaseRenderer {
    renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any>;
    renderString(value: String, _depth: number): Promise<any>;
    renderNumber(value: Number, _depth: number): Promise<any>;
    renderBoolean(value: Boolean, _depth: number): Promise<any>;
    renderDate(value: DateViewModel, _depth: number): Promise<any>;
    renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any>;
    renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any>;
    renderSemantic(value: SemanticViewModel, depth: number): Promise<any>;
    renderUrl(value: UrlViewModel, _depth: number): Promise<any>;
    renderBlock(block: {
        [key: string]: string;
    } | {
        [key: string]: string;
    }[], depth: number): any;
    renderArray(value: any, depth: number): Promise<any>;
}
declare class MarkdownRenderer extends Renderer {
    conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined;
    dateToText: ((value: DateViewModel) => string) | undefined;
    domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined;
    resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined;
    nodeToUrl: ((value: string) => string) | undefined;
    constructor(callbacks: {
        conceptValueToUrl: ((value: ConceptValueViewModel) => string) | undefined;
        dateToText: ((value: DateViewModel) => string) | undefined;
        domainValueToUrl: ((value: DomainValueViewModel) => string) | undefined;
        resourceReferenceToUrl: ((value: ResourceInstanceViewModel<any>) => string) | undefined;
        nodeToUrl: ((value: string) => string) | undefined;
    });
    renderUrl(value: UrlViewModel, _depth: number): Promise<any>;
    renderDomainValue(domainValue: DomainValueViewModel, _: number): Promise<any>;
    renderDate(date: DateViewModel, _: number): Promise<any>;
    renderConceptValue(conceptValue: ConceptValueViewModel, _: number): Promise<any>;
    renderResourceReference(rivm: ResourceInstanceViewModel<any>, _: number): Promise<any>;
}
declare class FlatMarkdownRenderer extends MarkdownRenderer {
    renderSemantic(vm: SemanticViewModel, depth: number): Promise<any>;
    renderArray(value: any, depth: number): Promise<any>;
    renderString(value: String, _depth: number): Promise<any>;
}
declare class JsonRenderer extends Renderer {
    renderDate(value: DateViewModel, _depth: number): Promise<any>;
    renderConceptValue(value: ConceptValueViewModel, _depth: number): Promise<any>;
    renderDomainValue(value: DomainValueViewModel, _depth: number): Promise<any>;
    renderResourceReference(value: ResourceInstanceViewModel<any>, _depth: number): Promise<any>;
}
export { MarkdownRenderer, JsonRenderer, Cleanable, FlatMarkdownRenderer };
