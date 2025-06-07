import { IStringKeyedObject } from "./interfaces";

// TODO: make this customizable.
const DEFAULT_LANGUAGE = "en";

function getCurrentLanguage(): string {
  return ((typeof navigator != 'undefined' && navigator.language) || DEFAULT_LANGUAGE).slice(0, 2);
}

class AttrPromise<T> extends Promise<T> implements IStringKeyedObject {
  [key: string | symbol]: any;
  [Symbol.toPrimitive]: undefined;
  constructor(
    executor: (
      resolve: (value: T | PromiseLike<T>) => void,
      reject: (reason: any) => void,
    ) => void,
  ) {
    super(executor);
    const proxy = new Proxy(this, {
      set: (object: AttrPromise<T>, keyObj, value) => {
        object.then((val: any) => {
          val[keyObj] = value;
          return val;
        });
        return true;
      },
      get: (object: AttrPromise<T>, keyObj: string | symbol) => {
        if (keyObj in object) {
          const value: any = object[keyObj];
          if (typeof value === "function") {
            return value.bind(object);
          }
          return value;
        }
        const key = keyObj.toString();
        if (key in object) {
          const value: any = object[key];
          if (typeof value === "function") {
            return value.bind(object);
          }
          return value;
        }
        if (object instanceof Promise) {
          return object.then((val: any) => {
            return val ? val[keyObj] : val;
          });
        }
        return object[keyObj];
      },
    });
    return proxy;
  }
}

export { AttrPromise, getCurrentLanguage };
