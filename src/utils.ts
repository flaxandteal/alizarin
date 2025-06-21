import { v5 as uuidv5 } from 'uuid';
import { IStringKeyedObject } from "./interfaces";

// TODO: make this customizable.
const DEFAULT_LANGUAGE = "en";
const SLUG_LENGTH = 20;
const UUID_NAMESPACE = '1a79f1c8-9505-4bea-a18e-28a053f725ca'; // Generated for this purpose.
const UUID_NAMESPACE_COMPRESSION = uuidv5('compression', '1a79f1c8-9505-4bea-a18e-28a053f725ca');

function slugify(original: any): string {
    return `${original}`.replaceAll(/[^A-Za-z0-9_]/g, "").slice(0, SLUG_LENGTH);
}

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

const KEY_COMPRESSION_LENGTH = 1000;
function generateUuidv5(group: [type: string, id?: string], key: string | string[]) {
  if (Array.isArray(key)) {
    let shortKey = '';
    let keyTracker = '';
    key.forEach(k => {
      if (keyTracker.length + k.length + 1 > KEY_COMPRESSION_LENGTH) {
        shortKey = uuidv5(shortKey + '>' + keyTracker, UUID_NAMESPACE_COMPRESSION);
        keyTracker = k;
      } else {
        keyTracker += ';' + k;
      }
    });
  }
  return uuidv5(`${group[0]}:${group[1]}:${key}`, UUID_NAMESPACE);
}

export { slugify, AttrPromise, getCurrentLanguage, generateUuidv5 };
