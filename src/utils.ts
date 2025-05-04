import { IStringKeyedObject } from "./interfaces";

// TODO: make this customizable.
const DEFAULT_LANGUAGE = "en";

function getCurrentLanguage(): string {
  return ((typeof navigator != 'undefined' && navigator.language) || DEFAULT_LANGUAGE).slice(0, 2);
}

class AttrPromise<T> extends Promise<T> implements IStringKeyedObject {
  [Symbol.toPrimitive]: undefined;
  constructor(
    executor: (
      resolve: (value: T | PromiseLike<T>) => void,
      reject: (reason: any) => void,
    ) => void,
  ) {
    super(executor);
    return new Proxy(this, {
      set: (object: IStringKeyedObject, keyObj, value) => {
        if (object instanceof Promise) {
          return object.then((val) => {
            val[keyObj] = value;
            return val;
          });
        }
        object[keyObj] = value;
        return this;
      },
      get: (object: IStringKeyedObject, keyObj) => {
        if (keyObj in object) {
          if (typeof object[keyObj] === "function") {
            return object[keyObj].bind(object);
          }
          return object[keyObj];
        }
        const key = keyObj.toString();
        if (key in object) {
          if (typeof object[key] === "function") {
            return object[key].bind(object);
          }
          return object[key];
        }
        if (object instanceof Promise) {
          return object.then((val) => {
            return val ? val[keyObj] : val;
          });
        }
        return object[keyObj];
      },
    });
  }
}

export { AttrPromise, getCurrentLanguage };
