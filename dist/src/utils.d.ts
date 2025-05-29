import { IStringKeyedObject } from './interfaces';
declare function getCurrentLanguage(): string;
declare class AttrPromise<T> extends Promise<T> implements IStringKeyedObject {
    [Symbol.toPrimitive]: undefined;
    constructor(executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason: any) => void) => void);
}
export { AttrPromise, getCurrentLanguage };
