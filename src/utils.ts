import { IStringKeyedObject } from "./interfaces";

class AttrPromise<T> extends Promise<T> implements IStringKeyedObject {
  constructor(
    executor: (
      resolve: (value: T | PromiseLike<T>) => void,
      reject: (reason: any) => void,
    ) => void,
  ) {
    super(executor);
    return new Proxy(this, {
      set: (object: IStringKeyedObject, keyObj, value) => {
        const key = keyObj.toString();
        if (object instanceof Promise) {
          return object.then((val) => {
            val[key] = value;
            return val;
          });
        }
        object[key] = value;
        return this;
      },
      get: (object: IStringKeyedObject, keyObj) => {
        const key = keyObj.toString();
        if (key in object) {
          if (typeof object[key] === "function") {
            return object[key].bind(object);
          }
          return object[key];
        }
        if (object instanceof Promise) {
          return object.then((val) => {
            return val ? val[key] : val;
          });
        }
        return object[key];
      },
    });
  }
}

export { AttrPromise };
