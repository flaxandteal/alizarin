var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
let AlizarinModel, GraphManager, GraphMutator, RDM, ResourceModelWrapper, WKRM, client, getCurrentLanguage, graphManager, interfaces, nodeConfig, renderers, run, setCurrentLanguage, staticStore, staticTypes, utils, viewModels;
let __tla = (async () => {
  var _a, _b, _c, _d, _e;
  const REGEX = /^(?:[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}|00000000-0000-0000-0000-000000000000|ffffffff-ffff-ffff-ffff-ffffffffffff)$/i;
  function validate(uuid) {
    return typeof uuid === "string" && REGEX.test(uuid);
  }
  function parse(uuid) {
    if (!validate(uuid)) {
      throw TypeError("Invalid UUID");
    }
    let v;
    return Uint8Array.of((v = parseInt(uuid.slice(0, 8), 16)) >>> 24, v >>> 16 & 255, v >>> 8 & 255, v & 255, (v = parseInt(uuid.slice(9, 13), 16)) >>> 8, v & 255, (v = parseInt(uuid.slice(14, 18), 16)) >>> 8, v & 255, (v = parseInt(uuid.slice(19, 23), 16)) >>> 8, v & 255, (v = parseInt(uuid.slice(24, 36), 16)) / 1099511627776 & 255, v / 4294967296 & 255, v >>> 24 & 255, v >>> 16 & 255, v >>> 8 & 255, v & 255);
  }
  const byteToHex = [];
  for (let i = 0; i < 256; ++i) {
    byteToHex.push((i + 256).toString(16).slice(1));
  }
  function unsafeStringify(arr, offset = 0) {
    return (byteToHex[arr[offset + 0]] + byteToHex[arr[offset + 1]] + byteToHex[arr[offset + 2]] + byteToHex[arr[offset + 3]] + "-" + byteToHex[arr[offset + 4]] + byteToHex[arr[offset + 5]] + "-" + byteToHex[arr[offset + 6]] + byteToHex[arr[offset + 7]] + "-" + byteToHex[arr[offset + 8]] + byteToHex[arr[offset + 9]] + "-" + byteToHex[arr[offset + 10]] + byteToHex[arr[offset + 11]] + byteToHex[arr[offset + 12]] + byteToHex[arr[offset + 13]] + byteToHex[arr[offset + 14]] + byteToHex[arr[offset + 15]]).toLowerCase();
  }
  let getRandomValues;
  const rnds8 = new Uint8Array(16);
  function rng() {
    if (!getRandomValues) {
      if (typeof crypto === "undefined" || !crypto.getRandomValues) {
        throw new Error("crypto.getRandomValues() not supported. See https://github.com/uuidjs/uuid#getrandomvalues-not-supported");
      }
      getRandomValues = crypto.getRandomValues.bind(crypto);
    }
    return getRandomValues(rnds8);
  }
  function stringToBytes(str) {
    str = unescape(encodeURIComponent(str));
    const bytes = new Uint8Array(str.length);
    for (let i = 0; i < str.length; ++i) {
      bytes[i] = str.charCodeAt(i);
    }
    return bytes;
  }
  const DNS = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
  const URL$1 = "6ba7b811-9dad-11d1-80b4-00c04fd430c8";
  function v35(version, hash, value, namespace, buf, offset) {
    const valueBytes = typeof value === "string" ? stringToBytes(value) : value;
    const namespaceBytes = typeof namespace === "string" ? parse(namespace) : namespace;
    if (typeof namespace === "string") {
      namespace = parse(namespace);
    }
    if ((namespace == null ? void 0 : namespace.length) !== 16) {
      throw TypeError("Namespace must be array-like (16 iterable integer values, 0-255)");
    }
    let bytes = new Uint8Array(16 + valueBytes.length);
    bytes.set(namespaceBytes);
    bytes.set(valueBytes, namespaceBytes.length);
    bytes = hash(bytes);
    bytes[6] = bytes[6] & 15 | version;
    bytes[8] = bytes[8] & 63 | 128;
    return unsafeStringify(bytes);
  }
  const randomUUID = typeof crypto !== "undefined" && crypto.randomUUID && crypto.randomUUID.bind(crypto);
  const native = {
    randomUUID
  };
  function v4(options, buf, offset) {
    var _a2;
    if (native.randomUUID && true && !options) {
      return native.randomUUID();
    }
    options = options || {};
    const rnds = options.random ?? ((_a2 = options.rng) == null ? void 0 : _a2.call(options)) ?? rng();
    if (rnds.length < 16) {
      throw new Error("Random bytes length must be >= 16");
    }
    rnds[6] = rnds[6] & 15 | 64;
    rnds[8] = rnds[8] & 63 | 128;
    return unsafeStringify(rnds);
  }
  function f(s, x, y, z) {
    switch (s) {
      case 0:
        return x & y ^ ~x & z;
      case 1:
        return x ^ y ^ z;
      case 2:
        return x & y ^ x & z ^ y & z;
      case 3:
        return x ^ y ^ z;
    }
  }
  function ROTL(x, n) {
    return x << n | x >>> 32 - n;
  }
  function sha1(bytes) {
    const K = [
      1518500249,
      1859775393,
      2400959708,
      3395469782
    ];
    const H = [
      1732584193,
      4023233417,
      2562383102,
      271733878,
      3285377520
    ];
    const newBytes = new Uint8Array(bytes.length + 1);
    newBytes.set(bytes);
    newBytes[bytes.length] = 128;
    bytes = newBytes;
    const l = bytes.length / 4 + 2;
    const N = Math.ceil(l / 16);
    const M = new Array(N);
    for (let i = 0; i < N; ++i) {
      const arr = new Uint32Array(16);
      for (let j = 0; j < 16; ++j) {
        arr[j] = bytes[i * 64 + j * 4] << 24 | bytes[i * 64 + j * 4 + 1] << 16 | bytes[i * 64 + j * 4 + 2] << 8 | bytes[i * 64 + j * 4 + 3];
      }
      M[i] = arr;
    }
    M[N - 1][14] = (bytes.length - 1) * 8 / Math.pow(2, 32);
    M[N - 1][14] = Math.floor(M[N - 1][14]);
    M[N - 1][15] = (bytes.length - 1) * 8 & 4294967295;
    for (let i = 0; i < N; ++i) {
      const W = new Uint32Array(80);
      for (let t = 0; t < 16; ++t) {
        W[t] = M[i][t];
      }
      for (let t = 16; t < 80; ++t) {
        W[t] = ROTL(W[t - 3] ^ W[t - 8] ^ W[t - 14] ^ W[t - 16], 1);
      }
      let a = H[0];
      let b = H[1];
      let c = H[2];
      let d = H[3];
      let e = H[4];
      for (let t = 0; t < 80; ++t) {
        const s = Math.floor(t / 20);
        const T = ROTL(a, 5) + f(s, b, c, d) + e + K[s] + W[t] >>> 0;
        e = d;
        d = c;
        c = ROTL(b, 30) >>> 0;
        b = a;
        a = T;
      }
      H[0] = H[0] + a >>> 0;
      H[1] = H[1] + b >>> 0;
      H[2] = H[2] + c >>> 0;
      H[3] = H[3] + d >>> 0;
      H[4] = H[4] + e >>> 0;
    }
    return Uint8Array.of(H[0] >> 24, H[0] >> 16, H[0] >> 8, H[0], H[1] >> 24, H[1] >> 16, H[1] >> 8, H[1], H[2] >> 24, H[2] >> 16, H[2] >> 8, H[2], H[3] >> 24, H[3] >> 16, H[3] >> 8, H[3], H[4] >> 24, H[4] >> 16, H[4] >> 8, H[4]);
  }
  function v5(value, namespace, buf, offset) {
    return v35(80, sha1, value, namespace);
  }
  v5.DNS = DNS;
  v5.URL = URL$1;
  const DEFAULT_LANGUAGE$1 = "en";
  const SLUG_LENGTH = 20;
  const UUID_NAMESPACE = "1a79f1c8-9505-4bea-a18e-28a053f725ca";
  const UUID_NAMESPACE_COMPRESSION = v5("compression", "1a79f1c8-9505-4bea-a18e-28a053f725ca");
  let currentLanguage;
  function slugify(original) {
    return `${original}`.replaceAll(/[^A-Za-z0-9_]/g, "").slice(0, SLUG_LENGTH);
  }
  function getCurrentLanguage$1() {
    return currentLanguage || (typeof navigator != "undefined" && navigator.language || DEFAULT_LANGUAGE$1).slice(0, 2);
  }
  function setCurrentLanguage$1(lang) {
    currentLanguage = lang;
  }
  class AttrPromise extends (_b = Promise, _a = Symbol.toPrimitive, _b) {
    constructor(executor) {
      super(executor);
      __publicField(this, _a);
      const proxy = new Proxy(this, {
        set: (object, keyObj, value) => {
          object.then((val) => {
            val[keyObj] = value;
            return val;
          });
          return true;
        },
        get: (object, keyObj) => {
          if (keyObj in object) {
            const value = object[keyObj];
            if (typeof value === "function") {
              return value.bind(object);
            }
            return value;
          }
          const key = keyObj.toString();
          if (key in object) {
            const value = object[key];
            if (typeof value === "function") {
              return value.bind(object);
            }
            return value;
          }
          if (object instanceof Promise) {
            return object.then((val) => {
              return val ? val[keyObj] : val;
            });
          }
          return object[keyObj];
        }
      });
      return proxy;
    }
  }
  const KEY_COMPRESSION_LENGTH = 1e3;
  function generateUuidv5(group, key) {
    if (Array.isArray(key)) {
      let shortKey = "";
      let keyTracker = "";
      key.forEach((k) => {
        if (keyTracker.length + k.length + 1 > KEY_COMPRESSION_LENGTH) {
          shortKey = v5(shortKey + ">" + keyTracker, UUID_NAMESPACE_COMPRESSION);
          keyTracker = k;
        } else {
          keyTracker += ";" + k;
        }
      });
    }
    return v5(`${group[0]}:${group[1]}:${key}`, UUID_NAMESPACE);
  }
  utils = Object.freeze(Object.defineProperty({
    __proto__: null,
    AttrPromise,
    generateUuidv5,
    getCurrentLanguage: getCurrentLanguage$1,
    setCurrentLanguage: setCurrentLanguage$1,
    slugify
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  let wasm;
  let cachedUint8ArrayMemory0 = null;
  function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
      cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
  }
  let cachedTextDecoder = new TextDecoder("utf-8", {
    ignoreBOM: true,
    fatal: true
  });
  cachedTextDecoder.decode();
  const MAX_SAFARI_DECODE_BYTES = 2146435072;
  let numBytesDecoded = 0;
  function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
      cachedTextDecoder = new TextDecoder("utf-8", {
        ignoreBOM: true,
        fatal: true
      });
      cachedTextDecoder.decode();
      numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
  }
  function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
  }
  let WASM_VECTOR_LEN = 0;
  const cachedTextEncoder = new TextEncoder();
  if (!("encodeInto" in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function(arg, view) {
      const buf = cachedTextEncoder.encode(arg);
      view.set(buf);
      return {
        read: arg.length,
        written: buf.length
      };
    };
  }
  function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === void 0) {
      const buf = cachedTextEncoder.encode(arg);
      const ptr2 = malloc(buf.length, 1) >>> 0;
      getUint8ArrayMemory0().subarray(ptr2, ptr2 + buf.length).set(buf);
      WASM_VECTOR_LEN = buf.length;
      return ptr2;
    }
    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;
    const mem = getUint8ArrayMemory0();
    let offset = 0;
    for (; offset < len; offset++) {
      const code = arg.charCodeAt(offset);
      if (code > 127) break;
      mem[ptr + offset] = code;
    }
    if (offset !== len) {
      if (offset !== 0) {
        arg = arg.slice(offset);
      }
      ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
      const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
      const ret = cachedTextEncoder.encodeInto(arg, view);
      offset += ret.written;
      ptr = realloc(ptr, len, offset, 1) >>> 0;
    }
    WASM_VECTOR_LEN = offset;
    return ptr;
  }
  let cachedDataViewMemory0 = null;
  function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || cachedDataViewMemory0.buffer.detached === void 0 && cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
      cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
  }
  function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_4.set(idx, obj);
    return idx;
  }
  function handleError(f2, args) {
    try {
      return f2.apply(this, args);
    } catch (e) {
      const idx = addToExternrefTable0(e);
      wasm.__wbindgen_exn_store(idx);
    }
  }
  function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
  }
  function isLikeNone(x) {
    return x === void 0 || x === null;
  }
  function debugString(val) {
    const type = typeof val;
    if (type == "number" || type == "boolean" || val == null) {
      return `${val}`;
    }
    if (type == "string") {
      return `"${val}"`;
    }
    if (type == "symbol") {
      const description = val.description;
      if (description == null) {
        return "Symbol";
      } else {
        return `Symbol(${description})`;
      }
    }
    if (type == "function") {
      const name = val.name;
      if (typeof name == "string" && name.length > 0) {
        return `Function(${name})`;
      } else {
        return "Function";
      }
    }
    if (Array.isArray(val)) {
      const length = val.length;
      let debug = "[";
      if (length > 0) {
        debug += debugString(val[0]);
      }
      for (let i = 1; i < length; i++) {
        debug += ", " + debugString(val[i]);
      }
      debug += "]";
      return debug;
    }
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
      className = builtInMatches[1];
    } else {
      return toString.call(val);
    }
    if (className == "Object") {
      try {
        return "Object(" + JSON.stringify(val) + ")";
      } catch (_) {
        return "Object";
      }
    }
    if (val instanceof Error) {
      return `${val.name}: ${val.message}
${val.stack}`;
    }
    return className;
  }
  function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_4.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
  }
  function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
      throw new Error(`expected instance of ${klass.name}`);
    }
  }
  function greet() {
    wasm.greet();
  }
  const StaticGraphMetaFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticgraphmeta_free(ptr >>> 0, 1));
  class StaticGraphMeta {
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticGraphMetaFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticgraphmeta_free(ptr, 0);
    }
    constructor(json_data) {
      const ret = wasm.staticgraphmeta_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticGraphMetaFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    get graphid() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticgraphmeta_get_graphid(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set graphid(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_set_graphid(this.__wbg_ptr, ptr0, len0);
    }
    toJSON() {
      const ret = wasm.staticgraphmeta_toJSON(this.__wbg_ptr);
      return ret;
    }
    get author() {
      const ret = wasm.staticgraphmeta_get_author_property(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    getAuthor() {
      const ret = wasm.staticgraphmeta_getAuthor(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    setAuthor(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setAuthor(this.__wbg_ptr, ptr0, len0);
    }
    get isresource() {
      const ret = wasm.staticgraphmeta_getIsResource(this.__wbg_ptr);
      return ret === 16777215 ? void 0 : ret !== 0;
    }
    getIsResource() {
      const ret = wasm.staticgraphmeta_getIsResource(this.__wbg_ptr);
      return ret === 16777215 ? void 0 : ret !== 0;
    }
    setIsResource(value) {
      wasm.staticgraphmeta_setIsResource(this.__wbg_ptr, isLikeNone(value) ? 16777215 : value ? 1 : 0);
    }
    getCards() {
      const ret = wasm.staticgraphmeta_getCards(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setCards(value) {
      wasm.staticgraphmeta_setCards(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    getCardsXNodesXWidgets() {
      const ret = wasm.staticgraphmeta_getCardsXNodesXWidgets(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setCardsXNodesXWidgets(value) {
      wasm.staticgraphmeta_setCardsXNodesXWidgets(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    getColor() {
      const ret = wasm.staticgraphmeta_getColor(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    setColor(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setColor(this.__wbg_ptr, ptr0, len0);
    }
    get description() {
      const ret = wasm.staticgraphmeta_get_description(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticTranslatableString.__wrap(ret);
    }
    set description(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticTranslatableString);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticgraphmeta_set_description(this.__wbg_ptr, ptr0);
    }
    getEdges() {
      const ret = wasm.staticgraphmeta_getEdges(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setEdges(value) {
      wasm.staticgraphmeta_setEdges(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    getIconClass() {
      const ret = wasm.staticgraphmeta_getIconClass(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    setIconClass(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setIconClass(this.__wbg_ptr, ptr0, len0);
    }
    getIsEditable() {
      const ret = wasm.staticgraphmeta_getIsEditable(this.__wbg_ptr);
      return ret === 16777215 ? void 0 : ret !== 0;
    }
    setIsEditable(value) {
      wasm.staticgraphmeta_setIsEditable(this.__wbg_ptr, isLikeNone(value) ? 16777215 : value ? 1 : 0);
    }
    getJsonLdContext() {
      const ret = wasm.staticgraphmeta_getJsonLdContext(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    setJsonLdContext(value) {
      const ret = wasm.staticgraphmeta_setJsonLdContext(this.__wbg_ptr, value);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    get name() {
      const ret = wasm.staticgraphmeta_get_name(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticTranslatableString.__wrap(ret);
    }
    set name(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticTranslatableString);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticgraphmeta_set_name(this.__wbg_ptr, ptr0);
    }
    getNodeGroups() {
      const ret = wasm.staticgraphmeta_getNodeGroups(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setNodeGroups(value) {
      wasm.staticgraphmeta_setNodeGroups(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    getNodes() {
      const ret = wasm.staticgraphmeta_getNodes(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setNodes(value) {
      wasm.staticgraphmeta_setNodes(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    getOntologyId() {
      const ret = wasm.staticgraphmeta_getOntologyId(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    setOntologyId(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setOntologyId(this.__wbg_ptr, ptr0, len0);
    }
    getPublication() {
      const ret = wasm.staticgraphmeta_getPublication(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    setPublication(value) {
      const ret = wasm.staticgraphmeta_setPublication(this.__wbg_ptr, value);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getRelatableResourceModelIds() {
      const ret = wasm.staticgraphmeta_getRelatableResourceModelIds(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    setRelatableResourceModelIds(value) {
      const ret = wasm.staticgraphmeta_setRelatableResourceModelIds(this.__wbg_ptr, value);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getResource2ResourceConstraints() {
      const ret = wasm.staticgraphmeta_getResource2ResourceConstraints(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    setResource2ResourceConstraints(value) {
      const ret = wasm.staticgraphmeta_setResource2ResourceConstraints(this.__wbg_ptr, value);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getRoot() {
      const ret = wasm.staticgraphmeta_getRoot(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticNode.__wrap(ret);
    }
    setRoot(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticNode);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticgraphmeta_setRoot(this.__wbg_ptr, ptr0);
    }
    get slug() {
      const ret = wasm.staticgraphmeta_get_slug(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set slug(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_set_slug(this.__wbg_ptr, ptr0, len0);
    }
    get subtitle() {
      const ret = wasm.staticgraphmeta_get_subtitle(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticTranslatableString.__wrap(ret);
    }
    set subtitle(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticTranslatableString);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticgraphmeta_set_subtitle(this.__wbg_ptr, ptr0);
    }
    getVersion() {
      const ret = wasm.staticgraphmeta_getVersion(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    setVersion(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setVersion(this.__wbg_ptr, ptr0, len0);
    }
  }
  if (Symbol.dispose) StaticGraphMeta.prototype[Symbol.dispose] = StaticGraphMeta.prototype.free;
  const StaticNodeFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticnode_free(ptr >>> 0, 1));
  class StaticNode {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticNode.prototype);
      obj.__wbg_ptr = ptr;
      StaticNodeFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticNodeFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticnode_free(ptr, 0);
    }
    constructor(json_data) {
      const ret = wasm.staticnode_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticNodeFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticnode_copy(this.__wbg_ptr);
      return StaticNode.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticnode_toJSON(this.__wbg_ptr);
      return ret;
    }
    get alias() {
      const ret = wasm.staticnode_get_alias(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set alias(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_alias(this.__wbg_ptr, ptr0, len0);
    }
    get datatype() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticnode_get_datatype(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set datatype(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_datatype(this.__wbg_ptr, ptr0, len0);
    }
    get description() {
      const ret = wasm.staticnode_get_description(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set description(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_description(this.__wbg_ptr, ptr0, len0);
    }
    get exportable() {
      const ret = wasm.staticnode_get_exportable(this.__wbg_ptr);
      return ret !== 0;
    }
    set exportable(value) {
      wasm.staticnode_set_exportable(this.__wbg_ptr, value);
    }
    get fieldname() {
      const ret = wasm.staticnode_get_fieldname(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set fieldname(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_fieldname(this.__wbg_ptr, ptr0, len0);
    }
    get graph_id() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticnode_get_graph_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set graph_id(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_graph_id(this.__wbg_ptr, ptr0, len0);
    }
    get hascustomalias() {
      const ret = wasm.staticnode_get_hascustomalias(this.__wbg_ptr);
      return ret !== 0;
    }
    set hascustomalias(value) {
      wasm.staticnode_set_hascustomalias(this.__wbg_ptr, value);
    }
    get is_collector() {
      const ret = wasm.staticnode_get_is_collector(this.__wbg_ptr);
      return ret !== 0;
    }
    set is_collector(value) {
      wasm.staticnode_set_is_collector(this.__wbg_ptr, value);
    }
    get isrequired() {
      const ret = wasm.staticnode_get_isrequired(this.__wbg_ptr);
      return ret !== 0;
    }
    set isrequired(value) {
      wasm.staticnode_set_isrequired(this.__wbg_ptr, value);
    }
    get issearchable() {
      const ret = wasm.staticnode_get_issearchable(this.__wbg_ptr);
      return ret !== 0;
    }
    set issearchable(value) {
      wasm.staticnode_set_issearchable(this.__wbg_ptr, value);
    }
    get istopnode() {
      const ret = wasm.staticnode_get_istopnode(this.__wbg_ptr);
      return ret !== 0;
    }
    set istopnode(value) {
      wasm.staticnode_set_istopnode(this.__wbg_ptr, value);
    }
    get name() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticnode_get_name(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set name(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_name(this.__wbg_ptr, ptr0, len0);
    }
    get nodegroup_id() {
      const ret = wasm.staticnode_get_nodegroup_id(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set nodegroup_id(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_nodegroup_id(this.__wbg_ptr, ptr0, len0);
    }
    get nodeid() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticnode_get_nodeid(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set nodeid(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_nodeid(this.__wbg_ptr, ptr0, len0);
    }
    get ontologyclass() {
      const ret = wasm.staticnode_get_ontologyclass(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set ontologyclass(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_ontologyclass(this.__wbg_ptr, ptr0, len0);
    }
    get parentproperty() {
      const ret = wasm.staticnode_get_parentproperty(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set parentproperty(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_parentproperty(this.__wbg_ptr, ptr0, len0);
    }
    get sortorder() {
      const ret = wasm.staticnode_get_sortorder(this.__wbg_ptr);
      return ret;
    }
    set sortorder(value) {
      wasm.staticnode_set_sortorder(this.__wbg_ptr, value);
    }
    get sourcebranchpublication_id() {
      const ret = wasm.staticnode_get_sourcebranchpublication_id(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set sourcebranchpublication_id(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_sourcebranchpublication_id(this.__wbg_ptr, ptr0, len0);
    }
    get config() {
      const ret = wasm.staticnode_get_config_property(this.__wbg_ptr);
      return ret;
    }
    getConfig() {
      const ret = wasm.staticnode_getConfig(this.__wbg_ptr);
      return ret;
    }
    set config(value) {
      wasm.staticnode_set_config_property(this.__wbg_ptr, value);
    }
    setConfig(value) {
      wasm.staticnode_setConfig(this.__wbg_ptr, value);
    }
    static compare(node_a, node_b) {
      const ret = wasm.staticnode_compare(node_a, node_b);
      return ret;
    }
  }
  if (Symbol.dispose) StaticNode.prototype[Symbol.dispose] = StaticNode.prototype.free;
  const StaticTranslatableStringFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_statictranslatablestring_free(ptr >>> 0, 1));
  class StaticTranslatableString {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticTranslatableString.prototype);
      obj.__wbg_ptr = ptr;
      StaticTranslatableStringFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticTranslatableStringFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_statictranslatablestring_free(ptr, 0);
    }
    constructor(value, lang) {
      var ptr0 = isLikeNone(lang) ? 0 : passStringToWasm0(lang, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.statictranslatablestring_new(value, ptr0, len0);
      this.__wbg_ptr = ret >>> 0;
      StaticTranslatableStringFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    toString() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.statictranslatablestring_toString(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    toJSON() {
      const ret = wasm.statictranslatablestring_toJSON(this.__wbg_ptr);
      return ret;
    }
    copy() {
      const ret = wasm.statictranslatablestring_copy(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    get lang() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.statictranslatablestring_get_lang(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get translations() {
      const ret = wasm.statictranslatablestring_get_translations(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) StaticTranslatableString.prototype[Symbol.dispose] = StaticTranslatableString.prototype.free;
  const EXPECTED_RESPONSE_TYPES = /* @__PURE__ */ new Set([
    "basic",
    "cors",
    "default"
  ]);
  async function __wbg_load(module, imports) {
    if (typeof Response === "function" && module instanceof Response) {
      if (typeof WebAssembly.instantiateStreaming === "function") {
        try {
          return await WebAssembly.instantiateStreaming(module, imports);
        } catch (e) {
          const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);
          if (validResponse && module.headers.get("Content-Type") !== "application/wasm") {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
          } else {
            throw e;
          }
        }
      }
      const bytes = await module.arrayBuffer();
      return await WebAssembly.instantiate(bytes, imports);
    } else {
      const instance = await WebAssembly.instantiate(module, imports);
      if (instance instanceof WebAssembly.Instance) {
        return {
          instance,
          module
        };
      } else {
        return instance;
      }
    }
  }
  function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_Error_e17e777aac105295 = function(arg0, arg1) {
      const ret = Error(getStringFromWasm0(arg0, arg1));
      return ret;
    };
    imports.wbg.__wbg_Number_998bea33bd87c3e0 = function(arg0) {
      const ret = Number(arg0);
      return ret;
    };
    imports.wbg.__wbg_String_8f0eb39a4a4c2f66 = function(arg0, arg1) {
      const ret = String(arg1);
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_call_13410aac570ffff7 = function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_done_75ed0ee6dd243d9d = function(arg0) {
      const ret = arg0.done;
      return ret;
    };
    imports.wbg.__wbg_entries_2be2f15bd5554996 = function(arg0) {
      const ret = Object.entries(arg0);
      return ret;
    };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
      let deferred0_0;
      let deferred0_1;
      try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
      } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
      }
    };
    imports.wbg.__wbg_error_99981e16d476aa5c = function(arg0) {
      console.error(arg0);
    };
    imports.wbg.__wbg_get_0da715ceaecea5c8 = function(arg0, arg1) {
      const ret = arg0[arg1 >>> 0];
      return ret;
    };
    imports.wbg.__wbg_get_458e874b43b18b25 = function() {
      return handleError(function(arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_getwithrefkey_1dc361bd10053bfe = function(arg0, arg1) {
      const ret = arg0[arg1];
      return ret;
    };
    imports.wbg.__wbg_instanceof_ArrayBuffer_67f3012529f6a2dd = function(arg0) {
      let result;
      try {
        result = arg0 instanceof ArrayBuffer;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    };
    imports.wbg.__wbg_instanceof_Map_ebb01a5b6b5ffd0b = function(arg0) {
      let result;
      try {
        result = arg0 instanceof Map;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    };
    imports.wbg.__wbg_instanceof_Uint8Array_9a8378d955933db7 = function(arg0) {
      let result;
      try {
        result = arg0 instanceof Uint8Array;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    };
    imports.wbg.__wbg_isArray_030cce220591fb41 = function(arg0) {
      const ret = Array.isArray(arg0);
      return ret;
    };
    imports.wbg.__wbg_isSafeInteger_1c0d1af5542e102a = function(arg0) {
      const ret = Number.isSafeInteger(arg0);
      return ret;
    };
    imports.wbg.__wbg_iterator_f370b34483c71a1c = function() {
      const ret = Symbol.iterator;
      return ret;
    };
    imports.wbg.__wbg_length_186546c51cd61acd = function(arg0) {
      const ret = arg0.length;
      return ret;
    };
    imports.wbg.__wbg_length_6bb7e81f9d7713e4 = function(arg0) {
      const ret = arg0.length;
      return ret;
    };
    imports.wbg.__wbg_new_19c25a3f2fa63a02 = function() {
      const ret = new Object();
      return ret;
    };
    imports.wbg.__wbg_new_1f3a344cf3123716 = function() {
      const ret = new Array();
      return ret;
    };
    imports.wbg.__wbg_new_2ff1f68f3676ea53 = function() {
      const ret = /* @__PURE__ */ new Map();
      return ret;
    };
    imports.wbg.__wbg_new_638ebfaedbf32a5e = function(arg0) {
      const ret = new Uint8Array(arg0);
      return ret;
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
      const ret = new Error();
      return ret;
    };
    imports.wbg.__wbg_next_5b3530e612fde77d = function(arg0) {
      const ret = arg0.next;
      return ret;
    };
    imports.wbg.__wbg_next_692e82279131b03c = function() {
      return handleError(function(arg0) {
        const ret = arg0.next();
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_prototypesetcall_3d4a26c1ed734349 = function(arg0, arg1, arg2) {
      Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
    };
    imports.wbg.__wbg_push_330b2eb93e4e1212 = function(arg0, arg1) {
      const ret = arg0.push(arg1);
      return ret;
    };
    imports.wbg.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
      arg0[arg1] = arg2;
    };
    imports.wbg.__wbg_set_453345bcda80b89a = function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_set_90f6c0f7bd8c0415 = function(arg0, arg1, arg2) {
      arg0[arg1 >>> 0] = arg2;
    };
    imports.wbg.__wbg_set_b7f1cf4fae26fe2a = function(arg0, arg1, arg2) {
      const ret = arg0.set(arg1, arg2);
      return ret;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
      const ret = arg1.stack;
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_dd9372230531eade = function(arg0) {
      const ret = arg0.value;
      return ret;
    };
    imports.wbg.__wbg_wbindgenbigintgetasi64_ac743ece6ab9bba1 = function(arg0, arg1) {
      const v = arg1;
      const ret = typeof v === "bigint" ? v : void 0;
      getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbg_wbindgenbooleanget_3fe6f642c7d97746 = function(arg0) {
      const v = arg0;
      const ret = typeof v === "boolean" ? v : void 0;
      return isLikeNone(ret) ? 16777215 : ret ? 1 : 0;
    };
    imports.wbg.__wbg_wbindgendebugstring_99ef257a3ddda34d = function(arg0, arg1) {
      const ret = debugString(arg1);
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_wbindgenin_d7a1ee10933d2d55 = function(arg0, arg1) {
      const ret = arg0 in arg1;
      return ret;
    };
    imports.wbg.__wbg_wbindgenisbigint_ecb90cc08a5a9154 = function(arg0) {
      const ret = typeof arg0 === "bigint";
      return ret;
    };
    imports.wbg.__wbg_wbindgenisfunction_8cee7dce3725ae74 = function(arg0) {
      const ret = typeof arg0 === "function";
      return ret;
    };
    imports.wbg.__wbg_wbindgenisobject_307a53c6bd97fbf8 = function(arg0) {
      const val = arg0;
      const ret = typeof val === "object" && val !== null;
      return ret;
    };
    imports.wbg.__wbg_wbindgenisstring_d4fa939789f003b0 = function(arg0) {
      const ret = typeof arg0 === "string";
      return ret;
    };
    imports.wbg.__wbg_wbindgenisundefined_c4b71d073b92f3c5 = function(arg0) {
      const ret = arg0 === void 0;
      return ret;
    };
    imports.wbg.__wbg_wbindgenjsvaleq_e6f2ad59ccae1b58 = function(arg0, arg1) {
      const ret = arg0 === arg1;
      return ret;
    };
    imports.wbg.__wbg_wbindgenjsvallooseeq_9bec8c9be826bed1 = function(arg0, arg1) {
      const ret = arg0 == arg1;
      return ret;
    };
    imports.wbg.__wbg_wbindgennumberget_f74b4c7525ac05cb = function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "number" ? obj : void 0;
      getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbg_wbindgenstringget_0f16a6ddddef376f = function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "string" ? obj : void 0;
      var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_wbindgenthrow_451ec1a8469d7eb6 = function(arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
      const ret = getStringFromWasm0(arg0, arg1);
      return ret;
    };
    imports.wbg.__wbindgen_cast_4625c577ab2ec9ee = function(arg0) {
      const ret = BigInt.asUintN(64, arg0);
      return ret;
    };
    imports.wbg.__wbindgen_cast_9ae0607507abb057 = function(arg0) {
      const ret = arg0;
      return ret;
    };
    imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
      const ret = arg0;
      return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
      const table = wasm.__wbindgen_export_4;
      const offset = table.grow(4);
      table.set(0, void 0);
      table.set(offset + 0, void 0);
      table.set(offset + 1, null);
      table.set(offset + 2, true);
      table.set(offset + 3, false);
    };
    return imports;
  }
  function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
  }
  async function __wbg_init(module_or_path) {
    if (wasm !== void 0) return wasm;
    if (typeof module_or_path !== "undefined") {
      if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
        ({ module_or_path } = module_or_path);
      } else {
        console.warn("using deprecated parameters for the initialization function; pass a single object instead");
      }
    }
    if (typeof module_or_path === "undefined") {
      module_or_path = new URL("data:application/wasm;base64,AGFzbQEAAAAB3QI3YAJ/fwBgAn9/AX9gA39/fwF/YAN/f38AYAACf39gAX8AYAF/AX9gAX8Cf39gBH9/f38AYAFvAX9gAAN/f39gBH9/f38Bf2ACf28AYAAAYAV/f39/fwBgAAF/YAABb2ABbwFvYAZ/f39/f38AYAJ/fABgAX8BfGACb28Bb2ACb28Bf2ADf39/AX5gAX8Df39/YAJ/bwJ/f2ABfwFvYAV/f39/fwF/YAJ/fwFvYAF+AW9gBn9/f39/fwF/YAJ/fwF+YAFvA39/f2ABfgF/YAFvAGADb29vAGACb38Bb2ADb39vAGADb29vAW9gA39/bwBgA29vbwF/YAFvAXxgAXwBb2AJf39/f39/fn5+AGADfn9/AX9gB39/f39/f38Bf2AFf39+f38AYAR/fn9/AGAFf399f38AYAR/fX9/AGAFf398f38AYAR/fH9/AGACf34AYANvf38Bf2ABfAF/Aq4QNQN3YmccX193YmdfZXJyb3JfOTk5ODFlMTZkNDc2YWE1YwAiA3diZx1fX3diZ19TdHJpbmdfOGYwZWIzOWE0YTRjMmY2NgAMA3diZyRfX3diZ19nZXR3aXRocmVma2V5XzFkYzM2MWJkMTAwNTNiZmUAFQN3YmcaX193Ymdfc2V0XzNmMWQwYjk4NGVkMjcyZWQAIwN3YmcaX193YmdfZ2V0XzBkYTcxNWNlYWVjZWE1YzgAJAN3YmcdX193YmdfbGVuZ3RoXzE4NjU0NmM1MWNkNjFhY2QACQN3YmcaX193YmdfbmV3XzFmM2EzNDRjZjMxMjM3MTYAEAN3YmcaX193YmdfbmV3XzJmZjFmNjhmMzY3NmVhNTMAEAN3YmcbX193YmdfbmV4dF81YjM1MzBlNjEyZmRlNzdkABEDd2JnG19fd2JnX2RvbmVfNzVlZDBlZTZkZDI0M2Q5ZAAJA3diZxxfX3diZ192YWx1ZV9kZDkzNzIyMzA1MzFlYWRlABEDd2JnH19fd2JnX2l0ZXJhdG9yX2YzNzBiMzQ0ODNjNzFhMWMAEAN3YmcaX193YmdfbmV3XzE5YzI1YTNmMmZhNjNhMDIAEAN3YmcaX193Ymdfc2V0XzkwZjZjMGY3YmQ4YzA0MTUAJQN3YmceX193YmdfaXNBcnJheV8wMzBjY2UyMjA1OTFmYjQxAAkDd2JnG19fd2JnX3B1c2hfMzMwYjJlYjkzZTRlMTIxMgAWA3diZy1fX3diZ19pbnN0YW5jZW9mX0FycmF5QnVmZmVyXzY3ZjMwMTI1MjlmNmEyZGQACQN3YmcbX193YmdfY2FsbF8xMzQxMGFhYzU3MGZmZmY3ABUDd2JnJV9fd2JnX2luc3RhbmNlb2ZfTWFwX2ViYjAxYTViNmI1ZmZkMGIACQN3YmcaX193Ymdfc2V0X2I3ZjFjZjRmYWUyNmZlMmEAJgN3YmcbX193YmdfbmV4dF82OTJlODIyNzkxMzFiMDNjABEDd2JnJF9fd2JnX2lzU2FmZUludGVnZXJfMWMwZDFhZjU1NDJlMTAyYQAJA3diZx5fX3diZ19lbnRyaWVzXzJiZTJmMTViZDU1NTQ5OTYAEQN3YmcdX193YmdfbGVuZ3RoXzZiYjdlODFmOWQ3NzEzZTQACQN3YmcnX193YmdfcHJvdG90eXBlc2V0Y2FsbF8zZDRhMjZjMWVkNzM0MzQ5ACcDd2JnGl9fd2JnX25ld182MzhlYmZhZWRiZjMyYTVlABEDd2JnLF9fd2JnX2luc3RhbmNlb2ZfVWludDhBcnJheV85YTgzNzhkOTU1OTMzZGI3AAkDd2JnGl9fd2JnX2dldF80NThlODc0YjQzYjE4YjI1ABUDd2JnGl9fd2JnX3NldF80NTMzNDViY2RhODBiODlhACgDd2JnGl9fd2JnX25ld184YTZmMjM4YTZlY2U4NmVhABADd2JnHF9fd2JnX3N0YWNrXzBlZDc1ZDY4NTc1YjBmM2MADAN3YmccX193YmdfZXJyb3JfNzUzNGI4ZTlhMzZmMWFiNAAAA3diZypfX3diZ193YmluZGdlbmRlYnVnc3RyaW5nXzk5ZWYyNTdhM2RkZGEzNGQADAN3YmccX193YmdfRXJyb3JfZTE3ZTc3N2FhYzEwNTI5NQAcA3diZx1fX3diZ19OdW1iZXJfOTk4YmVhMzNiZDg3YzNlMAApA3diZypfX3diZ193YmluZGdlbmlzdW5kZWZpbmVkX2M0YjcxZDA3M2I5MmYzYzUACQN3YmcnX193Ymdfd2JpbmRnZW5pc29iamVjdF8zMDdhNTNjNmJkOTdmYmY4AAkDd2JnKV9fd2JnX3diaW5kZ2VuaXNmdW5jdGlvbl84Y2VlN2RjZTM3MjVhZTc0AAkDd2JnJ19fd2JnX3diaW5kZ2VuaXNzdHJpbmdfZDRmYTkzOTc4OWYwMDNiMAAJA3diZydfX3diZ193YmluZGdlbmlzYmlnaW50X2VjYjkwY2MwOGE1YTkxNTQACQN3YmchX193Ymdfd2JpbmRnZW5pbl9kN2ExZWUxMDkzM2QyZDU1ABYDd2JnKF9fd2JnX3diaW5kZ2VubnVtYmVyZ2V0X2Y3NGI0Yzc1MjVhYzA1Y2IADAN3YmcpX193Ymdfd2JpbmRnZW5ib29sZWFuZ2V0XzNmZTZmNjQyYzdkOTc3NDYACQN3YmcoX193Ymdfd2JpbmRnZW5zdHJpbmdnZXRfMGYxNmE2ZGRkZGVmMzc2ZgAMA3diZy1fX3diZ193YmluZGdlbmJpZ2ludGdldGFzaTY0X2FjNzQzZWNlNmFiOWJiYTEADAN3YmckX193Ymdfd2JpbmRnZW50aHJvd180NTFlYzFhODQ2OWQ3ZWI2AAADd2JnJl9fd2JnX3diaW5kZ2VuanN2YWxlcV9lNmYyYWQ1OWNjYWUxYjU4ABYDd2JnK19fd2JnX3diaW5kZ2VuanN2YWxsb29zZWVxXzliZWM4YzliZTgyNmJlZDEAFgN3YmcfX193YmluZGdlbl9pbml0X2V4dGVybnJlZl90YWJsZQANA3diZyBfX3diaW5kZ2VuX2Nhc3RfZDZjZDE5YjgxNTYwZmQ2ZQAqA3diZyBfX3diaW5kZ2VuX2Nhc3RfNDYyNWM1NzdhYjJlYzllZQAdA3diZyBfX3diaW5kZ2VuX2Nhc3RfMjI0MWI2YWY0YzRiMjk0MQAcA3diZyBfX3diaW5kZ2VuX2Nhc3RfOWFlMDYwNzUwN2FiYjA1NwAdA7sDuQMABgAAAQECAQAIAAMDAx4IBQgDAQgCAQYCAwIBAAIAAgAfHwABAAAAKwAIASwCABISLQMBAQAAAAMSAQEBAAABEgABAAABAA8DAAgACAsAAQYOBgEBAA4AAAMFAAEDAAgADggIAQgAAwMDAwMDAwMDAwMDAwADAAgGAAADAAMDAwMBBgAAAgABAAAAAAABAgAFAAEGBgYABQ4FAwMAAAAAAAEBBQAAAAAAEgMDAAUFAAACCAAAExMTABMTAAYDAwABAQMAAAAAAAAAAAAFBgEDAwUFBQMCBQIBBQUNAgEAAAUGFBQUBhQUDQAGAwAAAQYGAwEFAAUABgYGBgYGABsFBQUFBgULBQEFBQEBAAUABgAFIBgYGBggBQAFARkZGRkIAAIHBwcHBwcHBwcHBwcHBwcHBwcHBx4BFQUXFxsOLjAyAA8PCwUIBQABBRcLCwsAAgABBgg0BgUDBQEBAAMLCAICAwUBAiEhNQABAQEBDg8PDwwBAQMBBgYBAQEAAAABAQEGGhoaGgMBAwYGBgMDAQUBAQEBAQEBBQEFAQEFAQUNDQABAAEBAAECAQEBAQEFNgEBAAYADQ0ECQJwAWxsbwCAAQUDAQARBgkBfwFBgIDAAAsHrBpwBm1lbW9yeQIAI19fd2JnX3N0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19mcmVlANkBHHN0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19uZXcAngMhc3RhdGljdHJhbnNsYXRhYmxlc3RyaW5nX3RvU3RyaW5nANgCHXN0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19jb3B5ALcBIXN0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19nZXRfbGFuZwDZAilzdGF0aWN0cmFuc2xhdGFibGVzdHJpbmdfZ2V0X3RyYW5zbGF0aW9ucwC5AxpfX3diZ19zdGF0aWNncmFwaG1ldGFfZnJlZQCAARNzdGF0aWNncmFwaG1ldGFfbmV3AMcCG3N0YXRpY2dyYXBobWV0YV9nZXRfZ3JhcGhpZADaAhtzdGF0aWNncmFwaG1ldGFfc2V0X2dyYXBoaWQAsAEWc3RhdGljZ3JhcGhtZXRhX3RvSlNPTgC6AxlzdGF0aWNncmFwaG1ldGFfZ2V0QXV0aG9yANsCGXN0YXRpY2dyYXBobWV0YV9zZXRBdXRob3IAnAEdc3RhdGljZ3JhcGhtZXRhX2dldElzUmVzb3VyY2UAlgIdc3RhdGljZ3JhcGhtZXRhX3NldElzUmVzb3VyY2UA6QEYc3RhdGljZ3JhcGhtZXRhX2dldENhcmRzAJcCGHN0YXRpY2dyYXBobWV0YV9zZXRDYXJkcwDqASZzdGF0aWNncmFwaG1ldGFfZ2V0Q2FyZHNYTm9kZXNYV2lkZ2V0cwCYAiZzdGF0aWNncmFwaG1ldGFfc2V0Q2FyZHNYTm9kZXNYV2lkZ2V0cwDrARhzdGF0aWNncmFwaG1ldGFfZ2V0Q29sb3IA3AIYc3RhdGljZ3JhcGhtZXRhX3NldENvbG9yAJ0BH3N0YXRpY2dyYXBobWV0YV9nZXRfZGVzY3JpcHRpb24AyAEfc3RhdGljZ3JhcGhtZXRhX3NldF9kZXNjcmlwdGlvbgC9ARhzdGF0aWNncmFwaG1ldGFfZ2V0RWRnZXMAmQIYc3RhdGljZ3JhcGhtZXRhX3NldEVkZ2VzAOwBHHN0YXRpY2dyYXBobWV0YV9nZXRJY29uQ2xhc3MA3QIcc3RhdGljZ3JhcGhtZXRhX3NldEljb25DbGFzcwCeAR1zdGF0aWNncmFwaG1ldGFfZ2V0SXNFZGl0YWJsZQCaAh1zdGF0aWNncmFwaG1ldGFfc2V0SXNFZGl0YWJsZQDtASBzdGF0aWNncmFwaG1ldGFfZ2V0SnNvbkxkQ29udGV4dADIAiBzdGF0aWNncmFwaG1ldGFfc2V0SnNvbkxkQ29udGV4dADRAhhzdGF0aWNncmFwaG1ldGFfZ2V0X25hbWUAyQEYc3RhdGljZ3JhcGhtZXRhX3NldF9uYW1lAL4BHXN0YXRpY2dyYXBobWV0YV9nZXROb2RlR3JvdXBzAJsCHXN0YXRpY2dyYXBobWV0YV9zZXROb2RlR3JvdXBzAO4BGHN0YXRpY2dyYXBobWV0YV9nZXROb2RlcwCcAhhzdGF0aWNncmFwaG1ldGFfc2V0Tm9kZXMA7wEdc3RhdGljZ3JhcGhtZXRhX2dldE9udG9sb2d5SWQA3gIdc3RhdGljZ3JhcGhtZXRhX3NldE9udG9sb2d5SWQAnwEec3RhdGljZ3JhcGhtZXRhX2dldFB1YmxpY2F0aW9uAMkCHnN0YXRpY2dyYXBobWV0YV9zZXRQdWJsaWNhdGlvbgDSAixzdGF0aWNncmFwaG1ldGFfZ2V0UmVsYXRhYmxlUmVzb3VyY2VNb2RlbElkcwDKAixzdGF0aWNncmFwaG1ldGFfc2V0UmVsYXRhYmxlUmVzb3VyY2VNb2RlbElkcwDTAi9zdGF0aWNncmFwaG1ldGFfZ2V0UmVzb3VyY2UyUmVzb3VyY2VDb25zdHJhaW50cwDLAi9zdGF0aWNncmFwaG1ldGFfc2V0UmVzb3VyY2UyUmVzb3VyY2VDb25zdHJhaW50cwDUAhdzdGF0aWNncmFwaG1ldGFfZ2V0Um9vdACtARdzdGF0aWNncmFwaG1ldGFfc2V0Um9vdADRARhzdGF0aWNncmFwaG1ldGFfZ2V0X3NsdWcA3wIYc3RhdGljZ3JhcGhtZXRhX3NldF9zbHVnAKABHHN0YXRpY2dyYXBobWV0YV9nZXRfc3VidGl0bGUAygEcc3RhdGljZ3JhcGhtZXRhX3NldF9zdWJ0aXRsZQC/ARpzdGF0aWNncmFwaG1ldGFfZ2V0VmVyc2lvbgDgAhpzdGF0aWNncmFwaG1ldGFfc2V0VmVyc2lvbgChARVfX3diZ19zdGF0aWNub2RlX2ZyZWUA9AEOc3RhdGljbm9kZV9uZXcAzAIPc3RhdGljbm9kZV9jb3B5AIICEXN0YXRpY25vZGVfdG9KU09OALsDFHN0YXRpY25vZGVfZ2V0X2FsaWFzAOECFHN0YXRpY25vZGVfc2V0X2FsaWFzAKIBF3N0YXRpY25vZGVfZ2V0X2RhdGF0eXBlAOICF3N0YXRpY25vZGVfc2V0X2RhdGF0eXBlALIBGnN0YXRpY25vZGVfZ2V0X2Rlc2NyaXB0aW9uAOMCGnN0YXRpY25vZGVfc2V0X2Rlc2NyaXB0aW9uAKMBGXN0YXRpY25vZGVfZ2V0X2V4cG9ydGFibGUArAIZc3RhdGljbm9kZV9zZXRfZXhwb3J0YWJsZQD5ARhzdGF0aWNub2RlX2dldF9maWVsZG5hbWUA5AIYc3RhdGljbm9kZV9zZXRfZmllbGRuYW1lAKQBF3N0YXRpY25vZGVfZ2V0X2dyYXBoX2lkAOUCF3N0YXRpY25vZGVfc2V0X2dyYXBoX2lkALMBHXN0YXRpY25vZGVfZ2V0X2hhc2N1c3RvbWFsaWFzAK0CHXN0YXRpY25vZGVfc2V0X2hhc2N1c3RvbWFsaWFzAPoBG3N0YXRpY25vZGVfZ2V0X2lzX2NvbGxlY3RvcgCuAhtzdGF0aWNub2RlX3NldF9pc19jb2xsZWN0b3IA+wEZc3RhdGljbm9kZV9nZXRfaXNyZXF1aXJlZACvAhlzdGF0aWNub2RlX3NldF9pc3JlcXVpcmVkAPwBG3N0YXRpY25vZGVfZ2V0X2lzc2VhcmNoYWJsZQCwAhtzdGF0aWNub2RlX3NldF9pc3NlYXJjaGFibGUA/QEYc3RhdGljbm9kZV9nZXRfaXN0b3Bub2RlALECGHN0YXRpY25vZGVfc2V0X2lzdG9wbm9kZQD+ARNzdGF0aWNub2RlX2dldF9uYW1lAOYCE3N0YXRpY25vZGVfc2V0X25hbWUAtAEbc3RhdGljbm9kZV9nZXRfbm9kZWdyb3VwX2lkAOcCG3N0YXRpY25vZGVfc2V0X25vZGVncm91cF9pZAClARVzdGF0aWNub2RlX2dldF9ub2RlaWQA6AIVc3RhdGljbm9kZV9zZXRfbm9kZWlkALUBHHN0YXRpY25vZGVfZ2V0X29udG9sb2d5Y2xhc3MA6QIcc3RhdGljbm9kZV9zZXRfb250b2xvZ3ljbGFzcwCmAR1zdGF0aWNub2RlX2dldF9wYXJlbnRwcm9wZXJ0eQDqAh1zdGF0aWNub2RlX3NldF9wYXJlbnRwcm9wZXJ0eQCnARhzdGF0aWNub2RlX2dldF9zb3J0b3JkZXIAuAIYc3RhdGljbm9kZV9zZXRfc29ydG9yZGVyAIACKXN0YXRpY25vZGVfZ2V0X3NvdXJjZWJyYW5jaHB1YmxpY2F0aW9uX2lkAOsCKXN0YXRpY25vZGVfc2V0X3NvdXJjZWJyYW5jaHB1YmxpY2F0aW9uX2lkAKgBFHN0YXRpY25vZGVfZ2V0Q29uZmlnALwDFHN0YXRpY25vZGVfc2V0Q29uZmlnAKgDEnN0YXRpY25vZGVfY29tcGFyZQDuAidzdGF0aWNncmFwaG1ldGFfZ2V0X2lzcmVzb3VyY2VfcHJvcGVydHkAlgIfc3RhdGljdHJhbnNsYXRhYmxlc3RyaW5nX3RvSlNPTgC5Ax5zdGF0aWNub2RlX2dldF9jb25maWdfcHJvcGVydHkAvAMjc3RhdGljZ3JhcGhtZXRhX2dldF9hdXRob3JfcHJvcGVydHkA2wIec3RhdGljbm9kZV9zZXRfY29uZmlnX3Byb3BlcnR5AKgDBG1haW4AnQIFZ3JlZXQA7QMRX193YmluZGdlbl9tYWxsb2MApwISX193YmluZGdlbl9yZWFsbG9jALoCFF9fd2JpbmRnZW5fZXhuX3N0b3JlAJkDF19fZXh0ZXJucmVmX3RhYmxlX2FsbG9jAHwTX193YmluZGdlbl9leHBvcnRfNAEBD19fd2JpbmRnZW5fZnJlZQCYAxlfX2V4dGVybnJlZl90YWJsZV9kZWFsbG9jANgBEF9fd2JpbmRnZW5fc3Rh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gICAeEciAwRAIAJB2AJqIAJBnAFqKAIANgIAIAIgAikClAE3A9ACDAELIAJBgICAgHg2AtACCwJAIAIoAqABQYCAgIB4RyIERQRAQdeVwABBBhD1ASEHIABBgICAgHg2ApwBIAAgBzYCAEEAIQcMAQsgAkHoAmogAkGoAWooAgA2AgAgAiACKQKgATcD4AICQCACKAKsASISQYGAgIB4RwRAIAJB+AJqIAJBtAFqKAIANgIAIAIgAikCrAE3A/ACDAELIAJBgICAgHg2AvACCwJAIAIoArgBIhRBgYCAgHhHBEAgAkGIA2ogAkHAAWooAgA2AgAgAiACKQK4ATcDgAMMAQsgAkGAgICAeDYCgAMLIAdFBEAgEkGBgICAeEchByAUQYGAgIB4RyESQfiVwABBCRD1ASEIIABBgICAgHg2ApwBIAAgCDYCACACQYADahCPAyACQfACahCPAyACQeACahDQAwwBCyAAIAIpA9ABNwIkIAAgAikD4AE3AwAgACACKQJYNwJ4IABBLGogAkHYAWooAgA2AgAgAEEIaiACQegBaikDADcDACAAQRBqIAJB8AFqKQMANwMAIABBGGogAkH4AWopAwA3AwAgAEGAAWogAkHgAGooAgA2AgAgAigCxAEhASACKQLIASEVIABBOGogAkGYAmooAgA2AgAgACACKQOQAjcCMCAAIAIpA6ACNwI8IABBxABqIAJBqAJqKAIANgIAIAAgAikCfDcChAEgAEGMAWogAkGEAWooAgA2AgAgACACKQKIATcCkAEgAEGYAWogAkGQAWooAgA2AgAgAEHQAGogAkHYAmooAgA2AgAgACACKQPQAjcCSCAAIBBBAXE6AK0BIAAgD0EBcToArAEgACAOQQFxOgCrASAAIA1BAXE6AKoBIAAgDEEBcToAqQEgACAIQQFxOgCoASAAIBU3A3AgAEGAgICAeCABIAFBgYCAgHhGGzYCbCAAIAo2AiAgAEGkAWogAkGoAWooAgA2AgAgACACKQKgATcCnAEgAEHcAGogAkH4AmooAgA2AgAgACACKQPwAjcCVCAAQegAaiACQYgDaigCADYCACAAIAIpA4ADNwJgDCALIAJB0AJqEI8DIAJBwAJqENADDAILQaCVwABBDBD1AQshBCAAQYCAgIB4NgKcASAAIAQ2AgBBACEHQQAhBEEAIQNBACEFCyACQbACahDQAwsgAkGgAmoQjwMgE0GBgICAeEcLIQAgAkGQAmoQjwMgAkGAAmoQ0AMgEUGBgICAeEcLIQggAkHgAWoQhwIgAkHQAWoQjwMgAUEARwwYCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQywEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADBgLIAIoAugBIQYgAigCxAFBgYCAgHhHBEAgAkHEAWoQjwMLIAIgBjYCzAEgAiADNgLIASACIAU2AsQBDBMLDBoLIAIoAhggAkEANgIYBEAgAigCHCEGIwBBEGsiBSQAIAUgBjYCDCMAQTBrIgckACAHQQhqIgogBUEMaiIDEK4DBH4gCiADKAIAJQEQIvwGNwMIQgEFQgALNwMAQQEhCiAFAn8gBygCCEEBRgRAIAcpAxAiFUKAgICACH1C/////29WBEBBACEKIBWnDAILIAdBAjoAGCAHIBU3AyAjAEEwayIDJAAgA0Hwj8AANgIEIAMgB0EvajYCACADQQI2AgwgA0G8nMAANgIIIANCAjcCFCADQRc2AiwgA0EYNgIkIAMgB0EYajYCICADIANBIGo2AhAgAyADNgIoIANBCGoQhwEgA0EwaiQADAELIAMgB0EvakHwj8AAEFQLNgIEIAUgCjYCACAHQTBqJAAgBSgCBCEDIAUoAgAhByAGQYQBTwRAIAYQ2AELIAIgBzYCACACIAM2AgQgBUEQaiQAQQEhByACKAIEIQogAigCAEEBcUUNEiAAQYCAgIB4NgKcASAAIAo2AgAgAiAENgI8IAIgATYCOAwWCwwZCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQywEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADBYLIAIoAugBIQYgAigCuAFBgYCAgHhHBEAgAkG4AWoQjwMLIAIgBjYCwAEgAiADNgK8ASACIAU2ArgBDBELDBgLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDLASACKALkASEDIAIoAuABIgVBgYCAgHhGBEAgAiAENgI8IAIgATYCOCAAQYCAgIB4NgKcASAAIAM2AgAMFQsgAigC6AEhBiACKAKsAUGBgICAeEcEQCACQawBahCPAwsgAiAGNgK0ASACIAM2ArABIAIgBTYCrAEMEAsMFwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwUCyACKALoASEGIAJBoAFqEI8DIAIgBjYCqAEgAiADNgKkASACIAU2AqABDA8LDBYLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDLASACKALkASEDIAIoAuABIgVBgYCAgHhGBEAgAiAENgI8IAIgATYCOCAAQYCAgIB4NgKcASAAIAM2AgAMEwsgAigC6AEhBiACKAKUAUGBgICAeEcEQCACQZQBahCPAwsgAiAGNgKcASACIAM2ApgBIAIgBTYClAEMDgsMFQsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwSCyACKALoASEGIAJBiAFqEI8DIAIgBjYCkAEgAiADNgKMASACIAU2AogBDA0LDBQLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDVASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMEQsgAi0A4QEhEAwMCwwTCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQ1QEgAi0A4AEEQCACIAQ2AjwgAiABNgI4IAIoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADBALIAItAOEBIQ8MCwsMEgsgAigCGCACQQA2AhgEQCACQeABaiACKAIcENUBIAItAOABBEAgAiAENgI8IAIgATYCOCACKALkASEBIABBgICAgHg2ApwBIAAgATYCAAwPCyACLQDhASEODAoLDBELIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDVASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMDgsgAi0A4QEhDQwJCwwQCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQ1QEgAi0A4AEEQCACIAQ2AjwgAiABNgI4IAIoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADA0LIAItAOEBIQwMCAsMDwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwMCyACKALoASEGIAJB/ABqEI8DIAIgBjYChAEgAiADNgKAASACIAU2AnwMBwsMDgsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMsBIAIoAuQBIQMgAigC4AEiBUGBgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwLCyACKALoASEGIAIoAnBBgYCAgHhHBEAgAkHwAGoQjwMLIAIgBjYCeCACIAM2AnQgAiAFNgJwDAYLDA0LIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDVASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMCgsgAi0A4QEhCAwFCwwMCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQywEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADAkLIAIoAugBIQYgAigCZEGBgICAeEcEQCACQeQAahCPAwsgAiAGNgJsIAIgAzYCaCACIAU2AmQMBAsMCwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwICyACKALoASEGIAJB2ABqEI8DIAIgBjYCYCACIAM2AlwgAiAFNgJYDAMLDAoLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBA3IAIoAuQBIQMgAigC4AEiAUUEQCACIAQ2AjwgAkEANgI4IABBgICAgHg2ApwBIAAgAzYCAAwHCyAJIAspAwA3AwAgCUEQaiALQRBqKQMANwMAIAlBCGogC0EIaikDADcDACADIQQMAgsgAiAENgI8IAJBADYCOEH4psAAQTEQ2AMACyACKAIYIAJBADYCGEUNASACQeABaiACKAIcEMsBIAIoAuQB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cWtBBGsoAgAiAxCDAiEGIAcgAxCDAiEDAkAgBgRAQQAhBCADRQ0EIAYgAxA8RQ0EDAELIANFDQBBACEEDAMLIAlCAX0gCYMhCSAFQQFrIgUNAAsLQQEhBAsgAkEIahCOAgwBCwsgAkHwAGokACAEC7IfAwt/AX4BfCMAQYABayIDJAAgAyABNgIMAkACQAJAAkAgA0EMaiIBEMQCRQRAIAEQjANB/wFxIgFBAkcEQCAAIAE6AAEgAEEBOgAADAMLAkACQAJAIANBDGoiASgCACUBECdFBEAgA0EQaiABEKECIAMoAhBFDQEgAysDGCEOIAEQrgMNAyADQdgAaiIBIA69Qv///////////wCDQv/////////3/wBYBH4gASAOOQMIQgIFQgMLNwMAIANBADoAaCAAIAMpA1hCA1IEfyADQc8AaiADQeAAaikDADcAACADIAMpA1g3AEcgA0HoAGoQqAJBAgVBAAs6AAAgACADKQBANwABIABBCWogA0HIAGopAAA3AAAgAEEQaiADQc8AaikAADcAAAwGCyADIAMoAgwiATYCQCADQegAaiICIANBQGsiBBCiAiADKAJoQQFHDQEgAyADKQNwIg0QnQMiATYCaCAEIAIQoQMgAUGEAU8EQCABENgBCyADKAJAIQFFDQEgAUGEAU8EQCABENgBCyAAIA03AxAgAEECOgAAIAAgDUI/iDcDCAwHCyADQSRqIANBDGoQ1AEgAygCJEGAgICAeEcEQCADQfMAaiADQSxqKAIANgAAIABBAzoAACADIAMpAiQ3AGsgACADKQBoNwABIABBCGogA0HvAGopAAA3AAAMBQsgA0EMaiIBEK0DDQMgA0HoAGogARCuASADKAJoQYCAgIB4RwRAIANBOGogA0HwAGooAgA2AgAgAyADKQJoNwMwIwBBIGsiASQAIANBMGoiAikCBCENIAFBBjoACCABIA03AgwgAUEIaiABQR9qQZCdwAAQ5gEhBCAAQQY6AAAgACAENgIEIAIQ0AMgAUEgaiQADAULIANBDGoiAhDBAwRAIAMQpgMiATYCQCADQUBrIAIQoANFBEAgAUGEAUkNByABENgBDAcLIAFBhAFPBEAgARDYAQsgA0EMaigCACUBEBINBgsgA0EMaiADQUBrQYiRwAAQVCEBIABBBjoAACAAIAE2AgQMBAsgAyABNgJAIANB6ABqIgIgA0FAayIEEKICAkAgAygCaEEBRw0AIAMgAykDcCINEJwDIgE2AmggBCACEKEDIAFBhAFPBEAgARDYAQsgAygCQCEBRQ0AIAFBhAFPBEAgARDYAQsgACANNwMQIABCADcDCCAAQQI6AAAMBgtBmJLAAEHPABDCASECIABBBjoAACAAIAI2AgQgAUGEAUkNBSABENgBDAULIABBAjoAACAAIA78BiINNwMQIAAgDUI/iDcDCAwCCyAAQQA6AAAMAQsgAygCDCECIwBBMGsiASQAIAEgAjYCAAJAIAEQrQMEQCMAQUBqIgIkACACQQxqIAEQhQMgAkEANgIYIAJBADYCJCACQoCAgICAATcCHCACQShqQQFyIgVBCGohBiAFQQ9qIQgCQANAAkAgAkEoaiACQQxqEJsBAkACQCACLQAoIglBBmsOAgIAAQsgACACKAIsNgIEIABBBjoAACACQRxqIgAQwgIgABDTAwwDCyACKAIkIgcgAigCHEYEQCACQRxqQeiQwAAQlAILIAIoAiAgB0EYbGoiBCAFKQAANwABIAQgCToAACAEQQlqIAYpAAA3AAAgBEEQaiAIKQAANwAAIAIgB0EBajYCJAwBCwsgAkEzaiACQSRqKAIANgAAIABBBDoAACACIAIpAhw3ACsgACACKQAoNwABIABBCGogAkEvaikAADcAAAsgAkFAayQADAELIAFBGGogARB4IAEoAhghAgJAAkACQCABLQAcIgRBAmsOAgEAAgsgAEEGOgAAIAAgAjYCBAwCCyABIAFBGGpBiJHAABBUIQIgAEEGOgAAIAAgAjYCBAwBCyABIAQ6AAggASACNgIEIAFBADYCFCABQoCAgICAATcCDCABQRhqQQFyIgRBCGohByAEQQ9qIQYCQANAAkAgAUEYaiABQQRqEJMBAkACQCABLQAYIghBBmsOAgIAAQsgACABKAIcNgIEIABBBjoAACABQQxqIgAQwgIgABDTAyABKAIEIgBBgwFNDQQMAwsgASgCFCIFIAEoAgxGBEAgAUEMakHokMAAEJQCCyABKAIQIAVBGGxqIgIgBCkAADcAASACIAg6AAAgAkEJaiAHKQAANwAAIAJBEGogBikAADcAACABIAVBAWo2AhQMAQsLIAFBI2ogAUEUaigCADYAACAAQQQ6AAAgASABKQIM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AihBASECIAFBEGoQhwEMAgtBACECIAq6EOYDDAELQQAhAiAKEJwDCzYCBCAGIAI2AgAgAUEwaiQAIAQoAgghASAEKAIMDAILIAEpAwghCiMAQTBrIgEkACABIAo3AwggBEEQaiIGAn8gAi0AAkUEQCAKQv////////8PfEL/////////H1oEQCABQQI2AhQgAUGUqsAANgIQIAFCATcCHCABQR82AiwgASABQShqNgIYIAEgAUEIajYCKEEBIQIgAUEQahCHAQwCC0EAIQIgCrkQ5gMMAQtBACECIAoQnQMLNgIEIAYgAjYCACABQTBqJAAgBCgCECEBIAQoAhQMAQsgBEEYaiICIAErAwgQ5gM2AgQgAkEANgIAIAQoAhghASAEKAIcCyECIAUgATYCACAFIAI2AgQgBEEgaiQAIAMoAhQhASADKAIQDAQLIANBGGogAiABKAIIIAEoAgwQlQMgAygCHCEBIAMoAhgMAwsgA0EgaiACIAFBBGoQbSADKAIkIQEgAygCIAwCC0GBAUGAASACLQAAGyEBQQAMAQsgA0HYAGohBSABKAIMIQYCfyACLQABRQRAEAchCxB8IgQgCyYBQQAMAQsQpwMhBEEBCyEHIAUgAjYCECAFQQA2AgggBSAENgIEIAUgBzYCAAJAIAMoAlhBAkcEQCADQdAAaiADQegAaigCADYCACADQcgAaiADQeAAaikCADcDACADIAMpAlg3A0AgASgCCCECIAMgBkEAIAEoAgQiARs2AnggAyACNgJ0IAMgATYCcCADQQA2AmwgAyABQQBHIgQ2AmggAyACNgJkIAMgATYCYCADQQA2AlwgAyAENgJYAkADQCADQThqIANB2ABqEH4gAygCOCIERQ0BIAMoAjwhByMAQRBrIgUkACMAQRBrIgIkACACQQhqIANBQGsiASgCECAEKAIEIAQoAggQlQNBASEEIAIoAgwhBiACKAIIQQFxRQRAAkAgASgCCEUNACABKAIMIgRBhAFJDQAgBBDYAQsgASAGNgIMIAFBATYCCEEAIQQLIANBMGohCCAFQQhqIgkgBjYCBCAJIAQ2AgAgAkEQaiQAQQEhAgJ/IAUoAghBAXEEQCAFKAIMDAELIwBBIGsiBCQAIAEoAgghAiABQQA2AggCQAJAIAIEQCAEIAEoAgwiBjYCFCAEQQhqIAcgASgC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AAsgAEIANwMIIABBAjoAACAAIAEpAwg3AxAMAgsgACABKQMIEIsDDAELIAErAwghCSMAQSBrIgEkACABQQA6AAgCQCAJvUL///////////8Ag0L/////////9/8AWARAIAAgCTkDECAAQgI3AwggAEECOgAAIAFBCGoQ4gEMAQsgACABKQMINwMAIABBEGogAUEYaikDADcDACAAQQhqIAFBEGopAwA3AwALIAFBIGokAAsMBQsgASgCCCEFIAJBKGogASgCDCIBQQFBARCYASACKAIsIQQgAigCKEEBRg0CIAIoAjAhAyABBEAgAyAFIAH8CgAACyAAIAE2AgwgACADNgIIIAAgBDYCBCAAQQM6AAAMBAsgACABQQRqEHMMAwsgAkEoaiABKAIMIgMQkgMgAigCKEGBgICAeEYEQCAAIAIoAiw2AgQgAEEGOgAADAMLIAJBIGogAkE4aikCADcDACACQRhqIAJBMGopAgA3AwAgAiACKQIoNwMQIAEoAgghBCACIANBACABKAIEIgEbNgJIIAIgBDYCRCACIAE2AkAgAkEANgI8IAIgAUEARyIDNgI4IAIgBDYCNCACIAE2AjAgAkEANgIsIAIgAzYCKANAIAJBCGogAkEoahB+IAIoAggiA0UNAiACKAIMIQcCfyACQRBqIQQjAEEQayIBJAAgAygCBCEIIAFBBGogAygCCCIFQQFBARCYASABKAIIIQYgASgCBEEBRwRAIAEoAgwhAyAFBEAgAyAIIAX8CgAACyAGQYCAgIB4RwRAIAQQjwMgBCAFNgIIIAQgAzYCBCAEIAY2AgBBACEDCyABQRBqJAAgAwwBCyAGIAEoAgxB+JnAABCOAwALIgFFBEACfyMAQeAAayIBJAAgBCgCACEDIARBgICAgHg2AgAgA0GAgICAeEcEQCABIAQpAgQ3AiggASADNgIkIAEgBzYCCCABQTBqIAFBCGoQUQJAIAEtADBBBkYEQCABKAI0IQQgAUEkahDQAwwBCyABQdgAaiABQUBrKQMANwMAIAFB0ABqIAFBOGopAwA3AwAgASABKQMwNwNIIAFBCGoiAyAEQQxqIAFBJGogAUHIAGoQfyADEKoCQQAhBAsgAUHgAGokACAEDAELQfiXwABBK0GMmcAAEPcBAAsiAUUNAQsLIABBBjoAACAAIAE2AgQgAkEcahDO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ADYCbCACQgA3AmQgAkEANgJUIAJBADYCRCACQQA2AjAgAkEANgIgIAIgASgCBCIENgJcIAIgASgCACIBNgJYIAIgBDYCTCACIAE2AkggAiAAKAIEIgQ2AjggAiAAKAIAIgA2AjQgAiAENgIoIAIgADYCJCACIANBACABGzYCYCACIAFBAEciATYCUCACIAE2AkAgAiADQQAgABs2AjwgAiAAQQBHIgA2AiwgAiAANgIcIAJBEGogAkEcahB+AkAgAigCECIBRQ0AIAJBQGshBiACKAIUIQADQCACQQhqIAYQfiACKAIIIgVFDQEgAigCDCEDQQAhBCABKAIEIAEoAgggBSgCBCAFKAIIEPoCRQ0CIAAtAAAiASADLQAARw0CAkACQAJAAkACQAJAIAFBAWsOBQABAgMEBQsgAC0AASADLQABRg0EDAcLIABBCGogA0EIahCNAg0DDAYLIAAoAgggACgCDCADKAIIIAMoAgwQ+gINAgwFCyAAKAIIIAAoAgwgAygCCCADKAIMEIIBDQEMBAsgAEEEaiADQQRqEFlFDQMLIAIgAkEcahB+IAIoAgQhACACKAIAIgENAAsLQQEhBAsgAkHwAGokACAEC/kDAQJ/IAAgAWohAgJAAkAgACgCBCIDQQFxDQAgA0ECcUUNASAAKAIAIgMgAWohASAAIANrIgBBtJLBACgCAEYEQCACKAIEQQNxQQNHDQFBrJLBACABNgIAIAIgAigCBEF+cTYCBCAAIAFBAXI2AgQgAiABNgIADAILIAAgAxBjCwJAAkACQCACKAIEIgNBAnFFBEAgAkG4ksEAKAIARg0CIAJBtJLBACgCAEYNAyACIANBeHEiAhBjIAAgASACaiIBQQFyNgIEIAAgAWogATYCACAAQbSSwQAoAgBHDQFBrJLBACABNgIADwsgAiADQX5xNgIEIAAgAUEBcjYCBCAAIAFqIAE2AgALIAFBgAJPBEAgACABEGoPCyABQfgBcUGckMEAaiECAn9BpJLBACgCACIDQQEgAUEDdnQiAXFFBEBBpJLBACABIANyNgIAIAIMAQsgAigCCAshASACIAA2AgggASAANgIMIAAgAjYCDCAAIAE2AggPC0G4ksEAIAA2AgBBsJLBAEGwksEAKAIAIAFqIgE2AgAgACABQQFyNgIEIABBtJLBACgCAEcNAUGsksEA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BToAACAEKAIQIgBBhAFJDQEgABDYAQwBCyAAQYAoOwEACyAEQSBqJAALkwMBB38jAEHQAGsiAyQAIANBIGogAhD+AiADIAMpAyA3AiggA0EYaiADQShqEIcDIANBQGsgASADKAIYIAMoAhwQigMgAAJ/AkACQCADKAJARQRAIAMoAkQhAQwBCyADQThqIANByABqKAIANgIAIAMgAykCQDcDMCADKAIoIQIgAygCLCEJA0AgAiAJRg0CIAMgAjYCQCACQRhqIQIgA0EQaiEGIwBBEGsiASQAIANBMGoiBCgCCCEHIAFBCGogA0FAaygCACAEKAIAEE5BASEFIAEoAgwhCCABKAIIQQFxRQRAIARBBGogByAIEL8DIAQgB0EBajYCCEEAIQULIAYgCDYCBCAGIAU2AgAgAUEQaiQAIAMoAhBBAXFFDQALIAMoAhQhASADIAI2AiggAygCNCICQYQBSQ0AIAIQ2AELQQEMAQsgAyACNgIoIANByABqIANBOGooAgA2AgAgAyADKQMwNwNAIANBCGogA0FAaxCfAyADKAIMIQEgAygCCAs2AgAgACABNgIEIANB0ABqJAALmAICBH8BfiMAQSBrIgYkAAJAIAVFDQAgAiADaiIDIAJJDQAgBCAFakEBa0EAIARrca0gAyABKAIAIghBAXQiAiACIANJGyICQQhBBEEBIAVBgQhJGyAFQQFGGyIDIAIgA0sbIgOtfiIKQiCIpw0AIAqnIglBgICAgHggBGtLDQACfyAIRQRAIAZBGGohB0EADAELIAZBHGohByAGIAQ2AhggBiABKAIENgIUIAUgCGwLIQUgByAFNgIAIAZBCGogBCAJIAZBFGoQrAEgBigCCEEBRgRAIAYoAhAhAiAGKAIMIQcMAQsgBigCDCEEIAEgAzYCACABIAQ2AgRBgYCAgHghBwsgACACNgIEIAAgBzYCACAGQSBqJAALnQIBBH8jAEEQayICJAACQCABQYABTwRAIAJBDGoiBEECciEDIAJBADYCDAJAIAFBgBBPBEAgBEEDciEFIAFBgIAETwRAIAJBEGohAyACIAFBEnZB8AFyOgAMIAIgAUEGdkE/cUGAAXI6AA4gAiABQQx2QT9xQYABcjoADSAFIQQMAgsgAiABQQx2QeABcjoADCACIAFBBnZBP3FBgAFyOgANIAMhBCAFIQMMAQsgAkEMakEBciEEIAIgAUEGdkHAAXI6AAwLIAQgAUE/cUGAAXI6AAAgACACQQxqIAMQhQIMAQsgACgCCCIDIAAoAgBGBEAgAEGYsMAAEKkBCyAAKAIEIANqIAE6AAAgACADQQFqNgIICyACQRBqJABBAAudAgEEfyMAQRBrIgIkAAJAIAFBgAFPBEAgAkEMaiIEQQJyIQMgAkEANgIMAkAgAUGAEE8EQCAEQQNyIQUgAUGAgARPBEAgAkEQaiEDIAIgAUESdkHwAXI6AAwgAiABQQZ2QT9xQYABcjoADiACIAFBDHZBP3FBgAFyOgANIAUhBAwCCyACIAFBDHZB4AFyOgAMIAIgAUEGdkE/cUGAAXI6AA0gAyEEIAUhAwwBCyACQQxqQQFyIQQgAiABQQZ2QcABcjoADAsgBCABQT9xQYABcjoAACAAIAJBDGogAxCEAgwBCyAAKAIIIgMgACgCAEYEQCAAQfS2wAAQqQELIAAoAgQgA2ogAToAACAAIANBAWo2AggLIAJBEGokAEEAC50CAQR/IwBBEGsiAiQAAkAgAUGAAU8EQCACQQxqIgRBAnIhAyACQQA2AgwCQCABQYAQTwRAIARBA3IhBSABQYCABE8EQCACQRBqIQMgAiABQRJ2QfABcjoADCACIAFBBnZBP3FBgAFyOgAOIAIgAUEMdkE/cUGAAXI6AA0gBSEEDAILIAIgAUEMdkHgAXI6AAwgAiABQQZ2QT9xQYABcjoADSADIQQgBSEDDAELIAJBDGpBAXIhBCACIAFBBnZBwAFyOgAMCyAEIAFBP3FBgAFyOgAAIAAgAkEMaiADEIQCDAELIAAoAggiAyAAKAIARgRAIABB4LzAABCpAQsgACgCBCADaiABOgAAIAAgA0EBajYCCAsgAkEQaiQAQQALrgICA38BfiMAQUBqIgIkACACQQRqIAEQ4QECQCACKAIEQQFGBEAgAigCDCEBIAJBHGogAigCCBDAASACKAIcQYCAgIB4RgRAIAAgAigCIDYCBCAAQYGAgIB4NgIAIAFBhAFJDQIgARDYAQwCCyACQRhqIgMgAkEkaiIEKAIANgIAIAIgAikCHDcDECACQRxqIAEQwAEgAigCHEGAgICAeEYEQCAAIAIoAiA2AgQgAEGBgICAeDYCACACQRBqENADDAILIAJBPGogBCgCADYCACACQTBqIgEgAygCADYCACACIAIpAhw3AjQgACACKQMQIgU3AgAgAEEIaiABKQMANwIAIABBEGogAkE4aikDADcCACACIAU3AygMAQsgAEGAgICAeDYCAAsgAkFAayQAC68DAQd/IwBBQGoiAiQAIAJBEGogARD+AiACIAIpAxA3AhggAkEIaiACQRhqEIcDIAJBMGogAigCCCACKAIMEM8BAkAgAigCMEGAgICAeEYEQCAAIAIoAjQ2AgQgAEEGOgAADAELIAJBKGoiByACQThqKAIANgIAIAIgAikCMDcDIAJAIAIoAhgiASACKAIcIghGDQADQAJAIAIgATYCMCACQSBqIQQjAEEgayIDJAAgA0EIaiACQTBqEFECfyADLQAIQQZGBEAgAygCDAwBCyAEKAIIIgUgBCgCAEYEQCAEQZyZwAAQlAILIAQoAgQgBUEYbGoiBiADKQMINwMAIAZBEGogA0EYaikDADcDACAGQQhqIANBEGopAwA3AwAgBCAFQQFqNgIIQQALIANBIGokACIDDQAgCCABQRhqIgFHDQEMAgsLIAAgAzYCBCAAQQY6AAAgAiABQRhqNgIYIAJBIGoiABDCAiAAENMDDAELIAJBO2ogBygCADYAACAAQQQ6AAAgAiACKQMgNwAzIAAgAikAMDcAASAAQQhqIAJBN2opAAA3AAALIAJBQGskAAuMAgEDfyMAQYABayIEJAAgACgCACEAAn8CQCABKAIIIgJBgICAEHFFBEAgAkGAgIAgcQ0BIAAoAgBBASABEGIMAgsgACgCACEAQQAhAgNAIAIgBGpB/wBqIABBD3EiA0EwciADQdcAaiADQQpJGzoAACACQQFrIQIgAEEPSyAAQQR2IQANAAsgAUEBQeH5wABBAiACIARqQYABakEAIAJrEEMMAQsgACgCACEAQQAhAgNAIAIgBGpB/wBqIABBD3EiA0EwciADQTdqIANBCkkbOgAAIAJBAWshAiAAQQ9LIABBBHYhAA0ACyABQQFB4fnAAEECIAIgBGpBgAFqQQAgAmsQQwsgBEGAAWokAAuBAgIEfwF+IwBBIGsiBiQAAkAgBUUNACACIANqIgMgAkkNACAEIAVqQQFrQQAgBGtxrSADIAEoAgAiCEEBdCICIAIgA0kbIgJBCEEEQQEgBUGBCEkbIAVBAUYbIgMgAiADSxsiA61+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AgQgA0EoahBNIQIMAgsgBSgCBCIFKAIAIAUoAgQgAhDfAyECDAELIAUoAgQiBSgCACACIAUoAgQoAhARAQAhAgsgA0FAayQAIAIMAQtBAUEUQZzJwAAQjgMACwwXCyACQZS3wABBGBCbAwwWCyACQay3wABBGxCbAwwVCyACQce3wABBGhCbAwwUCyACQeG3wABBGRCbAwwTCyACQfq3wABBDBCbAwwSCyACQYa4wABBExCbAwwRCyACQZm4wABBExCbAwwQCyACQay4wABBDhCbAwwPCyACQbq4wABBDhCbAwwOCyACQci4wABBDBCbAwwNCyACQdS4wABBDhCbAwwMCyACQeK4wABBDhCbAwwLCyACQfC4wABBExCbAwwKCyACQYO5wABBGhCbAwwJCyACQZ25wABBPhCbAwwICyACQdu5wABBFBCbAwwHCyACQe+5wABBNBCbAwwGCyACQaO6wABBLBCbAwwFCyACQc+6wABBJBCbAwwECyACQfO6wABBDhCbAwwDCyACQYG7wABBExCbAwwCCyACQZS7wABBHBCbAwwBCyACQbC7wABBGBCbAwsEQEHItcAAQTcgBEEgakG4tcAAQcy2wAAQzQEACyAEQUBrIARBzABqKAIANgIAIAQgBCkCRDcDOCAEQTQ2AjQgBEE0NgIsIARBNTYCJCAEQQQ2AgwgBEHku8AANgIIIARCAzcCFCAEIABBEGo2AjAgBCAAQQxqNgIoIAQgBEE4aiIANgIgIAQgBEEgajYCECABKAIAIAEoAgQgBEEIahBNIABBAUEBENABIARB4ABqJAAL1gMCB38BfiMAQUBqIgEkAAJAIAAoAiAiAkUEQEEAIQAMAQsgACACQQFrNgIgAkAgABDxASIGBEAgAUEwaiAGQQhqIgcoAgA2AgAgASAGKQIANwMoIAFBFGohAyABQShqIgAoAgQhBAJAAkAgACgCCCIFIAAoAgAiAC8BkgNJBEAgACECDAELA0AgACgCiAIiAkUEQCADIAQ2AgggAyAANgIEIANBADYCAAwDCyAEQQFqIQQgAC8BkAMhBSAFIAIiAC8BkgNPDQALCyADIAU2AgggAyAENgIEIAMgAjYCAAsgASgCFCIADQFBuJ/AABDGAwALQcifwAAQxgMACyABIAEpAhg3AjggASAANgI0IAFBNGoiAigCCEEBaiED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QQxqIgIQrQMEQCAEQRBqIgEgAhCFAyAEQQA2AhwjAEFAaiICJAAgASgCAARAIAEoAggiAyABKAIEayIFQQAgAyAFTxshAwsgAkEQakGq1QIgAyADQarVAk8bQQhBGBCYASACKAIUIQMCQCACKAIQQQFHBEAgAkEANgIMIAIgAigCGDYCCCACIAM2AgQgAkEoaiABEJsBAkACQCACLQAoQQdHBEADQCACQSBqIAJBOGoiBikDADcDACACQRhqIAJBMGoiBykDADcDACACIAIpAygiCDcDECAIp0H/AXFBBkYNAiACKAIMIgMgAigCBEYEQCACQQRqQfSjwAAQlAILIAIoAgggA0EYbGoiBSACKQMoNwMAIAVBCGogBykDADcDACAFQRBqIAYpAwA3AwAgAiADQQFqNgIMIAJBKGogARCbASACLQAoQQdHDQALCyAAIAIoAiw2AgQgAEGAgICAeDYCACACQQRqIgAQwgIgABDTAwwBCyACQRBqEKoCIAAgAikCBDcCACAAQQhqIAJBDGooAgA2AgALIAJBQGskAAwBCyADIAIoAhhB5KPAABCOAwALDAELIARBEGogBEEMahB4IAQoAhAhAgJAAkACQCAELQAUIgNBAmsOAgEAAgsgAEGAgICAeDYCACAAIAI2AgQMAgsgBEEMaiAEQRBqQfiQwAAQVCEBIABBgICAgHg2AgAgACABNgIEDAELIwBB0ABrIgEkACABIANBAXE6ABAgASACNgIMIAFBIGpBAEEIQRgQmAEgASgCJCECAkAgASgCIEEBRwRAIAFBADYCHCABIAEoAig2AhggASACNgIUIAFBOGogAUEMahCTAQJAAkACQCABLQA4QQdHBEADQCABQTBqIAFByABqIgUpAwA3AwAgAUEoaiABQUBrIgYpAwA3AwAgASABKQM4Igg3AyAgCKdB/wFxQQZGDQIgASgCHCICIAEoAhRGBEAgAUEUakH0o8AAEJQCCyABKAIYIAJBGGxqIgMgASkDODcDACADQQhqIAYpAwA3AwAgA0EQaiAFKQMANwMAIAEgAkEBajYCHCABQThqIAFBDGoQkwEgAS0AOEEHRw0ACwsgACABKAI8NgIEIABBgICAgHg2AgAgAUEUaiIAEMICIAAQ0wMgASgCDCIAQYMBSw0BDAILIAFBIGoQqgIgACABKQIUNwIAIABBCGogAUEcaigCADYCACABKAIMIgBBhAFJDQELIAAQ2AELIAFB0ABqJAAMAQsgAiABKAIoQeSjwAAQjgMACwsgBCgCDCIAQYMBSwRAIAAQ2AELIARBIGokAAu6AQECfyMAQSBrIgMkAAJAAn9BACABIAEgAmoiAksNABpBAEEIIAIgACgCACIBQQF0IgQgAiAESxsiAiACQQhNGyIEQQBIDQAaQQAhAiADIAEEfyADIAE2AhwgAyAAKAIENgIUQQEFIAILNgIYIANBCGogBCADQRRqEKoBIAMoAghBAUcNASADKAIQIQAgAygCDAsgAEH418AAEI4DAAsgAygCDCEBIAAgBDYCACAAIAE2AgQgA0EgaiQAC9UBAQF/IAAoAgAiAEGcAmoQjwMgAEGoAmoQjwMgAEE4ahDGAiAAQfgBahDQAyAAQbQCahCPAyAAQcgBahCqAiAAQegAahDGAiAAQcACahCPAyAAQeABahCqAiAAQYQCaiIBELkCIAEQ1QMgAEGQAmoiARDCAiABENMDIAAoAuQCIgEEQCABEMUBIAFBsAFBCBC9AwsgAEHMAmoQjwMgAEGYAWoQxgIgAEHYAmoQjwMCQCAAQX9GDQAgACAAKAIEQQFrIgE2AgQgAQ0AIABB8AJBCBC9AwsLzAQCCn8BfiMAQRBrIgQkAAJAIAEoAiAiAkUEQCABKAIAIAFBADYCAEEBcQRAIAEoAgwhAiABKAIIIQUCQCABKAIEIgEEQCAEIAU2AgggBCABNgIEDAELIAIEQANAIAUoApgDIQUgAkEBayICDQALC0EAIQIgBEEANgIIIAQgBTYCBAsgBCACNgIMIwBBEGsiASQAIAFBBGogBEEEaiICKAIAIAIoAgQQiQIDQCABKAIEIgIEQCABQQRqIAIgASgCCBCJAgwBCwsgAUEQaiQACyAAQQA2AgAMAQsgASACQQFrNgIgIAEQ8QEiCARAIwBBMGsiAyQAIANBCGohBiMAQRBrIgkkACAIKAIEIQICQAJAIAgoAggiCiAIKAIAIgEvAZIDTwRAA0AgCUEEaiABIAIQiQIgCSgCBCIBRQ0CIAkoAgghAiAJKAIMIgogAS8BkgNPDQALCyAKQQFqIQcCQCACRQRAIAEhBQwBCyABIAdBAnRqQZgDaiEHIAIhCwNAIAcoAgAiBUGYA2ohByALQQFrIgsNAAtBACEHCyAGIAo2AhQgBiACNgIQIAYgATYCDCAGIAc2AgggBkEANgIEIAYgBTYCAAwBCyAGQQA2AgALIAlBEGokACADKAIIBEAgACADKQIUNwIAIANBKGogA0EQaigCACIBNgIAIABBCGogA0EcaigCADYCACADIAMpAggiDDcDICAIQQhqIAE2AgAgCCAMNwIAIANBMGokAAwCC0GQtcAAEMYDAAtBmL7AABDGAwALIARBEGokAAvsAQECfyMAQTBrIgIkAAJAIAApAwBC////////////AINCgICAgICAgPj/AFoEQCACQQE2AhQgAkHEwsAANgIQIAJCATcCHCACQcIANgIsIAIgADYCKCACIAJBKGo2AhggASgCACABKAIEIAJBEGoQTSEDDAELIAJBADoADCACIAE2AghBASEDIAJBATYCFCACQcTCwAA2AhAgAkIBNwIcIAJBwgA2AiwgAiAANgIoIAIgAkEoajYCGCACQQhqIAJBEGoQ1AMNACACLQAMRQRAIAFBzMLAAEECEJsDDQELQQAhAwsgAkEwaiQAIAMLsAEBB38gASgCACIFLwGSAyIJQQxsIQFBfyEDIAVBjAJqIQQgAigCCCEGIAIoAgQhBUEBIQgCQANAIAFFBEAgCSEDDAILIAQoAgghByAEKAIEIQIgA0EBaiEDIAFBDGshASAEQQxqIQQgBSACIAYgByAGIAdJGxCMAiICIAYgB2sgAhsiAkEASiACQQBIa0H/AXEiAkEBRg0ACyACDQBBACEICyAAIAM2AgQgACAINgIAC8UBAQJ/IwBBIGsiAiQAIAIgARCrAgJAAkAgAigCACIDQQJHBEAgAigCBCEBIANBAXFFDQEgAEEHOgAAIAAgATYCBAwCCyAAQQY6AAAMAQsgAkEIaiABED0gAi0ACCIBQQZHBEAgACACLwAJOwABIAAgAikDEDcDCCAAQQNqIAItAAs6AAAgAEEQaiACQRhqKQMANwMAIAAgAigCDDYCBCAAIAE6AAAMAQsgAigCDCEBIABBBzoAACAAIAE2AgQLIAJBIGokAAu2AQEDfyMAQSBrIgQkACAEIAI2AhwgBCABNgIYIARBEGogBEEYaiADEJIBIAQoAhQhBQJAIAQoAhBBAXFFDQADQCACRQRAQQEhBkEAIQIMAgsgASAFQQJ0aigCmAMhASAEIAJBAWsiAjYCHCAEIAE2AhggBEEIaiAEQRhqIAMQkgEgBCgCDCEFIAQoAghBAXENAAsLIAAgBTYCDCAAIAI2AgggACABNgIEIAAgBjYCACAEQSBqJAALwQECA38BfiMAQTBrIgIkACABKAIAQYCAgIB4RgRAIAEoAgwhAyACQRRqIgRBADYCACACQoCAgIAQNwIMIAJBIGogAygCACIDQQhqKQIANwMAIAJBKGogA0EQaikCADcDACACIAMpAgA3AxggAkEMakG4yMAAIAJBGGoQTRogAkEIaiAEKAIAIgM2AgAgAiACKQIMIgU3AwAgAUEIaiADNgIAIAEgBTcCAAsgAEGA08AANgIEIAAgATYCACACQTBqJAALyQEBAn8CQCAAKAIEQQFrIARGBEAgACgCACIALwGSAyIEQQtPDQEgACAEQQFqIgU7AZIDIAAgBEEMbGoiBiABKQIANwKMAiAGQZQCaiABQQhqKAIANgIAIAAgBEEYbGoiASACKQMANwMAIAFBCGogAkEIaikDADcDACABQRBqIAJBEGopAwA3AwAgACAFQQJ0aiADNgKYAyADIAU7AZADIAMgADYCiAIPC0HAssAAQTBB8LLAABCmAgALQbSxwABBIEGAs8AAEKYCAAuiAQEGfyABKAIAIgUvAZIDIglBDGwhBkF/IQEgBUGMAmohBUEBIQgCQANAIAZFBEAgCSEBDAILIAUoAgghBCAFKAIEIQcgAUEBaiEBIAZBDGshBiAFQQxqIQUgAiAHIAMgBCADIARJGxCMAiIHIAMgBGsgBxsiBEEASiAEQQBIa0H/AXEiBEEBRg0ACyAEDQBBACEICyAAIAE2AgQgACAINgIAC5gBAgF+AX8gAAJ/AkAgAiADakEBa0EAIAJrca0gAa1+IgRCIIhQBEAgBKciA0GAgICAeCACa00NAQsgAEEANgIEQQEMAQsgA0UEQCAAIAI2AgggAEEANgIEQQAMAQtB0Y7BAC0AABogAyACEKoDIgUEQCAAIAU2AgggACABNgIEQQAMAQsgACADNgIIIAAgAjYCBEEBCzYCAAvFAQECfyMAQdAAayICJAAgAkEQaiAAKAIAJQEQICACKAIQIQAgAiACKAIUIgM2AkwgAiAANgJIIAIgAzYCRCACQQhqIAJBxABqQazEwAAQ3wEgAkHHADYCNCACQQI2AhwgAkHUxcAANgIYIAJCATcCJCACIAIoAgwiADYCQCACIAIoAgg2AjwgAiAANgI4IAIgAkE4aiIANgIwIAIgAkEwajYCICABKAIAIAEoAgQgAkEYahBNIABBAUEBENABIAJB0ABqJAALpQEBAn8jAEEgayIEJAAgAiADIAEQ1wIiAQR/IARBCGogARD3AiAEKAIMIQEgBCgCCAVBAAshAyAEQRRqIAFBACADGyIBQQFBARCYASAEKAIYIQIgBCgCFEEBRwRAIAQoAhwhBSABBEAgBSADQQEgAxsgAfwKAAALIAAgATYCCCAAIAU2AgQgACACNgIAIARBIGokAA8LIAIgBCgCHEG0gsAAEI4DAAuiAQECfyMAQSBrIgIkAAJAAkAgASgCAEUNACACIAEQkwIgAigCAEEBcUUNACACKAIEIQMgASABKAIMQQFqNgIMIAJBCGogAxA9IAItAAhBBkYEQCAAIAIoAgw2AgQgAEEHOgAADAILIAAgAikDCDcDACAAQRBqIAJBGGopAwA3AwAgAEEIaiACQRBqKQMANwMADAELIABBBjoAAAsgAkEgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEGMAmoQjwMgAEGUAmogA0EcaigCADYCACAAIAMpAhQ3AowCIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEGYAmoQjwMgAEGgAmogA0EcaigCADYCACAAIAMpAhQ3ApgCIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEGkAmoQjwMgAEGsAmogA0EcaigCADYCACAAIAMpAhQ3AqQCIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEGwAmoQjwMgAEG4AmogA0EcaigCADYCACAAIAMpAhQ3ArACIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEG8AmoQjwMgAEHEAmogA0EcaigCADYCACAAIAMpAhQ3ArwCIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEHIAmoQjwMgAEHQAmogA0EcaigCADYCACAAIAMpAhQ3AsgCIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6MBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEEkahCPAyAAQSxqIANBHGooAgA2AgAgACADKQIUNwIkIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC6MBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEEwahCPAyAAQThqIANBHGooAgA2AgAgACADKQIUNwIwIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC6QBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEE8ahCPAyAAQcQAaiADQRxqKAIANgIAIAAgAykCFDcCPCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAulAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAAkAgAQRAIAMgASACEKACIANBFGogAygCACADKAIEEJMDDAELIANBgICAgHg2AhQLIABByABqEI8DIABB0ABqIANBHGooAgA2AgAgACADKQIUNwJIIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC6UBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEHUAGoQjwMgAEHcAGogA0EcaigCADYCACAAIAMpAhQ3AlQgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqELcCCyADQSBqJAALpQEBAX8jAEEgayIDJAAgA0EIaiAAEPgBIAMoAgghAAJAIAEEQCADIAEgAhCgAiADQRRqIAMoAgAgAygCBBCTAwwBCyADQYCAgIB4NgIUCyAAQeAAahCPAyAAQegAaiADQRxqKAIANgIAIAAgAykCFDcCYCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAulAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAAkAgAQRAIAMgASACEKACIANBFGogAygCACADKAIEEJMDDAELIANBgICAgHg2AhQLIABB7ABqEI8DIABB9ABqIANBHGooAgA2AgAgACADKQIUNwJsIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC5kBAQR/IwBBIGsiAiQAQQggACgCACIEQQF0IgMgA0EITRsiA0EASARAQQBBACABEI4DAAsgAiAEBH8gAiAENgIcIAIgACgCBDYCFEEBBSAFCzYCGCACQQhqIAMgAkEUahCqASACKAIIQQFGBEAgAigCDCACKAIQIAEQjgMACyACKAIMIQEgACADNgIAIAAgATYCBCACQSBqJAALiQEBAX8gAUEATgRAAn8gAigCBARAIAIoAggiAwRAIAIoAgAgA0EBIAEQlAMMAgsLQQEgAUUNABpB0Y7BAC0AABogAUEBEKoDCyICRQRAIAAgATYCCCAAQQE2AgQgAEEBNgIADwsgACABNgIIIAAgAjYCBCAAQQA2AgAPCyAAQQA2AgQgAEEBNgIAC8MBAQN/IwBBMGsiAiQAIAEoAiAiA0EBakEBdiADIAAoAgwbIgQgACgCCEsEQCACIAAgBCAAQRBqED4LIAJBIGogAUEYaikCADcDACACQRhqIAFBEGopAgA3AwAgAkEQaiABQQhqKQIANwMAIAIgAzYCKCACIAEpAgA3AwgjAEEwayIBJAAgAUEMaiIDIAJBCGpBJPwKAAAgAxCFASIDBEADQCAAIAMQVSABQQxqEIUBIgMNAAsLIAFBMGokACACQTBqJAALiAEBAX8gAAJ/An8CQCACQQBOBEAgAygCBARAIAMoAggiBARAIAMoAgAgBCABIAIQlAMMBAsLIAJFDQFB0Y7BAC0AABogAiABEKoDDAILIABBADYCBEEBDAILIAELIgNFBEAgACACNgIIIAAgATYCBEEBDAELIAAgAjYCCCAAIAM2AgRBAAs2AgALigEBA38jAEHwAmsiASQAIAFBuAFqIgIgABDoAQJ/IAEoArgBKALUAiIARQRAIAIQtQJBAAwBCyABQQhqIgIgABA/IAEoAqQBIQAgAUG4AWoiAxC1AkEAIABBgICAgHhGDQAaIAFBwAFqIAJBsAH8CgAAIAFBADYCuAEgAxClAkEIagsgAUHwAmokAAuvAQIBfwFvIwBBEGsiAiQAAkAgASgCACUBEBoEQCACQQRqIAEQ3QEgAEEIaiACQQxqKAIANgIAIAAgAikCBDcCAAwBCyABKAIAJQEQEARAIAEoAgAlARAZIQMQfCIBIAMmASACIAE2AgAgAkEEaiACEN0BIABBCGogAkEMaigCADYCACAAIAIpAgQ3AgAgAUGEAUkNASABENgBDAELIABBgICAgHg2AgALIAJBEGokAAuIAQECfyMAQUBqIgIkAAJAIAEEQCABQQhrIgMoAgBBAUcNASACQQhqIAFBOPwKAAAgA0EANgIAAkAgA0F/Rg0AIAFBBGsiASABKAIAQQFrIgE2AgAgAQ0AIANBwABBCBC9AwsgACACQRBqQTD8CgAAIAJBQGskAA8LENYDAAtBzoPAAEE/ENgDAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABB6AFqENADIABB8AFqIANBHGooAgA2AgAgACADKQIUNwLoASADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQjwELIANBIGokAAuMAQECfyMAQcABayICJAACQCABBEAgAUEIayIDKAIAQQFHDQEgAkEIaiABQbgB/AoAACADQQA2AgACQCADQX9GDQAgAUEEayIBIAEoAgBBAWsiATYCACABDQAgA0HAAUEIEL0DCyAAIAJBEGpBsAH8CgAAIAJBwAFqJAAPCxDWAwALQc6DwABBPxDYAwALkAEBAX8jAEEgayIDJAAgA0EIaiAAEPgBIAMoAgghACADIAEgAhCgAiADQRRqIAMoAgAgAygCBBCTAyAAQfgAahDQAyAAQYABaiADQRxqKAIANgIAIAAgAykCFDcCeCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABBhAFqENADIABBjAFqIANBHGooAgA2AgAgACADKQIUNwKEASADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABBkAFqENADIABBmAFqIANBHGooAgA2AgAgACADKQIUNwKQASADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABBnAFqENADIABBpAFqIANBHGooAgA2AgAgACAD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SSABEI8DIAJB2ABqIgUgAkGQAWopAwA3AwAgAkHQAGoiByACQYgBaikDADcDACACQcgAaiADKQMANwMAIAIgAikDeDcDQCABQQJBAUEBEJgBIAIoAqgBIQEgAigCpAFBAUcNAgwGCyACKAIQIQMgAiACKAIUQQAgARs2AmAgAiADNgJcIAIgATYCWCACQQA2AlQgAiABQQBHIgU2AlAgAiADNgJMIAIgATYCSCACQQA2AkQgAiAFNgJAIAJBIGohByACQUBrIQUjAEEgayIDJABB4JLBAAJ+QdiSwQAoAgBBAUYEQEHoksEAKQMAIRJB4JLBACkDAAwBCyADEIECQdiSwQBCATcDAEHoksEAIAMpAwgiEjcDACADKQMACyITQgF8NwMAIANBCGoiDUGYmsAAKQMANwMAIANBGGoiDiASNwMAIANBEGoiDyATNwMAIANBkJrAACkDADcDACMAQYABayIBJAAgBSgCICEGIAFBKGogBUEYaikCADcDACABQSBqIAVBEGopAgA3AwAgAUEYaiAFQQhqKQIANwMAIAEgBjYCMCABIAUpAgA3AxAgAUHQAGogAUEQahCQAQJAIAEoAlAiBkUNACABQdwAaiEQIAFBOGohBQNAIAUgBiABKAJYIgpBGGxqIggpAgA3AgAgBUEIaiILIAhBCGopAgA3AgAgBUEQaiIMIAhBEGopAgA3AgAgBiAKQQxsaiIGKAKMAiIIQYCAgIB4Rg0BIAYpApACIRIgAUHgAGogDCkCADcDACABQdgAaiIRIAspAgA3AwAgASAFKQIANwNQIAFBCGogAUHQAGoQ9wIgASgCDCEGIAEoAgghCiABIBI3AmwgASAINgJoAkACQCAKBEAgAUH0AGogBkEBQQEQmAEgASgCeCELIAEoAnRBAUcNASALIAEoAnxB4KLAABCOAwALIAFB6ABqENADIAFB0ABqEKgCDAELIAEoAnwhDCAGBEAgDCAKIAb8CgAACyABQdAAahCoAiABIBI3AlQgASAINgJQIAEgBjYCZCABIAw2AmAgASALNgJcIAFB8ABqIBEoAgA2AgAgASABKQJQNwNoIAFB9ABqIgYgAyABQegAaiAQEEkgASgCdEGAgICAeEYNACAGENADCyABQdAAaiABQRBqEJABIAEoAlAiBg0ACwsgAUEQahDjASABQYABaiQAIAdB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ADYAACAAQQU6AAAgAiABKQAMNwAEIAAgAikAATcAASAAQQhqIAJBCGopAAA3AAAgASgCAEGAgICAeEcEQCABQQFBARDQAQsgAkEQaiQAC2IBAn8CQCAAKAIAIgFBAUcNACAAKAIEDQAgACgCCCEBIAAoAgwiAgRAA0AgASgCmAMhASACQQFrIgINAAsLIABCADcCCCAAIAE2AgRBASEBIABBATYCAAsgAEEEakEAIAEbC2MCAX8BbyMAQRBrIgMkACABKAIAJQEgAigCACUBEBEhBBB8IgEgBCYBIANBCGoQzwJBASECIAACfyADKAIIQQFxBEAgAygCDAwBC0EAIQIgAQs2AgQgACACNgIAIANBEGokAAtjAgF/AW8jAEEQayIDJAAgASgCACUBIAIoAgAlARAbIQQQfCIBIAQmASADQQhqEM8CQQEhAiAAAn8gAygCCEEBcQRAIAMoAgwMAQtBACECIAELNgIEIAAgAjYCACADQRBqJAALXgEBfyMAQbABayICJAACQAJAIAFFBEAgAiAAELEBIAIQxQEMAQsgAEUNASACIABBCGsiADYCACAAIAAoAgBBAWsiADYCACAADQAgAhC3AgsgAkGwAWokAA8LENYDAAtdAQF/IwBBMGsiAiQAIAIgATYCDCACIAA2AgggAkECNgIUIAJB3JzAADYCECACQgE3AhwgAkEZNgIsIAIgAkEoajYCGCACIAJBCGo2AiggAkEQahCHASACQTBqJAALXQEBfyMAQTBrIgIkACACIAE2AgwgAiAANgIIIAJBAjYCFCACQYCdwAA2AhAgAkIBNwIcIAJBGTYCLCACIAJBKGo2AhggAiACQQhqNgIoIAJBEGoQhwEgAkEwaiQAC1sBAX8jAEEwayIDJAAgAyABNgIMIAMgADYCCCADQQE2AhQgA0HY9sAANgIQIANCATcCHCADIANBCGqtQoCAgICwDIQ3AyggAyADQShqNgIYIANBEGogAhDFAgALVwECfwJAAkAgAQRAIAFBCGsiAiACKAIAQQFqIgM2AgAgA0UNASABKAIADQIgACACNgIIIAAgATYCBCABQX82AgAgACABQQhqNgIADwsQ1gMLAAsQ1wMAC1gBAX8jAEEQayICJAAgAkEEaiAAEPgBIAIoAgQgAUEARzoAqAEgAigCCEEANgIAIAIoAgwiACAAKAIAQQFrIgA2AgAgAEUEQCACQQxqELcCCyACQRBqJAALWAEBfyMAQRBrIgIkACACQQRqIAAQ+AEgAigCBCABQQBHOgCpASACKAIIQQA2AgAgAigCDCIAIAAoAgBBAWsiADYCACAARQRAIAJBDGoQtwILIAJBEGokAAtYAQF/IwBBEGsiAiQAIAJBBGogABD4ASACKAIEIAFBAEc6AKoBIAIoAghBADYCACACKAIMIgAgACgCAEEBayIANgIAIABFBEAgAkEMahC3AgsgAkEQaiQAC1gBAX8jAEEQayICJAAgAkEEaiAAEPgBIAIoAgQgAUEARzoAqwEgAigCCEEANgIAIAIoAgwiACAAKAIAQQFrIgA2AgAgAEUEQCACQQxqELcCCyACQRBqJAALWAEBfyMAQRBrIgIkACACQQRqIAAQ+AEgAigCBCABQQBHOgCsASACKAIIQQA2AgAgAigCDCIAIAAoAgBBAWsiADYCACAARQRAIAJBDGoQtwILIAJBEGokAAtYAQF/IwBBEGsiAiQAIAJBBGogABD4ASACKAIEIAFBAEc6AK0BIAIoAghBADYCACACKAIMIgAgACgCAEEBayIANgIAIABFBEAgAkEMahC3AgsgAkEQaiQAC1MBA38jAEEQayICJAAgAiABNgIMIAJBDGoiAUEAEL4DIQMgAUEBEL4DIQEgAigCDCIEQYQBTwRAIAQQ2AELIAAgATYCBCAAIAM2AgAgAkEQaiQAC1QBAX8jAEEQayICJAAgAkEEaiAAEPgBIAIoAgQgATYCICACKAIIQQA2AgAgAigCDCIAIAAoAgBBAWsiADYCACAARQRAIAJBDGoQtwILIAJBEGokAAtVAQJ/IwBBEGsiASQAQdGOwQAtAAAaIAFBADoAD0EBQQEQqgMiAkUEQEEBQQEQ3QMACyAAIAFBD2qtNwMAIAAgAq03AwggAkEBQQEQvQMgAUEQaiQAC0UBAn8jAEHQAWsiASQAIAFBDGoiAiAAEOgBIAFBIGogASgCDBA/IAIQtAIgAUEANgIYIAFBGGoQpQJBCGogAUHQAWokAAtKAQJ/IwBBEGsiAiQAAn9BACAAKAIAIgNFDQAaIAIgAyAAKAIEIAEQlAFBACACKAIADQAaIAIoAgQgAigCDEEYbGoLIAJBEGokAAuEAQECfyACIAFrIgIgACgCACAAKAIIIgNrSwRAIwBBEGsiBCQAIARBCGogACADIAJBAUEBEG4gBCgCCCIDQYGAgIB4RwRAIAMgBCgCDEGQwMAAEI4DAAsgBEEQaiQAIAAoAgghAwsgAgRAIAAoAgQgA2ogASAC/AoAAAsgACACIANqNgIIC4QBAQJ/IAIgAWsiAiAAKAIAIAAoAggiA2tLBEAjAEEQayIEJAAgBEEIaiAAIAMgAkEBQQEQdSAEKAIIIgNBgYCAgHhHBEAgAyAEKAIMQbTGwAAQjgMACyAEQRBqJAAgACgCCCEDCyACBEAgACgCBCADaiABIAL8CgAACyAAIAIgA2o2AggLSQEDfwJAIAAoAhAiAUUNACABIAAoAggiAiAAKAIEIAFBAWpsakEBa0EAIAJrcSIDakEJaiIBRQ0AIAAoAgwgA2sgASACEL0DCwuWAgIBfgV/AkAgACgCBCIFRQ0AIAAoAgwiBgRAIAAoAgAiA0EIaiEEIAMpAwBCf4VCgIGChIiQoMCAf4MhAQNAIAFQBEADQCADQcACayEDIAQpAwAgBEEIaiEEQoCBgoSIkKDAgH+DIgFCgIGChIiQoMCAf1ENAAsgAUKAgYKEiJCgwIB/hSEBCyADIAF6p0EDdkFYbGpBKGsiAhDQAwJAAkACQAJAIAItABAOBQMDAwECAAsgAkEUahDOAQwCCyACQRRqENADDAELIAJBFGoiAhDCAiACENMDCyABQgF9IAGDIQEgBkEBayIGDQALCyAFIAVBKGxBL2pBeHEiA2pBCWoiAkUNACAAKAIAIANrIAJBCBC9AwsL4gECAX4FfwJAIAAoAgQiBEUNACAAKAIMIgUEQCAAKAIAIgJBCGohAyACKQMAQn+FQoCBgoSIkKDAgH+DIQEDQCABUARAA0AgAkHAAWshAiADKQMAIANBCGohA0KAgYKEiJCgwIB/gyIBQoCBgoSIkKDAgH9RDQALIAFCgIGChIiQoMCAf4UhAQsgAiABeqdBA3ZBaGxqQRhrIgYQ0AMgBkEMahDQAyABQgF9IAGDIQEgBUEBayIFDQALCyAEIARBGGxBH2pBeHEiAmpBCWoiA0UNACAAKAIAIAJrIANBCBC9AwsLTAEDfyABIQMgAiEEIAEoAogCIgUEQCABLwGQAyEEIAJBAWohAwsgAUHIA0GYAyACG0EIEL0DIAAgBTYCACAAIAOtIAStQiCGhDcCBAtHAQF/IAAoAgAgACgCCCIDayACSQRAIAAgAyACEH0gACgCCCEDCyACBEAgACgCBCADaiABIAL8CgAACyAAIAIgA2o2AghBAAtNAQF/IwBBMGsiASQAIAFBATYCDCABQYD3wAA2AgggAUIBNwIUIAEgAUEvaq1CgICAgJAMhDcDICABIAFBIGo2AhAgAUEIaiAAEMUCAAtDAQN/AkAgAkUNAANAIAAtAAAiBCABLQAAIgVGBEAgAEEBaiEAIAFBAWohASACQQFrIgINAQwCCwsgBCAFayEDCyADC0ECAX8BfgJAIAApAwAiAyABKQMAUQR/AkAgA6dBAWsOAgIAAgsgACsDCCABKwMIYQUgAgsPCyAAKQMIIAEpAwhRCzgBAn8CQCAAKAIEIgFFDQAgASABQQJ0QQtqQXhxIgJqQQlqIgFFDQAgACgCACACayABQQgQvQMLC0ABAX8gACgCACIAQRBqEIgCIABBMGoQ0AMCQCAAQX9GDQAgACAAKAIEQQFrIgE2AgQgAQ0AIABBwABBCBC9AwsLOgEBfyMAQSBrIgAkACAAQQA2AhggAEEBNgIMIABB2NbAADYCCCAAQgQ3AhAgAEEIakGM18AAEMUCAAtIAQF/IAAoAgAgACgCCCIDayACSQRAIAAgAyACEI4BIAAoAgghAwsgAgRAIAAoAgQgA2ogASAC/AoAAAsgACACIANqNgIIQQALTwECfyAAKAIEIQIgACgCACEDAkAgACgCCCIALQAARQ0AIANB1PnAAEEEIAIoAgwRAgBFDQBBAQ8LIAAgAUEKRjoAACADIAEgAigCEBEBAAtCAQF/IAEoAgQiAiABKAIITwR/QQAFIAEgAkEBajYCBCABKAIAKAIAIAIQkQMhAUEBCyECIAAgATYCBCAAIAI2AgALRgEBfyMAQRBrIgIkACACQQhqIAAgACgCAEEBQQhBGBBuIAIoAggiAEGBgICAeEcEQCAAIAIoAgwgARCOAwALIAJBEGokAAtJAQF/IwBBEGsiASQAIAFBCGogACAAKAIAQQFBBEEMEHUgASgCCCIAQYGAgIB4RwRAIAAgASgCDEH0o8AAEI4DAAsgAUEQaiQACz0BAn8jAEEQayIBJAAgAUEEaiICIAAQ6AEgASgCBC0A2QIhACACELUCIAFBEGokAEH///8HIAAgAEECRhsLQwEDfyMAQRBrIgEkACABQQRqIgIgABDoASABKAIEIgAoAgAhAyAAKAIEIAIQtQIgAUEQaiQAuEQAABAAAADwQSADGwtDAQN/IwBBEGsiASQAIAFBBGoiAiAAEOgBIAEoAgQiACgCCCEDIAAoAgwgAhC1AiABQRBqJAC4RAAAEAAAAPBBIAMbC0MBA38jAEEQayIBJAAgAUEEaiICIAAQ6AEgASgCBCIAKAIQIQMgACgCFCACELUCIAFBEGokALhEAAAQAAAA8EEgAxsLPQECfyMAQRBrIgEkACABQQRqIgIgABDoASABKAIELQDYAiEAIAIQtQIgAUEQaiQAQf///wcgACAAQQJGGwtDAQN/IwBBEGsiASQAIAFBBGoiAiAAEOgBIAEoAgQiACgCGCEDIAAoAhwgAhC1AiABQRBqJAC4RAAAEAAAAPBBIAMbC0MBA38jAEEQayIBJAAgAUEEaiICIAAQ6AEgASgCBCIAKAIgIQMgACgCJCACELUCIAFBEGokALhEAAAQAAAA8EEgAxsLsgMBBn8jAEEQayICJABB7I7BAC0AAEEDRwRAIAJBAToACyACIAJBC2o2AgwgAkEMaiEAIwBBIGsiASQAAkACQAJAAkACQAJAAkBB7I7BAC0AAEEBaw4DAgQBAAtB7I7BAEECOgAAIAAoAgAiAC0AACAAQQA6AABFDQIjAEEgayIAJAACQAJAAkBBiI/BACgCAEH/////B3EEQEHQksEAKAIADQELQfyOwQAoAgANAUGEj8EAKAIAIQNBhI/BAEGIpcAANgIAQYCPwQAoAgAhBEGAj8EAQQE2AgACQCAERQ0AIAMoAgAiBQRAIAQgBREFAAsgAygCBCIFRQ0AIAQgBSADKAIIEL0DCyAAQSBqJAAMAgsgAEEANgIYIABBATYCDCAAQejSwAA2AgggAEIENwIQIABBCGpB8NLAABDFAgsAC0HsjsEAQQM6AAALIAFBIGokAAwECyABQQA2AhggAUEBNgIMIAFBzKXAADYCCAwCC0HopsAAEMYDAAsgAUEANgIYIAFBATYCDCABQYymwAA2AggLIAFCBDcCECABQQhqQfikwAAQxQIACwsgAkEQaiQAC08BAn9B0Y7BAC0AABogASgCBCECIAEoAgAhA0EIQQQQqgMiAUUEQEEEQQgQ3QMACyABIAI2AgQgASADNgIAIABBkNPAADYCBCAAIAE2AgALPQECfyMAQRBrIgEkAEEBIQIgAC0AAARAIAFBCGogABD3AiABKAIMRSABKAIIQQBHcSECCyABQRBqJAAgAgtFAQF/IwBBIGsiAyQAIAMgAjYCHCADIAE2AhggAyACNgIUIANBCGogA0EUakGsxMAAEN8BIAAgAykDCDcDACADQSBqJAALOwEBfyMAQRBrIgIkACACIAEoAgAlARApIAAgAigCAAR+IAAgAisDCDkDCEIBBUIACzcDACACQRBqJAALOwEBfyMAQRBrIgIkACACIAEoAgAlARAsIAAgAigCAAR+IAAgAikDCDcDCEIBBUIACzcDACACQRBqJAALnnQDI38afgF8IAEoAggiA0GAgIABcSECIAArAwAhPyADQYCAgIABcUUEQCABIAJBAEchAUEAIQAjAEGAAWsiByQAID+9ISUCf0EDID+ZRAAAAAAAAPB/YQ0AGkECICVCgICAgICAgPj/AIMiJkKAgICAgICA+P8AUQ0AGiAlQv////////8HgyIpQoCAgICAgIAIhCAlQgGGQv7///////8PgyAlQjSIp0H/D3EiABsiJ0IBgyEoICZQBEBBBCApUA0BGiAAQbMIayEAQgEhJiAoUAwBC0KAgICAgICAICAnQgGGICdCgICAgICAgAhRIgIbISdCAkIBIAIbISZBy3dBzHcgAhsgAGohACAoUAshAiAHIAA7AXggByAmNwNwIAdCATcDaCAHICc3A2AgByACOgB6An8CQAJAAkAgAkECayICBEBBASEAQYP2wABBhPbAACAlQgBTIgMbQYP2wABBASADGyABGyEXICVCP4inIAFyIRtBAyACIAJBA08bQQJrDgIDAgELIAdBAzYCKCAHQYX2wAA2AiQgB0ECOwEgQQEhF0EBIQAgB0EgagwDCyAHQQM2AiggB0GI9sAANgIkIAdBAjsBICAHQSBqDAILIAdBIGohBiAHQQ9qIQwjAEEwayIDJAACQAJAAn8CQAJAAkACQAJAAkACQAJAIAdB4ABqIgApAwAiJVBFBEAgACkDCCInUA0BIAApAxAiJlANAiAlICZ8IiYgJVQNAyAlICdUDQQgJkKAgICAgICAgCBaDQUgAyAALwEYIgA7AQggAyAlICd9Iic3AwAgACAAQSBrIAAgJkKAgICAEFQiARsiAkEQayACICZC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ASkDoAM3AkwgAkEwaiABQbgDaikDADcDACACQThqIAFBwANqKQMANwMAIAJBQGsgAUHIA2opAwA3AwAgAkHUAGogAUGoA2ooAgA2AgAgASgC5AIhBCABKQLoAiEpIAIgKzcDqAIgAkGAgICAeCAQIBBBgYCAgHhGGzYCpAIgAkEAIBYgEUGAgICAeEYiCRs2AvABIAJBASAUIAkbNgLsASACQQAgESAJGzYC6AEgAiADOgC4ASACIAEvAeQDOwC5ASACQbsBaiABQeYDai0AADoAACACIAEpA9ADNwK8ASACQcQBaiABQdgDaikDADcCACACQcwBaiABQeADaigCADYCACACIAEpA/gDNwNYIAJB4ABqIAFBgARqKQMANwMAIAJB6ABqIAFBiARqKQMANwMAIAJB8ABqIAFBkARqKQMANwMAIAIgCjYCeCACICw3ArQCIAJBgICAgHggBiAYGzYCsAIgAiALOgDQASACQYQBaiABQfADaigCADYCACACIAEpA+gDNwJ8IAIgAS8BrAQ7ANEBIAJB0wFqIAFBrgRqLQAAOgAAIAJB5AFqIAFBqARqKAIANgIAIAJB3AFqIAFBoARqKQMANwIAIAIgASkDmAQ3AtQBIAIgLTcDwAIgAiAVNgK8AiACIAc2AogCIAIgGzYChAIgAiAaNgKAAiACIBM2AvwBIAIgEjYC+AEgAiAINgL0ASACQaABaiABQYgDaikDADcDACACQZgBaiABQYADaikDADcDACACQZABaiABQfgCaikDADcDACACIAEpA/ACNwOIASACIAU2AqgBIAJBtAFqIAFBuARqKAIANgIAIAIgASkDsAQ3AqwBIAJBAiAhICFBA0YbOgDZAiACQQIgICAgQQNGGzoA2AIgAiAZNgLUAiACICk3AswCIAJBgICAgHggBCAEQYGAgIB4Rhs2AsgCIAIgJDYCJCACICNBACAjQQJHGzYCICACICU2AhwgAiAiQQAgIkECRxs2AhggAiAmNgIUIAIgH0EAIB9BAkcbNgIQIAIgJzYCDCACIB5BACAeQQJHGzYCCCACICg2AgQgAiAdQQAgHUECRxs2AgAMGgsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEMsBIAEoAvQCIQMgASgC8AIiBkGBgICAeEYEQCACQQI2AgAgAiADNgIEDBoLIAEoAvgCIQcgASgC5AJBgYCAgHhHBEAgAUHkAmoQjwMLIAEgBzYC7AIgASADNgLoAiABIAY2AuQCDBYLDBsLIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDTASABKALwAiEDIAEoApADIgZBgYCAgHhGBEAgAkECNgIAIAIgAzYCBAwZCyABQZAEaiIHIARBGGooAgA2AgAgAUGIBGoiCCAEQRBqKQIANwMAIAFBgARqIhIgBEEIaikCADcDACABQbgDaiITIApBCGooAgA2AgAgASAEKQIANwP4AyABIAopAgA3A7ADIAEoAtACQYGAgIB4RwRAIAFBsAJqEMYCCyAWIAEpA/gDNwIAIBwgASkDsAM3AgAgFkEIaiASKQMANwIAIBZBEGogCCkDADcCACAWQRhqIAcoAgA2AgAgHEEIaiATKAIANgIAIAEgAzYCsAIgASAGNgLQAgwVCwwaCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQywEgASgC9AIhAyABKALwAiIGQYGAgIB4RgRAIAJBAjYCACACIAM2AgQMGAsgASgC+AIhByABKAKkAkGBgICAeEcEQCABQaQCahCPAwsgASAHNgKsAiABIAM2AqgCIAEgBjYCpAIMFAsMGQsgASgCHCABQQA2AhwEQCABKAIgIQZBACEFIwBBEGsiAyQAIAMgBjYCDCABQQhqIgcCfyADQQxqEMQCRQRAIwBBsAFrIgUkACAFIAYQNQJAAkACfyAFKAKcAUGAgICAeEYEQCAFKAIAIQZBAQwBC0HRjsEALQAAGkGwAUEIEKoDIgZFDQEgBiAFQbAB/AoAAEEACyEIIAMgBjYCBCADIAg2AgAgBUGwAWokAAwBC0EIQbABEN0DAAsgAygCACEFIAMoAgQMAQtBACAGQYQBSQ0AGiAGENgBQQALNgIEIAcgBTYCACADQRBqJAAgASgCDCEFIAEoAghBAXEEQCACQQI2AgAgAiAFNgIEQQAhBQwXCyABIAU2AqACQQEhBSABQQE2ApwCDBMLDBgLIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBCNASABKAL0AiEDIAEoAvACIgZBgICAgHhGBEAgAkECNgIAIAIgAzYCBAwWCyABKAL4AiEHIAEoApACQYCAgIB4RwRAIAFBkAJqIggQwgIgCBDTAwsgASAHNgKYAiABIAM2ApQCIAEgBjYCkAIMEgsMFwsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIwBIAEoAvQCIQMgASgC8AIiBkGAgICAeEYEQCACQQI2AgAgAiADNgIEDBULIAEoAvgCIQcgASgChAJBgICAgHhHBEAgAUGEAmoiCBC5AiAIENUDCyABIAc2AowCIAEgAzYCiAIgASAGNgKEAgwRCwwWCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQuAEgAS0A8AIiA0EHRgRAIAEoAvQCIQQgAkECNgIAIAIgBDYCBAwUCyABQbIDaiIGIBBBAmotAAA6AAAgAUGABGoiByAPQQhqKQMANwMAIAEgEC8AADsBsAMgASAPKQMANwP4AyABKAL0AiEIIAEtAOgBQQZxQQZHBEAgAUHoAWoQqAILIBUgAS8BsAM7AAAgGyABKQP4AzcDACAVQQJqIAYtAAA6AAAgG0EIaiAHKQMANwMAIAEgAzoA6AEgASAINgLsAQwQCwwVCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQywEgASgC9AIhAyABKALwAiIGQYGAgIB4RgRAIAJBAjYCACACIAM2AgQMEwsgASgC+AIhByABKALcAUGBgICAeEcEQCABQdwBahCPAwsgASAHNgLkASABIAM2AuABIAEgBjYC3AEMDwsMFAsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCISQgASgC8AIiI0ECRw0OIAJBAjYCACACICQ2AgQMEQsMEwsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCISUgASgC8AIiIkECRw0NIAJBAjYCACACICU2AgQMEAsMEgsgASgCHCABQQA2AhwEQCABQfACaiABKAIgENMBIAEoAvACIQMgASgCkAMiBkGBgICAeEYEQCACQQI2AgAgAiADNgIEDBALIAFBkARqIgcgBEEYaigCADYCACABQYgEaiIIIARBEGopAgA3AwAgAUGABGoiEiAEQQhqKQIANwMAIAFBuANqIhMgCkEIaigCADYCACABIAQpAgA3A/gDIAEgCikCADcDsAMgASgCyAFBgYCAgHhHBEAgAUGoAWoQxgILIBEgASkD+AM3AgAgFCABKQOwAzcCACARQQhqIBIpAwA3AgAgEUEQaiAIKQMANwIAIBFBGGogBygCADYCACAUQQhqIBMoAgA2AgAgASADNgKoASABIAY2AsgBDAwLDBELIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBC4ASABLQDwAiIDQQdGBEAgASgC9AIhBCACQQI2AgAgAiAENgIEDA8LIAFBsgNqIgYgEEECai0AADoAACABQYAEaiIHIA9BCGopAwA3AwAgASAQLwAAOwGwAyABIA8pAwA3A/gDIAEoAvQCIQggAS0AkAFBBnFBBkcEQCABQZABahCoAgsgDiABLwGwAzsAACAaIAEpA/gDNwMAIA5BAmogBi0AADoAACAaQQhqIAcpAwA3AwAgASADOgCQASABIAg2ApQBDAsLDBALIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDSASABLQDwAgRAIAEoAvQCIQQgAkECNgIAIAIgBDYCBAwOCyABLQDxAiEhDAoLDA8LIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDSASABLQDwAgRAIAEoAvQCIQQgAkECNgIAIAIgBDYCBAwNCyABLQDxAiEgDAkLDA4LIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDLASABKAL0AiEDIAEoAvACIgZBgYCAgHhGBEAgAkECNgIAIAIgAzYCBAwMCyABKAL4AiEHIAEoAoQBQYGAgIB4RwRAIAFBhAFqEI8DCyABIAc2AowBIAEgAzYCiAEgASAGNgKEAQwICwwNCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQwAEgASgC9AIhAyABKALwAiIGQYCAgIB4RgRAIAJBAjYCACACIAM2AgQMCwsgASgC+AIhByABQfgAahCPAyABIAc2AoABIAEgAzYCfCABIAY2AngMBwsMDAsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCISYgASgC8AIiH0ECRw0GIAJBAjYCACACICY2AgQMCQsMCwsgASgCHCABQQA2AhwEQCABQfACaiABKAIgENMBIAEoAvACIQMgASgCkAMiBkGBgICAeEYEQCACQQI2AgAgAiADNgIEDAkLIAFBkARqIgcgBEEYaigCADYCACABQYgEaiIIIARBEGopAgA3AwAgAUGABGoiEiAEQQhqKQIANwMAIAFBuANqIhMgCkEIaigCADYCACABIAQpAgA3A/gDIAEgCikCADcDsAMgASgCaEGBgICAeEcEQCABQcgAahDGAgsgCSABKQP4AzcCACALIAEpA7ADNwIAIAlBCGogEikDADcCACAJQRBqIAgpAwA3AgAgCUEYaiAHKAIANgIAIAtBCGogEygCADYCACABIAM2AkggASAGNgJoDAULDAoLIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDLASABKAL0AiEDIAEoAvACIgZBgYCAgHhGBEAgAkECNgIAIAIgAzYCBAwICyABKAL4AiEHIAEoAjxBgYCAgHhHBEAgAUE8ahCPAwsgASAHNgJEIAEgAzYCQCABIAY2AjwMBAsMCQsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCIScgASgC8AIiHkECRw0DIAJBAjYCACACICc2AgQMBgsMCAsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCISggASgC8AIiHUECRw0CIAJBAjYCACACICg2AgQMBQsMBwsgASgCHCABQQA2AhxFDQYgAUHwAmogASgCIBDLASABKAL0AiEDIAEoAvACIgZBgYCAgHhGBEAgAkECNgIAIAIgAzYCBAwECyABKAL4AiEHIAEoAjBBgYCAgHhHBEAgAUEwahCPAwsgASAHNgI4IAEgAzYCNCABIAY2AjALIAFB8AJqIAFBHGoQayABLQDwAkUNAAsLIAEoAvQCIQQgAkECNgIAIAIgBDYCBAsgASgC5AJBgYCAgHhHBEAgAUHkAmoQjwMLIAEoAtACQYGAgIB4RwRAIAFBsAJqEMYCCyABKAKkAkGBgICAeEcEQCABQaQCahCPAwsgBUEBcQRAIAFBoAJqKAIAIgUEQCAFQSRqEI8DIAUQhwIgBUH4AGoQ0AMgBUEwahCPAyAFQTxqEI8DIAVBhAFqENADIAVBkAFqENADIAVByABqEI8DIAVBnAFqENADIAVB1ABqEI8DIAVB4ABqEI8DIAVB7ABqEI8DIAVBsAFBCBC9AwsLIAEoApACQYCAgIB4RwRAIAFBkAJqIgUQwgIgBRDTAwsgASgChAJBgICAgHhHBEAgAUGEAmoiBRC5AiAFENUDCyABLQDoAUEGcUEGRwRAIAFB6AFqEKgCCyABKALcAUGBgICAeEcEQCABQdwBahCPAwsgASgCyAFBgYCAgHhHBEAgAUGoAWoQxgILIAEtAJABQQZxQQZHBEAgAUGQAWoQqAILIAEoAoQBQYGAgIB4RwRAIAFBhAFqEI8DCyABKAJ4QYCAgIB4RwRAIAFB+ABqENADCyABKAJoQYGAgIB4RwRAIAFByABqEMYCCyABKAI8QYGAgIB4RwRAIAFBPGoQjwMLIAEoAjBBgYCAgHhGDQAgAUEwahCPAwsgAUEcahC+AgsgAUHABGokAAwBC0H4psAAQTEQ2AMACyACKAIEIQUCQCACKAIAIgRBAkYEQCACIAU2AuACIAJBATYC9AIgAkHwh8AANgLwAiACQgE3AvwCIAJBATYCjAMgAiACQYgDaiIDNgL4AiACIAJB4AJqNgKIAyACQeQCaiIFIAJB8AJqIgQQXiAEIAUQ3AEgAigC9AIgAigC+AIQ5wMhBSAEENADIAIgBTYCiAMgAxDlAyAFQYQBTwRAIAUQ2AELIAIoAugCIAIoAuwCEOcDIQUgAkHkAmoQ0AMgAigC4AIiBEGEAU8EQCAEENgBCyAXQQI2AgAgFyAFNgIEDAELIBdBCGogAkEIakHYAvwKAAAgFyAFNgIEIBcgBDYCAAsgAkGQA2okACANAn8gDCgCCEECRgRAIAwoAgwhBEEBDAELIAxB8AJqIAxBCGpB4AL8CgAAIAxBADYC6AICf0HRjsEALQAAGkHwAkEIEKoDIgUEQCAFQoGAgIAQNwMAIAVBCGogDEHoAmpB6AL8CgAAIAUMAQtBCEHwAhDdAwALQQhqIQRBAAsiBTYCCCANIARBACAFGzYCBCANQQAgBCAFGzYCACAMQdAFaiQAIA0oAgAgDSgCBCANKAIIIA1BEGokAAuHAQEEfyMAQRBrIgEkACMAQSBrIgIkACACQRRqIgMgABDoASACQQhqIAIoAhRBuAFqENsBIAIoAgghBCACKAIMIQAgAxC1AiABIARBAXEiAzYCCCABIABBACADGzYCBCABQQAgACADGzYCACACQSBqJAAgASgCACABKAIEIAEoAgggAUEQaiQAC4cBAQR/IwBBEGsiASQAIwBBIGsiAiQAIAJBFGoiAyAAEOgBIAJBCGogAigCFEHQAWoQ2wEgAigCCCEEIAIoAgwhACADELUCIAEgBEEBcSIDNgIIIAEgAEEAIAMbNgIEIAFBACAAIAMbNgIAIAJBIGokACABKAIAIAEoAgQgASgCCCABQRBqJAAL+AMBDX8jAEEQayIEJAAjAEEgayIDJAAgA0EQaiIKIAAQ6AEgAygCECADQQA2AhwjAEEwayIAJABB9AFqIgIoAgQhASAAQSBqIANBHGpBASACKAIIIgIQigMgA0EIaiILAn8CQAJAIAAoAiBFBEAgACgCJCEBDAELIABBGGoiDCAAQShqIg0oAgA2AgAgACAAKQIgNwMQIAJFDQEgAkEMbCEHA0ACQCAAIAE2AiAjAEEQayICJAAgAEEQaiIFKAIIIQggAkEIaiAFKAIAIABBIGooAgAiBigCBCAGKAIIEJUDQQEhBiACKAIMIQkgAigCCEEBcUUEQCAFQQRqIAggCRC/AyAFIAhBAWo2AghBACEGCyAAQQhqIgUgCTYCBCAFIAY2AgAgAkEQaiQAIAAoAghBAXENACABQQxqIQEgB0EMayIHDQEMAwsLIAAoAgwhASAAKAIUIgJBhAFJDQAgAhDYAQtBAQwBCyANIAwoAgA2AgAgACAAKQMQNwMgIAAgAEEgahCfAyAAKAIEIQEgACgCAAs2AgAgCyABNgIEIABBMGokACADKAIIIQEgAygCDCEAIAoQtQIgBCABQQFxIgE2AgggBCAAQQAgARs2AgQgBEEAIAAgARs2AgAgA0EgaiQAIAQoAgAgBCgCBCAEKAIIIARBEGokAAuWAQEEfyMAQRBrIgIkACMAQSBrIgEkACABQRBqIgMgABDoASABKAIQIQAgAUEANgIcIAFBCGogAUEcaiAAQYACahBtIAEoAgghBCABKAIMIQAgAxC1AiACIARBAXEiAzYCCCACIABBACADGzYCBCACQQAgACADGzYCACABQSBqJAAgAigCACACKAIEIAIoAgggAkEQaiQAC9wBAQR/IwBBEGsiAiQAEHwiAyAAJgEjAEHwAmsiASQAIAFBCGogAxA1IAEoAgghAyACIAEoAqQBIgRBgICAgHhGBH9BAQUgAUHEAWogAUEIakEEckGYAfwKAAAgAUHoAmogAUGwAWopAwA3AwAgASABKQOoATcD4AIgASAENgLcAiABIAM2AsABIAFBADYCuAEgAUG4AWoQpQJBCGohA0EACyIENgIIIAIgA0EAIAQbNgIEIAJBACADIAQbNgIAIAFB8AJqJAAgAigCACACKAIEIAIoAgggAkEQaiQACx4AIAAoAgBBgICAgHhHBEAgABDQAyAAQQxqENADCwsoAQF/IAEoAgAgAUEANgIARQRAQfimwABBMRDYAwALIAAgASgCBBA9Cy0BAX5B8I7BACkDACEBQfCOwQBCADcDACAAIAFCIIg+AgQgACABp0EBRjYCAAsiAQF/IAAoAgAiACAAQR91IgJzIAJrIABBf3NBH3YgARBiC/UCAQh/IwBBEGsiBSQAEHwiBiABJgEjAEEgayIDJAAgA0EUaiAAEPgBIANBCGohBCADKAIUIQIjAEEwayIAJAAgACAGELgBAn8gAC0AACIGQQdGBEAgACgCBCECQQEMAQsgAEEuaiIHIAAtAAM6AAAgAEEgaiIIIABBEGopAwA3AwAgACAALwABOwEsIAAgACkDCDcDGCAAKAIEIQkgAi0AuAFBBkcEQCACQbgBahCoAgsgAiAGOgC4ASACIAAvASw7ALkBIAIgCTYCvAEgAiAAKQMYNwPAASACQbsBaiAHLQAAOgAAIAJByAFqIAgpAwA3AwBBAAshBiAEIAI2AgQgBCAGNgIAIABBMGokACADKAIMIQIgAygCCCEAIAMoAhhBADYCACADKAIcIgQgBCgCAEEBayIENgIAIARFBEAgA0EcahCPAQsgBSAANgIEIAUgAkEAIABBAXEbNgIAIANBIGokACAFKAIAIAUoAgQgBUEQaiQAC/UCAQh/IwBBEGsiBSQAEHwiBiABJgEjAEEgayIDJAAgA0EUaiAAEPgBIANBCGohBCADKAIUIQIjAEEwayIAJAAgACAGELgBAn8gAC0AACIGQQdGBEAgACgCBCECQQEMAQsgAEEuaiIHIAAtAAM6AAAgAEEgaiIIIABBEGopAwA3AwAgACAALwABOwEsIAAgACkDCDcDGCAAKAIEIQkgAi0A0AFBBkcEQCACQdABahCoAgsgAiAGOgDQASACIAAvASw7ANEBIAIgCTYC1AEgAiAAKQMYNwPYASACQdMBaiAHLQAAOgAAIAJB4AFqIAgpAwA3AwBBAAshBiAEIAI2AgQgBCAGNgIAIABBMGokACADKAIMIQIgAygCCCEAIAMoAhhBADYCACADKAIcIgQgBCgCAEEBayIENgIAIARFBEAgA0EcahCPAQsgBSAANgIEIAUgAkEAIABBAXEbNgIAIANBIGokACAFKAIAIAUoAgQgBUEQaiQAC5QCAQh/IwBBEGsiBSQAEHwiBiABJgEjAEEgayICJAAgAkEUaiAAEPgBIAJBCGohBCACKAIUIQMjAEEQayIAJAAgAEEEaiAGEIwBIAAoAgghBiAAKAIEIgdBgICAgHhGBH9BAQUgACgCDCEIIANB9AFqIgkQuQIgCRDVAyADIAg2AvwBIAMgBjYC+AEgAyAHNgL0AUEACyEDIAQgBjYCBCAEIAM2AgAgAEEQaiQAIAIoAgwhAyACKAIIIQAgAigCGEEANgIAIAIoAhwiBCAEKAIAQQFrIgQ2AgAgBEUEQCACQRxqEI8BCyAFIAA2AgQgBSADQQAgAEEBcRs2AgAgAkEgaiQAIAUoAgAgBSgCBCAFQRBqJAALlAIBCH8jAEEQayIFJAAQfCIGIAEmASMAQSBrIgIkACACQRRqIAAQ+AEgAkEIaiEEIAIoAhQhAyMAQRBrIgAkACAAQQRqIAYQjQEgACgCCCEGIAAoAgQiB0GAgICAeEYEf0EBBSAAKAIMIQggA0GAAmoiCRDCAiAJENMDIAMgCDYCiAIgAyAGNgKEAiADIAc2AoACQQALIQMgBCAGNgIEIAQgAzYCACAAQRBqJAAgAigCDCEDIAIoAgghACACKAIYQQA2AgAgAigCHCIEIAQoAgBBAWsiBDYCACAERQRAIAJBHGoQjwELIAUgADYCBCAFIANBACAAQQFxGzYCACACQSBqJAAgBSgCACAFKAIEIAVBEGokAAskACAAIAI2AgggACABNgIQIABBADYCACAAIAIgA0EDdGo2AgwLJgAgACABKAIEQQFrNgIEIAAgASgCACABKAIIQQJ0aigCmAM2AgALjAIBBX8gAi0AAEEFRgR/IwBBEGsiAyQAAn9BACACQQRqIgIoAgAiBUUNABogAigCBCEEIwBBIGsiAiQAIAIgBDYCHCACIAU2AhggAkEQaiACQRhqIAAgARCXASACKAIUIQYCQCACKAIQQQFxRQ0AA0AgBEUEQEEBIQdBACEEDAILIAUgBkECdGooApgDIQUgAiAEQQFrIgQ2AhwgAiAFNgIYIAJBCGogAkEYaiAAIAEQlwEgAigCDCEGIAIoAghBAXENAAsLIAMgBjYCDCADIAQ2AgggAyAFNgIEIAMgBzYCACACQSBqJABBACADKAIADQAaIAMoAgQgAygCDEEYbGoLIANBEGokAAVBAAsL7gQCA34NfyMAQRBrIgYkACMAQTBrIgQkACAEQRhqIgsgABDoASAEQSRqIQcgBCgCGCEAIwBBEGsiCCQAAkACQCAAKAIMRQ0AIABBEGogAEEgahBXIQEgACgCBCIKIAGncSEJIAFCGYhC/wCDQoGChIiQoMCAAX4hAyAAKAIAIgVBGGshDCAAKAIoIQ0gACgCJCEOQQAhACAIQQRqAn8CQANAAkAgBSAJaikAACICIAOFIgFCf4UgAUKBgoSIkKDAgAF9g0KAgYKEiJCgwIB/gyIBUEUEQANAIA4gDSAMQQAgAXqnQQN2IAlqIApxayIPQRhsaiIQKAIEIBAoAggQ+gINAiABQgF9IAGDIgFQRQ0ACwsgAiACQgGGg0KAgYKEiJCgwIB/g1BFDQIgCSAAQQhqIgBqIApxIQkMAQsLIAUgD0EYbGoMAQsgBSkDAEKAgYKEiJCgwIB/gyIBQoCBgoSIkKDAgH9RBEAgBUEIaiEAA0AgBUHAAWshBSAAKQMAIABBCGohAEKAgYKEiJCgwIB/gyIBQoCBgoSIkKDAgH9RDQALCyAFIAFCgIGChIiQoMCAf4V6p0EDdkFobGoLQQxrENwBIAgoAgRBgICAgHhGDQAgByAIKQIENwIAIAdBCGogCEEMaigCADYCAAwBCyAHQQA2AgggB0KAgICAEDcCAAsgCEEQaiQAIAsQtgIgBEEQaiAHQYiBwAAQ3wEgBEEIaiAEKAIQIAQoAhQQqwMgBCAEKAIIIAQoAgwQqwMgBCgCBCEAIAYgBCgCADYCACAGIAA2AgQgBEEwaiQAIAYoAgAgBigCBCAGQRBqJAALmQEBA38jAEEQayICJAAjAEEwayIBJAAgAUEYaiIDIAAQ6AEgAUEkaiIAIAEoAhhBIGoQ3AEgAxC2AiABQRBqIABBiIHAABDfASABQQhqIAEoAhAgASgCFBCrAyABIAEoAgggASgCDBCrAyABKAIEIQAgAiABKAIANgIAIAIgADYCBCABQTBqJAAgAigCACACKAIEIAJBEGokAAuaAQEDfyMAQRBrIgIkACMAQTBrIgEkACABQRhqIgMgABDoASABQSRqIgAgASgCGEHoAWoQ3AEgAxC1AiABQRBqIABBiIHAABDfASABQQhqIAEoAhAgASgCFBCrAyABIAEoAgggASgCDBCrAyABKAIEIQAgAiABKAIANgIAIAIgADYCBCABQTBqJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCjAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBjAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCmAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBmAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCpAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBpAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCsAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBsAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCvAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBvAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCyAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABByAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv7AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCJEGAgICAeEYEQCADELQCQQAhAEEADAELIAFBJGogAEEkahDcASABKAIkIQMgAUEwaiIEELQCQQAhAEEAIANBgICAgHhGDQAaIAFBOGogAUEsaigCADYCACABIAEpAiQ3AzAgAUEYaiAEQYiBwAAQ3wEgAUEQaiABKAIYIAEoAhwQqwMgASgCECEAIAEoAhQLIQMgAUEIaiAAIAMQqwMgASgCDCEAIAIgASgCCDYCACACIAA2AgQgAUFAayQAIAIo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QRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv8AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCbEGAgICAeEYEQCADELQCQQAhAEEADAELIAFBJGogAEHsAGoQ3AEgASgCJCEDIAFBMGoiBBC0AkEAIQBBACADQYCAgIB4Rg0AGiABQThqIAFBLGooAgA2AgAgASABKQIkNwMwIAFBGGogBEGIgcAAEN8BIAFBEGogASgCGCABKAIcEKsDIAEoAhAhACABKAIUCyEDIAFBCGogACADEKsDIAEoAgwhACACIAEoAgg2AgAgAiAANgIEIAFBQGskACACKAIAIAIoAgQgAkEQaiQACyUAIABFBEBBkK3AAEEyENgDAAsgACACIAMgBCAFIAEoAhARGwALHwECfiAAKQMAIgIgAkI/hyIDhSADfSACQgBZIAEQYQvIGAIMfwJ+EHwiBCAAJgEQfCIFIAEmASMAQYABayIDJAAgAyAFNgIEIAMgBDYCACADQQhqIAMQUwJAAkACQCADKAIIQYCAgIB4RwRAIANBKGogA0EQaigCADYCACADIAMpAgg3AyAgA0EBNgJsIANBqI/AADYCaCADQgE3AnQgA0EDNgJYIAMgA0HUAGo2AnAgAyADQSBqNgJUIANBOGoiBSADQegAaiICEF4gAygCPCADKAJAEOcDIQQgBRDQAyADIAQ2AmggAhDlAyAEQYQBTwRAIAQQ2AELIANBIGoQ0AMMAQsgAygCDCEEIANBCGogA0EEahBTIAMoAghBgICAgHhHBEAgA0EoaiADQRBqKAIANgIAIAMgAykCCDcDICADQQE2AmwgA0GEj8AANgJoIANCATcCdCADQQM2AlggAyADQdQAajYCcCADIANBIGo2AlQgA0E4aiICIANB6ABqIgYQXiADKAI8IAMoAkAQ5wMhBSACENADIAMgBTYCaCAGEOUDIAVBhAFPBEAgBRDYAQsgA0EgahDQAyAEQYQBSQ0BIAQQ2AEMAQsgAygCDCEFIANBOGogBBA9IAMtADhBBkYEQCADIAMoAjw2AmAgA0EBNgJs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QRBqEKgCCwsaACAAKAIAKAIAIAEoAgAgAkFYbGpBKGsQVwsaACAAKAIAKAIAIAEoAgAgAkFobGpBGGsQVwsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBELAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBEIAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBEvAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBExAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBEzAAshACAAIAEoAgw2AgQgACABKAIIQQAgAS0AAEEDRhs2AgALJgEBf0HRjsEALQAAGkGYA0EIEKoDIgBFBEBBCEGYAxDdAwALIAALJgEBf0HRjsEALQAAGkHIA0EIEKoDIgBFBEBBCEHIAxDdAwALIAALGQEBfyABIANGBH8gACACIAEQjAJFBSAECwsoAQF/IAAoAgAiAUGAgICAeHJBgICAgHhHBEAgACgCBCABQQEQvQMLCyEAIABFBEBBkK3AAEEyENgDAAsgACACIAMgASgCEBEDAAslAQF/EPgCIgFBADsBkgMgAUEANgKIAiAAQQA2AgQgACABNgIACyABAX8gACABKAIEIgI2AgAgACACIAEoAghBGGxqNgIECyIAIAAtAABFBEAgAUGG/MAAQQUQSg8LIAFBi/zAAEEEEEoLGwAgACgCACIABEAgABDFASAAQbABQQgQvQMLCxoAIAAoAgAoAgAgASgCACACQQJ0a0EEaxBWCxoAIAAgASACEMMBIgIEfyACBSAAIAMQiAELCxoAIAAgASACEMMBIgIEfyACBSAAIAMQ1gELCxoAIAAgASACEMMBIgIEfyACBSAAIAMQiQELCx4AIAAgASgCACUBEAU2AgggAEEANgIEIAAgATYCAAsfACAARQRAQZCtwABBMhDYAwALIAAgAiABKAIQEQEACxwAIABBATYCACAAIAEoAgQgASgCAGtBGG42AgQLKQAgACAALQAEIAFBLkZyOgAEIAAoAgAiACgCACABIAAoAgQoAhARAQALyAMCA34Gf0HUjsEAKAIARQRAIwBBMGsiBiQAAn8gAEUEQEHQqcAAIQRBAAwBCyAAKAIAIQQgAEEANgIAIABBCGpB0KnA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aiAJKAIAIAkoAgQQkwMMAQsgCUGAgICAeDYCDAsgCUEgaiEGIwBBkAFrIgQkACAEIAg2AgQCQAJAAkACQAJAAkAgCUEMaiIBKAIAQYCAgIB4RwRAIARBEGogAUEIaigCADYCACAEIAEpAgA3AwgMAQsgBEHIAGpBAkEBQQEQmAEgBCgCTCECIAQoAkhBAUYNASAEKAJQIgFB5dwBOwAAIARBAjYCECAEIAE2AgwgBCACNgIICyAEQcgAaiEHIAgQuAMhASMAQcABayIDJAAgAyABNgIMIANByABqIANBDGoQeCADKAJIIQECQAJAAkACQAJAAkACQAJAIAMtAEwiCkECaw4CAgABCyAHQQA2AgAgByABNgIEIAMoAgwiBUGDAUsNAwwECyADIAo6AHQgAyABNgJwIANBADYCaCADAn5B2JLBACgCAEEBRgRAQeiSwQApAwAhD0HgksEAKQMADAELIANByABqEIECQdiSwQBCATcDAEHoksEAIAMpA1AiDzcDACADKQNICyINNwNYQeCSwQAgDUIBfDcDACADIA83A2AgA0IANwNQIANBADYCTCADQeiSwAA2AkggA0GMAWohBQJAA0ACQCADQagBaiADQegAahCKAQJAAkACQAJAIAMoAqgBIgJBgICAgHhrDgIEAAELIAMoAqwBIQUMAQsgAyADKQKsASIPNwK4ASADIAI2ArQBIAMoAmggA0EANgJoRQ0JIANBEGogAygCbBDAASADKAIQQYCAgIB4Rw0BIAMoAhQhBSADQbQBahDQAwsgB0EANgIAIAcgBTYCBCADQcgAahCIAgwDCyADQTBqIANBGGooAgAiATYCACADQYgBaiAPQiCIpyIKNgIAIANBQGsgCjYCACADQaABaiABNgIAIAMgAykCECINNwMoIAVBCGogATYCACAFIA03AgAgAyACNgKAASADIA+nIgs2AoQBIAMgDTcDmAEgAyADKQKAATcDOCADQRBqIgEgA0HIAGogA0E4aiADQZgBahBJIAEQjwMMAQsLIAMgCjYCiAEgAyALNgKEASADQYCAgIB4NgKAASADQYABahDNAiAHQRhqIANB4ABqKQMANwMAIAdBEGogA0HYAGopAwA3AwAgB0EIaiADQdAAaikDADcDACAHIAMpA0g3AwALIANB6ABqEL0CDAELIAMgA0EMahDDAiAD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BEAgARDYAQsgAEHQAGokAAwBC0HsrsAAQTcgAEEoakHcrsAAQfCvwAAQzQEACwsHABAwEJ0CCwIACwv+jQEZAEGAgMAACwv//////////wAAEABBmIDAAAvVDy9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTA0L3NyYy9jb252ZXJ0L3NsaWNlcy5ycwAYABAAbwAAACMBAAAOAAAABQAAAAQAAAAEAAAABgAAAGNhbGxlZCBgUmVzdWx0Ojp1bndyYXAoKWAgb24gYW4gYEVycmAgdmFsdWUAAAAAAAQAAAAEAAAABwAAAEVycm9yL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMA6QAQAEoAAAC+AQAAHQAAAHNyYy9ncmFwaC5yc25vZGVpZGdyYXBoX2lkbm9kZWdyb3VwX2lkY29tcGFyZWNvcHkAAABEARAADAAAAGUEAAAUAAAARAEQAAwAAABtBAAAHAAAAEQBEAAMAAAAcwQAABQAAABEARAADAAAAI0EAAANAAAARAEQAAwAAACLBAAADQAAAGNvbmZpZ2F0dGVtcHRlZCB0byB0YWtlIG93bmVyc2hpcCBvZiBSdXN0IHZhbHVlIHdoaWxlIGl0IHdhcyBib3Jyb3dlZFN0YXRpY0dyYXBoTWV0YWF1dGhvcmNhcmRzY2FyZHNfeF9ub2Rlc194X3dpZGdldHNjb2xvcmRlc2NyaXB0aW9uZWRnZXNncmFwaGlkaWNvbmNsYXNzaXNfZWRpdGFibGVpc3Jlc291cmNlanNvbmxkY29udGV4dG5hbWVub2RlZ3JvdXBzbm9kZXNvbnRvbG9neV9pZHB1YmxpY2F0aW9ucmVsYXRhYmxlX3Jlc291cmNlX21vZGVsX2lkc3Jlc291cmNlXzJfcmVzb3VyY2VfY29uc3RyYWludHNyb290c2x1Z3N1YnRpdGxldmVyc2lvbhwCEAAGAAAAIgIQAAUAAAAnAhAAFwAAAD4CEAAFAAAAQwIQAAsAAABOAhAABQAAAFMCEAAHAAAAWgIQAAkAAABjAhAACwAAAG4CEAAKAAAAeAIQAA0AAACFAhAABAAAAIkCEAAKAAAAkwIQAAUAAACYAhAACwAAAKMCEAALAAAArgIQABwAAADKAhAAHwAAAOkCEAAEAAAA7QIQAAQAAADxAhAACAAAAPkCEAAHAAAAc3RydWN0IFN0YXRpY0dyYXBoTWV0YUZhaWxlZCB0byBkZXNlcmlhbGl6ZSBTdGF0aWNHcmFwaE1ldGE6IAAAAMYDEAAnAAAARAEQAAwAAAAfAQAALgAAAEQBEAAMAAAAKAEAAC0AAABEARAADAAAACsBAAAzAAAARAEQAAwAAAAwAQAAKwAAAEQBEAAMAAAAMgEAADEAAABEARAADAAAADsBAAA1AAAARAEQAAwAAAA+AQAALAAAAEQBEAAMAAAARwEAADMAAABEARAADAAAAEoBAAAzAAAARAEQAAwAAABNAQAARAAAAEQBEAAMAAAAUAEAAEcAAABEARAADAAAAFYBAAAsAAAARAEQAAwAAABZAQAAMAAAAEQBEAAMAAAAXAEAAC8AAABTdGF0aWNOb2RlYWxpYXNkYXRhdHlwZWV4cG9ydGFibGVmaWVsZG5hbWVoYXNjdXN0b21hbGlhc2lzX2NvbGxlY3RvcmlzcmVxdWlyZWRpc3NlYXJjaGFibGVpc3RvcG5vZGVvbnRvbG9neWNsYXNzcGFyZW50cHJvcGVydHlzb3J0b3JkZXJzb3VyY2VicmFuY2hwdWJsaWNhdGlvbl9pZAAAAOIEEAAFAAAAyAEQAAYAAADnBBAACAAAAEMCEAALAAAA7wQQAAoAAAD5BBAACQAAAFYBEAAIAAAAAgUQAA4AAAAQBRAADAAAABwFEAAKAAAAJgUQAAwAAAAyBRAACQAAAIUCEAAEAAAAXgEQAAwAAABQARAABgAAADsFEAANAAAASAUQAA4AAABWBRAACQAAAF8FEAAaAAAAc3RydWN0IFN0YXRpY05vZGUAAABEARAADAAAAHoCAAAtAAAARAEQAAwAAAB8AgAAKgAAAEQBEAAMAAAAfQIAACwAAABEARAADAAAAH8CAAAzAAAARAEQAAwAAACDAgAAMQAAAEQBEAAMAAAAhQIAACwAAABEARAADAAAAIsCAAAoAAAARAEQAAwAAACNAgAANAAAAEQBEAAMAAAAjwIAACoAAABEARAADAAAAJECAAA1AAAARAEQAAwAAACUAgAANgAAAEQBEAAMAAAAlgIAAC0AAABEARAADAAAAJgCAABCAAAARmFpbGVkIHRvIHNldCBjb25maWc6IAAA+AYQABYAAABGYWlsZWQgdG8gZGVzZXJpYWxpemUgbm9kZV9iOiAAABgHEAAeAAAARmFpbGVkIHRvIGRlc2VyaWFsaXplIG5vZGVfYTogAABABxAAHgAAAEZhaWxlZCB0byBub3JtYWxpemUgbm9kZV9iOiBoBxAAHAAAAEZhaWxlZCB0byBub3JtYWxpemUgbm9kZV9hOiCMBxAAHAAAAHRvSlNPTnRvSlNPTiBpcyBub3QgYSBmdW5jdGlvbkZhaWxlZCB0byBjYWxsIHRvSlNPTjogAAAAzgcQABcAQfiPwAALfQEAAAAIAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGVfanNvbi0xLjAuMTQ1L3NyYy92YWx1ZS9kZS5ycwAACBAAZwAAAHIAAAAZAEGAkcAACwUBAAAACQBBkJHAAAsFAQAAAAoAQaCRwAALBQEAAAALAEGwkcAACwUBAAAADABBwJHAAAsFAQAAAA0AQdCRwAALBQEAAAAOAEHgkcAACwUBAAAADQBB8JHAAAsFAQAAAA8AQYCSwAALBQEAAAAQAEGQksAAC2MBAAAACQAAAENvdWxkbid0IGRlc2VyaWFsaXplIGk2NCBvciB1NjQgZnJvbSBhIEJpZ0ludCBvdXRzaWRlIGk2NDo6TUlOLi51NjQ6Ok1BWCBib3VuZHMA//////////9oCRAAQYCTwAALowRhdXRob3JjYXJkc2NhcmRzX3hfbm9kZXNfeF93aWRnZXRzY29sb3JkZXNjcmlwdGlvbmVkZ2VzZ3JhcGhpZGljb25jbGFzc2lzX2VkaXRhYmxlaXNyZXNvdXJjZWpzb25sZGNvbnRleHRuYW1lbm9kZWdyb3Vwc25vZGVzb250b2xvZ3lfaWRwdWJsaWNhdGlvbnJlbGF0YWJsZV9yZXNvdXJjZV9tb2RlbF9pZHNyZXNvdXJjZV8yX3Jlc291cmNlX2NvbnN0cmFpbnRzcm9vdHNsdWdzdWJ0aXRsZXZlcnNpb25hbGlhc2NvbmZpZ2RhdGF0eXBlZXhwb3J0YWJsZWZpZWxkbmFtZWdyYXBoX2lkaGFzY3VzdG9tYWxpYXNpc19jb2xsZWN0b3Jpc3JlcXVpcmVkaXNzZWFyY2hhYmxlaXN0b3Bub2Rlbm9kZWdyb3VwX2lkbm9kZWlkb250b2xvZ3ljbGFzc3BhcmVudHByb3BlcnR5c29ydG9yZGVyc291cmNlYnJhbmNocHVibGljYXRpb25faWQvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZV9qc29uLTEuMC4xNDUvc3JjL3ZhbHVlL21vZC5ycwAbCxAAaAAAAHMAAAAKAAAAAAAAAP//////////mAsQAEG0l8AAC98CBAAAAAQAAAARAAAAEgAAABIAAAAAAAAABAAAAAQAAAATAAAAFAAAABQAAAAAAAAABAAAAAQAAAAVAAAAFgAAABYAAABzZXJpYWxpemVfdmFsdWUgY2FsbGVkIGJlZm9yZSBzZXJpYWxpemVfa2V5L2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGVfanNvbi0xLjAuMTQ1L3NyYy92YWx1ZS9zZXIucnMAIwwQAGgAAACqAQAAHwAAACMMEABoAAAATwEAABIAAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zbGljZS5ycwAArAwQAEoAAAC+AQAAHQAAAP//////////CA0QAEGgmsAAC+0CL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAACANEABKAAAAvgEAAB0AAABNYXBBY2Nlc3M6Om5leHRfdmFsdWUgY2FsbGVkIGJlZm9yZSBuZXh0X2tleS9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3NlcmRlX2NvcmUtMS4wLjIyOC9zcmMvZGUvdmFsdWUucnMAqA0QAGcAAABnBQAAGwAAAGludmFsaWQgdmFsdWU6ICwgZXhwZWN0ZWQgAAAgDhAADwAAAC8OEAALAAAAbWlzc2luZyBmaWVsZCBgYEwOEAAPAAAAWw4QAAEAAABkdXBsaWNhdGUgZmllbGQgYAAAAGwOEAARAAAAWw4QAAEAQZidwAAL7QcBAAAACgAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3NsaWNlLnJzAACgDhAASgAAAL4BAAAdAAAAL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAAPwOEABKAAAAqAEAAB8AAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9jb2xsZWN0aW9ucy9idHJlZS9uYXZpZ2F0ZS5ycwBYDxAAXwAAABYCAAAvAAAAWA8QAF8AAAChAAAAJAAAAC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3NlcmRlX2pzb24tMS4wLjE0NS9zcmMvdmFsdWUvbW9kLnJz2A8QAGgAAABzAAAACgAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL21hcC5ycwAAUBAQAFoAAADjAAAAOwAAAGludGVybmFsIGVycm9yOiBlbnRlcmVkIHVucmVhY2hhYmxlIGNvZGVQEBAAWgAAAOYAAAAsAAAAUBAQAFoAAAD6AAAAPwAAAFAQEABaAAAAHwEAAC4AAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zbGljZS5ycwAAFBEQAEoAAAC+AQAAHQAAAGEgc2VxdWVuY2UvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZV9jb3JlLTEuMC4yMjgvc3JjL2RlL2ltcGxzLnJzAAAAehEQAGcAAACKBAAAIgAAAHoREABnAAAAjQQAABwAAABhIG1hcC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL2NvbnNvbGVfZXJyb3JfcGFuaWNfaG9vay0wLjEuNy9zcmMvbGliLnJzAAkSEABuAAAAlQAAAA4AQZClwAALwwQBAAAAGgAAABsAAAAcAAAAT25jZSBpbnN0YW5jZSBoYXMgcHJldmlvdXNseSBiZWVuIHBvaXNvbmVkAACgEhAAKgAAAG9uZS10aW1lIGluaXRpYWxpemF0aW9uIG1heSBub3QgYmUgcGVyZm9ybWVkIHJlY3Vyc2l2ZWx51BIQADgAAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L3N0ZC9zcmMvc3luYy9wb2lzb24vb25jZS5ycwAUExAAUwAAAJsAAAAyAAAAY2FsbGVkIGBPcHRpb246OnVud3JhcF90aHJvdygpYCBvbiBhIGBOb25lYCB2YWx1ZU1hcCBrZXkgaXMgbm90IGEgc3RyaW5nIGFuZCBjYW5ub3QgYmUgYW4gb2JqZWN0IGtleWludmFsaWQgdHlwZTogLCBleHBlY3RlZCAAAADcExAADgAAAOoTEAALAAAAL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9zdGQvc3JjL3RocmVhZC9sb2NhbC5ycwAIFBAATwAAABUBAAAZAAAAL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAAGgUEABKAAAAvgEAAB0AAAAAAAAA///////////IFBAAQeCpwAAL+QQBAAAAAAAAACBjYW4ndCBiZSByZXByZXNlbnRlZCBhcyBhIEphdmFTY3JpcHQgbnVtYmVyAQAAAAAAAADoFBAALAAAACEAAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZS13YXNtLWJpbmRnZW4tMC42LjUvc3JjL2xpYi5ycygVEABoAAAANQAAAA4AAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9qcy1zeXMtMC4zLjgxL3NyYy9saWIucnMAAACgFRAAXQAAAOwZAAABAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvd2FzbS1iaW5kZ2VuLTAuMi4xMDQvc3JjL2NvbnZlcnQvc2xpY2VzLnJzABAWEABvAAAAIwEAAA4AAABjbG9zdXJlIGludm9rZWQgcmVjdXJzaXZlbHkgb3IgYWZ0ZXIgYmVpbmcgZHJvcHBlZC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTA0L3NyYy9jb252ZXJ0L3NsaWNlcy5ycwAAAMIWEABvAAAAIwEAAA4AAAAvAAAADAAAAAQAAAAwAAAAMQAAADIAQeSuwAAL0QYBAAAAMwAAAGEgRGlzcGxheSBpbXBsZW1lbnRhdGlvbiByZXR1cm5lZCBhbiBlcnJvciB1bmV4cGVjdGVkbHkvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAAKMXEABLAAAA0QoAAA4AAABFcnJvcgAAAKMXEABLAAAAfwUAABoAAACjFxAASwAAAH0FAAAbAAAAoxcQAEsAAABYBAAAEgAAAAoKU3RhY2s6CgoKCi9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL21hcC9lbnRyeS5yc0QYEABgAAAAoQEAAC4AAABhc3NlcnRpb24gZmFpbGVkOiBpZHggPCBDQVBBQ0lUWS9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25vZGUucnMA1BgQAFsAAACVAgAACQAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYuaGVpZ2h0IC0gMdQYEABbAAAArQIAAAkAAADUGBAAWwAAALECAAAJAAAAYXNzZXJ0aW9uIGZhaWxlZDogc3JjLmxlbigpID09IGRzdC5sZW4oKdQYEABbAAAASgcAAAUAAADUGBAAWwAAAMcEAAAjAAAA1BgQAFsAAAAKBQAAJAAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYubm9kZS5oZWlnaHQgLSAxAAAA1BgQAFsAAAD6AwAACQAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25hdmlnYXRlLnJzADAaEABfAAAAWAIAADAAAAA2AAAADAAAAAQAAAA3AAAAOAAAADkAQcC1wAAL8SIBAAAAOgAAAGEgRGlzcGxheSBpbXBsZW1lbnRhdGlvbiByZXR1cm5lZCBhbiBlcnJvciB1bmV4cGVjdGVkbHkvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAAP8aEABLAAAA0QoAAA4AAABFcnJvcgAAAP8aEABLAAAAfwUAABoAAAD/GhAASwAAAH0FAAAbAAAA/xoQAEsAAABYBAAAEgAAAEVPRiB3aGlsZSBwYXJzaW5nIGEgbGlzdEVPRiB3aGlsZSBwYXJzaW5nIGFuIG9iamVjdEVPRiB3aGlsZSBwYXJzaW5nIGEgc3RyaW5nRU9GIHdoaWxlIHBhcnNpbmcgYSB2YWx1ZWV4cGVjdGVkIGA6YGV4cGVjdGVkIGAsYCBvciBgXWBleHBlY3RlZCBgLGAgb3IgYH1gZXhwZWN0ZWQgaWRlbnRleHBlY3RlZCB2YWx1ZWV4cGVjdGVkIGAiYGludmFsaWQgZXNjYXBlaW52YWxpZCBudW1iZXJudW1iZXIgb3V0IG9mIHJhbmdlaW52YWxpZCB1bmljb2RlIGNvZGUgcG9pbnRjb250cm9sIGNoYXJhY3RlciAoXHUwMDAwLVx1MDAxRikgZm91bmQgd2hpbGUgcGFyc2luZyBhIHN0cmluZ2tleSBtdXN0IGJlIGEgc3RyaW5naW52YWxpZCB2YWx1ZTogZXhwZWN0ZWQga2V5IHRvIGJlIGEgbnVtYmVyIGluIHF1b3Rlc2Zsb2F0IGtleSBtdXN0IGJlIGZpbml0ZSAoZ290IE5hTiBvciArLy1pbmYpbG9uZSBsZWFkaW5nIHN1cnJvZ2F0ZSBpbiBoZXggZXNjYXBldHJhaWxpbmcgY29tbWF0cmFpbGluZyBjaGFyYWN0ZXJzdW5leHBlY3RlZCBlbmQgb2YgaGV4IGVzY2FwZXJlY3Vyc2lvbiBsaW1pdCBleGNlZWRlZEVycm9yKCwgbGluZTogLCBjb2x1bW46ICkAAADIHRAABgAAAM4dEAAIAAAA1h0QAAoAAADgHRAAAQAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3N0cmluZy5ycwAEHhAASwAAAH8FAAAaAAAABB4QAEsAAAB9BQAAGwAAAAQeEABLAAAAWAQAABIAAAA7AAAADAAAAAQAAAA8AAAAPQAAADkAAABhbnkgdmFsaWQgSlNPTiB2YWx1ZWEgc3RyaW5nIGtleS9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25hdmlnYXRlLnJzALgeEABfAAAAxgAAACcAAAC4HhAAXwAAABYCAAAvAAAAuB4QAF8AAAChAAAAJAAAAC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3NlcmRlX2pzb24tMS4wLjE0NS9zcmMvdmFsdWUvc2VyLnJzSB8QAGgAAADrAAAAEgAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJzwB8QAFAAAAAuAgAAEQAAAAAAAAAIAAAABAAAAEMAAABEAAAARQAAAGEgYm9vbGVhbmEgc3RyaW5nYnl0ZSBhcnJheWJvb2xlYW4gYGAAAABTIBAACQAAAFwgEAABAAAAaW50ZWdlciBgAAAAcCAQAAkAAABcIBAAAQAAAGZsb2F0aW5nIHBvaW50IGCMIBAAEAAAAFwgEAABAAAAY2hhcmFjdGVyIGAArCAQAAsAAABcIBAAAQAAAHN0cmluZyAAyCAQAAcAAAB1bml0IHZhbHVlT3B0aW9uIHZhbHVlbmV3dHlwZSBzdHJ1Y3RzZXF1ZW5jZW1hcGVudW11bml0IHZhcmlhbnRuZXd0eXBlIHZhcmlhbnR0dXBsZSB2YXJpYW50c3RydWN0IHZhcmlhbnQAAAABAAAAAAAAAC4waTMyL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvd2FzbS1iaW5kZ2VuLTAuMi4xMDQvc3JjL2NvbnZlcnQvc2xpY2VzLnJzbnVsbCBwb2ludGVyIHBhc3NlZCB0byBydXN0cmVjdXJzaXZlIHVzZSBvZiBhbiBvYmplY3QgZGV0ZWN0ZWQgd2hpY2ggd291bGQgbGVhZCB0byB1bnNhZmUgYWxpYXNpbmcgaW4gcnVzdAAAUSEQAG8AAADnAAAAAQAAAC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTA0L3NyYy9leHRlcm5yZWYucnMAADwiEABqAAAAfgAAABEAAAA8IhAAagAAAIsAAAARAAAASnNWYWx1ZSgpAAAAyCIQAAgAAADQIhAAAQAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJz5CIQAFAAAAAuAgAAEQAAAGxpYnJhcnkvc3RkL3NyYy9wYW5pY2tpbmcucnMvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAYCMQAEsAAAB9BQAAGwAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJzvCMQAFAAAAAuAgAAEQAAADoAAAABAAAAAAAAABwkEAABAAAAHCQQAAEAAABNAAAADAAAAAQAAABOAAAATwAAAFAAAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zbGljZS5ycwAAUCQQAEoAAAC+AQAAHQAAAC9ydXN0L2RlcHMvZGxtYWxsb2MtMC4yLjcvc3JjL2RsbWFsbG9jLnJzYXNzZXJ0aW9uIGZhaWxlZDogcHNpemUgPj0gc2l6ZSArIG1pbl9vdmVyaGVhZACsJBAAKQAAAKgEAAAJAAAAYXNzZXJ0aW9uIGZhaWxlZDogcHNpemUgPD0gc2l6ZSArIG1heF9vdmVyaGVhZAAArCQQACkAAACuBAAADQAAAEFjY2Vzc0Vycm9yY2Fubm90IGFjY2VzcyBhIFRocmVhZCBMb2NhbCBTdG9yYWdlIHZhbHVlIGR1cmluZyBvciBhZnRlciBkZXN0cnVjdGlvbjogAF8lEABIAAAAAQAAAAAAAABlbnRpdHkgbm90IGZvdW5kcGVybWlzc2lvbiBkZW5pZWRjb25uZWN0aW9uIHJlZnVzZWRjb25uZWN0aW9uIHJlc2V0aG9zdCB1bnJlYWNoYWJsZW5ldHdvcmsgdW5yZWFjaGFibGVjb25uZWN0aW9uIGFib3J0ZWRub3QgY29ubmVjdGVkYWRkcmVzcyBpbiB1c2VhZGRyZXNzIG5vdCBhdmFpbGFibGVuZXR3b3JrIGRvd25icm9rZW4gcGlwZWVudGl0eSBhbHJlYWR5IGV4aXN0c29wZXJhdGlvbiB3b3VsZCBibG9ja25vdCBhIGRpcmVjdG9yeWlzIGEgZGlyZWN0b3J5ZGlyZWN0b3J5IG5vdCBlbXB0eXJlYWQtb25seSBmaWxlc3lzdGVtIG9yIHN0b3JhZ2UgbWVkaXVtZmlsZXN5c3RlbSBsb29wIG9yIGluZGlyZWN0aW9uIGxpbWl0IChlLmcuIHN5bWxpbmsgbG9vcClzdGFsZSBuZXR3b3JrIGZpbGUgaGFuZGxlaW52YWxpZCBpbnB1dCBwYXJhbWV0ZXJpbnZhbGlkIGRhdGF0aW1lZCBvdXR3cml0ZSB6ZXJvbm8gc3RvcmFnZSBzcGFjZXNlZWsgb24gdW5zZWVrYWJsZSBmaWxlcXVvdGEgZXhjZWVkZWRmaWxlIHRvbyBsYXJnZXJlc291cmNlIGJ1c3lleGVjdXRhYmxlIGZpbGUgYnVzeWRlYWRsb2NrY3Jvc3MtZGV2aWNlIGxpbmsgb3IgcmVuYW1ldG9vIG1hbnkgbGlua3NpbnZhbGlkIGZpbGVuYW1lYXJndW1lbnQgbGlzdCB0b28gbG9uZ29wZXJhdGlvbiBpbnRlcnJ1cHRlZHVuc3VwcG9ydGVkdW5leHBlY3RlZCBlbmQgb2YgZmlsZW91dCBvZiBtZW1vcnlpbiBwcm9ncmVzc290aGVyIGVycm9ydW5jYXRlZ29yaXplZCBlcnJvciAob3MgZXJyb3IgKQAAAAEAAAAAAAAApSgQAAsAAACwKBAAAQAAAHBhbmlja2VkIGF0IDoKbWVtb3J5IGFsbG9jYXRpb24gb2YgIGJ5dGVzIGZhaWxlZNooEAAVAAAA7ygQAA0AAABsaWJyYXJ5L3N0ZC9zcmMvYWxsb2MucnMMKRAAGAAAAGQBAAAJAAAAY2Fubm90IG1vZGlmeSB0aGUgcGFuaWMgaG9vayBmcm9tIGEgcGFuaWNraW5nIHRocmVhZDQpEAA0AAAARCMQABwAAACQAAAACQAAAE0AAAAMAAAABAAAAFEAAAAAAAAACAAAAAQAAABSAAAAAAAAAAgAAAAEAAAAUwAAAFQAAABVAAAAVgAAAFcAAAAQAAAABAAAAFgAAABZAAAAWgAAAFsAAABvcGVyYXRpb24gc3VjY2Vzc2Z1bBAAAAARAAAAEgAAABAAAAAQAAAAEwAAABIAAAANAAAADgAAABUAAAAMAAAACwAAABUAAAAVAAAADwAAAA4A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dW5kczogdGhlIGxlbiBpcyAgYnV0IHRoZSBpbmRleCBpcyAAAACzOxAAIAAAANM7EAASAAAAAAAAAAQAAAAEAAAAZwAAAAAAAAAEAAAABAAAAGgAAAA9PSE9bWF0Y2hlc2Fzc2VydGlvbiBgbGVmdCAgcmlnaHRgIGZhaWxlZAogIGxlZnQ6IAogcmlnaHQ6IAAjPBAAEAAAADM8EAAXAAAASjwQAAkAAAAgcmlnaHRgIGZhaWxlZDogCiAgbGVmdDogAAAAIzwQABAAAABsPBAAEAAAAHw8EAAJAAAASjwQAAkAAAA6IAAAAQAAAAAAAACoPBAAAgAAAAAAAAAMAAAABAAAAGkAAABqAAAAawAAACAgICAsCn0gfSgoCiwweDAwMDEwMjAzMDQwNTA2MDcwODA5MTAxMTEyMTMxNDE1MTYxNzE4MTkyMDIxMjIyMzI0MjUyNjI3MjgyOTMwMzEzMjMzMzQzNTM2MzczODM5NDA0MTQyNDM0NDQ1NDY0NzQ4NDk1MDUxNTI1MzU0NTU1NjU3NTg1OTYwNjE2MjYzNjQ2NTY2Njc2ODY5NzA3MTcyNzM3NDc1NzY3Nzc4Nzk4MDgxODI4Mzg0ODU4Njg3ODg4OTkwOTE5MjkzOTQ5NTk2OTc5ODk5MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMGxpYnJhcnkvY29yZS9zcmMvZm10L21vZC5yc2ZhbHNldHJ1ZQDrPRAAGwAAANgKAAAmAAAA6z0QABsAAADhCgAAGgAAAGxpYnJhcnkvY29yZS9zcmMvc3RyL21vZC5yc1suLi5dYmVnaW4gPD0gZW5kICggPD0gKSB3aGVuIHNsaWNpbmcgYGAAUD4QAA4AAABePhAABAAAAGI+EAAQAAAAcj4QAAEAAABieXRlIGluZGV4ICBpcyBub3QgYSBjaGFyIGJvdW5kYXJ5OyBpdCBpcyBpbnNpZGUgIChieXRlcyApIG9mIGAAlD4QAAsAAACfPhAAJgAAAMU+EAAIAAAAzT4QAAYAAAByPhAAAQAAACBpcyBvdXQgb2YgYm91bmRzIG9mIGAAAJQ+EAALAAAA/D4QABYAAAByPhAAAQAAADA+EAAbAAAAnAEAACwAAABsaWJyYXJ5L2Nv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", import.meta.url);
    }
    const imports = __wbg_get_imports();
    if (typeof module_or_path === "string" || typeof Request === "function" && module_or_path instanceof Request || typeof URL === "function" && module_or_path instanceof URL) {
      module_or_path = fetch(module_or_path);
    }
    const { instance, module } = await __wbg_load(await module_or_path, imports);
    return __wbg_finalize_init(instance, module);
  }
  class StaticNodegroup {
    constructor(jsonData) {
      __publicField(this, "cardinality");
      __publicField(this, "legacygroupid");
      __publicField(this, "nodegroupid");
      __publicField(this, "parentnodegroup_id");
      this.legacygroupid = jsonData.legacygroupid;
      this.nodegroupid = jsonData.nodegroupid;
      this.parentnodegroup_id = jsonData.parentnodegroup_id;
      this.cardinality = jsonData.cardinality;
    }
    copy() {
      return new StaticNodegroup(this);
    }
  }
  class StaticConstraint {
    constructor(jsonData) {
      __publicField(this, "card_id");
      __publicField(this, "constraintid");
      __publicField(this, "nodes");
      __publicField(this, "uniquetoallinstances");
      this.card_id = jsonData.card_id;
      this.constraintid = jsonData.constraintid;
      this.nodes = jsonData.nodes;
      this.uniquetoallinstances = jsonData.uniquetoallinstances;
    }
  }
  class StaticCard {
    constructor(jsonData) {
      __publicField(this, "active");
      __publicField(this, "cardid");
      __publicField(this, "component_id");
      __publicField(this, "config");
      __publicField(this, "constraints");
      __publicField(this, "cssclass");
      __publicField(this, "description");
      __publicField(this, "graph_id");
      __publicField(this, "helpenabled");
      __publicField(this, "helptext");
      __publicField(this, "helptitle");
      __publicField(this, "instructions");
      __publicField(this, "is_editable");
      __publicField(this, "name");
      __publicField(this, "nodegroup_id");
      __publicField(this, "sortorder");
      __publicField(this, "visible");
      this.active = jsonData.active;
      this.cardid = jsonData.cardid;
      this.component_id = jsonData.component_id;
      this.config = jsonData.config;
      this.constraints = jsonData.constraints.map((constraint) => new StaticConstraint(constraint));
      this.cssclass = jsonData.cssclass;
      this.description = jsonData.description;
      this.graph_id = jsonData.graph_id;
      this.helpenabled = jsonData.helpenabled;
      this.helptext = new StaticTranslatableString(jsonData.helptext);
      this.helptitle = new StaticTranslatableString(jsonData.helptitle);
      this.instructions = new StaticTranslatableString(jsonData.instructions);
      this.is_editable = jsonData.is_editable;
      this.name = new StaticTranslatableString(jsonData.name);
      this.nodegroup_id = jsonData.nodegroup_id;
      this.sortorder = jsonData.sortorder;
      this.visible = jsonData.visible;
    }
  }
  class StaticCardsXNodesXWidgets {
    constructor(jsonData) {
      __publicField(this, "card_id");
      __publicField(this, "config");
      __publicField(this, "id");
      __publicField(this, "label");
      __publicField(this, "node_id");
      __publicField(this, "sortorder");
      __publicField(this, "visible");
      __publicField(this, "widget_id");
      this.card_id = jsonData.card_id;
      this.config = jsonData.config;
      this.id = jsonData.id;
      this.label = new StaticTranslatableString(jsonData.label);
      this.node_id = jsonData.node_id;
      this.sortorder = jsonData.sortorder;
      this.visible = jsonData.visible;
      this.widget_id = jsonData.widget_id;
    }
  }
  class StaticEdge {
    constructor(jsonData) {
      __publicField(this, "description");
      __publicField(this, "domainnode_id");
      __publicField(this, "edgeid");
      __publicField(this, "graph_id");
      __publicField(this, "name");
      __publicField(this, "ontologyproperty", null);
      __publicField(this, "rangenode_id");
      this.description = jsonData.description;
      this.domainnode_id = jsonData.domainnode_id;
      this.edgeid = jsonData.edgeid;
      this.graph_id = jsonData.graph_id;
      this.name = jsonData.name;
      this.rangenode_id = jsonData.rangenode_id;
      this.ontologyproperty = jsonData.ontologyproperty;
    }
    copy() {
      return new StaticEdge(this);
    }
  }
  class StaticFunctionsXGraphs {
    constructor(jsonData) {
      __publicField(this, "config");
      __publicField(this, "function_id");
      __publicField(this, "graph_id");
      __publicField(this, "id");
      this.config = jsonData.config;
      this.function_id = jsonData.function_id;
      this.graph_id = jsonData.graph_id;
      this.id = jsonData.id;
    }
    copy() {
      return new StaticFunctionsXGraphs(this);
    }
  }
  class StaticPublication {
    constructor(jsonData) {
      __publicField(this, "graph_id");
      __publicField(this, "notes");
      __publicField(this, "publicationid");
      __publicField(this, "published_time");
      this.graph_id = jsonData.graph_id;
      this.notes = jsonData.notes;
      this.publicationid = jsonData.publicationid;
      this.published_time = jsonData.published_time;
    }
    copy() {
      return new StaticPublication(this);
    }
  }
  class StaticGraph {
    constructor(jsonData) {
      __publicField(this, "author");
      __publicField(this, "cards", null);
      __publicField(this, "cards_x_nodes_x_widgets", null);
      __publicField(this, "color");
      __publicField(this, "config");
      __publicField(this, "deploymentdate", null);
      __publicField(this, "deploymentfile", null);
      __publicField(this, "description");
      __publicField(this, "edges");
      __publicField(this, "functions_x_graphs", null);
      __publicField(this, "graphid");
      __publicField(this, "iconclass");
      __publicField(this, "is_editable", null);
      __publicField(this, "isresource");
      __publicField(this, "jsonldcontext", null);
      __publicField(this, "name");
      __publicField(this, "nodegroups");
      __publicField(this, "nodes");
      __publicField(this, "ontology_id", null);
      __publicField(this, "publication", null);
      __publicField(this, "relatable_resource_model_ids");
      __publicField(this, "resource_2_resource_constraints", null);
      __publicField(this, "root");
      __publicField(this, "slug", null);
      __publicField(this, "subtitle");
      __publicField(this, "template_id");
      __publicField(this, "version");
      this.author = jsonData.author;
      this.cards = jsonData.cards && jsonData.cards.map((card) => new StaticCard(card));
      this.cards_x_nodes_x_widgets = jsonData.cards_x_nodes_x_widgets && jsonData.cards_x_nodes_x_widgets.map((card_x_node_x_widget) => new StaticCardsXNodesXWidgets(card_x_node_x_widget));
      this.color = jsonData.color;
      this.config = jsonData.config;
      this.deploymentdate = jsonData.deploymentdate;
      this.deploymentfile = jsonData.deploymentfile;
      this.description = jsonData.description instanceof StaticTranslatableString ? jsonData.description : new StaticTranslatableString(jsonData.description);
      this.edges = jsonData.edges.map((edge) => new StaticEdge(edge));
      this.functions_x_graphs = jsonData.functions_x_graphs && jsonData.functions_x_graphs.map((functions_x_graphs) => new StaticFunctionsXGraphs(functions_x_graphs));
      this.graphid = jsonData.graphid;
      this.iconclass = jsonData.iconclass;
      this.is_editable = jsonData.is_editable;
      this.isresource = jsonData.isresource;
      this.jsonldcontext = jsonData.jsonldcontext;
      this.name = jsonData.name instanceof StaticTranslatableString ? jsonData.name : new StaticTranslatableString(jsonData.name);
      this.nodegroups = jsonData.nodegroups.map((nodegroup) => new StaticNodegroup(nodegroup));
      this.nodes = jsonData.nodes.map((node) => new StaticNode(node));
      this.ontology_id = jsonData.ontology_id;
      this.publication = jsonData.publication && new StaticPublication(jsonData.publication);
      this.relatable_resource_model_ids = jsonData.relatable_resource_model_ids;
      this.resource_2_resource_constraints = jsonData.resource_2_resource_constraints;
      this.root = jsonData.root;
      this.slug = jsonData.slug;
      this.subtitle = jsonData.subtitle instanceof StaticTranslatableString ? jsonData.subtitle : new StaticTranslatableString(jsonData.subtitle);
      this.template_id = jsonData.template_id;
      this.version = jsonData.version;
    }
    copy() {
      var _a2, _b2, _c2, _d2, _e2, _f;
      const newGraph = new StaticGraph(this);
      Object.assign(newGraph, {
        author: this.author,
        cards: ((_a2 = this.cards) == null ? void 0 : _a2.map((card) => new StaticCard(card))) || [],
        cards_x_nodes_x_widgets: ((_b2 = this.cards_x_nodes_x_widgets) == null ? void 0 : _b2.map((cnw) => new StaticCardsXNodesXWidgets(cnw))) || [],
        color: this.color,
        config: Object.assign({}, this.config),
        deploymentdate: this.deploymentdate,
        deploymentfile: this.deploymentfile,
        description: this.description.copy && this.description.copy() || this.description,
        edges: this.edges.map((edge) => edge.copy && edge.copy() || edge),
        functions_x_graphs: ((_c2 = this.functions_x_graphs) == null ? void 0 : _c2.map((fxg) => fxg.copy())) || [],
        graphid: this.graphid,
        iconclass: this.iconclass,
        is_editable: this.is_editable,
        isresource: this.isresource,
        jsonldcontext: this.jsonldcontext,
        name: this.name.copy && this.name.copy() || this.name,
        nodegroups: (_d2 = this.nodegroups) == null ? void 0 : _d2.map((ng) => ng.copy && ng.copy() || ng),
        nodes: (_e2 = this.nodes) == null ? void 0 : _e2.map((n) => n.copy && n.copy() || n),
        ontology_id: this.ontology_id,
        publication: ((_f = this.publication) == null ? void 0 : _f.copy) && this.publication.copy() || null,
        relatable_resource_model_ids: [
          ...this.relatable_resource_model_ids || []
        ],
        resource_2_resource_constraints: [
          ...this.resource_2_resource_constraints || []
        ],
        root: this.root.copy && this.root.copy() || this.root,
        slug: this.slug,
        subtitle: this.subtitle.copy && this.subtitle.copy(),
        template_id: this.template_id,
        version: this.version
      });
      return newGraph;
    }
    static create(props, published = true) {
      const graphid = props.graphid || v4();
      const publication = published ? new StaticPublication({
        graph_id: graphid,
        notes: null,
        publicationid: v4(),
        published_time: (/* @__PURE__ */ new Date()).toISOString()
      }) : null;
      const name = props.name ? props.name instanceof StaticTranslatableString ? props.name : new StaticTranslatableString(props.name) : new StaticTranslatableString("");
      const alias = slugify(name);
      const root = new StaticNode({
        "alias": alias,
        "config": {},
        "datatype": "semantic",
        "description": "",
        "exportable": false,
        "fieldname": "",
        "graph_id": graphid,
        "hascustomalias": false,
        "is_collector": false,
        "isrequired": false,
        "issearchable": true,
        "istopnode": true,
        "name": name.toString(),
        "nodegroup_id": null,
        "nodeid": graphid,
        "ontologyclass": props.ontology_id || null,
        "parentproperty": null,
        "sortorder": 0,
        "sourcebranchpublication_id": null
      });
      return new StaticGraph({
        author: props.author,
        cards: null,
        cards_x_nodes_x_widgets: null,
        color: props.color || null,
        config: props.config || {},
        deploymentdate: props.deploymentdate || null,
        deploymentfile: props.deploymentfile || null,
        description: props.description ? props.description instanceof StaticTranslatableString ? props.description : new StaticTranslatableString(props.description) : null,
        edges: [],
        functions_x_graphs: [],
        graphid,
        iconclass: props.iconclass || "",
        is_editable: props.is_editable || null,
        isresource: props.isresource || null,
        jsonldcontext: props.jsonldcontext || null,
        name,
        nodegroups: [],
        nodes: [
          root.copy()
        ],
        ontology_id: props.ontology_id || null,
        publication,
        relatable_resource_model_ids: props.relatable_resource_model_ids || [],
        resource_2_resource_constraints: props.resource_2_resource_constraints || null,
        root,
        slug: props.slug || null,
        subtitle: props.subtitle ? props.subtitle instanceof StaticTranslatableString ? props.subtitle : new StaticTranslatableString(props.subtitle) : new StaticTranslatableString(""),
        template_id: props.template_id || "",
        version: props.version || ""
      });
    }
  }
  class StaticValue {
    constructor(jsonData, concept = null) {
      __publicField(this, "id");
      __publicField(this, "value");
      __publicField(this, "__concept");
      __publicField(this, "__conceptId");
      this.id = jsonData.id;
      this.value = jsonData.value;
      if (concept instanceof StaticConcept) {
        this.__concept = concept;
        this.__conceptId = concept ? concept.id : null;
      } else {
        this.__concept = null;
        this.__conceptId = concept;
      }
    }
    toString() {
      return this.value;
    }
    static create(referent, valueType, value, language) {
      const lang = language || getCurrentLanguage$1();
      const referentId = referent instanceof StaticConcept ? referent.id : referent;
      const concept = referent instanceof StaticConcept ? referent : null;
      const id = generateUuidv5([
        "value"
      ], `${referentId}/${valueType}/${value}/${lang}`);
      return new StaticValue({
        id,
        value
      }, concept);
    }
  }
  class StaticConcept {
    constructor(jsonData) {
      __publicField(this, "id");
      __publicField(this, "prefLabels");
      __publicField(this, "source");
      __publicField(this, "sortOrder");
      __publicField(this, "children");
      this.id = jsonData.id;
      this.prefLabels = jsonData.prefLabels;
      for (const [lang, value] of Object.entries(this.prefLabels)) {
        if (!(value instanceof StaticValue)) {
          this.prefLabels[lang] = new StaticValue(value, this);
        }
      }
      this.source = jsonData.source;
      this.sortOrder = jsonData.sortOrder;
      this.children = jsonData.children;
      if (this.children) {
        this.children = this.children.map((child) => {
          return child instanceof StaticConcept ? child : new StaticConcept(child);
        });
      }
    }
    getPrefLabel() {
      return this.prefLabels[getCurrentLanguage$1()] || Object.values(this.prefLabels)[0];
    }
    toString() {
      if (!this.getPrefLabel) {
        return this.constructor(this).getPrefLabel().value;
      }
      return this.getPrefLabel().value;
    }
    static fromValue(conceptScheme, value, children, config = {}) {
      let lang = (config == null ? void 0 : config.baseLanguage) || getCurrentLanguage$1();
      let tmpValue;
      let prefLabels;
      if (typeof value === "string") {
        tmpValue = value;
        prefLabels = {
          [lang]: new StaticValue({
            id: "",
            value: tmpValue
          })
        };
      } else if (value instanceof StaticValue) {
        tmpValue = value.value;
        prefLabels = {
          [lang]: value
        };
      } else if (lang in value) {
        tmpValue = value[lang].value;
        prefLabels = value;
      } else {
        const firstValue = Object.entries(value).sort()[0];
        if (firstValue === void 0) {
          throw Error("Cannot create a concept from values without a non-empty value");
        }
        lang = firstValue[0];
        tmpValue = firstValue[1].value;
        prefLabels = value;
      }
      const conceptId = generateUuidv5([
        "concept"
      ], `${(conceptScheme == null ? void 0 : conceptScheme.id) || "(none)"}/${tmpValue}`);
      const childConcepts = (children || []).map((child) => {
        if (!(child instanceof StaticConcept)) {
          return StaticConcept.fromValue(conceptScheme, value, [], {
            baseLanguage: config.baseLanguage
          });
        }
        return child;
      });
      return new StaticConcept({
        id: conceptId,
        prefLabels,
        source: config.source || null,
        sortOrder: config.sortOrder || null,
        children: childConcepts
      });
    }
  }
  class StaticCollection {
    constructor(jsonData) {
      __publicField(this, "id");
      __publicField(this, "prefLabels");
      __publicField(this, "concepts");
      __publicField(this, "__allConcepts");
      __publicField(this, "__values");
      this.id = jsonData.id;
      this.prefLabels = jsonData.prefLabels;
      this.concepts = jsonData.concepts;
      this.__allConcepts = {};
      this.__values = {};
      const addValues = (concept) => {
        this.__allConcepts[concept.id] = concept;
        for (const [, value] of Object.entries(concept.prefLabels)) {
          this.__values[value.id] = value;
        }
        if (concept.children) {
          for (let child of concept.children) {
            if (!(child instanceof StaticConcept)) {
              child = new StaticConcept(child);
            }
            addValues(child);
          }
        }
      };
      for (const [id, concept] of Object.entries(this.concepts)) {
        if (!(concept instanceof StaticConcept)) {
          this.concepts[id] = new StaticConcept(concept);
        }
        addValues(this.concepts[id]);
      }
    }
    static fromConceptScheme(props) {
      const collectionName = props.name ?? props.conceptScheme.toString();
      return StaticCollection.create({
        name: collectionName,
        concepts: props.conceptScheme.children || []
      });
    }
    static create(props) {
      let concepts = props.concepts;
      if (Array.isArray(concepts)) {
        concepts = concepts.reduce((acc, c) => {
          acc[c.id] = c;
          return acc;
        }, {});
      }
      const name = typeof props.name === "string" ? StaticValue.create("", "prefLabel", props.name) : props.name;
      let collectionid = props.collectionid;
      if (!collectionid) {
        if (typeof name === "string") {
          collectionid = generateUuidv5([
            "collection"
          ], name);
        } else if (name instanceof StaticValue) {
          collectionid = generateUuidv5([
            "collection"
          ], name.value);
        } else {
          throw Error("Must have a unique name to create a collection ID");
        }
      }
      const prefLabels = name instanceof StaticValue ? {
        [getCurrentLanguage$1()]: name
      } : name;
      return new StaticCollection({
        id: collectionid,
        prefLabels,
        concepts,
        __allConcepts: {},
        __values: {}
      });
    }
    getConceptValue(valueId) {
      return this.__values[valueId];
    }
    getConceptByValue(label) {
      var _a2;
      return (_a2 = Object.values(this.__values).find((value) => value.value == label)) == null ? void 0 : _a2.__concept;
    }
    toString() {
      return (this.prefLabels[getCurrentLanguage$1()] || Object.values(this.prefLabels)[0] || "").toString();
    }
  }
  class StaticTile {
    constructor(jsonData) {
      __publicField(this, "data");
      __publicField(this, "nodegroup_id");
      __publicField(this, "resourceinstance_id");
      __publicField(this, "tileid");
      __publicField(this, "parenttile_id", null);
      __publicField(this, "provisionaledits", null);
      __publicField(this, "sortorder", null);
      this.data = jsonData.data;
      if (typeof this.data === "object" && !(this.data instanceof Map)) {
        this.data = new Map(Object.entries(this.data));
      }
      this.nodegroup_id = jsonData.nodegroup_id;
      this.resourceinstance_id = jsonData.resourceinstance_id;
      this.tileid = jsonData.tileid;
      this.parenttile_id = jsonData.parenttile_id;
      this.provisionaledits = jsonData.provisionaledits;
      this.sortorder = jsonData.sortorder;
    }
    ensureId() {
      if (!this.tileid) {
        this.tileid = crypto.randomUUID();
      }
      return this.tileid;
    }
  }
  class StaticResourceDescriptors {
    constructor(jsonData) {
      __publicField(this, "name");
      __publicField(this, "map_popup");
      __publicField(this, "description");
      if (jsonData) {
        this.name = jsonData.name;
        this.map_popup = jsonData.map_popup;
        this.description = jsonData.description;
      }
    }
    isEmpty() {
      return !(this.name || this.map_popup || this.description);
    }
  }
  class StaticResourceMetadata {
    constructor(jsonData) {
      __publicField(this, "descriptors");
      __publicField(this, "graph_id");
      __publicField(this, "name");
      __publicField(this, "resourceinstanceid");
      __publicField(this, "publication_id", null);
      __publicField(this, "principaluser_id", null);
      __publicField(this, "legacyid", null);
      __publicField(this, "graph_publication_id", null);
      this.descriptors = jsonData.descriptors;
      if (!(this.descriptors instanceof StaticResourceDescriptors)) {
        if (jsonData.descriptors instanceof Map) {
          this.descriptors = new StaticResourceDescriptors(Object.fromEntries(jsonData.descriptors.entries()));
        } else {
          this.descriptors = new StaticResourceDescriptors(this.descriptors);
        }
      }
      this.graph_id = jsonData.graph_id;
      this.name = jsonData.name;
      this.resourceinstanceid = jsonData.resourceinstanceid;
      this.publication_id = jsonData.publication_id;
      this.principaluser_id = jsonData.principaluser_id;
      this.legacyid = jsonData.legacyid;
      this.graph_publication_id = jsonData.graph_publication_id;
    }
  }
  class StaticDomainValue {
    constructor(jsonData) {
      __publicField(this, "id");
      __publicField(this, "selected");
      __publicField(this, "text");
      this.id = jsonData.id;
      this.selected = jsonData.selected;
      this.text = jsonData.text;
    }
    toString() {
      const lang = getCurrentLanguage$1();
      let localized = this.text[lang];
      if (typeof localized !== "string") {
        localized = Object.values(this.text)[0];
      }
      if (!localized) {
        throw Error(`Could not render domain value ${this.id} in language ${lang}`);
      }
      return localized;
    }
    lang(lang) {
      return this.text[lang];
    }
    async forJson() {
      return {
        id: this.id,
        selected: this.selected,
        text: this.text
      };
    }
  }
  class StaticResourceReference {
    constructor(jsonData) {
      __publicField(this, "id");
      __publicField(this, "type");
      __publicField(this, "graphId");
      __publicField(this, "title");
      __publicField(this, "root");
      __publicField(this, "meta");
      this.id = jsonData.id;
      this.type = jsonData.type;
      this.graphId = jsonData.graphId;
      this.root = jsonData.root;
      this.title = jsonData.title;
      if (jsonData.meta) {
        this.meta = jsonData.meta;
      }
    }
  }
  class StaticResource {
    constructor(jsonData) {
      __publicField(this, "resourceinstance");
      __publicField(this, "tiles", null);
      __publicField(this, "metadata");
      __publicField(this, "__cache");
      __publicField(this, "__source");
      __publicField(this, "__scopes");
      this.resourceinstance = new StaticResourceMetadata(jsonData.resourceinstance);
      this.tiles = jsonData.tiles && jsonData.tiles.map((tile) => new StaticTile(tile));
      this.metadata = jsonData.metadata || {};
      this.__cache = jsonData.__cache;
      this.__scopes = jsonData.__scopes;
    }
  }
  staticTypes = Object.freeze(Object.defineProperty({
    __proto__: null,
    StaticCard,
    StaticCardsXNodesXWidgets,
    StaticCollection,
    StaticConcept,
    StaticConstraint,
    StaticDomainValue,
    StaticEdge,
    StaticFunctionsXGraphs,
    StaticGraph,
    StaticGraphMeta,
    StaticNode,
    StaticNodegroup,
    StaticResource,
    StaticResourceDescriptors,
    StaticResourceMetadata,
    StaticResourceReference,
    StaticTile,
    StaticTranslatableString,
    StaticValue
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  class GraphResult {
    constructor(jsonData) {
      __publicField(this, "models");
      this.models = Object.fromEntries(Object.entries(jsonData.models).map(([k, v]) => [
        k,
        new StaticGraphMeta(v)
      ]));
    }
  }
  class ArchesClient {
  }
  class ArchesClientRemote extends ArchesClient {
    constructor(archesUrl) {
      super();
      __publicField(this, "archesUrl");
      this.archesUrl = archesUrl;
    }
    async getGraphs() {
      const response = await fetch(`${this.archesUrl}/api/arches/graphs?format=arches-json&hide_empty_nodes=false&compact=false`);
      return await response.json();
    }
    async getGraph(graph) {
      return this.getGraphByIdOnly(graph.graphid);
    }
    async getGraphByIdOnly(graphId) {
      const response = await fetch(`${this.archesUrl}/graphs/${graphId}?format=arches-json&gen=`);
      return await response.json();
    }
    async getResource(resourceId) {
      throw Error(`Not implemented yet: getResource(${resourceId}`);
    }
    async getCollection(collectionId) {
      throw Error(`Not implemented yet: getCollection(${collectionId}`);
    }
    async getResources(graphId, limit) {
      const response = await fetch(`${this.archesUrl}/resources?graph_uuid=${graphId}&format=arches-json&hide_empty_nodes=false&compact=false&limit=${limit}`);
      return await response.json();
    }
  }
  class ArchesClientRemoteStatic extends ArchesClient {
    constructor(archesUrl, { allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile } = {}) {
      super();
      __publicField(this, "archesUrl");
      __publicField(this, "allGraphFile");
      __publicField(this, "graphToGraphFile");
      __publicField(this, "graphIdToGraphFile");
      __publicField(this, "graphIdToResourcesFiles");
      __publicField(this, "resourceIdToFile");
      __publicField(this, "collectionIdToFile");
      this.archesUrl = archesUrl;
      this.allGraphFile = allGraphFile || (() => "resource_models/_all.json");
      this.graphToGraphFile = graphToGraphFile;
      this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `resource_models/${graphId}.json`);
      this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [
        `business_data/_${graphId}.json`
      ]);
      this.resourceIdToFile = resourceIdToFile || ((resourceId) => `business_data/${resourceId}.json`);
      this.collectionIdToFile = collectionIdToFile || ((collectionId) => `collections/${collectionId}.json`);
    }
    async getGraphs() {
      const response = await fetch(`${this.archesUrl}/${this.allGraphFile()}`);
      return await response.json();
    }
    async getGraph(graph) {
      if (!this.graphToGraphFile) {
        return this.getGraphByIdOnly(graph.graphid);
      }
      const response = await fetch(`${this.archesUrl}/${this.graphToGraphFile(graph)}`);
      return (await response.json()).graph[0];
    }
    async getGraphByIdOnly(graphId) {
      const response = await fetch(`${this.archesUrl}/${this.graphIdToGraphFile(graphId)}`);
      return (await response.json()).graph[0];
    }
    async getResource(resourceId) {
      const source = `${this.archesUrl}/${this.resourceIdToFile(resourceId)}`;
      const response = await fetch(source);
      return response.json().then((response2) => {
        response2.__source = source;
        return response2;
      });
    }
    async getCollection(collectionId) {
      const response = await fetch(`${this.archesUrl}/${this.collectionIdToFile(collectionId)}`);
      return await response.json();
    }
    async getResources(graphId, limit) {
      const resources = [];
      for (const file of this.graphIdToResourcesFiles(graphId)) {
        const source = `${this.archesUrl}/${file}`;
        const response = await fetch(source);
        const resourceSet = (await response.json()).business_data.resources;
        for (const resource of resourceSet) {
          resource.__source = source;
        }
        resources.push(...limit ? resourceSet.slice(0, limit) : resourceSet);
        if (limit && resources.length > limit) {
          break;
        }
      }
      return resources;
    }
  }
  class ArchesClientLocal extends ArchesClient {
    constructor({ allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile } = {}) {
      super();
      __publicField(this, "fs");
      __publicField(this, "allGraphFile");
      __publicField(this, "graphToGraphFile");
      __publicField(this, "graphIdToGraphFile");
      __publicField(this, "graphIdToResourcesFiles");
      __publicField(this, "resourceIdToFile");
      __publicField(this, "collectionIdToFile");
      this.fs = import("./__vite-browser-external-DGN5jhtd.js");
      this.allGraphFile = allGraphFile || (() => "tests/definitions/models/_all.json");
      this.graphToGraphFile = graphToGraphFile || ((graph) => `tests/definitions/models/${graph.graphid}.json`);
      this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `tests/definitions/models/${graphId}.json`);
      this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [
        `tests/definitions/resources/_${graphId}.json`
      ]);
      this.resourceIdToFile = resourceIdToFile || ((resourceId) => `tests/definitions/resources/${resourceId}.json`);
      this.collectionIdToFile = collectionIdToFile || ((collectionId) => `tests/definitions/collections/${collectionId}.json`);
    }
    async getGraphs() {
      const fs = await this.fs;
      const response = await fs.promises.readFile(this.allGraphFile(), "utf8");
      return new GraphResult(await JSON.parse(response));
    }
    async getGraph(graph) {
      const fs = await this.fs;
      const graphFile = this.graphToGraphFile ? this.graphToGraphFile(graph) : this.graphIdToGraphFile(graph.graphid);
      if (!graphFile) {
        return null;
      }
      const response = await fs.promises.readFile(graphFile, "utf8");
      return await JSON.parse(response).graph[0];
    }
    async getGraphByIdOnly(graphId) {
      const fs = await this.fs;
      const graphFile = this.graphIdToGraphFile(graphId);
      if (!graphFile) {
        return null;
      }
      const response = await fs.promises.readFile(graphFile, "utf8");
      return await JSON.parse(response).graph[0];
    }
    async getResource(resourceId) {
      const fs = await this.fs;
      const source = this.resourceIdToFile(resourceId);
      const response = await fs.promises.readFile(source, "utf8");
      return JSON.parse(response).then((resource) => {
        resource.__source = source;
        return resource;
      });
    }
    async getCollection(collectionId) {
      const fs = await this.fs;
      const response = await fs.promises.readFile(this.collectionIdToFile(collectionId), "utf8");
      return await JSON.parse(response);
    }
    async getResources(graphId, limit) {
      const fs = await this.fs;
      const resources = [];
      for (const file of this.graphIdToResourcesFiles(graphId)) {
        const response = JSON.parse(await fs.promises.readFile(file, "utf8"));
        const source = file;
        const resourceSet = response.business_data.resources.filter((resource) => graphId === resource.resourceinstance.graph_id);
        for (const resource of resourceSet) {
          resource.__source = source;
        }
        resources.push(...limit ? resourceSet.slice(0, limit) : resourceSet);
        if (limit && resources.length > limit) {
          break;
        }
      }
      return resources;
    }
  }
  const archesClient = new ArchesClientRemote("http://localhost:8000");
  client = Object.freeze(Object.defineProperty({
    __proto__: null,
    ArchesClient,
    ArchesClientLocal,
    ArchesClientRemote,
    ArchesClientRemoteStatic,
    GraphResult,
    archesClient
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  interfaces = Object.freeze(Object.defineProperty({
    __proto__: null
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  class ReferenceDataManager {
    constructor(archesClient2) {
      __publicField(this, "archesClient");
      __publicField(this, "collections");
      this.archesClient = archesClient2;
      this.collections = /* @__PURE__ */ new Map();
    }
    retrieveCollection(id) {
      let collection = this.collections.get(id);
      if (collection !== void 0) {
        return collection;
      }
      collection = this.archesClient.getCollection(id).then((jsonData) => new StaticCollection(jsonData));
      this.collections.set(id, collection);
      return collection;
    }
  }
  RDM = new ReferenceDataManager(archesClient);
  class StaticStore {
    constructor(archesClient2, cacheMetadataOnly = true) {
      __publicField(this, "archesClient");
      __publicField(this, "cache");
      __publicField(this, "cacheMetadataOnly");
      this.archesClient = archesClient2;
      this.cache = /* @__PURE__ */ new Map();
      this.cacheMetadataOnly = cacheMetadataOnly;
    }
    async getMeta(id, onlyIfCached = true) {
      if (this.cache.has(id)) {
        const resource = this.cache.get(id);
        if (resource instanceof StaticResource) {
          return resource.resourceinstance;
        }
        return resource || null;
      }
      if (!onlyIfCached) {
        const resource = await this.loadOne(id);
        return resource.resourceinstance;
      }
      return null;
    }
    async *loadAll(graphId, limit = void 0) {
      const resourcesJSON = await this.archesClient.getResources(graphId, limit || 0);
      for (const resourceJSON of resourcesJSON.values()) {
        const resource = new StaticResource(resourceJSON);
        if (resource.resourceinstance.graph_id !== graphId) {
          continue;
        }
        this.cache.set(resource.resourceinstance.resourceinstanceid, this.cacheMetadataOnly ? resource.resourceinstance : resource);
        yield resource;
      }
    }
    async loadOne(id) {
      if (this.cache.has(id)) {
        const resource2 = this.cache.get(id);
        if (resource2 instanceof StaticResource) {
          return resource2;
        }
      }
      const resourceJSON = await this.archesClient.getResource(id);
      const resource = new StaticResource(resourceJSON);
      if (this.cacheMetadataOnly) {
        this.cache.set(id, this.cacheMetadataOnly ? resource.resourceinstance : resource);
      }
      return resource;
    }
  }
  staticStore = new StaticStore(archesClient);
  class CardComponent {
    constructor(id, name) {
      __publicField(this, "id");
      __publicField(this, "name");
      this.id = id;
      this.name = name;
    }
  }
  class Widget {
    constructor(id, name, datatype, defaultConfig) {
      __publicField(this, "id");
      __publicField(this, "name");
      __publicField(this, "datatype");
      __publicField(this, "defaultConfig");
      this.id = id;
      this.name = name;
      this.datatype = datatype;
      this.defaultConfig = defaultConfig;
    }
    getDefaultConfig() {
      return JSON.parse(this.defaultConfig);
    }
  }
  const DEFAULT_CARD_COMPONENT = new CardComponent("f05e4d3a-53c1-11e8-b0ea-784f435179ea", "Default Card");
  const _WIDGET_VALUES = [
    [
      "10000000-0000-0000-0000-000000000001",
      "text-widget",
      "string",
      '{ "placeholder": "Enter text", "width": "100%", "maxLength": null}'
    ],
    [
      "10000000-0000-0000-0000-000000000002",
      "concept-select-widget",
      "concept",
      '{ "placeholder": "Select an option", "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000012",
      "concept-multiselect-widget",
      "concept-list",
      '{ "placeholder": "Select an option", "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000015",
      "domain-select-widget",
      "domain-value",
      '{ "placeholder": "Select an option" }'
    ],
    [
      "10000000-0000-0000-0000-000000000016",
      "domain-multiselect-widget",
      "domain-value-list",
      '{ "placeholder": "Select an option" }'
    ],
    [
      "10000000-0000-0000-0000-000000000003",
      "switch-widget",
      "boolean",
      '{ "subtitle": "Click to switch"}'
    ],
    [
      "10000000-0000-0000-0000-000000000004",
      "datepicker-widget",
      "date",
      `{
      "placeholder": "Enter date",
      "viewMode": "days",
      "dateFormat": "YYYY-MM-DD",
      "minDate": false,
      "maxDate": false
    }`
    ],
    [
      "10000000-0000-0000-0000-000000000005",
      "rich-text-widget",
      "string",
      "{}"
    ],
    [
      "10000000-0000-0000-0000-000000000006",
      "radio-boolean-widget",
      "boolean",
      '{"trueLabel": "Yes", "falseLabel": "No"}'
    ],
    [
      "10000000-0000-0000-0000-000000000007",
      "map-widget",
      "geojson-feature-collection",
      `{
      "basemap": "streets",
      "geometryTypes": [{"text":"Point", "id":"Point"}, {"text":"Line", "id":"Line"}, {"text":"Polygon", "id":"Polygon"}],
      "overlayConfigs": [],
      "overlayOpacity": 0.0,
      "geocodeProvider": "MapzenGeocoder",
      "zoom": 0,
      "maxZoom": 20,
      "minZoom": 0,
      "centerX": 0,
      "centerY": 0,
      "pitch": 0.0,
      "bearing": 0.0,
      "geocodePlaceholder": "Search",
      "geocoderVisible": true,
      "featureColor": null,
      "featureLineWidth": null,
      "featurePointSize": null
    }`
    ],
    [
      "10000000-0000-0000-0000-000000000008",
      "number-widget",
      "number",
      '{ "placeholder": "Enter number", "width": "100%", "min":"", "max":""}'
    ],
    [
      "10000000-0000-0000-0000-000000000009",
      "concept-radio-widget",
      "concept",
      '{ "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000013",
      "concept-checkbox-widget",
      "concept-list",
      '{ "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000017",
      "domain-radio-widget",
      "domain-value",
      "{}"
    ],
    [
      "10000000-0000-0000-0000-000000000018",
      "domain-checkbox-widget",
      "domain-value-list",
      "{}"
    ],
    [
      "10000000-0000-0000-0000-000000000019",
      "file-widget",
      "file-list",
      '{"acceptedFiles": "", "maxFilesize": "200"}'
    ]
  ];
  const WIDGETS = Object.fromEntries(_WIDGET_VALUES.map((constructor) => [
    constructor[1],
    new Widget(...constructor)
  ]));
  function getDefaultWidgetForNode(node, preferences = {}) {
    const datatype = node.datatype;
    if (datatype in preferences) {
      return WIDGETS[preferences[datatype]];
    }
    if (datatype === "semantic") {
      throw Error("Not default widget for a semantic node");
    } else if (datatype === "number") {
      return WIDGETS["number-widget"];
    } else if (datatype === "string") {
      return WIDGETS["text-widget"];
    } else if (datatype === "concept") {
      return WIDGETS["concept-select-widget"];
    } else if (datatype === "concept-list") {
      return WIDGETS["concept-multiselect-widget"];
    } else if (datatype === "domain-value") {
      return WIDGETS["domain-select-widget"];
    } else if (datatype === "domain-value-list") {
      return WIDGETS["domain-multiselect-widget"];
    } else if (datatype === "geojson-feature-collection") {
      return WIDGETS["geojson-feature-collection"];
    } else if (datatype === "boolean") {
      return WIDGETS["switch-widget"];
    } else if (datatype === "date") {
      return WIDGETS["datepicker-widget"];
    } else {
      throw Error(`No default widget for ${datatype} datatype - perhaps you could supply a manual preference`);
    }
  }
  class StaticNodeConfigBoolean {
    constructor(jsonData) {
      __publicField(this, "i18n_properties");
      __publicField(this, "falseLabel");
      __publicField(this, "trueLabel");
      this.i18n_properties = jsonData.i18n_properties;
      this.falseLabel = jsonData.falseLabel;
      this.trueLabel = jsonData.trueLabel;
    }
  }
  class StaticNodeConfigConcept {
    constructor(jsonData) {
      __publicField(this, "rdmCollection");
      this.rdmCollection = jsonData.rdmCollection;
    }
  }
  class StaticNodeConfigDomain {
    constructor(jsonData) {
      __publicField(this, "i18n_config");
      __publicField(this, "options");
      this.i18n_config = jsonData.i18n_config;
      this.options = jsonData.options;
      if (this.options) {
        this.options = this.options.map((sdv) => {
          if (!(sdv instanceof StaticDomainValue)) {
            return new StaticDomainValue(sdv);
          }
          return sdv;
        });
      }
    }
    getSelected() {
      return this.options.find((option) => option.selected);
    }
    valueFromId(id) {
      return this.options.find((option) => option.id == id);
    }
  }
  const _NodeConfigManager = class _NodeConfigManager {
    constructor(cache = void 0) {
      __publicField(this, "cache");
      if (!cache) {
        cache = _NodeConfigManager._cache || /* @__PURE__ */ new Map();
      }
      this.cache = cache;
    }
    retrieve(node) {
      if (this.cache.has(node.nodeid)) {
        return this.cache.get(node.nodeid);
      }
      let nodeConfig2 = null;
      switch (node.datatype) {
        case "boolean":
          nodeConfig2 = new StaticNodeConfigBoolean(node.config);
          break;
        case "domain-value-list":
        case "domain-value":
          nodeConfig2 = new StaticNodeConfigDomain(node.config);
          break;
      }
      this.cache.set(node.nodeid, nodeConfig2);
      return nodeConfig2;
    }
  };
  __publicField(_NodeConfigManager, "_cache");
  let NodeConfigManager = _NodeConfigManager;
  const nodeConfigManager = new NodeConfigManager();
  nodeConfig = Object.freeze(Object.defineProperty({
    __proto__: null,
    StaticNodeConfigBoolean,
    StaticNodeConfigConcept,
    StaticNodeConfigDomain,
    nodeConfigManager
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  const DEFAULT_LANGUAGE = "en";
  class ViewContext {
    constructor() {
      __publicField(this, "graphManager");
    }
  }
  const viewContext = new ViewContext();
  function tileLoadingError(reason, exc) {
    {
      console.error(reason, exc);
      {
        throw exc;
      }
    }
  }
  class ValueList {
    constructor(values, allNodegroups, wrapper, tiles) {
      __publicField(this, "values");
      __publicField(this, "wrapper");
      __publicField(this, "tiles");
      __publicField(this, "promises");
      __publicField(this, "writeLock");
      this.values = values;
      this.wrapper = wrapper;
      this.tiles = tiles;
      this.promises = allNodegroups;
      this.writeLock = null;
    }
    async get(key) {
      return this.retrieve(key, this.values.get(key), true);
    }
    set(key, value) {
      this.values.set(key, value);
    }
    async has(key) {
      await this.retrieve(key, null);
      return this.values.has(key);
    }
    async retrieve(key, dflt = null, raiseError = false) {
      let result = this.values.get(key);
      if (Array.isArray(result)) {
        return result;
      }
      const node = this.wrapper.model.getNodeObjectsByAlias().get(key);
      result = await result;
      if (!node) {
        throw Error(`This key ${key} has no corresponding node`);
      }
      const nodegroupId = node.nodegroup_id || "";
      const promise = node ? await this.promises.get(nodegroupId) : false;
      if (promise === false) {
        await this.writeLock;
        if (this.wrapper.resource) {
          const node2 = this.wrapper.model.getNodeObjectsByAlias().get(key);
          if (node2 === void 0) {
            throw Error("Tried to retrieve a node key that does not exist on this resource");
          }
          const values = new Map([
            ...this.values.entries()
          ]);
          const promise2 = new Promise((resolve) => {
            return this.wrapper.ensureNodegroup(values, this.promises, nodegroupId, this.wrapper.model.getNodeObjects(), this.wrapper.model.getNodegroupObjects(), this.wrapper.model.getEdges(), false, this.tiles, true).then(async ([ngValues]) => {
              let original = false;
              const processValue = (k, concreteValue) => {
                if (key === k) {
                  original = concreteValue;
                }
                if (concreteValue !== false) {
                  this.values.set(k, concreteValue);
                }
              };
              return Promise.all([
                ...ngValues.entries()
              ].map(([k, value]) => {
                if (value instanceof Promise) {
                  return value.then((concreteValue) => processValue(k, concreteValue));
                }
                processValue(k, value);
              })).then(() => {
                resolve(original);
              });
            });
          });
          this.writeLock = promise2;
          this.promises.set(nodegroupId, promise2);
          this.values.set(key, promise2);
          await promise2;
          this.promises.set(nodegroupId, true);
        } else {
          this.values.delete(key);
        }
        result = await this.values.get(key);
      }
      result = await result;
      if (result === void 0 || result === false) {
        if (raiseError) {
          throw Error(`Unset key ${key}`);
        } else {
          return dflt;
        }
      }
      return result;
    }
    async setDefault(key, value) {
      const newValue = await this.retrieve(key, value, false);
      this.values.set(key, newValue);
      return newValue;
    }
  }
  class ConceptListCacheEntry {
    constructor({ meta, _ }) {
      __publicField(this, "datatype", "concept-list");
      __publicField(this, "_");
      __publicField(this, "meta");
      this._ = _.map((instance) => {
        if (instance instanceof ConceptValueCacheEntry) {
          return instance;
        } else if (instance) {
          return new ConceptValueCacheEntry(instance);
        }
        return null;
      }).filter((cvce) => cvce !== null);
      this.meta = meta || {};
    }
  }
  class ConceptValueCacheEntry {
    constructor({ meta, id, value, conceptId }) {
      __publicField(this, "datatype", "concept");
      __publicField(this, "id");
      __publicField(this, "value");
      __publicField(this, "conceptId");
      __publicField(this, "meta");
      this.id = id;
      this.value = value;
      this.conceptId = conceptId;
      this.meta = meta || {};
    }
  }
  class ResourceInstanceListCacheEntry {
    constructor({ meta, _ }) {
      __publicField(this, "datatype", "resource-instance-list");
      __publicField(this, "_");
      __publicField(this, "meta");
      this._ = _.map((instance) => {
        if (instance instanceof ResourceInstanceCacheEntry) {
          return instance;
        }
        return new ResourceInstanceCacheEntry(instance);
      });
      this.meta = meta || {};
    }
  }
  class ResourceInstanceCacheEntry {
    constructor({ meta, id, type, graphId, title }) {
      __publicField(this, "datatype", "resource-instance");
      __publicField(this, "id");
      __publicField(this, "type");
      __publicField(this, "graphId");
      __publicField(this, "title");
      __publicField(this, "meta");
      this.id = id;
      this.type = type;
      this.graphId = graphId;
      this.meta = meta || {};
      this.title = this.meta.title || title;
    }
  }
  class ResourceInstanceListViewModel extends Array {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value", null);
    }
    async forJson() {
      const value = await this._value;
      return value ? value.map((v) => v ? v.forJson() : null) : null;
    }
    async __forJsonCache(getMeta) {
      return new ResourceInstanceListCacheEntry({
        meta: getMeta ? await getMeta(this) : getMeta,
        _: await Promise.all([
          ...this.values()
        ].map(async (rivmPromise) => {
          const rivm = await rivmPromise;
          return await rivm.__forJsonCache(getMeta);
        }))
      });
    }
    static async __create(tile, node, value, cacheEntry = null) {
      const nodeid = node.nodeid;
      let val;
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          tile.data.set(nodeid, []);
          if (!Array.isArray(value)) {
            throw Error(`Cannot set an (entire) resource list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
          }
          val = value.map((v, i) => {
            if (v instanceof ResourceInstanceViewModel) {
              return v;
            }
            return ResourceInstanceViewModel.__create(tile, node, v, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
          });
          Promise.all(val.map(async (c) => {
            const v = await c;
            return v ? (await v).id : null;
          })).then((ids) => {
            tile.data.set(nodeid, ids.map((id) => {
              return {
                resourceId: id
              };
            }));
            return ids;
          });
          value = val;
        }
      } else {
        value = [];
      }
      if (!tile || !value) {
        return null;
      }
      const str = new ResourceInstanceListViewModel(...value);
      return str;
    }
    async __asTileData() {
      return this._value ? await this._value : null;
    }
  }
  _c = Symbol.toPrimitive;
  const _ResourceInstanceViewModel = class _ResourceInstanceViewModel {
    constructor(id, modelWrapper, instanceWrapperFactory, cacheEntry) {
      __publicField(this, "_");
      __publicField(this, "$");
      __publicField(this, "__");
      __publicField(this, "__parentPseudo");
      __publicField(this, "__cacheEntry", null);
      __publicField(this, "id");
      __publicField(this, "then");
      __publicField(this, _c);
      __publicField(this, "gm");
      this.id = id;
      this.$ = instanceWrapperFactory ? instanceWrapperFactory(this) : null;
      this.__ = modelWrapper;
      if (cacheEntry instanceof ResourceInstanceCacheEntry) {
        this.__cacheEntry = cacheEntry;
      }
      return new Proxy(this, {
        set: async (object, key, value) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            object[key] = value;
          } else if (k in object || k.startsWith("__")) {
            object[k] = value;
          } else {
            if (!object.$) {
              await this.retrieve();
              if (!object.$) {
                throw Error("Could not retrieve resource");
              }
            }
            object.$.setOrmAttribute(k, value);
          }
          return true;
        },
        get: (object, key) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            return object[key];
          } else if (k in object || k.startsWith("__")) {
            return object[k];
          }
          return new AttrPromise(async (resolve) => {
            if (!object.$) {
              await this.retrieve();
              if (!object.$) {
                throw Error("Could not retrieve resource");
              }
            }
            return object.$.getOrmAttribute(k).then((v) => {
              return resolve(v);
            });
          });
        }
      });
    }
    toString() {
      if (!this.__) {
        return `[Resource:${this.id}]`;
      }
      return `[${this.__.wkrm.modelClassName}:${this.id ?? "-"}]`;
    }
    async __has(key) {
      if (!this.$) {
        return void 0;
      }
      return (await this.$.getRootViewModel() || /* @__PURE__ */ new Map()).__has(key);
    }
    async __asTileData() {
      return {
        resourceId: this.id
      };
    }
    async __forJsonCache(getMeta) {
      let wrapper;
      if (!this.__) {
        if (this.__cacheEntry) {
          return this.__cacheEntry;
        } else {
          [, wrapper] = await this.retrieve();
        }
      } else {
        wrapper = this.__;
      }
      this.__cacheEntry = new ResourceInstanceCacheEntry({
        meta: getMeta ? await getMeta(this) : void 0,
        id: this.id,
        type: wrapper.wkrm.modelClassName,
        graphId: wrapper.wkrm.graphId,
        title: null
      });
      return this.__cacheEntry;
    }
    async forJson(cascade = false) {
      let jsonData;
      if (!cascade && this.__cacheEntry) {
        jsonData = {
          type: this.__cacheEntry.type,
          graphId: this.__cacheEntry.graphId,
          id: this.__cacheEntry.id,
          title: this.__cacheEntry.title || void 0,
          meta: this.__cacheEntry.meta || void 0,
          root: null
        };
      } else if (this.__) {
        jsonData = {
          type: this.__.wkrm.modelClassName,
          graphId: this.__.wkrm.graphId,
          id: this.id,
          title: void 0,
          meta: void 0,
          root: null
        };
      } else {
        jsonData = {
          type: "(unknown)",
          graphId: "",
          id: this.id,
          title: void 0,
          meta: void 0,
          root: null
        };
      }
      const basic = new StaticResourceReference(jsonData);
      if (cascade) {
        if (!this.$) {
          await this.retrieve();
          if (!this.$) {
            throw Error("Could not retrieve resource");
          }
        }
        const root = await this.$.getRootViewModel();
        basic.root = await root.forJson();
      }
      return basic;
    }
    async retrieve() {
      let iw;
      let mw;
      if (viewContext.graphManager) {
        const replacement = await viewContext.graphManager.getResource(this.id, true);
        iw = replacement.$;
        mw = replacement.__;
      } else {
        throw Error("Cannot traverse resource relationships without a GraphManager");
      }
      this.$ = iw;
      this.__ = mw;
      return [
        iw,
        mw
      ];
    }
    static async __create(tile, node, value, cacheEntry) {
      const nodeid = node.nodeid;
      let val = value;
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          if (!value && !(value instanceof StaticResource) && !(value instanceof StaticResourceReference)) {
            val = null;
          } else if (value instanceof Promise) {
            return value.then((value2) => {
              return _ResourceInstanceViewModel.__create(tile, node, value2, cacheEntry);
            });
          } else if (typeof value == "string") {
            if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(value)) {
              val = value;
            } else {
              throw Error(`Set resource instances using id, not strings in node ${node.alias}: ${value}`);
            }
          } else if (value instanceof Object && value.resourceId) {
            val = value.resourceId;
          } else if (value instanceof Array && value.length < 2) {
            if (value.length == 1) {
              return _ResourceInstanceViewModel.__create(tile, node, value[0], cacheEntry);
            }
          } else {
            throw Error("Could not set resource instance from this data");
          }
          tile.data.set(nodeid, val ? [
            {
              resourceId: val
            }
          ] : null);
        }
      }
      if (!tile || !val) {
        return null;
      }
      const str = new _ResourceInstanceViewModel(val, null, null, cacheEntry);
      return str;
    }
  };
  let ResourceInstanceViewModel = _ResourceInstanceViewModel;
  class FileListViewModel extends Array {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value", null);
    }
    async forJson() {
      const value = await this._value;
      return value ? value.map((v) => v ? v.forJson() : null) : null;
    }
    async __forJsonCache() {
      return null;
    }
    static async __create(tile, node, value) {
      const nodeid = node.nodeid;
      let val = [];
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(`Cannot set an (entire) file list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
        }
        val = value.map((c) => {
          return c;
        });
        Promise.all(val).then((vals) => {
          Promise.all(vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()).id : null;
          })).then((ids) => {
            tile.data.set(nodeid, ids);
          });
        });
        value = val;
      } else {
        value = [];
      }
      const str = new FileListViewModel(...value);
      return str;
    }
    async __asTileData() {
      return this._value ? await this._value : null;
    }
  }
  class ConceptListViewModel extends Array {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value", null);
    }
    async forJson() {
      const value = await this._value;
      return value ? value.map((v) => v ? v.forJson() : null) : null;
    }
    async __forJsonCache(getMeta) {
      return new ConceptListCacheEntry({
        meta: getMeta ? await getMeta(this) : getMeta,
        _: (await Promise.all([
          ...this.values()
        ].map(async (rivmPromise) => {
          const rivm = await rivmPromise;
          if (rivm) {
            return await rivm.__forJsonCache(getMeta);
          }
        }))).filter((val) => val !== void 0)
      });
    }
    static async __create(tile, node, value, cacheEntry = null) {
      const nodeid = node.nodeid;
      let val = [];
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(`Cannot set an (entire) concept list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
        }
        val = value.map((c, i) => {
          if (c instanceof ConceptValueViewModel) {
            return c;
          }
          return ConceptValueViewModel.__create(tile, node, c, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
        });
        Promise.all(val).then((vals) => {
          Promise.all(vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()).id : null;
          })).then((ids) => {
            tile.data.set(nodeid, ids);
          });
        });
        value = val;
      } else {
        value = [];
      }
      const str = new ConceptListViewModel(...value);
      return str;
    }
    async __asTileData() {
      return this._value ? await this._value : null;
    }
  }
  class DomainValueListViewModel extends Array {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value", null);
    }
    async forJson() {
      const value = await this._value;
      return value ? value.map((v) => v ? v.forJson() : null) : null;
    }
    __forJsonCache() {
      return null;
    }
    static async __create(tile, node, value) {
      const nodeid = node.nodeid;
      let val;
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          tile.data.set(nodeid, []);
          if (!Array.isArray(value)) {
            throw Error(`Cannot set an (entire) domain list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
          }
          val = value.map((c) => {
            if (c instanceof DomainValueViewModel) {
              return c;
            }
            return DomainValueViewModel.__create(tile, node, c);
          });
          Promise.all(val).then(async (vals) => {
            const ids = Promise.all(vals.map(async (val2) => val2 === null ? val2 : (await val2._value).id));
            ids.then((ids2) => {
              tile.data.set(nodeid, ids2);
            });
          });
        }
      } else {
        value = [];
      }
      const str = new DomainValueListViewModel(...value);
      return str;
    }
    async __asTileData() {
      const value = await this._value;
      return value ?? null;
    }
  }
  class DomainValueViewModel extends String {
    constructor(value) {
      super(value.toString());
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value");
      this._value = value;
    }
    async forJson() {
      return this._value;
    }
    __forJsonCache() {
      return null;
    }
    getValue() {
      return this._value;
    }
    async lang(lang) {
      return (await this._value).lang(lang);
    }
    static async __create(tile, node, value) {
      const nodeid = node.nodeid;
      let val = value;
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          if (!value && !(value instanceof StaticDomainValue)) {
            val = null;
          } else if (value instanceof Promise) {
            return value.then((value2) => {
              return DomainValueViewModel.__create(tile, node, value2);
            });
          } else if (typeof value == "string") {
            if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$/i.exec(value)) {
              const config = nodeConfigManager.retrieve(node);
              if (!config || !(config instanceof StaticNodeConfigDomain)) {
                throw Error(`Cannot form domain value for ${node.nodeid} without config`);
              }
              val = config.valueFromId(value) || null;
            } else {
              throw Error("Set domain values using values from domain lists, not strings");
            }
          } else {
            throw Error("Could not set domain value from this data");
          }
          if (!(val instanceof Promise)) {
            tile.data.set(nodeid, val ? val.id : null);
          }
        }
      }
      if (!tile || !val) {
        return null;
      }
      const str = new DomainValueViewModel(val);
      return str;
    }
    async __asTileData() {
      const value = await this._value;
      return value ? value.id : null;
    }
  }
  class ConceptValueViewModel extends String {
    constructor(value) {
      super(value.value);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value");
      this._value = value;
    }
    async forJson() {
      return this._value;
    }
    async __forJsonCache(getMeta) {
      const value = await this._value;
      return new ConceptValueCacheEntry({
        meta: getMeta ? await getMeta(this) : void 0,
        id: value.id,
        value: value.value,
        conceptId: value.__conceptId
      });
    }
    getValue() {
      return this._value;
    }
    static async __create(tile, node, value, cacheEntry) {
      var _a2;
      const nodeid = node.nodeid;
      const collectionId = (_a2 = node.config) == null ? void 0 : _a2.rdmCollection;
      if (!collectionId) {
        throw Error(`Node ${node.alias} (${node.nodeid}) missing rdmCollection in config`);
      }
      let val = value;
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          if (value instanceof StaticConcept) {
            if (value.getPrefLabel) {
              val = value.getPrefLabel();
            } else {
              throw Error("Recognizing value as StaticConcept, but no getPrefLabel member");
            }
          }
          if (!value) {
            val = null;
          } else if (value instanceof StaticValue) ;
          else if (value instanceof Promise) {
            return value.then((value2) => {
              return ConceptValueViewModel.__create(tile, node, value2, cacheEntry);
            });
          } else if (typeof value == "string") {
            if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(value)) {
              if (cacheEntry) {
                val = new StaticValue({
                  id: cacheEntry.id,
                  value: cacheEntry.value,
                  __concept: null,
                  __conceptId: cacheEntry.conceptId
                }, cacheEntry.conceptId);
                return new ConceptValueViewModel(val);
              } else {
                const collection = RDM.retrieveCollection(collectionId);
                return collection.then((collection2) => {
                  if (!collection2.getConceptValue) {
                    throw Error(`Collection ${collection2.id} must be a StaticCollection here, not a key/value object`);
                  }
                  const val2 = collection2.getConceptValue(value);
                  if (!val2) {
                    console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
                  }
                  tile.data.set(nodeid, val2 ? val2.id : null);
                  if (!tile || !val2) {
                    return null;
                  }
                  const str2 = new ConceptValueViewModel(val2);
                  return str2;
                });
              }
            } else {
              throw Error(`Set concepts using values from collections, not strings: ${value}`);
            }
          } else {
            throw Error("Could not set concept from this data");
          }
          if (!(val instanceof Promise)) {
            if (!val) {
              console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
            }
            tile.data.set(nodeid, val ? val.id : null);
          }
        }
      }
      if (!tile || !val) {
        return null;
      }
      const str = new ConceptValueViewModel(val);
      return str;
    }
    async __asTileData() {
      const value = await this._value;
      return value ? value.id : null;
    }
  }
  class DateViewModel extends Date {
    constructor(val) {
      super(val);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "__original");
      __publicField(this, "then");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      this.__original = val;
    }
    __forJsonCache() {
      return null;
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => DateViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          tile.data.set(nodeid, value);
        }
      }
      let val = tile.data.get(nodeid);
      if (typeof val == "object" && val["en"] !== void 0) {
        val = val.en;
      }
      if (!tile || val === null || val === void 0 || val === "") {
        return null;
      }
      if (typeof val != "string") {
        throw Error("Date should be a string");
      }
      const str = new DateViewModel(val);
      return str;
    }
    async forJson() {
      try {
        return this.toISOString();
      } catch (e) {
        console.warn(e);
        return this.__original;
      }
    }
    __asTileData() {
      return this.toISOString();
    }
  }
  _d = Symbol.toPrimitive;
  const _GeoJSONViewModel = class _GeoJSONViewModel {
    constructor(jsonData) {
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "then");
      __publicField(this, _d);
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value");
      this._value = jsonData;
      return new Proxy(this, {
        get: (object, key) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            return object[key];
          } else if (k in object) {
            return object[k];
          }
          return this._value[k];
        },
        set: (object, key, value) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            object[key] = value;
          } else if (k in object) {
            object[k] = value;
          } else {
            this._value[k] = value;
          }
          return true;
        }
      });
    }
    __forJsonCache() {
      return null;
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => _GeoJSONViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          tile.data.set(nodeid, value);
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      if (!(val instanceof Object)) {
        throw Error("GeoJSON should be a JSON object");
      }
      const str = new _GeoJSONViewModel(val);
      return str;
    }
    async forJson() {
      return await this._value;
    }
    __asTileData() {
      return this._value;
    }
  };
  let GeoJSONViewModel = _GeoJSONViewModel;
  class EDTFViewModel extends String {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    }
    __forJsonCache() {
      return null;
    }
    forJson() {
      return this.toString();
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => EDTFViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (value !== null) {
          tile.data.set(nodeid, value);
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      const string = new EDTFViewModel(val);
      return string;
    }
    __asTileData() {
      return `${this}`;
    }
  }
  class NonLocalizedStringViewModel extends String {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    }
    __forJsonCache() {
      return null;
    }
    forJson() {
      return this.toString();
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => NonLocalizedStringViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (value !== null) {
          tile.data.set(nodeid, value);
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      const string = new NonLocalizedStringViewModel(val);
      return string;
    }
    __asTileData() {
      return `${this}`;
    }
  }
  class NumberViewModel extends Number {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    }
    toString() {
      return `${this.valueOf()}`;
    }
    __forJsonCache() {
      return null;
    }
    forJson() {
      return this.valueOf();
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => NumberViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (value !== null) {
          tile.data.set(nodeid, value);
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      const num = new NumberViewModel(val);
      return num;
    }
    __asTileData() {
      return this.valueOf();
    }
  }
  class BooleanViewModel extends Boolean {
    constructor(value, config) {
      super(value);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "__config");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      this.__config = config;
    }
    toString(lang) {
      const labelLang = lang || DEFAULT_LANGUAGE;
      return this.valueOf() ? this.__config && this.__config.trueLabel ? this.__config.trueLabel[labelLang] || "true" : "true" : this.__config && this.__config.trueLabel ? this.__config.falseLabel[labelLang] || "false" : "false";
    }
    __forJsonCache() {
      return null;
    }
    forJson() {
      return this.valueOf();
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => BooleanViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (value !== null) {
          tile.data.set(nodeid, value);
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      const config = nodeConfigManager.retrieve(node);
      if (!config || !(config instanceof StaticNodeConfigBoolean)) {
        throw Error(`Cannot form boolean value for ${node.nodeid} without config`);
      }
      if (typeof val !== "boolean" && val !== 0 && val !== 1) {
        throw Error(`Refusing to use truthiness for value ${val} in boolean`);
      }
      const bool = new BooleanViewModel(val ? true : false, config);
      return bool;
    }
    __asTileData() {
      return this.valueOf();
    }
  }
  class Url {
    constructor(url, url_label) {
      __publicField(this, "url");
      __publicField(this, "url_label");
      this.url = url;
      this.url_label = url_label;
    }
  }
  class UrlViewModel extends String {
    constructor(value) {
      const displayValue = value.url_label || value.url;
      super(displayValue);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value");
      this._value = value;
    }
    __forJsonCache() {
      return null;
    }
    forJson() {
      return {
        url: this._value.url,
        url_label: this._value.url_label || ""
      };
    }
    label() {
      return this._value.url_label || this._value.url;
    }
    href() {
      return this._value.url;
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => UrlViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, {});
        }
        if (value !== null) {
          if (value instanceof UrlViewModel) {
            value = value._value;
          } else if (value instanceof Object) {
            if (!value.url) {
              throw Error(`A URL must be null or have a 'url' field: ${value}`);
            }
          }
          tile.data.set(nodeid, {
            url: value.url,
            url_label: value.url_label
          });
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      let url;
      if (typeof val !== "object") {
        url = new Url(`${val}`);
      } else if (val instanceof Map) {
        url = new Url(val.get("url"), val.get("url_label"));
      } else if ("url" in val && typeof val === "object" && typeof val.url === "string" && "url_label" in val && (val.url_label === void 0 || typeof val.url_label === "string")) {
        url = new Url(val.url, val.url_label);
      } else {
        throw Error(`Unrecognised URL type: ${val}`);
      }
      const str = new UrlViewModel(url);
      return str;
    }
    __asTileData() {
      return this.forJson();
    }
  }
  class StringViewModel extends String {
    constructor(value, language = null) {
      const lang = value.get(language || DEFAULT_LANGUAGE);
      let displayValue;
      if (lang) {
        if (typeof lang == "string") {
          displayValue = lang;
        } else {
          displayValue = lang.value;
        }
      } else {
        displayValue = "";
      }
      super(displayValue);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value");
      this._value = value;
    }
    __forJsonCache() {
      return null;
    }
    forJson() {
      return `${this}`;
    }
    lang(language) {
      const elt = this._value.get(language);
      if (elt) {
        if (elt instanceof Object) {
          return elt.value;
        }
        return elt;
      } else {
        return void 0;
      }
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => StringViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, {});
        }
        if (value !== null) {
          if (value instanceof Object) {
            const entries = value instanceof Map ? value.entries() : Object.entries(value);
            for (const [k, v] of [
              ...entries
            ]) {
              const val2 = tile.data.get(nodeid);
              if (val2 instanceof Map) {
                val2.set(k, v);
              } else if (val2 instanceof Object) {
                val2[k] = v;
              } else if (val2 !== null) {
                throw Error("Malformed string in tile data");
              }
            }
          } else {
            tile.data.set(nodeid, {
              [DEFAULT_LANGUAGE]: value
            });
          }
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      let mapVal;
      if (val instanceof Map) {
        mapVal = val;
      } else {
        mapVal = new Map(Object.entries(val));
      }
      const str = new StringViewModel(mapVal);
      return str;
    }
    __asTileData() {
      return this._value;
    }
  }
  _e = Symbol.toPrimitive;
  const _SemanticViewModel = class _SemanticViewModel {
    constructor(parentWkri, childNodes, tile, node) {
      __publicField(this, "_");
      __publicField(this, "then");
      __publicField(this, _e);
      __publicField(this, "__parentPseudo");
      __publicField(this, "__childValues");
      __publicField(this, "__parentWkri");
      __publicField(this, "__childNodes");
      __publicField(this, "__tile");
      __publicField(this, "__node");
      this.__childValues = /* @__PURE__ */ new Map();
      this.__parentWkri = parentWkri;
      this.__tile = tile;
      this.__node = node;
      this.__childNodes = childNodes;
      return new Proxy(this, {
        set: (object, key, value) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            object[key] = value;
          } else if (k.startsWith("__") || k in object) {
            object[k] = value;
          } else {
            object.__set(k, value);
          }
          return true;
        },
        get: (object, key) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key.toString() === "Symbol.toStringTag") {
            return () => this.constructor.name;
          }
          if (key in object) {
            return object[key];
          } else if (k.startsWith("__") || k in object) {
            return object[k];
          }
          if (k == "length") {
            return object.__childNodes.size;
          }
          return new AttrPromise((resolve) => {
            object.__get(k).then(resolve);
          });
        }
      });
    }
    __forJsonCache() {
      return null;
    }
    async toString() {
      const entries = [
        ...this.__childValues.entries()
      ].map(([k, v]) => `${k}: ${v}`);
      return `[[${entries.join(",")}]]`;
    }
    async toObject() {
      const entries = [
        ...(await this.__getChildValues()).entries()
      ];
      return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
        return [
          k,
          (await vl).getValue()
        ];
      })));
    }
    async forJson() {
      async function _forJson(v) {
        v = await v;
        if (!v) {
          return null;
        }
        return await v.forJson();
      }
      const entries = [
        ...(await this.__getChildValues()).entries()
      ];
      return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
        return [
          k,
          vl ? await _forJson(vl) : vl
        ];
      })));
    }
    async __update(map) {
      return Promise.all([
        ...map.entries()
      ].map(([k, v]) => {
        this.__set(k, v);
      }));
    }
    async __get(key) {
      const childValue = await this.__getChildValue(key);
      return childValue.getValue();
    }
    async __set(key, value) {
      if (!this.__childNodes.has(key)) {
        throw Error(`Semantic node does not have this key: ${key} (${[
          ...this.__childNodes.keys()
        ]})`);
      }
      throw Error(`Setting semantic keys (${key} = ${value}) is not implemented yet in Javascript`);
    }
    __has(key) {
      return this.__childNodes.has(key);
    }
    async __getChildTypes() {
      const promises = [
        ...this.__childNodes.keys()
      ].map(async (key) => [
        key,
        await this.__getChildValue(key)
      ]);
      const entries = await Promise.all(promises);
      return new Map([
        ...entries
      ]);
    }
    async __getChildren(direct = null) {
      const items = /* @__PURE__ */ new Map();
      for (const [key, value] of [
        ...(await this.__getChildValues()).entries()
      ]) {
        items.set(key, value);
      }
      const children = [
        ...items.entries()
      ].filter((entry) => {
        const child = this.__childNodes.get(entry[0]);
        if (!child) {
          throw Error("Child key is not in child nodes");
        }
        return (direct === null || direct === !child.is_collector) && entry[1] !== null;
      }).map((entry) => entry[1]);
      return children;
    }
    async __getChildValue(key, setDefault = false) {
      if (!this.__childNodes.has(key)) {
        throw Error(`Semantic node does not have this key: ${key} (${[
          ...this.__childNodes.keys()
        ]})`);
      }
      let child;
      if (!this.__childValues.has(key)) {
        const children = await this.__getChildValues();
        child = children.get(key) || null;
        let set = true;
        if (child === null) {
          child = this.__makePseudo(key);
          set = setDefault;
        }
        if (set) {
          this.__childValues.set(key, child);
        }
        child.parentNode = this.__parentPseudo || null;
      } else {
        child = this.__childValues.get(key);
      }
      return child;
    }
    __makePseudo(key) {
      const childNode = this.__childNodes.get(key);
      if (!childNode) {
        throw Error(`Child node key ${key} missing`);
      }
      if (!this.__parentWkri) {
        throw Error("This semantic node is currently parentless (no WKRI)");
      }
      if (!this.__parentWkri.$) {
        throw Error("This semantic node is currently on an unloaded WKRI");
      }
      const child = this.__parentWkri.$.addPseudo(childNode, this.__tile, this.__node);
      child.parentNode = this.__parentPseudo || null;
      return child;
    }
    static async __create(tile, node, value, parent, childNodes) {
      const svm = new _SemanticViewModel(parent, childNodes, tile, node);
      if (value) {
        try {
          await svm.__update(value);
        } catch (e) {
          tileLoadingError(`
          Suppressed a tile loading error: ${e}: ${typeof e} (tile: ${tile}; node: ${node}) - ${value}
        `, e);
        }
      }
      return svm;
    }
    async __asTileData() {
      const relationships = [];
      for (const value of [
        ...await this.__getChildren(true)
      ]) {
        const [, subrelationships] = await value.getTile();
        relationships.push(...subrelationships);
      }
      return [
        null,
        relationships
      ];
    }
    async __getChildValues() {
      const parent = this.__parentWkri;
      const childNodes = this.__childNodes;
      const tile = this.__tile;
      const node = this.__node;
      if (!parent || !parent.$) {
        return /* @__PURE__ */ new Map();
      }
      await parent.$.loadNodes([
        ...childNodes.keys()
      ]);
      const children = /* @__PURE__ */ new Map();
      for (const entry of [
        ...parent.$.allEntries()
      ]) {
        const key = entry[0];
        let values = entry[1];
        if (values instanceof Promise) {
          values = await values;
        }
        if (values === false || values === null || values === void 0) {
          continue;
        }
        const childNode = childNodes.get(key);
        if (childNode) {
          for (let value of values) {
            if (value !== null && value.node && (!value.parentNode || value.parentNode === this.__parentPseudo)) {
              value = await value;
              if (!value.node) {
                throw Error(`Node ${childNode.alias} (${childNode.nodeid}) is unavailable`);
              }
              if (value.node.nodegroup_id != node.nodegroup_id && tile && value.tile && (!value.tile.parenttile_id || value.tile.parenttile_id == tile.tileid) || value.node.nodegroup_id == node.nodegroup_id && tile && value.tile == tile && !childNode.is_collector) {
                children.set(key, value);
              } else if (node.nodegroup_id != value.node.nodegroup_id && childNode.is_collector) {
                const childValue = value instanceof PseudoList ? value : value.isIterable() ? await value.getValue() : null;
                let listValue;
                if (childValue && Array.isArray(childValue)) {
                  listValue = childValue;
                } else {
                  listValue = null;
                }
                if (listValue !== null) {
                  if (children.has(key)) {
                    children.get(key).push(...listValue);
                  } else {
                    children.set(key, listValue);
                  }
                } else {
                  children.set(key, value);
                }
              }
            }
          }
        }
      }
      for (const [key, value] of [
        ...children.entries()
      ]) {
        value.parentNode = this.__parentPseudo;
        this.__childValues.set(key, value);
      }
      return children;
    }
  };
  let SemanticViewModel = _SemanticViewModel;
  const CUSTOM_DATATYPES = /* @__PURE__ */ new Map();
  async function getViewModel(parentPseudo, tile, node, data, parent, childNodes, isInner = false) {
    let vm;
    const cacheEntries = parentPseudo.parent && parentPseudo.parent.$ ? await parentPseudo.parent.$.getValueCache(false, void 0) : void 0;
    let cacheEntry = null;
    if (cacheEntries) {
      cacheEntry = (tile.tileid ? cacheEntries[tile.tileid] ?? {} : {})[node.nodeid];
    }
    const datatype = isInner ? "semantic" : CUSTOM_DATATYPES.get(node.datatype) ?? node.datatype;
    let conceptCacheEntry;
    let conceptValueCacheEntry;
    let resourceInstanceCacheEntry;
    let resourceInstanceListCacheEntry;
    if (!(typeof datatype == "string")) {
      vm = await datatype.__create(tile, node, data, cacheEntry);
    } else {
      switch (datatype) {
        case "semantic":
          vm = await SemanticViewModel.__create(tile, node, data, parent, childNodes);
          break;
        case "domain-value":
          vm = await DomainValueViewModel.__create(tile, node, data);
          break;
        case "domain-value-list":
          vm = await DomainValueListViewModel.__create(tile, node, data);
          break;
        case "concept":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptValueCacheEntry)) {
            conceptValueCacheEntry = new ConceptValueCacheEntry(cacheEntry);
          } else {
            conceptValueCacheEntry = cacheEntry;
          }
          vm = await ConceptValueViewModel.__create(tile, node, data, conceptValueCacheEntry);
          break;
        case "resource-instance":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceCacheEntry)) {
            resourceInstanceCacheEntry = new ResourceInstanceCacheEntry(cacheEntry);
          } else {
            resourceInstanceCacheEntry = cacheEntry;
          }
          vm = await ResourceInstanceViewModel.__create(tile, node, data, resourceInstanceCacheEntry);
          break;
        case "resource-instance-list":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceListCacheEntry)) {
            resourceInstanceListCacheEntry = new ResourceInstanceListCacheEntry(cacheEntry);
          } else {
            resourceInstanceListCacheEntry = cacheEntry;
          }
          vm = await ResourceInstanceListViewModel.__create(tile, node, data, resourceInstanceListCacheEntry);
          break;
        case "concept-list":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptListCacheEntry)) {
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
          break;
        case "number":
          vm = await NumberViewModel.__create(tile, node, data);
          break;
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
          vme.then((vmep) => {
            if (vmep !== null) vmep.__parentPseudo = parentPseudo;
          });
        } else {
          vme.__parentPseudo = parentPseudo;
        }
      }
    }
    return vm;
  }
  viewModels = Object.freeze(Object.defineProperty({
    __proto__: null,
    BooleanViewModel,
    CUSTOM_DATATYPES,
    ConceptValueViewModel,
    DEFAULT_LANGUAGE,
    DateViewModel,
    DomainValueViewModel,
    GeoJSONViewModel,
    NonLocalizedStringViewModel,
    NumberViewModel,
    ResourceInstanceCacheEntry,
    ResourceInstanceViewModel,
    SemanticViewModel,
    StringViewModel,
    UrlViewModel,
    ValueList,
    getViewModel,
    viewContext
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  class PseudoUnavailable {
    constructor(node) {
      __publicField(this, "parentNode", null);
      __publicField(this, "tile", null);
      __publicField(this, "node");
      __publicField(this, "isOuter", false);
      this.node = node;
    }
    async forJson() {
      return null;
    }
    describeField() {
      return "Unavailable field";
    }
    describeFieldGroup() {
      return "Unavailable field";
    }
    getValue() {
      return new AttrPromise((resolve) => resolve(null));
    }
    getLength() {
      return 0;
    }
    getChildren(_ = false) {
      return [];
    }
    isIterable() {
      return false;
    }
  }
  const ITERABLE_DATATYPES = [
    "concept-list",
    "resource-instance-list",
    "domain-value-list"
  ];
  class PseudoValue {
    constructor(node, tile, value, parent, childNodes, inner) {
      __publicField(this, "node");
      __publicField(this, "tile");
      __publicField(this, "value");
      __publicField(this, "parent");
      __publicField(this, "parentNode");
      __publicField(this, "valueLoaded", false);
      __publicField(this, "datatype", null);
      __publicField(this, "originalTile");
      __publicField(this, "accessed");
      __publicField(this, "childNodes");
      __publicField(this, "isOuter", false);
      __publicField(this, "isInner", false);
      __publicField(this, "inner", null);
      __publicField(this, "independent");
      this.node = node;
      this.tile = tile;
      this.independent = tile === null;
      if (!parent) {
        throw Error("Must have a parent or parent class for a pseudo-node");
      }
      this.parent = parent;
      this.parentNode = null;
      this.childNodes = childNodes;
      this.value = value;
      this.accessed = false;
      this.originalTile = tile;
      this.datatype = node.datatype;
      if (inner instanceof PseudoValue) {
        this.isOuter = true;
        this.inner = inner;
      }
      if (inner === true) {
        this.isInner = true;
        this.datatype = "semantic";
      }
    }
    isIterable() {
      return this.datatype !== null && ITERABLE_DATATYPES.includes(this.datatype);
    }
    describeField() {
      let fieldName = this.node.name;
      if (this.parent && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
      }
      return fieldName;
    }
    describeFieldGroup() {
      let fieldName = this.node.name;
      if (this.parent && this.node.nodegroup_id && this.parent.$) {
        const nodegroup = this.parent.$.model.getNodeObjects().get(this.node.nodegroup_id);
        if (nodegroup && this.parent.__) {
          fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
        }
      }
      return fieldName;
    }
    getParentTileId() {
      return this.tile ? this.tile.parenttile_id : null;
    }
    async getTile() {
      await this.updateValue();
      let relationships = [];
      if (this.inner) {
        [this.tile, relationships] = await this.inner.getTile();
      }
      let tileValue;
      if (this.value !== null) {
        const [newTileValue, ownRelationships] = await (await this.value).__asTileData();
        tileValue = newTileValue;
        relationships = [
          ...relationships,
          ...ownRelationships
        ];
      } else {
        tileValue = null;
      }
      if (!this.tile) {
        throw Error();
      }
      this.tile.data = this.tile.data || {};
      if (tileValue === null) {
        if (this.tile.data.get(this.node.nodeid)) {
          this.tile.data.delete(this.node.nodeid);
        }
      } else {
        this.tile.data.set(this.node.nodeid, tileValue);
      }
      const tile = this.independent ? this.tile : null;
      return [
        tile,
        relationships
      ];
    }
    clear() {
      this.value = null;
      if (this.tile && this.tile.data && this.tile.data.has(this.node.nodeid)) {
        this.tile.data.delete(this.node.nodeid);
      }
    }
    updateValue(tile) {
      if (tile) {
        this.tile = tile;
      }
      this.accessed = true;
      if (this.inner) {
        this.inner.accessed = true;
      }
      if (!this.tile) {
        if (!this.node) {
          throw Error("Empty tile");
        }
        if (this.inner) {
          return new AttrPromise(async (resolve) => {
            var _a2;
            const tile2 = await ((_a2 = this.inner) == null ? void 0 : _a2.getTile());
            resolve(this.updateValue(tile2 ? tile2[0] : void 0));
          });
        }
        if (!this.tile) {
          this.tile = new StaticTile({
            nodegroup_id: this.node.nodegroup_id || "",
            tileid: null,
            data: /* @__PURE__ */ new Map(),
            sortorder: this.node.sortorder,
            resourceinstance_id: "",
            parenttile_id: null,
            provisionaledits: null,
            ensureId: () => ""
          });
        }
      }
      if (this.valueLoaded === false) {
        this.valueLoaded = void 0;
        let data;
        if (this.value === null && this.tile.data !== null && this.tile.data.has(this.node.nodeid) && this.datatype !== "semantic") {
          data = this.tile.data.get(this.node.nodeid);
        } else {
          data = this.value;
        }
        if (this.isOuter && typeof data === "object" && this.inner && data) {
          let outerData = void 0;
          if ("_" in data && !data.constructor) {
            outerData = data["_"];
            delete data["_"];
            this.inner.getValue().then((v) => v && v.update(data));
            data = outerData;
          } else if (data instanceof Map && data.has("_")) {
            outerData = data.get("_");
            data.delete("_");
            this.inner.getValue().then((v) => v && v.update(data));
            data = outerData;
          }
        }
        const vm = getViewModel(this, this.tile, this.node, data, this.parent, this.childNodes, this.isInner);
        const resolveAttr = (vm2) => {
          if (vm2 !== null && vm2 instanceof Object) {
            vm2.__parentPseudo = this;
            if (this.isOuter && this.inner) {
              vm2._ = this.inner.getValue();
            }
            this.valueLoaded = true;
          }
          return vm2;
        };
        this.value = new AttrPromise((resolve) => {
          vm.then((vm2) => resolve(vm2 ? resolveAttr(vm2) : vm2));
        });
      }
      return this.value;
    }
    getValue() {
      return this.updateValue();
    }
    getLength() {
      return this.getChildren().length;
    }
    async getChildTypes() {
      await this.updateValue();
      let childTypes = {};
      if (this.value && this.value instanceof Object && "getChildTypes" in this.value && typeof this.value.getChildTypes === "function") {
        childTypes = this.value.getChildTypes();
      }
      if (this.inner) {
        Object.assign(childTypes, this.inner.getChildTypes());
      }
      return childTypes;
    }
    getChildren(direct = null) {
      let children = [];
      if (this.value && this.value instanceof Object && "getChildren" in this.value && typeof this.value.getChildren === "function") {
        children = this.value.getChildren(direct);
      }
      if (this.inner) {
        children = [
          ...children,
          ...this.inner.getChildren(direct)
        ];
      }
      return children;
    }
    async forJson() {
      const value = await this.getValue();
      return value instanceof Object ? value.forJson() : value;
    }
  }
  class PseudoList extends Array {
    constructor() {
      super(...arguments);
      __publicField(this, "node");
      __publicField(this, "parent");
      __publicField(this, "parentNode", null);
      __publicField(this, "tile");
      __publicField(this, "parenttileId");
      __publicField(this, "ghostChildren", null);
      __publicField(this, "isOuter", false);
    }
    isIterable() {
      return true;
    }
    async sorted() {
      const resolved = await Promise.all(this.map(async (pn) => await pn));
      const sorted = resolved.sort((a, b) => {
        const vals = [
          a,
          b
        ].map((val) => {
          if (val && a.__parentPseudo && a.__parentPseudo.tile) {
            if (val.__parentPseudo.tile.sortorder > 0) ;
            return val.__parentPseudo.tile.sortorder;
          } else {
            return 0;
          }
        });
        return vals[0] - vals[1];
      });
      return sorted;
    }
    describeField() {
      if (!this.node) {
        return "[(uninitialized node)]";
      }
      let fieldName = this.node.name;
      if (this.parent && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
      }
      return `[${fieldName}]`;
    }
    describeFieldGroup() {
      if (!this.node) {
        return "[(uninitialized node)]";
      }
      let fieldName = this.node.name;
      if (this.parent && this.node.nodegroup_id && this.parent.$) {
        const nodegroup = this.parent.$.model.getNodeObjects().get(this.node.nodegroup_id);
        if (nodegroup && this.parent.__) {
          fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
        }
      }
      return `[${fieldName}]`;
    }
    initialize(node, parent) {
      this.node = node;
      if (Array.isArray(this.node)) {
        throw Error("Cannot make a list of lists");
      }
      if (!parent) {
        throw Error("Must have a parent or parent class for a pseudo-node");
      }
      this.parent = parent;
      this.tile = void 0;
      this.parenttileId = void 0;
      this.ghostChildren = /* @__PURE__ */ new Set();
    }
    async forJson() {
      const array = Array.from((await this.sorted()).map(async (entry) => {
        const value = await entry;
        return value && value instanceof Object && value.forJson ? value.forJson() : value;
      }));
      return Promise.all(array);
    }
    getValue() {
      return new AttrPromise((resolve) => resolve(this));
    }
    toString() {
      return `<PL: ${this.length}>`;
    }
  }
  function makePseudoCls(model, key, single, tile = null, wkri = null) {
    const nodeObjs = model.getNodeObjectsByAlias();
    const nodeObj = nodeObjs.get(key);
    if (!nodeObj) {
      throw Error("Could not find node by alias");
    }
    const nodegroups = model.getNodegroupObjects();
    const nodegroup = nodegroups.get(nodeObj.nodegroup_id || "");
    let value = null;
    if (nodeObj.nodegroup_id && nodeObj.is_collector && nodegroup && nodegroup.cardinality == "n" && true) {
      value = new PseudoList();
      value.initialize(nodeObj, wkri);
    }
    if (value === null || tile) {
      let nodeValue;
      const isPermitted = model.isNodegroupPermitted(nodeObj.nodegroup_id || "", tile, nodeObjs);
      if (isPermitted) {
        const childNodes = model.getChildNodes(nodeObj.nodeid);
        let inner = false;
        if (childNodes && childNodes.size && nodeObj.datatype !== "semantic") {
          inner = new PseudoValue(nodeObj, tile, null, wkri, childNodes, true);
        }
        nodeValue = new PseudoValue(nodeObj, tile, null, wkri, inner !== false ? /* @__PURE__ */ new Map() : childNodes, inner);
      } else {
        nodeValue = new PseudoUnavailable(nodeObj);
      }
      if (value) {
        value.push(nodeValue.getValue());
      } else {
        value = nodeValue;
      }
    }
    return value;
  }
  const MAX_GRAPH_DEPTH = 100;
  const DESCRIPTOR_FUNCTION_ID = "60000000-0000-0000-0000-000000000001";
  WKRM = class {
    constructor(meta) {
      __publicField(this, "modelName");
      __publicField(this, "modelClassName");
      __publicField(this, "graphId");
      __publicField(this, "meta");
      const name = meta.name ? meta.name.toString() : void 0;
      this.modelName = name || "Unnamed";
      this.graphId = meta.graphid;
      this.modelClassName = (meta.slug || this.modelName).replace(/[_-]/g, " ").replace(/\s(.)/g, (c) => c.toUpperCase()).replace(/\s/g, "");
      this.modelClassName = this.modelClassName[0].toUpperCase() + this.modelClassName.slice(1);
      this.meta = meta;
    }
  };
  class ConfigurationOptions {
    constructor() {
      __publicField(this, "graphs");
      __publicField(this, "eagerLoadGraphs", false);
      this.graphs = null;
    }
  }
  class ResourceInstanceWrapper {
    constructor(wkri, model, resource, pruneTiles = true) {
      __publicField(this, "wkri");
      __publicField(this, "model");
      __publicField(this, "resource");
      __publicField(this, "valueList");
      __publicField(this, "cache");
      __publicField(this, "scopes");
      __publicField(this, "metadata");
      this.wkri = wkri;
      this.model = model;
      if (resource) {
        this.model.stripTiles(resource);
      }
      this.resource = resource;
      this.valueList = new ValueList(/* @__PURE__ */ new Map(), /* @__PURE__ */ new Map(), this, []);
      this.cache = resource ? resource.__cache : void 0;
      this.scopes = resource ? resource.__scopes : void 0;
      this.metadata = resource ? resource.metadata : void 0;
      if (pruneTiles && this.resource) {
        this.pruneResourceTiles();
      }
    }
    pruneResourceTiles() {
      if (!this.resource) {
        console.warn("Trying to prune tiles for an empty resource", this.wkri.modelClassName);
        return;
      }
      this.resource.tiles = (this.resource.tiles || []).filter((tile) => {
        return this.model.isNodegroupPermitted(tile.nodegroup_id || "", tile);
      });
    }
    async loadNodes(aliases) {
      for (const key of aliases) {
        await this.valueList.retrieve(key);
      }
    }
    async getName(update = false) {
      let resourceName = this.resource && this.resource.resourceinstance.name;
      if (update || !resourceName) {
        const descriptors = await this.getDescriptors(update);
        resourceName = descriptors && descriptors.name || resourceName || "<Unnamed>";
        if (this.resource && this.resource.resourceinstance) {
          this.resource.resourceinstance.name = resourceName;
        }
      }
      return resourceName;
    }
    async getDescriptors(update = false) {
      let descriptors = this.resource && this.resource.resourceinstance.descriptors;
      if (update || !descriptors || descriptors.isEmpty()) {
        descriptors = new StaticResourceDescriptors();
        let descriptorConfig = void 0;
        if (this.model.graph.functions_x_graphs) {
          const descriptorNode = this.model.graph.functions_x_graphs.find((node) => node.function_id === DESCRIPTOR_FUNCTION_ID);
          if (descriptorNode) {
            descriptorConfig = descriptorNode.config;
          }
        }
        const nodes = this.model.getNodeObjects();
        if (descriptorConfig) {
          for (const [descriptor, config] of Object.entries(descriptorConfig.descriptor_types)) {
            const semanticNode = nodes.get(config.nodegroup_id);
            let description = config.string_template;
            if (!description) {
              continue;
            }
            let requestedNodes = description.match(/<[A-Za-z _-]*>/g) || [];
            const relevantNodes = [
              ...nodes.values()
            ].filter((node) => node.nodegroup_id === config.nodegroup_id && [
              ...requestedNodes
            ].includes(`<${node.name}>`)).map((node) => [
              node.name,
              node.alias || ""
            ]);
            let relevantValues = [];
            if (semanticNode) {
              let semanticValue = await (await this.valueList.retrieve(semanticNode.alias || ""))[0];
              if (semanticValue instanceof PseudoList) {
                semanticValue = await semanticValue[0];
              } else if (semanticValue.inner) {
                relevantValues.push([
                  semanticNode.name || "",
                  await semanticValue.getValue()
                ]);
                semanticValue = await semanticValue.inner.getValue();
              } else {
                semanticValue = await semanticValue.getValue();
              }
              if (semanticValue) {
                relevantValues = [
                  ...relevantValues,
                  ...await Promise.all(relevantNodes.filter(([_, alias]) => semanticValue.__has(alias)).map(([name, alias]) => semanticValue[alias].then((value) => [
                    name,
                    value
                  ])))
                ];
              }
            }
            if (relevantValues) {
              description = relevantValues.reduce((desc, [name, value]) => value ? desc.replace(`<${name}>`, value) : desc, description);
            }
            requestedNodes = description.match(/<[A-Za-z _-]*>/g) || [];
            if (requestedNodes.length) {
              relevantValues = await Promise.all(relevantNodes.map(([name, alias]) => this.valueList.retrieve(alias).then((values) => [
                name,
                values ? values[0] : void 0
              ])));
              if (relevantValues) {
                description = relevantValues.reduce((desc, [name, value]) => value ? desc.replace(`<${name}>`, value) : desc, description);
              }
            }
            descriptors[descriptor] = description;
          }
        }
      }
      if (this.resource && this.resource.resourceinstance) {
        this.resource.resourceinstance.descriptors = descriptors;
        if (descriptors.name) {
          this.resource.resourceinstance.descriptors.name = descriptors.name;
        }
      }
      return descriptors;
    }
    addPseudo(childNode, tile, node) {
      const key = childNode.alias;
      if (!key) {
        throw Error(`Cannot add a pseudo node with no alias ${childNode.nodeid}`);
      }
      const child = makePseudoCls(this.model, key, false, !childNode.is_collector && childNode.nodegroup_id === node.nodegroup_id ? tile : null, this.wkri);
      const valueList = this.valueList;
      valueList.setDefault(key, []).then((val) => val.push(child));
      return child;
    }
    allEntries() {
      return this.valueList.values.entries();
    }
    async keys() {
      return (await this.getRootViewModel()).keys();
    }
    async values() {
      return (await this.getRootViewModel()).values();
    }
    async entries() {
      return (await this.getRootViewModel()).entries();
    }
    async getRootViewModel() {
      const root = await this.getRoot();
      let value = null;
      if (root) {
        const rootValue = await root.getValue();
        if (!Array.isArray(rootValue)) {
          value = rootValue;
        }
      }
      if (!value || !(value instanceof SemanticViewModel)) {
        throw Error(`Tried to get root on ${this.model.wkrm.modelClassName}, which has no root`);
      }
      return value;
    }
    getOrmAttribute(key) {
      let promise;
      if (this.resource === null) {
        promise = this.model.findStatic(this.wkri.id).then((resource) => {
          this.resource = resource;
        }).then(() => this.populate(true));
      } else {
        promise = new Promise((resolve) => {
          resolve();
        });
      }
      return new AttrPromise((resolve) => {
        promise.then(() => this.getRootViewModel()).then((root) => resolve(root[key]));
      });
    }
    async getRoot() {
      const values = this.valueList;
      const node = this.model.getRootNode();
      if (node) {
        let value;
        const alias = node.alias;
        if (!(typeof alias == "string")) {
          throw Error(`Alias missing for node ${node.nodeid}`);
        }
        await values.setDefault(alias, []);
        const nodeValues = await values.get(alias);
        if (nodeValues.length > 1) {
          throw Error("Cannot have multiple root tiles");
        } else if (nodeValues.length == 1) {
          value = nodeValues[0];
        } else {
          value = makePseudoCls(this.model, alias, false, null, this.wkri);
          values.set(alias, [
            value
          ]);
        }
        return value;
      }
    }
    setOrmAttribute(key, value) {
      return this.getRootViewModel().then((root) => {
        if (root) {
          root[key] = value;
        } else {
          throw Error(`Tried to set ${key} on ${self}, which has no root`);
        }
      });
    }
    async ensureNodegroup(allValues, allNodegroups, nodegroupId, nodeObjs, nodegroupObjs, edges, addIfMissing, tiles, doImpliedNodegroups = true) {
      const impliedNodegroups = /* @__PURE__ */ new Set();
      const sentinel = allNodegroups.get(nodegroupId);
      let newValues = /* @__PURE__ */ new Map();
      if (sentinel === false || addIfMissing && sentinel === void 0) {
        [
          ...nodeObjs.values()
        ].filter((node) => {
          return node.nodegroup_id === nodegroupId;
        }).forEach((node) => allValues.delete(node.alias || ""));
        let nodegroupTiles;
        if (tiles === null) {
          nodegroupTiles = [];
          console.error("Tiles must be provided and cannot be lazy-loaded yet");
        } else {
          nodegroupTiles = tiles.filter((tile) => tile.nodegroup_id == nodegroupId && this.model.isNodegroupPermitted(nodegroupId, tile));
          if (nodegroupTiles.length == 0 && addIfMissing) {
            nodegroupTiles = [
              null
            ];
          }
          const rgValues = await this.valuesFromResourceNodegroup(allValues, nodegroupTiles, nodegroupId, nodeObjs, edges);
          newValues = rgValues[0];
          const newImpliedNodegroups = rgValues[1];
          [
            ...newValues.entries()
          ].forEach((entry) => {
            if (entry[1] !== void 0) {
              allValues.set(entry[0], entry[1]);
            }
          });
          [
            ...newImpliedNodegroups
          ].forEach((v) => {
            impliedNodegroups.add(v);
          });
          allNodegroups.set(nodegroupId, true);
        }
      }
      if (doImpliedNodegroups) {
        for (const nodegroupId2 of [
          ...impliedNodegroups
        ]) {
          const [impliedValues] = await this.ensureNodegroup(allValues, allNodegroups, nodegroupId2, nodeObjs, nodegroupObjs, edges, true, tiles, true);
          for (const [key, value] of impliedValues) {
            newValues.set(key, value);
          }
        }
        impliedNodegroups.clear();
      }
      return [
        newValues,
        impliedNodegroups
      ];
    }
    async populate(lazy) {
      const nodeObjs = this.model.getNodeObjects();
      const nodegroupObjs = this.model.getNodegroupObjects();
      const edges = this.model.getEdges();
      const allValues = /* @__PURE__ */ new Map();
      const allNodegroups = new Map([
        ...nodegroupObjs.keys()
      ].map((id) => {
        return [
          id || "",
          false
        ];
      }));
      const rootNode = this.model.getRootNode();
      if (rootNode.alias === null) {
        throw Error("Cannot populate a model with no proper root node");
      }
      allValues.set(rootNode.alias, false);
      let tiles = null;
      if (!lazy && this.resource) {
        tiles = this.resource.tiles;
        let impliedNodegroups = /* @__PURE__ */ new Set();
        for (const [ng] of nodegroupObjs) {
          const [_, newImpliedNodegroups] = await this.ensureNodegroup(allValues, allNodegroups, ng, nodeObjs, nodegroupObjs, edges, true, tiles, false);
          for (const impliedNodegroup of [
            ...newImpliedNodegroups
          ]) {
            impliedNodegroups.add(impliedNodegroup);
          }
          impliedNodegroups.delete(ng);
        }
        while (impliedNodegroups.size) {
          const newImpliedNodegroups = /* @__PURE__ */ new Set();
          for (const nodegroupId of [
            ...impliedNodegroups
          ]) {
            const currentValue = allNodegroups.get(nodegroupId);
            if (currentValue === false || currentValue === void 0) {
              const [_, newImpliedNodegroups2] = await this.ensureNodegroup(allValues, allNodegroups, nodegroupId, nodeObjs, nodegroupObjs, edges, true, tiles, true);
              for (const impliedNodegroup of [
                ...newImpliedNodegroups2
              ]) {
                newImpliedNodegroups2.add(impliedNodegroup);
              }
            }
          }
          impliedNodegroups = newImpliedNodegroups;
        }
      } else if (this.resource) {
        this.model.stripTiles(this.resource);
      }
      this.valueList = new ValueList(allValues, allNodegroups, this, this.resource ? this.resource.tiles : null);
    }
    async getValueCache(build = true, getMeta = void 0) {
      if (build) {
        this.cache = await this.buildValueCache(getMeta);
      }
      return this.cache;
    }
    async buildValueCache(getMeta) {
      const cacheByTile = {};
      for (let pseudos of this.valueList.values.values()) {
        pseudos = await pseudos;
        if (pseudos) {
          await Promise.all(pseudos.map(async (pseudo) => {
            const value = await pseudo.getValue();
            if (pseudo.tile && value && !Array.isArray(pseudo)) {
              const cacheJson = await value.__forJsonCache(getMeta);
              if (cacheJson) {
                const tileId = pseudo.tile.ensureId();
                const nodeId = pseudo.node.nodeid;
                if (!(tileId in cacheByTile)) {
                  cacheByTile[tileId] = {};
                }
                if (!(nodeId in cacheByTile[tileId])) {
                  cacheByTile[tileId][nodeId] = {};
                }
                cacheByTile[tileId][nodeId] = cacheJson;
              }
            }
          }));
        }
      }
      return cacheByTile;
    }
    async valuesFromResourceNodegroup(existingValues, nodegroupTiles, nodegroupId, nodeObjs, edges) {
      const allValues = /* @__PURE__ */ new Map();
      const impliedNodegroups = /* @__PURE__ */ new Set();
      const impliedNodes = /* @__PURE__ */ new Map();
      const nodesUnseen = new Set([
        ...nodeObjs.values()
      ].filter((node) => node.nodegroup_id == nodegroupId).map((node) => node.alias));
      const tileNodesSeen = /* @__PURE__ */ new Set();
      const _addPseudo = async (node, tile) => {
        const key = node.alias || "";
        nodesUnseen.delete(node.alias);
        const tileid = tile && tile.tileid;
        if (tileid) {
          tileNodesSeen.add([
            node.nodeid,
            tileid
          ]);
        }
        let existing = existingValues.get(key);
        if (existing instanceof Promise) {
          existing = await existing;
        }
        if (existing !== false && existing !== void 0) {
          allValues.set(key, existing);
        }
        if (!allValues.has(key)) {
          allValues.set(key, []);
        }
        const pseudoNode = makePseudoCls(this.model, key, false, tile, this.wkri);
        for (const [domain, ranges] of edges) {
          if (ranges.includes(node.nodeid)) {
            const domainNode = nodeObjs.get(domain);
            if (!domainNode) {
              throw Error("Edge error in graph");
            }
            const toAdd = domainNode.nodegroup_id ? domainNode.nodegroup_id : "";
            if (toAdd && toAdd !== nodegroupId) {
              impliedNodegroups.add(toAdd);
            }
            if (domainNode.nodegroup_id && tile && domainNode.nodegroup_id === tile.nodegroup_id && domainNode.nodegroup_id !== domainNode.nodeid && tileid && !impliedNodes.has(domainNode.nodeid + tileid)) {
              impliedNodes.set(domainNode.nodeid + tileid, [
                domainNode,
                tile
              ]);
            }
            break;
          }
        }
        if (Array.isArray(pseudoNode)) {
          const value = allValues.get(key);
          if (value !== void 0 && value !== false) {
            for (const pseudoList of allValues.get(key)) {
              if (!(pseudoList instanceof PseudoList) || !(pseudoNode instanceof PseudoList)) {
                throw Error(`Should be all lists not ${typeof pseudoList} and ${typeof pseudoNode}`);
              }
              if (pseudoList.parentNode == pseudoNode.parentNode) {
                for (const ps of pseudoNode) {
                  pseudoList.push(ps);
                }
                return;
              }
            }
          }
        }
        allValues.get(key).push(pseudoNode);
      };
      for (const tile of nodegroupTiles) {
        const parentNode = nodeObjs.get(nodegroupId);
        if (parentNode === void 0) {
          continue;
        }
        if (!parentNode.nodegroup_id || parentNode.nodegroup_id == nodegroupId) {
          await _addPseudo(parentNode, tile);
        }
        if (tile) {
          const tileNodes = /* @__PURE__ */ new Map();
          for (const [key, value] of [
            ...tile.data.entries()
          ]) {
            tileNodes.set(key, value);
          }
          [
            ...nodeObjs.values()
          ].filter((node) => {
            return node.nodegroup_id === nodegroupId && !tileNodes.get(node.nodeid) && node.datatype === "semantic";
          }).forEach((node) => tileNodes.set(node.nodeid, {}));
          if (!tileNodes.has(tile.nodegroup_id)) {
            tileNodes.set(tile.nodegroup_id, {});
          }
          for (const [nodeid, nodeValue] of [
            ...tileNodes.entries()
          ]) {
            if (nodeid == nodegroupId) {
              continue;
            }
            const node = nodeObjs.get(nodeid);
            if (!node) {
              throw Error(`Unknown node in nodegroup: ${nodeid} in ${nodegroupId}`);
            }
            if (nodeValue !== null) {
              await _addPseudo(node, tile);
            }
          }
        }
      }
      while (impliedNodes.size > 0) {
        const value = impliedNodes.entries().next().value;
        if (value) {
          const [node, tile] = value[1];
          if (tile.tileid && !tileNodesSeen.has([
            node.nodeid,
            tile.tileid
          ])) {
            await _addPseudo(node, tile);
          }
          impliedNodes.delete(value[0]);
        }
      }
      [
        ...nodesUnseen.keys()
      ].forEach((nodeUnseen) => {
        if (nodeUnseen) {
          allValues.set(nodeUnseen, void 0);
        }
      });
      return [
        allValues,
        impliedNodegroups
      ];
    }
  }
  GraphMutator = class {
    constructor(baseGraph, options = {}) {
      __publicField(this, "baseGraph");
      __publicField(this, "mutations");
      __publicField(this, "autocreateCard");
      this.baseGraph = baseGraph;
      this.mutations = [];
      this.autocreateCard = options.autocreateCard === void 0 || options.autocreateCard;
    }
    _generateUuidv5(key) {
      return generateUuidv5([
        "graph",
        this.baseGraph.graphid
      ], key);
    }
    _generateEdge(fromNode, toNode, ontologyProperty, name, description) {
      const edgeId = this._generateUuidv5(`node-${fromNode}-${toNode}`);
      return new StaticEdge({
        description: description || null,
        domainnode_id: fromNode,
        edgeid: edgeId,
        graph_id: this.baseGraph.graphid,
        name: name || null,
        rangenode_id: toNode,
        ontologyproperty: ontologyProperty
      });
    }
    addSemanticNode(parentAlias, alias, name, cardinality, ontologyClass, parentProperty, description, options = {}, config) {
      return this._addGenericNode(parentAlias, alias, name, cardinality, "semantic", ontologyClass, parentProperty, description, options, config);
    }
    addConceptNode(parentAlias, alias, name, collection, cardinality, ontologyClass, parentProperty, description, options = {}, config) {
      config = config || {};
      if (collection == null ? void 0 : collection.id) {
        config["rdmCollection"] = collection.id;
      }
      return this._addGenericNode(parentAlias, alias, name, cardinality, options.is_list ? "concept-list" : "concept", ontologyClass, parentProperty, description, options, config);
    }
    addCard(nodegroup, name, component, options = {}, config) {
      const nodegroupId = typeof nodegroup === "string" ? nodegroup : nodegroup.nodegroupid;
      const cardName = name instanceof StaticTranslatableString ? name : new StaticTranslatableString(name);
      const cardComponent = component || DEFAULT_CARD_COMPONENT;
      const helptext = (options == null ? void 0 : options.helptext) && (options.helptext instanceof StaticTranslatableString ? options.helptext : new StaticTranslatableString(options.helptext));
      const helptitle = (options == null ? void 0 : options.helptitle) && (options.helptitle instanceof StaticTranslatableString ? options.helptitle : new StaticTranslatableString(options.helptitle));
      const instructions = (options == null ? void 0 : options.instructions) && (options.instructions instanceof StaticTranslatableString ? options.instructions : new StaticTranslatableString(options.instructions));
      this.mutations.push((graph) => {
        graph.cards = graph.cards || [];
        if (graph.cards.filter((card2) => card2.nodegroup_id === nodegroup).length > 0) {
          throw Error(`This nodegroup, ${nodegroupId}, already has a card`);
        }
        const cardId = this._generateUuidv5(`card-ng-${nodegroupId}`);
        const card = new StaticCard({
          active: options.active === void 0 ? true : options.active,
          cardid: cardId,
          component_id: cardComponent.id,
          config: config || null,
          constraints: options.constraints || [],
          cssclass: options.cssclass || null,
          description: options.description || null,
          graph_id: graph.graphid,
          helpenabled: !!(options.helpenabled || options.helpenabled === void 0 && (helptext || helptitle)),
          helptext: helptext || new StaticTranslatableString(""),
          helptitle: helptitle || new StaticTranslatableString(""),
          instructions: instructions || new StaticTranslatableString(""),
          is_editable: options.is_editable === void 0 ? true : options.is_editable,
          name: cardName,
          nodegroup_id: nodegroupId,
          sortorder: options.sortorder || null,
          visible: options.visible === void 0 ? true : options.visible
        });
        graph.cards.push(card);
        return graph;
      });
    }
    addStringNode(parentAlias, alias, name, cardinality, ontologyClass, parentProperty, description, options = {}, config) {
      return this._addGenericNode(parentAlias, alias, name, cardinality, "string", ontologyClass, parentProperty, description, options, config);
    }
    _addNodegroup(parentAlias, nodegroupId, cardinality, name) {
      this.mutations.push((graph) => {
        const prnt = parentAlias === null ? graph.root : graph.nodes.find((node) => node.alias === parentAlias);
        if (!prnt) {
          throw Error(`Missing parent for nodegroup: ${parentAlias}`);
        }
        const nodegroup = new StaticNodegroup({
          cardinality,
          legacygroupid: null,
          nodegroupid: nodegroupId,
          parentnodegroup_id: prnt.nodegroup_id
        });
        graph.nodegroups.push(nodegroup);
        return graph;
      });
      if (this.autocreateCard) {
        this.addCard(nodegroupId, name || "(unnamed)");
      }
      return this;
    }
    _addGenericNode(parentAlias, alias, name, cardinality, datatype, ontologyClass, parentProperty, description, options = {}, config) {
      const nodeId = this._generateUuidv5(`node-${alias}`);
      const node = {
        alias,
        config: config || {},
        datatype,
        description: description || null,
        exportable: options.exportable || false,
        fieldname: options.fieldname || null,
        graph_id: this.baseGraph.graphid,
        hascustomalias: options.hascustomalias || false,
        is_collector: options.is_collector || false,
        isrequired: options.isrequired || false,
        issearchable: options.issearchable || true,
        istopnode: options.istopnode || false,
        name,
        nodegroup_id: "",
        nodeid: nodeId,
        parentproperty: parentProperty,
        sortorder: options.sortorder || 0,
        ontologyclass: ontologyClass,
        sourcebranchpublication_id: null
      };
      if (cardinality === "n" || parentAlias === null) {
        node.nodegroup_id = nodeId;
        this._addNodegroup(parentAlias, node.nodegroup_id, cardinality, new StaticTranslatableString(name));
      }
      this.mutations.push((graph) => {
        const prnt = parentAlias === null ? graph.root : graph.nodes.find((node2) => node2.alias === parentAlias);
        if (!prnt) {
          throw Error(`Parent node does not exist: ${parentAlias}`);
        }
        node.nodegroup_id = node.nodegroup_id !== "" ? node.nodegroup_id : prnt.nodegroup_id || "";
        const newNode = new StaticNode(node);
        graph.nodes.push(newNode);
        const edge = this._generateEdge(prnt.nodeid, nodeId, parentProperty);
        graph.edges.push(edge);
        return graph;
      });
      if (this.autocreateCard && datatype !== "semantic") {
        const widget = getDefaultWidgetForNode(node);
        const config2 = widget.getDefaultConfig();
        config2.label = name;
        this.addWidgetToCard(nodeId, widget, name, config2, {
          sortorder: node.sortorder,
          silentSkip: true
        });
      }
      return this;
    }
    addWidgetToCard(nodeId, widget, name, config, options = {}) {
      this.mutations.push((graph) => {
        var _a2;
        const node = graph.nodes.find((node2) => node2.nodeid === nodeId);
        if (!node) {
          throw Error(`Tried to add card to graph ${graph.graphid} for node ${nodeId} but it was not found.`);
        }
        const card = (_a2 = graph.cards) == null ? void 0 : _a2.find((card2) => card2.nodegroup_id === node.nodegroup_id);
        if (card) {
          const cardXNodeXWidgetId = this._generateUuidv5(`cxnxw-${nodeId}-${widget.id}`);
          const cardXNodeXWidget = new StaticCardsXNodesXWidgets({
            card_id: card.cardid,
            config,
            id: cardXNodeXWidgetId,
            label: new StaticTranslatableString(name),
            node_id: nodeId,
            sortorder: options.sortorder || 0,
            visible: options.visible === void 0 || options.visible,
            widget_id: widget.id
          });
          graph.cards_x_nodes_x_widgets = graph.cards_x_nodes_x_widgets || [];
          graph.cards_x_nodes_x_widgets.push(cardXNodeXWidget);
        } else if (!options.silentSkip) {
          throw Error(`Failed adding widget for ${nodeId} to card for ${node.nodegroup_id} on graph ${graph.graphid}, as no card for this nodegroup (yet?)`);
        }
        return graph;
      });
      return this;
    }
    apply() {
      if (!this.baseGraph.copy) {
        throw Error("Attempt to build a mutator without a proper StaticGraph base graph");
      }
      const graph = this.baseGraph.copy();
      return this.mutations.reduce((graph2, mutation) => mutation(graph2), graph);
    }
  };
  ResourceModelWrapper = class {
    constructor(wkrm, graph, viewModelClass) {
      __publicField(this, "wkrm");
      __publicField(this, "graph");
      __publicField(this, "viewModelClass");
      __publicField(this, "permittedNodegroups");
      __publicField(this, "edges");
      __publicField(this, "nodes");
      __publicField(this, "nodegroups");
      __publicField(this, "nodesByAlias");
      this.wkrm = wkrm;
      this.graph = graph;
      this.viewModelClass = viewModelClass;
    }
    getBranchPublicationIds(accessible) {
      const accessibleOnly = accessible || false;
      const nodes = [
        ...this.graph.nodes.values()
      ];
      return [
        ...nodes.reduce((acc, node) => {
          if (node.sourcebranchpublication_id) {
            if (accessibleOnly) {
              if (this.isNodegroupPermitted(node.nodegroup_id || "", null)) {
                acc.add(node.sourcebranchpublication_id);
              }
            } else {
              acc.add(node.sourcebranchpublication_id);
            }
          }
          return acc;
        }, /* @__PURE__ */ new Set())
      ];
    }
    getCollections(accessible) {
      const accessibleOnly = accessible || false;
      const nodes = [
        ...this.graph.nodes.values()
      ];
      return [
        ...nodes.reduce((acc, node) => {
          var _a2;
          if ([
            "concept",
            "concept-list"
          ].includes(node.datatype) && ((_a2 = node.config) == null ? void 0 : _a2.rdmCollection)) {
            if (accessibleOnly) {
              if (this.isNodegroupPermitted(node.nodegroup_id || "", null)) {
                acc.add(node.config.rdmCollection);
              }
            } else {
              acc.add(node.config.rdmCollection);
            }
          }
          return acc;
        }, /* @__PURE__ */ new Set())
      ];
    }
    pruneGraph(keepFunctions) {
      const allNodegroups = this.getNodegroupObjects();
      const root = this.graph.root.nodeid;
      const allowedNodegroups = new Map([
        ...allNodegroups.values()
      ].filter((nodegroup) => {
        return this.isNodegroupPermitted(nodegroup.nodegroupid || "", null);
      }).map((nodegroup) => [
        nodegroup.nodegroupid,
        nodegroup.nodegroupid === null || nodegroup.nodegroupid === "" || nodegroup.nodegroupid === root
      ]));
      const backedges = /* @__PURE__ */ new Map();
      for (const [d, rs] of this.getEdges()) {
        for (const r of rs) {
          if (backedges.has(r)) {
            throw Error(`Graph is malformed, node ${r} has multiple parents, ${backedges.get(r)} and ${d} at least`);
          }
          backedges.set(r, d);
        }
      }
      let loops = 0;
      allowedNodegroups.set(root, true);
      while (loops < MAX_GRAPH_DEPTH) {
        const unrooted = [
          ...allowedNodegroups.entries()
        ].filter(([_, rooted]) => !rooted);
        if (unrooted.length === 0) {
          break;
        }
        for (const [ng] of unrooted) {
          if (ng === root) {
            continue;
          }
          const next = backedges.get(ng);
          if (!next) {
            throw Error(`Graph does not have a parent for ${ng}`);
          }
          allowedNodegroups.set(ng, true);
          if (!allowedNodegroups.has(next)) {
            allowedNodegroups.set(next, false);
          }
        }
        loops += 1;
      }
      if (loops >= MAX_GRAPH_DEPTH) {
        throw Error("Hit edge traversal limit when pruning, is the graph well-formed without cycles?");
      }
      const allowedNodes = new Set([
        ...this.getNodeObjects().values()
      ].filter((node) => {
        return node.nodegroup_id && allowedNodegroups.get(node.nodegroup_id) || node.nodeid === root;
      }).map((node) => node.nodeid));
      this.graph.cards = (this.graph.cards || []).filter((card) => allowedNodegroups.get(card.nodegroup_id));
      this.graph.cards_x_nodes_x_widgets = (this.graph.cards_x_nodes_x_widgets || []).filter((card) => allowedNodes.has(card.node_id));
      this.graph.edges = (this.graph.edges || []).filter((edge) => (edge.domainnode_id === root || allowedNodes.has(edge.domainnode_id)) && allowedNodes.has(edge.rangenode_id));
      this.graph.nodegroups = (this.graph.nodegroups || []).filter((ng) => allowedNodegroups.has(ng.nodegroupid));
      this.graph.nodes = (this.graph.nodes || []).filter((node) => allowedNodes.has(node.nodeid));
      if (Array.isArray(keepFunctions) && this.graph.functions_x_graphs) {
        this.graph.functions_x_graphs = this.graph.functions_x_graphs.filter((fxg) => keepFunctions.includes(fxg.function_id));
      } else {
        this.graph.functions_x_graphs = [];
      }
    }
    exportGraph() {
      const graph = this.graph;
      return new StaticGraph({
        author: graph.author,
        cards: graph.cards,
        cards_x_nodes_x_widgets: graph.cards_x_nodes_x_widgets,
        color: graph.color,
        config: graph.config,
        deploymentdate: graph.deploymentdate,
        deploymentfile: graph.deploymentfile,
        description: graph.description,
        edges: graph.edges,
        functions_x_graphs: graph.functions_x_graphs,
        graphid: graph.graphid,
        iconclass: graph.iconclass,
        is_editable: graph.is_editable,
        isresource: graph.isresource,
        jsonldcontext: graph.jsonldcontext,
        name: graph.name,
        nodegroups: graph.nodegroups,
        nodes: graph.nodes,
        ontology_id: graph.ontology_id,
        publication: graph.publication,
        relatable_resource_model_ids: graph.relatable_resource_model_ids,
        resource_2_resource_constraints: graph.resource_2_resource_constraints,
        root: graph.root,
        slug: graph.slug,
        subtitle: graph.subtitle,
        template_id: graph.template_id,
        version: graph.version
      });
    }
    async all(params = void 0) {
      const paramObj = params || {
        limit: void 0,
        lazy: void 0
      };
      const promises = [];
      for await (const resource of this.iterAll(paramObj)) {
        promises.push(resource);
      }
      return Promise.all(promises);
    }
    stripTiles(resource) {
      if (resource.tiles) {
        const nodes = this.getNodeObjects();
        resource.tiles = resource.tiles.filter((tile) => {
          const node = nodes.get(tile.nodegroup_id);
          if (!node) {
            throw Error(`Tile ${tile.tileid} has nodegroup ${tile.nodegroup_id} that is not on the model ${this.graph.graphid}`);
          }
          return this.isNodegroupPermitted(tile.nodegroup_id || "", tile);
        });
      }
    }
    async *resourceGenerator(staticResources, lazy = false, pruneTiles = true) {
      for await (const staticResource of staticResources) {
        yield this.fromStaticResource(staticResource, lazy, pruneTiles);
      }
    }
    async *iterAll(params) {
      yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy);
    }
    async findStatic(id) {
      return await staticStore.loadOne(id);
    }
    async find(id, lazy = true, pruneTiles = true) {
      const rivm = await this.findStatic(id);
      return this.fromStaticResource(rivm, lazy, pruneTiles);
    }
    setPermittedNodegroups(permissions) {
      const nodegroups = this.getNodegroupObjects();
      const nodes = this.getNodeObjectsByAlias();
      this.permittedNodegroups = new Map([
        ...permissions
      ].map(([key, value]) => {
        const k = key || "";
        if (nodegroups.has(k) || k === "") {
          return [
            key,
            value
          ];
        } else {
          const node = nodes.get(k);
          if (node) {
            return [
              node.nodeid,
              value
            ];
          } else {
            throw Error(`Could not find ${key} in nodegroups for permissions`);
          }
        }
      }));
    }
    getPermittedNodegroups() {
      if (!this.permittedNodegroups) {
        const permissions = new Map([
          ...this.getNodegroupObjects()
        ].map(([k, _]) => [
          k,
          true
        ]));
        permissions.set("", true);
        this.setPermittedNodegroups(permissions);
      }
      const permittedNodegroups = this.permittedNodegroups;
      if (permittedNodegroups === void 0) {
        throw Error("Could not set permitted nodegroups");
      }
      return permittedNodegroups;
    }
    isNodegroupPermitted(nodegroupId, tile) {
      let permitted = this.getPermittedNodegroups().get(nodegroupId);
      if (permitted && typeof permitted == "function") {
        const nodes = this.getNodeObjectsByAlias();
        permitted = permitted(nodegroupId, tile, nodes);
      }
      if (!permitted) {
        return false;
      }
      if (permitted === true) {
        return true;
      }
      throw Error(`Ambiguous permission state: ${permitted} for nodegroup ${nodegroupId}`);
    }
    makeInstance(id, resource, pruneTiles = true) {
      if (!this.viewModelClass) {
        throw Error(`Cannot instantiate without a viewModelClass in ${this.wkrm.modelClassName}`);
      }
      const instance = new this.viewModelClass(id, this.viewModelClass.prototype.__, (rivm) => new ResourceInstanceWrapper(rivm, this, resource, pruneTiles), null);
      return instance;
    }
    getChildNodes(nodeId) {
      const childNodes = /* @__PURE__ */ new Map();
      const edges = this.getEdges().get(nodeId);
      if (edges) {
        for (const [, n] of this.getNodeObjects()) {
          if (edges.includes(n.nodeid)) {
            if (n.alias) {
              childNodes.set(n.alias, n);
            }
          }
        }
      }
      return childNodes;
    }
    buildNodes() {
      if (this.nodes || this.nodegroups) {
        throw Error("Cache should never try and rebuild nodes when non-empty");
      }
      this.edges = /* @__PURE__ */ new Map();
      this.nodes = /* @__PURE__ */ new Map();
      this.nodegroups = /* @__PURE__ */ new Map();
      const graph = this.graph ?? graphManager.getGraph(this.wkrm.graphId);
      if (!graph) {
        throw Error(`Could not find graph ${this.wkrm.graphId} for resource`);
      }
      const nodes = new Map(graph.nodes.map((node) => [
        node.nodeid,
        node
      ]));
      const nodegroups = new Map(graph.nodes.filter((node) => node.nodegroup_id).map((node) => [
        node.nodegroup_id || "",
        new StaticNodegroup({
          cardinality: "n",
          legacygroupid: null,
          nodegroupid: node.nodegroup_id || "",
          parentnodegroup_id: null
        })
      ]));
      for (const nodegroup of graph.nodegroups) {
        nodegroups.set(nodegroup.nodegroupid, nodegroup);
      }
      const edgePairs = graph.edges.map((edge) => [
        edge.domainnode_id,
        edge.rangenode_id
      ]);
      const edges = edgePairs.reduce((edges2, dr) => {
        const range = edges2.get(dr[0]) || [];
        range.push(dr[1]);
        edges2.set(dr[0], range);
        return edges2;
      }, /* @__PURE__ */ new Map());
      this.nodes = nodes;
      this.nodegroups = nodegroups;
      this.edges = edges;
      this.nodesByAlias = new Map([
        ...nodes.values()
      ].map((node) => [
        node.alias || "",
        node
      ]));
    }
    getNodeObjectsByAlias() {
      if (!this.nodesByAlias) {
        this.buildNodes();
      }
      if (!this.nodesByAlias) {
        throw Error("Could not build nodes");
      }
      return this.nodesByAlias;
    }
    getEdges() {
      if (!this.edges) {
        this.buildNodes();
      }
      if (!this.edges) {
        throw Error("Could not build edges");
      }
      return this.edges;
    }
    getNodeObjects() {
      if (!this.nodes) {
        this.buildNodes();
      }
      if (!this.nodes) {
        throw Error("Could not build nodes");
      }
      return this.nodes;
    }
    getNodegroupObjects() {
      if (!this.nodegroups) {
        this.buildNodes();
      }
      if (!this.nodegroups) {
        throw Error("Could not build nodegroups");
      }
      return this.nodegroups;
    }
    getRootNode() {
      const nodes = this.getNodeObjects();
      const rootNode = [
        ...nodes.values()
      ].find((n) => !n.nodegroup_id);
      if (!rootNode) {
        throw Error(`COULD NOT FIND ROOT NODE FOR ${this.wkrm.modelClassName}. Does the graph ${this.graph.graphid} still exist?`);
      }
      rootNode.alias = rootNode.alias || "";
      return rootNode;
    }
    fromStaticResource(resource, lazy = false, pruneTiles = true) {
      const wkri = this.makeInstance(resource.resourceinstance.resourceinstanceid, resource, pruneTiles);
      if (!wkri.$) {
        throw Error("Could not load resource from static definition");
      }
      return wkri.$.populate(lazy).then(() => wkri);
    }
    asTree() {
      const root = this.getRootNode();
      const nodegroups = this.getNodegroupObjects();
      const addChildren = (node) => {
        const branch = {};
        const children = this.getChildNodes(node.nodeid);
        if (!children.size) {
          return false;
        }
        for (const child of children.values()) {
          const nodegroup = nodegroups.get(child.nodegroup_id || "");
          const multiple = child.nodegroup_id && child.is_collector && nodegroup && nodegroup.cardinality == "n" && node.nodegroup_id !== child.nodegroup_id || child.datatype.endsWith("-list");
          const childBranch = addChildren(child);
          const alias = child.alias || "";
          if (childBranch === false) {
            branch[alias] = child.datatype;
          } else {
            branch[alias] = childBranch;
            if (child.datatype !== "semantic") {
              branch[alias]["_"] = child.datatype;
            }
          }
          if (multiple) {
            branch[alias] = [
              branch[alias]
            ];
          }
        }
        return branch;
      };
      return addChildren(root) || {};
    }
  };
  function makeResourceModelWrapper(viewModelClass, wkrm, graph) {
    var _a2;
    let vmc;
    if (!viewModelClass) {
      const viewModelClassObj = {
        [wkrm.modelClassName]: (_a2 = class extends ResourceInstanceViewModel {
        }, __publicField(_a2, "_"), __publicField(_a2, "__"), _a2)
      };
      vmc = viewModelClassObj[wkrm.modelClassName];
    } else {
      vmc = viewModelClass;
    }
    const wrapper = new ResourceModelWrapper(wkrm, graph, vmc);
    vmc.prototype.__ = wrapper;
    return vmc;
  }
  GraphManager = class {
    constructor(archesClient2) {
      __publicField(this, "_initialized", false);
      __publicField(this, "archesClient");
      __publicField(this, "graphs");
      __publicField(this, "wkrms");
      this.archesClient = archesClient2;
      this.graphs = /* @__PURE__ */ new Map();
      this.wkrms = /* @__PURE__ */ new Map();
    }
    async initialize(configurationOptions = void 0) {
      if (this._initialized) {
        return;
      }
      if (configurationOptions === void 0) {
        configurationOptions = new ConfigurationOptions();
      }
      const graphJsons = await this.archesClient.getGraphs();
      let graphs = Object.entries(graphJsons["models"]);
      const allowedGraphs = configurationOptions.graphs;
      if (allowedGraphs !== null) {
        if (allowedGraphs === false) {
          throw Error("No current meaning of allowedGraphs === false");
        } else if (allowedGraphs !== true) {
          graphs = graphs.filter(([graphId, _]) => allowedGraphs.includes(graphId));
        }
      }
      graphs.forEach(([graphId, meta]) => {
        meta.graphid = meta.graphid || graphId;
        const wkrm = new WKRM(meta);
        this.wkrms.set(wkrm.modelClassName, wkrm);
      });
      if (configurationOptions.eagerLoadGraphs) {
        await Promise.all(graphs.map(([g]) => this.loadGraph(g)));
      }
      this._initialized = true;
    }
    async loadGraph(modelClass) {
      let modelClassName;
      if (typeof modelClass == "string") {
        modelClassName = modelClass;
      } else {
        modelClassName = modelClass.name;
      }
      let wkrm = this.wkrms.get(modelClassName);
      if (wkrm === void 0) {
        wkrm = [
          ...this.wkrms.values()
        ].find((wkrm2) => wkrm2.graphId === modelClassName);
        if (wkrm === void 0) {
          throw Error(`Only loading graphs for which metadata is present, not ${modelClassName}`);
        }
        modelClass = wkrm.modelClassName;
      }
      const wrapper = this.graphs.get(wkrm.graphId);
      if (wrapper !== void 0) {
        return wrapper;
      }
      const bodyJson = await this.archesClient.getGraph(wkrm.meta);
      if (!bodyJson) {
        throw Error(`Could not load graph ${wkrm.graphId}`);
      }
      const graph = new StaticGraph(bodyJson);
      let model;
      if (typeof modelClass == "string") {
        modelClassName = modelClass;
        model = makeResourceModelWrapper(void 0, wkrm, graph);
      } else {
        modelClassName = modelClass.name;
        model = makeResourceModelWrapper(modelClass, wkrm, graph);
      }
      this.graphs.set(graph.graphid, model.prototype.__);
      return model.prototype.__;
    }
    async get(modelClass) {
      let modelClassName;
      if (typeof modelClass == "string") {
        modelClassName = modelClass;
      } else {
        modelClassName = modelClass.name;
      }
      this.initialize(void 0);
      let wkrm = this.wkrms.get(modelClassName);
      if (wkrm === void 0) {
        wkrm = [
          ...this.wkrms.values()
        ].find((w) => w.graphId === modelClassName);
        if (wkrm === void 0) {
          throw Error(`Cannot find model requested: ${modelClassName}`);
        }
      }
      const wrapper = this.graphs.get(wkrm.graphId);
      if (wrapper === void 0) {
        return this.loadGraph(modelClass);
      }
      return wrapper;
    }
    async getResource(resourceId, lazy = true, pruneTiles = true) {
      const rivm = await staticStore.loadOne(resourceId);
      let graph = this.graphs.get(rivm.resourceinstance.graph_id);
      if (!graph) {
        graph = await this.loadGraph(rivm.resourceinstance.graph_id);
        if (!graph) {
          throw Error(`Graph not found for resource ${resourceId}`);
        }
      }
      return graph.fromStaticResource(rivm, lazy, pruneTiles);
    }
    getGraph(graphId) {
      const wrapper = this.graphs.get(graphId);
      if (wrapper === void 0) {
        throw Error(`Cannot find graph requested: ${graphId}`);
      }
      return wrapper.graph;
    }
  };
  graphManager = new GraphManager(archesClient);
  viewContext.graphManager = graphManager;
  class Cleanable extends String {
    constructor() {
      super(...arguments);
      __publicField(this, "__clean");
    }
  }
  class BaseRenderer {
    async render(asset) {
      if (!asset.$) {
        throw Error("Cannot render unloaded asset - do you want to await asset.retrieve()?");
      }
      const root = await await asset.$.getRootViewModel();
      return this.renderValue(root, 0);
    }
    async renderValue(value, depth) {
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
      } else if (value instanceof StringViewModel || value instanceof NonLocalizedStringViewModel || typeof value === "string") {
        newValue = this.renderString(value, depth);
      } else if (value instanceof BooleanViewModel) {
        newValue = this.renderBoolean(value, depth);
      } else if (value instanceof NumberViewModel) {
        newValue = this.renderNumber(value, depth);
      } else if (value instanceof GeoJSONViewModel) {
        newValue = this.renderBlock(await value.forJson(), depth);
      } else if (value instanceof UrlViewModel) {
        newValue = this.renderUrl(await value, depth);
      } else if (value instanceof Object) {
        newValue = this.renderBlock(value, depth);
      } else {
        newValue = value;
      }
      return newValue;
    }
  }
  class Renderer extends BaseRenderer {
    async renderDomainValue(value, _depth) {
      return value;
    }
    async renderString(value, _depth) {
      return `${value}`;
    }
    async renderNumber(value, _depth) {
      return `${value}`;
    }
    async renderBoolean(value, _depth) {
      return value.toString();
    }
    async renderDate(value, _depth) {
      return value;
    }
    async renderConceptValue(value, _depth) {
      return value;
    }
    async renderResourceReference(value, _depth) {
      return value;
    }
    async renderSemantic(value, depth) {
      return this.renderBlock(await value.toObject(), depth);
    }
    async renderUrl(value, _depth) {
      return value;
    }
    renderBlock(block, depth) {
      const renderedBlock = {};
      const promises = [];
      for (const [key, value] of Object.entries(block)) {
        promises.push(this.renderValue(value, depth + 1).then((val) => {
          renderedBlock[key] = val;
        }));
      }
      return Promise.all(promises).then(() => renderedBlock);
    }
    async renderArray(value, depth) {
      return Promise.all(value.map((val) => this.renderValue(val, depth + 1)));
    }
  }
  class MarkdownRenderer extends Renderer {
    constructor(callbacks) {
      super();
      __publicField(this, "conceptValueToUrl");
      __publicField(this, "dateToText");
      __publicField(this, "domainValueToUrl");
      __publicField(this, "resourceReferenceToUrl");
      __publicField(this, "nodeToUrl");
      this.conceptValueToUrl = callbacks.conceptValueToUrl;
      this.dateToText = callbacks.dateToText;
      this.domainValueToUrl = callbacks.domainValueToUrl;
      this.resourceReferenceToUrl = callbacks.resourceReferenceToUrl;
      this.nodeToUrl = callbacks.nodeToUrl;
    }
    async renderUrl(value, _depth) {
      const text = `[${value}](${value})`;
      const wrapper = new Cleanable(text);
      wrapper.__clean = value.href();
      return wrapper;
    }
    async renderDomainValue(domainValue, _) {
      const value = await domainValue.getValue();
      const url = this.domainValueToUrl ? await this.domainValueToUrl(domainValue) : null;
      const text = url ? `[${value.toString()}](${url.trim()})` : value.toString();
      const wrapper = new Cleanable(`
    <span
      class='alizarin-domain-value' data-id='${value.id}'
    >
      ${text}
    </span>`.replace(/\n/g, " ").trim());
      wrapper.__clean = domainValue.toString();
      return wrapper;
    }
    async renderDate(date, _) {
      const value = await date;
      const text = this.dateToText ? await this.dateToText(value) : value.toISOString();
      const wrapper = new Cleanable(`
    <time datetime='${text}'>
      ${text}
    </time>`.replace(/\n/g, " ").trim());
      wrapper.__clean = text;
      return wrapper;
    }
    async renderConceptValue(conceptValue, _) {
      const value = await conceptValue.getValue();
      const url = this.conceptValueToUrl ? await this.conceptValueToUrl(conceptValue) : null;
      const text = url ? `[${value.value}](${url.trim()})` : value.value;
      const wrapper = new Cleanable(`
    <span
      class='alizarin-concept-value' data-id='${value.id}'
      data-concept-id='${value.__concept ? value.__concept.id : ""}'
      data-concept-ref='$${value.__concept ? value.__concept.source : ""}'
    >
      ${text}
    </span>`.replace(/\n/g, " ").trim());
      wrapper.__clean = conceptValue.toString();
      return wrapper;
    }
    async renderResourceReference(rivm, _) {
      const value = await rivm.forJson(false);
      const url = this.resourceReferenceToUrl ? await this.resourceReferenceToUrl(rivm) : null;
      let title = value.title || value.type || "Resource";
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
    </span>`.replace(/\n/g, " ").trim());
      wrapper.__clean = rivm.toString();
      return wrapper;
    }
  }
  class FlatMarkdownRenderer extends MarkdownRenderer {
    async renderSemantic(vm, depth) {
      const children = [
        ...(await vm.__getChildValues()).entries()
      ].map(([_, v]) => [
        v.node.alias,
        v.node
      ]);
      const nodes = Object.fromEntries(await Promise.all(children));
      return super.renderSemantic(vm, depth).then(async (block) => {
        const text = [
          `* <span class='node-type'>${vm.__node.name}</span> &rarr;`,
          ...Object.entries(await block).map(([key, value]) => {
            const node = nodes[key];
            let nodeName = node.name;
            if (this.nodeToUrl) {
              nodeName = `[${node.name}](${this.nodeToUrl(node)})`;
            }
            if ((typeof value == "string" || value instanceof String) && value.indexOf("\n") != -1) {
              return `  * <span class='node-name'>${nodeName}</span> <span class='node-alias'>[*${node.alias}*]</span>:<span class='node-value'>
${value.split("\n").map((x) => `    ${x}`).join("\n")}
    </span>`;
            } else {
              return `  * <span class='node-name'>${nodeName}</span> <span class='node-alias'>[*${node.alias}*]</span>: <span class='node-value'>${value}</span>`;
            }
          }).join("\n").split("\n")
        ];
        if (text[1] == "") {
          text[0] += `<span class='node-empty'>&lt;empty&gt;</span>`;
          text.pop();
        }
        return text.map((line) => `  ${line}`).join("\n");
      });
    }
    async renderArray(value, depth) {
      const rows = await super.renderArray(value, depth);
      if (value instanceof PseudoList || value.indexOf("\n") != -1) {
        return rows.map((x) => `${x}`).join("\n");
      } else {
        return rows.join(", ");
      }
    }
    async renderString(value, _depth) {
      if (value.indexOf("\n") != -1) {
        value = "\n    " + value.split("\n").join("\n    ");
      }
      return value;
    }
  }
  class JsonRenderer extends Renderer {
    async renderDate(value, _depth) {
      return value.forJson();
    }
    async renderBoolean(value, _depth) {
      return typeof value === "boolean" ? value : value.forJson();
    }
    async renderConceptValue(value, _depth) {
      return value.forJson();
    }
    async renderDomainValue(value, _depth) {
      return value.forJson();
    }
    async renderResourceReference(value, _depth) {
      const val = value.forJson();
      return val;
    }
  }
  renderers = Object.freeze(Object.defineProperty({
    __proto__: null,
    Cleanable,
    FlatMarkdownRenderer,
    JsonRenderer,
    MarkdownRenderer
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  const wasmURL = "data:application/wasm;base64,AGFzbQEAAAAB3QI3YAJ/fwBgAn9/AX9gA39/fwF/YAN/f38AYAACf39gAX8AYAF/AX9gAX8Cf39gBH9/f38AYAFvAX9gAAN/f39gBH9/f38Bf2ACf28AYAAAYAV/f39/fwBgAAF/YAABb2ABbwFvYAZ/f39/f38AYAJ/fABgAX8BfGACb28Bb2ACb28Bf2ADf39/AX5gAX8Df39/YAJ/bwJ/f2ABfwFvYAV/f39/fwF/YAJ/fwFvYAF+AW9gBn9/f39/fwF/YAJ/fwF+YAFvA39/f2ABfgF/YAFvAGADb29vAGACb38Bb2ADb39vAGADb29vAW9gA39/bwBgA29vbwF/YAFvAXxgAXwBb2AJf39/f39/fn5+AGADfn9/AX9gB39/f39/f38Bf2AFf39+f38AYAR/fn9/AGAFf399f38AYAR/fX9/AGAFf398f38AYAR/fH9/AGACf34AYANvf38Bf2ABfAF/Aq4QNQN3YmccX193YmdfZXJyb3JfOTk5ODFlMTZkNDc2YWE1YwAiA3diZx1fX3diZ19TdHJpbmdfOGYwZWIzOWE0YTRjMmY2NgAMA3diZyRfX3diZ19nZXR3aXRocmVma2V5XzFkYzM2MWJkMTAwNTNiZmUAFQN3YmcaX193Ymdfc2V0XzNmMWQwYjk4NGVkMjcyZWQAIwN3YmcaX193YmdfZ2V0XzBkYTcxNWNlYWVjZWE1YzgAJAN3YmcdX193YmdfbGVuZ3RoXzE4NjU0NmM1MWNkNjFhY2QACQN3YmcaX193YmdfbmV3XzFmM2EzNDRjZjMxMjM3MTYAEAN3YmcaX193YmdfbmV3XzJmZjFmNjhmMzY3NmVhNTMAEAN3YmcbX193YmdfbmV4dF81YjM1MzBlNjEyZmRlNzdkABEDd2JnG19fd2JnX2RvbmVfNzVlZDBlZTZkZDI0M2Q5ZAAJA3diZxxfX3diZ192YWx1ZV9kZDkzNzIyMzA1MzFlYWRlABEDd2JnH19fd2JnX2l0ZXJhdG9yX2YzNzBiMzQ0ODNjNzFhMWMAEAN3YmcaX193YmdfbmV3XzE5YzI1YTNmMmZhNjNhMDIAEAN3YmcaX193Ymdfc2V0XzkwZjZjMGY3YmQ4YzA0MTUAJQN3YmceX193YmdfaXNBcnJheV8wMzBjY2UyMjA1OTFmYjQxAAkDd2JnG19fd2JnX3B1c2hfMzMwYjJlYjkzZTRlMTIxMgAWA3diZy1fX3diZ19pbnN0YW5jZW9mX0FycmF5QnVmZmVyXzY3ZjMwMTI1MjlmNmEyZGQACQN3YmcbX193YmdfY2FsbF8xMzQxMGFhYzU3MGZmZmY3ABUDd2JnJV9fd2JnX2luc3RhbmNlb2ZfTWFwX2ViYjAxYTViNmI1ZmZkMGIACQN3YmcaX193Ymdfc2V0X2I3ZjFjZjRmYWUyNmZlMmEAJgN3YmcbX193YmdfbmV4dF82OTJlODIyNzkxMzFiMDNjABEDd2JnJF9fd2JnX2lzU2FmZUludGVnZXJfMWMwZDFhZjU1NDJlMTAyYQAJA3diZx5fX3diZ19lbnRyaWVzXzJiZTJmMTViZDU1NTQ5OTYAEQN3YmcdX193YmdfbGVuZ3RoXzZiYjdlODFmOWQ3NzEzZTQACQN3YmcnX193YmdfcHJvdG90eXBlc2V0Y2FsbF8zZDRhMjZjMWVkNzM0MzQ5ACcDd2JnGl9fd2JnX25ld182MzhlYmZhZWRiZjMyYTVlABEDd2JnLF9fd2JnX2luc3RhbmNlb2ZfVWludDhBcnJheV85YTgzNzhkOTU1OTMzZGI3AAkDd2JnGl9fd2JnX2dldF80NThlODc0YjQzYjE4YjI1ABUDd2JnGl9fd2JnX3NldF80NTMzNDViY2RhODBiODlhACgDd2JnGl9fd2JnX25ld184YTZmMjM4YTZlY2U4NmVhABADd2JnHF9fd2JnX3N0YWNrXzBlZDc1ZDY4NTc1YjBmM2MADAN3YmccX193YmdfZXJyb3JfNzUzNGI4ZTlhMzZmMWFiNAAAA3diZypfX3diZ193YmluZGdlbmRlYnVnc3RyaW5nXzk5ZWYyNTdhM2RkZGEzNGQADAN3YmccX193YmdfRXJyb3JfZTE3ZTc3N2FhYzEwNTI5NQAcA3diZx1fX3diZ19OdW1iZXJfOTk4YmVhMzNiZDg3YzNlMAApA3diZypfX3diZ193YmluZGdlbmlzdW5kZWZpbmVkX2M0YjcxZDA3M2I5MmYzYzUACQN3YmcnX193Ymdfd2JpbmRnZW5pc29iamVjdF8zMDdhNTNjNmJkOTdmYmY4AAkDd2JnKV9fd2JnX3diaW5kZ2VuaXNmdW5jdGlvbl84Y2VlN2RjZTM3MjVhZTc0AAkDd2JnJ19fd2JnX3diaW5kZ2VuaXNzdHJpbmdfZDRmYTkzOTc4OWYwMDNiMAAJA3diZydfX3diZ193YmluZGdlbmlzYmlnaW50X2VjYjkwY2MwOGE1YTkxNTQACQN3YmchX193Ymdfd2JpbmRnZW5pbl9kN2ExZWUxMDkzM2QyZDU1ABYDd2JnKF9fd2JnX3diaW5kZ2VubnVtYmVyZ2V0X2Y3NGI0Yzc1MjVhYzA1Y2IADAN3YmcpX193Ymdfd2JpbmRnZW5ib29sZWFuZ2V0XzNmZTZmNjQyYzdkOTc3NDYACQN3YmcoX193Ymdfd2JpbmRnZW5zdHJpbmdnZXRfMGYxNmE2ZGRkZGVmMzc2ZgAMA3diZy1fX3diZ193YmluZGdlbmJpZ2ludGdldGFzaTY0X2FjNzQzZWNlNmFiOWJiYTEADAN3YmckX193Ymdfd2JpbmRnZW50aHJvd180NTFlYzFhODQ2OWQ3ZWI2AAADd2JnJl9fd2JnX3diaW5kZ2VuanN2YWxlcV9lNmYyYWQ1OWNjYWUxYjU4ABYDd2JnK19fd2JnX3diaW5kZ2VuanN2YWxsb29zZWVxXzliZWM4YzliZTgyNmJlZDEAFgN3YmcfX193YmluZGdlbl9pbml0X2V4dGVybnJlZl90YWJsZQANA3diZyBfX3diaW5kZ2VuX2Nhc3RfZDZjZDE5YjgxNTYwZmQ2ZQAqA3diZyBfX3diaW5kZ2VuX2Nhc3RfNDYyNWM1NzdhYjJlYzllZQAdA3diZyBfX3diaW5kZ2VuX2Nhc3RfMjI0MWI2YWY0YzRiMjk0MQAcA3diZyBfX3diaW5kZ2VuX2Nhc3RfOWFlMDYwNzUwN2FiYjA1NwAdA7sDuQMABgAAAQECAQAIAAMDAx4IBQgDAQgCAQYCAwIBAAIAAgAfHwABAAAAKwAIASwCABISLQMBAQAAAAMSAQEBAAABEgABAAABAA8DAAgACAsAAQYOBgEBAA4AAAMFAAEDAAgADggIAQgAAwMDAwMDAwMDAwMDAwADAAgGAAADAAMDAwMBBgAAAgABAAAAAAABAgAFAAEGBgYABQ4FAwMAAAAAAAEBBQAAAAAAEgMDAAUFAAACCAAAExMTABMTAAYDAwABAQMAAAAAAAAAAAAFBgEDAwUFBQMCBQIBBQUNAgEAAAUGFBQUBhQUDQAGAwAAAQYGAwEFAAUABgYGBgYGABsFBQUFBgULBQEFBQEBAAUABgAFIBgYGBggBQAFARkZGRkIAAIHBwcHBwcHBwcHBwcHBwcHBwcHBx4BFQUXFxsOLjAyAA8PCwUIBQABBRcLCwsAAgABBgg0BgUDBQEBAAMLCAICAwUBAiEhNQABAQEBDg8PDwwBAQMBBgYBAQEAAAABAQEGGhoaGgMBAwYGBgMDAQUBAQEBAQEBBQEFAQEFAQUNDQABAAEBAAECAQEBAQEFNgEBAAYADQ0ECQJwAWxsbwCAAQUDAQARBgkBfwFBgIDAAAsHrBpwBm1lbW9yeQIAI19fd2JnX3N0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19mcmVlANkBHHN0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19uZXcAngMhc3RhdGljdHJhbnNsYXRhYmxlc3RyaW5nX3RvU3RyaW5nANgCHXN0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19jb3B5ALcBIXN0YXRpY3RyYW5zbGF0YWJsZXN0cmluZ19nZXRfbGFuZwDZAilzdGF0aWN0cmFuc2xhdGFibGVzdHJpbmdfZ2V0X3RyYW5zbGF0aW9ucwC5AxpfX3diZ19zdGF0aWNncmFwaG1ldGFfZnJlZQCAARNzdGF0aWNncmFwaG1ldGFfbmV3AMcCG3N0YXRpY2dyYXBobWV0YV9nZXRfZ3JhcGhpZADaAhtzdGF0aWNncmFwaG1ldGFfc2V0X2dyYXBoaWQAsAEWc3RhdGljZ3JhcGhtZXRhX3RvSlNPTgC6AxlzdGF0aWNncmFwaG1ldGFfZ2V0QXV0aG9yANsCGXN0YXRpY2dyYXBobWV0YV9zZXRBdXRob3IAnAEdc3RhdGljZ3JhcGhtZXRhX2dldElzUmVzb3VyY2UAlgIdc3RhdGljZ3JhcGhtZXRhX3NldElzUmVzb3VyY2UA6QEYc3RhdGljZ3JhcGhtZXRhX2dldENhcmRzAJcCGHN0YXRpY2dyYXBobWV0YV9zZXRDYXJkcwDqASZzdGF0aWNncmFwaG1ldGFfZ2V0Q2FyZHNYTm9kZXNYV2lkZ2V0cwCYAiZzdGF0aWNncmFwaG1ldGFfc2V0Q2FyZHNYTm9kZXNYV2lkZ2V0cwDrARhzdGF0aWNncmFwaG1ldGFfZ2V0Q29sb3IA3AIYc3RhdGljZ3JhcGhtZXRhX3NldENvbG9yAJ0BH3N0YXRpY2dyYXBobWV0YV9nZXRfZGVzY3JpcHRpb24AyAEfc3RhdGljZ3JhcGhtZXRhX3NldF9kZXNjcmlwdGlvbgC9ARhzdGF0aWNncmFwaG1ldGFfZ2V0RWRnZXMAmQIYc3RhdGljZ3JhcGhtZXRhX3NldEVkZ2VzAOwBHHN0YXRpY2dyYXBobWV0YV9nZXRJY29uQ2xhc3MA3QIcc3RhdGljZ3JhcGhtZXRhX3NldEljb25DbGFzcwCeAR1zdGF0aWNncmFwaG1ldGFfZ2V0SXNFZGl0YWJsZQCaAh1zdGF0aWNncmFwaG1ldGFfc2V0SXNFZGl0YWJsZQDtASBzdGF0aWNncmFwaG1ldGFfZ2V0SnNvbkxkQ29udGV4dADIAiBzdGF0aWNncmFwaG1ldGFfc2V0SnNvbkxkQ29udGV4dADRAhhzdGF0aWNncmFwaG1ldGFfZ2V0X25hbWUAyQEYc3RhdGljZ3JhcGhtZXRhX3NldF9uYW1lAL4BHXN0YXRpY2dyYXBobWV0YV9nZXROb2RlR3JvdXBzAJsCHXN0YXRpY2dyYXBobWV0YV9zZXROb2RlR3JvdXBzAO4BGHN0YXRpY2dyYXBobWV0YV9nZXROb2RlcwCcAhhzdGF0aWNncmFwaG1ldGFfc2V0Tm9kZXMA7wEdc3RhdGljZ3JhcGhtZXRhX2dldE9udG9sb2d5SWQA3gIdc3RhdGljZ3JhcGhtZXRhX3NldE9udG9sb2d5SWQAnwEec3RhdGljZ3JhcGhtZXRhX2dldFB1YmxpY2F0aW9uAMkCHnN0YXRpY2dyYXBobWV0YV9zZXRQdWJsaWNhdGlvbgDSAixzdGF0aWNncmFwaG1ldGFfZ2V0UmVsYXRhYmxlUmVzb3VyY2VNb2RlbElkcwDKAixzdGF0aWNncmFwaG1ldGFfc2V0UmVsYXRhYmxlUmVzb3VyY2VNb2RlbElkcwDTAi9zdGF0aWNncmFwaG1ldGFfZ2V0UmVzb3VyY2UyUmVzb3VyY2VDb25zdHJhaW50cwDLAi9zdGF0aWNncmFwaG1ldGFfc2V0UmVzb3VyY2UyUmVzb3VyY2VDb25zdHJhaW50cwDUAhdzdGF0aWNncmFwaG1ldGFfZ2V0Um9vdACtARdzdGF0aWNncmFwaG1ldGFfc2V0Um9vdADRARhzdGF0aWNncmFwaG1ldGFfZ2V0X3NsdWcA3wIYc3RhdGljZ3JhcGhtZXRhX3NldF9zbHVnAKABHHN0YXRpY2dyYXBobWV0YV9nZXRfc3VidGl0bGUAygEcc3RhdGljZ3JhcGhtZXRhX3NldF9zdWJ0aXRsZQC/ARpzdGF0aWNncmFwaG1ldGFfZ2V0VmVyc2lvbgDgAhpzdGF0aWNncmFwaG1ldGFfc2V0VmVyc2lvbgChARVfX3diZ19zdGF0aWNub2RlX2ZyZWUA9AEOc3RhdGljbm9kZV9uZXcAzAIPc3RhdGljbm9kZV9jb3B5AIICEXN0YXRpY25vZGVfdG9KU09OALsDFHN0YXRpY25vZGVfZ2V0X2FsaWFzAOECFHN0YXRpY25vZGVfc2V0X2FsaWFzAKIBF3N0YXRpY25vZGVfZ2V0X2RhdGF0eXBlAOICF3N0YXRpY25vZGVfc2V0X2RhdGF0eXBlALIBGnN0YXRpY25vZGVfZ2V0X2Rlc2NyaXB0aW9uAOMCGnN0YXRpY25vZGVfc2V0X2Rlc2NyaXB0aW9uAKMBGXN0YXRpY25vZGVfZ2V0X2V4cG9ydGFibGUArAIZc3RhdGljbm9kZV9zZXRfZXhwb3J0YWJsZQD5ARhzdGF0aWNub2RlX2dldF9maWVsZG5hbWUA5AIYc3RhdGljbm9kZV9zZXRfZmllbGRuYW1lAKQBF3N0YXRpY25vZGVfZ2V0X2dyYXBoX2lkAOUCF3N0YXRpY25vZGVfc2V0X2dyYXBoX2lkALMBHXN0YXRpY25vZGVfZ2V0X2hhc2N1c3RvbWFsaWFzAK0CHXN0YXRpY25vZGVfc2V0X2hhc2N1c3RvbWFsaWFzAPoBG3N0YXRpY25vZGVfZ2V0X2lzX2NvbGxlY3RvcgCuAhtzdGF0aWNub2RlX3NldF9pc19jb2xsZWN0b3IA+wEZc3RhdGljbm9kZV9nZXRfaXNyZXF1aXJlZACvAhlzdGF0aWNub2RlX3NldF9pc3JlcXVpcmVkAPwBG3N0YXRpY25vZGVfZ2V0X2lzc2VhcmNoYWJsZQCwAhtzdGF0aWNub2RlX3NldF9pc3NlYXJjaGFibGUA/QEYc3RhdGljbm9kZV9nZXRfaXN0b3Bub2RlALECGHN0YXRpY25vZGVfc2V0X2lzdG9wbm9kZQD+ARNzdGF0aWNub2RlX2dldF9uYW1lAOYCE3N0YXRpY25vZGVfc2V0X25hbWUAtAEbc3RhdGljbm9kZV9nZXRfbm9kZWdyb3VwX2lkAOcCG3N0YXRpY25vZGVfc2V0X25vZGVncm91cF9pZAClARVzdGF0aWNub2RlX2dldF9ub2RlaWQA6AIVc3RhdGljbm9kZV9zZXRfbm9kZWlkALUBHHN0YXRpY25vZGVfZ2V0X29udG9sb2d5Y2xhc3MA6QIcc3RhdGljbm9kZV9zZXRfb250b2xvZ3ljbGFzcwCmAR1zdGF0aWNub2RlX2dldF9wYXJlbnRwcm9wZXJ0eQDqAh1zdGF0aWNub2RlX3NldF9wYXJlbnRwcm9wZXJ0eQCnARhzdGF0aWNub2RlX2dldF9zb3J0b3JkZXIAuAIYc3RhdGljbm9kZV9zZXRfc29ydG9yZGVyAIACKXN0YXRpY25vZGVfZ2V0X3NvdXJjZWJyYW5jaHB1YmxpY2F0aW9uX2lkAOsCKXN0YXRpY25vZGVfc2V0X3NvdXJjZWJyYW5jaHB1YmxpY2F0aW9uX2lkAKgBFHN0YXRpY25vZGVfZ2V0Q29uZmlnALwDFHN0YXRpY25vZGVfc2V0Q29uZmlnAKgDEnN0YXRpY25vZGVfY29tcGFyZQDuAidzdGF0aWNncmFwaG1ldGFfZ2V0X2lzcmVzb3VyY2VfcHJvcGVydHkAlgIfc3RhdGljdHJhbnNsYXRhYmxlc3RyaW5nX3RvSlNPTgC5Ax5zdGF0aWNub2RlX2dldF9jb25maWdfcHJvcGVydHkAvAMjc3RhdGljZ3JhcGhtZXRhX2dldF9hdXRob3JfcHJvcGVydHkA2wIec3RhdGljbm9kZV9zZXRfY29uZmlnX3Byb3BlcnR5AKgDBG1haW4AnQIFZ3JlZXQA7QMRX193YmluZGdlbl9tYWxsb2MApwISX193YmluZGdlbl9yZWFsbG9jALoCFF9fd2JpbmRnZW5fZXhuX3N0b3JlAJkDF19fZXh0ZXJucmVmX3RhYmxlX2FsbG9jAHwTX193YmluZGdlbl9leHBvcnRfNAEBD19fd2JpbmRnZW5fZnJlZQCYAxlfX2V4dGVybnJlZl90YWJsZV9kZWFsbG9jANgBEF9fd2JpbmRnZW5fc3Rh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gICAeEciAwRAIAJB2AJqIAJBnAFqKAIANgIAIAIgAikClAE3A9ACDAELIAJBgICAgHg2AtACCwJAIAIoAqABQYCAgIB4RyIERQRAQdeVwABBBhD1ASEHIABBgICAgHg2ApwBIAAgBzYCAEEAIQcMAQsgAkHoAmogAkGoAWooAgA2AgAgAiACKQKgATcD4AICQCACKAKsASISQYGAgIB4RwRAIAJB+AJqIAJBtAFqKAIANgIAIAIgAikCrAE3A/ACDAELIAJBgICAgHg2AvACCwJAIAIoArgBIhRBgYCAgHhHBEAgAkGIA2ogAkHAAWooAgA2AgAgAiACKQK4ATcDgAMMAQsgAkGAgICAeDYCgAMLIAdFBEAgEkGBgICAeEchByAUQYGAgIB4RyESQfiVwABBCRD1ASEIIABBgICAgHg2ApwBIAAgCDYCACACQYADahCPAyACQfACahCPAyACQeACahDQAwwBCyAAIAIpA9ABNwIkIAAgAikD4AE3AwAgACACKQJYNwJ4IABBLGogAkHYAWooAgA2AgAgAEEIaiACQegBaikDADcDACAAQRBqIAJB8AFqKQMANwMAIABBGGogAkH4AWopAwA3AwAgAEGAAWogAkHgAGooAgA2AgAgAigCxAEhASACKQLIASEVIABBOGogAkGYAmooAgA2AgAgACACKQOQAjcCMCAAIAIpA6ACNwI8IABBxABqIAJBqAJqKAIANgIAIAAgAikCfDcChAEgAEGMAWogAkGEAWooAgA2AgAgACACKQKIATcCkAEgAEGYAWogAkGQAWooAgA2AgAgAEHQAGogAkHYAmooAgA2AgAgACACKQPQAjcCSCAAIBBBAXE6AK0BIAAgD0EBcToArAEgACAOQQFxOgCrASAAIA1BAXE6AKoBIAAgDEEBcToAqQEgACAIQQFxOgCoASAAIBU3A3AgAEGAgICAeCABIAFBgYCAgHhGGzYCbCAAIAo2AiAgAEGkAWogAkGoAWooAgA2AgAgACACKQKgATcCnAEgAEHcAGogAkH4AmooAgA2AgAgACACKQPwAjcCVCAAQegAaiACQYgDaigCADYCACAAIAIpA4ADNwJgDCALIAJB0AJqEI8DIAJBwAJqENADDAILQaCVwABBDBD1AQshBCAAQYCAgIB4NgKcASAAIAQ2AgBBACEHQQAhBEEAIQNBACEFCyACQbACahDQAwsgAkGgAmoQjwMgE0GBgICAeEcLIQAgAkGQAmoQjwMgAkGAAmoQ0AMgEUGBgICAeEcLIQggAkHgAWoQhwIgAkHQAWoQjwMgAUEARwwYCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQywEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADBgLIAIoAugBIQYgAigCxAFBgYCAgHhHBEAgAkHEAWoQjwMLIAIgBjYCzAEgAiADNgLIASACIAU2AsQBDBMLDBoLIAIoAhggAkEANgIYBEAgAigCHCEGIwBBEGsiBSQAIAUgBjYCDCMAQTBrIgckACAHQQhqIgogBUEMaiIDEK4DBH4gCiADKAIAJQEQIvwGNwMIQgEFQgALNwMAQQEhCiAFAn8gBygCCEEBRgRAIAcpAxAiFUKAgICACH1C/////29WBEBBACEKIBWnDAILIAdBAjoAGCAHIBU3AyAjAEEwayIDJAAgA0Hwj8AANgIEIAMgB0EvajYCACADQQI2AgwgA0G8nMAANgIIIANCAjcCFCADQRc2AiwgA0EYNgIkIAMgB0EYajYCICADIANBIGo2AhAgAyADNgIoIANBCGoQhwEgA0EwaiQADAELIAMgB0EvakHwj8AAEFQLNgIEIAUgCjYCACAHQTBqJAAgBSgCBCEDIAUoAgAhByAGQYQBTwRAIAYQ2AELIAIgBzYCACACIAM2AgQgBUEQaiQAQQEhByACKAIEIQogAigCAEEBcUUNEiAAQYCAgIB4NgKcASAAIAo2AgAgAiAENgI8IAIgATYCOAwWCwwZCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQywEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADBYLIAIoAugBIQYgAigCuAFBgYCAgHhHBEAgAkG4AWoQjwMLIAIgBjYCwAEgAiADNgK8ASACIAU2ArgBDBELDBgLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDLASACKALkASEDIAIoAuABIgVBgYCAgHhGBEAgAiAENgI8IAIgATYCOCAAQYCAgIB4NgKcASAAIAM2AgAMFQsgAigC6AEhBiACKAKsAUGBgICAeEcEQCACQawBahCPAwsgAiAGNgK0ASACIAM2ArABIAIgBTYCrAEMEAsMFwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwUCyACKALoASEGIAJBoAFqEI8DIAIgBjYCqAEgAiADNgKkASACIAU2AqABDA8LDBYLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDLASACKALkASEDIAIoAuABIgVBgYCAgHhGBEAgAiAENgI8IAIgATYCOCAAQYCAgIB4NgKcASAAIAM2AgAMEwsgAigC6AEhBiACKAKUAUGBgICAeEcEQCACQZQBahCPAwsgAiAGNgKcASACIAM2ApgBIAIgBTYClAEMDgsMFQsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwSCyACKALoASEGIAJBiAFqEI8DIAIgBjYCkAEgAiADNgKMASACIAU2AogBDA0LDBQLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDVASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMEQsgAi0A4QEhEAwMCwwTCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQ1QEgAi0A4AEEQCACIAQ2AjwgAiABNgI4IAIoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADBALIAItAOEBIQ8MCwsMEgsgAigCGCACQQA2AhgEQCACQeABaiACKAIcENUBIAItAOABBEAgAiAENgI8IAIgATYCOCACKALkASEBIABBgICAgHg2ApwBIAAgATYCAAwPCyACLQDhASEODAoLDBELIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDVASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMDgsgAi0A4QEhDQwJCwwQCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQ1QEgAi0A4AEEQCACIAQ2AjwgAiABNgI4IAIoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADA0LIAItAOEBIQwMCAsMDwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwMCyACKALoASEGIAJB/ABqEI8DIAIgBjYChAEgAiADNgKAASACIAU2AnwMBwsMDgsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMsBIAIoAuQBIQMgAigC4AEiBUGBgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwLCyACKALoASEGIAIoAnBBgYCAgHhHBEAgAkHwAGoQjwMLIAIgBjYCeCACIAM2AnQgAiAFNgJwDAYLDA0LIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBDVASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMCgsgAi0A4QEhCAwFCwwMCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQywEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADAkLIAIoAugBIQYgAigCZEGBgICAeEcEQCACQeQAahCPAwsgAiAGNgJsIAIgAzYCaCACIAU2AmQMBAsMCwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEMABIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwICyACKALoASEGIAJB2ABqEI8DIAIgBjYCYCACIAM2AlwgAiAFNgJYDAMLDAoLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBA3IAIoAuQBIQMgAigC4AEiAUUEQCACIAQ2AjwgAkEANgI4IABBgICAgHg2ApwBIAAgAzYCAAwHCyAJIAspAwA3AwAgCUEQaiALQRBqKQMANwMAIAlBCGogC0EIaikDADcDACADIQQMAgsgAiAENgI8IAJBADYCOEH4psAAQTEQ2AMACyACKAIYIAJBADYCGEUNASACQeABaiACKAIcEMsBIAIoAuQB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cWtBBGsoAgAiAxCDAiEGIAcgAxCDAiEDAkAgBgRAQQAhBCADRQ0EIAYgAxA8RQ0EDAELIANFDQBBACEEDAMLIAlCAX0gCYMhCSAFQQFrIgUNAAsLQQEhBAsgAkEIahCOAgwBCwsgAkHwAGokACAEC7IfAwt/AX4BfCMAQYABayIDJAAgAyABNgIMAkACQAJAAkAgA0EMaiIBEMQCRQRAIAEQjANB/wFxIgFBAkcEQCAAIAE6AAEgAEEBOgAADAMLAkACQAJAIANBDGoiASgCACUBECdFBEAgA0EQaiABEKECIAMoAhBFDQEgAysDGCEOIAEQrgMNAyADQdgAaiIBIA69Qv///////////wCDQv/////////3/wBYBH4gASAOOQMIQgIFQgMLNwMAIANBADoAaCAAIAMpA1hCA1IEfyADQc8AaiADQeAAaikDADcAACADIAMpA1g3AEcgA0HoAGoQqAJBAgVBAAs6AAAgACADKQBANwABIABBCWogA0HIAGopAAA3AAAgAEEQaiADQc8AaikAADcAAAwGCyADIAMoAgwiATYCQCADQegAaiICIANBQGsiBBCiAiADKAJoQQFHDQEgAyADKQNwIg0QnQMiATYCaCAEIAIQoQMgAUGEAU8EQCABENgBCyADKAJAIQFFDQEgAUGEAU8EQCABENgBCyAAIA03AxAgAEECOgAAIAAgDUI/iDcDCAwHCyADQSRqIANBDGoQ1AEgAygCJEGAgICAeEcEQCADQfMAaiADQSxqKAIANgAAIABBAzoAACADIAMpAiQ3AGsgACADKQBoNwABIABBCGogA0HvAGopAAA3AAAMBQsgA0EMaiIBEK0DDQMgA0HoAGogARCuASADKAJoQYCAgIB4RwRAIANBOGogA0HwAGooAgA2AgAgAyADKQJoNwMwIwBBIGsiASQAIANBMGoiAikCBCENIAFBBjoACCABIA03AgwgAUEIaiABQR9qQZCdwAAQ5gEhBCAAQQY6AAAgACAENgIEIAIQ0AMgAUEgaiQADAULIANBDGoiAhDBAwRAIAMQpgMiATYCQCADQUBrIAIQoANFBEAgAUGEAUkNByABENgBDAcLIAFBhAFPBEAgARDYAQsgA0EMaigCACUBEBINBgsgA0EMaiADQUBrQYiRwAAQVCEBIABBBjoAACAAIAE2AgQMBAsgAyABNgJAIANB6ABqIgIgA0FAayIEEKICAkAgAygCaEEBRw0AIAMgAykDcCINEJwDIgE2AmggBCACEKEDIAFBhAFPBEAgARDYAQsgAygCQCEBRQ0AIAFBhAFPBEAgARDYAQsgACANNwMQIABCADcDCCAAQQI6AAAMBgtBmJLAAEHPABDCASECIABBBjoAACAAIAI2AgQgAUGEAUkNBSABENgBDAULIABBAjoAACAAIA78BiINNwMQIAAgDUI/iDcDCAwCCyAAQQA6AAAMAQsgAygCDCECIwBBMGsiASQAIAEgAjYCAAJAIAEQrQMEQCMAQUBqIgIkACACQQxqIAEQhQMgAkEANgIYIAJBADYCJCACQoCAgICAATcCHCACQShqQQFyIgVBCGohBiAFQQ9qIQgCQANAAkAgAkEoaiACQQxqEJsBAkACQCACLQAoIglBBmsOAgIAAQsgACACKAIsNgIEIABBBjoAACACQRxqIgAQwgIgABDTAwwDCyACKAIkIgcgAigCHEYEQCACQRxqQeiQwAAQlAILIAIoAiAgB0EYbGoiBCAFKQAANwABIAQgCToAACAEQQlqIAYpAAA3AAAgBEEQaiAIKQAANwAAIAIgB0EBajYCJAwBCwsgAkEzaiACQSRqKAIANgAAIABBBDoAACACIAIpAhw3ACsgACACKQAoNwABIABBCGogAkEvaikAADcAAAsgAkFAayQADAELIAFBGGogARB4IAEoAhghAgJAAkACQCABLQAcIgRBAmsOAgEAAgsgAEEGOgAAIAAgAjYCBAwCCyABIAFBGGpBiJHAABBUIQIgAEEGOgAAIAAgAjYCBAwBCyABIAQ6AAggASACNgIEIAFBADYCFCABQoCAgICAATcCDCABQRhqQQFyIgRBCGohByAEQQ9qIQYCQANAAkAgAUEYaiABQQRqEJMBAkACQCABLQAYIghBBmsOAgIAAQsgACABKAIcNgIEIABBBjoAACABQQxqIgAQwgIgABDTAyABKAIEIgBBgwFNDQQMAwsgASgCFCIFIAEoAgxGBEAgAUEMakHokMAAEJQCCyABKAIQIAVBGGxqIgIgBCkAADcAASACIAg6AAAgAkEJaiAHKQAANwAAIAJBEGogBikAADcAACABIAVBAWo2AhQMAQsLIAFBI2ogAUEUaigCADYAACAAQQQ6AAAgASABKQIM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AihBASECIAFBEGoQhwEMAgtBACECIAq6EOYDDAELQQAhAiAKEJwDCzYCBCAGIAI2AgAgAUEwaiQAIAQoAgghASAEKAIMDAILIAEpAwghCiMAQTBrIgEkACABIAo3AwggBEEQaiIGAn8gAi0AAkUEQCAKQv////////8PfEL/////////H1oEQCABQQI2AhQgAUGUqsAANgIQIAFCATcCHCABQR82AiwgASABQShqNgIYIAEgAUEIajYCKEEBIQIgAUEQahCHAQwCC0EAIQIgCrkQ5gMMAQtBACECIAoQnQMLNgIEIAYgAjYCACABQTBqJAAgBCgCECEBIAQoAhQMAQsgBEEYaiICIAErAwgQ5gM2AgQgAkEANgIAIAQoAhghASAEKAIcCyECIAUgATYCACAFIAI2AgQgBEEgaiQAIAMoAhQhASADKAIQDAQLIANBGGogAiABKAIIIAEoAgwQlQMgAygCHCEBIAMoAhgMAwsgA0EgaiACIAFBBGoQbSADKAIkIQEgAygCIAwCC0GBAUGAASACLQAAGyEBQQAMAQsgA0HYAGohBSABKAIMIQYCfyACLQABRQRAEAchCxB8IgQgCyYBQQAMAQsQpwMhBEEBCyEHIAUgAjYCECAFQQA2AgggBSAENgIEIAUgBzYCAAJAIAMoAlhBAkcEQCADQdAAaiADQegAaigCADYCACADQcgAaiADQeAAaikCADcDACADIAMpAlg3A0AgASgCCCECIAMgBkEAIAEoAgQiARs2AnggAyACNgJ0IAMgATYCcCADQQA2AmwgAyABQQBHIgQ2AmggAyACNgJkIAMgATYCYCADQQA2AlwgAyAENgJYAkADQCADQThqIANB2ABqEH4gAygCOCIERQ0BIAMoAjwhByMAQRBrIgUkACMAQRBrIgIkACACQQhqIANBQGsiASgCECAEKAIEIAQoAggQlQNBASEEIAIoAgwhBiACKAIIQQFxRQRAAkAgASgCCEUNACABKAIMIgRBhAFJDQAgBBDYAQsgASAGNgIMIAFBATYCCEEAIQQLIANBMGohCCAFQQhqIgkgBjYCBCAJIAQ2AgAgAkEQaiQAQQEhAgJ/IAUoAghBAXEEQCAFKAIMDAELIwBBIGsiBCQAIAEoAgghAiABQQA2AggCQAJAIAIEQCAEIAEoAgwiBjYCFCAEQQhqIAcgASgC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AAsgAEIANwMIIABBAjoAACAAIAEpAwg3AxAMAgsgACABKQMIEIsDDAELIAErAwghCSMAQSBrIgEkACABQQA6AAgCQCAJvUL///////////8Ag0L/////////9/8AWARAIAAgCTkDECAAQgI3AwggAEECOgAAIAFBCGoQ4gEMAQsgACABKQMINwMAIABBEGogAUEYaikDADcDACAAQQhqIAFBEGopAwA3AwALIAFBIGokAAsMBQsgASgCCCEFIAJBKGogASgCDCIBQQFBARCYASACKAIsIQQgAigCKEEBRg0CIAIoAjAhAyABBEAgAyAFIAH8CgAACyAAIAE2AgwgACADNgIIIAAgBDYCBCAAQQM6AAAMBAsgACABQQRqEHMMAwsgAkEoaiABKAIMIgMQkgMgAigCKEGBgICAeEYEQCAAIAIoAiw2AgQgAEEGOgAADAMLIAJBIGogAkE4aikCADcDACACQRhqIAJBMGopAgA3AwAgAiACKQIoNwMQIAEoAgghBCACIANBACABKAIEIgEbNgJIIAIgBDYCRCACIAE2AkAgAkEANgI8IAIgAUEARyIDNgI4IAIgBDYCNCACIAE2AjAgAkEANgIsIAIgAzYCKANAIAJBCGogAkEoahB+IAIoAggiA0UNAiACKAIMIQcCfyACQRBqIQQjAEEQayIBJAAgAygCBCEIIAFBBGogAygCCCIFQQFBARCYASABKAIIIQYgASgCBEEBRwRAIAEoAgwhAyAFBEAgAyAIIAX8CgAACyAGQYCAgIB4RwRAIAQQjwMgBCAFNgIIIAQgAzYCBCAEIAY2AgBBACEDCyABQRBqJAAgAwwBCyAGIAEoAgxB+JnAABCOAwALIgFFBEACfyMAQeAAayIBJAAgBCgCACEDIARBgICAgHg2AgAgA0GAgICAeEcEQCABIAQpAgQ3AiggASADNgIkIAEgBzYCCCABQTBqIAFBCGoQUQJAIAEtADBBBkYEQCABKAI0IQQgAUEkahDQAwwBCyABQdgAaiABQUBrKQMANwMAIAFB0ABqIAFBOGopAwA3AwAgASABKQMwNwNIIAFBCGoiAyAEQQxqIAFBJGogAUHIAGoQfyADEKoCQQAhBAsgAUHgAGokACAEDAELQfiXwABBK0GMmcAAEPcBAAsiAUUNAQsLIABBBjoAACAAIAE2AgQgAkEcahDO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ADYCbCACQgA3AmQgAkEANgJUIAJBADYCRCACQQA2AjAgAkEANgIgIAIgASgCBCIENgJcIAIgASgCACIBNgJYIAIgBDYCTCACIAE2AkggAiAAKAIEIgQ2AjggAiAAKAIAIgA2AjQgAiAENgIoIAIgADYCJCACIANBACABGzYCYCACIAFBAEciATYCUCACIAE2AkAgAiADQQAgABs2AjwgAiAAQQBHIgA2AiwgAiAANgIcIAJBEGogAkEcahB+AkAgAigCECIBRQ0AIAJBQGshBiACKAIUIQADQCACQQhqIAYQfiACKAIIIgVFDQEgAigCDCEDQQAhBCABKAIEIAEoAgggBSgCBCAFKAIIEPoCRQ0CIAAtAAAiASADLQAARw0CAkACQAJAAkACQAJAIAFBAWsOBQABAgMEBQsgAC0AASADLQABRg0EDAcLIABBCGogA0EIahCNAg0DDAYLIAAoAgggACgCDCADKAIIIAMoAgwQ+gINAgwFCyAAKAIIIAAoAgwgAygCCCADKAIMEIIBDQEMBAsgAEEEaiADQQRqEFlFDQMLIAIgAkEcahB+IAIoAgQhACACKAIAIgENAAsLQQEhBAsgAkHwAGokACAEC/kDAQJ/IAAgAWohAgJAAkAgACgCBCIDQQFxDQAgA0ECcUUNASAAKAIAIgMgAWohASAAIANrIgBBtJLBACgCAEYEQCACKAIEQQNxQQNHDQFBrJLBACABNgIAIAIgAigCBEF+cTYCBCAAIAFBAXI2AgQgAiABNgIADAILIAAgAxBjCwJAAkACQCACKAIEIgNBAnFFBEAgAkG4ksEAKAIARg0CIAJBtJLBACgCAEYNAyACIANBeHEiAhBjIAAgASACaiIBQQFyNgIEIAAgAWogATYCACAAQbSSwQAoAgBHDQFBrJLBACABNgIADwsgAiADQX5xNgIEIAAgAUEBcjYCBCAAIAFqIAE2AgALIAFBgAJPBEAgACABEGoPCyABQfgBcUGckMEAaiECAn9BpJLBACgCACIDQQEgAUEDdnQiAXFFBEBBpJLBACABIANyNgIAIAIMAQsgAigCCAshASACIAA2AgggASAANgIMIAAgAjYCDCAAIAE2AggPC0G4ksEAIAA2AgBBsJLBAEGwksEAKAIAIAFqIgE2AgAgACABQQFyNgIEIABBtJLBACgCAEcNAUGsksEA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BToAACAEKAIQIgBBhAFJDQEgABDYAQwBCyAAQYAoOwEACyAEQSBqJAALkwMBB38jAEHQAGsiAyQAIANBIGogAhD+AiADIAMpAyA3AiggA0EYaiADQShqEIcDIANBQGsgASADKAIYIAMoAhwQigMgAAJ/AkACQCADKAJARQRAIAMoAkQhAQwBCyADQThqIANByABqKAIANgIAIAMgAykCQDcDMCADKAIoIQIgAygCLCEJA0AgAiAJRg0CIAMgAjYCQCACQRhqIQIgA0EQaiEGIwBBEGsiASQAIANBMGoiBCgCCCEHIAFBCGogA0FAaygCACAEKAIAEE5BASEFIAEoAgwhCCABKAIIQQFxRQRAIARBBGogByAIEL8DIAQgB0EBajYCCEEAIQULIAYgCDYCBCAGIAU2AgAgAUEQaiQAIAMoAhBBAXFFDQALIAMoAhQhASADIAI2AiggAygCNCICQYQBSQ0AIAIQ2AELQQEMAQsgAyACNgIoIANByABqIANBOGooAgA2AgAgAyADKQMwNwNAIANBCGogA0FAaxCfAyADKAIMIQEgAygCCAs2AgAgACABNgIEIANB0ABqJAALmAICBH8BfiMAQSBrIgYkAAJAIAVFDQAgAiADaiIDIAJJDQAgBCAFakEBa0EAIARrca0gAyABKAIAIghBAXQiAiACIANJGyICQQhBBEEBIAVBgQhJGyAFQQFGGyIDIAIgA0sbIgOtfiIKQiCIpw0AIAqnIglBgICAgHggBGtLDQACfyAIRQRAIAZBGGohB0EADAELIAZBHGohByAGIAQ2AhggBiABKAIENgIUIAUgCGwLIQUgByAFNgIAIAZBCGogBCAJIAZBFGoQrAEgBigCCEEBRgRAIAYoAhAhAiAGKAIMIQcMAQsgBigCDCEEIAEgAzYCACABIAQ2AgRBgYCAgHghBwsgACACNgIEIAAgBzYCACAGQSBqJAALnQIBBH8jAEEQayICJAACQCABQYABTwRAIAJBDGoiBEECciEDIAJBADYCDAJAIAFBgBBPBEAgBEEDciEFIAFBgIAETwRAIAJBEGohAyACIAFBEnZB8AFyOgAMIAIgAUEGdkE/cUGAAXI6AA4gAiABQQx2QT9xQYABcjoADSAFIQQMAgsgAiABQQx2QeABcjoADCACIAFBBnZBP3FBgAFyOgANIAMhBCAFIQMMAQsgAkEMakEBciEEIAIgAUEGdkHAAXI6AAwLIAQgAUE/cUGAAXI6AAAgACACQQxqIAMQhQIMAQsgACgCCCIDIAAoAgBGBEAgAEGYsMAAEKkBCyAAKAIEIANqIAE6AAAgACADQQFqNgIICyACQRBqJABBAAudAgEEfyMAQRBrIgIkAAJAIAFBgAFPBEAgAkEMaiIEQQJyIQMgAkEANgIMAkAgAUGAEE8EQCAEQQNyIQUgAUGAgARPBEAgAkEQaiEDIAIgAUESdkHwAXI6AAwgAiABQQZ2QT9xQYABcjoADiACIAFBDHZBP3FBgAFyOgANIAUhBAwCCyACIAFBDHZB4AFyOgAMIAIgAUEGdkE/cUGAAXI6AA0gAyEEIAUhAwwBCyACQQxqQQFyIQQgAiABQQZ2QcABcjoADAsgBCABQT9xQYABcjoAACAAIAJBDGogAxCEAgwBCyAAKAIIIgMgACgCAEYEQCAAQfS2wAAQqQELIAAoAgQgA2ogAToAACAAIANBAWo2AggLIAJBEGokAEEAC50CAQR/IwBBEGsiAiQAAkAgAUGAAU8EQCACQQxqIgRBAnIhAyACQQA2AgwCQCABQYAQTwRAIARBA3IhBSABQYCABE8EQCACQRBqIQMgAiABQRJ2QfABcjoADCACIAFBBnZBP3FBgAFyOgAOIAIgAUEMdkE/cUGAAXI6AA0gBSEEDAILIAIgAUEMdkHgAXI6AAwgAiABQQZ2QT9xQYABcjoADSADIQQgBSEDDAELIAJBDGpBAXIhBCACIAFBBnZBwAFyOgAMCyAEIAFBP3FBgAFyOgAAIAAgAkEMaiADEIQCDAELIAAoAggiAyAAKAIARgRAIABB4LzAABCpAQsgACgCBCADaiABOgAAIAAgA0EBajYCCAsgAkEQaiQAQQALrgICA38BfiMAQUBqIgIkACACQQRqIAEQ4QECQCACKAIEQQFGBEAgAigCDCEBIAJBHGogAigCCBDAASACKAIcQYCAgIB4RgRAIAAgAigCIDYCBCAAQYGAgIB4NgIAIAFBhAFJDQIgARDYAQwCCyACQRhqIgMgAkEkaiIEKAIANgIAIAIgAikCHDcDECACQRxqIAEQwAEgAigCHEGAgICAeEYEQCAAIAIoAiA2AgQgAEGBgICAeDYCACACQRBqENADDAILIAJBPGogBCgCADYCACACQTBqIgEgAygCADYCACACIAIpAhw3AjQgACACKQMQIgU3AgAgAEEIaiABKQMANwIAIABBEGogAkE4aikDADcCACACIAU3AygMAQsgAEGAgICAeDYCAAsgAkFAayQAC68DAQd/IwBBQGoiAiQAIAJBEGogARD+AiACIAIpAxA3AhggAkEIaiACQRhqEIcDIAJBMGogAigCCCACKAIMEM8BAkAgAigCMEGAgICAeEYEQCAAIAIoAjQ2AgQgAEEGOgAADAELIAJBKGoiByACQThqKAIANgIAIAIgAikCMDcDIAJAIAIoAhgiASACKAIcIghGDQADQAJAIAIgATYCMCACQSBqIQQjAEEgayIDJAAgA0EIaiACQTBqEFECfyADLQAIQQZGBEAgAygCDAwBCyAEKAIIIgUgBCgCAEYEQCAEQZyZwAAQlAILIAQoAgQgBUEYbGoiBiADKQMINwMAIAZBEGogA0EYaikDADcDACAGQQhqIANBEGopAwA3AwAgBCAFQQFqNgIIQQALIANBIGokACIDDQAgCCABQRhqIgFHDQEMAgsLIAAgAzYCBCAAQQY6AAAgAiABQRhqNgIYIAJBIGoiABDCAiAAENMDDAELIAJBO2ogBygCADYAACAAQQQ6AAAgAiACKQMgNwAzIAAgAikAMDcAASAAQQhqIAJBN2opAAA3AAALIAJBQGskAAuMAgEDfyMAQYABayIEJAAgACgCACEAAn8CQCABKAIIIgJBgICAEHFFBEAgAkGAgIAgcQ0BIAAoAgBBASABEGIMAgsgACgCACEAQQAhAgNAIAIgBGpB/wBqIABBD3EiA0EwciADQdcAaiADQQpJGzoAACACQQFrIQIgAEEPSyAAQQR2IQANAAsgAUEBQeH5wABBAiACIARqQYABakEAIAJrEEMMAQsgACgCACEAQQAhAgNAIAIgBGpB/wBqIABBD3EiA0EwciADQTdqIANBCkkbOgAAIAJBAWshAiAAQQ9LIABBBHYhAA0ACyABQQFB4fnAAEECIAIgBGpBgAFqQQAgAmsQQwsgBEGAAWokAAuBAgIEfwF+IwBBIGsiBiQAAkAgBUUNACACIANqIgMgAkkNACAEIAVqQQFrQQAgBGtxrSADIAEoAgAiCEEBdCICIAIgA0kbIgJBCEEEQQEgBUGBCEkbIAVBAUYbIgMgAiADSxsiA61+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AgQgA0EoahBNIQIMAgsgBSgCBCIFKAIAIAUoAgQgAhDfAyECDAELIAUoAgQiBSgCACACIAUoAgQoAhARAQAhAgsgA0FAayQAIAIMAQtBAUEUQZzJwAAQjgMACwwXCyACQZS3wABBGBCbAwwWCyACQay3wABBGxCbAwwVCyACQce3wABBGhCbAwwUCyACQeG3wABBGRCbAwwTCyACQfq3wABBDBCbAwwSCyACQYa4wABBExCbAwwRCyACQZm4wABBExCbAwwQCyACQay4wABBDhCbAwwPCyACQbq4wABBDhCbAwwOCyACQci4wABBDBCbAwwNCyACQdS4wABBDhCbAwwMCyACQeK4wABBDhCbAwwLCyACQfC4wABBExCbAwwKCyACQYO5wABBGhCbAwwJCyACQZ25wABBPhCbAwwICyACQdu5wABBFBCbAwwHCyACQe+5wABBNBCbAwwGCyACQaO6wABBLBCbAwwFCyACQc+6wABBJBCbAwwECyACQfO6wABBDhCbAwwDCyACQYG7wABBExCbAwwCCyACQZS7wABBHBCbAwwBCyACQbC7wABBGBCbAwsEQEHItcAAQTcgBEEgakG4tcAAQcy2wAAQzQEACyAEQUBrIARBzABqKAIANgIAIAQgBCkCRDcDOCAEQTQ2AjQgBEE0NgIsIARBNTYCJCAEQQQ2AgwgBEHku8AANgIIIARCAzcCFCAEIABBEGo2AjAgBCAAQQxqNgIoIAQgBEE4aiIANgIgIAQgBEEgajYCECABKAIAIAEoAgQgBEEIahBNIABBAUEBENABIARB4ABqJAAL1gMCB38BfiMAQUBqIgEkAAJAIAAoAiAiAkUEQEEAIQAMAQsgACACQQFrNgIgAkAgABDxASIGBEAgAUEwaiAGQQhqIgcoAgA2AgAgASAGKQIANwMoIAFBFGohAyABQShqIgAoAgQhBAJAAkAgACgCCCIFIAAoAgAiAC8BkgNJBEAgACECDAELA0AgACgCiAIiAkUEQCADIAQ2AgggAyAANgIEIANBADYCAAwDCyAEQQFqIQQgAC8BkAMhBSAFIAIiAC8BkgNPDQALCyADIAU2AgggAyAENgIEIAMgAjYCAAsgASgCFCIADQFBuJ/AABDGAwALQcifwAAQxgMACyABIAEpAhg3AjggASAANgI0IAFBNGoiAigCCEEBaiED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QQxqIgIQrQMEQCAEQRBqIgEgAhCFAyAEQQA2AhwjAEFAaiICJAAgASgCAARAIAEoAggiAyABKAIEayIFQQAgAyAFTxshAwsgAkEQakGq1QIgAyADQarVAk8bQQhBGBCYASACKAIUIQMCQCACKAIQQQFHBEAgAkEANgIMIAIgAigCGDYCCCACIAM2AgQgAkEoaiABEJsBAkACQCACLQAoQQdHBEADQCACQSBqIAJBOGoiBikDADcDACACQRhqIAJBMGoiBykDADcDACACIAIpAygiCDcDECAIp0H/AXFBBkYNAiACKAIMIgMgAigCBEYEQCACQQRqQfSjwAAQlAILIAIoAgggA0EYbGoiBSACKQMoNwMAIAVBCGogBykDADcDACAFQRBqIAYpAwA3AwAgAiADQQFqNgIMIAJBKGogARCbASACLQAoQQdHDQALCyAAIAIoAiw2AgQgAEGAgICAeDYCACACQQRqIgAQwgIgABDTAwwBCyACQRBqEKoCIAAgAikCBDcCACAAQQhqIAJBDGooAgA2AgALIAJBQGskAAwBCyADIAIoAhhB5KPAABCOAwALDAELIARBEGogBEEMahB4IAQoAhAhAgJAAkACQCAELQAUIgNBAmsOAgEAAgsgAEGAgICAeDYCACAAIAI2AgQMAgsgBEEMaiAEQRBqQfiQwAAQVCEBIABBgICAgHg2AgAgACABNgIEDAELIwBB0ABrIgEkACABIANBAXE6ABAgASACNgIMIAFBIGpBAEEIQRgQmAEgASgCJCECAkAgASgCIEEBRwRAIAFBADYCHCABIAEoAig2AhggASACNgIUIAFBOGogAUEMahCTAQJAAkACQCABLQA4QQdHBEADQCABQTBqIAFByABqIgUpAwA3AwAgAUEoaiABQUBrIgYpAwA3AwAgASABKQM4Igg3AyAgCKdB/wFxQQZGDQIgASgCHCICIAEoAhRGBEAgAUEUakH0o8AAEJQCCyABKAIYIAJBGGxqIgMgASkDODcDACADQQhqIAYpAwA3AwAgA0EQaiAFKQMANwMAIAEgAkEBajYCHCABQThqIAFBDGoQkwEgAS0AOEEHRw0ACwsgACABKAI8NgIEIABBgICAgHg2AgAgAUEUaiIAEMICIAAQ0wMgASgCDCIAQYMBSw0BDAILIAFBIGoQqgIgACABKQIUNwIAIABBCGogAUEcaigCADYCACABKAIMIgBBhAFJDQELIAAQ2AELIAFB0ABqJAAMAQsgAiABKAIoQeSjwAAQjgMACwsgBCgCDCIAQYMBSwRAIAAQ2AELIARBIGokAAu6AQECfyMAQSBrIgMkAAJAAn9BACABIAEgAmoiAksNABpBAEEIIAIgACgCACIBQQF0IgQgAiAESxsiAiACQQhNGyIEQQBIDQAaQQAhAiADIAEEfyADIAE2AhwgAyAAKAIENgIUQQEFIAILNgIYIANBCGogBCADQRRqEKoBIAMoAghBAUcNASADKAIQIQAgAygCDAsgAEH418AAEI4DAAsgAygCDCEBIAAgBDYCACAAIAE2AgQgA0EgaiQAC9UBAQF/IAAoAgAiAEGcAmoQjwMgAEGoAmoQjwMgAEE4ahDGAiAAQfgBahDQAyAAQbQCahCPAyAAQcgBahCqAiAAQegAahDGAiAAQcACahCPAyAAQeABahCqAiAAQYQCaiIBELkCIAEQ1QMgAEGQAmoiARDCAiABENMDIAAoAuQCIgEEQCABEMUBIAFBsAFBCBC9AwsgAEHMAmoQjwMgAEGYAWoQxgIgAEHYAmoQjwMCQCAAQX9GDQAgACAAKAIEQQFrIgE2AgQgAQ0AIABB8AJBCBC9AwsLzAQCCn8BfiMAQRBrIgQkAAJAIAEoAiAiAkUEQCABKAIAIAFBADYCAEEBcQRAIAEoAgwhAiABKAIIIQUCQCABKAIEIgEEQCAEIAU2AgggBCABNgIEDAELIAIEQANAIAUoApgDIQUgAkEBayICDQALC0EAIQIgBEEANgIIIAQgBTYCBAsgBCACNgIMIwBBEGsiASQAIAFBBGogBEEEaiICKAIAIAIoAgQQiQIDQCABKAIEIgIEQCABQQRqIAIgASgCCBCJAgwBCwsgAUEQaiQACyAAQQA2AgAMAQsgASACQQFrNgIgIAEQ8QEiCARAIwBBMGsiAyQAIANBCGohBiMAQRBrIgkkACAIKAIEIQICQAJAIAgoAggiCiAIKAIAIgEvAZIDTwRAA0AgCUEEaiABIAIQiQIgCSgCBCIBRQ0CIAkoAgghAiAJKAIMIgogAS8BkgNPDQALCyAKQQFqIQcCQCACRQRAIAEhBQwBCyABIAdBAnRqQZgDaiEHIAIhCwNAIAcoAgAiBUGYA2ohByALQQFrIgsNAAtBACEHCyAGIAo2AhQgBiACNgIQIAYgATYCDCAGIAc2AgggBkEANgIEIAYgBTYCAAwBCyAGQQA2AgALIAlBEGokACADKAIIBEAgACADKQIUNwIAIANBKGogA0EQaigCACIBNgIAIABBCGogA0EcaigCADYCACADIAMpAggiDDcDICAIQQhqIAE2AgAgCCAMNwIAIANBMGokAAwCC0GQtcAAEMYDAAtBmL7AABDGAwALIARBEGokAAvsAQECfyMAQTBrIgIkAAJAIAApAwBC////////////AINCgICAgICAgPj/AFoEQCACQQE2AhQgAkHEwsAANgIQIAJCATcCHCACQcIANgIsIAIgADYCKCACIAJBKGo2AhggASgCACABKAIEIAJBEGoQTSEDDAELIAJBADoADCACIAE2AghBASEDIAJBATYCFCACQcTCwAA2AhAgAkIBNwIcIAJBwgA2AiwgAiAANgIoIAIgAkEoajYCGCACQQhqIAJBEGoQ1AMNACACLQAMRQRAIAFBzMLAAEECEJsDDQELQQAhAwsgAkEwaiQAIAMLsAEBB38gASgCACIFLwGSAyIJQQxsIQFBfyEDIAVBjAJqIQQgAigCCCEGIAIoAgQhBUEBIQgCQANAIAFFBEAgCSEDDAILIAQoAgghByAEKAIEIQIgA0EBaiEDIAFBDGshASAEQQxqIQQgBSACIAYgByAGIAdJGxCMAiICIAYgB2sgAhsiAkEASiACQQBIa0H/AXEiAkEBRg0ACyACDQBBACEICyAAIAM2AgQgACAINgIAC8UBAQJ/IwBBIGsiAiQAIAIgARCrAgJAAkAgAigCACIDQQJHBEAgAigCBCEBIANBAXFFDQEgAEEHOgAAIAAgATYCBAwCCyAAQQY6AAAMAQsgAkEIaiABED0gAi0ACCIBQQZHBEAgACACLwAJOwABIAAgAikDEDcDCCAAQQNqIAItAAs6AAAgAEEQaiACQRhqKQMANwMAIAAgAigCDDYCBCAAIAE6AAAMAQsgAigCDCEBIABBBzoAACAAIAE2AgQLIAJBIGokAAu2AQEDfyMAQSBrIgQkACAEIAI2AhwgBCABNgIYIARBEGogBEEYaiADEJIBIAQoAhQhBQJAIAQoAhBBAXFFDQADQCACRQRAQQEhBkEAIQIMAgsgASAFQQJ0aigCmAMhASAEIAJBAWsi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AEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEGwAmoQjwMgAEG4AmogA0EcaigCADYCACAAIAMpAhQ3ArACIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEG8AmoQjwMgAEHEAmogA0EcaigCADYCACAAIAMpAhQ3ArwCIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6YBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEHIAmoQjwMgAEHQAmogA0EcaigCADYCACAAIAMpAhQ3AsgCIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahCPAQsgA0EgaiQAC6MBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEEkahCPAyAAQSxqIANBHGooAgA2AgAgACADKQIUNwIkIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC6MBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEEwahCPAyAAQThqIANBHGooAgA2AgAgACADKQIUNwIwIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC6QBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEE8ahCPAyAAQcQAaiADQRxqKAIANgIAIAAgAykCFDcCPCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAulAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAAkAgAQRAIAMgASACEKACIANBFGogAygCACADKAIEEJMDDAELIANBgICAgHg2AhQLIABByABqEI8DIABB0ABqIANBHGooAgA2AgAgACADKQIUNwJIIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC6UBAQF/IwBBIGsiAyQAIANBCGogABD4ASADKAIIIQACQCABBEAgAyABIAIQoAIgA0EUaiADKAIAIAMoAgQQkwMMAQsgA0GAgICAeDYCFAsgAEHUAGoQjwMgAEHcAGogA0EcaigCADYCACAAIAMpAhQ3AlQgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqELcCCyADQSBqJAALpQEBAX8jAEEgayIDJAAgA0EIaiAAEPgBIAMoAgghAAJAIAEEQCADIAEgAhCgAiADQRRqIAMoAgAgAygCBBCTAwwBCyADQYCAgIB4NgIUCyAAQeAAahCPAyAAQegAaiADQRxqKAIANgIAIAAgAykCFDcCYCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAulAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAAkAgAQRAIAMgASACEKACIANBFGogAygCACADKAIEEJMDDAELIANBgICAgHg2AhQLIABB7ABqEI8DIABB9ABqIANBHGooAgA2AgAgACADKQIUNwJsIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahC3AgsgA0EgaiQAC5kBAQR/IwBBIGsiAiQAQQggACgCACIEQQF0IgMgA0EITRsiA0EASARAQQBBACABEI4DAAsgAiAEBH8gAiAENgIcIAIgACgCBDYCFEEBBSAFCzYCGCACQQhqIAMgAkEUahCqASACKAIIQQFGBEAgAigCDCACKAIQIAEQjgMACyACKAIMIQEgACADNgIAIAAgATYCBCACQSBqJAALiQEBAX8gAUEATgRAAn8gAigCBARAIAIoAggiAwRAIAIoAgAgA0EBIAEQlAMMAgsLQQEgAUUNABpB0Y7BAC0AABogAUEBEKoDCyICRQRAIAAgATYCCCAAQQE2AgQgAEEBNgIADwsgACABNgIIIAAgAjYCBCAAQQA2AgAPCyAAQQA2AgQgAEEBNgIAC8MBAQN/IwBBMGsiAiQAIAEoAiAiA0EBakEBdiADIAAoAgwbIgQgACgCCEsEQCACIAAgBCAAQRBqED4LIAJBIGogAUEYaikCADcDACACQRhqIAFBEGopAgA3AwAgAkEQaiABQQhqKQIANwMAIAIgAzYCKCACIAEpAgA3AwgjAEEwayIBJAAgAUEMaiIDIAJBCGpBJPwKAAAgAxCFASIDBEADQCAAIAMQVSABQQxqEIUBIgMNAAsLIAFBMGokACACQTBqJAALiAEBAX8gAAJ/An8CQCACQQBOBEAgAygCBARAIAMoAggiBARAIAMoAgAgBCABIAIQlAMMBAsLIAJFDQFB0Y7BAC0AABogAiABEKoDDAILIABBADYCBEEBDAILIAELIgNFBEAgACACNgIIIAAgATYCBEEBDAELIAAgAjYCCCAAIAM2AgRBAAs2AgALigEBA38jAEHwAmsiASQAIAFBuAFqIgIgABDoAQJ/IAEoArgBKALUAiIARQRAIAIQtQJBAAwBCyABQQhqIgIgABA/IAEoAqQBIQAgAUG4AWoiAxC1AkEAIABBgICAgHhGDQAaIAFBwAFqIAJBsAH8CgAAIAFBADYCuAEgAxClAkEIagsgAUHwAmokAAuvAQIBfwFvIwBBEGsiAiQAAkAgASgCACUBEBoEQCACQQRqIAEQ3QEgAEEIaiACQQxqKAIANgIAIAAgAikCBDcCAAwBCyABKAIAJQEQEARAIAEoAgAlARAZIQMQfCIBIAMmASACIAE2AgAgAkEEaiACEN0BIABBCGogAkEMaigCADYCACAAIAIpAgQ3AgAgAUGEAUkNASABENgBDAELIABBgICAgHg2AgALIAJBEGokAAuIAQECfyMAQUBqIgIkAAJAIAEEQCABQQhrIgMoAgBBAUcNASACQQhqIAFBOPwKAAAgA0EANgIAAkAgA0F/Rg0AIAFBBGsiASABKAIAQQFrIgE2AgAgAQ0AIANBwABBCBC9AwsgACACQRBqQTD8CgAAIAJBQGskAA8LENYDAAtBzoPAAEE/ENgDAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABB6AFqENADIABB8AFqIANBHGooAgA2AgAgACADKQIUNwLoASADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQjwELIANBIGokAAuMAQECfyMAQcABayICJAACQCABBEAgAUEIayIDKAIAQQFHDQEgAkEIaiABQbgB/AoAACADQQA2AgACQCADQX9GDQAgAUEEayIBIAEoAgBBAWsiATYCACABDQAgA0HAAUEIEL0DCyAAIAJBEGpBsAH8CgAAIAJBwAFqJAAPCxDWAwALQc6DwABBPxDYAwALkAEBAX8jAEEgayIDJAAgA0EIaiAAEPgBIAMoAgghACADIAEgAhCgAiADQRRqIAMoAgAgAygCBBCTAyAAQfgAahDQAyAAQYABaiADQRxqKAIANgIAIAAgAykCFDcCeCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABBhAFqENADIABBjAFqIANBHGooAgA2AgAgACADKQIUNwKEASADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABBkAFqENADIABBmAFqIANBHGooAgA2AgAgACADKQIUNwKQASADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQtwILIANBIGokAAuRAQEBfyMAQSBrIgMkACADQQhqIAAQ+AEgAygCCCEAIAMgASACEKACIANBFGogAygCACADKAIEEJMDIABBnAFqENADIABBpAFqIANBHGooAgA2AgAgACAD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SSABEI8DIAJB2ABqIgUgAkGQAWopAwA3AwAgAkHQAGoiByACQYgBaikDADcDACACQcgAaiADKQMANwMAIAIgAikDeDcDQCABQQJBAUEBEJgBIAIoAqgBIQEgAigCpAFBAUcNAgwGCyACKAIQIQMgAiACKAIUQQAgARs2AmAgAiADNgJcIAIgATYCWCACQQA2AlQgAiABQQBHIgU2AlAgAiADNgJMIAIgATYCSCACQQA2AkQgAiAFNgJAIAJBIGohByACQUBrIQUjAEEgayIDJABB4JLBAAJ+QdiSwQAoAgBBAUYEQEHoksEAKQMAIRJB4JLBACkDAAwBCyADEIECQdiSwQBCATcDAEHoksEAIAMpAwgiEjcDACADKQMACyITQgF8NwMAIANBCGoiDUGYmsAAKQMANwMAIANBGGoiDiASNwMAIANBEGoiDyATNwMAIANBkJrAACkDADcDACMAQYABayIBJAAgBSgCICEGIAFBKGogBUEYaikCADcDACABQSBqIAVBEGopAgA3AwAgAUEYaiAFQQhqKQIANwMAIAEgBjYCMCABIAUpAgA3AxAgAUHQAGogAUEQahCQAQJAIAEoAlAiBkUNACABQdwAaiEQIAFBOGohBQNAIAUgBiABKAJYIgpBGGxqIggpAgA3AgAgBUEIaiILIAhBCGopAgA3AgAgBUEQaiIMIAhBEGopAgA3AgAgBiAKQQxsaiIGKAKMAiIIQYCAgIB4Rg0BIAYpApACIRIgAUHgAGogDCkCADcDACABQdgAaiIRIAspAgA3AwAgASAFKQIANwNQIAFBCGogAUHQAGoQ9wIgASgCDCEGIAEoAgghCiABIBI3AmwgASAINgJoAkACQCAKBEAgAUH0AGogBkEBQQEQmAEgASgCeCELIAEoAnRBAUcNASALIAEoAnxB4KLAABCOAwALIAFB6ABqENADIAFB0ABqEKgCDAELIAEoAnwhDCAGBEAgDCAKIAb8CgAACyABQdAAahCoAiABIBI3AlQgASAINgJQIAEgBjYCZCABIAw2AmAgASALNgJcIAFB8ABqIBEoAgA2AgAgASABKQJQNwNoIAFB9ABqIgYgAyABQegAaiAQEEkgASgCdEGAgICAeEYNACAGENADCyABQdAAaiABQRBqEJABIAEoAlAiBg0ACwsgAUEQahDjASABQYABaiQAIAdB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ADYAACAAQQU6AAAgAiABKQAMNwAEIAAgAikAATcAASAAQQhqIAJBCGopAAA3AAAgASgCAEGAgICAeEcEQCABQQFBARDQAQsgAkEQaiQAC2IBAn8CQCAAKAIAIgFBAUcNACAAKAIEDQAgACgCCCEBIAAoAgwiAgRAA0AgASgCmAMhASACQQFrIgINAAsLIABCADcCCCAAIAE2AgRBASEBIABBATYCAAsgAEEEakEAIAEbC2MCAX8BbyMAQRBrIgMkACABKAIAJQEgAigCACUBEBEhBBB8IgEgBCYBIANBCGoQzwJBASECIAACfyADKAIIQQFxBEAgAygCDAwBC0EAIQIgAQs2AgQgACACNgIAIANBEGokAAtjAgF/AW8jAEEQayIDJAAgASgCACUBIAIoAgAlARAbIQQQfCIBIAQmASADQQhqEM8CQQEhAiAAAn8gAygCCEEBcQRAIAMoAgwMAQtBACECIAELNgIEIAAgAjYCACADQRBqJAALXgEBfyMAQbABayICJAACQAJAIAFFBEAgAiAAELEBIAIQxQEMAQsgAEUNASACIABBCGsiADYCACAAIAAoAgBBAWsiADYCACAADQAgAhC3AgsgAkGwAWokAA8LENYDAAtdAQF/IwBBMGsiAiQAIAIgATYCDCACIAA2AgggAkECNgIUIAJB3JzAADYCECACQgE3AhwgAkEZNgIsIAIgAkEoajYCGCACIAJBCGo2AiggAkEQahCHASACQTBqJAALXQEBfyMAQTBrIgIkACACIAE2AgwgAiAANgIIIAJBAjYCFCACQYCdwAA2AhAgAkIBNwIcIAJBGTYCLCACIAJBKGo2AhggAiACQQhqNgIoIAJBEGoQhwEgAkEwaiQAC1sBAX8jAEEwayIDJAAgAyABNgIMIAMgADYCCCADQQE2AhQgA0HY9sAANgIQIANCATcCHCADIANBCGqtQoCAgICwDIQ3AyggAyADQShqNgIYIANBEGogAhDFAgALVwECfwJAAkAgAQRAIAFBCGsiAiACKAIAQQFqIgM2AgAgA0UNASABKAIADQIgACACNgIIIAAgATYCBCABQX82AgAgACABQQhqNgIADwsQ1gMLAAsQ1wMAC1gBAX8jAEEQayICJAAgAkEEaiAAEPgBIAIoAgQgAUEARzoAqAEgAigCCEEANgIAIAIoAgwiACAAKAIA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ADYCBCAAIAE2AgALPQECfyMAQRBrIgEkAEEBIQIgAC0AAARAIAFBCGogABD3AiABKAIMRSABKAIIQQBHcSECCyABQRBqJAAgAgtFAQF/IwBBIGsiAyQAIAMgAjYCHCADIAE2AhggAyACNgIUIANBCGogA0EUakGsxMAAEN8BIAAgAykDCDcDACADQSBqJAALOwEBfyMAQRBrIgIkACACIAEoAgAlARApIAAgAigCAAR+IAAgAisDCDkDCEIBBUIACzcDACACQRBqJAALOwEBfyMAQRBrIgIkACACIAEoAgAlARAsIAAgAigCAAR+IAAgAikDCDcDCEIBBUIACzcDACACQRBqJAALnnQDI38afgF8IAEoAggiA0GAgIABcSECIAArAwAhPyADQYCAgIABcUUEQCABIAJBAEchAUEAIQAjAEGAAWsiByQAID+9ISUCf0EDID+ZRAAAAAAAAPB/YQ0AGkECICVCgICAgICAgPj/AIMiJkKAgICAgICA+P8AUQ0AGiAlQv////////8HgyIpQoCAgICAgIAIhCAlQgGGQv7///////8PgyAlQjSIp0H/D3EiABsiJ0IBgyEoICZQBEBBBCApUA0BGiAAQbMIayEAQgEhJiAoUAwBC0KAgICAgICAICAnQgGGICdCgICAgICAgAhRIgIbISdCAkIBIAIbISZBy3dBzHcgAhsgAGohACAoUAshAiAHIAA7AXggByAmNwNwIAdCATcDaCAHICc3A2AgByACOgB6An8CQAJAAkAgAkECayICBEBBASEAQYP2wABBhPbAACAlQgBTIgMbQYP2wABBASADGyABGyEXICVCP4inIAFyIRtBAyACIAJBA08bQQJrDgIDAgELIAdBAzYCKCAHQYX2wAA2AiQgB0ECOwEgQQEhF0EBIQAgB0EgagwDCyAHQQM2AiggB0GI9sAANgIkIAdBAjsBICAHQSBqDAILIAdBIGohBiAHQQ9qIQwjAEEwayIDJAACQAJAAn8CQAJAAkACQAJAAkACQAJAIAdB4ABqIgApAwAiJVBFBEAgACkDCCInUA0BIAApAxAiJlANAiAlICZ8IiYgJVQNAyAlICdUDQQgJkKAgICAgICAgCBaDQUgAyAALwEYIgA7AQggAyAlICd9Iic3AwAgACAAQSBrIAAgJkKAgICAEFQiARsiAkEQayACICZC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ASkDoAM3AkwgAkEwaiABQbgDaikDADcDACACQThqIAFBwANqKQMANwMAIAJBQGsgAUHIA2opAwA3AwAgAkHUAGogAUGoA2ooAgA2AgAgASgC5AIhBCABKQLoAiEpIAIgKzcDqAIgAkGAgICAeCAQIBBBgYCAgHhGGzYCpAIgAkEAIBYgEUGAgICAeEYiCRs2AvABIAJBASAUIAkbNgLsASACQQAgESAJGzYC6AEgAiADOgC4ASACIAEvAeQDOwC5ASACQbsBaiABQeYDai0AADoAACACIAEpA9ADNwK8ASACQcQBaiABQdgDaikDADcCACACQcwBaiABQeADaigCADYCACACIAEpA/gDNwNYIAJB4ABqIAFBgARqKQMANwMAIAJB6ABqIAFBiARqKQMANwMAIAJB8ABqIAFBkARqKQMANwMAIAIgCjYCeCACICw3ArQCIAJBgICAgHggBiAYGzYCsAIgAiALOgDQASACQYQBaiABQfADaigCADYCACACIAEpA+gDNwJ8IAIgAS8BrAQ7ANEBIAJB0wFqIAFBrgRqLQAAOgAAIAJB5AFqIAFBqARqKAIANgIAIAJB3AFqIAFBoARqKQMANwIAIAIgASkDmAQ3AtQBIAIgLTcDwAIgAiAVNgK8AiACIAc2AogCIAIgGzYChAIgAiAaNgKAAiACIBM2AvwBIAIgEjYC+AEgAiAINgL0ASACQaABaiABQYgDaikDADcDACACQZgBaiABQYADaikDADcDACACQZABaiABQfgCaikDADcDACACIAEpA/ACNwOIASACIAU2AqgBIAJBtAFqIAFBuARqKAIANgIAIAIgASkDsAQ3AqwBIAJBAiAhICFBA0YbOgDZAiACQQIgICAgQQNGGzoA2AIgAiAZNgLUAiACICk3AswCIAJBgICAgHggBCAEQYGAgIB4Rhs2AsgCIAIgJDYCJCACICNBACAjQQJHGzYCICACICU2AhwgAiAiQQAgIkECRxs2AhggAiAmNgIUIAIgH0EAIB9BAkcbNgIQIAIgJzYCDCACIB5BACAeQQJHGzYCCCACICg2AgQgAiAdQQAgHUECRxs2AgAMGgsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEMsBIAEoAvQCIQMgASgC8AIiBkGBgICAeEYEQCACQQI2AgAgAiADNgIEDBoLIAEoAvgCIQcgASgC5AJBgYCAgHhHBEAgAUHkAmoQjwMLIAEgBzYC7AIgASADNgLoAiABIAY2AuQCDBYLDBsLIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDTASABKALwAiEDIAEoApADIgZBgYCAgHhGBEAgAkECNgIAIAIgAzYCBAwZCyABQZAEaiIHIARBGGooAgA2AgAgAUGIBGoiCCAEQRBqKQIANwMAIAFBgARqIhIgBEEIaikCADcDACABQbgDaiITIApBCGooAgA2AgAgASAEKQIANwP4AyABIAopAgA3A7ADIAEoAtACQYGAgIB4RwRAIAFBsAJqEMYCCyAWIAEpA/gDNwIAIBwgASkDsAM3AgAgFkEIaiASKQMANwIAIBZBEGogCCkDADcCACAWQRhqIAcoAgA2AgAgHEEIaiATKAIANgIAIAEgAzYCsAIgASAGNgLQAgwVCwwaCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQywEgASgC9AIhAyABKALwAiIGQYGAgIB4RgRAIAJBAjYCACACIAM2AgQMGAsgASgC+AIhByABKAKkAkGBgICAeEcEQCABQaQCahCPAwsgASAHNgKsAiABIAM2AqgCIAEgBjYCpAIMFAsMGQsgASgCHCABQQA2AhwEQCABKAIgIQZBACEFIwBBEGsiAyQAIAMgBjYCDCABQQhqIgcCfyADQQxqEMQCRQRAIwBBsAFrIgUkACAFIAYQNQJAAkACfyAFKAKcAUGAgICAeEYEQCAFKAIAIQZBAQwBC0HRjsEALQAAGkGwAUEIEKoDIgZFDQEgBiAFQbAB/AoAAEEACyEIIAMgBjYCBCADIAg2AgAgBUGwAWokAAwBC0EIQbABEN0DAAsgAygCACEFIAMoAgQMAQtBACAGQYQBSQ0AGiAGENgBQQALNgIEIAcgBTYCACADQRBqJAAgASgCDCEFIAEoAghBAXEEQCACQQI2AgAgAiAFNgIEQQAhBQwXCyABIAU2AqACQQEhBSABQQE2ApwCDBMLDBgLIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBCNASABKAL0AiEDIAEoAvACIgZBgICAgHhGBEAgAkECNgIAIAIgAzYCBAwWCyABKAL4AiEHIAEoApACQYCAgIB4RwRAIAFBkAJqIggQwgIgCBDTAwsgASAHNgKYAiABIAM2ApQCIAEgBjYCkAIMEgsMFwsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIwBIAEoAvQCIQMgASgC8AIiBkGAgICAeEYEQCACQQI2AgAgAiADNgIEDBULIAEoAvgCIQcgASgChAJBgICAgHhHBEAgAUGEAmoiCBC5AiAIENUDCyABIAc2AowCIAEgAzYCiAIgASAGNgKEAgwRCwwWCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQuAEgAS0A8AIiA0EHRgRAIAEoAvQCIQQgAkECNgIAIAIgBDYCBAwUCyABQbIDaiIGIBBBAmotAAA6AAAgAUGABGoiByAPQQhqKQMANwMAIAEgEC8AADsBsAMgASAPKQMANwP4AyABKAL0AiEIIAEtAOgBQQZxQQZHBEAgAUHoAWoQqAILIBUgAS8BsAM7AAAgGyABKQP4AzcDACAVQQJqIAYtAAA6AAAgG0EIaiAHKQMANwMAIAEgAzoA6AEgASAINgLsAQwQCwwVCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQywEgASgC9AIhAyABKALwAiIGQYGAgIB4RgRAIAJBAjYCACACIAM2AgQMEwsgASgC+AIhByABKALcAUGBgICAeEcEQCABQdwBahCPAwsgASAHNgLkASABIAM2AuABIAEgBjYC3AEMDwsMFAsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCISQgASgC8AIiI0ECRw0OIAJBAjYCACACICQ2AgQMEQsMEwsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCISUgASgC8AIiIkECRw0NIAJBAjYCACACICU2AgQMEAsMEgsgASgCHCABQQA2AhwEQCABQfACaiABKAIgENMBIAEoAvACIQMgASgCkAMiBkGBgICAeEYEQCACQQI2AgAgAiADNgIEDBALIAFBkARqIgcgBEEYaigCADYCACABQYgEaiIIIARBEGopAgA3AwAgAUGABGoiEiAEQQhqKQIANwMAIAFBuANqIhMgCkEIaigCADYCACABIAQpAgA3A/gDIAEgCikCADcDsAMgASgCyAFBgYCAgHhHBEAgAUGoAWoQxgILIBEgASkD+AM3AgAgFCABKQOwAzcCACARQQhqIBIpAwA3AgAgEUEQaiAIKQMANwIAIBFBGGogBygCADYCACAUQQhqIBMoAgA2AgAgASADNgKoASABIAY2AsgBDAwLDBELIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBC4ASABLQDwAiIDQQdGBEAgASgC9AIhBCACQQI2AgAgAiAENgIEDA8LIAFBsgNqIgYgEEECai0AADoAACABQYAEaiIHIA9BCGopAwA3AwAgASAQLwAAOwGwAyABIA8pAwA3A/gDIAEoAvQCIQggAS0AkAFBBnFBBkcEQCABQZABahCoAgsgDiABLwGwAzsAACAaIAEpA/gDNwMAIA5BAmogBi0AADoAACAaQQhqIAcpAwA3AwAgASADOgCQASABIAg2ApQBDAsLDBALIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDSASABLQDwAgRAIAEoAvQCIQQgAkECNgIAIAIgBDYCBAwOCyABLQDxAiEhDAoLDA8LIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDSASABLQDwAgRAIAEoAvQCIQQgAkECNgIAIAIgBDYCBAwNCyABLQDxAiEgDAkLDA4LIAEoAhwgAUEANgIcBEAgAUHwAmogASgCIBDLASABKAL0AiEDIAEoAvACIgZBgYCAgHhGBEAgAkECNgIAIAIgAzYCBAwMCyABKAL4AiEHIAEoAoQBQYGAgIB4RwRAIAFBhAFqEI8DCyABIAc2AowBIAEgAzYCiAEgASAGNgKEAQwICwwNCyABKAIcIAFBADYCHARAIAFB8AJqIAEoAiAQwAEgASgC9AIhAyABKALwAiIGQYCAgIB4RgRAIAJBAjYCACACIAM2AgQMCwsgASgC+AIhByABQfgAahCPAyABIAc2AoABIAEgAzYCfCABIAY2AngMBwsMDAsgASgCHCABQQA2AhwEQCABQfACaiABKAIgEIMBIAEoAvQCISYgASgC8AIiH0ECRw0GIAJBAjYCACACICY2AgQMCQsMCwsgASgCHCABQQA2AhwEQCABQfACaiABKAIgENMBIAEoAvACIQMgASgCkAMiBkGBgICAeEYEQCACQQI2AgAgAiADNgIEDAkLIAFBkARqIgcgBEEYaigCADYCACABQYgEaiIIIARBEGopAgA3AwAgAUGABGoiEiAEQQhqKQIANwMAIAFBuANqIhMgCkEIaigCADYCACABIAQpAgA3A/gDIAEg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IAIgADYCBCABQTBqJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCjAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBjAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCmAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBmAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCpAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBpAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCsAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBsAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCvAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABBvAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv9AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCyAJBgICAgHhGBEAgAxC1AkEAIQBBAAwBCyABQSRqIABByAJqENwBIAEoAiQhAyABQTBqIgQQtQJBACEAQQAgA0GAgICAeEYNABogAUE4aiABQSxqKAIANgIAIAEgASkCJDcDMCABQRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv7AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCJEGAgICAeEYEQCADELQCQQAhAEEADAELIAFBJGogAEEkahDcASABKAIkIQMgAUEwaiIEELQCQQAhAEEAIANBgICAgHhGDQAaIAFBOGogAUEsaigCADYCACABIAEpAiQ3AzAgAUEYaiAEQYiBwAAQ3wEgAUEQaiABKAIYIAEoAhwQqwMgASgCECEAIAEoAhQLIQMgAUEIaiAAIAMQqwMgASgCDCEAIAIgASgCCDYCACACIAA2AgQgAUFAayQAIAIo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QRhqIARBiIHAABDfASABQRBqIAEoAhggASgCHBCrAyABKAIQIQAgASgCFAshAyABQQhqIAAgAxCrAyABKAIMIQAgAiABKAIINgIAIAIgADYCBCABQUBrJAAgAigCACACKAIEIAJBEGokAAv8AQEEfyMAQRBrIgIkACMAQUBqIgEkACABQTBqIgMgABDoAQJ/IAEoAjAiACgCbEGAgICAeEYEQCADELQCQQAhAEEADAELIAFBJGogAEHsAGoQ3AEgASgCJCEDIAFBMGoiBBC0AkEAIQBBACADQYCAgIB4Rg0AGiABQThqIAFBLGooAgA2AgAgASABKQIkNwMwIAFBGGogBEGIgcAAEN8BIAFBEGogASgCGCABKAIcEKsDIAEoAhAhACABKAIUCyEDIAFBCGogACADEKsDIAEoAgwhACACIAEoAgg2AgAgAiAANgIEIAFBQGskACACKAIAIAIoAgQgAkEQaiQACyUAIABFBEBBkK3AAEEyENgDAAsgACACIAMgBCAFIAEoAhARGwALHwECfiAAKQMAIgIgAkI/hyIDhSADfSACQgBZIAEQYQvIGAIMfwJ+EHwiBCAAJgEQfCIFIAEmASMAQYABayIDJAAgAyAFNgIEIAMgBDYCACADQQhqIAMQUwJAAkACQCADKAIIQYCAgIB4RwRAIANBKGogA0EQaigCADYCACADIAMpAgg3AyAgA0EBNgJsIANBqI/AADYCaCADQgE3AnQgA0EDNgJYIAMgA0HUAGo2AnAgAyADQSBqNgJUIANBOGoiBSADQegAaiICEF4gAygCPCADKAJAEOcDIQQgBRDQAyADIAQ2AmggAhDlAyAEQYQBTwRAIAQQ2AELIANBIGoQ0AMMAQsgAygCDCEEIANBCGogA0EEahBTIAMoAghBgICAgHhHBEAgA0EoaiADQRBqKAIANgIAIAMgAykCCDcDICADQQE2AmwgA0GEj8AANgJoIANCATcCdCADQQM2AlggAyADQdQAajYCcCADIANBIGo2AlQgA0E4aiICIANB6ABqIgYQXiADKAI8IAMoAkAQ5wMhBSACENADIAMgBTYCaCAGEOUDIAVBhAFPBEAgBRDYAQsgA0EgahDQAyAEQYQBSQ0BIAQQ2AEMAQsgAygCDCEFIANBOGogBBA9IAMtADhBBkYEQCADIAMoAjw2AmAgA0EBNgJsIANB4I7AADYCaCADQgE3AnQgA0EBNgJYIAMgA0HUAGo2AnAgAyADQeAAajYCVCADQSBqIgIgA0HoAGoiBhBeIAMoAiQgAygCKBDnAyEEIAIQ0AMgAyAENgJoIAYQ5QMgBEGEAU8EQCAEENgBCyADKAJgIgRBhAFPBEAgBBDYAQsgBUGEAUkNASAFENgBDAELIANBGGogA0HIAGoiBCkDADcDACADQRBqIANBQGsiAikDADcDACADIAMpAzg3AwggA0E4aiAFED0gAy0AOEEGRgRAIAMgAygCPDYCUCADQQE2AmwgA0G4jsAANgJoIANCATcCdCADQQE2AmQgAyADQeAAajYCcCADIANB0ABqNgJgIANB1ABqIgUgA0HoAGoiAhBeIAMoAlggAygCXBDnAyEEIAUQ0AMgAyAENgJoIAIQ5QMgBEGEAU8EQCAEENgBCyADKAJQIgRBhAFPBEAgBBDYAQsgA0EIahCoAgwBCyADQTBqIAQpAwA3AwAgA0EoaiACKQMANwMAIAMgAykDODcDICADQQhqIQojAEHQAWsiAiQAIAJBBDYCTCACQfGCwAA2AkggAkEHNgJEIAJB6oLAADYCQCACQQw2AjwgAkHegsAANgI4IAJBCDYCNCACQdaCwAA2AjAgAkEGNgIsIAJB0ILAADYCKCACQdAAaiEEIwBBEGsiBSQAAn5B2JLBACgCAEEBRgRAQeiSwQApAwAhDkHgksEAKQMADAELIAUQgQJB2JLBAEIBNwMAQeiSwQAgBSkDCCIONwMAIAUpAwALIQ8gBCAONwMYIAQgDzcDEEHgksEAIA9CAXw3AwAgBEGQmsAAKQMANwMAIARBCGpBmJrAACkDADcDACAFQRBqJAAgCi0AAEEFRgRAIAooAgghBiACIAooAgxBACAKKAIEIgUbNgLIASACIAY2AsQBIAIgBTYCwAEgAkEANgK8ASACIAVBAEciBzYCuAEgAiAGNgK0ASACIAU2ArABIAJBADYCrAEgAiAHNgKoASACIAJBKGo2AswBIAJBqAFqIAQQ2gELIANBIGoiDC0AAEEFRgRAIAwoAgghBSACIAwoAgxBACAMKAIEIgQbNgLIASACIAU2AsQBIAIgBDYCwAEgAkEANgK8ASACIARBAEciBjYCuAEgAiAFNgK0ASACIAQ2ArABIAJBADYCrAEgAiAGNgKoASACIAJBKGo2AswBIAJBqAFqIAJB0ABqENoBCyADQegAaiELAkAgAigCXCIFBEAgAigCUCIJQQhqIQYgCSkDAEJ/hUKAgYKEiJCgwIB/gyEOA0AgDlAEQANAIAlBIGshCSAGKQMAIAZBCGohBkKAgYKEiJCgwIB/gyIOQoCBgoSIkKDAgH9RDQALIA5CgIGChIiQoMCAf4UhDgsgCSAOeqdBAXZBPHFrQQRrIggoAgAiBCgCBCAEKAIIIAoQ1wIhByAIKAIAIgQoAgQgBCgCCCAMENcCIQQCQAJAAkACQAJAAkAgCCgCACIIKAIEIAgoAghByIPAAEEGEPoCRQ0AAkAgB0UNACAHLQAAIghFDQAgCEEFRw0CIAcoAgwNAgsgBEUNBSAELQAAIghFDQUgCEEFRw0AIAQoAgxFDQULIAdFDQELIARFDQEgByAEEDwNAwwCCyAERQ0CIAQtAAANAQwCCyAHLQAARQ0BCyALQQE7AQAMAwsgDkIBfSAOgyEOIAVBAWsiBQ0ACwsgAkH0AGogCkHWgsAAQQgQmgEgAkGAAWogDEHWgsAAQQgQmgFB3oLAAEEMIAoQ1wIhBkHegsAAQQwgDBDXAiEHIAJBjAFqIApB0ILAAEEGEJoBIAJBmAFqIAxB0ILAAEEGEJoBAkACQAJAAkACQCACKAJ8IgRFDQAgAigCiAEiBUUNACACKAJ4IAQgAigChAEgBRD6Ag0AIAJBqAFqQn0QiwMgAi0AqAFBBkYNAQwECyAGRSAHRXIiCQ0CIAYQnwINAiAHEJ8CDQIgAkEgaiAGEPcCIAIoAiQgAigCICEEIAJBGGogBxD3AkEAIAQbIghFDQIgAigCHEEAIAIoAhgiBRsiDUUNAiAEQQEgBBsgCCAFQQEgBRsgDRD6Ag0CIAJBqAFqQn4QiwMgAi0AqAFBBkYNAQwDCyACIAIoAqwBNgKkAUGogcAAQSsgAkGkAWpBmIHAAEH4gsAAEM0BAAsgAiACKAKsATYCpAFBqIHAAEErIAJBpAFqQZiBwABBiIPAABDNAQALAkACQAJAAkACQAJAIAIoApQBIgRFDQAgAigCoAEiBUUNACACKAKQASAEIAIoApwBIAUQ+gJFDQELIAIoAogBIQUgAigCfCIIDQEgBSEEDAMLIAJBqAFqQn8QiwMgAi0AqAFBBkYNAQwEC0EAIQQgBUUNAQwCCyACIAIoAqwBNgKkAUGogcAAQSsgAkGkAWpBmIHAAEGYg8AAEM0BAAsgAigCeCAIIAIoAoQBIAQQ+gJBAXMhBAsCQCAJBEAgBiAHckUhBQwBCwJAAkAgBhCfAg0AIAcQnwINACACQRBqIAYQ9wIgAigCFCACKAIQIQUgAkEIaiAHEPcCQQAgBRsiB0EAIAIoAgxBACACKAIIIgYbIgkbDQEgBUEBIAUbIAcgBkEBIAYbIAkQ+gIhBQwCCyAGEJ8CBEAgBxCfAg0BCyAGLQAAIgkgBy0AAEcEQEEAIQUMAgtBASEFAkACQAJAAkACQCAJQQFrDgUAAQIDBAYLIAYtAAEgBy0AAUYhBQwFCyAGQQhqIAdBCGoQjQIhBQwECyAGKAIIIAYoAgwgBygCCCAHKAIMEPoCIQUMAwsgBigCCCAGKAIMIAcoAgggBygCDBCCASEFDAILIAZBBGogB0EEahBZIQUMAQtBASEFCyACKAKgASEGAkACQAJAAkACQCACKAKUASIHBEAgBg0BQQAhBgsgBCACKAKQASAHIAIoApwBIAYQ+gIgBXFBAXNyDQEMAgsgBCAFQQFzckEBRw0BCyACQagBakIBEIsDIAItAKgBQQZHDQEgAiACKAKsATYCpAFBqIHAAEErIAJBpAFqQZiBwABBqIPAABDNAQALIAJBqAFqQgIQiwMgAi0AqAFBBkYNAQsgCyACKQOoATcDACALQRBqIAJBuAFqKQMANwMAIAtBCGogAkGwAWopAwA3AwAgAkGYAWoQ0AMgAkGMAWoQ0AMgAkGAAWoQ0AMgAkH0AGoQ0AMMAgsgAiACKAKsATYCpAFBqIHAAEErIAJBpAFqQZiBwABBuIPAABDNAQALIAsgAikDqAE3AwAgC0EQaiACQbgBaikDADcDACALQQhqIAJBsAFqKQMANwMAIAJBmAFqENADIAJBjAFqENADIAJBgAFqENADIAJB9ABqENADCyACQdAAahCOAiACQdABaiQAIAsQTCEEIAsQqAIgDBCoAiAKEKgCIAMoAgQiBUGEAU8EQCAFENgBCyADKAIAIgVBhAFJDQIMAQsgAygCBCIEQYQBTwRAIAQQ2AELQYEBIQQgAygCACIFQYMBTQ0BCyAFENgBCyADQYABaiQAIAQlASAEENgBCx4AIAAoAgBBgICAgHhHBEAgABDQAyAAQRBqEKgCCwsaACAAKAIAKAIAIAEoAgAgAkFYbGpBKGsQVwsaACAAKAIAKAIAIAEoAgAgAkFobGpBGGsQVwsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBELAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBEIAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBEvAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBExAAsjACAARQRAQZCtwABBMhDYAwALIAAgAiADIAQgASgCEBEzAAshACAAIAEoAgw2AgQgACABKAIIQQAgAS0AAEEDRhs2AgALJgEBf0HRjsEALQAAGkGYA0EIEKoDIgBFBEBBCEGYAxDdAwALIAALJgEBf0HRjsEALQAAGkHIA0EIEKoDIgBFBEBBCEHIAxDdAwALIAALGQEBfyABIANGBH8gACACIAEQjAJFBSAECwsoAQF/IAAoAgAiAUGAgICAeHJBgICAgHhHBEAgACgCBCABQQEQvQMLCyEAIABFBEBBkK3AAEEyENgDAAsgACACIAMgASgCEBEDAAslAQF/EPgCIgFBADsBkgMgAUEANgKIAiAAQQA2AgQgACABNgIACyABAX8gACABKAIEIgI2AgAgACACIAEoAghBGGxqNgIECyIAIAAtAABFBEAgAUGG/MAAQQUQSg8LIAFBi/zAAEEEEEoLGwAgACgCACIABEAgABDFASAAQbABQQgQvQMLCxoAIAAoAgAoAgAgASgCACACQQJ0a0EEaxBWCxoAIAAgASACEMMBIgIEfyACBSAAIAMQiAELCxoAIAAgASACEMMBIgIEfyACBSAAIAMQ1gELCxoAIAAgASACEMMBIgIEfyACBSAAIAMQiQELCx4AIAAgASgCACUBEAU2AgggAEEANgIEIAAgATYCAAsfACAARQRAQZCtwABBMhDYAwALIAAgAiABKAIQEQEACxwAIABBATYCACAAIAEoAgQgASgCAGtBGG42AgQLKQAgACAALQAEIAFBLkZyOgAEIAAoAgAiACgCACABIAAoAgQoAhARAQALyAMCA34Gf0HUjsEAKAIARQRAIwBBMGsiBiQAAn8gAEUEQEHQqcAAIQRBAAwBCyAAKAIAIQQgAEEANgIAIABBCGpB0KnAACAEQQFxIgUbIQQgACgCBEEAIAUbCyEFIAZBEGogBEEIaikCACICNwMAIAYgBCkCACIDNwMIIAZBKGpB5I7BACkCADcDACAGQSBqIgBB3I7BACkCADcDAEHUjsEAKQIAIQFB2I7BACAFNgIAQdSOwQBBATYCAEHcjsEAIAM3AgBB5I7BACACNwIAIAYgATcDGCABpwRAAkAgACgCBCIHRQ0AIAAoAgwiCARAIAAoAgAiBEEIaiEFIAQpAwBCf4VCgIGChIiQoMCAf4MhAQNAIAFQBEADQCAEQeAAayEEIAUpAwAgBUEIaiEFQoCBgoSIkKDAgH+DIgFCgIGChIiQoMCAf1ENAAsgAUKAgYKEiJCgwIB/hSEBCyAEIAF6p0EDdkF0bGpBBGsoAgAiCUGEAU8EQCAJENgBCyABQgF9IAGDIQEgCEEBayIIDQALCyAHIAdBDGxBE2pBeHEiBGpBCWoiBUUNACAAKAIAIARrIAVBCBC9AwsLIAZBMGokAAtB2I7BAAscABClAyECIABBADYCCCAAIAI2AgQgACABNgIACxoAIAAgATcDECAAQQI6AAAgACABQj+INwMICxsAQQIgACgCACUBECoiAEEARyAAQf///wdGGwsaAQF/IAAoAgAiAQRAIAAoAgQgAUEBEL0DCwtCACAABEAgACABEN0DAAsjAEEgayIAJAAgAEEANgIYIABBATYCDCAAQdDXwAA2AgggAEIENwIQIABBCGogAhDFAgALFgAgACgCAEGAgICAeEcEQCAAENADCwsVACABaUEBRiAAQYCAgIB4IAFrTXELGAEBbyAAJQEgARAEIQIQfCIAIAImASAACxsAIABBADYCFCAAQQA2AgwgAEGAgICAeDYCAAsXACAAIAI2AgggACABNgIEIAAgAjYCAAvtBgEGfwJ/AkACQAJAAkACQCAAQQRrIgUoAgAiBkF4cSIEQQRBCCAGQQNxIgcbIAFqTwRAIAdBACABQSdqIgkgBEkbDQECQAJAIAJBCU8EQCACIAMQYCIIDQFBAAwJCyADQcz/e0sNAUEQIANBC2pBeHEgA0ELSRshAQJAIAdFBEAgAUGAAkkgBCABQQRySXIgBCABa0GBgAhPcg0BDAkLIABBCGsiAiAEaiEHAkACQAJAAkAgASAESwRAIAdBuJLBACgCAEYNBCAHQbSSwQAoAgBGDQIgBygCBCIGQQJxDQUgBkF4cSIGIARqIgQgAUkNBSAHIAYQYyAEIAFrIgNBEEkNASAFIAEgBSgCAEEBcXJBAnI2AgAgASACaiIBIANBA3I2AgQgAiAEaiICIAIoAgRBAXI2AgQgASADEFoMDQsgBCABayIDQQ9LDQIMDAsgBSAEIAUoAgBBAXFyQQJyNgIAIAIgBGoiASABKAIEQQFyNgIEDAsLQaySwQAoAgAgBGoiBCABSQ0CAkAgBCABayIDQQ9NBEAgBSAGQQFxIARyQQJyNgIAIAIgBGoiASABKAIEQQFyNgIEQQAhA0EAIQEMAQsgBSABIAZBAXFyQQJyNgIAIAEgAmoiASADQQFyNgIEIAIgBGoiAiADNgIAIAIgAigCBEF+cTYCBAtBtJLBACABNgIAQaySwQAgAzYCAAwKCyAFIAEgBkEBcXJBAnI2AgAgASACaiIBIANBA3I2AgQgByAHKAIEQQFyNgIEIAEgAxBaDAkLQbCSwQAoAgAgBGoiBCABSw0HCyADEDYiAUUNASADQXxBeCAFKAIAIgJBA3EbIAJBeHFqIgIgAiADSxsiAgRAIAEgACAC/AoAAAsgABBFIAEMCAsgAyABIAEgA0sbIgIEQCAIIAAgAvwKAAALIAUoAgAiAkF4cSIDIAFBBEEIIAJBA3EiAhtqSQ0DIAJBACADIAlLGw0EIAAQRQsgCAwGC0HVycAAQS5BhMrAABCmAgALQZTKwABBLkHEysAAEKYCAAtB1cnAAEEuQYTKwAAQpgIAC0GUysAAQS5BxMrAABCmAgALIAUgASAGQQFxckECcjYCACABIAJqIgIgBCABayIBQQFyNgIEQbCSwQAgATYCAEG4ksEAIAI2AgAgAAwBCyAACwsVACAAIAIgAxDnAzYCBCAAQQA2AgALEAAgACABIAEgAmoQhQJBAAsQACAAIAEgASACahCEAkEACxAAIAEEQCAAIAEgAhC9AwsLEwBB8I7BACAArUIghkIBhDcDAAsZACABKAIAQeD2wABBDiABKAIEKAIMEQIACxYAIAAoAgAgASACIAAoAgQoAgwRAgALFgIBbwF/IAAQMiEBEHwiAiABJgEgAgsWAgFvAX8gABA0IQEQfCICIAEmASACC60ZAgp/BH4QfCIIIAAmASMAQdAAayIJJAACQCABBEAgCSABIAIQoAIgCUEMaiAJKAIAIAkoAgQQkwMMAQsgCUGAgICAeDYCDAsgCUEgaiEGIwBBkAFrIgQkACAEIAg2AgQCQAJAAkACQAJAAkAgCUEMaiIBKAIAQYCAgIB4RwRAIARBEGogAUEIaigCADYCACAEIAEpAgA3AwgMAQsgBEHIAGpBAkEBQQEQmAEgBCgCTCECIAQoAkhBAUYNASAEKAJQIgFB5dwBOwAAIARBAjYCECAEIAE2AgwgBCACNgIICyAEQcgAaiEHIAgQuAMhASMAQcABayIDJAAgAyABNgIMIANByABqIANBDGoQeCADKAJIIQECQAJAAkACQAJAAkACQAJAIAMtAEwiCkECaw4CAgABCyAHQQA2AgAgByABNgIEIAMoAgwiBUGDAUsNAwwECyADIAo6AHQgAyABNgJwIANBADYCaCADAn5B2JLBACgCAEEBRgRAQeiSwQApAwAhD0HgksEAKQMADAELIANByABqEIECQdiSwQBCATcDAEHoksEAIAMpA1AiDzcDACADKQNICyINNwNYQeCSwQAgDUIBfDcDACADIA83A2AgA0IANwNQIANBADYCTCADQeiSwAA2AkggA0GMAWohBQJAA0ACQCADQagBaiADQegAahCKAQJAAkACQAJAIAMoAqgBIgJBgICAgHhrDgIEAAELIAMoAqwBIQUMAQsgAyADKQKsASIPNwK4ASADIAI2ArQBIAMoAmggA0EANgJoRQ0JIANBEGogAygCbBDAASADKAIQQYCAgIB4Rw0BIAMoAhQhBSADQbQBahDQAwsgB0EANgIAIAcgBTYCBCADQcgAahCIAgwDCyADQTBqIANBGGooAgAiATYCACADQYgBaiAPQiCIpyIKNgIAIANBQGsgCjYCACADQaABaiABNgIAIAMgAykCECINNwMoIAVBCGogATYCACAFIA03AgAgAyACNgKAASADIA+nIgs2AoQBIAMgDTcDmAEgAyADKQKAATcDOCADQRBqIgEgA0HIAGogA0E4aiADQZgBahBJIAEQjwMMAQsLIAMgCjYCiAEgAyALNgKEASADQYCAgIB4NgKAASADQYABahDNAiAHQRhqIANB4ABqKQMANwMAIAdBEGogA0HYAGopAwA3AwAgB0EIaiADQdAAaikDADcDACAHIAMpA0g3AwALIANB6ABqEL0CDAELIAMgA0EMahDDAiADKAIAQQFxBEAgAyADKAIENgI4IANBGGogA0E4ahCFAyADQQA2AiQgA0EANgIQIAMoAhgEQCADKAIgIgIgAygCHGsiAUEAIAEgAk0bIQULQQAhCkHgksEAAn5B2JLBACgCAEEBRgRAQeiSwQApAwAhD0HgksEAKQMADAELIANBgAFqEIECQdiSwQBCATcDAEHoksEAIAMpA4gBIg83AwAgAykDgAELIg1CAXw3AwACQCAFRQRAQeiSwAAhBQwBCyADQYABaiADQbQBakEYIAVBB00Ef0EEQQggBUEESRsFQX9BqtUCIAUgBUGq1QJPG0EDdEEHbkEBa2d2QQFqCxCBASADKAKEASEKIAMoAoABIgVFBEAgAzUCiAEhDkEAIQUMAQsgAykCiAEhDiAKQQlqIgFFDQAgBUH/ASAB/AsACyADIA83A2AgAyANNwNYIAMgDjcDUCADIAo2AkwgAyAFNgJIIANBgAFqIANBEGoQcgJAAkACQCADKAKAAUGBgICAeEcEQCADQYwBaiECA0AgA0H4AGogA0GQAWopAgA3AwAgA0HwAGogA0GIAWoiASkCADcDACADIAMpAoABIg03A2ggDadBgICAgHhGDQIgA0GgAWogASgCADYCACADQbABaiACQQhqKAIANgIAIAMgAykCgAE3A5gBIAMgAikCADcDqAEgA0G0AWoiASADQcgAaiADQZgBaiADQagBahBJIAEQjwMgA0GAAWogA0EQahByIAMoAoABQYGAgIB4Rw0ACwsgByADKAKEATYCBCAHQQA2AgAgA0HIAGoQiAIgAygCEEUNAiADKAIUIgVBgwFLDQEMAgsgA0HoAGoQzQIgB0EYaiADQeAAaikDADcDACAHQRBqIANB2ABqKQMANwMAIAdBCGogA0HQAGopAwA3AwAgByADKQNINwMAIAMoAhBFDQEgAygCFCIFQYQBSQ0BCyAFENgBCyADKAI4IgFBhAFJDQEgARDYAQwBCyADQQxqIANBtAFqQbiRwAAQVCEBIAdBADYCACAHIAE2AgQLIAMoAgwiBUGEAUkNAQsgBRDYAQsgA0HAAWokAAwBC0H4psAAQTEQ2AMACwJAQeCSwQACfgJAIAQoAkhFBEACQCAHKAIABEAgBxCIAgwBCyAHKAIEIgFBhAFPBEAgARDYAQsLIARB7ABqIARBBGoQ1AEgBCgCbEGAgICAeEYNAyAEQUBrIARB9ABqKAIANgIAIAQgBCkCbDcDOEHYksEAKAIAQQFHDQFB6JLBACkDACEOQeCSwQApAwAMAgsgBEEgaiAEQdAAaikDADcDACAEQTBqIARB4ABqKQMANwMAIARBKGoiASAEQdgAaikDADcDACAEIAQpA0g3AxgCQAJAAkAgBCgCJEUNACABIARBCGoQVyENIAQoAhgiCkEYayELIAQoAhwiByANp3EhAyANQhmIQv8Ag0KBgoSIkKDAgAF+IQ8gBCgCECEFIAQoAgwhAgNAIAMgCmopAAAiECAPhSINQn+FIA1CgYKEiJCgwIABfYNCgIGChIiQoMCAf4MiDlBFBEADQCACIAUgCyAOeqdBA3YgA2ogB3FBaGxqIgEoAgQgASgCCBD6Ag0FIA5CAX0gDoMiDlBFDQALCyAQIBBCAYaDQoCBgoSIkKDAgH+DUARAIAMgDEEIaiIMaiAHcSEDDAELCyAEKAIkRQ0AIAQoAhgiASkDAEKAgYKEiJCgwIB/gyIOQoCBgoSIkKDAgH9RBEAgAUEIaiEDA0AgAUHAAWshASADKQMAIANBCGohA0KAgYKEiJCgwIB/gyIOQoCBgoSIkKDAgH9RDQALCyAEQfgAaiABIA5CgIGChIiQoMCAf4V6p0EDdkFobGpBGGsQ3AEgBCgCeCIDQYCAgIB4Rg0AIAQoAoABIQEgBCgCfCEMDAELQQIhASAEQYQBakECQQFBARCYASAEKAKIASEDIAQoAoQBQQFGDQYgBCgCjAEiDEHl3AE7AAALIAYgBCkDGDcDACAGIAE2AiggBiAMNgIkIAYgAzYCICAGQRhqIARBMGopAwA3AwAgBkEQaiAEQShqKQMANwMAIAZBCGogBEEgaikDADcDACAEQQhqENADDAYLIAYgBCkDGDcDACAGIAQoAhA2AiggBiAEKQMINwMgIAZBCGogBEEgaikDADcDACAGQRBqIARBKGopAwA3AwAgBkEYaiAEQTBqKQMANwMADAULIARByABqEIECQdiSwQBCATcDAEHoksEAIAQpA1AiDjcDACAEKQNICyINQgF8NwMAIARB0ABqIgtBkIDAACkDADcDACAEQeAAaiIFIA43AwAgBEHYAGoiCCANNwMAIARBiIDAACkDADcDSCAEQYQBaiICIARBCGoQ3AEgBEH4AGoiASAEQcgAaiACIARBOGoQSSABEI8DIAZBGGogBSkDADcDACAGQRBqIAgpAwA3AwAgBkEIaiALKQMANwMAIAYgBCkDSDcDACAGIAQpAwg3AiAgBkEoaiAEQRBqKAIANgIAIAQoAgQhCAwDCyAEQewAahCPA0HgksEAAn5B2JLBACgCAEEBRgRAQeiSwQApAwAhDkHgksEAKQMADAELIARByABqEIECQdiSwQBCATcDAEHoksEAIAQpA1AiDjcDACAEKQNICyINQgF8NwMAIARB0ABqIgtBkIDAACkDADcDACAEQeAAaiIFIA43AwAgBEHYAGoiCCANNwMAIARBiIDAACkDADcDSCAEQfgAaiICIARBCGoQ3AEgBEEANgKMASAEQoCAgIAQNwKEASAEQewAaiIBIARByABqIAIgBEGEAWoQSSABEI8DIAZBGGogBSkDADcDACAGQRBqIAgpAwA3AwAgBkEIaiALKQMANwMAIAYgBCkDSDcDACAGIAQpAwg3AiAgBkEoaiAEQRBqKAIANgIAIAQoAgQiCEGEAUkNBAwDCyACIAQoAlBBtILAABCOAwALIAMgBCgCjAFBtILAABCOAwALIAhBgwFNDQELIAgQ2AELIARBkAFqJAAgCUEANgIYIAlBGGoQpAJBCGogCUHQAGokAAsTACAAIAEoAgQ2AgQgAEEANgIACxUAIAAoAgAlASABKAIAJQEQKEEARwsVACAAKAIAJQEgASgCACUBEC5BAEcLFQAgACgCACUBIAEoAgAlARAvQQBHCxQAIAAoAgAgASAAKAIEKAIMEQEAC4MIAQR/IwBB8ABrIgUkACAFIAM2AgwgBSACNgIIAn8gAUGBAk8EQAJ/QYACIAAsAIACQb9/Sg0AGkH/ASAALAD/AUG/f0oNABpB/gFB/QEgACwA/gFBv39KGwsiBiAAaiwAAEG/f0oEQEHL/MAAIQdBBQwCCyAAIAFBACAGIAQQpAMAC0EBIQcgASEGQQALIQggBSAGNgIUIAUgADYCECAFIAg2AhwgBSAHNgIYAkACQAJAAkAgASACSSIGIAEgA0lyRQRAIAIgA0sNASACRSABIAJNckUEQCAFQQxqIAVBCGogACACaiwAAEG/f0obKAIAIQMLIAUgAzYCICADIAEiAkkEQCADQQFqIgIgA0EDayIGQQAgAyAGTxsiBkkNAwJ/IAIgBmsiB0EBayAAIANqLAAAQb9/Sg0AGiAHQQJrIAAgAmoiAkECaywAAEG/f0oNABogB0EDayACQQNrLAAAQb9/Sg0AGiAHQXxBeyACQQRrLAAAQb9/ShtqCyAGaiECCwJAIAJFDQAgASACTQRAIAEgAkYNAQwFCyAAIAJqLAAAQb9/TA0ECwJ/AkACQCABIAJGDQACQAJAIAAgAmoiASwAACIAQQBIBEAgAS0AAUE/cSEGIABBH3EhAyAAQV9LDQEgA0EGdCAGciEADAILIAUgAEH/AXE2AiRBAQwECyABLQACQT9xIAZBBnRyIQYgAEFwSQRAIAYgA0EMdHIhAAwBCyADQRJ0QYCA8ABxIAEtAANBP3EgBkEGdHJyIgBBgIDEAEYNAQsgBSAANgIkIABBgAFPDQFBAQwCCyAEEMYDAAtBAiAAQYAQSQ0AGkEDQQQgAEGAgARJGwshACAFIAI2AiggBSAAIAJqNgIsIAVBBTYCNCAFQdT9wAA2AjAgBUIFNwI8IAUgBUEYaq1CgICAgLAMhDcDaCAFIAVBEGqtQoCAgICwDIQ3A2AgBSAFQShqrUKAgICA0AyENwNYIAUgBUEkaq1CgICAgOAMhDcDUCAFIAVBIGqtQoCAgIDABoQ3A0gMBAsgBSACIAMgBhs2AiggBUEDNgI0IAVBlP7AADYCMCAFQgM3AjwgBSAFQRhqrUKAgICAsAyENwNYIAUgBUEQaq1CgICAgLAMhDcDUCAFIAVBKGqtQoCAgIDABoQ3A0gMAwsgBUEENgI0IAVB9PzAADYCMCAFQgQ3AjwgBSAFQRhqrUKAgICAsAyENwNgIAUgBUEQaq1CgICAgLAMhDcDWCAFIAVBDGqtQoCAgIDABoQ3A1AgBSAFQQhqrUKAgICAwAaENwNIDAILIAYgAkGs/sAAEMQDAAsgACABIAIgASAEEKQDAAsgBSAFQcgAajYCOCAFQTBqIAQQxQIACxQCAW8BfxAGIQAQfCIBIAAmASABCxQCAW8BfxALIQAQfCIBIAAmASABCxQCAW8BfxAMIQAQfCIBIAAmASABC8wCAQR/EHwiBCABJgEjAEEQayIDJAAgA0EEaiAAEPgBIAMoAgQhAiMAQdAAayIAJAAgACAEEDcCQCAAKAIARQRAIAAgACgCBDYCICAAQQE2AjQgAEGQjsAANgIwIABCATcCPCAAQQI2AkwgACAAQcgAajYCOCAAIABBIGo2AkggAEEkaiIEIABBMGoiBRBeIAAoAiggACgCLBDnAyECIAQQ0AMgACACNgIwIAUQ5QMgAkGEAU8EQCACENgBCyAAKAIgIgJBhAFJDQEgAhDYAQwBCyACEIcCIAJBGGogAEEYaikDADcDACACQRBqIABBEGopAwA3AwAgAkEIaiAAQQhqKQMANwMAIAIgACkDADcDAAsgAEHQAGokACADKAIIQQA2AgAgAygCDCIAIAAoAgBBAWsiADYCACAARQRAIANBDGoQtwILIANBEGokAAsRACAAKAIEIAAoAgggARDfAwsZAAJ/IAFBCU8EQCABIAAQYAwBCyAAEDYLCxAAIAAgAjYCBCAAIAE2AgALIAEBbyAAKAIAJQEgASgCACUBEAIhAhB8IgAgAiYBIAALDgAgACgCACUBEA5BAEcLDgAgACgCACUBEBVBAEcLEAAgACgCBCAAKAIIIAEQOwsQACAAKAIAIAAoAgQgARA7CxEAIAAoAgAgACgCBCABEN8DCyIAIABC7bqtts2F1PXjADcDCCAAQviCmb2V7sbFuX83AwALIQAgAEKAvN+Fq6X4myc3AwggAEKf9ZaU1u7tw6F/NwMACxMAIABBkNPAADYCBCAAIAE2AgALEQAgASAAKAIAIAAoAgQQmwMLEAAgASAAKAIAIAAoAgQQSgsQACABKAIAIAEoAgQgABBNCxABAX8QfCIBIAAlASYBIAELhAMDBn8BfgFvIwBBEGsiBCQAIARBBGoiBiAAEOgBIAQoAgQhAyMAQSBrIgAkACAAEKcDIgI2AgwgAygCDCIFBEAgAygCACICQQhqIQMgAikDAEJ/hUKAgYKEiJCgwIB/gyEHA0AgB1AEQANAIAJBwAFrIQIgAykDACADQQhqIQNCgIGChIiQoMCAf4MiB0KAgYKEiJCgwIB/UQ0ACyAHQoCBgoSIkKDAgH+FIQcLIAAgAiAHeqdBA3ZBaGxqIgFBFGsoAgAgAUEQaygCABDnAzYCGCAAIAFBCGsoAgAgAUEEaygCABDnAzYCHCAAQRBqIABBDGogAEEYaiAAQRxqEOcBAkAgAC0AECIBQQFHIAFFcg0AIAAoAhQiAUGEAUkNACABENgBCyAAKAIcIgFBhAFPBEAgARDYAQsgACgCGCIBQYQBTwRAIAEQ2AELIAdCAX0gB4MhByAFQQFrIgUNAAsgACgCDCECCyAAQSBqJAAgBhC2AiAEQRBqJAAgAiUBIAIQ2AELtjMDEH8BfgFvIwBBEGsiCSQAIAlBBGoiCyAAEOgBAn8gCSgCBCEEIwBB4ABrIgAkACAAQQA2AgggAEEANgIAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkAgBCgCjAJBgICAgHhGDQAgACAEQYwCajYCDCAAQcgAaiIBQQZBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIDQQRqQaCEwAAvAAA7AAAgA0GchMAAKAAANgAAIAAgAzYCHCAAIAI2AhggAEEGNgIgIAEgAEEMahDEASAALQBIQQZGDQEgAEFAayAAQdgAaikDADcDACAAQThqIABB0ABqKQMANwMAIAAgACkDSDcDMCABIAAgAEEYaiAAQTBqEH8gAC0ASEEGRg0AIAEQqAILAkAgBCgCAEEBRw0AIAQ1AgQhESAAQcgAaiIDQQVBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIBQQRqQaaEwAAtAAA6AAAgAUGihMAAKAAANgAAIABBBTYCICAAIAE2AhwgACACNgIYIAAgETcDWCAAQgA3A1AgAEECOgBIIABBMGoiAiAAIABBGGogAxB/IAAtADBBBkYNACACEKgCCwJAIAQoAghBAUcNACAENQIMIREgAEHIAGoiA0EXQQFBARCYASAAKAJMIQIgACgCSEEBRg0PIAAoAlAiAUEPakG2hMAAKQAANwAAIAFBCGpBr4TAACkAADcAACABQaeEwAApAAA3AAAgAEEXNgIgIAAgATYCHCAAIAI2AhggACARNwNYIABCADcDUCAAQQI6AEggAEEwaiICIAAgAEEYaiADEH8gAC0AMEEGRg0AIAIQqAILAkAgBCgCmAJBgICAgHhGDQAgACAEQZgCajYCDCAAQcgAaiIBQQVBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIDQQRqQcKEwAAtAAA6AAAgA0G+hMAAKAAANgAAIABBBTYCICAAIAM2AhwgACACNgIYIAEgAEEMahDEASAALQBIQQZGDQIgAEFAayAAQdgAaikDADcDACAAQThqIABB0ABqKQMANwMAIAAgACkDSDcDMCABIAAgAEEYaiAAQTBqEH8gAC0ASEEGRg0AIAEQqAILAkAgBCgCSEGAgICAeEYNACAAQcgAaiIBQQtBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIDQQdqQcqEwAAoAAA2AAAgA0HDhMAAKQAANwAAIABBCzYCICAAIAM2AhwgACACNgIYIAEgBEEoahBbIAAtAEhBBkYNAyAAQUBrIABB2ABqKQMANwMAIABBOGogAEHQAGopAwA3AwAgACAAKQNINwMwIAEgACAAQRhqIABBMGoQfyAALQBIQQZGDQAgARCoAgsCQCAEKAIQQQFHDQAgBDUCFCERIABByABqIgNBBUEBQQEQmAEgACgCTCECIAAoAkhBAUYNDyAAKAJQIgFBBGpB0oTAAC0AADoAACABQc6EwAAoAAA2AAAgAEEFNgIgIAAgATYCHCAAIAI2AhggACARNwNYIABCADcDUCAAQQI6AEggAEEwaiICIAAgAEEYaiADEH8gAC0AMEEGRg0AIAIQqAILIABByABqIgFBB0EBQQEQmAEgACgCTCECIAAoAkhBAUYNDiAAKAJQIgNBA2pB1oTAACgAADYAACADQdOEwAAoAAA2AAAgAEEHNgIgIAAgAzYCHCAAIAI2AhggASAEQegBahDGASAALQBIQQZGDQMgAEFAayAAQdgAaikDADcDACAAQThqIABB0ABqKQMANwMAIAAgACkDSDcDMCABIAAgAEEYaiAAQTBqEH8gAC0ASEEGRwRAIAEQqAILAkAgBCgCpAJBgICAgHhGDQAgACAEQaQCajYCDCAAQcgAaiIBQQlBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIDQQhqQeKEwAAtAAA6AAAgA0HahMAAKQAANwAAIABBCTYCICAAIAM2AhwgACACNgIYIAEgAEEMahDEASAALQBIQQZGDQUgAEFAayAAQdgAaikDADcDACAAQThqIABB0ABqKQMANwMAIAAgACkDSDcDMCABIAAgAEEYaiAAQTBqEH8gAC0ASEEGRg0AIAEQqAILAkAgBC0A2AIiA0ECRg0AIABByABqIgVBC0EBQQEQmAEgACgCTCECIAAoAkhBAUYNDyAAKAJQIgFBB2pB6oTAACgAADYAACABQeOEwAApAAA3AAAgAEELNgIgIAAgATYCHCAAIAI2AhggACADOgBJIABBAToASCAAQTBqIgIgACAAQRhqIAUQfyAALQAwQQZGDQAgAhCoAgsCQCAELQDZAiIDQQJGDQAgAEHIAGoiBUEKQQFBARCYASAAKAJMIQIgACgCSEEBRg0PIAAoAlAiAUEIakH2hMAALwAAOwAAIAFB7oTAACkAADcAACAAQQo2AiAgACABNgIcIAAgAjYCGCAAIAM6AEkgAEEBOgBIIABBMGoiAiAAIABBGGogBRB/IAAtADBBBkYNACACEKgCCwJAIAQtALgBQQZGDQAgACAEQbgBajYCDCAAQcgAaiIBQQ1BAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIDQQVqQf2EwAApAAA3AAAgA0H4hMAAKQAANwAAIABBDTYCICAAIAM2AhwgACACNgIYIAEgAEEMahBRIAAtAEhBBkYNBiAAQUBrIABB2ABqKQMANwMAIABBOGogAEHQAGopAwA3AwAgACAAKQNINwMwIAEgACAAQRhqIABBMGoQfyAALQBIQQZGDQAgARCoAgsCQCAEKAJ4QYCAgIB4Rg0AIABByABqIgFBBEEBQQEQmAEgACgCTCECIAAoAkhBAUYNDyAAKAJQIgNB7sK1qwY2AAAgAEEENgIgIAAgAzYCHCAAIAI2AhggASAEQdgAahBbIAAtAEhBBkYNByAAQUBrIABB2ABqKQMANwMAIABBOGogAEHQAGopAwA3AwAgACAAKQNINwMwIAEgACAAQRhqIABBMGoQfyAALQBIQQZGDQAgARCoAgsCQCAEKAIYQQFHDQAgBDUCHCERIABByABqIgNBCkEBQQEQmAEgACgCTCECIAAoAkhBAUYNDyAAKAJQIgFBCGpBkYXAAC8AADsAACABQYmFwAApAAA3AAAgAEEKNgIgIAAgATYCHCAAIAI2AhggACARNwNYIABCADcDUCAAQQI6AEggAEEwaiICIAAgAEEYaiADEH8gAC0AMEEGRg0AIAIQqAILAkAgBCgCIEEBRw0AIAQ1AiQhESAAQcgAaiIDQQVBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIBQQRqQZeFwAAtAAA6AAAgAUGThcAAKAAANgAAIABBBTYCICAAIAE2AhwgACACNgIYIAAgETcDWCAAQgA3A1AgAEECOgBIIABBMGoiAiAAIABBGGogAxB/IAAtADBBBkYNACACEKgCCwJAIAQoArACQYCAgIB4Rg0AIAAgBEGwAmo2AgwgAEHIAGoiAUELQQFBARCYASAAKAJMIQIgACgCSEEBRg0PIAAoAlAiA0EHakGfhcAAKAAANgAAIANBmIXAACkAADcAACAAQQs2AiAgACADNgIcIAAgAjYCGCABIABBDGoQxAEgAC0ASEEGRg0IIABBQGsgAEHYAGopAwA3AwAgAEE4aiAAQdAAaikDADcDACAAIAApA0g3AzAgASAAIABBGGogAEEwahB/IAAtAEhBBkYNACABEKgCCwJAIAQtANABQQZGDQAgACAEQdABajYCDCAAQcgAaiIBQQtBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIDQQdqQaqFwAAoAAA2AAAgA0GjhcAAKQAANwAAIABBCzYCICAAIAM2AhwgACACNgIYIAEgAEEMahBRIAAtAEhBBkYNCSAAQUBrIABB2ABqKQMANwMAIABBOGogAEHQAGopAwA3AwAgACAAKQNINwMwIAEgACAAQRhqIABBMGoQfyAALQBIQQZGDQAgARCoAgsCQCAEKAL8AUUNACAAQcgAaiIDQRxBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIBQRhqQcaFwAAoAAA2AAAgAUEQakG+hcAAKQAANwAAIAFBCGpBtoXAACkAADcAACABQa6FwAApAAA3AAAgAEEcNgIgIAAgATYCHCAAIAI2AhgjAEEgayICJAAgBEH0AWoiBSgCBCEBIAJBEGpBASAFKAIIIgUQzwECQCACKAIQQYCAgIB4RgRAIAMgAigCFDYCBCADQQY6AAAMAQsgAkEIaiIKIAJBGGooAgA2AgAgAiACKQIQNwMAAkAgBUUNACAFQQxsIQYDQAJAIAIgATYCECMAQSBrIgUkACAFQQhqIAJBEGoQxAECfyAFLQAIQQZGBEAgBSgCDAwBCyACKAIIIgcgAigCAEYEQCACQZyZwAAQlAILIAIoAgQgB0EYbGoiCCAFKQMINwMAIAhBEGogBUEYaikDADcDACAIQQhqIAVBEGopAwA3AwAgAiAHQQFqNgIIQQALIQcgBUEgaiQAIAcNACABQQxqIQEgBkEMayIGDQEMAgsLIANBBjoAACADIAc2AgQgAhDCAiACENMDDAELIAJBG2ogCigCADYAACADQQQ6AAAgAiACKQMANwATIAMgAikAEDcAASADQQhqIAJBF2opAAA3AAALIAJBIGokACAALQBIQQZGDQogAEFAayAAQdgAaikDADcDACAAQThqIABB0ABqKQMANwMAIAAgACkDSDcDMCADIAAgAEEYaiAAQTBqEH8gAC0ASEEGRg0AIAMQqAILAkAgBCgCiAJFDQAgAEHIAGoiA0EfQQFBARCYASAAKAJMIQIgACgCSEEBRg0PIAAoAlAiAUEXakHhhcAAKQAANwAAIAFBEGpB2oXAACkAADcAACABQQhqQdKFwAApAAA3AAAgAUHKhcAAKQAANwAAIABBHzYCICAAIAE2AhwgACACNgIYIAMgBEGAAmoQcyAALQBIQQZGDQsgAEFAayAAQdgAaikDADcDACAAQThqIABB0ABqKQMANwMAIAAgACkDSDcDMCADIAAgAEEYaiAAQTBqEH8gAC0ASEEGRg0AIAMQqAILAkAgBCgC1AIiA0UNACAAQcgAaiIHQQRBAUEBEJgBIAAoAkwhAiAAKAJIQQFGDQ8gACgCUCIBQfLevaMHNgAAIABBBDYCFCAAIAE2AhAgACACNgIMIABBMGohBSMAQTBrIgIkACADKAIwIQggAygCJEGAgICAeEYhBiADKAI8IQogAygCSCEMIAMoAlQhDSADKAJgIQ4gAygCbCEPIAJBGGoiAUEANgIUIAFBADYCDCABQYCAgIB4NgIAIAIoAhwhAQJAIAIoAhgiEEGBgICAeEYEQCAFQQY6AAAgBSABNgIEDAELIAJBEGogAkEoaikCADcCACACIAIpAiA3AgggAiABNgIEIAIgEDYCAAJAIAZFBEAgAkHiicAAQQUQwwEiAQ0BIAIgA0EkahCIASIBDQELIAJByIPAAEEGEMMBIgENAAJ/IwBB4ABrIgEkACACKAIAIQYgAkGAgICAeDYCACAGQYCAgIB4RwRAIAEgAikCBDcCKCABIAY2AiQgAUEwaiADEFwCQCABLQAwQQZGBEAgASgCNCEGIAFBJGoQ0AMMAQsgAUHYAGogAUFAaykDADcDACABQdAAaiABQThqKQMANwMAIAEgASkDMDcDSCABQQhqIgYgAkEMaiABQSRqIAFByABqEH8gBhCqAkEAIQYLIAFB4ABqJAAgBgwBC0H4l8AAQStBjJnAABD3AQALIgENACACQeeJwABBCBDDASIBDQAgAiADQfgAahCJASIBDQAgCEGAgICAeEcEQCACQcOEwABBCxDDASIBDQEgAiADQTBqEIgBIgENAQsgAkHvicAAQQoQwwEiAQ0AIAIgA0GoAWoQ1gEiAQ0AIApBgICAgHhHBEAgAkH5icAAQQkgA0E8ahCCAyIBDQELIAJB1oLAAEEIIANBhAFqEIQDIgENACACQYKKwABBDiADQakBahCDAyIBDQAgAkGQisAAQQwgA0GqAWoQgwMiAQ0AIAJBnIrAAEEKIANBqwFqEIMDIgENACACQaaKwABBDCADQawBahCDAyIBDQAgAkGyisAAQQkgA0GtAWoQgwMiAQ0AIAJBhYXAAEEEIANBkAFqEIQDIgENACAMQYCAgIB4RwRAIAJB3oLAAEEMIANByABqEIIDIgENAQsgAkHQgsAAQQYgA0GcAWoQhAMiAQ0AIA1BgICAgHhHBEAgAkG7isAAQQ0gA0HUAGoQggMiAQ0BCyAOQYCAgIB4RwRAIAJByIrAAEEOIANB4ABqEIIDIgENAQsgAkHWisAAQQkQwwEiAUUEQAJ/IwBB4ABrIgEkACACKAIAIQYgAkGAgICAeDYCACAGQYCAgIB4RwRAIAEgAikCBDcCKCABIAY2AiQgAUEwaiADQSBqNAIAEIsDAkAgAS0AMEEGRgRAIAEoAjQhBiABQSRqENADDAELIAFB2ABqIAFBQGspAwA3AwAgAUHQAGogAUE4aikDADcDACABIAEpAzA3A0ggAUEIaiIGIAJBDGogAUEkaiABQcgAahB/IAYQqgJBACEGCyABQeAAaiQAIAYMAQtB+JfAAEErQYyZwAAQ9wEACyEBCyABDQAgD0GAgICAeEcEQCACQd+KwABBGiADQewAahCCAyIBDQELIAJBKGogAkEQaikCADcDACACQSBqIAJBCGopAgA3AwAgAiACKQIANwMYIAUgAkEYahDwAQwBCyAFQQY6AAAgBSABNgIEIAJBDGoQzgEgAhCPAwsgAkEwaiQAIABBADoASAJAIAAtADBBBkcEQCAAQShqIABBQGspAwA3AwAgAEEgaiAAQThqKQMANwMAIAAgACkDMDcDGCAHEKgCDAELIABBKGogAEHYAGopAwA3AwAgAEEgaiAAQdAAaikDADcDACAAIAApA0g3AxggAEEwakEEchDMAQsgAEHIAGoiAiAAIABBDGogAEEYahB/IAAtAEhBBkYNACACEKgCCwJAIAQoArwCQYCAgIB4Rg0AIAAgBEG8Amo2AgwgAEHIAGoiAUEEQQFBARCYASAAKAJMIQIgACgCSEEBRg0PIAAoAlAiA0Hz2NW7BjYAACAAQQQ2AiAgACADNgIcIAAgAjYCGCABIABBDGoQxAEgAC0ASEEGRg0MIABBQGsgAEHYAGopAwA3AwAgAEE4aiAAQdAAaikDADcDACAAIAApA0g3AzAgASAAIABBGGogAEEwahB/IAAtAEhBBkYNACABEKgCCwJAIAQoAqgBQYCAgIB4Rg0AIABByABqIgFBCEEBQQEQmAEgACgCTCECIAAoAkhBAUYNDyAAKAJQIgNC8+qJo5eNnbblADcAACAAQQg2AiAgACADNgIcIAAgAjYCGCABIARBiAFqEFsgAC0ASEEGRg0NIABBQGsgAEHYAGopAwA3AwAgAEE4aiAAQdAAaikDADcDACAAIAApA0g3AzAgASAAIABBGGogAEEwahB/IAAtAEhBBkYNACABEKgCCwJAIAQoAsgCQYCAgIB4Rg0AIAAgBEHIAmo2AgwgAEHIAGoiBEEHQQFBARCYASAAKAJMIQIgACgCSEEBRg0PIAAoAlAiAUEDakH8hcAAKAAANgAAIAFB+YXAACgAADYAACAAQQc2AiAgACABNgIcIAAgAjYCGCAEIABBDGoQxAEgAC0ASEEGRg0OIABBQGsgAEHYAGopAwA3AwAgAEE4aiAAQdAAaikDADcDACAAIAApA0g3AzAgBCAAIABBGGogAEEwahB/IAAtAEhBBkYNACAEEKgCCyAAQdQAaiAAQQhqKAIANgIAIAAgACkCADcCTCAAQQU6AEggAEHIAGoiAhBMIAIQqAIgAEHgAGokAAwPCyAAIAAoAkw2AjBBqIHAAEErIABBMGpBmIHAAEH4h8AAEM0BAAsgACAAKAJMNgIwQaiBwABBKyAAQTBqQZiBwABBiIjAABDNAQALIAAgACgCTDYCMEGogcAAQSsgAEEwakGYgcAAQZiIwAAQzQEACyAAIAAoAkw2AjBBqIHAAEErIABBMGpBmIHAAEGoiMAAEM0BAAsgACAAKAJMNgIwQaiBwABBKyAAQTBqQZiBwABBuIjAABDNAQALIAAgACgCTDYCMEGogcAAQSsgAEEwakGYgcAAQciIwAAQzQEACyAAIAAoAkw2AjBBqIHAAEErIABBMGpBmIHAAEHYiMAAEM0BAAsgACAAKAJMNgIwQaiBwABBKyAAQTBqQZiBwABB6IjAABDNAQALIAAgACgCTDYCMEGogcAAQSsgAEEwakGYgcAAQfiIwAAQzQEACyAAIAAoAkw2AjBBqIHAAEErIABBMGpBmIHAAEGIicAAEM0BAAsgACAAKAJMNgIwQaiBwABBKyAAQTBqQZiBwABBmInAABDNAQALIAAgACgCTDYCMEGogcAAQSsgAEEwakGYgcAAQaiJwAAQzQEACyAAIAAoAkw2AjBBqIHAAEErIABBMGpBmIHAAEG4icAAEM0BAAsgACAAKAJMNgIwQaiBwABBKyAAQTBqQZiBwABByInAABDNAQALIAIgACgCUEG0gsAAEI4DAAshACALELUCIAlBEGokACAAJQEgABDYAQuWIAIGfwFvIwBBEGsiBSQAIAVBBGoiBiAAEOgBAn8gBSgCBCEEIwBB0ABrIgAkACAAQQA2AgwgAEEANgIEAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAIAQoAiRBgICAgHhGDQAgACAEQSRqNgIQIABBOGoiAUEFQQFBARCYASAAKAI8IQIgACgCOEEBRg0OIAAoAkAiA0EEakHmicAALQAAOgAAIANB4onAACgAADYAACAAQQU2AhwgACADNgIYIAAgAjYCFCABIABBEGoQxAEgAC0AOEEGRg0BIABBMGogAEHIAGopAwA3AwAgAEEoaiAAQUBrKQMANwMAIAAgACkDODcDICABIABBBGogAEEUaiAAQSBqEH8gAC0AOEEGRg0AIAEQqAILIABBOGoiAUEGQQFBARCYASAAKAI8IQIgACgCOEEBRg0NIAAoAkAiA0EEakHMg8AALwAAOwAAIANByIPAACgAADYAACAAIAM2AhggACACNgIUIABBBjYCHCABIAQQXCAALQA4QQZGDQEgAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZHBEAgARCoAgsgAEE4aiIBQQhBAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ0gACgCQCIDQuTC0YvGrp645QA3AAAgAEEINgIcIAAgAzYCGCAAIAI2AhQgASAEQfgAahDGASAALQA4QQZGDQIgAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZHBEAgARCoAgsCQCAEKAIwQYCAgIB4Rg0AIAAgBEEwajYCECAAQThqIgFBC0EBQQEQmAEgACgCPCECIAAoAjhBAUYNDiAAKAJAIgNBB2pByoTAACgAADYAACADQcOEwAApAAA3AAAgAEELNgIcIAAgAzYCGCAAIAI2AhQgASAAQRBqEMQBIAAtADhBBkYNBCAAQTBqIABByABqKQMANwMAIABBKGogAEFAaykDADcDACAAIAApAzg3AyAgASAAQQRqIABBFGogAEEgahB/IAAtADhBBkYNACABEKgCCyAAQThqIgNBCkEBQQEQmAEgACgCPCECIAAoAjhBAUYNDSAAKAJAIgFBCGpB94nAAC8AADsAACABQe+JwAApAAA3AAAgAEEKNgIcIAAgATYCGCAAIAI2AhQgAEEBOgA4IAAgBC0AqAE6ADkgAEEgaiICIABBBGogAEEUaiADEH8gAC0AIEEGRwRAIAIQqAILAkAgBCgCPEGAgICAeEYNACAAIARBPGo2AhAgAEE4aiIBQQlBAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ4gACgCQCIDQQhqQYGKwAAtAAA6AAAgA0H5icAAKQAANwAAIABBCTYCHCAAIAM2AhggACACNgIUIAEgAEEQahDEASAALQA4QQZGDQUgAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZGDQAgARCoAgsgAEE4aiIBQQhBAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ0gACgCQCIDQufkhYOH7de05AA3AAAgAEEINgIcIAAgAzYCGCAAIAI2AhQgASAEQYQBahDGASAALQA4QQZGDQUgAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZHBEAgARCoAgsgAEE4aiIDQQ5BAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ0gACgCQCIBQQZqQYiKwAApAAA3AAAgAUGCisAAKQAANwAAIABBDjYCHCAAIAE2AhggACACNgIUIABBAToAOCAAIAQtAKkBOgA5IABBIGoiAiAAQQRqIABBFGogAxB/IAAtACBBBkcEQCACEKgCCyAAQThqIgNBDEEBQQEQmAEgACgCPCECIAAoAjhBAUYNDSAAKAJAIgFBCGpBmIrAACgAADYAACABQZCKwAApAAA3AAAgAEEMNgIcIAAgATYCGCAAIAI2AhQgAEEBOgA4IAAgBC0AqgE6ADkgAEEgaiICIABBBGogAEEUaiADEH8gAC0AIEEGRwRAIAIQqAILIABBOGoiA0EKQQFBARCYASAAKAI8IQIgACgCOEEBRg0NIAAoAkAiAUEIakGkisAALwAAOwAAIAFBnIrAACkAADcAACAAQQo2AhwgACABNgIYIAAgAjYCFCAAQQE6ADggACAELQCrAToAOSAAQSBqIgIgAEEEaiAAQRRqIAMQfyAALQAgQQZHBEAgAhCoAgsgAEE4aiIDQQxBAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ0gACgCQCIBQQhqQa6KwAAoAAA2AAAgAUGmisAAKQAANwAAIABBDDYCHCAAIAE2AhggACACNgIUIABBAToAOCAAIAQtAKwBOgA5IABBIGoiAiAAQQRqIABBFGogAxB/IAAtACBBBkcEQCACEKgCCyAAQThqIgNBCUEBQQEQmAEgACgCPCECIAAoAjhBAUYNDSAAKAJAIgFBCGpBuorAAC0AADoAACABQbKKwAApAAA3AAAgAEEJNgIcIAAgATYCGCAAIAI2AhQgAEEBOgA4IAAgBC0ArQE6ADkgAEEgaiICIABBBGogAEEUaiADEH8gAC0AIEEGRwRAIAIQqAILIABBOGoiAUEEQQFBARCYASAAKAI8IQIgACgCOEEBRg0NIAAoAkAiA0HuwrWrBjYAACAAQQQ2AhwgACADNgIYIAAgAjYCFCABIARBkAFqEMYBIAAtADhBBkYNBiAAQTBqIABByABqKQMANwMAIABBKGogAEFAaykDADcDACAAIAApAzg3AyAgASAAQQRqIABBFGogAEEgahB/IAAtADhBBkcEQCABEKgCCwJAIAQoAkhBgICAgHhGDQAgACAEQcgAajYCECAAQThqIgFBDEEBQQEQmAEgACgCPCECIAAoAjhBAUYNDiAAKAJAIgNBCGpB5oLAACgAADYAACADQd6CwAApAAA3AAAgAEEMNgIcIAAgAzYCGCAAIAI2AhQgASAAQRBqEMQBIAAtADhBBkYNCCAAQTBqIABByABqKQMANwMAIABBKGogAEFAaykDADcDACAAIAApAzg3AyAgASAAQQRqIABBFGogAEEgahB/IAAtADhBBkYNACABEKgCCyAAQThqIgFBBkEBQQEQmAEgACgCPCECIAAoAjhBAUYNDSAAKAJAIgNBBGpB1ILAAC8AADsAACADQdCCwAAoAAA2AAAgACADNgIYIAAgAjYCFCAAQQY2AhwgASAEQZwBahDGASAALQA4QQZGDQggAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZHBEAgARCoAgsCQCAEKAJUQYCAgIB4Rg0AIAAgBEHUAGo2AhAgAEE4aiIBQQ1BAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ4gACgCQCIDQQVqQcCKwAApAAA3AAAgA0G7isAAKQAANwAAIABBDTYCHCAAIAM2AhggACACNgIUIAEgAEEQahDEASAALQA4QQZGDQogAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZGDQAgARCoAgsCQCAEKAJgQYCAgIB4Rg0AIAAgBEHgAGo2AhAgAEE4aiIBQQ5BAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ4gACgCQCIDQQZqQc6KwAApAAA3AAAgA0HIisAAKQAANwAAIABBDjYCHCAAIAM2AhggACACNgIUIAEgAEEQahDEASAALQA4QQZGDQsgAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZGDQAgARCoAgsgAEE4aiIBQQlBAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ0gACgCQCIDQQhqQd6KwAAtAAA6AAAgA0HWisAAKQAANwAAIABBCTYCHCAAIAM2AhggACACNgIUIAEgBDQCIBCLAyAALQA4QQZGDQsgAEEwaiAAQcgAaikDADcDACAAQShqIABBQGspAwA3AwAgACAAKQM4NwMgIAEgAEEEaiAAQRRqIABBIGoQfyAALQA4QQZHBEAgARCoAgsCQCAEKAJsQYCAgIB4Rg0AIAAgBEHsAGo2AhAgAEE4aiIBQRpBAUEBEJgBIAAoAjwhAiAAKAI4QQFGDQ4gACgCQCIEQRhqQfeKwAAvAAA7AAAgBEEQakHvisAAKQAANwAAIARBCGpB54rAACkAADcAACAEQd+KwAApAAA3AAAgAEEaNgIcIAAgBDYCGCAAIAI2AhQgASAAQRBqEMQBIAAtADhBBkYNDSAAQTBqIABByABqKQMANwMAIABBKGogAEFAaykDADcDACAAIAApAzg3AyAgASAAQQRqIABBFGogAEEgahB/IAAtADhBBkYNACABEKgCCyAAQcQAaiAAQQxqKAIANgIAIAAgACkCBDcCPCAAQQU6ADggAEE4aiICEEwgAhCoAiAAQdAAaiQADA4LIAAgACgCPDYCIEGogcAAQSsgAEEgakGYgcAAQaiMwAAQzQEACyAAIAAoAjw2AiBBqIHAAEErIABBIGpBmIHAAEG4jMAAEM0BAAsgACAAKAI8NgIgQaiBwABBKyAAQSBqQZiBwABByIzAABDNAQALIAAgACgCPDYCIEGogcAAQSsgAEEgakGYgcAAQdiMwAAQzQEACyAAIAAoAjw2AiBBqIHAAEErIABBIGpBmIHAAEHojMAAEM0BAAsgACAAKAI8NgIgQaiBwABBKyAAQSBqQZiBwABB+IzAABDNAQALIAAgACgCPDYCIEGogcAAQSsgAEEgakGYgcAAQYiNwAAQzQEACyAAIAAoAjw2AiBBqIHAAEErIABBIGpBmIHAAEGYjcAAEM0BAAsgACAAKAI8NgIgQaiBwABBKyAAQSBqQZiBwABBqI3AABDNAQALIAAgACgCPDYCIEGogcAAQSsgAEEgakGYgcAAQbiNwAAQzQEACyAAIAAoAjw2AiBBqIHAAEErIABBIGpBmIHAAEHIjcAAEM0BAAsgACAAKAI8NgIgQaiBwABBKyAAQSBqQZiBwABB2I3AABDNAQALIAAgACgCPDYCIEGogcAAQSsgAEEgakGYgcAAQeiNwAAQzQEACyACIAAoAkBBtILAABCOAwALIQAgBhC0AiAFQRBqJAAgACUBIAAQ2AEL7wECBH8BbyMAQRBrIgIkACACQQRqIgMgABDoASACKAIEIQEjAEHQAGsiACQAIABBIGogARBcIABBADYCRCAAQQA2AjwgAEEFOgA4AkAgAC0AIEEGRwRAIABBGGogAEEwaikDADcDACAAQRBqIABBKGopAwA3AwAgACAAKQMgNwMIIABBOGoQqAIMAQsgAEEYaiAAQcgAaikDADcDACAAQRBqIABBQGspAwA3AwAgACAAKQM4NwMIIABBIGpBBHIQzAELIABBCGoiBBBMIQEgBBCoAiAAQdAAaiQAIAMQtAIgAkEQaiQAIAElASABENgBC2EBAX8CQAJAIABBBGsoAgAiAkF4cSIDQQRBCCACQQNxIgIbIAFqTwRAIAJBACADIAFBJ2pLGw0BIAAQRQwCC0HVycAAQS5BhMrAABCmAgALQZTKwABBLkHEysAAEKYCAAsLDAAgACgCACABEJEDCxgBAW8gACgCACUBIAEgAiUBIAIQ2AEQDQsOACAAKAIAJQEQI0EARwsOACAAKAIAJQEQJEEARwsOACAAKAIAJQEQJUEARwtrAQF/IwBBMGsiAyQAIAMgATYCBCADIAA2AgAgA0ECNgIMIANB0IzBADYCCCADQgI3AhQgAyADQQRqrUKAgICAwAaENwMoIAMgA61CgICAgMAGhDcDICADIANBIGo2AhAgA0EIaiACEMUCAAtrAQF/IwBBMGsiAyQAIAMgATYCBCADIAA2AgAgA0ECNgIMIANBhI3BADYCCCADQgI3AhQgAyADQQRqrUKAgICAwAaENwMoIAMgA61CgICAgMAGhDcDICADIANBIGo2AhAgA0EIaiACEMUCAAsNACAAKAIAQQEgARBiCw8AQYj3wABBKyAAEKYCAAsNACAAKQMAQQEgARBhCw4AIAFBsIfAAEEWEJsDCw4AIAFBlIzAAEEREJsDCw4AIAFB8KLAAEEKEJsDCw4AIAFBhKTAAEEFEJsDCwwAIAAoAgAgARCZAQsNACAAQcSuwAAgARBNCwcAIAAQ0AMLDgAgAUGAsMAAQQUQmwMLCwAgAEEBQQEQ0AELDgAgAUHctsAAQQUQmwMLDQAgAEGAvcAAIAEQTQsLACAAQQhBGBDQAQsNACAAQaDAwAAgARBNCwsAIABBBEEMENABCw0AQcDDwABBGxDYAwALDgBB28PAAEHPABDYAwALCQAgACABEC0ACw0AIABBuMjAACABEE0LDAAgACABKQIANwMACw0AIABBpNfAACABEE0LDgAgAUGc18AAQQUQmwMLGgAgACABQfiOwQAoAgAiAEHMACAAGxEAAAALDQAgAEG8+cAAIAEQTQsKACACIAAgARBKCw4AIAFBmL3AAEEUEJsDCw4AIAFBrL3AAEEMEJsDCw4AIAFBzsLAAEEDEJsDCw4AIAFBuMDAAEEJEJsDCw4AIAFBwcDAAEEIEJsDCwsAIAAoAgAlARAACxYCAW8BfyAAEDEhARB8IgIgASYBIAILFgEBbyAAIAEQMyECEHwiACACJgEgAAsWAQFvIAAgARAhIQIQfCIAIAImASAACwkAIABBADYCAAsIACAAJQEQFwuwBQIHfwFvAkAjAEHQAGsiACQAIABBADYCPCAAQoCAgIAQNwI0IABBxK7AADYCRCAAQqCAgIAONwJIIAAgAEE0aiIGNgJAIwBBMGsiAiQAQQEhBAJAIABBQGsiBUHM0cAAQQwQmwMNACAFKAIEIQcgBSgCACABKAIIIQMgAkEDNgIEIAJBoMjAADYCACACQgM3AgwgAiADrUKAgICAsAmENwMYIAIgA0EMaq1CgICAgMAGhDcDKCACIANBCGqtQoCAgIDABoQ3AyAgAiACQRhqIgM2AgggByACEE0NACADIAEoAgAiAyABKAIEQQxqIgEoAgARAAACfyACKQMYQviCmb2V7sbFuX9RBEBBBCEEIAMgAikDIELtuq22zYXU9eMAUQ0BGgsgAkEYaiADIAEoAgARAABBACEEIAIpAxhCn/WWlNbu7cOhf1INASACKQMgQoC834WrpfibJ1INAUEIIQQgA0EEagsgAyAEaigCACEDKAIAIQEgBUHY0cAAQQIQmwNFBEBBACEEIAUgASADEJsDRQ0BC0EBIQQLIAJBMGokACAERQRAIABBMGoiBCAAQTxqKAIANgIAIAAgACkCNDcDKCAAQShqIgJBuLDAAEHCsMAAEIUCEB0hCRB8IgEgCSYBIABBIGogASUBEB4gAEEYaiAAKAIgIAAoAiQQqwMgAEEQaiAAKAIYIAAoAhwQoAIgBiAAKAIQIAAoAhQQkwMgAiAAKAI4IgMgAyAAKAI8ahCFAiACQcKwwABBxLDAABCFAiAAQcgAaiAEKAIANgIAIAAgACkDKDcDQCAAQQhqIAVBtK7AABDfASAAKAIIIAAoAgwQHyAGENADIAFBhAFPBEAgARDYAQsgAEHQAGokAAwBC0HsrsAAQTcgAEEoakHcrsAAQfCvwAAQzQEACwsHABAwEJ0CCwIACwv+jQEZAEGAgMAACwv//////////wAAEABBmIDAAAvVDy9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTA0L3NyYy9jb252ZXJ0L3NsaWNlcy5ycwAYABAAbwAAACMBAAAOAAAABQAAAAQAAAAEAAAABgAAAGNhbGxlZCBgUmVzdWx0Ojp1bndyYXAoKWAgb24gYW4gYEVycmAgdmFsdWUAAAAAAAQAAAAEAAAABwAAAEVycm9yL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMA6QAQAEoAAAC+AQAAHQAAAHNyYy9ncmFwaC5yc25vZGVpZGdyYXBoX2lkbm9kZWdyb3VwX2lkY29tcGFyZWNvcHkAAABEARAADAAAAGUEAAAUAAAARAEQAAwAAABtBAAAHAAAAEQBEAAMAAAAcwQAABQAAABEARAADAAAAI0EAAANAAAARAEQAAwAAACLBAAADQAAAGNvbmZpZ2F0dGVtcHRlZCB0byB0YWtlIG93bmVyc2hpcCBvZiBSdXN0IHZhbHVlIHdoaWxlIGl0IHdhcyBib3Jyb3dlZFN0YXRpY0dyYXBoTWV0YWF1dGhvcmNhcmRzY2FyZHNfeF9ub2Rlc194X3dpZGdldHNjb2xvcmRlc2NyaXB0aW9uZWRnZXNncmFwaGlkaWNvbmNsYXNzaXNfZWRpdGFibGVpc3Jlc291cmNlanNvbmxkY29udGV4dG5hbWVub2RlZ3JvdXBzbm9kZXNvbnRvbG9neV9pZHB1YmxpY2F0aW9ucmVsYXRhYmxlX3Jlc291cmNlX21vZGVsX2lkc3Jlc291cmNlXzJfcmVzb3VyY2VfY29uc3RyYWludHNyb290c2x1Z3N1YnRpdGxldmVyc2lvbhwCEAAGAAAAIgIQAAUAAAAnAhAAFwAAAD4CEAAFAAAAQwIQAAsAAABOAhAABQAAAFMCEAAHAAAAWgIQAAkAAABjAhAACwAAAG4CEAAKAAAAeAIQAA0AAACFAhAABAAAAIkCEAAKAAAAkwIQAAUAAACYAhAACwAAAKMCEAALAAAArgIQABwAAADKAhAAHwAAAOkCEAAEAAAA7QIQAAQAAADxAhAACAAAAPkCEAAHAAAAc3RydWN0IFN0YXRpY0dyYXBoTWV0YUZhaWxlZCB0byBkZXNlcmlhbGl6ZSBTdGF0aWNHcmFwaE1ldGE6IAAAAMYDEAAnAAAARAEQAAwAAAAfAQAALgAAAEQBEAAMAAAAKAEAAC0AAABEARAADAAAACsBAAAzAAAARAEQAAwAAAAwAQAAKwAAAEQBEAAMAAAAMgEAADEAAABEARAADAAAADsBAAA1AAAARAEQAAwAAAA+AQAALAAAAEQBEAAMAAAARwEAADMAAABEARAADAAAAEoBAAAzAAAARAEQAAwAAABNAQAARAAAAEQBEAAMAAAAUAEAAEcAAABEARAADAAAAFYBAAAsAAAARAEQAAwAAABZAQAAMAAAAEQBEAAMAAAAXAEAAC8AAABTdGF0aWNOb2RlYWxpYXNkYXRhdHlwZWV4cG9ydGFibGVmaWVsZG5hbWVoYXNjdXN0b21hbGlhc2lzX2NvbGxlY3RvcmlzcmVxdWlyZWRpc3NlYXJjaGFibGVpc3RvcG5vZGVvbnRvbG9neWNsYXNzcGFyZW50cHJvcGVydHlzb3J0b3JkZXJzb3VyY2VicmFuY2hwdWJsaWNhdGlvbl9pZAAAAOIEEAAFAAAAyAEQAAYAAADnBBAACAAAAEMCEAALAAAA7wQQAAoAAAD5BBAACQAAAFYBEAAIAAAAAgUQAA4AAAAQBRAADAAAABwFEAAKAAAAJgUQAAwAAAAyBRAACQAAAIUCEAAEAAAAXgEQAAwAAABQARAABgAAADsFEAANAAAASAUQAA4AAABWBRAACQAAAF8FEAAaAAAAc3RydWN0IFN0YXRpY05vZGUAAABEARAADAAAAHoCAAAtAAAARAEQAAwAAAB8AgAAKgAAAEQBEAAMAAAAfQIAACwAAABEARAADAAAAH8CAAAzAAAARAEQAAwAAACDAgAAMQAAAEQBEAAMAAAAhQIAACwAAABEARAADAAAAIsCAAAoAAAARAEQAAwAAACNAgAANAAAAEQBEAAMAAAAjwIAACoAAABEARAADAAAAJECAAA1AAAARAEQAAwAAACUAgAANgAAAEQBEAAMAAAAlgIAAC0AAABEARAADAAAAJgCAABCAAAARmFpbGVkIHRvIHNldCBjb25maWc6IAAA+AYQABYAAABGYWlsZWQgdG8gZGVzZXJpYWxpemUgbm9kZV9iOiAAABgHEAAeAAAARmFpbGVkIHRvIGRlc2VyaWFsaXplIG5vZGVfYTogAABABxAAHgAAAEZhaWxlZCB0byBub3JtYWxpemUgbm9kZV9iOiBoBxAAHAAAAEZhaWxlZCB0byBub3JtYWxpemUgbm9kZV9hOiCMBxAAHAAAAHRvSlNPTnRvSlNPTiBpcyBub3QgYSBmdW5jdGlvbkZhaWxlZCB0byBjYWxsIHRvSlNPTjogAAAAzgcQABcAQfiPwAALfQEAAAAIAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGVfanNvbi0xLjAuMTQ1L3NyYy92YWx1ZS9kZS5ycwAACBAAZwAAAHIAAAAZAEGAkcAACwUBAAAACQBBkJHAAAsFAQAAAAoAQaCRwAALBQEAAAALAEGwkcAACwUBAAAADABBwJHAAAsFAQAAAA0AQdCRwAALBQEAAAAOAEHgkcAACwUBAAAADQBB8JHAAAsFAQAAAA8AQYCSwAALBQEAAAAQAEGQksAAC2MBAAAACQAAAENvdWxkbid0IGRlc2VyaWFsaXplIGk2NCBvciB1NjQgZnJvbSBhIEJpZ0ludCBvdXRzaWRlIGk2NDo6TUlOLi51NjQ6Ok1BWCBib3VuZHMA//////////9oCRAAQYCTwAALowRhdXRob3JjYXJkc2NhcmRzX3hfbm9kZXNfeF93aWRnZXRzY29sb3JkZXNjcmlwdGlvbmVkZ2VzZ3JhcGhpZGljb25jbGFzc2lzX2VkaXRhYmxlaXNyZXNvdXJjZWpzb25sZGNvbnRleHRuYW1lbm9kZWdyb3Vwc25vZGVzb250b2xvZ3lfaWRwdWJsaWNhdGlvbnJlbGF0YWJsZV9yZXNvdXJjZV9tb2RlbF9pZHNyZXNvdXJjZV8yX3Jlc291cmNlX2NvbnN0cmFpbnRzcm9vdHNsdWdzdWJ0aXRsZXZlcnNpb25hbGlhc2NvbmZpZ2RhdGF0eXBlZXhwb3J0YWJsZWZpZWxkbmFtZWdyYXBoX2lkaGFzY3VzdG9tYWxpYXNpc19jb2xsZWN0b3Jpc3JlcXVpcmVkaXNzZWFyY2hhYmxlaXN0b3Bub2Rlbm9kZWdyb3VwX2lkbm9kZWlkb250b2xvZ3ljbGFzc3BhcmVudHByb3BlcnR5c29ydG9yZGVyc291cmNlYnJhbmNocHVibGljYXRpb25faWQvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZV9qc29uLTEuMC4xNDUvc3JjL3ZhbHVlL21vZC5ycwAbCxAAaAAAAHMAAAAKAAAAAAAAAP//////////mAsQAEG0l8AAC98CBAAAAAQAAAARAAAAEgAAABIAAAAAAAAABAAAAAQAAAATAAAAFAAAABQAAAAAAAAABAAAAAQAAAAVAAAAFgAAABYAAABzZXJpYWxpemVfdmFsdWUgY2FsbGVkIGJlZm9yZSBzZXJpYWxpemVfa2V5L2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGVfanNvbi0xLjAuMTQ1L3NyYy92YWx1ZS9zZXIucnMAIwwQAGgAAACqAQAAHwAAACMMEABoAAAATwEAABIAAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zbGljZS5ycwAArAwQAEoAAAC+AQAAHQAAAP//////////CA0QAEGgmsAAC+0CL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAACANEABKAAAAvgEAAB0AAABNYXBBY2Nlc3M6Om5leHRfdmFsdWUgY2FsbGVkIGJlZm9yZSBuZXh0X2tleS9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3NlcmRlX2NvcmUtMS4wLjIyOC9zcmMvZGUvdmFsdWUucnMAqA0QAGcAAABnBQAAGwAAAGludmFsaWQgdmFsdWU6ICwgZXhwZWN0ZWQgAAAgDhAADwAAAC8OEAALAAAAbWlzc2luZyBmaWVsZCBgYEwOEAAPAAAAWw4QAAEAAABkdXBsaWNhdGUgZmllbGQgYAAAAGwOEAARAAAAWw4QAAEAQZidwAAL7QcBAAAACgAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3NsaWNlLnJzAACgDhAASgAAAL4BAAAdAAAAL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAAPwOEABKAAAAqAEAAB8AAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9jb2xsZWN0aW9ucy9idHJlZS9uYXZpZ2F0ZS5ycwBYDxAAXwAAABYCAAAvAAAAWA8QAF8AAAChAAAAJAAAAC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3NlcmRlX2pzb24tMS4wLjE0NS9zcmMvdmFsdWUvbW9kLnJz2A8QAGgAAABzAAAACgAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL21hcC5ycwAAUBAQAFoAAADjAAAAOwAAAGludGVybmFsIGVycm9yOiBlbnRlcmVkIHVucmVhY2hhYmxlIGNvZGVQEBAAWgAAAOYAAAAsAAAAUBAQAFoAAAD6AAAAPwAAAFAQEABaAAAAHwEAAC4AAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zbGljZS5ycwAAFBEQAEoAAAC+AQAAHQAAAGEgc2VxdWVuY2UvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZV9jb3JlLTEuMC4yMjgvc3JjL2RlL2ltcGxzLnJzAAAAehEQAGcAAACKBAAAIgAAAHoREABnAAAAjQQAABwAAABhIG1hcC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL2NvbnNvbGVfZXJyb3JfcGFuaWNfaG9vay0wLjEuNy9zcmMvbGliLnJzAAkSEABuAAAAlQAAAA4AQZClwAALwwQBAAAAGgAAABsAAAAcAAAAT25jZSBpbnN0YW5jZSBoYXMgcHJldmlvdXNseSBiZWVuIHBvaXNvbmVkAACgEhAAKgAAAG9uZS10aW1lIGluaXRpYWxpemF0aW9uIG1heSBub3QgYmUgcGVyZm9ybWVkIHJlY3Vyc2l2ZWx51BIQADgAAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L3N0ZC9zcmMvc3luYy9wb2lzb24vb25jZS5ycwAUExAAUwAAAJsAAAAyAAAAY2FsbGVkIGBPcHRpb246OnVud3JhcF90aHJvdygpYCBvbiBhIGBOb25lYCB2YWx1ZU1hcCBrZXkgaXMgbm90IGEgc3RyaW5nIGFuZCBjYW5ub3QgYmUgYW4gb2JqZWN0IGtleWludmFsaWQgdHlwZTogLCBleHBlY3RlZCAAAADcExAADgAAAOoTEAALAAAAL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9zdGQvc3JjL3RocmVhZC9sb2NhbC5ycwAIFBAATwAAABUBAAAZAAAAL3J1c3RjLzE3MDY3ZTlhYzZkN2VjYjcwZTUwZjkyYzE5NDRlNTQ1MTg4ZDIzNTkvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAAGgUEABKAAAAvgEAAB0AAAAAAAAA///////////IFBAAQeCpwAAL+QQBAAAAAAAAACBjYW4ndCBiZSByZXByZXNlbnRlZCBhcyBhIEphdmFTY3JpcHQgbnVtYmVyAQAAAAAAAADoFBAALAAAACEAAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZS13YXNtLWJpbmRnZW4tMC42LjUvc3JjL2xpYi5ycygVEABoAAAANQAAAA4AAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9qcy1zeXMtMC4zLjgxL3NyYy9saWIucnMAAACgFRAAXQAAAOwZAAABAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvd2FzbS1iaW5kZ2VuLTAuMi4xMDQvc3JjL2NvbnZlcnQvc2xpY2VzLnJzABAWEABvAAAAIwEAAA4AAABjbG9zdXJlIGludm9rZWQgcmVjdXJzaXZlbHkgb3IgYWZ0ZXIgYmVpbmcgZHJvcHBlZC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTA0L3NyYy9jb252ZXJ0L3NsaWNlcy5ycwAAAMIWEABvAAAAIwEAAA4AAAAvAAAADAAAAAQAAAAwAAAAMQAAADIAQeSuwAAL0QYBAAAAMwAAAGEgRGlzcGxheSBpbXBsZW1lbnRhdGlvbiByZXR1cm5lZCBhbiBlcnJvciB1bmV4cGVjdGVkbHkvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAAKMXEABLAAAA0QoAAA4AAABFcnJvcgAAAKMXEABLAAAAfwUAABoAAACjFxAASwAAAH0FAAAbAAAAoxcQAEsAAABYBAAAEgAAAAoKU3RhY2s6CgoKCi9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL21hcC9lbnRyeS5yc0QYEABgAAAAoQEAAC4AAABhc3NlcnRpb24gZmFpbGVkOiBpZHggPCBDQVBBQ0lUWS9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25vZGUucnMA1BgQAFsAAACVAgAACQAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYuaGVpZ2h0IC0gMdQYEABbAAAArQIAAAkAAADUGBAAWwAAALECAAAJAAAAYXNzZXJ0aW9uIGZhaWxlZDogc3JjLmxlbigpID09IGRzdC5sZW4oKdQYEABbAAAASgcAAAUAAADUGBAAWwAAAMcEAAAjAAAA1BgQAFsAAAAKBQAAJAAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYubm9kZS5oZWlnaHQgLSAxAAAA1BgQAFsAAAD6AwAACQAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25hdmlnYXRlLnJzADAaEABfAAAAWAIAADAAAAA2AAAADAAAAAQAAAA3AAAAOAAAADkAQcC1wAAL8SIBAAAAOgAAAGEgRGlzcGxheSBpbXBsZW1lbnRhdGlvbiByZXR1cm5lZCBhbiBlcnJvciB1bmV4cGVjdGVkbHkvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAAP8aEABLAAAA0QoAAA4AAABFcnJvcgAAAP8aEABLAAAAfwUAABoAAAD/GhAASwAAAH0FAAAbAAAA/xoQAEsAAABYBAAAEgAAAEVPRiB3aGlsZSBwYXJzaW5nIGEgbGlzdEVPRiB3aGlsZSBwYXJzaW5nIGFuIG9iamVjdEVPRiB3aGlsZSBwYXJzaW5nIGEgc3RyaW5nRU9GIHdoaWxlIHBhcnNpbmcgYSB2YWx1ZWV4cGVjdGVkIGA6YGV4cGVjdGVkIGAsYCBvciBgXWBleHBlY3RlZCBgLGAgb3IgYH1gZXhwZWN0ZWQgaWRlbnRleHBlY3RlZCB2YWx1ZWV4cGVjdGVkIGAiYGludmFsaWQgZXNjYXBlaW52YWxpZCBudW1iZXJudW1iZXIgb3V0IG9mIHJhbmdlaW52YWxpZCB1bmljb2RlIGNvZGUgcG9pbnRjb250cm9sIGNoYXJhY3RlciAoXHUwMDAwLVx1MDAxRikgZm91bmQgd2hpbGUgcGFyc2luZyBhIHN0cmluZ2tleSBtdXN0IGJlIGEgc3RyaW5naW52YWxpZCB2YWx1ZTogZXhwZWN0ZWQga2V5IHRvIGJlIGEgbnVtYmVyIGluIHF1b3Rlc2Zsb2F0IGtleSBtdXN0IGJlIGZpbml0ZSAoZ290IE5hTiBvciArLy1pbmYpbG9uZSBsZWFkaW5nIHN1cnJvZ2F0ZSBpbiBoZXggZXNjYXBldHJhaWxpbmcgY29tbWF0cmFpbGluZyBjaGFyYWN0ZXJzdW5leHBlY3RlZCBlbmQgb2YgaGV4IGVzY2FwZXJlY3Vyc2lvbiBsaW1pdCBleGNlZWRlZEVycm9yKCwgbGluZTogLCBjb2x1bW46ICkAAADIHRAABgAAAM4dEAAIAAAA1h0QAAoAAADgHRAAAQAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3N0cmluZy5ycwAEHhAASwAAAH8FAAAaAAAABB4QAEsAAAB9BQAAGwAAAAQeEABLAAAAWAQAABIAAAA7AAAADAAAAAQAAAA8AAAAPQAAADkAAABhbnkgdmFsaWQgSlNPTiB2YWx1ZWEgc3RyaW5nIGtleS9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25hdmlnYXRlLnJzALgeEABfAAAAxgAAACcAAAC4HhAAXwAAABYCAAAvAAAAuB4QAF8AAAChAAAAJAAAAC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3NlcmRlX2pzb24tMS4wLjE0NS9zcmMvdmFsdWUvc2VyLnJzSB8QAGgAAADrAAAAEgAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJzwB8QAFAAAAAuAgAAEQAAAAAAAAAIAAAABAAAAEMAAABEAAAARQAAAGEgYm9vbGVhbmEgc3RyaW5nYnl0ZSBhcnJheWJvb2xlYW4gYGAAAABTIBAACQAAAFwgEAABAAAAaW50ZWdlciBgAAAAcCAQAAkAAABcIBAAAQAAAGZsb2F0aW5nIHBvaW50IGCMIBAAEAAAAFwgEAABAAAAY2hhcmFjdGVyIGAArCAQAAsAAABcIBAAAQAAAHN0cmluZyAAyCAQAAcAAAB1bml0IHZhbHVlT3B0aW9uIHZhbHVlbmV3dHlwZSBzdHJ1Y3RzZXF1ZW5jZW1hcGVudW11bml0IHZhcmlhbnRuZXd0eXBlIHZhcmlhbnR0dXBsZSB2YXJpYW50c3RydWN0IHZhcmlhbnQAAAABAAAAAAAAAC4waTMyL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvd2FzbS1iaW5kZ2VuLTAuMi4xMDQvc3JjL2NvbnZlcnQvc2xpY2VzLnJzbnVsbCBwb2ludGVyIHBhc3NlZCB0byBydXN0cmVjdXJzaXZlIHVzZSBvZiBhbiBvYmplY3QgZGV0ZWN0ZWQgd2hpY2ggd291bGQgbGVhZCB0byB1bnNhZmUgYWxpYXNpbmcgaW4gcnVzdAAAUSEQAG8AAADnAAAAAQAAAC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTA0L3NyYy9leHRlcm5yZWYucnMAADwiEABqAAAAfgAAABEAAAA8IhAAagAAAIsAAAARAAAASnNWYWx1ZSgpAAAAyCIQAAgAAADQIhAAAQAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJz5CIQAFAAAAAuAgAAEQAAAGxpYnJhcnkvc3RkL3NyYy9wYW5pY2tpbmcucnMvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAYCMQAEsAAAB9BQAAGwAAAC9ydXN0Yy8xNzA2N2U5YWM2ZDdlY2I3MGU1MGY5MmMxOTQ0ZTU0NTE4OGQyMzU5L2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJzvCMQAFAAAAAuAgAAEQAAADoAAAABAAAAAAAAABwkEAABAAAAHCQQAAEAAABNAAAADAAAAAQAAABOAAAATwAAAFAAAAAvcnVzdGMvMTcwNjdlOWFjNmQ3ZWNiNzBlNTBmOTJjMTk0NGU1NDUxODhkMjM1OS9saWJyYXJ5L2FsbG9jL3NyYy9zbGljZS5ycwAAUCQQAEoAAAC+AQAAHQAAAC9ydXN0L2RlcHMvZGxtYWxsb2MtMC4yLjcvc3JjL2RsbWFsbG9jLnJzYXNzZXJ0aW9uIGZhaWxlZDogcHNpemUgPj0gc2l6ZSArIG1pbl9vdmVyaGVhZACsJBAAKQAAAKgEAAAJAAAAYXNzZXJ0aW9uIGZhaWxlZDogcHNpemUgPD0gc2l6ZSArIG1heF9vdmVyaGVhZAAArCQQACkAAACuBAAADQAAAEFjY2Vzc0Vycm9yY2Fubm90IGFjY2VzcyBhIFRocmVhZCBMb2NhbCBTdG9yYWdlIHZhbHVlIGR1cmluZyBvciBhZnRlciBkZXN0cnVjdGlvbjogAF8lEABIAAAAAQAAAAAAAABlbnRpdHkgbm90IGZvdW5kcGVybWlzc2lvbiBkZW5pZWRjb25uZWN0aW9uIHJlZnVzZWRjb25uZWN0aW9uIHJlc2V0aG9zdCB1bnJlYWNoYWJsZW5ldHdvcmsgdW5yZWFjaGFibGVjb25uZWN0aW9uIGFib3J0ZWRub3QgY29ubmVjdGVkYWRkcmVzcyBpbiB1c2VhZGRyZXNzIG5vdCBhdmFpbGFibGVuZXR3b3JrIGRvd25icm9rZW4gcGlwZWVudGl0eSBhbHJlYWR5IGV4aXN0c29wZXJhdGlvbiB3b3VsZCBibG9ja25vdCBhIGRpcmVjdG9yeWlzIGEgZGlyZWN0b3J5ZGlyZWN0b3J5IG5vdCBlbXB0eXJlYWQtb25seSBmaWxlc3lzdGVtIG9yIHN0b3JhZ2UgbWVkaXVtZmlsZXN5c3RlbSBsb29wIG9yIGluZGlyZWN0aW9uIGxpbWl0IChlLmcuIHN5bWxpbmsgbG9vcClzdGFsZSBuZXR3b3JrIGZpbGUgaGFuZGxlaW52YWxpZCBpbnB1dCBwYXJhbWV0ZXJpbnZhbGlkIGRhdGF0aW1lZCBvdXR3cml0ZSB6ZXJvbm8gc3RvcmFnZSBzcGFjZXNlZWsgb24gdW5zZWVrYWJsZSBmaWxlcXVvdGEgZXhjZWVkZWRmaWxlIHRvbyBsYXJnZXJlc291cmNlIGJ1c3lleGVjdXRhYmxlIGZpbGUgYnVzeWRlYWRsb2NrY3Jvc3MtZGV2aWNlIGxpbmsgb3IgcmVuYW1ldG9vIG1hbnkgbGlua3NpbnZhbGlkIGZpbGVuYW1lYXJndW1lbnQgbGlzdCB0b28gbG9uZ29wZXJhdGlvbiBpbnRlcnJ1cHRlZHVuc3VwcG9ydGVkdW5leHBlY3RlZCBlbmQgb2YgZmlsZW91dCBvZiBtZW1vcnlpbiBwcm9ncmVzc290aGVyIGVycm9ydW5jYXRlZ29yaXplZCBlcnJvciAob3MgZXJyb3IgKQAAAAEAAAAAAAAApSgQAAsAAACwKBAAAQAAAHBhbmlja2VkIGF0IDoKbWVtb3J5IGFsbG9jYXRpb24gb2YgIGJ5dGVzIGZhaWxlZNooEAAVAAAA7ygQAA0AAABsaWJyYXJ5L3N0ZC9zcmMvYWxsb2MucnMMKRAAGAAAAGQBAAAJAAAAY2Fubm90IG1vZGlmeSB0aGUgcGFuaWMgaG9vayBmcm9tIGEgcGFuaWNraW5nIHRocmVhZDQpEAA0AAAARCMQABwAAACQAAAACQAAAE0AAAAMAAAABAAAAFEAAAAAAAAACAAAAAQAAABSAAAAAAAAAAgAAAAEAAAAUwAAAFQAAABVAAAAVgAAAFcAAAAQAAAABAAAAFgAAABZAAAAWgAAAFsAAABvcGVyYXRpb24gc3VjY2Vzc2Z1bBAAAAARAAAAEgAAABAAAAAQAAAAEwAAABIAAAANAAAADgAAABUAAAAMAAAACwAAABUAAAAVAAAADwAAAA4AAAATAAAAJgAAADgAAAAZAAAAFwAAAAwAAAAJAAAACgAAABAAAAAXAAAADgAAAA4AAAANAAAAFAAAAAgAAAAbAAAADgAAABAAAAAWAAAAFQAAAAsAAAAWAAAADQAAAAsAAAALAAAAEwAAALglEADIJRAA2SUQAOslEAD7JRAACyYQAB4mEAAwJhAAPSYQAEsmEABgJhAAbCYQAHcmEACMJhAAoSYQALAmEAC+JhAA0SYQAPcmEAAvJxAASCcQAF8nEABrJxAAdCcQAH4nEACOJxAApScQALMnEADBJxAAzicQAOInEADqJxAABSgQABMoEAAjKBAAOSgQAE4oEABZKBAAbygQAHwoEACHKBAAkigQAEhhc2ggdGFibGUgY2FwYWNpdHkgb3ZlcmZsb3c8KxAAHAAAAC9ydXN0L2RlcHMvaGFzaGJyb3duLTAuMTUuMi9zcmMvcmF3L21vZC5ycwAAYCsQACoAAAAjAAAAKAAAAEVycm9yAAAAXAAAAAwAAAAEAAAAXQAAAF4AAABfAAAAY2FwYWNpdHkgb3ZlcmZsb3cAAAC8KxAAEQAAAGxpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJz2CsQACAAAAAuAgAAEQAAAGxpYnJhcnkvYWxsb2Mvc3JjL3N0cmluZy5ycwAILBAAGwAAAOgBAAAXAEG82MAAC/gSAQAAAGAAAABhIGZvcm1hdHRpbmcgdHJhaXQgaW1wbGVtZW50YXRpb24gcmV0dXJuZWQgYW4gZXJyb3Igd2hlbiB0aGUgdW5kZXJseWluZyBzdHJlYW0gZGlkIG5vdGxpYnJhcnkvYWxsb2Mvc3JjL2ZtdC5ycwAAmiwQABgAAACKAgAADgAAAGxpYnJhcnkvYWxsb2Mvc3JjL3NsaWNlLnJzAADELBAAGgAAAL4BAAAdAAAACCwQABsAAAB9BQAAGwAAAABwAAcALQEBAQIBAgEBSAswFRABZQcCBgICAQQjAR4bWws6CQkBGAQBCQEDAQUrAzsJKhgBIDcBAQEECAQBAwcKAh0BOgEBAQIECAEJAQoCGgECAjkBBAIEAgIDAwEeAgMBCwI5AQQFAQIEARQCFgYBAToBAQIBBAgBBwMKAh4BOwEBAQwBCQEoAQMBNwEBAwUDAQQHAgsCHQE6AQICAQEDAwEEBwILAhwCOQIBAQIECAEJAQoCHQFIAQQBAgMBAQgBUQECBwwIYgECCQsHSQIbAQEBAQE3DgEFAQIFCwEkCQFmBAEGAQICAhkCBAMQBA0BAgIGAQ8BAAMABBwDHQIeAkACAQcIAQILCQEtAwEBdQIiAXYDBAIJAQYD2wICAToBAQcBAQEBAggGCgIBMB8xBDAKBAMmCQwCIAQCBjgBAQIDAQEFOAgCApgDAQ0BBwQBBgEDAsZAAAHDIQADjQFgIAAGaQIABAEKIAJQAgABAwEEARkCBQGXAhoSDQEmCBkLAQEsAzABAgQCAgIBJAFDBgICAgIMAQgBLwEzAQEDAgIFAgEBKgIIAe4BAgEEAQABABAQEAACAAHiAZUFAAMBAgUEKAMEAaUCAARBBQACTwRGCzEEewE2DykBAgIKAzEEAgIHAT0DJAUBCD4BDAI0CQEBCAQCAV8DAgQGAQIBnQEDCBUCOQIBAQEBDAEJAQ4HAwVDAQIGAQECAQEDBAMBAQ4CVQgCAwEBFwFRAQIGAQECAQECAQLrAQIEBgIBAhsCVQgCAQECagEBAQIIZQEBAQIEAQUACQEC9QEKBAQBkAQCAgQBIAooBgIECAEJBgIDLg0BAgAHAQYBAVIWAgcBAgECegYDAQECAQcBAUgCAwEBAQACCwI0BQUDFwEAAQYPAAwDAwAFOwcAAT8EUQELAgACAC4CFwAFAwYICAIHHgSUAwA3BDIIAQ4BFgUBDwAHARECBwECAQVkAaAHAAE9BAAE/gIAB20HAGCA8ABhc3NlcnRpb24gZmFpbGVkOiBlZGVsdGEgPj0gMGxpYnJhcnkvY29yZS9zcmMvbnVtL2RpeV9mbG9hdC5ycwAAAAwwEAAhAAAATAAAAAkAAAAMMBAAIQAAAE4AAAAJAAAAwW/yhiMAAACB76yFW0FtLe4EAAABH2q/ZO04bu2Xp9r0+T/pA08YAAE+lS4Jmd8D/TgVDy/kdCPs9c/TCNwExNqwzbwZfzOmAyYf6U4CAAABfC6YW4fTvnKf2diHLxUSxlDea3BuSs8P2JXVbnGyJrBmxq0kNhUdWtNCPA5U/2PAc1XMF+/5ZfIovFX3x9yA3O1u9M7v3F/3UwUAbGlicmFyeS9jb3JlL3NyYy9udW0vZmx0MmRlYy9zdHJhdGVneS9kcmFnb24ucnNhc3NlcnRpb24gZmFpbGVkOiBkLm1hbnQgPiAwAOwwEAAvAAAAdgAAAAUAAABhc3NlcnRpb24gZmFpbGVkOiBkLm1pbnVzID4gMAAAAOwwEAAvAAAAdwAAAAUAAABhc3NlcnRpb24gZmFpbGVkOiBkLnBsdXMgPiAw7DAQAC8AAAB4AAAABQAAAGFzc2VydGlvbiBmYWlsZWQ6IGJ1Zi5sZW4oKSA+PSBNQVhfU0lHX0RJR0lUUwAAAOwwEAAvAAAAewAAAAUAAADsMBAALwAAAMIAAAAJAAAA7DAQAC8AAAD7AAAADQAAAOwwEAAvAAAAAgEAABIAAABhc3NlcnRpb24gZmFpbGVkOiBkLm1hbnQuY2hlY2tlZF9zdWIoZC5taW51cykuaXNfc29tZSgpAOwwEAAvAAAAegAAAAUAAABhc3NlcnRpb24gZmFpbGVkOiBkLm1hbnQuY2hlY2tlZF9hZGQoZC5wbHVzKS5pc19zb21lKCkAAOwwEAAvAAAAeQAAAAUAAADsMBAALwAAAAsBAAAFAAAA7DAQAC8AAAAMAQAABQAAAOwwEAAvAAAADQEAAAUAAADsMBAALwAAAHIBAAAkAAAA7DAQAC8AAAB3AQAALwAAAOwwEAAvAAAAhAEAABIAAADsMBAALwAAAGYBAAANAAAA7DAQAC8AAABMAQAAIgAAAOwwEAAvAAAADwEAAAUAAADsMBAALwAAAA4BAAAFAAAAAAAAAN9FGj0DzxrmwfvM/gAAAADKxprHF/5wq9z71P4AAAAAT9y8vvyxd//2+9z+AAAAAAzWa0HvkVa+Efzk/gAAAAA8/H+QrR/QjSz87P4AAAAAg5pVMShcUdNG/PT+AAAAALXJpq2PrHGdYfz8/gAAAADLi+4jdyKc6nv8BP8AAAAAbVN4QJFJzK6W/Az/AAAAAFfOtl15EjyCsfwU/wAAAAA3VvtNNpQQwsv8HP8AAAAAT5hIOG/qlpDm/CT/AAAAAMc6giXLhXTXAP0s/wAAAAD0l7+Xzc+GoBv9NP8AAAAA5awqF5gKNO81/Tz/AAAAAI6yNSr7ZziyUP1E/wAAAAA7P8bS39TIhGv9TP8AAAAAus3TGidE3cWF/VT/AAAAAJbJJbvOn2uToP1c/wAAAACEpWJ9JGys27r9ZP8AAAAA9tpfDVhmq6PV/Wz/AAAAACbxw96T+OLz7/10/wAAAAC4gP+qqK21tQr+fP8AAAAAi0p8bAVfYocl/oT/AAAAAFMwwTRg/7zJP/6M/wAAAABVJrqRjIVOllr+lP8AAAAAvX4pcCR3+d90/pz/AAAAAI+45bifvd+mj/6k/wAAAACUfXSIz1+p+Kn+rP8AAAAAz5uoj5NwRLnE/rT/AAAAAGsVD7/48AiK3/68/wAAAAC2MTFlVSWwzfn+xP8AAAAArH970MbiP5kU/8z/AAAAAAY7KyrEEFzkLv/U/wAAAADTknNpmSQkqkn/3P8AAAAADsoAg/K1h/1j/+T/AAAAAOsaEZJkCOW8fv/s/wAAAADMiFBvCcy8jJn/9P8AAAAALGUZ4lgXt9Gz//z/AEG+68AACwVAnM7/BABBzOvAAAvlIhCl1Ojo/wwAAAAAAAAAYqzF63itAwAUAAAAAACECZT4eDk/gR4AHAAAAAAAsxUHyXvOl8A4ACQAAAAAAHBc6nvOMn6PUwAsAAAAAABogOmrpDjS1W0ANAAAAAAARSKaFyYnT5+IADwAAAAAACf7xNQxomPtogBEAAAAAACorciMOGXesL0ATAAAAAAA22WrGo4Ix4PYAFQAAAAAAJodcUL5HV3E8gBcAAAAAABY5xumLGlNkg0BZAAAAAAA6o1wGmTuAdonAWwAAAAAAEp375qZo22iQgF0AAAAAACFa320e3gJ8lwBfAAAAAAAdxjdeaHkVLR3AYQAAAAAAMLFm1uShluGkgGMAAAAAAA9XZbIxVM1yKwBlAAAAAAAs6CX+ly0KpXHAZwAAAAAAONfoJm9n0be4QGkAAAAAAAljDnbNMKbpfwBrAAAAAAAXJ+Yo3KaxvYWArQAAAAAAM6+6VRTv9y3MQK8AAAAAADiQSLyF/P8iEwCxAAAAAAApXhc05vOIMxmAswAAAAAAN9TIXvzWhaYgQLUAAAAAAA6MB+X3LWg4psC3AAAAAAAlrPjXFPR2ai2AuQAAAAAADxEp6TZfJv70ALsAAAAAAAQRKSnTEx2u+sC9AAAAAAAGpxAtu+Oq4sGA/wAAAAAACyEV6YQ7x/QIAMEAQAAAAApMZHp5aQQmzsDDAEAAAAAnQycofubEOdVAxQBAAAAACn0O2LZICiscAMcAQAAAACFz6d6XktEgIsDJAEAAAAALd2sA0DkIb+lAywBAAAAAI//RF4vnGeOwAM0AQAAAABBuIycnRcz1NoDPAEAAAAAqRvjtJLbGZ71A0QBAAAAANl337puv5brDwRMAQAAAABsaWJyYXJ5L2NvcmUvc3JjL251bS9mbHQyZGVjL3N0cmF0ZWd5L2dyaXN1LnJzAABYOBAALgAAAH0AAAAVAAAAWDgQAC4AAACpAAAABQAAAFg4EAAuAAAAqgAAAAUAAABYOBAALgAAAKsAAAAFAAAAWDgQAC4AAACuAAAABQAAAGFzc2VydGlvbiBmYWlsZWQ6IGQubWFudCArIGQucGx1cyA8ICgxIDw8IDYxKQAAAFg4EAAuAAAArwAAAAUAAABYOBAALgAAAAoBAAARAAAAWDgQAC4AAAANAQAACQAAAFg4EAAuAAAAQAEAAAkAAABYOBAALgAAAK0AAAAFAAAAWDgQAC4AAACsAAAABQAAAGFzc2VydGlvbiBmYWlsZWQ6ICFidWYuaXNfZW1wdHkoKQAAAFg4EAAuAAAA3AEAAAUAAABhc3NlcnRpb24gZmFpbGVkOiBkLm1hbnQgPCAoMSA8PCA2MSlYOBAALgAAAN0BAAAFAAAAWDgQAC4AAADeAQAABQAAAAEAAAAKAAAAZAAAAOgDAAAQJwAAoIYBAEBCDwCAlpgAAOH1BQDKmjtYOBAALgAAADMCAAARAAAAWDgQAC4AAAA2AgAACQAAAFg4EAAuAAAAbAIAAAkAAABYOBAALgAAAOMCAAAmAAAAWDgQAC4AAADvAgAAJgAAAFg4EAAuAAAAzAIAACYAAABsaWJyYXJ5L2NvcmUvc3JjL251bS9mbHQyZGVjL21vZC5ycwBoOhAAIwAAALsAAAAFAAAAYXNzZXJ0aW9uIGZhaWxlZDogYnVmWzBdID4gYicwJwBoOhAAIwAAALwAAAAFAAAAYXNzZXJ0aW9uIGZhaWxlZDogcGFydHMubGVuKCkgPj0gNAAAaDoQACMAAAC9AAAABQAAAC4wLi0rTmFOaW5mMGFzc2VydGlvbiBmYWlsZWQ6IGJ1Zi5sZW4oKSA+PSBtYXhsZW4AAABoOhAAIwAAAH4CAAANAAAAKS4uMDEyMzQ1Njc4OWFiY2RlZgABAAAAAAAAAEJvcnJvd011dEVycm9yYWxyZWFkeSBib3Jyb3dlZDogbjsQABIAAABjYWxsZWQgYE9wdGlvbjo6dW53cmFwKClgIG9uIGEgYE5vbmVgIHZhbHVlaW5kZXggb3V0IG9mIGJvdW5kczogdGhlIGxlbiBpcyAgYnV0IHRoZSBpbmRleCBpcyAAAACzOxAAIAAAANM7EAASAAAAAAAAAAQAAAAEAAAAZwAAAAAAAAAEAAAABAAAAGgAAAA9PSE9bWF0Y2hlc2Fzc2VydGlvbiBgbGVmdCAgcmlnaHRgIGZhaWxlZAogIGxlZnQ6IAogcmlnaHQ6IAAjPBAAEAAAADM8EAAXAAAASjwQAAkAAAAgcmlnaHRgIGZhaWxlZDogCiAgbGVmdDogAAAAIzwQABAAAABsPBAAEAAAAHw8EAAJAAAASjwQAAkAAAA6IAAAAQAAAAAAAACoPBAAAgAAAAAAAAAMAAAABAAAAGkAAABqAAAAawAAACAgICAsCn0gfSgoCiwweDAwMDEwMjAzMDQwNTA2MDcwODA5MTAxMTEyMTMxNDE1MTYxNzE4MTkyMDIxMjIyMzI0MjUyNjI3MjgyOTMwMzEzMjMzMzQzNTM2MzczODM5NDA0MTQyNDM0NDQ1NDY0NzQ4NDk1MDUxNTI1MzU0NTU1NjU3NTg1OTYwNjE2MjYzNjQ2NTY2Njc2ODY5NzA3MTcyNzM3NDc1NzY3Nzc4Nzk4MDgxODI4Mzg0ODU4Njg3ODg4OTkwOTE5MjkzOTQ5NTk2OTc5ODk5MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMGxpYnJhcnkvY29yZS9zcmMvZm10L21vZC5yc2ZhbHNldHJ1ZQDrPRAAGwAAANgKAAAmAAAA6z0QABsAAADhCgAAGgAAAGxpYnJhcnkvY29yZS9zcmMvc3RyL21vZC5yc1suLi5dYmVnaW4gPD0gZW5kICggPD0gKSB3aGVuIHNsaWNpbmcgYGAAUD4QAA4AAABePhAABAAAAGI+EAAQAAAAcj4QAAEAAABieXRlIGluZGV4ICBpcyBub3QgYSBjaGFyIGJvdW5kYXJ5OyBpdCBpcyBpbnNpZGUgIChieXRlcyApIG9mIGAAlD4QAAsAAACfPhAAJgAAAMU+EAAIAAAAzT4QAAYAAAByPhAAAQAAACBpcyBvdXQgb2YgYm91bmRzIG9mIGAAAJQ+EAALAAAA/D4QABYAAAByPhAAAQAAADA+EAAbAAAAnAEAACwAAABsaWJyYXJ5L2Nv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";
  run = async function() {
    console.warn("Does init need called for Rust except in unbundled browser?");
    await __wbg_init(wasmURL);
    greet();
  };
  AlizarinModel = ResourceInstanceViewModel;
  setCurrentLanguage = setCurrentLanguage$1;
  getCurrentLanguage = getCurrentLanguage$1;
})();
export {
  AlizarinModel,
  GraphManager,
  GraphMutator,
  RDM,
  ResourceModelWrapper,
  WKRM,
  __tla,
  client,
  getCurrentLanguage,
  graphManager,
  interfaces,
  nodeConfig,
  renderers,
  run,
  setCurrentLanguage,
  staticStore,
  staticTypes,
  utils,
  viewModels
};
