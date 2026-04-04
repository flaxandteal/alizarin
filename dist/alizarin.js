var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
let AlizarinModel, CollectionMutator, GraphManager, GraphMutator, RDM, ResourceModelWrapper, WASMResourceModelWrapper, WKRM, client, collectionToSkosXml, collectionsToSkosXml, ensureWasmRdmCache, getCurrentLanguage, getTimingStats, getValueFromPath, getValueFromPathSync, graphManager, initWasm, interfaces, logTimingStats, newWASMResourceInstanceWrapperForResource, nodeConfig, parseSkosXml, parseSkosXmlToCollection, registerExtensionHandler, registerResolvableDatatype, renderers, resetTimingStats, setCurrentLanguage, setWasmURL, slugify, staticStore, staticTypes, index, unregisterResolvableDatatype, utils, version, viewModels, wasmReady;
let __tla = (async () => {
  var _a, _b, _c, _d, _e, _f;
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
  function v35(version2, hash, value, namespace, buf, offset) {
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
    bytes[6] = bytes[6] & 15 | version2;
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
  function getCurrentLanguage$1() {
    return currentLanguage || (typeof navigator != "undefined" && navigator.language || DEFAULT_LANGUAGE$1).slice(0, 2);
  }
  function setCurrentLanguage$1(lang) {
    currentLanguage = lang;
  }
  function slugify$1(name, maxLength = 100, useUnderscoreForHyphen = false) {
    if (!name || typeof name !== "string") {
      name = `${name}`;
    }
    let slug = name.toLowerCase().trim().replace(/[\s_]+/g, "-").replace(/[^a-z0-9-]/g, "").replace(/-+/g, "-").replace(/^-|-$/g, "").slice(0, maxLength);
    if (useUnderscoreForHyphen) {
      slug = slug.replace("-", "_");
    }
    if (name && !slug) {
      throw new Error(`Slugification resulted in empty string for input: ${name}`);
    }
    slug = slug.substr(0, SLUG_LENGTH);
    return slug;
  }
  class AttrPromise extends (_b = Promise, _a = Symbol.toPrimitive, _b) {
    constructor(executor) {
      super(executor);
      __publicField(this, _a);
      const proxy = new Proxy(this, {
        set: (object, keyObj, value) => {
          if (keyObj.toString() == "id") {
            object.id = value;
          }
          object.then((val) => {
            val[keyObj] = value;
            return val;
          });
          return true;
        },
        get: (object, keyObj) => {
          if (keyObj == "then") {
            return object.then.bind(this);
          }
          if (keyObj == "id") {
            return object.id;
          }
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
  const DESCRIPTOR_FUNCTION_ID = "60000000-0000-0000-0000-000000000001";
  async function buildResourceDescriptors(graph, nodes, wrapper) {
    const { StaticResourceDescriptors: StaticResourceDescriptors2 } = await Promise.resolve().then(() => staticTypes);
    const descriptors = StaticResourceDescriptors2.empty();
    let descriptorConfig = void 0;
    if (graph.functions_x_graphs) {
      const descriptorNode = graph.functions_x_graphs.find((node) => node.function_id === DESCRIPTOR_FUNCTION_ID);
      if (descriptorNode) {
        descriptorConfig = descriptorNode.config;
      }
    }
    if (!descriptorConfig) {
      return descriptors;
    }
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
        const retrieved = await wrapper.retrievePseudo(semanticNode.alias || "");
        let semanticValue = retrieved ? await retrieved[0] : null;
        if (Array.isArray(semanticValue)) {
          semanticValue = await semanticValue[0];
          if (semanticValue && typeof semanticValue.getValue === "function") {
            semanticValue = await semanticValue.getValue();
          }
        } else if (semanticValue && semanticValue.inner) {
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
        relevantValues = await Promise.all(relevantNodes.map(async ([name, alias]) => {
          const values = await wrapper.retrievePseudo(alias);
          if (!values || values.length === 0) {
            return [
              name,
              void 0
            ];
          }
          const viewModel = values[0];
          const value = viewModel && typeof viewModel.getValue === "function" ? await viewModel.getValue() : viewModel;
          return [
            name,
            typeof value === "string" ? value : void 0
          ];
        }));
        if (relevantValues) {
          description = relevantValues.reduce((desc, [name, value]) => value ? desc.replace(`<${name}>`, value) : desc, description);
        }
      }
      descriptors[descriptor] = description;
    }
    return descriptors;
  }
  function serializeValuesMap(map) {
    const result = {};
    for (const [key, value] of map.entries()) {
      if (value === void 0) {
        result[key] = null;
      } else if (value === false) {
        result[key] = false;
      } else {
        result[key] = true;
      }
    }
    return result;
  }
  function getValueFromPathSync$1(asset, path) {
    let segments = path.split(".");
    function get(value, key) {
      if (key !== "_" && Array.isArray(value)) {
        const results2 = value;
        const result = results2[Number(key)];
        return result;
      }
      return value[key];
    }
    if (segments[0] == "") {
      segments.shift();
    }
    const results = [];
    let multi = false;
    function descend(headValue, segments2) {
      let segment = segments2.shift();
      while (segment !== void 0 && headValue) {
        if (segment === "*") {
          multi = true;
          return headValue.map((headSubvalue) => {
            return descend(headSubvalue, [
              ...segments2
            ]);
          }).flat();
        } else {
          headValue = get(headValue, segment);
        }
        segment = segments2.shift();
      }
      if (headValue !== void 0) {
        results.push(headValue);
      }
      return segments2;
    }
    segments = descend(asset, segments);
    return segments.length ? void 0 : multi ? results : results[0];
  }
  async function getValueFromPath$1(asset, path) {
    let segments = path.split(".");
    async function get(value, key) {
      if (key !== "_" && Array.isArray(value)) {
        const results2 = await Promise.all(value);
        const result = results2[Number(key)];
        return result;
      }
      return value[key];
    }
    if (segments[0] == "") {
      segments.shift();
    }
    const results = [];
    let multi = false;
    async function descend(headValue, segments2) {
      let segment = segments2.shift();
      while (segment !== void 0 && headValue) {
        if (segment === "*") {
          multi = true;
          return (await Promise.all(headValue.map(async (headSubvalue) => {
            return descend(await headSubvalue, [
              ...segments2
            ]);
          }))).flat();
        } else {
          headValue = await get(await headValue, segment);
        }
        segment = segments2.shift();
      }
      if (headValue !== void 0) {
        results.push(headValue);
      }
      return segments2;
    }
    segments = await descend(asset, segments);
    return segments.length ? void 0 : multi ? results : results[0];
  }
  utils = Object.freeze(Object.defineProperty({
    __proto__: null,
    AttrPromise,
    DESCRIPTOR_FUNCTION_ID,
    buildResourceDescriptors,
    generateUuidv5,
    getCurrentLanguage: getCurrentLanguage$1,
    getValueFromPath: getValueFromPath$1,
    getValueFromPathSync: getValueFromPathSync$1,
    serializeValuesMap,
    setCurrentLanguage: setCurrentLanguage$1,
    slugify: slugify$1
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
  const CLOSURE_DTORS = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((state) => {
    wasm.__wbindgen_export_6.get(state.dtor)(state.a, state.b);
  });
  function makeMutClosure(arg0, arg1, dtor, f2) {
    const state = {
      a: arg0,
      b: arg1,
      cnt: 1,
      dtor
    };
    const real = (...args) => {
      state.cnt++;
      const a = state.a;
      state.a = 0;
      try {
        return f2(a, state.b, ...args);
      } finally {
        if (--state.cnt === 0) {
          wasm.__wbindgen_export_6.get(state.dtor)(a, state.b);
          CLOSURE_DTORS.unregister(state);
        } else {
          state.a = a;
        }
      }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
  }
  function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
      throw new Error(`expected instance of ${klass.name}`);
    }
  }
  function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_4.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
  }
  function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
      result.push(wasm.__wbindgen_export_4.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
  }
  function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
      const add = addToExternrefTable0(array[i]);
      getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
  }
  function newWASMResourceInstanceWrapperForModel(graph_id) {
    const ptr0 = passStringToWasm0(graph_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.newWASMResourceInstanceWrapperForModel(ptr0, len0);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    return WASMResourceInstanceWrapper.__wrap(ret[0]);
  }
  newWASMResourceInstanceWrapperForResource = function(resource) {
    _assertClass(resource, StaticResource);
    const ret = wasm.newWASMResourceInstanceWrapperForResource(resource.__wbg_ptr);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    return WASMResourceInstanceWrapper.__wrap(ret[0]);
  };
  registerExtensionHandler = function(datatype, options) {
    const ptr0 = passStringToWasm0(datatype, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.registerExtensionHandler(ptr0, len0, options);
    if (ret[1]) {
      throw takeFromExternrefTable0(ret[0]);
    }
  };
  function getDefaultResolvableDatatypes() {
    const ret = wasm.getDefaultResolvableDatatypes();
    var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
  }
  function getDefaultConfigKeys() {
    const ret = wasm.getDefaultConfigKeys();
    var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
  }
  function buildAliasToCollectionMap(graph_json, resolvable_datatypes, config_keys) {
    const ptr0 = passStringToWasm0(graph_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(resolvable_datatypes) ? 0 : passArrayJsValueToWasm0(resolvable_datatypes, wasm.__wbindgen_malloc);
    var len1 = WASM_VECTOR_LEN;
    var ptr2 = isLikeNone(config_keys) ? 0 : passArrayJsValueToWasm0(config_keys, wasm.__wbindgen_malloc);
    var len2 = WASM_VECTOR_LEN;
    const ret = wasm.buildAliasToCollectionMap(ptr0, len0, ptr1, len1, ptr2, len2);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
  }
  function findNeededCollections(tree_json, alias_to_collection) {
    const ptr0 = passStringToWasm0(tree_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.findNeededCollections(ptr0, len0, alias_to_collection);
    if (ret[3]) {
      throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v2;
  }
  function getRscvTimings() {
    const ret = wasm.getRscvTimings();
    return ret;
  }
  collectionsToSkosXml = function(collections_js, base_uri) {
    let deferred3_0;
    let deferred3_1;
    try {
      const ptr0 = passStringToWasm0(base_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.collectionsToSkosXml(collections_js, ptr0, len0);
      var ptr2 = ret[0];
      var len2 = ret[1];
      if (ret[3]) {
        ptr2 = 0;
        len2 = 0;
        throw takeFromExternrefTable0(ret[2]);
      }
      deferred3_0 = ptr2;
      deferred3_1 = len2;
      return getStringFromWasm0(ptr2, len2);
    } finally {
      wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
  };
  parseSkosXmlToCollection = function(xml_content, base_uri) {
    const ptr0 = passStringToWasm0(xml_content, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(base_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.parseSkosXmlToCollection(ptr0, len0, ptr1, len1);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
  };
  parseSkosXml = function(xml_content, base_uri) {
    const ptr0 = passStringToWasm0(xml_content, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(base_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.parseSkosXml(ptr0, len0, ptr1, len1);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
  };
  collectionToSkosXml = function(collection_js, base_uri) {
    let deferred3_0;
    let deferred3_1;
    try {
      const ptr0 = passStringToWasm0(base_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.collectionToSkosXml(collection_js, ptr0, len0);
      var ptr2 = ret[0];
      var len2 = ret[1];
      if (ret[3]) {
        ptr2 = 0;
        len2 = 0;
        throw takeFromExternrefTable0(ret[2]);
      }
      deferred3_0 = ptr2;
      deferred3_1 = len2;
      return getStringFromWasm0(ptr2, len2);
    } finally {
      wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
  };
  function __wbg_adapter_14(arg0, arg1, arg2) {
    wasm.closure191_externref_shim(arg0, arg1, arg2);
  }
  function __wbg_adapter_773(arg0, arg1, arg2, arg3) {
    wasm.closure543_externref_shim(arg0, arg1, arg2, arg3);
  }
  typeof FinalizationRegistry === "undefined" ? {} : new FinalizationRegistry((ptr) => wasm.__wbg_exampleedgewrapper_free(ptr >>> 0, 1));
  const ExampleNodegroupWrapperFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_examplenodegroupwrapper_free(ptr >>> 0, 1));
  class ExampleNodegroupWrapper {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(ExampleNodegroupWrapper.prototype);
      obj.__wbg_ptr = ptr;
      ExampleNodegroupWrapperFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      ExampleNodegroupWrapperFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_examplenodegroupwrapper_free(ptr, 0);
    }
    get cardinality() {
      const ret = wasm.examplenodegroupwrapper_get_cardinality(this.__wbg_ptr);
      return ret;
    }
    get nodegroupid() {
      const ret = wasm.examplenodegroupwrapper_get_nodegroupid(this.__wbg_ptr);
      return ret;
    }
    set cardinality(value) {
      wasm.examplenodegroupwrapper_set_cardinality(this.__wbg_ptr, value);
    }
    get parentnodegroup_id() {
      const ret = wasm.examplenodegroupwrapper_get_parentnodegroup_id(this.__wbg_ptr);
      return ret;
    }
    set parentnodegroup_id(value) {
      wasm.examplenodegroupwrapper_set_parentnodegroup_id(this.__wbg_ptr, value);
    }
    constructor(json_data) {
      const ret = wasm.examplenodegroupwrapper_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      ExampleNodegroupWrapperFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.examplenodegroupwrapper_copy(this.__wbg_ptr);
      return ExampleNodegroupWrapper.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.examplenodegroupwrapper_toJSON(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) ExampleNodegroupWrapper.prototype[Symbol.dispose] = ExampleNodegroupWrapper.prototype.free;
  const PseudoListFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_pseudolist_free(ptr >>> 0, 1));
  let PseudoList$1 = class PseudoList2 {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(PseudoList2.prototype);
      obj.__wbg_ptr = ptr;
      PseudoListFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      PseudoListFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_pseudolist_free(ptr, 0);
    }
    get nodeAlias() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudolist_nodeAlias(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    isIterable() {
      const ret = wasm.pseudolist_isIterable(this.__wbg_ptr);
      return ret !== 0;
    }
    get totalValues() {
      const ret = wasm.pseudolist_totalValues(this.__wbg_ptr);
      return ret >>> 0;
    }
    getAllValues() {
      const ret = wasm.pseudolist_getAllValues(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    getValue(value_index) {
      const ret = wasm.pseudolist_getValue(this.__wbg_ptr, value_index);
      return ret === 0 ? void 0 : PseudoValue$1.__wrap(ret);
    }
    get isLoaded() {
      const ret = wasm.pseudolist_isLoaded(this.__wbg_ptr);
      return ret !== 0;
    }
    get isSingle() {
      const ret = wasm.pseudolist_isSingle(this.__wbg_ptr);
      return ret !== 0;
    }
  };
  if (Symbol.dispose) PseudoList$1.prototype[Symbol.dispose] = PseudoList$1.prototype.free;
  const PseudoNodeFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_pseudonode_free(ptr >>> 0, 1));
  class PseudoNode {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(PseudoNode.prototype);
      obj.__wbg_ptr = ptr;
      PseudoNodeFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      PseudoNodeFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_pseudonode_free(ptr, 0);
    }
    get config() {
      const ret = wasm.pseudonode_get_config(this.__wbg_ptr);
      return ret;
    }
    get nodeid() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudonode_get_nodeid(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    isIterable() {
      const ret = wasm.pseudonode_isIterable(this.__wbg_ptr);
      return ret !== 0;
    }
    get datatype() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudonode_get_datatype(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get graphId() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudonode_get_graph_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get isInner() {
      const ret = wasm.pseudonode_get_is_inner(this.__wbg_ptr);
      return ret !== 0;
    }
    get isOuter() {
      const ret = wasm.pseudonode_get_is_outer(this.__wbg_ptr);
      return ret !== 0;
    }
    get fieldname() {
      const ret = wasm.pseudonode_get_fieldname(this.__wbg_ptr);
      return ret;
    }
    get istopnode() {
      const ret = wasm.pseudonode_get_istopnode(this.__wbg_ptr);
      return ret !== 0;
    }
    get sortorder() {
      const ret = wasm.pseudonode_get_sortorder(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    get exportable() {
      const ret = wasm.pseudonode_get_exportable(this.__wbg_ptr);
      return ret !== 0;
    }
    get isrequired() {
      const ret = wasm.pseudonode_get_isrequired(this.__wbg_ptr);
      return ret !== 0;
    }
    get childNodes() {
      const ret = wasm.pseudonode_get_child_nodes(this.__wbg_ptr);
      return ret;
    }
    get description() {
      const ret = wasm.pseudonode_get_description(this.__wbg_ptr);
      return ret;
    }
    get parentNode() {
      const ret = wasm.pseudonode_get_parent_node(this.__wbg_ptr);
      return ret;
    }
    set parentNode(value) {
      wasm.pseudonode_set_parent_node(this.__wbg_ptr, value);
    }
    get isCollector() {
      const ret = wasm.pseudonode_get_is_collector(this.__wbg_ptr);
      return ret !== 0;
    }
    get issearchable() {
      const ret = wasm.pseudonode_get_issearchable(this.__wbg_ptr);
      return ret !== 0;
    }
    get nodegroup_id() {
      const ret = wasm.pseudonode_get_nodegroup_id(this.__wbg_ptr);
      return ret;
    }
    get ontologyclass() {
      const ret = wasm.pseudonode_get_ontologyclass(this.__wbg_ptr);
      return ret;
    }
    get hascustomalias() {
      const ret = wasm.pseudonode_get_hascustomalias(this.__wbg_ptr);
      return ret !== 0;
    }
    get parentproperty() {
      const ret = wasm.pseudonode_get_parentproperty(this.__wbg_ptr);
      return ret;
    }
    getNodePlaceholder() {
      let deferred2_0;
      let deferred2_1;
      try {
        const ret = wasm.pseudonode_getNodePlaceholder(this.__wbg_ptr);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
          ptr1 = 0;
          len1 = 0;
          throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
      } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
      }
    }
    get childNodeAliases() {
      const ret = wasm.pseudonode_get_child_node_aliases(this.__wbg_ptr);
      return ret;
    }
    get sourcebranchpublicationId() {
      const ret = wasm.pseudonode_get_sourcebranchpublication_id(this.__wbg_ptr);
      return ret;
    }
    toJSON() {
      const ret = wasm.pseudonode_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudonode_get_name(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get node() {
      const ret = wasm.pseudonode_get_node(this.__wbg_ptr);
      return ret;
    }
    get size() {
      const ret = wasm.pseudonode_get_size(this.__wbg_ptr);
      return ret >>> 0;
    }
    get alias() {
      const ret = wasm.pseudonode_get_alias(this.__wbg_ptr);
      return ret;
    }
    get inner() {
      const ret = wasm.pseudonode_get_inner(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return PseudoNode.__wrap(ret[0]);
    }
  }
  if (Symbol.dispose) PseudoNode.prototype[Symbol.dispose] = PseudoNode.prototype.free;
  const PseudoValueFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_pseudovalue_free(ptr >>> 0, 1));
  let PseudoValue$1 = class PseudoValue2 {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(PseudoValue2.prototype);
      obj.__wbg_ptr = ptr;
      PseudoValueFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      PseudoValueFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_pseudovalue_free(ptr, 0);
    }
    get exportable() {
      const ret = wasm.pseudovalue_exportable(this.__wbg_ptr);
      return ret !== 0;
    }
    get isrequired() {
      const ret = wasm.pseudovalue_isrequired(this.__wbg_ptr);
      return ret !== 0;
    }
    get nodeAlias() {
      const ret = wasm.pseudovalue_nodeAlias(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set parent(parent) {
      wasm.pseudovalue_set_parent(this.__wbg_ptr, parent);
    }
    get description() {
      const ret = wasm.pseudovalue_description(this.__wbg_ptr);
      return ret;
    }
    get independent() {
      const ret = wasm.pseudovalue_independent(this.__wbg_ptr);
      return ret !== 0;
    }
    isIterable() {
      const ret = wasm.pseudovalue_isIterable(this.__wbg_ptr);
      return ret !== 0;
    }
    toSnapshot() {
      const ret = wasm.pseudovalue_toSnapshot(this.__wbg_ptr);
      return ret;
    }
    get isCollector() {
      const ret = wasm.pseudovalue_isCollector(this.__wbg_ptr);
      return ret !== 0;
    }
    get issearchable() {
      const ret = wasm.pseudovalue_issearchable(this.__wbg_ptr);
      return ret !== 0;
    }
    get nodegroupId() {
      const ret = wasm.pseudovalue_nodegroupId(this.__wbg_ptr);
      return ret;
    }
    updateValue(get_view_model) {
      const ret = wasm.pseudovalue_updateValue(this.__wbg_ptr, get_view_model);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    get valueLoaded() {
      const ret = wasm.pseudovalue_valueLoaded(this.__wbg_ptr);
      return ret;
    }
    hasTileData() {
      const ret = wasm.pseudovalue_hasTileData(this.__wbg_ptr);
      return ret !== 0;
    }
    get ontologyclass() {
      const ret = wasm.pseudovalue_ontologyclass(this.__wbg_ptr);
      return ret;
    }
    setTileData(value) {
      wasm.pseudovalue_setTileData(this.__wbg_ptr, value);
    }
    get hascustomalias() {
      const ret = wasm.pseudovalue_hascustomalias(this.__wbg_ptr);
      return ret !== 0;
    }
    get parentproperty() {
      const ret = wasm.pseudovalue_parentproperty(this.__wbg_ptr);
      return ret;
    }
    get tileDataJson() {
      const ret = wasm.pseudovalue_tileDataJson(this.__wbg_ptr);
      return ret;
    }
    toDisplayValue(rdm_cache, node_config_manager, language) {
      _assertClass(rdm_cache, WasmRdmCache);
      _assertClass(node_config_manager, WasmNodeConfigManager);
      var ptr0 = isLikeNone(language) ? 0 : passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.pseudovalue_toDisplayValue(this.__wbg_ptr, rdm_cache.__wbg_ptr, node_config_manager.__wbg_ptr, ptr0, len0);
      return ret;
    }
    getChildNodeId(index2) {
      const ret = wasm.pseudovalue_getChildNodeId(this.__wbg_ptr, index2);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get childNodeAliases() {
      const ret = wasm.pseudovalue_childNodeAliases(this.__wbg_ptr);
      return ret;
    }
    get childNodeIdsCount() {
      const ret = wasm.pseudovalue_childNodeIdsCount(this.__wbg_ptr);
      return ret >>> 0;
    }
    get name() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudovalue_name(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get node() {
      const ret = wasm.pseudovalue_node(this.__wbg_ptr);
      return StaticNode.__wrap(ret);
    }
    get size() {
      const ret = wasm.pseudovalue_size(this.__wbg_ptr);
      return ret >>> 0;
    }
    get tile() {
      const ret = wasm.pseudovalue_tile(this.__wbg_ptr);
      return ret;
    }
    get alias() {
      const ret = wasm.pseudovalue_alias(this.__wbg_ptr);
      return ret;
    }
    clear() {
      wasm.pseudovalue_clear(this.__wbg_ptr);
    }
    get value() {
      const ret = wasm.pseudovalue_value(this.__wbg_ptr);
      return ret;
    }
    get config() {
      const ret = wasm.pseudovalue_config(this.__wbg_ptr);
      return ret;
    }
    get parent() {
      const ret = wasm.pseudovalue_parent(this.__wbg_ptr);
      return ret;
    }
    get nodeId() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudovalue_nodeId(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get tileId() {
      const ret = wasm.pseudovalue_tileId(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get accessed() {
      const ret = wasm.pseudovalue_accessed(this.__wbg_ptr);
      return ret !== 0;
    }
    get datatype() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudovalue_datatype(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get graphId() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.pseudovalue_graphId(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get hasTile() {
      const ret = wasm.pseudovalue_hasTile(this.__wbg_ptr);
      return ret !== 0;
    }
    get isInner() {
      const ret = wasm.pseudovalue_isInner(this.__wbg_ptr);
      return ret !== 0;
    }
    get isOuter() {
      const ret = wasm.pseudovalue_isOuter(this.__wbg_ptr);
      return ret !== 0;
    }
    set tile(tile) {
      wasm.pseudovalue_set_tile(this.__wbg_ptr, tile);
    }
    get fieldname() {
      const ret = wasm.pseudovalue_fieldname(this.__wbg_ptr);
      return ret;
    }
    get inner() {
      const ret = wasm.pseudovalue_inner(this.__wbg_ptr);
      return ret === 0 ? void 0 : PseudoValue2.__wrap(ret);
    }
    getValue(get_view_model) {
      const ret = wasm.pseudovalue_getValue(this.__wbg_ptr, get_view_model);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    get istopnode() {
      const ret = wasm.pseudovalue_istopnode(this.__wbg_ptr);
      return ret !== 0;
    }
    get sortorder() {
      const ret = wasm.pseudovalue_sortorder(this.__wbg_ptr);
      return ret;
    }
    get tileData() {
      const ret = wasm.pseudovalue_tileData(this.__wbg_ptr);
      return ret;
    }
  };
  if (Symbol.dispose) PseudoValue$1.prototype[Symbol.dispose] = PseudoValue$1.prototype.free;
  const StaticCardFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticcard_free(ptr >>> 0, 1));
  class StaticCard {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticCard.prototype);
      obj.__wbg_ptr = ptr;
      StaticCardFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticCardFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticcard_free(ptr, 0);
    }
    get active() {
      const ret = wasm.staticcard_get_active(this.__wbg_ptr);
      return ret;
    }
    get cardid() {
      const ret = wasm.staticcard_get_cardid(this.__wbg_ptr);
      return ret;
    }
    get config() {
      const ret = wasm.staticcard_get_config(this.__wbg_ptr);
      return ret;
    }
    set active(value) {
      wasm.staticcard_set_active(this.__wbg_ptr, value);
    }
    set cardid(value) {
      wasm.staticcard_set_cardid(this.__wbg_ptr, value);
    }
    set config(value) {
      wasm.staticcard_set_config(this.__wbg_ptr, value);
    }
    get visible() {
      const ret = wasm.staticcard_get_visible(this.__wbg_ptr);
      return ret;
    }
    set visible(value) {
      wasm.staticcard_set_visible(this.__wbg_ptr, value);
    }
    get cssclass() {
      const ret = wasm.staticcard_get_cssclass(this.__wbg_ptr);
      return ret;
    }
    get graph_id() {
      const ret = wasm.staticcard_get_graph_id(this.__wbg_ptr);
      return ret;
    }
    get helptext() {
      const ret = wasm.staticcard_get_helptext(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    set cssclass(value) {
      wasm.staticcard_set_cssclass(this.__wbg_ptr, value);
    }
    set graph_id(value) {
      wasm.staticcard_set_graph_id(this.__wbg_ptr, value);
    }
    set helptext(value) {
      _assertClass(value, StaticTranslatableString);
      var ptr0 = value.__destroy_into_raw();
      wasm.staticcard_set_helptext(this.__wbg_ptr, ptr0);
    }
    get helptitle() {
      const ret = wasm.staticcard_get_helptitle(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    get sortorder() {
      const ret = wasm.staticcard_get_sortorder(this.__wbg_ptr);
      return ret;
    }
    set helptitle(value) {
      _assertClass(value, StaticTranslatableString);
      var ptr0 = value.__destroy_into_raw();
      wasm.staticcard_set_helptitle(this.__wbg_ptr, ptr0);
    }
    set sortorder(value) {
      wasm.staticcard_set_sortorder(this.__wbg_ptr, value);
    }
    get constraints() {
      const ret = wasm.staticcard_get_constraints(this.__wbg_ptr);
      return ret;
    }
    get description() {
      const ret = wasm.staticcard_get_description(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticTranslatableString.__wrap(ret);
    }
    get helpenabled() {
      const ret = wasm.staticcard_get_helpenabled(this.__wbg_ptr);
      return ret;
    }
    get is_editable() {
      const ret = wasm.staticcard_get_is_editable(this.__wbg_ptr);
      return ret;
    }
    set constraints(value) {
      wasm.staticcard_set_constraints(this.__wbg_ptr, value);
    }
    set description(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticTranslatableString);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticcard_set_description(this.__wbg_ptr, ptr0);
    }
    set helpenabled(value) {
      wasm.staticcard_set_helpenabled(this.__wbg_ptr, value);
    }
    set is_editable(value) {
      wasm.staticcard_set_is_editable(this.__wbg_ptr, value);
    }
    get component_id() {
      const ret = wasm.staticcard_get_component_id(this.__wbg_ptr);
      return ret;
    }
    get instructions() {
      const ret = wasm.staticcard_get_instructions(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    get nodegroup_id() {
      const ret = wasm.staticcard_get_nodegroup_id(this.__wbg_ptr);
      return ret;
    }
    set component_id(value) {
      wasm.staticcard_set_component_id(this.__wbg_ptr, value);
    }
    set instructions(value) {
      _assertClass(value, StaticTranslatableString);
      var ptr0 = value.__destroy_into_raw();
      wasm.staticcard_set_instructions(this.__wbg_ptr, ptr0);
    }
    set nodegroup_id(value) {
      wasm.staticcard_set_nodegroup_id(this.__wbg_ptr, value);
    }
    constructor(json_data) {
      const ret = wasm.staticcard_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticCardFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    toJSON() {
      const ret = wasm.staticcard_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      const ret = wasm.staticcard_get_name(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    set name(value) {
      _assertClass(value, StaticTranslatableString);
      var ptr0 = value.__destroy_into_raw();
      wasm.staticcard_set_name(this.__wbg_ptr, ptr0);
    }
  }
  if (Symbol.dispose) StaticCard.prototype[Symbol.dispose] = StaticCard.prototype.free;
  const StaticCardsXNodesXWidgetsFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticcardsxnodesxwidgets_free(ptr >>> 0, 1));
  class StaticCardsXNodesXWidgets {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticCardsXNodesXWidgets.prototype);
      obj.__wbg_ptr = ptr;
      StaticCardsXNodesXWidgetsFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticCardsXNodesXWidgetsFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticcardsxnodesxwidgets_free(ptr, 0);
    }
    get config() {
      const ret = wasm.staticcardsxnodesxwidgets_get_config(this.__wbg_ptr);
      return ret;
    }
    set config(value) {
      wasm.staticcardsxnodesxwidgets_set_config(this.__wbg_ptr, value);
    }
    get card_id() {
      const ret = wasm.staticcardsxnodesxwidgets_get_card_id(this.__wbg_ptr);
      return ret;
    }
    get node_id() {
      const ret = wasm.staticcardsxnodesxwidgets_get_node_id(this.__wbg_ptr);
      return ret;
    }
    get visible() {
      const ret = wasm.staticcardsxnodesxwidgets_get_visible(this.__wbg_ptr);
      return ret;
    }
    set card_id(value) {
      wasm.staticcardsxnodesxwidgets_set_card_id(this.__wbg_ptr, value);
    }
    set node_id(value) {
      wasm.staticcardsxnodesxwidgets_set_node_id(this.__wbg_ptr, value);
    }
    set visible(value) {
      wasm.staticcardsxnodesxwidgets_set_visible(this.__wbg_ptr, value);
    }
    get sortorder() {
      const ret = wasm.staticcardsxnodesxwidgets_get_sortorder(this.__wbg_ptr);
      return ret;
    }
    get widget_id() {
      const ret = wasm.staticcardsxnodesxwidgets_get_widget_id(this.__wbg_ptr);
      return ret;
    }
    set sortorder(value) {
      wasm.staticcardsxnodesxwidgets_set_sortorder(this.__wbg_ptr, value);
    }
    set widget_id(value) {
      wasm.staticcardsxnodesxwidgets_set_widget_id(this.__wbg_ptr, value);
    }
    constructor(json_data) {
      const ret = wasm.staticcardsxnodesxwidgets_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticCardsXNodesXWidgetsFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    get id() {
      const ret = wasm.staticcardsxnodesxwidgets_get_id(this.__wbg_ptr);
      return ret;
    }
    set id(value) {
      wasm.staticcardsxnodesxwidgets_set_id(this.__wbg_ptr, value);
    }
    toJSON() {
      const ret = wasm.staticcardsxnodesxwidgets_toJSON(this.__wbg_ptr);
      return ret;
    }
    get label() {
      const ret = wasm.staticcardsxnodesxwidgets_get_label(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    set label(value) {
      _assertClass(value, StaticTranslatableString);
      var ptr0 = value.__destroy_into_raw();
      wasm.staticcardsxnodesxwidgets_set_label(this.__wbg_ptr, ptr0);
    }
  }
  if (Symbol.dispose) StaticCardsXNodesXWidgets.prototype[Symbol.dispose] = StaticCardsXNodesXWidgets.prototype.free;
  const StaticConstraintFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticconstraint_free(ptr >>> 0, 1));
  class StaticConstraint {
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticConstraintFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticconstraint_free(ptr, 0);
    }
    get card_id() {
      const ret = wasm.staticconstraint_get_card_id(this.__wbg_ptr);
      return ret;
    }
    set card_id(value) {
      wasm.staticconstraint_set_card_id(this.__wbg_ptr, value);
    }
    get constraintid() {
      const ret = wasm.staticconstraint_get_constraintid(this.__wbg_ptr);
      return ret;
    }
    set constraintid(value) {
      wasm.staticconstraint_set_constraintid(this.__wbg_ptr, value);
    }
    get uniquetoallinstances() {
      const ret = wasm.staticconstraint_get_uniquetoallinstances(this.__wbg_ptr);
      return ret;
    }
    set uniquetoallinstances(value) {
      wasm.staticconstraint_set_uniquetoallinstances(this.__wbg_ptr, value);
    }
    constructor(json_data) {
      const ret = wasm.staticconstraint_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticConstraintFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    toJSON() {
      const ret = wasm.staticconstraint_toJSON(this.__wbg_ptr);
      return ret;
    }
    get nodes() {
      const ret = wasm.staticconstraint_get_nodes(this.__wbg_ptr);
      return ret;
    }
    set nodes(value) {
      wasm.staticconstraint_set_nodes(this.__wbg_ptr, value);
    }
  }
  if (Symbol.dispose) StaticConstraint.prototype[Symbol.dispose] = StaticConstraint.prototype.free;
  const StaticEdgeFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticedge_free(ptr >>> 0, 1));
  class StaticEdge {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticEdge.prototype);
      obj.__wbg_ptr = ptr;
      StaticEdgeFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticEdgeFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticedge_free(ptr, 0);
    }
    get edgeid() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticedge_get_edgeid(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get graph_id() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticedge_get_graph_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get description() {
      const ret = wasm.staticedge_get_description(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get rangenode_id() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticedge_get_rangenode_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get domainnode_id() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticedge_get_domainnode_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get ontologyproperty() {
      const ret = wasm.staticedge_get_ontologyproperty(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    constructor(json_data) {
      const ret = wasm.staticedge_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticEdgeFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticedge_copy(this.__wbg_ptr);
      return StaticEdge.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticedge_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      const ret = wasm.staticedge_get_name(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
  }
  if (Symbol.dispose) StaticEdge.prototype[Symbol.dispose] = StaticEdge.prototype.free;
  const StaticFunctionsXGraphsFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticfunctionsxgraphs_free(ptr >>> 0, 1));
  class StaticFunctionsXGraphs {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticFunctionsXGraphs.prototype);
      obj.__wbg_ptr = ptr;
      StaticFunctionsXGraphsFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticFunctionsXGraphsFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticfunctionsxgraphs_free(ptr, 0);
    }
    get config() {
      const ret = wasm.staticfunctionsxgraphs_get_config(this.__wbg_ptr);
      return ret;
    }
    set config(value) {
      wasm.staticfunctionsxgraphs_set_config(this.__wbg_ptr, value);
    }
    get graph_id() {
      const ret = wasm.staticfunctionsxgraphs_get_graph_id(this.__wbg_ptr);
      return ret;
    }
    set graph_id(value) {
      wasm.staticfunctionsxgraphs_set_graph_id(this.__wbg_ptr, value);
    }
    get function_id() {
      const ret = wasm.staticfunctionsxgraphs_get_function_id(this.__wbg_ptr);
      return ret;
    }
    set function_id(value) {
      wasm.staticfunctionsxgraphs_set_function_id(this.__wbg_ptr, value);
    }
    constructor(json_data) {
      const ret = wasm.staticfunctionsxgraphs_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticFunctionsXGraphsFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticfunctionsxgraphs_copy(this.__wbg_ptr);
      return StaticFunctionsXGraphs.__wrap(ret);
    }
    get id() {
      const ret = wasm.staticfunctionsxgraphs_get_id(this.__wbg_ptr);
      return ret;
    }
    set id(value) {
      wasm.staticfunctionsxgraphs_set_id(this.__wbg_ptr, value);
    }
    toJSON() {
      const ret = wasm.staticfunctionsxgraphs_toJSON(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) StaticFunctionsXGraphs.prototype[Symbol.dispose] = StaticFunctionsXGraphs.prototype.free;
  const StaticGraphFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticgraph_free(ptr >>> 0, 1));
  class StaticGraph {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticGraph.prototype);
      obj.__wbg_ptr = ptr;
      StaticGraphFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticGraphFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticgraph_free(ptr, 0);
    }
    get author() {
      const ret = wasm.staticgraph_get_author(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get config() {
      const ret = wasm.staticgraph_get_config(this.__wbg_ptr);
      return ret;
    }
    getSchema() {
      const ret = wasm.staticgraph_getSchema(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    get graphid() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticgraph_get_graphid(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get version() {
      const ret = wasm.staticgraph_get_version(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get subtitle() {
      const ret = wasm.staticgraph_get_subtitle(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    get iconclass() {
      const ret = wasm.staticgraph_get_iconclass(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get isresource() {
      const ret = wasm.staticgraph_get_isresource(this.__wbg_ptr);
      return ret === 16777215 ? void 0 : ret !== 0;
    }
    get nodegroups() {
      const ret = wasm.staticgraph_get_nodegroups(this.__wbg_ptr);
      return ret;
    }
    pushNodegroup(nodegroup) {
      _assertClass(nodegroup, StaticNodegroup);
      var ptr0 = nodegroup.__destroy_into_raw();
      wasm.staticgraph_pushNodegroup(this.__wbg_ptr, ptr0);
    }
    set nodegroups(value) {
      wasm.staticgraph_set_nodegroups(this.__wbg_ptr, value);
    }
    get description() {
      const ret = wasm.staticgraph_get_description(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    get is_editable() {
      const ret = wasm.staticgraph_get_is_editable(this.__wbg_ptr);
      return ret;
    }
    get ontology_id() {
      const ret = wasm.staticgraph_get_ontology_id(this.__wbg_ptr);
      return ret;
    }
    get template_id() {
      const ret = wasm.staticgraph_get_template_id(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    static fromJsonString(json_str) {
      const ptr0 = passStringToWasm0(json_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticgraph_fromJsonString(ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticGraph.__wrap(ret[0]);
    }
    get jsonldcontext() {
      const ret = wasm.staticgraph_get_jsonldcontext(this.__wbg_ptr);
      return ret;
    }
    getNodeById(id) {
      var ptr0 = isLikeNone(id) ? 0 : passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticgraph_getNodeById(this.__wbg_ptr, ptr0, len0);
      return ret;
    }
    get deploymentdate() {
      const ret = wasm.staticgraph_get_deploymentdate(this.__wbg_ptr);
      return ret;
    }
    get deploymentfile() {
      const ret = wasm.staticgraph_get_deploymentfile(this.__wbg_ptr);
      return ret;
    }
    getNodeByAlias(alias) {
      var ptr0 = isLikeNone(alias) ? 0 : passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticgraph_getNodeByAlias(this.__wbg_ptr, ptr0, len0);
      return ret;
    }
    setDescriptorTemplate(descriptor_type, string_template) {
      const ptr0 = passStringToWasm0(descriptor_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(string_template, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      const ret = wasm.staticgraph_setDescriptorTemplate(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    pushCardXNodeXWidget(cxnxw) {
      _assertClass(cxnxw, StaticCardsXNodesXWidgets);
      var ptr0 = cxnxw.__destroy_into_raw();
      wasm.staticgraph_pushCardXNodeXWidget(this.__wbg_ptr, ptr0);
    }
    get cards_x_nodes_x_widgets() {
      const ret = wasm.staticgraph_get_cards_x_nodes_x_widgets(this.__wbg_ptr);
      return ret;
    }
    set cards_x_nodes_x_widgets(value) {
      wasm.staticgraph_set_cards_x_nodes_x_widgets(this.__wbg_ptr, value);
    }
    get relatable_resource_model_ids() {
      const ret = wasm.staticgraph_get_relatable_resource_model_ids(this.__wbg_ptr);
      return ret;
    }
    get resource_2_resource_constraints() {
      const ret = wasm.staticgraph_get_resource_2_resource_constraints(this.__wbg_ptr);
      return ret;
    }
    constructor(json_data) {
      const ret = wasm.staticgraph_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticGraphFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticgraph_copy(this.__wbg_ptr);
      return StaticGraph.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticgraph_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      const ret = wasm.staticgraph_get_name(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    get root() {
      const ret = wasm.staticgraph_get_root(this.__wbg_ptr);
      return StaticNode.__wrap(ret);
    }
    get slug() {
      const ret = wasm.staticgraph_get_slug(this.__wbg_ptr);
      return ret;
    }
    get cards() {
      const ret = wasm.staticgraph_get_cards(this.__wbg_ptr);
      return ret;
    }
    get color() {
      const ret = wasm.staticgraph_get_color(this.__wbg_ptr);
      return ret;
    }
    get edges() {
      const ret = wasm.staticgraph_get_edges(this.__wbg_ptr);
      return ret;
    }
    get nodes() {
      const ret = wasm.staticgraph_get_nodes(this.__wbg_ptr);
      return ret;
    }
    pushCard(card) {
      _assertClass(card, StaticCard);
      var ptr0 = card.__destroy_into_raw();
      wasm.staticgraph_pushCard(this.__wbg_ptr, ptr0);
    }
    pushEdge(edge) {
      _assertClass(edge, StaticEdge);
      var ptr0 = edge.__destroy_into_raw();
      wasm.staticgraph_pushEdge(this.__wbg_ptr, ptr0);
    }
    pushNode(node) {
      _assertClass(node, StaticNode);
      var ptr0 = node.__destroy_into_raw();
      wasm.staticgraph_pushNode(this.__wbg_ptr, ptr0);
    }
    set cards(value) {
      wasm.staticgraph_set_cards(this.__wbg_ptr, value);
    }
    set edges(value) {
      wasm.staticgraph_set_edges(this.__wbg_ptr, value);
    }
    set nodes(value) {
      wasm.staticgraph_set_nodes(this.__wbg_ptr, value);
    }
  }
  if (Symbol.dispose) StaticGraph.prototype[Symbol.dispose] = StaticGraph.prototype.free;
  const StaticGraphMetaFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticgraphmeta_free(ptr >>> 0, 1));
  class StaticGraphMeta {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticGraphMeta.prototype);
      obj.__wbg_ptr = ptr;
      StaticGraphMetaFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
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
    getVersion() {
      const ret = wasm.staticgraphmeta_getVersion(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    set graphid(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_set_graphid(this.__wbg_ptr, ptr0, len0);
    }
    setVersion(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setVersion(this.__wbg_ptr, ptr0, len0);
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
    getIsResource() {
      const ret = wasm.staticgraphmeta_getIsResource(this.__wbg_ptr);
      return ret === 16777215 ? void 0 : ret !== 0;
    }
    getNodeGroups() {
      const ret = wasm.staticgraphmeta_getNodeGroups(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setIsResource(value) {
      wasm.staticgraphmeta_setIsResource(this.__wbg_ptr, isLikeNone(value) ? 16777215 : value ? 1 : 0);
    }
    setNodeGroups(value) {
      wasm.staticgraphmeta_setNodeGroups(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    get description() {
      const ret = wasm.staticgraphmeta_get_description(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticTranslatableString.__wrap(ret);
    }
    getIsEditable() {
      const ret = wasm.staticgraphmeta_getIsEditable(this.__wbg_ptr);
      return ret === 16777215 ? void 0 : ret !== 0;
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
    getPublication() {
      const ret = wasm.staticgraphmeta_getPublication(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    set description(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticTranslatableString);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticgraphmeta_set_description(this.__wbg_ptr, ptr0);
    }
    setIsEditable(value) {
      wasm.staticgraphmeta_setIsEditable(this.__wbg_ptr, isLikeNone(value) ? 16777215 : value ? 1 : 0);
    }
    setOntologyId(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setOntologyId(this.__wbg_ptr, ptr0, len0);
    }
    setPublication(value) {
      const ret = wasm.staticgraphmeta_setPublication(this.__wbg_ptr, value);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
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
    get author() {
      const ret = wasm.staticgraphmeta_get_author_property(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get isresource() {
      const ret = wasm.staticgraphmeta_getIsResource(this.__wbg_ptr);
      return ret === 16777215 ? void 0 : ret !== 0;
    }
    getCardsXNodesXWidgets() {
      const ret = wasm.staticgraphmeta_getCardsXNodesXWidgets(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setCardsXNodesXWidgets(value) {
      wasm.staticgraphmeta_setCardsXNodesXWidgets(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
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
    constructor(json_data) {
      const ret = wasm.staticgraphmeta_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticGraphMetaFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    toJSON() {
      const ret = wasm.staticgraphmeta_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      const ret = wasm.staticgraphmeta_get_name(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticTranslatableString.__wrap(ret);
    }
    getRoot() {
      const ret = wasm.staticgraphmeta_getRoot(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticNode.__wrap(ret);
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
    set name(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticTranslatableString);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticgraphmeta_set_name(this.__wbg_ptr, ptr0);
    }
    setRoot(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticNode);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticgraphmeta_setRoot(this.__wbg_ptr, ptr0);
    }
    set slug(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_set_slug(this.__wbg_ptr, ptr0, len0);
    }
    getCards() {
      const ret = wasm.staticgraphmeta_getCards(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
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
    getEdges() {
      const ret = wasm.staticgraphmeta_getEdges(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    getNodes() {
      const ret = wasm.staticgraphmeta_getNodes(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    setCards(value) {
      wasm.staticgraphmeta_setCards(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    setColor(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticgraphmeta_setColor(this.__wbg_ptr, ptr0, len0);
    }
    setEdges(value) {
      wasm.staticgraphmeta_setEdges(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
    }
    setNodes(value) {
      wasm.staticgraphmeta_setNodes(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >>> 0);
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
    getConfig() {
      const ret = wasm.staticnode_getConfig(this.__wbg_ptr);
      return ret;
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
    setConfig(value) {
      wasm.staticnode_setConfig(this.__wbg_ptr, value);
    }
    set nodeid(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_nodeid(this.__wbg_ptr, ptr0, len0);
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
    set datatype(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_datatype(this.__wbg_ptr, ptr0, len0);
    }
    set graph_id(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_graph_id(this.__wbg_ptr, ptr0, len0);
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
    get istopnode() {
      const ret = wasm.staticnode_get_istopnode(this.__wbg_ptr);
      return ret !== 0;
    }
    get sortorder() {
      const ret = wasm.staticnode_get_sortorder(this.__wbg_ptr);
      return ret === 4294967297 ? void 0 : ret;
    }
    set fieldname(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_fieldname(this.__wbg_ptr, ptr0, len0);
    }
    set istopnode(value) {
      wasm.staticnode_set_istopnode(this.__wbg_ptr, value);
    }
    set sortorder(value) {
      wasm.staticnode_set_sortorder(this.__wbg_ptr, isLikeNone(value) ? 4294967297 : value >> 0);
    }
    static compare(node_a, node_b) {
      const ret = wasm.staticnode_compare(node_a, node_b);
      return ret;
    }
    get exportable() {
      const ret = wasm.staticnode_get_exportable(this.__wbg_ptr);
      return ret !== 0;
    }
    get isrequired() {
      const ret = wasm.staticnode_get_isrequired(this.__wbg_ptr);
      return ret !== 0;
    }
    set exportable(value) {
      wasm.staticnode_set_exportable(this.__wbg_ptr, value);
    }
    set isrequired(value) {
      wasm.staticnode_set_isrequired(this.__wbg_ptr, value);
    }
    get description() {
      const ret = wasm.staticnode_get_description(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticTranslatableString.__wrap(ret);
    }
    set description(value) {
      let ptr0 = 0;
      if (!isLikeNone(value)) {
        _assertClass(value, StaticTranslatableString);
        ptr0 = value.__destroy_into_raw();
      }
      wasm.staticnode_set_description(this.__wbg_ptr, ptr0);
    }
    get is_collector() {
      const ret = wasm.staticnode_get_is_collector(this.__wbg_ptr);
      return ret !== 0;
    }
    get issearchable() {
      const ret = wasm.staticnode_get_issearchable(this.__wbg_ptr);
      return ret !== 0;
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
    set is_collector(value) {
      wasm.staticnode_set_is_collector(this.__wbg_ptr, value);
    }
    set issearchable(value) {
      wasm.staticnode_set_issearchable(this.__wbg_ptr, value);
    }
    set nodegroup_id(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_nodegroup_id(this.__wbg_ptr, ptr0, len0);
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
    get hascustomalias() {
      const ret = wasm.staticnode_get_hascustomalias(this.__wbg_ptr);
      return ret !== 0;
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
    set hascustomalias(value) {
      wasm.staticnode_set_hascustomalias(this.__wbg_ptr, value);
    }
    set parentproperty(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnode_set_parentproperty(this.__wbg_ptr, ptr0, len0);
    }
    get config() {
      const ret = wasm.staticnode_get_config_property(this.__wbg_ptr);
      return ret;
    }
    set config(value) {
      wasm.staticnode_set_config_property(this.__wbg_ptr, value);
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
  }
  if (Symbol.dispose) StaticNode.prototype[Symbol.dispose] = StaticNode.prototype.free;
  const StaticNodegroupFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticnodegroup_free(ptr >>> 0, 1));
  class StaticNodegroup {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticNodegroup.prototype);
      obj.__wbg_ptr = ptr;
      StaticNodegroupFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticNodegroupFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticnodegroup_free(ptr, 0);
    }
    get cardinality() {
      const ret = wasm.staticnodegroup_get_cardinality(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get nodegroupid() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.staticnodegroup_get_nodegroupid(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set cardinality(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnodegroup_set_cardinality(this.__wbg_ptr, ptr0, len0);
    }
    set nodegroupid(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticnodegroup_set_nodegroupid(this.__wbg_ptr, ptr0, len0);
    }
    get legacygroupid() {
      const ret = wasm.staticnodegroup_get_legacygroupid(this.__wbg_ptr);
      return ret;
    }
    get parentnodegroup_id() {
      const ret = wasm.staticnodegroup_get_parentnodegroup_id(this.__wbg_ptr);
      return ret;
    }
    set parentnodegroup_id(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticnodegroup_set_parentnodegroup_id(this.__wbg_ptr, ptr0, len0);
    }
    constructor(json_data) {
      const ret = wasm.staticnodegroup_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticNodegroupFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticnodegroup_copy(this.__wbg_ptr);
      return StaticNodegroup.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticnodegroup_toJSON(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) StaticNodegroup.prototype[Symbol.dispose] = StaticNodegroup.prototype.free;
  const StaticPublicationFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticpublication_free(ptr >>> 0, 1));
  class StaticPublication {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticPublication.prototype);
      obj.__wbg_ptr = ptr;
      StaticPublicationFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticPublicationFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticpublication_free(ptr, 0);
    }
    get graph_id() {
      const ret = wasm.staticpublication_get_graph_id(this.__wbg_ptr);
      return ret;
    }
    set graph_id(value) {
      wasm.staticpublication_set_graph_id(this.__wbg_ptr, value);
    }
    get publicationid() {
      const ret = wasm.staticpublication_get_publicationid(this.__wbg_ptr);
      return ret;
    }
    set publicationid(value) {
      wasm.staticpublication_set_publicationid(this.__wbg_ptr, value);
    }
    get published_time() {
      const ret = wasm.staticpublication_get_published_time(this.__wbg_ptr);
      return ret;
    }
    set published_time(value) {
      wasm.staticpublication_set_published_time(this.__wbg_ptr, value);
    }
    constructor(json_data) {
      const ret = wasm.staticpublication_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticPublicationFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticpublication_copy(this.__wbg_ptr);
      return StaticPublication.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticpublication_toJSON(this.__wbg_ptr);
      return ret;
    }
    get notes() {
      const ret = wasm.staticpublication_get_notes(this.__wbg_ptr);
      return ret;
    }
    set notes(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticpublication_set_notes(this.__wbg_ptr, ptr0, len0);
    }
  }
  if (Symbol.dispose) StaticPublication.prototype[Symbol.dispose] = StaticPublication.prototype.free;
  const StaticResourceFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticresource_free(ptr >>> 0, 1));
  class StaticResource {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticResource.prototype);
      obj.__wbg_ptr = ptr;
      StaticResourceFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    static __unwrap(jsValue) {
      if (!(jsValue instanceof StaticResource)) {
        return 0;
      }
      return jsValue.__destroy_into_raw();
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticResourceFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticresource_free(ptr, 0);
    }
    get __scopes() {
      const ret = wasm.staticresource_get_scopes(this.__wbg_ptr);
      return ret;
    }
    set __scopes(value) {
      wasm.staticresource_set_scopes(this.__wbg_ptr, value);
    }
    static fromSummary(summary) {
      _assertClass(summary, StaticResourceSummary);
      const ret = wasm.staticresource_fromSummary(summary.__wbg_ptr);
      return StaticResource.__wrap(ret);
    }
    set metadata(value) {
      wasm.staticresource_set_metadata(this.__wbg_ptr, value);
    }
    get tilesLoaded() {
      const ret = wasm.staticresource_tiles_loaded(this.__wbg_ptr);
      return ret !== 0;
    }
    static fromJsonString(json_str) {
      const ptr0 = passStringToWasm0(json_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresource_fromJsonString(ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticResource.__wrap(ret[0]);
    }
    get resourceinstance() {
      const ret = wasm.staticresource_resourceinstance(this.__wbg_ptr);
      return StaticResourceMetadata.__wrap(ret);
    }
    static fromBusinessDataJsonString(json_str) {
      const ptr0 = passStringToWasm0(json_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresource_fromBusinessDataJsonString(ptr0, len0);
      if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
      }
      var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v2;
    }
    constructor(data) {
      const ret = wasm.staticresource_new(data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticResourceFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticresource_copy(this.__wbg_ptr);
      return StaticResource.__wrap(ret);
    }
    get tiles() {
      const ret = wasm.staticresource_tiles(this.__wbg_ptr);
      return ret;
    }
    toJSON() {
      const ret = wasm.staticresource_toJSON(this.__wbg_ptr);
      return ret;
    }
    get metadata() {
      const ret = wasm.staticresource_metadata(this.__wbg_ptr);
      return ret;
    }
    get __cache() {
      const ret = wasm.staticresource_get_cache(this.__wbg_ptr);
      return ret;
    }
    set __cache(value) {
      wasm.staticresource_set_cache(this.__wbg_ptr, value);
    }
    set tiles(value) {
      wasm.staticresource_set_tiles(this.__wbg_ptr, value);
    }
  }
  if (Symbol.dispose) StaticResource.prototype[Symbol.dispose] = StaticResource.prototype.free;
  const StaticResourceDescriptorsFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticresourcedescriptors_free(ptr >>> 0, 1));
  class StaticResourceDescriptors {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticResourceDescriptors.prototype);
      obj.__wbg_ptr = ptr;
      StaticResourceDescriptorsFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticResourceDescriptorsFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticresourcedescriptors_free(ptr, 0);
    }
    set mapPopup(value) {
      var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      wasm.staticresourcedescriptors_set_map_popup(this.__wbg_ptr, ptr0, len0);
    }
    get description() {
      const ret = wasm.staticresourcedescriptors_get_description(this.__wbg_ptr);
      return ret;
    }
    set description(value) {
      wasm.staticresourcedescriptors_set_description(this.__wbg_ptr, value);
    }
    constructor(json_data) {
      const ret = wasm.staticresourcedescriptors_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticResourceDescriptorsFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticresourcedescriptors_copy(this.__wbg_ptr);
      return StaticResourceDescriptors.__wrap(ret);
    }
    static empty() {
      const ret = wasm.staticresourcedescriptors_empty();
      return StaticResourceDescriptors.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticresourcedescriptors_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      const ret = wasm.staticresourcedescriptors_get_name(this.__wbg_ptr);
      return ret;
    }
    get slug() {
      const ret = wasm.staticresourcedescriptors_get_slug(this.__wbg_ptr);
      return ret;
    }
    isEmpty() {
      const ret = wasm.staticresourcedescriptors_isEmpty(this.__wbg_ptr);
      return ret !== 0;
    }
    set name(value) {
      wasm.staticresourcedescriptors_set_name(this.__wbg_ptr, value);
    }
    set slug(value) {
      wasm.staticresourcedescriptors_set_slug(this.__wbg_ptr, value);
    }
    get mapPopup() {
      const ret = wasm.staticresourcedescriptors_map_popup(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
  }
  if (Symbol.dispose) StaticResourceDescriptors.prototype[Symbol.dispose] = StaticResourceDescriptors.prototype.free;
  const StaticResourceMetadataFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticresourcemetadata_free(ptr >>> 0, 1));
  class StaticResourceMetadata {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticResourceMetadata.prototype);
      obj.__wbg_ptr = ptr;
      StaticResourceMetadataFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticResourceMetadataFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticresourcemetadata_free(ptr, 0);
    }
    get descriptors() {
      const ret = wasm.staticresourcemetadata_descriptors(this.__wbg_ptr);
      return StaticResourceDescriptors.__wrap(ret);
    }
    get graph_id() {
      const ret = wasm.staticresourcemetadata_get_graph_id(this.__wbg_ptr);
      return ret;
    }
    get legacyid() {
      const ret = wasm.staticresourcemetadata_get_legacyid(this.__wbg_ptr);
      return ret;
    }
    get createdtime() {
      const ret = wasm.staticresourcemetadata_get_createdtime(this.__wbg_ptr);
      return ret;
    }
    get lastmodified() {
      const ret = wasm.staticresourcemetadata_get_lastmodified(this.__wbg_ptr);
      return ret;
    }
    get publication_id() {
      const ret = wasm.staticresourcemetadata_get_publication_id(this.__wbg_ptr);
      return ret;
    }
    get principaluser_id() {
      const ret = wasm.staticresourcemetadata_get_principaluser_id(this.__wbg_ptr);
      return ret;
    }
    get resourceinstanceid() {
      const ret = wasm.staticresourcemetadata_get_resourceinstanceid(this.__wbg_ptr);
      return ret;
    }
    get graph_publication_id() {
      const ret = wasm.staticresourcemetadata_get_graph_publication_id(this.__wbg_ptr);
      return ret;
    }
    constructor(json_data) {
      const ret = wasm.staticresourcemetadata_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticResourceMetadataFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticresourcemetadata_copy(this.__wbg_ptr);
      return StaticResourceMetadata.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticresourcemetadata_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      const ret = wasm.staticresourcemetadata_get_name(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) StaticResourceMetadata.prototype[Symbol.dispose] = StaticResourceMetadata.prototype.free;
  const StaticResourceReferenceFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticresourcereference_free(ptr >>> 0, 1));
  class StaticResourceReference {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticResourceReference.prototype);
      obj.__wbg_ptr = ptr;
      StaticResourceReferenceFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticResourceReferenceFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticresourcereference_free(ptr, 0);
    }
    get graph_id() {
      const ret = wasm.staticresourcereference_get_graph_id(this.__wbg_ptr);
      return ret;
    }
    get type() {
      const ret = wasm.staticresourcereference_resource_type(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    constructor(data) {
      const ret = wasm.staticresourcereference_new(data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticResourceReferenceFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.staticresourcereference_copy(this.__wbg_ptr);
      return StaticResourceReference.__wrap(ret);
    }
    get meta() {
      const ret = wasm.staticresourcereference_meta(this.__wbg_ptr);
      return ret;
    }
    get root() {
      const ret = wasm.staticresourcereference_root(this.__wbg_ptr);
      return ret;
    }
    get title() {
      const ret = wasm.staticresourcereference_title(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    get id() {
      const ret = wasm.staticresourcereference_get_id(this.__wbg_ptr);
      return ret;
    }
    toJSON() {
      const ret = wasm.staticresourcereference_toJSON(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) StaticResourceReference.prototype[Symbol.dispose] = StaticResourceReference.prototype.free;
  const StaticResourceRegistryFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticresourceregistry_free(ptr >>> 0, 1));
  class StaticResourceRegistry {
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticResourceRegistryFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticresourceregistry_free(ptr, 0);
    }
    getSummary(resource_id) {
      var ptr0 = isLikeNone(resource_id) ? 0 : passStringToWasm0(resource_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_getSummary(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : StaticResourceSummary.__wrap(ret);
    }
    getAllFull() {
      const ret = wasm.staticresourceregistry_getAllFull(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    getGraphId(resource_id) {
      var ptr0 = isLikeNone(resource_id) ? 0 : passStringToWasm0(resource_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_getGraphId(this.__wbg_ptr, ptr0, len0);
      let v2;
      if (ret[0] !== 0) {
        v2 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v2;
    }
    populateCaches(resources, graph, enrich_relationships, strict, recompute_descriptors) {
      const ptr0 = passArrayJsValueToWasm0(resources, wasm.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      _assertClass(graph, StaticGraph);
      const ret = wasm.staticresourceregistry_populateCaches(this.__wbg_ptr, ptr0, len0, graph.__wbg_ptr, isLikeNone(enrich_relationships) ? 16777215 : enrich_relationships ? 1 : 0, isLikeNone(strict) ? 16777215 : strict ? 1 : 0, isLikeNone(recompute_descriptors) ? 16777215 : recompute_descriptors ? 1 : 0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    insertFromJson(summary_json) {
      const ptr0 = passStringToWasm0(summary_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_insertFromJson(this.__wbg_ptr, ptr0, len0);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    mergeFromResources(resources, store_full, include_caches) {
      const ptr0 = passArrayJsValueToWasm0(resources, wasm.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.staticresourceregistry_mergeFromResources(this.__wbg_ptr, ptr0, len0, isLikeNone(store_full) ? 16777215 : store_full ? 1 : 0, isLikeNone(include_caches) ? 16777215 : include_caches ? 1 : 0);
    }
    getNodeValuesIndex(graph, node_identifier) {
      _assertClass(graph, StaticGraph);
      const ptr0 = passStringToWasm0(node_identifier, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_getNodeValuesIndex(this.__wbg_ptr, graph.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getAllFullForGraph(graph_id) {
      var ptr0 = isLikeNone(graph_id) ? 0 : passStringToWasm0(graph_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_getAllFullForGraph(this.__wbg_ptr, ptr0, len0);
      var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v2;
    }
    mergeFromResourcesJson(resources_json, store_full, include_caches) {
      const ptr0 = passStringToWasm0(resources_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_mergeFromResourcesJson(this.__wbg_ptr, ptr0, len0, isLikeNone(store_full) ? 16777215 : store_full ? 1 : 0, isLikeNone(include_caches) ? 16777215 : include_caches ? 1 : 0);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    populateCachesFromJson(resources_json, graph, enrich_relationships, strict, recompute_descriptors) {
      const ptr0 = passStringToWasm0(resources_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      _assertClass(graph, StaticGraph);
      const ret = wasm.staticresourceregistry_populateCachesFromJson(this.__wbg_ptr, ptr0, len0, graph.__wbg_ptr, isLikeNone(enrich_relationships) ? 16777215 : enrich_relationships ? 1 : 0, isLikeNone(strict) ? 16777215 : strict ? 1 : 0, isLikeNone(recompute_descriptors) ? 16777215 : recompute_descriptors ? 1 : 0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getValueToResourcesIndex(graph, node_identifier, flatten_localized) {
      _assertClass(graph, StaticGraph);
      const ptr0 = passStringToWasm0(node_identifier, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_getValueToResourcesIndex(this.__wbg_ptr, graph.__wbg_ptr, ptr0, len0, isLikeNone(flatten_localized) ? 16777215 : flatten_localized ? 1 : 0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    constructor() {
      const ret = wasm.staticresourceregistry_new();
      this.__wbg_ptr = ret >>> 0;
      StaticResourceRegistryFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    insert(summary) {
      _assertClass(summary, StaticResourceSummary);
      wasm.staticresourceregistry_insert(this.__wbg_ptr, summary.__wbg_ptr);
    }
    get length() {
      const ret = wasm.staticresourceregistry_length(this.__wbg_ptr);
      return ret >>> 0;
    }
    contains(resource_id) {
      var ptr0 = isLikeNone(resource_id) ? 0 : passStringToWasm0(resource_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_contains(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    getFull(resource_id) {
      var ptr0 = isLikeNone(resource_id) ? 0 : passStringToWasm0(resource_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_getFull(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : StaticResource.__wrap(ret);
    }
    hasFull(resource_id) {
      var ptr0 = isLikeNone(resource_id) ? 0 : passStringToWasm0(resource_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourceregistry_hasFull(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    get isEmpty() {
      const ret = wasm.staticresourceregistry_is_empty(this.__wbg_ptr);
      return ret !== 0;
    }
  }
  if (Symbol.dispose) StaticResourceRegistry.prototype[Symbol.dispose] = StaticResourceRegistry.prototype.free;
  const StaticResourceSummaryFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_staticresourcesummary_free(ptr >>> 0, 1));
  class StaticResourceSummary {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticResourceSummary.prototype);
      obj.__wbg_ptr = ptr;
      StaticResourceSummaryFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticResourceSummaryFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_staticresourcesummary_free(ptr, 0);
    }
    get descriptors() {
      const ret = wasm.staticresourcesummary_descriptors(this.__wbg_ptr);
      return ret === 0 ? void 0 : StaticResourceDescriptors.__wrap(ret);
    }
    toMetadata() {
      const ret = wasm.staticresourcesummary_toMetadata(this.__wbg_ptr);
      return StaticResourceMetadata.__wrap(ret);
    }
    get graph_id() {
      const ret = wasm.staticresourcesummary_get_graph_id(this.__wbg_ptr);
      return ret;
    }
    static fromResource(resource) {
      _assertClass(resource, StaticResource);
      const ret = wasm.staticresourcesummary_fromResource(resource.__wbg_ptr);
      return StaticResourceSummary.__wrap(ret);
    }
    static fromJsonString(json_str) {
      const ptr0 = passStringToWasm0(json_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourcesummary_fromJsonString(ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticResourceSummary.__wrap(ret[0]);
    }
    get resourceinstanceid() {
      const ret = wasm.staticresourcesummary_get_resourceinstanceid(this.__wbg_ptr);
      return ret;
    }
    constructor(data) {
      const ret = wasm.staticresourcesummary_new(data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticResourceSummaryFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    static summariesFromBusinessDataJsonString(json_str) {
      const ptr0 = passStringToWasm0(json_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.staticresourcesummary_summariesFromBusinessDataJsonString(ptr0, len0);
      if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
      }
      var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v2;
    }
    copy() {
      const ret = wasm.staticresourcesummary_copy(this.__wbg_ptr);
      return StaticResourceSummary.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.staticresourcesummary_toJSON(this.__wbg_ptr);
      return ret;
    }
    get name() {
      const ret = wasm.staticresourcesummary_get_name(this.__wbg_ptr);
      return ret;
    }
    get metadata() {
      const ret = wasm.staticresourcesummary_metadata(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) StaticResourceSummary.prototype[Symbol.dispose] = StaticResourceSummary.prototype.free;
  const StaticTileFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_statictile_free(ptr >>> 0, 1));
  class StaticTile {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(StaticTile.prototype);
      obj.__wbg_ptr = ptr;
      StaticTileFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    static __unwrap(jsValue) {
      if (!(jsValue instanceof StaticTile)) {
        return 0;
      }
      return jsValue.__destroy_into_raw();
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      StaticTileFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_statictile_free(ptr, 0);
    }
    get tileid() {
      const ret = wasm.statictile_get_tileid(this.__wbg_ptr);
      return ret;
    }
    set tileid(value) {
      wasm.statictile_set_tileid(this.__wbg_ptr, value);
    }
    get sortorder() {
      const ret = wasm.statictile_get_sortorder(this.__wbg_ptr);
      return ret;
    }
    set sortorder(value) {
      wasm.statictile_set_sortorder(this.__wbg_ptr, value);
    }
    get nodegroup_id() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.statictile_get_nodegroup_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set nodegroup_id(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.statictile_set_nodegroup_id(this.__wbg_ptr, ptr0, len0);
    }
    get parenttile_id() {
      const ret = wasm.statictile_get_parenttile_id(this.__wbg_ptr);
      return ret;
    }
    set parenttile_id(value) {
      wasm.statictile_set_parenttile_id(this.__wbg_ptr, value);
    }
    get provisionaledits() {
      const ret = wasm.statictile_get_provisionaledits(this.__wbg_ptr);
      return ret;
    }
    set provisionaledits(value) {
      wasm.statictile_set_provisionaledits(this.__wbg_ptr, value);
    }
    get resourceinstance_id() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.statictile_get_resourceinstance_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set resourceinstance_id(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.statictile_set_resourceinstance_id(this.__wbg_ptr, ptr0, len0);
    }
    constructor(json_data) {
      const ret = wasm.statictile_new(json_data);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      StaticTileFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.statictile_copy(this.__wbg_ptr);
      return StaticTile.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.statictile_toJSON(this.__wbg_ptr);
      return ret;
    }
    get data() {
      const ret = wasm.statictile_get_data(this.__wbg_ptr);
      return ret;
    }
    set data(value) {
      wasm.statictile_set_data(this.__wbg_ptr, value);
    }
    ensureId() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.statictile_ensureId(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    intoCore() {
      const ptr = this.__destroy_into_raw();
      const ret = wasm.statictile_intoCore(ptr);
      return ret;
    }
  }
  if (Symbol.dispose) StaticTile.prototype[Symbol.dispose] = StaticTile.prototype.free;
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
    get translations() {
      const ret = wasm.statictranslatablestring_get_translations(this.__wbg_ptr);
      return ret;
    }
    constructor(value, lang) {
      var ptr0 = isLikeNone(lang) ? 0 : passStringToWasm0(lang, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.statictranslatablestring_new(value, ptr0, len0);
      this.__wbg_ptr = ret >>> 0;
      StaticTranslatableStringFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    copy() {
      const ret = wasm.statictranslatablestring_copy(this.__wbg_ptr);
      return StaticTranslatableString.__wrap(ret);
    }
    toJSON() {
      const ret = wasm.statictranslatablestring_toJSON(this.__wbg_ptr);
      return ret;
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
  }
  if (Symbol.dispose) StaticTranslatableString.prototype[Symbol.dispose] = StaticTranslatableString.prototype.free;
  const WASMResourceInstanceWrapperFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmresourceinstancewrapper_free(ptr >>> 0, 1));
  class WASMResourceInstanceWrapper {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WASMResourceInstanceWrapper.prototype);
      obj.__wbg_ptr = ptr;
      WASMResourceInstanceWrapperFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WASMResourceInstanceWrapperFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmresourceinstancewrapper_free(ptr, 0);
    }
    toResource() {
      const ret = wasm.wasmresourceinstancewrapper_toResource(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    tilesLoaded() {
      const ret = wasm.wasmresourceinstancewrapper_tilesLoaded(this.__wbg_ptr);
      return ret !== 0;
    }
    getTileData(tile_id, node_id) {
      var ptr0 = isLikeNone(tile_id) ? 0 : passStringToWasm0(tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(node_id) ? 0 : passStringToWasm0(node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getTileData(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    loadTiles(tiles_js) {
      const ret = wasm.wasmresourceinstancewrapper_loadTiles(this.__wbg_ptr, tiles_js);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getTileCount() {
      const ret = wasm.wasmresourceinstancewrapper_getTileCount(this.__wbg_ptr);
      return ret >>> 0;
    }
    serializeCard(card_id, parent_tile_id, parent_nodegroup_id, max_depth) {
      const ptr0 = passStringToWasm0(card_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(parent_tile_id) ? 0 : passStringToWasm0(parent_tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      var ptr2 = isLikeNone(parent_nodegroup_id) ? 0 : passStringToWasm0(parent_nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len2 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_serializeCard(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, isLikeNone(max_depth) ? 4294967297 : max_depth >>> 0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    appendTiles(tiles_js) {
      const ret = wasm.wasmresourceinstancewrapper_appendTiles(this.__wbg_ptr, tiles_js);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getDescriptors(recompute) {
      const ret = wasm.wasmresourceinstancewrapper_getDescriptors(this.__wbg_ptr, recompute);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticResourceDescriptors.__wrap(ret[0]);
    }
    getResourceId() {
      const ret = wasm.wasmresourceinstancewrapper_getResourceId(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
    getRootPseudo() {
      const ret = wasm.wasmresourceinstancewrapper_getRootPseudo(this.__wbg_ptr);
      return ret === 0 ? void 0 : PseudoValue$1.__wrap(ret);
    }
    getTileLoader() {
      const ret = wasm.wasmresourceinstancewrapper_getTileLoader(this.__wbg_ptr);
      return ret;
    }
    loadTilesWasm(tiles, assume_tiles_comprehensive_for_nodegroup) {
      const ptr0 = passArrayJsValueToWasm0(tiles, wasm.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_loadTilesWasm(this.__wbg_ptr, ptr0, len0, isLikeNone(assume_tiles_comprehensive_for_nodegroup) ? 16777215 : assume_tiles_comprehensive_for_nodegroup ? 1 : 0);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    setTileLoader(loader) {
      wasm.wasmresourceinstancewrapper_setTileLoader(this.__wbg_ptr, loader);
    }
    toDisplayJson(rdm_cache, node_config_manager, language) {
      _assertClass(rdm_cache, WasmRdmCache);
      _assertClass(node_config_manager, WasmNodeConfigManager);
      var ptr0 = isLikeNone(language) ? 0 : passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_toDisplayJson(this.__wbg_ptr, rdm_cache.__wbg_ptr, node_config_manager.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    ensureNodegroup(all_values_js, all_nodegroups_js, nodegroup_id, add_if_missing, nodegroup_permissions_js, do_implied_nodegroups) {
      const ptr0 = passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_ensureNodegroup(this.__wbg_ptr, all_values_js, all_nodegroups_js, ptr0, len0, add_if_missing, nodegroup_permissions_js, do_implied_nodegroups);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return WasmEnsureNodegroupResult.__wrap(ret[0]);
    }
    getAllTileIds() {
      const ret = wasm.wasmresourceinstancewrapper_getAllTileIds(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    cachePseudoList(alias, wasm_list) {
      const ptr0 = passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      _assertClass(wasm_list, PseudoList$1);
      var ptr1 = wasm_list.__destroy_into_raw();
      wasm.wasmresourceinstancewrapper_cachePseudoList(this.__wbg_ptr, ptr0, len0, ptr1);
    }
    getCachedPseudo(alias) {
      var ptr0 = isLikeNone(alias) ? 0 : passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getCachedPseudo(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : PseudoList$1.__wrap(ret);
    }
    makePseudoValue(alias, tile_id, is_permitted, is_single) {
      const ptr0 = passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(tile_id) ? 0 : passStringToWasm0(tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_makePseudoValue(this.__wbg_ptr, ptr0, len0, ptr1, len1, is_permitted, is_single);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    cachePseudoValue(alias, wasm_value) {
      const ptr0 = passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      _assertClass(wasm_value, PseudoValue$1);
      var ptr1 = wasm_value.__destroy_into_raw();
      wasm.wasmresourceinstancewrapper_cachePseudoValue(this.__wbg_ptr, ptr0, len0, ptr1);
    }
    clearPseudoCache() {
      wasm.wasmresourceinstancewrapper_clearPseudoCache(this.__wbg_ptr);
    }
    getValuesAtPath(path) {
      const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getValuesAtPath(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return PseudoList$1.__wrap(ret[0]);
    }
    getNodegroupCount() {
      const ret = wasm.wasmresourceinstancewrapper_getNodegroupCount(this.__wbg_ptr);
      return ret >>> 0;
    }
    isNodegroupLoaded(nodegroup_id) {
      var ptr0 = isLikeNone(nodegroup_id) ? 0 : passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_isNodegroupLoaded(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    pruneResourceTiles() {
      const ret = wasm.wasmresourceinstancewrapper_pruneResourceTiles(this.__wbg_ptr);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    serializeRootCards(max_depth) {
      const ret = wasm.wasmresourceinstancewrapper_serializeRootCards(this.__wbg_ptr, isLikeNone(max_depth) ? 4294967297 : max_depth >>> 0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    findSemanticChildren(parent_tile_id, parent_node_id, parent_nodegroup_id) {
      var ptr0 = isLikeNone(parent_tile_id) ? 0 : passStringToWasm0(parent_tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(parent_node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      var ptr2 = isLikeNone(parent_nodegroup_id) ? 0 : passStringToWasm0(parent_nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len2 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_findSemanticChildren(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    nodegroupIdsForCard(card_id, max_depth) {
      const ptr0 = passStringToWasm0(card_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_nodegroupIdsForCard(this.__wbg_ptr, ptr0, len0, isLikeNone(max_depth) ? 4294967297 : max_depth >>> 0);
      if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
      }
      var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v2;
    }
    serializeCardDisplay(card_id, rdm_cache, node_config_manager, parent_tile_id, parent_nodegroup_id, max_depth, language) {
      const ptr0 = passStringToWasm0(card_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      _assertClass(rdm_cache, WasmRdmCache);
      _assertClass(node_config_manager, WasmNodeConfigManager);
      var ptr1 = isLikeNone(parent_tile_id) ? 0 : passStringToWasm0(parent_tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      var ptr2 = isLikeNone(parent_nodegroup_id) ? 0 : passStringToWasm0(parent_nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len2 = WASM_VECTOR_LEN;
      var ptr3 = isLikeNone(language) ? 0 : passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len3 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_serializeCardDisplay(this.__wbg_ptr, ptr0, len0, rdm_cache.__wbg_ptr, node_config_manager.__wbg_ptr, ptr1, len1, ptr2, len2, isLikeNone(max_depth) ? 4294967297 : max_depth >>> 0, ptr3, len3);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    toDisplayJsonSimple(language) {
      var ptr0 = isLikeNone(language) ? 0 : passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_toDisplayJsonSimple(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    hasTilesForNodegroup(nodegroup_id) {
      const ptr0 = passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_hasTilesForNodegroup(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    loadTilesFromResource(resource, assume_tiles_comprehensive_for_nodegroup) {
      _assertClass(resource, StaticResource);
      const ret = wasm.wasmresourceinstancewrapper_loadTilesFromResource(this.__wbg_ptr, resource.__wbg_ptr, isLikeNone(assume_tiles_comprehensive_for_nodegroup) ? 16777215 : assume_tiles_comprehensive_for_nodegroup ? 1 : 0);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getTileIdsByNodegroup(nodegroup_id) {
      var ptr0 = isLikeNone(nodegroup_id) ? 0 : passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getTileIdsByNodegroup(this.__wbg_ptr, ptr0, len0);
      var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v2;
    }
    tryAcquireNodegroupLock(nodegroup_id) {
      const ptr0 = passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_tryAcquireNodegroupLock(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    requestTilesForNodegroup(_nodegroup_id) {
      var ptr0 = isLikeNone(_nodegroup_id) ? 0 : passStringToWasm0(_nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_requestTilesForNodegroup(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    serializeRootCardsDisplay(rdm_cache, node_config_manager, max_depth, language) {
      _assertClass(rdm_cache, WasmRdmCache);
      _assertClass(node_config_manager, WasmNodeConfigManager);
      var ptr0 = isLikeNone(language) ? 0 : passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_serializeRootCardsDisplay(this.__wbg_ptr, rdm_cache.__wbg_ptr, node_config_manager.__wbg_ptr, isLikeNone(max_depth) ? 4294967297 : max_depth >>> 0, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getAllSemanticChildValues(parent_tile_id, parent_node_id, parent_nodegroup_id) {
      var ptr0 = isLikeNone(parent_tile_id) ? 0 : passStringToWasm0(parent_tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(parent_node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      var ptr2 = isLikeNone(parent_nodegroup_id) ? 0 : passStringToWasm0(parent_nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len2 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getAllSemanticChildValues(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getSemanticChildValue(parent_tile_id, parent_node_id, parent_nodegroup_id, child_alias) {
      var ptr0 = isLikeNone(parent_tile_id) ? 0 : passStringToWasm0(parent_tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(parent_node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      var ptr2 = isLikeNone(parent_nodegroup_id) ? 0 : passStringToWasm0(parent_nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len2 = WASM_VECTOR_LEN;
      const ptr3 = passStringToWasm0(child_alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len3 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getSemanticChildValue(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    retrieveSemanticChildValue(parent_tile_id, parent_node_id, parent_nodegroup_id, child_alias) {
      var ptr0 = isLikeNone(parent_tile_id) ? 0 : passStringToWasm0(parent_tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(parent_node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      var ptr2 = isLikeNone(parent_nodegroup_id) ? 0 : passStringToWasm0(parent_nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len2 = WASM_VECTOR_LEN;
      const ptr3 = passStringToWasm0(child_alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len3 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_retrieveSemanticChildValue(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
      return ret;
    }
    isNodegroupLoadedOrLoading(nodegroup_id) {
      var ptr0 = isLikeNone(nodegroup_id) ? 0 : passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_isNodegroupLoadedOrLoading(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    valuesFromResourceNodegroup(existing_values_js, nodegroup_tile_ids, nodegroup_id, _node_objs_js, _edges_js) {
      const ptr0 = passArrayJsValueToWasm0(nodegroup_tile_ids, wasm.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_valuesFromResourceNodegroup(this.__wbg_ptr, existing_values_js, ptr0, len0, ptr1, len1, _node_objs_js, _edges_js);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return WasmValuesFromNodegroupResult.__wrap(ret[0]);
    }
    getMissingNodegroupsForChildren(parent_node_id) {
      const ptr0 = passStringToWasm0(parent_node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getMissingNodegroupsForChildren(this.__wbg_ptr, ptr0, len0);
      if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
      }
      var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v2;
    }
    release() {
      wasm.wasmresourceinstancewrapper_release(this.__wbg_ptr);
    }
    toJson() {
      const ret = wasm.wasmresourceinstancewrapper_toJson(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getName() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wasmresourceinstancewrapper_getName(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    getTile(tile_id) {
      var ptr0 = isLikeNone(tile_id) ? 0 : passStringToWasm0(tile_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_getTile(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticTile.__wrap(ret[0]);
    }
    populate(lazy, nodegroup_ids, root_node_alias) {
      const ptr0 = passArrayJsValueToWasm0(nodegroup_ids, wasm.__wbindgen_malloc);
      const len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(root_node_alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourceinstancewrapper_populate(this.__wbg_ptr, lazy, ptr0, len0, ptr1, len1);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return WasmPopulateResult.__wrap(ret[0]);
    }
    setLazy(lazy) {
      wasm.wasmresourceinstancewrapper_setLazy(this.__wbg_ptr, lazy);
    }
    toTiles() {
      const ret = wasm.wasmresourceinstancewrapper_toTiles(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
  }
  if (Symbol.dispose) WASMResourceInstanceWrapper.prototype[Symbol.dispose] = WASMResourceInstanceWrapper.prototype.free;
  const WASMResourceModelWrapperFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmresourcemodelwrapper_free(ptr >>> 0, 1));
  WASMResourceModelWrapper = class {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WASMResourceModelWrapper.prototype);
      obj.__wbg_ptr = ptr;
      WASMResourceModelWrapperFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WASMResourceModelWrapperFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmresourcemodelwrapper_free(ptr, 0);
    }
    build_nodes() {
      const ret = wasm.wasmresourcemodelwrapper_build_nodes(this.__wbg_ptr);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    pruneGraph(keep_functions) {
      var ptr0 = isLikeNone(keep_functions) ? 0 : passArrayJsValueToWasm0(keep_functions, wasm.__wbindgen_malloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_pruneGraph(this.__wbg_ptr, ptr0, len0);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getGraphId() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wasmresourcemodelwrapper_getGraphId(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    getRootNode() {
      const ret = wasm.wasmresourcemodelwrapper_getRootNode(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticNode.__wrap(ret[0]);
    }
    get edges() {
      const ret = wasm.wasmresourcemodelwrapper_get_edges_prop(this.__wbg_ptr);
      return ret;
    }
    get nodes() {
      const ret = wasm.wasmresourcemodelwrapper_get_nodes_prop(this.__wbg_ptr);
      return ret;
    }
    getChildNodes(node_id) {
      var ptr0 = isLikeNone(node_id) ? 0 : passStringToWasm0(node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_getChildNodes(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    setGraphEdges(edges) {
      wasm.wasmresourcemodelwrapper_setGraphEdges(this.__wbg_ptr, edges);
    }
    setGraphNodes(nodes) {
      wasm.wasmresourcemodelwrapper_setGraphNodes(this.__wbg_ptr, nodes);
    }
    getNodeObjects() {
      const ret = wasm.wasmresourcemodelwrapper_getNodeObjects(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getNodegroupIds() {
      const ret = wasm.wasmresourcemodelwrapper_getNodegroupIds(this.__wbg_ptr);
      if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
      }
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    createPseudoNode(child_node) {
      var ptr0 = isLikeNone(child_node) ? 0 : passStringToWasm0(child_node, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_createPseudoNode(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return PseudoNode.__wrap(ret[0]);
    }
    getNodegroupName(nodegroup_id) {
      let deferred3_0;
      let deferred3_1;
      try {
        var ptr0 = isLikeNone(nodegroup_id) ? 0 : passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmresourcemodelwrapper_getNodegroupName(this.__wbg_ptr, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
          ptr2 = 0;
          len2 = 0;
          throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
      } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
      }
    }
    createPseudoValue(alias, tile, parent) {
      var ptr0 = isLikeNone(alias) ? 0 : passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_createPseudoValue(this.__wbg_ptr, ptr0, len0, tile, parent);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return PseudoValue$1.__wrap(ret[0]);
    }
    get nodegroups() {
      const ret = wasm.wasmresourcemodelwrapper_get_nodegroups_prop(this.__wbg_ptr);
      return ret;
    }
    static isModelRegistered(graph_id) {
      var ptr0 = isLikeNone(graph_id) ? 0 : passStringToWasm0(graph_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_isModelRegistered(ptr0, len0);
      return ret !== 0;
    }
    setGraphNodegroups(nodegroups) {
      wasm.wasmresourcemodelwrapper_setGraphNodegroups(this.__wbg_ptr, nodegroups);
    }
    buildNodesForGraph(graph) {
      _assertClass(graph, StaticGraph);
      const ret = wasm.wasmresourcemodelwrapper_buildNodesForGraph(this.__wbg_ptr, graph.__wbg_ptr);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    static getModelByGraphId(graph_id) {
      var ptr0 = isLikeNone(graph_id) ? 0 : passStringToWasm0(graph_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_getModelByGraphId(ptr0, len0);
      return ret === 0 ? void 0 : WASMResourceModelWrapper.__wrap(ret);
    }
    getNodegroupObjects() {
      const ret = wasm.wasmresourcemodelwrapper_getNodegroupObjects(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getChildNodeAliases(node_id) {
      var ptr0 = isLikeNone(node_id) ? 0 : passStringToWasm0(node_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_getChildNodeAliases(this.__wbg_ptr, ptr0, len0);
      if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
      }
      var v2 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v2;
    }
    getNodeIdFromAlias(alias) {
      let deferred3_0;
      let deferred3_1;
      try {
        var ptr0 = isLikeNone(alias) ? 0 : passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmresourcemodelwrapper_getNodeIdFromAlias(this.__wbg_ptr, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
          ptr2 = 0;
          len2 = 0;
          throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
      } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
      }
    }
    isNodegroupPermitted(nodegroup_id, _tile) {
      const ptr0 = passStringToWasm0(nodegroup_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      let ptr1 = 0;
      if (!isLikeNone(_tile)) {
        _assertClass(_tile, StaticTile);
        ptr1 = _tile.__destroy_into_raw();
      }
      const ret = wasm.wasmresourcemodelwrapper_isNodegroupPermitted(this.__wbg_ptr, ptr0, len0, ptr1);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return ret[0] !== 0;
    }
    getNodeObjectFromId(id) {
      var ptr0 = isLikeNone(id) ? 0 : passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_getNodeObjectFromId(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticNode.__wrap(ret[0]);
    }
    get nodesByAlias() {
      const ret = wasm.wasmresourcemodelwrapper_get_nodes_by_alias_prop(this.__wbg_ptr);
      return ret;
    }
    createPseudoNodeChild(child_node, parent_pseudo) {
      const ptr0 = passStringToWasm0(child_node, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      _assertClass(parent_pseudo, PseudoNode);
      const ret = wasm.wasmresourcemodelwrapper_createPseudoNodeChild(this.__wbg_ptr, ptr0, len0, parent_pseudo.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return PseudoNode.__wrap(ret[0]);
    }
    getPermittedNodegroups() {
      const ret = wasm.wasmresourcemodelwrapper_getPermittedNodegroups(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    setPermittedNodegroups(permissions) {
      const ret = wasm.wasmresourcemodelwrapper_setPermittedNodegroups(this.__wbg_ptr, permissions);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getNodeObjectsByAlias() {
      const ret = wasm.wasmresourcemodelwrapper_getNodeObjectsByAlias(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getNodeObjectFromAlias(alias) {
      var ptr0 = isLikeNone(alias) ? 0 : passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmresourcemodelwrapper_getNodeObjectFromAlias(this.__wbg_ptr, ptr0, len0);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return StaticNode.__wrap(ret[0]);
    }
    setDefaultAllowAllNodegroups(default_allow) {
      wasm.wasmresourcemodelwrapper_setDefaultAllowAllNodegroups(this.__wbg_ptr, default_allow);
    }
    constructor(wkrm, graph, default_allow) {
      _assertClass(wkrm, WKRM);
      _assertClass(graph, StaticGraph);
      const ret = wasm.wasmresourcemodelwrapper_new(wkrm.__wbg_ptr, graph.__wbg_ptr, default_allow);
      this.__wbg_ptr = ret >>> 0;
      WASMResourceModelWrapperFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    get wkrm() {
      const ret = wasm.wasmresourcemodelwrapper_get_wkrm(this.__wbg_ptr);
      return WKRM.__wrap(ret);
    }
    getEdges() {
      const ret = wasm.wasmresourcemodelwrapper_getEdges(this.__wbg_ptr);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    get graph() {
      const ret = wasm.wasmresourcemodelwrapper_get_graph(this.__wbg_ptr);
      return StaticGraph.__wrap(ret);
    }
    set graph(graph) {
      _assertClass(graph, StaticGraph);
      var ptr0 = graph.__destroy_into_raw();
      wasm.wasmresourcemodelwrapper_set_graph(this.__wbg_ptr, ptr0);
    }
  };
  if (Symbol.dispose) WASMResourceModelWrapper.prototype[Symbol.dispose] = WASMResourceModelWrapper.prototype.free;
  const WKRMFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wkrm_free(ptr >>> 0, 1));
  WKRM = class {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WKRM.prototype);
      obj.__wbg_ptr = ptr;
      WKRMFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WKRMFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wkrm_free(ptr, 0);
    }
    get graphId() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wkrm_get_graph_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set graphId(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.wkrm_set_graph_id(this.__wbg_ptr, ptr0, len0);
    }
    get modelName() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wkrm_get_model_name(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set modelName(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.wkrm_set_model_name(this.__wbg_ptr, ptr0, len0);
    }
    get modelClassName() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wkrm_get_model_class_name(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    set modelClassName(value) {
      const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.wkrm_set_model_class_name(this.__wbg_ptr, ptr0, len0);
    }
    constructor(meta_js) {
      const ret = wasm.wkrm_new(meta_js);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      this.__wbg_ptr = ret[0] >>> 0;
      WKRMFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    get meta() {
      const ret = wasm.wkrm_get_meta(this.__wbg_ptr);
      return StaticGraphMeta.__wrap(ret);
    }
    set meta(value) {
      wasm.wkrm_set_meta(this.__wbg_ptr, value);
    }
  };
  if (Symbol.dispose) WKRM.prototype[Symbol.dispose] = WKRM.prototype.free;
  typeof FinalizationRegistry === "undefined" ? {} : new FinalizationRegistry((ptr) => wasm.__wbg_wasmcoercionresult_free(ptr >>> 0, 1));
  const WasmEnsureNodegroupResultFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmensurenodegroupresult_free(ptr >>> 0, 1));
  class WasmEnsureNodegroupResult {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmEnsureNodegroupResult.prototype);
      obj.__wbg_ptr = ptr;
      WasmEnsureNodegroupResultFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmEnsureNodegroupResultFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmensurenodegroupresult_free(ptr, 0);
    }
    getAllValues() {
      const ret = wasm.wasmensurenodegroupresult_getAllValues(this.__wbg_ptr);
      return ret;
    }
    getValueAliases() {
      const ret = wasm.wasmensurenodegroupresult_getValueAliases(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    get allNodegroupsMap() {
      const ret = wasm.wasmensurenodegroupresult_all_nodegroups_map(this.__wbg_ptr);
      return ret;
    }
    get impliedNodegroups() {
      const ret = wasm.wasmensurenodegroupresult_implied_nodegroups(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    getValue(alias) {
      var ptr0 = isLikeNone(alias) ? 0 : passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmensurenodegroupresult_getValue(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : PseudoList$1.__wrap(ret);
    }
  }
  if (Symbol.dispose) WasmEnsureNodegroupResult.prototype[Symbol.dispose] = WasmEnsureNodegroupResult.prototype.free;
  const WasmNodeConfigBooleanFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmnodeconfigboolean_free(ptr >>> 0, 1));
  class WasmNodeConfigBoolean {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmNodeConfigBoolean.prototype);
      obj.__wbg_ptr = ptr;
      WasmNodeConfigBooleanFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmNodeConfigBooleanFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmnodeconfigboolean_free(ptr, 0);
    }
    get trueLabel() {
      const ret = wasm.wasmnodeconfigboolean_trueLabel(this.__wbg_ptr);
      return ret;
    }
    get falseLabel() {
      const ret = wasm.wasmnodeconfigboolean_falseLabel(this.__wbg_ptr);
      return ret;
    }
    getLabel(value, language) {
      const ptr0 = passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigboolean_getLabel(this.__wbg_ptr, value, ptr0, len0);
      let v2;
      if (ret[0] !== 0) {
        v2 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v2;
    }
  }
  if (Symbol.dispose) WasmNodeConfigBoolean.prototype[Symbol.dispose] = WasmNodeConfigBoolean.prototype.free;
  const WasmNodeConfigConceptFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmnodeconfigconcept_free(ptr >>> 0, 1));
  class WasmNodeConfigConcept {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmNodeConfigConcept.prototype);
      obj.__wbg_ptr = ptr;
      WasmNodeConfigConceptFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmNodeConfigConceptFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmnodeconfigconcept_free(ptr, 0);
    }
    get rdmCollection() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wasmnodeconfigconcept_rdmCollection(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
  }
  if (Symbol.dispose) WasmNodeConfigConcept.prototype[Symbol.dispose] = WasmNodeConfigConcept.prototype.free;
  const WasmNodeConfigDomainFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmnodeconfigdomain_free(ptr >>> 0, 1));
  class WasmNodeConfigDomain {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmNodeConfigDomain.prototype);
      obj.__wbg_ptr = ptr;
      WasmNodeConfigDomainFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmNodeConfigDomainFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmnodeconfigdomain_free(ptr, 0);
    }
    getSelected() {
      const ret = wasm.wasmnodeconfigdomain_getSelected(this.__wbg_ptr);
      return ret === 0 ? void 0 : WasmStaticDomainValue.__wrap(ret);
    }
    valueFromId(id) {
      const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigdomain_valueFromId(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : WasmStaticDomainValue.__wrap(ret);
    }
    getOptionIds() {
      const ret = wasm.wasmnodeconfigdomain_getOptionIds(this.__wbg_ptr);
      return ret;
    }
    get options() {
      const ret = wasm.wasmnodeconfigdomain_options(this.__wbg_ptr);
      return ret;
    }
  }
  if (Symbol.dispose) WasmNodeConfigDomain.prototype[Symbol.dispose] = WasmNodeConfigDomain.prototype.free;
  const WasmNodeConfigManagerFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmnodeconfigmanager_free(ptr >>> 0, 1));
  class WasmNodeConfigManager {
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmNodeConfigManagerFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmnodeconfigmanager_free(ptr, 0);
    }
    fromGraph(graph) {
      _assertClass(graph, StaticGraph);
      wasm.wasmnodeconfigmanager_fromGraph(this.__wbg_ptr, graph.__wbg_ptr);
    }
    getDomain(nodeid) {
      var ptr0 = isLikeNone(nodeid) ? 0 : passStringToWasm0(nodeid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_getDomain(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : WasmNodeConfigDomain.__wrap(ret);
    }
    hasConfig(nodeid) {
      var ptr0 = isLikeNone(nodeid) ? 0 : passStringToWasm0(nodeid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_hasConfig(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    getBoolean(nodeid) {
      var ptr0 = isLikeNone(nodeid) ? 0 : passStringToWasm0(nodeid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_getBoolean(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : WasmNodeConfigBoolean.__wrap(ret);
    }
    getConcept(nodeid) {
      var ptr0 = isLikeNone(nodeid) ? 0 : passStringToWasm0(nodeid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_getConcept(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : WasmNodeConfigConcept.__wrap(ret);
    }
    getReference(nodeid) {
      var ptr0 = isLikeNone(nodeid) ? 0 : passStringToWasm0(nodeid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_getReference(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : WasmNodeConfigReference.__wrap(ret);
    }
    fromGraphJson(graph_json) {
      const ptr0 = passStringToWasm0(graph_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_fromGraphJson(this.__wbg_ptr, ptr0, len0);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getConfigType(nodeid) {
      var ptr0 = isLikeNone(nodeid) ? 0 : passStringToWasm0(nodeid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_getConfigType(this.__wbg_ptr, ptr0, len0);
      let v2;
      if (ret[0] !== 0) {
        v2 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v2;
    }
    lookupDomainValue(nodeid, value_id) {
      var ptr0 = isLikeNone(nodeid) ? 0 : passStringToWasm0(nodeid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(value_id) ? 0 : passStringToWasm0(value_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmnodeconfigmanager_lookupDomainValue(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      return ret === 0 ? void 0 : WasmStaticDomainValue.__wrap(ret);
    }
    constructor() {
      const ret = wasm.wasmnodeconfigmanager_new();
      this.__wbg_ptr = ret >>> 0;
      WasmNodeConfigManagerFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    clear() {
      wasm.wasmnodeconfigmanager_clear(this.__wbg_ptr);
    }
    get length() {
      const ret = wasm.wasmnodeconfigmanager_length(this.__wbg_ptr);
      return ret >>> 0;
    }
  }
  if (Symbol.dispose) WasmNodeConfigManager.prototype[Symbol.dispose] = WasmNodeConfigManager.prototype.free;
  const WasmNodeConfigReferenceFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmnodeconfigreference_free(ptr >>> 0, 1));
  class WasmNodeConfigReference {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmNodeConfigReference.prototype);
      obj.__wbg_ptr = ptr;
      WasmNodeConfigReferenceFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmNodeConfigReferenceFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmnodeconfigreference_free(ptr, 0);
    }
    get multiValue() {
      const ret = wasm.wasmnodeconfigreference_multiValue(this.__wbg_ptr);
      return ret !== 0;
    }
    get rdmCollection() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wasmnodeconfigreference_rdmCollection(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get controlledList() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wasmnodeconfigreference_controlledList(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    getCollectionId() {
      const ret = wasm.wasmnodeconfigreference_getCollectionId(this.__wbg_ptr);
      let v1;
      if (ret[0] !== 0) {
        v1 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v1;
    }
  }
  if (Symbol.dispose) WasmNodeConfigReference.prototype[Symbol.dispose] = WasmNodeConfigReference.prototype.free;
  typeof FinalizationRegistry === "undefined" ? {} : new FinalizationRegistry((ptr) => wasm.__wbg_wasmnodegroupresult_free(ptr >>> 0, 1));
  const WasmPopulateResultFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmpopulateresult_free(ptr >>> 0, 1));
  class WasmPopulateResult {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmPopulateResult.prototype);
      obj.__wbg_ptr = ptr;
      WasmPopulateResultFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmPopulateResultFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmpopulateresult_free(ptr, 0);
    }
    get allValuesMap() {
      const ret = wasm.wasmpopulateresult_all_values_map(this.__wbg_ptr);
      return ret;
    }
    getAllValues() {
      const ret = wasm.wasmpopulateresult_getAllValues(this.__wbg_ptr);
      return ret;
    }
    getValueAliases() {
      const ret = wasm.wasmpopulateresult_getValueAliases(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    get allNodegroupsMap() {
      const ret = wasm.wasmpopulateresult_all_nodegroups_map(this.__wbg_ptr);
      return ret;
    }
    getValue(alias) {
      var ptr0 = isLikeNone(alias) ? 0 : passStringToWasm0(alias, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmpopulateresult_getValue(this.__wbg_ptr, ptr0, len0);
      return ret === 0 ? void 0 : PseudoList$1.__wrap(ret);
    }
  }
  if (Symbol.dispose) WasmPopulateResult.prototype[Symbol.dispose] = WasmPopulateResult.prototype.free;
  const WasmRdmCacheFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmrdmcache_free(ptr >>> 0, 1));
  class WasmRdmCache {
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmRdmCacheFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmrdmcache_free(ptr, 0);
    }
    lookupLabel(collection_id, concept_id, language) {
      var ptr0 = isLikeNone(collection_id) ? 0 : passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(concept_id) ? 0 : passStringToWasm0(concept_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      var ptr2 = isLikeNone(language) ? 0 : passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len2 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_lookupLabel(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
      let v42;
      if (ret[0] !== 0) {
        v42 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v42;
    }
    lookupValue(collection_id, value_id) {
      var ptr0 = isLikeNone(collection_id) ? 0 : passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(value_id) ? 0 : passStringToWasm0(value_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_lookupValue(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    getParentId(collection_id, concept_id) {
      var ptr0 = isLikeNone(collection_id) ? 0 : passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(concept_id) ? 0 : passStringToWasm0(concept_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_getParentId(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      let v3;
      if (ret[0] !== 0) {
        v3 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v3;
    }
    hasCollection(collection_id) {
      var ptr0 = isLikeNone(collection_id) ? 0 : passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_hasCollection(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    lookupConcept(collection_id, concept_id) {
      var ptr0 = isLikeNone(collection_id) ? 0 : passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(concept_id) ? 0 : passStringToWasm0(concept_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_lookupConcept(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
      }
      return takeFromExternrefTable0(ret[0]);
    }
    resolveLabels(tree_json, alias_to_collection, strict) {
      let deferred3_0;
      let deferred3_1;
      try {
        const ptr0 = passStringToWasm0(tree_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmrdmcache_resolveLabels(this.__wbg_ptr, ptr0, len0, alias_to_collection, strict);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
          ptr2 = 0;
          len2 = 0;
          throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
      } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
      }
    }
    validateValue(collection_id, value_id) {
      var ptr0 = isLikeNone(collection_id) ? 0 : passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(value_id) ? 0 : passStringToWasm0(value_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_validateValue(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      return ret !== 0;
    }
    removeCollection(collection_id) {
      const ptr0 = passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_removeCollection(this.__wbg_ptr, ptr0, len0);
      return ret !== 0;
    }
    getCollectionIds() {
      const ret = wasm.wasmrdmcache_getCollectionIds(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
    addCollectionFromJson(collection_id, concepts_json) {
      const ptr0 = passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ptr1 = passStringToWasm0(concepts_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_addCollectionFromJson(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
      }
    }
    getConceptIdForValue(collection_id, value_id) {
      var ptr0 = isLikeNone(collection_id) ? 0 : passStringToWasm0(collection_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len0 = WASM_VECTOR_LEN;
      var ptr1 = isLikeNone(value_id) ? 0 : passStringToWasm0(value_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      const ret = wasm.wasmrdmcache_getConceptIdForValue(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      let v3;
      if (ret[0] !== 0) {
        v3 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v3;
    }
    constructor() {
      const ret = wasm.wasmrdmcache_new();
      this.__wbg_ptr = ret >>> 0;
      WasmRdmCacheFinalization.register(this, this.__wbg_ptr, this);
      return this;
    }
    clear() {
      wasm.wasmrdmcache_clear(this.__wbg_ptr);
    }
    get length() {
      const ret = wasm.wasmrdmcache_length(this.__wbg_ptr);
      return ret >>> 0;
    }
  }
  if (Symbol.dispose) WasmRdmCache.prototype[Symbol.dispose] = WasmRdmCache.prototype.free;
  typeof FinalizationRegistry === "undefined" ? {} : new FinalizationRegistry((ptr) => wasm.__wbg_wasmspan_free(ptr >>> 0, 1));
  const WasmStaticDomainValueFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmstaticdomainvalue_free(ptr >>> 0, 1));
  class WasmStaticDomainValue {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmStaticDomainValue.prototype);
      obj.__wbg_ptr = ptr;
      WasmStaticDomainValueFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmStaticDomainValueFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmstaticdomainvalue_free(ptr, 0);
    }
    get id() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wasmstaticdomainvalue_id(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    lang(language) {
      const ptr0 = passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      const ret = wasm.wasmstaticdomainvalue_lang(this.__wbg_ptr, ptr0, len0);
      let v2;
      if (ret[0] !== 0) {
        v2 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
      }
      return v2;
    }
    get text() {
      const ret = wasm.wasmstaticdomainvalue_text(this.__wbg_ptr);
      return ret;
    }
    display() {
      let deferred1_0;
      let deferred1_1;
      try {
        const ret = wasm.wasmstaticdomainvalue_display(this.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
      } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
      }
    }
    get selected() {
      const ret = wasm.wasmstaticdomainvalue_selected(this.__wbg_ptr);
      return ret !== 0;
    }
  }
  if (Symbol.dispose) WasmStaticDomainValue.prototype[Symbol.dispose] = WasmStaticDomainValue.prototype.free;
  const WasmValuesFromNodegroupResultFinalization = typeof FinalizationRegistry === "undefined" ? {
    register: () => {
    },
    unregister: () => {
    }
  } : new FinalizationRegistry((ptr) => wasm.__wbg_wasmvaluesfromnodegroupresult_free(ptr >>> 0, 1));
  class WasmValuesFromNodegroupResult {
    static __wrap(ptr) {
      ptr = ptr >>> 0;
      const obj = Object.create(WasmValuesFromNodegroupResult.prototype);
      obj.__wbg_ptr = ptr;
      WasmValuesFromNodegroupResultFinalization.register(obj, obj.__wbg_ptr, obj);
      return obj;
    }
    __destroy_into_raw() {
      const ptr = this.__wbg_ptr;
      this.__wbg_ptr = 0;
      WasmValuesFromNodegroupResultFinalization.unregister(this);
      return ptr;
    }
    free() {
      const ptr = this.__destroy_into_raw();
      wasm.__wbg_wasmvaluesfromnodegroupresult_free(ptr, 0);
    }
    getAllValues() {
      const ret = wasm.wasmvaluesfromnodegroupresult_getAllValues(this.__wbg_ptr);
      return ret;
    }
    get impliedNodegroups() {
      const ret = wasm.wasmvaluesfromnodegroupresult_implied_nodegroups(this.__wbg_ptr);
      var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
      wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
      return v1;
    }
  }
  if (Symbol.dispose) WasmValuesFromNodegroupResult.prototype[Symbol.dispose] = WasmValuesFromNodegroupResult.prototype.free;
  const EXPECTED_RESPONSE_TYPES = /* @__PURE__ */ new Set([
    "basic",
    "cors",
    "default"
  ]);
  async function __wbg_load(module2, imports) {
    if (typeof Response === "function" && module2 instanceof Response) {
      if (typeof WebAssembly.instantiateStreaming === "function") {
        try {
          return await WebAssembly.instantiateStreaming(module2, imports);
        } catch (e) {
          const validResponse = module2.ok && EXPECTED_RESPONSE_TYPES.has(module2.type);
          if (validResponse && module2.headers.get("Content-Type") !== "application/wasm") {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
          } else {
            throw e;
          }
        }
      }
      const bytes = await module2.arrayBuffer();
      return await WebAssembly.instantiate(bytes, imports);
    } else {
      const instance = await WebAssembly.instantiate(module2, imports);
      if (instance instanceof WebAssembly.Instance) {
        return {
          instance,
          module: module2
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
    imports.wbg.__wbg_call_641db1bb5db5a579 = function() {
      return handleError(function(arg0, arg1, arg2, arg3) {
        const ret = arg0.call(arg1, arg2, arg3);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_call_a5400b25a865cfd8 = function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_call_e1bdd86bd1bc0f12 = function() {
      return handleError(function(arg0, arg1, arg2, arg3, arg4, arg5) {
        const ret = arg0.call(arg1, arg2, arg3, arg4, arg5);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_clearMarks_ffc83ba0394c7952 = function(arg0, arg1) {
      performance.clearMarks(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_crypto_574e78ad8b13b65f = function(arg0) {
      const ret = arg0.crypto;
      return ret;
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
    imports.wbg.__wbg_forEach_859dfd887a0f866c = function(arg0, arg1, arg2) {
      try {
        var state0 = {
          a: arg1,
          b: arg2
        };
        var cb0 = (arg02, arg12) => {
          const a = state0.a;
          state0.a = 0;
          try {
            return __wbg_adapter_773(a, state0.b, arg02, arg12);
          } finally {
            state0.a = a;
          }
        };
        arg0.forEach(cb0);
      } finally {
        state0.a = state0.b = 0;
      }
    };
    imports.wbg.__wbg_from_88bc52ce20ba6318 = function(arg0) {
      const ret = Array.from(arg0);
      return ret;
    };
    imports.wbg.__wbg_getRandomValues_b8f5dbd5f3995a9e = function() {
      return handleError(function(arg0, arg1) {
        arg0.getRandomValues(arg1);
      }, arguments);
    };
    imports.wbg.__wbg_getTime_6bb3f64e0f18f817 = function(arg0) {
      const ret = arg0.getTime();
      return ret;
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
    imports.wbg.__wbg_has_b89e451f638123e3 = function() {
      return handleError(function(arg0, arg1) {
        const ret = Reflect.has(arg0, arg1);
        return ret;
      }, arguments);
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
    imports.wbg.__wbg_log_6c7b5f4f00b8ce3f = function(arg0) {
      console.log(arg0);
    };
    imports.wbg.__wbg_mark_65792f7db18cbebe = function(arg0, arg1) {
      performance.mark(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_measure_bc4c215eae44fd16 = function(arg0, arg1, arg2) {
      performance.measure(getStringFromWasm0(arg0, arg1), arg2);
    };
    imports.wbg.__wbg_msCrypto_a61aeb35a24c1329 = function(arg0) {
      const ret = arg0.msCrypto;
      return ret;
    };
    imports.wbg.__wbg_new0_b0a0a38c201e6df5 = function() {
      const ret = /* @__PURE__ */ new Date();
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
    imports.wbg.__wbg_new_2e3c58a15f39f5f9 = function(arg0, arg1) {
      try {
        var state0 = {
          a: arg0,
          b: arg1
        };
        var cb0 = (arg02, arg12) => {
          const a = state0.a;
          state0.a = 0;
          try {
            return __wbg_adapter_773(a, state0.b, arg02, arg12);
          } finally {
            state0.a = a;
          }
        };
        const ret = new Promise(cb0);
        return ret;
      } finally {
        state0.a = state0.b = 0;
      }
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
    imports.wbg.__wbg_newnoargs_254190557c45b4ec = function(arg0, arg1) {
      const ret = new Function(getStringFromWasm0(arg0, arg1));
      return ret;
    };
    imports.wbg.__wbg_newwithlength_a167dcc7aaa3ba77 = function(arg0) {
      const ret = new Uint8Array(arg0 >>> 0);
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
    imports.wbg.__wbg_node_905d3e251edff8a2 = function(arg0) {
      const ret = arg0.node;
      return ret;
    };
    imports.wbg.__wbg_now_9d7d80f543db604d = function() {
      const ret = performance.now();
      return ret;
    };
    imports.wbg.__wbg_parse_442f5ba02e5eaf8b = function() {
      return handleError(function(arg0, arg1) {
        const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_process_dc0fbacc7c1c06f7 = function(arg0) {
      const ret = arg0.process;
      return ret;
    };
    imports.wbg.__wbg_prototypesetcall_3d4a26c1ed734349 = function(arg0, arg1, arg2) {
      Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
    };
    imports.wbg.__wbg_pseudolist_new = function(arg0) {
      const ret = PseudoList$1.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_pseudovalue_new = function(arg0) {
      const ret = PseudoValue$1.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_push_330b2eb93e4e1212 = function(arg0, arg1) {
      const ret = arg0.push(arg1);
      return ret;
    };
    imports.wbg.__wbg_queueMicrotask_25d0739ac89e8c88 = function(arg0) {
      queueMicrotask(arg0);
    };
    imports.wbg.__wbg_queueMicrotask_4488407636f5bf24 = function(arg0) {
      const ret = arg0.queueMicrotask;
      return ret;
    };
    imports.wbg.__wbg_randomFillSync_ac0988aba3254290 = function() {
      return handleError(function(arg0, arg1) {
        arg0.randomFillSync(arg1);
      }, arguments);
    };
    imports.wbg.__wbg_require_60cc747a6bc5215a = function() {
      return handleError(function() {
        const ret = module.require;
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_resolve_4055c623acdd6a1b = function(arg0) {
      const ret = Promise.resolve(arg0);
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
    imports.wbg.__wbg_static_accessor_GLOBAL_8921f820c2ce3f12 = function() {
      const ret = typeof global === "undefined" ? null : global;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_f0a4409105898184 = function() {
      const ret = typeof globalThis === "undefined" ? null : globalThis;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_995b214ae681ff99 = function() {
      const ret = typeof self === "undefined" ? null : self;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_cde3890479c675ea = function() {
      const ret = typeof window === "undefined" ? null : window;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_staticcard_new = function(arg0) {
      const ret = StaticCard.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_staticcardsxnodesxwidgets_new = function(arg0) {
      const ret = StaticCardsXNodesXWidgets.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_staticedge_new = function(arg0) {
      const ret = StaticEdge.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_staticnode_new = function(arg0) {
      const ret = StaticNode.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_staticnodegroup_new = function(arg0) {
      const ret = StaticNodegroup.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_staticresource_new = function(arg0) {
      const ret = StaticResource.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_staticresource_unwrap = function(arg0) {
      const ret = StaticResource.__unwrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_staticresourcesummary_new = function(arg0) {
      const ret = StaticResourceSummary.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_statictile_new = function(arg0) {
      const ret = StaticTile.__wrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_statictile_unwrap = function(arg0) {
      const ret = StaticTile.__unwrap(arg0);
      return ret;
    };
    imports.wbg.__wbg_stringify_b98c93d0a190446a = function() {
      return handleError(function(arg0) {
        const ret = JSON.stringify(arg0);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_subarray_70fd07feefe14294 = function(arg0, arg1, arg2) {
      const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
      return ret;
    };
    imports.wbg.__wbg_then_b33a773d723afa3e = function(arg0, arg1, arg2) {
      const ret = arg0.then(arg1, arg2);
      return ret;
    };
    imports.wbg.__wbg_then_e22500defe16819f = function(arg0, arg1) {
      const ret = arg0.then(arg1);
      return ret;
    };
    imports.wbg.__wbg_value_dd9372230531eade = function(arg0) {
      const ret = arg0.value;
      return ret;
    };
    imports.wbg.__wbg_versions_c01dfd4722a88165 = function(arg0) {
      const ret = arg0.versions;
      return ret;
    };
    imports.wbg.__wbg_warn_e2ada06313f92f09 = function(arg0) {
      console.warn(arg0);
    };
    imports.wbg.__wbg_wasmstaticdomainvalue_new = function(arg0) {
      const ret = WasmStaticDomainValue.__wrap(arg0);
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
    imports.wbg.__wbg_wbindgencbdrop_eb10308566512b88 = function(arg0) {
      const obj = arg0.original;
      if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
      }
      const ret = false;
      return ret;
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
    imports.wbg.__wbg_wbindgenisnull_f3037694abe4d97a = function(arg0) {
      const ret = arg0 === null;
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
    imports.wbg.__wbindgen_cast_32618cb5d5fd0676 = function(arg0, arg1) {
      const ret = makeMutClosure(arg0, arg1, 180, __wbg_adapter_14);
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
    imports.wbg.__wbindgen_cast_cb9088102bce6b30 = function(arg0, arg1) {
      const ret = getArrayU8FromWasm0(arg0, arg1);
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
  function __wbg_finalize_init(instance, module2) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module2;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
  }
  function initSync(module2) {
    if (wasm !== void 0) return wasm;
    if (typeof module2 !== "undefined") {
      if (Object.getPrototypeOf(module2) === Object.prototype) {
        ({ module: module2 } = module2);
      } else {
        console.warn("using deprecated parameters for `initSync()`; pass a single object instead");
      }
    }
    const imports = __wbg_get_imports();
    if (!(module2 instanceof WebAssembly.Module)) {
      module2 = new WebAssembly.Module(module2);
    }
    const instance = new WebAssembly.Instance(module2, imports);
    return __wbg_finalize_init(instance, module2);
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
      module_or_path = new URL("alizarin_bg.wasm", import.meta.url);
    }
    const imports = __wbg_get_imports();
    if (typeof module_or_path === "string" || typeof Request === "function" && module_or_path instanceof Request || typeof URL === "function" && module_or_path instanceof URL) {
      module_or_path = fetch(module_or_path);
    }
    const { instance, module: module2 } = await __wbg_load(await module_or_path, imports);
    return __wbg_finalize_init(instance, module2);
  }
  function generateId(length) {
    const chars = "0123456789abcdef";
    let result = "";
    for (let i = 0; i < length; i++) {
      result += chars[Math.floor(Math.random() * chars.length)];
    }
    return result;
  }
  function generateTraceId() {
    return generateId(32);
  }
  function generateSpanId() {
    return generateId(16);
  }
  function now() {
    if (typeof performance !== "undefined" && performance.now) {
      return performance.now();
    }
    return Date.now();
  }
  class SpanStack {
    constructor() {
      __publicField(this, "stack", []);
    }
    push(span) {
      this.stack.push(span);
    }
    pop() {
      return this.stack.pop();
    }
    current() {
      return this.stack[this.stack.length - 1];
    }
  }
  const spanStack = new SpanStack();
  class Span {
    constructor(name, tracer, parentContext, attributes) {
      __publicField(this, "data");
      __publicField(this, "ended", false);
      __publicField(this, "tracer");
      this.tracer = tracer;
      this.data = {
        name,
        context: {
          traceId: (parentContext == null ? void 0 : parentContext.traceId) ?? generateTraceId(),
          spanId: generateSpanId(),
          parentSpanId: parentContext == null ? void 0 : parentContext.spanId
        },
        startTime: now(),
        attributes: attributes ?? {},
        status: "unset",
        events: []
      };
    }
    get context() {
      return this.data.context;
    }
    get name() {
      return this.data.name;
    }
    setAttribute(key, value) {
      if (!this.ended) {
        this.data.attributes[key] = value;
      }
      return this;
    }
    setAttributes(attributes) {
      if (!this.ended) {
        Object.assign(this.data.attributes, attributes);
      }
      return this;
    }
    addEvent(name, attributes) {
      if (!this.ended) {
        this.data.events.push({
          name,
          timestamp: now(),
          attributes
        });
      }
      return this;
    }
    setStatus(status, message) {
      if (!this.ended) {
        this.data.status = status;
        if (message) {
          this.data.attributes["status.message"] = message;
        }
      }
      return this;
    }
    recordException(error) {
      if (!this.ended) {
        this.addEvent("exception", {
          "exception.type": error.name,
          "exception.message": error.message,
          "exception.stacktrace": error.stack
        });
        this.setStatus("error", error.message);
      }
      return this;
    }
    end() {
      if (this.ended) return;
      this.ended = true;
      this.data.endTime = now();
      this.data.duration = this.data.endTime - this.data.startTime;
      if (spanStack.current() === this) {
        spanStack.pop();
      }
      this.tracer.onSpanEnd(this.data);
    }
    getData() {
      return {
        ...this.data
      };
    }
  }
  class Tracer {
    constructor(name, version2) {
      __publicField(this, "name");
      __publicField(this, "version");
      __publicField(this, "exporters", []);
      __publicField(this, "pendingSpans", []);
      __publicField(this, "batchSize", 100);
      __publicField(this, "flushIntervalMs", 5e3);
      __publicField(this, "flushTimer");
      this.name = name;
      this.version = version2;
    }
    addExporter(exporter) {
      this.exporters.push(exporter);
      return this;
    }
    startSpan(name, attributes) {
      const parentSpan = spanStack.current();
      const span = new Span(name, this, parentSpan == null ? void 0 : parentSpan.context, attributes);
      return span;
    }
    startActiveSpan(name, fnOrAttributes, maybeFn) {
      let fn;
      let attributes;
      if (typeof fnOrAttributes === "function") {
        fn = fnOrAttributes;
      } else {
        attributes = fnOrAttributes;
        fn = maybeFn;
      }
      const span = this.startSpan(name, attributes);
      spanStack.push(span);
      try {
        const result = fn(span);
        if (result instanceof Promise) {
          return result.then((value) => {
            span.setStatus("ok");
            span.end();
            return value;
          }).catch((error) => {
            span.recordException(error);
            span.end();
            throw error;
          });
        }
        span.setStatus("ok");
        span.end();
        return result;
      } catch (error) {
        span.recordException(error);
        span.end();
        throw error;
      }
    }
    onSpanEnd(spanData) {
      spanData.attributes["tracer.name"] = this.name;
      if (this.version) {
        spanData.attributes["tracer.version"] = this.version;
      }
      this.pendingSpans.push(spanData);
      if (this.pendingSpans.length >= this.batchSize) {
        this.flush();
      } else if (!this.flushTimer) {
        this.flushTimer = setTimeout(() => this.flush(), this.flushIntervalMs);
      }
    }
    flush() {
      if (this.flushTimer) {
        clearTimeout(this.flushTimer);
        this.flushTimer = void 0;
      }
      if (this.pendingSpans.length === 0) return;
      const spans = this.pendingSpans;
      this.pendingSpans = [];
      for (const exporter of this.exporters) {
        try {
          exporter(spans);
        } catch (e) {
          console.error("Exporter error:", e);
        }
      }
    }
    getCurrentSpan() {
      return spanStack.current();
    }
  }
  class TracerProvider {
    constructor() {
      __publicField(this, "tracers", /* @__PURE__ */ new Map());
      __publicField(this, "globalExporters", []);
    }
    getTracer(name, version2) {
      const key = `${name}@${version2 ?? "unknown"}`;
      if (!this.tracers.has(key)) {
        const tracer = new Tracer(name, version2);
        for (const exporter of this.globalExporters) {
          tracer.addExporter(exporter);
        }
        this.tracers.set(key, tracer);
      }
      return this.tracers.get(key);
    }
    addGlobalExporter(exporter) {
      this.globalExporters.push(exporter);
      for (const tracer of this.tracers.values()) {
        tracer.addExporter(exporter);
      }
      return this;
    }
    flushAll() {
      for (const tracer of this.tracers.values()) {
        tracer.flush();
      }
    }
  }
  const globalProvider = new TracerProvider();
  function getTracer(name, version2) {
    return globalProvider.getTracer(name, version2);
  }
  function addGlobalExporter(exporter) {
    globalProvider.addGlobalExporter(exporter);
  }
  function flushAll() {
    globalProvider.flushAll();
  }
  function consoleExporter(spans) {
    var _a2;
    console.log("=== Trace Spans ===");
    for (const span of spans) {
      const indent = span.context.parentSpanId ? "  " : "";
      console.log(`${indent}[${span.name}] ${(_a2 = span.duration) == null ? void 0 : _a2.toFixed(2)}ms`, span.status !== "unset" ? `(${span.status})` : "", Object.keys(span.attributes).length > 2 ? span.attributes : "");
    }
  }
  class SummaryExporter {
    constructor() {
      __publicField(this, "stats", /* @__PURE__ */ new Map());
      __publicField(this, "export", (spans) => {
        for (const span of spans) {
          const duration = span.duration ?? 0;
          const existing = this.stats.get(span.name);
          if (existing) {
            existing.count++;
            existing.totalMs += duration;
            existing.minMs = Math.min(existing.minMs, duration);
            existing.maxMs = Math.max(existing.maxMs, duration);
          } else {
            this.stats.set(span.name, {
              count: 1,
              totalMs: duration,
              minMs: duration,
              maxMs: duration
            });
          }
        }
      });
    }
    getSummary() {
      const result = /* @__PURE__ */ new Map();
      for (const [name, stats] of this.stats) {
        result.set(name, {
          ...stats,
          avgMs: stats.count > 0 ? stats.totalMs / stats.count : 0
        });
      }
      return result;
    }
    printSummary(label = "") {
      console.log(`=== Timing Summary ${label} ===`);
      const entries = [
        ...this.stats.entries()
      ].sort((a, b) => b[1].totalMs - a[1].totalMs);
      for (const [name, stats] of entries) {
        const avg = stats.count > 0 ? stats.totalMs / stats.count : 0;
        console.log(`${name}: count=${stats.count}, total=${stats.totalMs.toFixed(2)}ms, avg=${avg.toFixed(2)}ms, min=${stats.minMs.toFixed(2)}ms, max=${stats.maxMs.toFixed(2)}ms`);
      }
    }
    reset() {
      this.stats.clear();
    }
    mergeStats(stats, prefix = "") {
      const entries = stats instanceof Map ? stats.entries() : Object.entries(stats);
      for (const [name, stat] of entries) {
        const fullName = prefix + name;
        const existing = this.stats.get(fullName);
        if (existing) {
          existing.count += stat.count;
          existing.totalMs += stat.totalMs;
          existing.minMs = Math.min(existing.minMs, stat.minMs);
          existing.maxMs = Math.max(existing.maxMs, stat.maxMs);
        } else {
          this.stats.set(fullName, {
            count: stat.count,
            totalMs: stat.totalMs,
            minMs: stat.minMs,
            maxMs: stat.maxMs
          });
        }
      }
    }
  }
  function collectWasmPerformanceEntries(tracer) {
    if (typeof performance === "undefined" || !performance.getEntriesByType) {
      return;
    }
    const measures = performance.getEntriesByType("measure");
    for (const measure of measures) {
      if (!measure.name.startsWith("alizarin:")) {
        continue;
      }
      const spanName = measure.name.replace("alizarin:", "");
      const span = tracer.startSpan(spanName);
      if (measure.detail && typeof measure.detail === "object") {
        span.setAttributes(measure.detail);
      }
      const spanData = span.getData();
      spanData.startTime = measure.startTime;
      spanData.endTime = measure.startTime + measure.duration;
      spanData.duration = measure.duration;
      spanData.attributes["source"] = "wasm";
      span.end();
    }
    for (const measure of measures) {
      if (measure.name.startsWith("alizarin:")) {
        performance.clearMeasures(measure.name);
      }
    }
  }
  function traced(name, fn, tracer) {
    const t = tracer ?? getTracer("alizarin");
    return (...args) => {
      return t.startActiveSpan(name, (span) => {
        span.setAttribute("args.count", args.length);
        return fn(...args);
      });
    };
  }
  async function timed(name, fn, tracer) {
    const t = tracer ?? getTracer("alizarin");
    return t.startActiveSpan(name, async (_span) => {
      const result = await fn();
      return result;
    });
  }
  let _getRscvTimings = null;
  let _getAlizarinTimingStats = null;
  let _getWasmTimings = null;
  function registerRustTimingGetter(getter) {
    _getRscvTimings = getter;
  }
  function registerAlizarinTimingGetter(getter) {
    _getAlizarinTimingStats = getter;
  }
  function registerWasmTimingGetter(getter) {
    _getWasmTimings = getter;
  }
  function collectAllTimings(exporter) {
    if (_getRscvTimings) {
      try {
        const rustTimings = _getRscvTimings();
        if (rustTimings && rustTimings.size > 0) {
          const rustStats = {};
          rustTimings.forEach((value, key) => {
            rustStats[key] = {
              count: value.count || 0,
              totalMs: value.totalMs || 0,
              minMs: value.minMs || 0,
              maxMs: value.maxMs || 0
            };
          });
          exporter.mergeStats(rustStats, "rust: ");
        }
      } catch (e) {
        console.warn("Failed to collect Rust timings:", e);
      }
    }
    if (_getAlizarinTimingStats) {
      try {
        const jsTimings = _getAlizarinTimingStats();
        if (jsTimings) {
          const jsStats = {};
          if (jsTimings.wasmCalls > 0) {
            jsStats["wasm calls"] = {
              count: jsTimings.wasmCalls,
              totalMs: jsTimings.wasmTotalMs,
              minMs: jsTimings.wasmTotalMs / jsTimings.wasmCalls,
              maxMs: jsTimings.wasmTotalMs / jsTimings.wasmCalls
            };
          }
          if (jsTimings.wrapCalls > 0) {
            jsStats["wrap calls"] = {
              count: jsTimings.wrapCalls,
              totalMs: jsTimings.wrapTotalMs,
              minMs: jsTimings.wrapTotalMs / jsTimings.wrapCalls,
              maxMs: jsTimings.wrapTotalMs / jsTimings.wrapCalls
            };
          }
          if (Object.keys(jsStats).length > 0) {
            exporter.mergeStats(jsStats, "alizarin: ");
          }
        }
      } catch (e) {
        console.warn("Failed to collect alizarin JS timings:", e);
      }
    }
    if (_getWasmTimings) {
      try {
        const wasmTimings2 = _getWasmTimings();
        if (wasmTimings2 && wasmTimings2.size > 0) {
          const wasmStats = {};
          wasmTimings2.forEach((value, key) => {
            wasmStats[key] = {
              count: value.count || 0,
              totalMs: value.totalMs || 0,
              minMs: value.minMs || 0,
              maxMs: value.maxMs || 0
            };
          });
          exporter.mergeStats(wasmStats, "js: ");
        }
      } catch (e) {
        console.warn("Failed to collect detailed WASM timings:", e);
      }
    }
  }
  index = Object.freeze(Object.defineProperty({
    __proto__: null,
    Span,
    SummaryExporter,
    Tracer,
    addGlobalExporter,
    collectAllTimings,
    collectWasmPerformanceEntries,
    consoleExporter,
    flushAll,
    getTracer,
    registerAlizarinTimingGetter,
    registerRustTimingGetter,
    registerWasmTimingGetter,
    timed,
    traced
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  let wasmURL = (() => {
    try {
      return new URL("alizarin_bg.wasm", import.meta.url).href;
    } catch {
      return "alizarin_bg.wasm";
    }
  })();
  setWasmURL = function(url) {
    if (wasmInitialized) {
      throw new Error("Cannot set WASM URL after initialization");
    }
    wasmURL = url;
  };
  let wasmInitialized = false;
  let _globalWasmRdmCache = null;
  function getGlobalWasmRdmCache() {
    if (!wasmInitialized) {
      throw new Error("WASM not initialized. Call initWasm() first.");
    }
    if (!_globalWasmRdmCache) {
      _globalWasmRdmCache = new WasmRdmCache();
    }
    return _globalWasmRdmCache;
  }
  function hasGlobalWasmRdmCache() {
    return wasmInitialized && _globalWasmRdmCache !== null;
  }
  ensureWasmRdmCache = async function() {
    if (!wasmInitialized) {
      await initWasm();
    }
    return getGlobalWasmRdmCache();
  };
  function patchStringLikePrototype(proto) {
    const stringProto = String.prototype;
    const protoRecord = proto;
    for (const method of Object.getOwnPropertyNames(stringProto)) {
      if (typeof stringProto[method] === "function" && !(method in protoRecord) && method !== "constructor") {
        protoRecord[method] = function(...args) {
          const str = this.toString();
          return str[method] instanceof Function ? str[method](...args) : str[method];
        };
      }
    }
    Object.defineProperty(proto, "length", {
      get() {
        return this.toString().length;
      },
      configurable: true
    });
    if (!("valueOf" in protoRecord)) {
      protoRecord["valueOf"] = function() {
        return this.toString();
      };
    }
  }
  function applyPrototypePatches() {
    patchStringLikePrototype(StaticTranslatableString.prototype);
  }
  initWasm = async function() {
    var _a2;
    if (!wasmInitialized) {
      try {
        new StaticTranslatableString("test");
        console.log("[alizarin] WASM already available from another module instance");
        wasmInitialized = true;
        return;
      } catch {
      }
      if (typeof process !== "undefined" && ((_a2 = process.versions) == null ? void 0 : _a2.node)) {
        try {
          console.log("[alizarin] Initializing WASM in Node.js environment");
          if (wasmURL.startsWith("data:")) {
            const base64Data = wasmURL.slice(wasmURL.indexOf(",") + 1);
            const wasmBuffer2 = Buffer.from(base64Data, "base64");
            initSync({
              module: wasmBuffer2
            });
            wasmInitialized = true;
            console.log("[alizarin] WASM initialized from inline data URI in Node.js");
            applyPrototypePatches();
            registerRustTimingGetter(getRscvTimings);
            return;
          }
          const fs = await import("fs").then(async (m) => {
            await m.__tla;
            return m;
          });
          const path = await import("path").then(async (m) => {
            await m.__tla;
            return m;
          });
          const { fileURLToPath } = await import("url").then(async (m) => {
            await m.__tla;
            return m;
          });
          const moduleDir = path.dirname(fileURLToPath(import.meta.url));
          const possiblePaths = [
            path.join(moduleDir, "../pkg", "alizarin_bg.wasm"),
            path.join(process.cwd(), "pkg", "alizarin_bg.wasm"),
            path.join(moduleDir, "alizarin/pkg", "alizarin_bg.wasm"),
            path.join(moduleDir, "../../pkg", "alizarin_bg.wasm"),
            path.join(moduleDir, "../../../pkg", "alizarin_bg.wasm")
          ];
          let wasmPath;
          for (const candidate of possiblePaths) {
            if (fs.existsSync(candidate)) {
              wasmPath = candidate;
              break;
            }
          }
          if (!wasmPath) {
            const error = new Error(`Could not find alizarin_bg.wasm in any of these locations:
${possiblePaths.map((p) => `  - ${p}`).join("\n")}`);
            console.error("[alizarin]", error.message);
            throw error;
          }
          const wasmBuffer = fs.readFileSync(wasmPath);
          try {
            initSync({
              module: wasmBuffer
            });
            wasmInitialized = true;
            console.log("[alizarin] WASM initialized successfully in Node.js");
          } catch (initError) {
            const initMsg = initError instanceof Error ? initError.message : String(initError);
            if (initMsg.includes("memory already initialized") || initMsg.includes("unreachable")) {
              console.log("[alizarin] WASM already initialized (detected during initSync), continuing");
              wasmInitialized = true;
              return;
            }
            throw initError;
          }
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : String(error);
          if (errorMsg.includes("memory already initialized") || errorMsg.includes("unreachable")) {
            console.log("[alizarin] WASM already initialized (from another import), continuing");
            wasmInitialized = true;
            return;
          }
          console.error("Failed to initialize WASM in Node.js:", error);
          throw error;
        }
      } else {
        console.log("[alizarin] Initializing WASM in browser environment", {
          init: __wbg_init,
          wasmURL
        });
        try {
          await __wbg_init(wasmURL);
          console.log("[alizarin] WASM initialized successfully in browser");
        } catch (error) {
          console.error("[alizarin] Failed to initialize WASM in browser:", error);
          throw error;
        }
      }
      applyPrototypePatches();
      registerRustTimingGetter(getRscvTimings);
      wasmInitialized = true;
    }
  };
  function createStaticGraph(props, published = true) {
    const graphid = props.graphid || v4();
    const publication = published ? new StaticPublication({
      graph_id: graphid,
      notes: null,
      publicationid: v4(),
      published_time: (/* @__PURE__ */ new Date()).toISOString()
    }) : null;
    let nameForRust;
    if (props.name instanceof StaticTranslatableString) {
      nameForRust = props.name.toJSON();
    } else if (typeof props.name === "string") {
      nameForRust = props.name;
    } else {
      throw Error(`Name of graph must be string or StaticTranslatableString, not ${props.name}`);
    }
    const alias = slugify$1(typeof nameForRust === "string" ? nameForRust : JSON.stringify(nameForRust));
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
      "name": typeof nameForRust === "string" ? nameForRust : nameForRust.en || Object.values(nameForRust)[0] || "",
      "nodegroup_id": null,
      "nodeid": graphid,
      "ontologyclass": props.ontology_id || null,
      "parentproperty": null,
      "sortorder": 0,
      "sourcebranchpublication_id": null
    });
    let subtitleForRust;
    if (props.subtitle) {
      if (props.subtitle instanceof StaticTranslatableString) {
        subtitleForRust = props.subtitle.toJSON();
      } else {
        subtitleForRust = props.subtitle;
      }
    } else {
      subtitleForRust = "";
    }
    return new StaticGraph({
      author: props.author || "",
      cards: null,
      cards_x_nodes_x_widgets: null,
      color: props.color || null,
      config: props.config || {},
      deploymentdate: props.deploymentdate || null,
      deploymentfile: props.deploymentfile || null,
      description: props.description ? props.description instanceof StaticTranslatableString ? props.description : new StaticTranslatableString(props.description) : new StaticTranslatableString(""),
      edges: [],
      functions_x_graphs: [],
      graphid,
      iconclass: props.iconclass || "",
      is_editable: props.is_editable || null,
      isresource: props.isresource ?? false,
      jsonldcontext: props.jsonldcontext || null,
      name: nameForRust,
      nodegroups: [],
      nodes: root ? [
        root.copy()
      ] : [],
      ontology_id: props.ontology_id || null,
      publication,
      relatable_resource_model_ids: props.relatable_resource_model_ids || [],
      resource_2_resource_constraints: props.resource_2_resource_constraints || null,
      root,
      slug: props.slug || null,
      subtitle: subtitleForRust,
      template_id: props.template_id || "",
      version: props.version || ""
    });
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
    toJSON() {
      return {
        id: this.id,
        value: this.value
      };
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
        } else {
          value.__concept = this;
          value.__conceptId = this.id;
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
      let prefLabels = null;
      if (typeof value === "string") {
        tmpValue = value;
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
      if (prefLabels === null) {
        const valueId = generateUuidv5([
          "value"
        ], `${conceptId}/prefLabel/${tmpValue}/${lang}`);
        prefLabels = {
          [lang]: new StaticValue({
            id: valueId,
            value: tmpValue
          })
        };
      }
      const childConcepts = (children || []).map((child) => {
        if (!(child instanceof StaticConcept)) {
          return StaticConcept.fromValue(conceptScheme, child, [], {
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
      __publicField(this, "uri");
      __publicField(this, "prefLabels");
      __publicField(this, "altLabels");
      __publicField(this, "scopeNotes");
      __publicField(this, "nodeType");
      __publicField(this, "concepts");
      __publicField(this, "__allConcepts");
      __publicField(this, "__values");
      __publicField(this, "_syncedToRustCache", false);
      this.id = jsonData.id;
      this.uri = jsonData.uri;
      this.prefLabels = jsonData.prefLabels;
      this.altLabels = jsonData.altLabels;
      this.scopeNotes = jsonData.scopeNotes;
      this.nodeType = jsonData.nodeType || "ConceptScheme";
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
        collectionid: props.collectionid,
        name: collectionName,
        concepts: props.conceptScheme.children || [],
        nodeType: props.nodeType
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
        uri: props.uri,
        prefLabels,
        nodeType: props.nodeType || "ConceptScheme",
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
    ensureInCache() {
      getGlobalWasmRdmCache();
      this._ensureInRustCache();
    }
    getParentId(conceptId) {
      const cache = getGlobalWasmRdmCache();
      this._ensureInRustCache();
      return cache.getParentId(this.id, conceptId) ?? null;
    }
    getParent(conceptId) {
      var _a2;
      const parentId = (_a2 = this.getParentId) == null ? void 0 : _a2.call(this, conceptId);
      return parentId ? this.__allConcepts[parentId] ?? null : null;
    }
    getAncestors(conceptId) {
      var _a2;
      const ancestors = [];
      let currentId = conceptId;
      while (currentId) {
        const parentId = (_a2 = this.getParentId) == null ? void 0 : _a2.call(this, currentId);
        if (!parentId) break;
        const parent = this.__allConcepts[parentId];
        if (parent) {
          ancestors.push(parent);
        }
        currentId = parentId;
      }
      return ancestors;
    }
    _ensureInRustCache() {
      if (this._syncedToRustCache) return;
      const cache = getGlobalWasmRdmCache();
      if (cache.hasCollection(this.id)) {
        this._syncedToRustCache = true;
        return;
      }
      const concepts = this._toRustCacheFormat();
      cache.addCollectionFromJson(this.id, JSON.stringify(concepts));
      this._syncedToRustCache = true;
    }
    _toRustCacheFormat() {
      const result = [];
      const addConcept = (concept, parentId) => {
        var _a2;
        const prefLabels = {};
        for (const [lang, value] of Object.entries(concept.prefLabels)) {
          prefLabels[lang] = {
            id: value.id,
            value: value.value
          };
        }
        const broader = parentId ? [
          parentId
        ] : [];
        const narrower = ((_a2 = concept.children) == null ? void 0 : _a2.map((c) => c.id)) || [];
        result.push({
          id: concept.id,
          prefLabels,
          broader,
          narrower
        });
        if (concept.children) {
          for (const child of concept.children) {
            addConcept(child, concept.id);
          }
        }
      };
      for (const concept of Object.values(this.concepts)) {
        addConcept(concept, null);
      }
      return result;
    }
    toString() {
      return (this.prefLabels[getCurrentLanguage$1()] || Object.values(this.prefLabels)[0] || "").toString();
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
    StaticPublication,
    StaticResource,
    StaticResourceDescriptors,
    StaticResourceMetadata,
    StaticResourceReference,
    StaticResourceRegistry,
    StaticResourceSummary,
    StaticTile,
    StaticTranslatableString,
    StaticValue,
    createStaticGraph
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  class GraphResult {
    constructor(jsonData) {
      __publicField(this, "models");
      this.models = Object.fromEntries(Object.entries(jsonData.models).map(([k, v]) => {
        const data = {
          graphid: k,
          ...v
        };
        return [
          k,
          new StaticGraphMeta(data)
        ];
      }));
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
      const jsonText = await response.text();
      return StaticGraph.fromJsonString(jsonText);
    }
    async getResource(resourceId) {
      throw Error(`Not implemented yet: getResource(${resourceId}`);
    }
    async getCollection(collectionId) {
      throw Error(`Not implemented yet: getCollection(${collectionId}`);
    }
    async getResources(graphId, limit, _reloadIfSeen) {
      const response = await fetch(`${this.archesUrl}/resources?graph_uuid=${graphId}&format=arches-json&hide_empty_nodes=false&compact=false&limit=${limit}`);
      return await response.json();
    }
    async getResourceSummaries(graphId, limit) {
      const resources = await this.getResources(graphId, limit, false);
      return resources.map((resource) => new StaticResourceSummary({
        resourceinstanceid: resource.resourceinstance.resourceinstanceid,
        graph_id: resource.resourceinstance.graph_id,
        name: resource.resourceinstance.name,
        descriptors: resource.resourceinstance.descriptors || {},
        metadata: resource.metadata || {},
        publication_id: resource.resourceinstance.publication_id,
        principaluser_id: resource.resourceinstance.principaluser_id,
        legacyid: resource.resourceinstance.legacyid,
        graph_publication_id: resource.resourceinstance.graph_publication_id
      }));
    }
    async getResourceTiles(resourceId) {
      const resource = await this.getResource(resourceId);
      return resource.tiles || [];
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
      const jsonText = await response.text();
      return StaticGraph.fromJsonString(jsonText);
    }
    async getGraphByIdOnly(graphId) {
      const response = await fetch(`${this.archesUrl}/${this.graphIdToGraphFile(graphId)}`);
      const jsonText = await response.text();
      return StaticGraph.fromJsonString(jsonText);
    }
    async getResource(resourceId) {
      const source = `${this.archesUrl}/${this.resourceIdToFile(resourceId)}`;
      const response = await fetch(source);
      const jsonText = await response.text();
      const resource = StaticResource.fromJsonString(jsonText);
      resource.__source = source;
      return resource;
    }
    async getCollection(collectionId) {
      const response = await fetch(`${this.archesUrl}/${this.collectionIdToFile(collectionId)}`);
      return await response.json();
    }
    async getResources(graphId, limit, _reloadIfSeen) {
      const resources = [];
      const result = this.graphIdToResourcesFiles(graphId);
      const files = typeof result[Symbol.asyncIterator] === "function" || typeof result[Symbol.iterator] === "function" ? result : await result;
      for await (const file of files) {
        const source = `${this.archesUrl}/${file}`;
        const response = await fetch(source);
        const jsonText = await response.text();
        const resourceSet = StaticResource.fromBusinessDataJsonString(jsonText);
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
    async getResourceSummaries(graphId, limit) {
      const summaries = [];
      const result = this.graphIdToResourcesFiles(graphId);
      const files = typeof result[Symbol.asyncIterator] === "function" || typeof result[Symbol.iterator] === "function" ? result : await result;
      for await (const file of files) {
        const source = `${this.archesUrl}/${file}`;
        const response = await fetch(source);
        const jsonText = await response.text();
        const summarySet = StaticResourceSummary.summariesFromBusinessDataJsonString(jsonText);
        summaries.push(...summarySet);
        if (limit && summaries.length >= limit) {
          return summaries.slice(0, limit);
        }
      }
      return limit ? summaries.slice(0, limit) : summaries;
    }
    async getResourceTiles(resourceId) {
      const resource = await this.getResource(resourceId);
      return resource.tiles || [];
    }
  }
  class ArchesClientLocal extends ArchesClient {
    constructor({ allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile } = {}) {
      super();
      __publicField(this, "allGraphFile");
      __publicField(this, "graphToGraphFile");
      __publicField(this, "graphIdToGraphFile");
      __publicField(this, "graphIdToResourcesFiles");
      __publicField(this, "resourceIdToFile");
      __publicField(this, "collectionIdToFile");
      __publicField(this, "__loadedFileCache");
      this.allGraphFile = allGraphFile || (() => "tests/definitions/models/_all.json");
      this.graphToGraphFile = graphToGraphFile || ((graph) => `tests/definitions/models/${graph.graphid}.json`);
      this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `tests/definitions/models/${graphId}.json`);
      this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [
        `tests/definitions/resources/_${graphId}.json`
      ]);
      this.resourceIdToFile = resourceIdToFile || ((resourceId) => `tests/definitions/resources/${resourceId}.json`);
      this.collectionIdToFile = collectionIdToFile || ((collectionId) => `tests/definitions/collections/${collectionId}.json`);
      this.__loadedFileCache = {};
    }
    async ensureFs() {
      var _a2;
      if (typeof process === "undefined" || !((_a2 = process.versions) == null ? void 0 : _a2.node)) {
        throw new Error("ArchesClientLocal requires Node.js filesystem access. Use ArchesClientRemoteStatic for browser environments.");
      }
      return import("fs").then(async (m) => {
        await m.__tla;
        return m;
      });
    }
    async getGraphs() {
      const fs = await this.ensureFs();
      const response = await fs.promises.readFile(this.allGraphFile(), "utf8");
      return new GraphResult(await JSON.parse(response));
    }
    async getGraph(graph) {
      const fs = await this.ensureFs();
      const graphFile = this.graphToGraphFile ? this.graphToGraphFile(graph) : this.graphIdToGraphFile(graph.graphid);
      if (!graphFile) {
        return null;
      }
      const jsonText = await fs.promises.readFile(graphFile, "utf8");
      return StaticGraph.fromJsonString(jsonText);
    }
    async getGraphByIdOnly(graphId) {
      const fs = await this.ensureFs();
      const graphFile = this.graphIdToGraphFile(graphId);
      if (!graphFile) {
        return null;
      }
      const jsonText = await fs.promises.readFile(graphFile, "utf8");
      return StaticGraph.fromJsonString(jsonText);
    }
    async getResource(resourceId) {
      const fs = await this.ensureFs();
      const source = this.resourceIdToFile(resourceId);
      const response = await fs.promises.readFile(source, "utf8");
      const resource = JSON.parse(response).business_data.resources.filter((resource2) => resource2.resourceinstance.resourceinstanceid === resourceId).map((resource2) => new StaticResource(resource2))[0];
      resource.__source = source;
      return resource;
    }
    async getCollection(collectionId) {
      const fs = await this.ensureFs();
      const response = await fs.promises.readFile(this.collectionIdToFile(collectionId), "utf8");
      return await JSON.parse(response);
    }
    async getResources(graphId, limit, reloadIfSeen) {
      const fs = await this.ensureFs();
      const resources = [];
      const result = this.graphIdToResourcesFiles(graphId);
      const files = typeof result[Symbol.asyncIterator] === "function" || typeof result[Symbol.iterator] === "function" ? result : await result;
      for await (const file of files) {
        if (this.__loadedFileCache[graphId] && this.__loadedFileCache[graphId].includes(file)) {
          if (!reloadIfSeen) {
            continue;
          }
        } else {
          if (!this.__loadedFileCache[graphId]) {
            this.__loadedFileCache[graphId] = [];
          }
          this.__loadedFileCache[graphId].push(file);
        }
        const response = JSON.parse(await fs.promises.readFile(file, "utf8"));
        const source = file;
        const resourceSet = response.business_data.resources.filter((resource) => graphId === resource.resourceinstance.graph_id).map((resource) => {
          const sr = new StaticResource(resource);
          sr.__source = source;
          return sr;
        });
        resources.push(...limit ? resourceSet.slice(0, limit) : resourceSet);
        if (limit && resources.length > limit) {
          break;
        }
      }
      return resources;
    }
    async getResourceSummaries(graphId, limit) {
      const fs = await this.ensureFs();
      const summaries = [];
      const result = this.graphIdToResourcesFiles(graphId);
      const files = typeof result[Symbol.asyncIterator] === "function" || typeof result[Symbol.iterator] === "function" ? result : await result;
      for await (const file of files) {
        const response = JSON.parse(await fs.promises.readFile(file, "utf8"));
        const resourceSet = response.business_data.resources.filter((resource) => graphId === resource.resourceinstance.graph_id);
        for (const resource of resourceSet) {
          const summary = new StaticResourceSummary({
            resourceinstanceid: resource.resourceinstance.resourceinstanceid,
            graph_id: resource.resourceinstance.graph_id,
            name: resource.resourceinstance.name,
            descriptors: resource.resourceinstance.descriptors,
            metadata: resource.metadata || {},
            publication_id: resource.resourceinstance.publication_id,
            principaluser_id: resource.resourceinstance.principaluser_id,
            legacyid: resource.resourceinstance.legacyid,
            graph_publication_id: resource.resourceinstance.graph_publication_id
          });
          summaries.push(summary);
        }
        if (limit && summaries.length >= limit) {
          return summaries.slice(0, limit);
        }
      }
      return limit ? summaries.slice(0, limit) : summaries;
    }
    async getResourceTiles(resourceId) {
      const resource = await this.getResource(resourceId);
      return resource.tiles || [];
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
  let _labelResolvableDatatypes = null;
  let _collectionConfigKeys = null;
  const _extensionResolvableDatatypes = /* @__PURE__ */ new Set();
  function getLabelResolvableDatatypes() {
    if (_labelResolvableDatatypes === null) {
      _labelResolvableDatatypes = new Set(getDefaultResolvableDatatypes());
    }
    if (_extensionResolvableDatatypes.size > 0) {
      return /* @__PURE__ */ new Set([
        ..._labelResolvableDatatypes,
        ..._extensionResolvableDatatypes
      ]);
    }
    return _labelResolvableDatatypes;
  }
  registerResolvableDatatype = function(datatype) {
    _extensionResolvableDatatypes.add(datatype);
  };
  unregisterResolvableDatatype = function(datatype) {
    _extensionResolvableDatatypes.delete(datatype);
  };
  function getCollectionConfigKeys() {
    if (_collectionConfigKeys === null) {
      _collectionConfigKeys = getDefaultConfigKeys();
    }
    return _collectionConfigKeys;
  }
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
    addCollection(collection) {
      this.collections.set(collection.id, Promise.resolve(collection));
    }
    hasCollection(id) {
      return this.collections.has(id);
    }
    clearCollection(id) {
      this.collections.delete(id);
      if (hasGlobalWasmRdmCache()) {
        try {
          getGlobalWasmRdmCache().removeCollection(id);
        } catch {
        }
      }
    }
    clear() {
      this.collections.clear();
      if (hasGlobalWasmRdmCache()) {
        try {
          getGlobalWasmRdmCache().clear();
        } catch {
        }
      }
    }
    async resolveLabels(tree, graph, options = {}) {
      const { strict = false, additionalDatatypes = [], additionalConfigKeys = [] } = options;
      const resolvableDatatypes = [
        ...getLabelResolvableDatatypes(),
        ...additionalDatatypes
      ];
      const configKeys = [
        ...getCollectionConfigKeys(),
        ...additionalConfigKeys
      ];
      const treeJson = JSON.stringify(tree);
      const graphJson = JSON.stringify({
        graph: [
          graph
        ]
      });
      const aliasToCollectionMap = buildAliasToCollectionMap(graphJson, resolvableDatatypes, configKeys);
      const aliasToCollection = Object.fromEntries(aliasToCollectionMap);
      if (Object.keys(aliasToCollection).length === 0) {
        return tree;
      }
      const neededCollectionIds = findNeededCollections(treeJson, aliasToCollection);
      const collectionPromises = neededCollectionIds.map((id) => this.retrieveCollection(id));
      const loadedCollections = await Promise.all(collectionPromises);
      const cache = getGlobalWasmRdmCache();
      for (const collection of loadedCollections) {
        if (collection.ensureInCache) {
          collection.ensureInCache();
        }
      }
      const resolvedJson = cache.resolveLabels(treeJson, aliasToCollection, strict);
      return JSON.parse(resolvedJson);
    }
  }
  RDM = new ReferenceDataManager(archesClient);
  class StaticStore {
    constructor(registry) {
      __publicField(this, "_registry", null);
      __publicField(this, "archesClient", null);
      if (registry) {
        this._registry = registry;
      }
    }
    get registry() {
      if (!this._registry) {
        this._registry = new StaticResourceRegistry();
      }
      return this._registry;
    }
    set registry(registry) {
      this._registry = registry;
    }
    async getMeta(id, onlyIfCached = true) {
      const summary = this.registry.getSummary(id);
      if (summary) {
        return summary.toMetadata();
      }
      if (!onlyIfCached && this.archesClient) {
        const resource = await this.loadOne(id);
        return resource.resourceinstance;
      }
      return null;
    }
    async *loadAll(graphId, limit = void 0, useCache = true) {
      var _a2;
      let count = 0;
      const yielded = /* @__PURE__ */ new Set();
      if (useCache) {
        const resources = this.registry.getAllFullForGraph(graphId);
        for (const resource of resources) {
          if (limit && count >= limit) return;
          yielded.add(resource.resourceinstance.resourceinstanceid);
          yield resource;
          count++;
        }
      }
      if (this.archesClient && (!limit || count < limit)) {
        const remaining = limit ? limit - count : void 0;
        const resourcesJSON = await this.archesClient.getResources(graphId, remaining || 0, !useCache);
        for (let resourceJSON of resourcesJSON) {
          if (limit && count >= limit) return;
          const id = resourceJSON.resourceinstanceid || ((_a2 = resourceJSON.resourceinstance) == null ? void 0 : _a2.resourceinstanceid);
          if (id && yielded.has(id)) continue;
          let resource;
          if (resourceJSON instanceof StaticResource) {
            resource = resourceJSON;
          } else if (resourceJSON.resourceinstanceid) {
            resource = await this.archesClient.getResource(resourceJSON.resourceinstanceid);
          } else {
            resource = new StaticResource(resourceJSON);
          }
          if (resource.resourceinstance.graph_id !== graphId) continue;
          const resourceId = resource.resourceinstance.resourceinstanceid;
          this.registry.mergeFromResources([
            resource
          ], true, false);
          const fresh = this.registry.getFull(resourceId);
          if (fresh) {
            yield fresh;
            count++;
          }
        }
      }
    }
    async loadOne(id) {
      const cached = this.registry.getFull(id);
      if (cached) {
        return cached;
      }
      if (this.archesClient) {
        const resource = await this.archesClient.getResource(id);
        const resourceId = resource.resourceinstance.resourceinstanceid;
        this.registry.mergeFromResources([
          resource
        ], true, false);
        const fresh = this.registry.getFull(resourceId);
        if (!fresh) {
          throw new Error(`Resource ${id} (${resourceId}) was merged but not found in registry`);
        }
        return fresh;
      }
      throw new Error(`Resource ${id} not in registry and no archesClient available.`);
    }
    async *loadAllSummaries(graphId, limit = void 0) {
      let count = 0;
      const yielded = /* @__PURE__ */ new Set();
      const resources = this.registry.getAllFullForGraph(graphId);
      for (const resource of resources) {
        if (limit && count >= limit) return;
        yielded.add(resource.resourceinstance.resourceinstanceid);
        yield StaticResourceSummary.fromResource(resource);
        count++;
      }
      if (this.archesClient && (!limit || count < limit)) {
        const remaining = limit ? limit - count : 0;
        const summariesJSON = await this.archesClient.getResourceSummaries(graphId, remaining);
        for (const summaryJSON of summariesJSON) {
          if (limit && count >= limit) return;
          const summary = new StaticResourceSummary(summaryJSON);
          if (summary.graph_id !== graphId) continue;
          if (yielded.has(summary.resourceinstanceid)) continue;
          this.registry.insert(summary);
          yield summary;
          count++;
        }
      }
    }
    async loadTiles(id, nodegroupId) {
      const cached = this.registry.getFull(id);
      if (cached && cached.tilesLoaded) {
        const tiles = cached.tiles ?? [];
        return nodegroupId ? tiles.filter((tile) => tile.nodegroup_id === nodegroupId) : tiles;
      }
      if (this.archesClient) {
        const tiles = await this.archesClient.getResourceTiles(id);
        return nodegroupId ? tiles.filter((tile) => tile.nodegroup_id === nodegroupId) : tiles;
      }
      throw new Error(`Resource ${id} tiles not in registry and no archesClient available.`);
    }
    async ensureFullResource(id) {
      const cached = this.registry.getFull(id);
      if (cached && cached.tilesLoaded) {
        return cached;
      }
      if (this.archesClient) {
        const resource = await this.archesClient.getResource(id);
        const resourceId = resource.resourceinstance.resourceinstanceid;
        this.registry.mergeFromResources([
          resource
        ], true, false);
        const fresh = this.registry.getFull(resourceId);
        if (!fresh) {
          throw new Error(`Resource ${id} (${resourceId}) was merged but not found in registry`);
        }
        return fresh;
      }
      if (cached) {
        throw new Error(`Resource ${id} in registry but tiles not loaded.`);
      }
      throw new Error(`Resource ${id} not in registry and no archesClient available.`);
    }
    contains(id) {
      return this.registry.contains(id);
    }
    hasFull(id) {
      return this.registry.hasFull(id);
    }
  }
  staticStore = new StaticStore();
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
  const DEFAULT_LANGUAGE = "en";
  class ViewContext {
    constructor() {
      __publicField(this, "graphManager");
    }
  }
  const viewContext = new ViewContext();
  const CUSTOM_DATATYPES = /* @__PURE__ */ new Map();
  class StringTranslatedLanguage {
    constructor() {
      __publicField(this, "value", "");
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
    constructor(input) {
      __publicField(this, "datatype", "resource-instance-list");
      __publicField(this, "_");
      __publicField(this, "meta");
      let entries;
      if ("_" in input && Array.isArray(input._)) {
        entries = input._;
        this.meta = input.meta || {};
      } else if ("id" in input) {
        entries = [
          input
        ];
        this.meta = input.meta || {};
      } else {
        entries = [];
        this.meta = {};
      }
      this._ = entries.map((instance) => {
        if (instance instanceof ResourceInstanceCacheEntry) {
          return instance;
        }
        return new ResourceInstanceCacheEntry(instance);
      });
    }
  }
  class ResourceInstanceCacheEntry {
    constructor({ meta, id, type, graphId, title, descriptors }) {
      __publicField(this, "datatype", "resource-instance");
      __publicField(this, "id");
      __publicField(this, "type");
      __publicField(this, "graphId");
      __publicField(this, "title");
      __publicField(this, "descriptors");
      __publicField(this, "meta");
      this.id = id;
      this.type = type;
      this.graphId = graphId;
      this.meta = meta || {};
      this.title = this.meta.title || title;
      this.descriptors = descriptors || null;
    }
  }
  const wasmTimings = /* @__PURE__ */ new Map();
  function recordWasmTiming(label, ms) {
    let stats = wasmTimings.get(label);
    if (!stats) {
      stats = {
        count: 0,
        totalMs: 0,
        minMs: Infinity,
        maxMs: -Infinity
      };
      wasmTimings.set(label, stats);
    }
    stats.count++;
    stats.totalMs += ms;
    stats.minMs = Math.min(stats.minMs, ms);
    stats.maxMs = Math.max(stats.maxMs, ms);
  }
  function getWasmTimings() {
    return wasmTimings;
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
                throw new Error("Could not retrieve resource");
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
                throw new Error("Could not retrieve resource");
              }
            }
            const p = object.$.getOrmAttribute(k);
            return p.then((v) => {
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
    async getName(retrieveIfNeeded = false) {
      var _a2;
      return (_a2 = await this.getDescriptors(retrieveIfNeeded)) == null ? void 0 : _a2.name;
    }
    async getSlug(retrieveIfNeeded = false) {
      var _a2;
      return (_a2 = await this.getDescriptors(retrieveIfNeeded)) == null ? void 0 : _a2.slug;
    }
    async getDescription(retrieveIfNeeded = false) {
      var _a2;
      return (_a2 = await this.getDescriptors(retrieveIfNeeded)) == null ? void 0 : _a2.description;
    }
    async getMapPopup(retrieveIfNeeded = false) {
      var _a2;
      return (_a2 = await this.getDescriptors(retrieveIfNeeded)) == null ? void 0 : _a2.map_popup;
    }
    async getDescriptors(retrieveIfNeeded = false) {
      var _a2;
      if ((_a2 = this.__cacheEntry) == null ? void 0 : _a2.descriptors) {
        return this.__cacheEntry.descriptors;
      }
      if (this.$) {
        return this.$.getDescriptors();
      }
      if (retrieveIfNeeded) {
        await this.retrieve();
        if (this.$) {
          return this.$.getDescriptors();
        }
      }
      return void 0;
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
      const forJsonStart = performance.now();
      let rootJson = null;
      if (cascade) {
        let t0 = performance.now();
        if (!this.$) {
          await this.retrieve();
          if (!this.$) {
            throw new Error("Could not retrieve resource");
          }
        }
        recordWasmTiming("forJson: retrieve check", performance.now() - t0);
        t0 = performance.now();
        await this.$.populate(false);
        recordWasmTiming("forJson: populate", performance.now() - t0);
        t0 = performance.now();
        rootJson = this.$.wasmWrapper.toJson();
        recordWasmTiming("forJson: toJson", performance.now() - t0);
      }
      recordWasmTiming("forJson total (viewModels)", performance.now() - forJsonStart);
      if (!cascade && this.__cacheEntry) {
        return {
          type: this.__cacheEntry.type,
          graphId: this.__cacheEntry.graphId,
          id: this.__cacheEntry.id,
          title: this.__cacheEntry.title || void 0,
          descriptors: this.__cacheEntry.descriptors || void 0,
          meta: this.__cacheEntry.meta || void 0,
          root: rootJson
        };
      } else if (this.__) {
        return {
          type: this.__.wkrm.modelClassName,
          graphId: this.__.wkrm.graphId,
          id: this.id,
          title: void 0,
          descriptors: void 0,
          meta: void 0,
          root: rootJson
        };
      } else {
        return {
          type: "(unknown)",
          graphId: "",
          id: this.id,
          title: void 0,
          descriptors: void 0,
          meta: void 0,
          root: rootJson
        };
      }
    }
    async forDisplayJson(cascade = false, language) {
      const forJsonStart = performance.now();
      let rootJson = null;
      if (cascade) {
        let t0 = performance.now();
        if (!this.$) {
          await this.retrieve();
          if (!this.$) {
            throw new Error("Could not retrieve resource");
          }
        }
        recordWasmTiming("forDisplayJson: retrieve check", performance.now() - t0);
        t0 = performance.now();
        await this.$.populate(false);
        recordWasmTiming("forDisplayJson: populate", performance.now() - t0);
        t0 = performance.now();
        const lang = language || DEFAULT_LANGUAGE;
        rootJson = this.$.wasmWrapper.toDisplayJsonSimple(lang);
        recordWasmTiming("forDisplayJson: toDisplayJsonSimple", performance.now() - t0);
      }
      recordWasmTiming("forDisplayJson total (viewModels)", performance.now() - forJsonStart);
      if (!cascade && this.__cacheEntry) {
        return {
          type: this.__cacheEntry.type,
          graphId: this.__cacheEntry.graphId,
          id: this.__cacheEntry.id,
          title: this.__cacheEntry.title || void 0,
          descriptors: this.__cacheEntry.descriptors || void 0,
          meta: this.__cacheEntry.meta || void 0,
          root: rootJson
        };
      } else if (this.__) {
        return {
          type: this.__.wkrm.modelClassName,
          graphId: this.__.wkrm.graphId,
          id: this.id,
          title: void 0,
          descriptors: void 0,
          meta: void 0,
          root: rootJson
        };
      } else {
        return {
          type: "(unknown)",
          graphId: "",
          id: this.id,
          title: void 0,
          descriptors: void 0,
          meta: void 0,
          root: rootJson
        };
      }
    }
    async retrieve() {
      let iw;
      let mw;
      if (viewContext.graphManager) {
        const replacement = await viewContext.graphManager.getResource(this.id, true);
        iw = replacement.$;
        mw = replacement.__;
      } else {
        throw new Error("Cannot traverse resource relationships without a GraphManager");
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
              throw new Error(`Set resource instances using id, not strings in node ${node.alias}: ${value}`);
            }
          } else if (value instanceof Object && value.resourceId) {
            val = value.resourceId;
          } else if (value instanceof Map && value.get("resourceId")) {
            val = value.get("resourceId");
          } else if (value instanceof Array && value.length < 2) {
            if (value.length == 1) {
              return _ResourceInstanceViewModel.__create(tile, node, value[0], cacheEntry);
            }
          } else {
            throw new Error("Could not set resource instance from this data");
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
            throw new Error(`Cannot set an (entire) resource list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
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
      return this.toString();
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
                throw new Error("Malformed string in tile data");
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
      if (val && typeof val == "object" && val["en"] !== void 0) {
        val = val.en;
      }
      if (!tile || val === null || val === void 0 || val === "") {
        return null;
      }
      if (typeof val != "string") {
        throw new Error("Date should be a string");
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
        throw new Error("GeoJSON should be a JSON object");
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
  class StaticNodeConfigBoolean {
    constructor(wasmConfig) {
      __publicField(this, "_wasm");
      this._wasm = wasmConfig;
    }
    get i18n_properties() {
      return [];
    }
    get falseLabel() {
      return this._wasm.falseLabel;
    }
    get trueLabel() {
      return this._wasm.trueLabel;
    }
    getLabel(value, language) {
      return this._wasm.getLabel(value, language) ?? void 0;
    }
  }
  class StaticNodeConfigConcept {
    constructor(wasmConfig) {
      __publicField(this, "_wasm");
      this._wasm = wasmConfig;
    }
    get rdmCollection() {
      return this._wasm.rdmCollection;
    }
  }
  class StaticNodeConfigReference {
    constructor(wasmConfig) {
      __publicField(this, "_wasm");
      this._wasm = wasmConfig;
    }
    get controlledList() {
      return this._wasm.controlledList;
    }
    get rdmCollection() {
      return this._wasm.rdmCollection;
    }
    get multiValue() {
      return this._wasm.multiValue;
    }
    getCollectionId() {
      return this._wasm.getCollectionId() ?? void 0;
    }
  }
  class StaticNodeConfigDomain {
    constructor(wasmConfig) {
      __publicField(this, "_wasm");
      __publicField(this, "_optionsCache", null);
      this._wasm = wasmConfig;
    }
    get i18n_config() {
      return {};
    }
    get options() {
      if (!this._optionsCache) {
        this._optionsCache = this._wasm.options.map((opt) => new StaticDomainValue({
          id: opt.id,
          selected: opt.selected,
          text: opt.text
        }));
      }
      return this._optionsCache;
    }
    getSelected() {
      const wasmSelected = this._wasm.getSelected();
      if (!wasmSelected) return void 0;
      return new StaticDomainValue({
        id: wasmSelected.id,
        selected: wasmSelected.selected,
        text: wasmSelected.text
      });
    }
    valueFromId(id) {
      const wasmValue = this._wasm.valueFromId(id);
      if (!wasmValue) return void 0;
      return new StaticDomainValue({
        id: wasmValue.id,
        selected: wasmValue.selected,
        text: wasmValue.text
      });
    }
  }
  class NodeConfigManager {
    constructor() {
      __publicField(this, "_wasmManager", null);
      __publicField(this, "_cache", /* @__PURE__ */ new Map());
      __publicField(this, "_graphsLoaded", /* @__PURE__ */ new Set());
    }
    getWasmManager() {
      if (!this._wasmManager) {
        this._wasmManager = new WasmNodeConfigManager();
      }
      return this._wasmManager;
    }
    loadFromGraph(graph) {
      const graphId = graph.graphid;
      if (this._graphsLoaded.has(graphId)) return;
      this.getWasmManager().fromGraph(graph);
      this._graphsLoaded.add(graphId);
    }
    retrieve(node) {
      const { nodeid, datatype } = node;
      if (this._cache.has(nodeid)) {
        return this._cache.get(nodeid) ?? null;
      }
      const config = this.getWasmManager().hasConfig(nodeid) ? this.getConfigFromWasm(nodeid, datatype) : null;
      this._cache.set(nodeid, config);
      return config;
    }
    getConfigFromWasm(nodeid, datatype) {
      const wasmManager = this.getWasmManager();
      switch (datatype) {
        case "boolean": {
          const wasm2 = wasmManager.getBoolean(nodeid);
          return wasm2 ? new StaticNodeConfigBoolean(wasm2) : null;
        }
        case "domain-value":
        case "domain-value-list": {
          const wasm2 = wasmManager.getDomain(nodeid);
          return wasm2 ? new StaticNodeConfigDomain(wasm2) : null;
        }
        case "concept":
        case "concept-list": {
          const wasm2 = wasmManager.getConcept(nodeid);
          return wasm2 ? new StaticNodeConfigConcept(wasm2) : null;
        }
        case "reference": {
          const wasm2 = wasmManager.getReference(nodeid);
          return wasm2 ? new StaticNodeConfigReference(wasm2) : null;
        }
        default:
          return null;
      }
    }
    clear() {
      this._cache.clear();
      this._graphsLoaded.clear();
      if (this._wasmManager) {
        this._wasmManager.clear();
      }
    }
    hasGraph(graphId) {
      return this._graphsLoaded.has(graphId);
    }
  }
  const nodeConfigManager = new NodeConfigManager();
  nodeConfig = Object.freeze(Object.defineProperty({
    __proto__: null,
    NodeConfigManager,
    StaticNodeConfigBoolean,
    StaticNodeConfigConcept,
    StaticNodeConfigDomain,
    StaticNodeConfigReference,
    nodeConfigManager
  }, Symbol.toStringTag, {
    value: "Module"
  }));
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
        throw new Error(`Cannot form boolean value for ${node.nodeid} without config`);
      }
      if (typeof val !== "boolean" && val !== 0 && val !== 1) {
        throw new Error(`Refusing to use truthiness for value ${val} in boolean`);
      }
      const bool = new BooleanViewModel(val ? true : false, config);
      return bool;
    }
    __asTileData() {
      return this.valueOf();
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
              throw new Error(`A URL must be null or have a 'url' field: ${value}`);
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
        throw new Error(`Unrecognised URL type: ${val}`);
      }
      const str = new UrlViewModel(url);
      return str;
    }
    __asTileData() {
      return this.forJson();
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
                const graphLoaded = nodeConfigManager.hasGraph(node.graph_id);
                const configType = config ? "non-domain config" : "no config";
                throw new Error(`Cannot form domain value for ${node.nodeid} (${node.alias || "no alias"}): ${configType}. Graph ${node.graph_id} loaded: ${graphLoaded}. Ensure nodeConfigManager.loadFromGraph(graph) is called before creating ViewModels.`);
              }
              val = config.valueFromId(value) || null;
            } else {
              throw new Error("Set domain values using values from domain lists, not strings");
            }
          } else {
            throw new Error("Could not set domain value from this data");
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
            throw new Error(`Cannot set an (entire) domain list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
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
        } else {
          value = [];
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
  class ConceptValueViewModel extends String {
    constructor(value, collectionId) {
      super(value.value);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value");
      __publicField(this, "_collectionId", null);
      this._value = value;
      this._collectionId = collectionId ?? null;
    }
    async parent() {
      const value = await this._value;
      const conceptId = value.__conceptId;
      if (!conceptId || !this._collectionId) {
        return null;
      }
      const collection = await RDM.retrieveCollection(this._collectionId);
      if (!collection.getParentId) {
        throw new Error(`Collection ${this._collectionId} does not support hierarchy lookups. Ensure WASM is initialized and the collection is a StaticCollection.`);
      }
      const parentId = collection.getParentId(conceptId);
      if (!parentId) {
        return null;
      }
      const parentConcept = collection.__allConcepts[parentId];
      if (!(parentConcept == null ? void 0 : parentConcept.getPrefLabel)) {
        return null;
      }
      const parentValue = parentConcept.getPrefLabel();
      return new ConceptValueViewModel(parentValue, this._collectionId);
    }
    async ancestors() {
      const result = [];
      let current = await this.parent();
      while (current !== null) {
        result.push(current);
        current = await current.parent();
      }
      return result;
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
        conceptId: value.__conceptId ?? null
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
        throw new Error(`Node ${node.alias} (${node.nodeid}) missing rdmCollection in config`);
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
              throw new Error("Recognizing value as StaticConcept, but no getPrefLabel member");
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
                return new ConceptValueViewModel(val, collectionId);
              } else {
                const collection = RDM.retrieveCollection(collectionId);
                return collection.then((collection2) => {
                  if (!collection2.getConceptValue) {
                    throw new Error(`Collection ${collection2.id} must be a StaticCollection here, not a key/value object`);
                  }
                  const val2 = collection2.getConceptValue(value);
                  if (!val2) {
                    console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
                  }
                  tile.data.set(nodeid, val2 ? val2.id : null);
                  if (!tile || !val2) {
                    return null;
                  }
                  const str2 = new ConceptValueViewModel(val2, collectionId);
                  return str2;
                });
              }
            } else {
              throw new Error(`Set concepts using values from collections, not strings: ${value}`);
            }
          } else {
            throw new Error("Could not set concept from this data");
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
      const str = new ConceptValueViewModel(val, collectionId);
      return str;
    }
    async __asTileData() {
      const value = await this._value;
      return value ? value.id : null;
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
          throw new Error(`Cannot set an (entire) concept list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
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
  _e = Symbol.toPrimitive;
  const _NodeViewModel = class _NodeViewModel {
    constructor(parentPseudo, parentWkrm) {
      __publicField(this, "then");
      __publicField(this, _e);
      __publicField(this, "__parentPseudo");
      __publicField(this, "__parentWkrm");
      this.__parentPseudo = parentPseudo;
      this.__parentWkrm = parentWkrm;
      return new Proxy(this, {
        set: (object, key, value) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
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
          const k = typeof key === "symbol" ? key.description || "" : key;
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
        }
      });
    }
    __forJsonCache() {
      return null;
    }
    async toString() {
      if (!this.__parentPseudo) {
        return "[NodeViewModel]";
      }
      const alias = this.__parentPseudo.alias;
      return alias || "[unnamed]";
    }
    async __getEdgeTo(key) {
      const childNode = this.__parentPseudo.childNodes.get(key);
      if (!childNode) {
        throw new Error(`Child node key ${key} missing`);
      }
      const domainId = this.__parentPseudo.nodeid;
      const rangeId = childNode.nodeid;
      const edges = this.__parentWkrm.graph.edges.filter((edge) => edge.domainnode_id === domainId && edge.rangenode_id === rangeId);
      if (edges.length !== 1) {
        throw new Error(`Number of edges from ${domainId}->${rangeId} != 1`);
      }
      return edges[0];
    }
    async __get(key) {
      const pseudo = this.__parentWkrm.createPseudoNodeChild(key, this.__parentPseudo);
      return _NodeViewModel.__create(pseudo, this.__parentWkrm);
    }
    static async __create(pseudo, parent) {
      const node = new _NodeViewModel(pseudo, parent);
      return node;
    }
  };
  let NodeViewModel = _NodeViewModel;
  const timingStats = {
    wasmCalls: 0,
    wasmTotalMs: 0,
    wrapCalls: 0,
    wrapTotalMs: 0
  };
  resetTimingStats = function() {
    timingStats.wasmCalls = 0;
    timingStats.wasmTotalMs = 0;
    timingStats.wrapCalls = 0;
    timingStats.wrapTotalMs = 0;
  };
  getTimingStats = function() {
    return {
      ...timingStats,
      wasmAvgMs: timingStats.wasmCalls > 0 ? timingStats.wasmTotalMs / timingStats.wasmCalls : 0,
      wrapAvgMs: timingStats.wrapCalls > 0 ? timingStats.wrapTotalMs / timingStats.wrapCalls : 0
    };
  };
  logTimingStats = function(label = "") {
    const stats = getTimingStats();
    console.log(`[timing-stats] ${label} wasm: ${stats.wasmCalls} calls, ${stats.wasmTotalMs.toFixed(1)}ms total (${stats.wasmAvgMs.toFixed(2)}ms avg) | wrap: ${stats.wrapCalls} calls, ${stats.wrapTotalMs.toFixed(1)}ms total`);
  };
  class SemanticViewModel {
    constructor(parentWkri, tile, node) {
      __publicField(this, "_");
      __publicField(this, "then");
      __publicField(this, _f);
      __publicField(this, "__parentPseudo");
      __publicField(this, "__childValues");
      __publicField(this, "__parentWkri");
      __publicField(this, "__tile");
      __publicField(this, "__node");
      this.__childValues = /* @__PURE__ */ new Map();
      this.__parentWkri = parentWkri;
      this.__tile = tile;
      this.__node = node;
      return new Proxy(this, {
        set: (object, key, value) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (k.startsWith("__") || key in object) {
            object[k] = value;
            return true;
          }
          throw Error(`Setting semantic values via proxy (key: ${String(key)}) is not supported`);
        },
        get: (object, key) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            return object[key];
          } else if (k.startsWith("__") || k in object) {
            return object[k];
          }
          if (k == "length") {
            throw Error("TODO");
          }
          return new AttrPromise((resolve) => {
            const p = object.__get(k);
            p.then(resolve);
          });
        }
      });
    }
    get [(_f = Symbol.toPrimitive, Symbol.toStringTag)]() {
      return "SemanticViewModel";
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
      var _a2;
      if (!this.__parentWkri || !this.__parentWkri.$) {
        return {};
      }
      const wasmWrapper = this.__parentWkri.$.wasmWrapper;
      const model = this.__parentWkri.$.model;
      const t0 = performance.now();
      const missingNodegroups = wasmWrapper.getMissingNodegroupsForChildren(this.__node.nodeid);
      timingStats.wasmCalls++;
      timingStats.wasmTotalMs += performance.now() - t0;
      if (missingNodegroups.length > 0) {
        const tileLoader = wasmWrapper.getTileLoader();
        if (tileLoader) {
          for (const nodegroupId of missingNodegroups) {
            const t1 = performance.now();
            const tiles = await tileLoader(nodegroupId);
            timingStats.wasmTotalMs += performance.now() - t1;
            if (tiles && tiles.length > 0) {
              const t2 = performance.now();
              wasmWrapper.appendTiles(tiles);
              timingStats.wasmCalls++;
              timingStats.wasmTotalMs += performance.now() - t2;
            }
          }
        }
      }
      const t3 = performance.now();
      const childValuesMap = wasmWrapper.getAllSemanticChildValues(((_a2 = this.__tile) == null ? void 0 : _a2.tileid) || null, this.__node.nodeid, this.__node.nodegroup_id || null);
      timingStats.wasmCalls++;
      timingStats.wasmTotalMs += performance.now() - t3;
      const t4 = performance.now();
      const wrappedChildren = [];
      for (const [alias, rustValue] of childValuesMap.entries()) {
        if (rustValue === null || rustValue === void 0) {
          continue;
        }
        const child = wrapRustPseudo(rustValue, this.__parentWkri, model);
        timingStats.wrapCalls++;
        if (child === null || child === void 0) {
          continue;
        }
        child.parentNode = this.__parentPseudo || null;
        this.__childValues.set(alias, child);
        wrappedChildren.push([
          alias,
          child
        ]);
      }
      const valuePromises = wrappedChildren.map(([alias, child]) => child.getValue().then((value) => [
        alias,
        value
      ]));
      const resolvedValues = await Promise.all(valuePromises);
      const result = {};
      for (const [alias, value] of resolvedValues) {
        result[alias] = value;
      }
      timingStats.wrapTotalMs += performance.now() - t4;
      return result;
    }
    async __get(key) {
      const childValue = await this.__getChildValue(key);
      if (!childValue) {
        return null;
      }
      return childValue.getValue();
    }
    __has(key) {
      const childAliases = this.__parentWkri.$.model.getChildNodeAliases(this.__node.nodeid);
      return childAliases.includes(key);
    }
    async __getChildValue(key) {
      const parent = this.__parentWkri;
      const tile = this.__tile;
      const node = this.__node;
      const wasmWrapper = parent.$.wasmWrapper;
      const childAliases = parent.$.model.getChildNodeAliases(node.nodeid);
      if (!childAliases.includes(key)) {
        throw Error(`Semantic node does not have this key: ${key} (${childAliases.join(", ")})`);
      }
      if (this.__childValues.has(key)) {
        return this.__childValues.get(key);
      }
      let rustValue;
      const t0 = performance.now();
      try {
        rustValue = wasmWrapper.getSemanticChildValue((tile == null ? void 0 : tile.tileid) || null, node.nodeid, node.nodegroup_id || null, key);
        timingStats.wasmCalls++;
        timingStats.wasmTotalMs += performance.now() - t0;
      } catch (e) {
        const errorStr = (e == null ? void 0 : e.message) || String(e);
        if (errorStr.startsWith("TILES_NOT_LOADED:")) {
          const nodegroupId = errorStr.split(":")[1].split(" ")[0];
          const tileLoader = wasmWrapper.getTileLoader();
          if (tileLoader) {
            const t1 = performance.now();
            const tiles = await tileLoader(nodegroupId);
            timingStats.wasmTotalMs += performance.now() - t1;
            if (tiles && tiles.length > 0) {
              const t2 = performance.now();
              wasmWrapper.appendTiles(tiles);
              timingStats.wasmCalls++;
              timingStats.wasmTotalMs += performance.now() - t2;
            }
          }
          const t3 = performance.now();
          rustValue = wasmWrapper.getSemanticChildValue((tile == null ? void 0 : tile.tileid) || null, node.nodeid, node.nodegroup_id || null, key);
          timingStats.wasmCalls++;
          timingStats.wasmTotalMs += performance.now() - t3;
        } else {
          throw e;
        }
      }
      const t4 = performance.now();
      const child = wrapRustPseudo(rustValue, parent, parent.$.model);
      timingStats.wrapCalls++;
      timingStats.wrapTotalMs += performance.now() - t4;
      if (child === null || child === void 0) {
        return child;
      }
      child.parentNode = this.__parentPseudo || null;
      this.__childValues.set(key, child);
      return child;
    }
    static async __create(tile, node, _value, parent) {
      return new SemanticViewModel(parent, tile, node);
    }
    async __asTileData() {
      const relationships = [];
      for (const [_, child] of this.__childValues.entries()) {
        if (child && typeof child.getTile === "function") {
          const [, childRelationships] = await child.getTile();
          relationships.push(...childRelationships);
        }
      }
      return [
        null,
        relationships
      ];
    }
  }
  async function getViewModel(parentPseudo, tile, node, data, parent, isInner = false) {
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
          vm = await SemanticViewModel.__create(tile, node, data, parent);
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
    ConceptListCacheEntry,
    ConceptListViewModel,
    ConceptValueCacheEntry,
    ConceptValueViewModel,
    DEFAULT_LANGUAGE,
    DateViewModel,
    DomainValueListViewModel,
    DomainValueViewModel,
    EDTFViewModel,
    GeoJSONViewModel,
    NodeViewModel,
    NonLocalizedStringViewModel,
    NumberViewModel,
    ResourceInstanceCacheEntry,
    ResourceInstanceListCacheEntry,
    ResourceInstanceListViewModel,
    ResourceInstanceViewModel,
    SemanticViewModel,
    StringTranslatedLanguage,
    StringViewModel,
    Url,
    UrlViewModel,
    ViewContext,
    getViewModel,
    viewContext
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  class PseudoUnavailable {
    constructor(node) {
      __publicField(this, "parentValue", null);
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
  class PseudoValue {
    constructor(wasm2, parent) {
      __publicField(this, "_wasm");
      __publicField(this, "parent");
      __publicField(this, "_parentValue", null);
      __publicField(this, "_inner", null);
      __publicField(this, "_cachedValue", null);
      __publicField(this, "_snapshot", null);
      if (!parent) {
        throw Error("Must have a parent or parent class for a pseudo-node");
      }
      this._wasm = wasm2;
      this.parent = parent;
      this._wasm.parent = parent;
    }
    getSnapshot() {
      if (!this._snapshot) {
        this._snapshot = this._wasm.toSnapshot();
      }
      return this._snapshot;
    }
    get datatype() {
      return this.getSnapshot().datatype;
    }
    get isInner() {
      return this.getSnapshot().isInner;
    }
    get isOuter() {
      return this.getSnapshot().isOuter;
    }
    get tile() {
      return this._wasm.tile;
    }
    set tile(t) {
      this._wasm.tile = t;
      this._snapshot = null;
    }
    get value() {
      return this._wasm.value;
    }
    get valueLoaded() {
      const v = this.getSnapshot().valueLoaded;
      if (v === void 0) return void 0;
      return v;
    }
    get accessed() {
      return this.getSnapshot().accessed;
    }
    get independent() {
      return this.getSnapshot().independent;
    }
    get originalTile() {
      return this._wasm.tile;
    }
    get node() {
      return this._wasm;
    }
    get nodeid() {
      return this.getSnapshot().nodeId;
    }
    get inner() {
      if (this.getSnapshot().isOuter && !this._inner) {
        const wasmInner = this._wasm.inner;
        if (wasmInner) {
          this._inner = PseudoValue.fromWasm(wasmInner, this.parent);
          this._inner._parentValue = this;
        }
      }
      return this._inner;
    }
    get parentValue() {
      return this._parentValue;
    }
    set parentValue(newParentValue) {
      this._parentValue = newParentValue;
    }
    isIterable() {
      return this._wasm.isIterable();
    }
    describeField() {
      let fieldName = this._wasm.name;
      if (this.parent && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
      }
      return fieldName;
    }
    describeFieldGroup() {
      let fieldName = this._wasm.name;
      const nodegroupId = this._wasm.nodegroupId;
      if (this.parent && nodegroupId && this.parent.$) {
        const nodegroupName = this.parent.$.model.getNodegroupName(nodegroupId);
        if (nodegroupName && this.parent.__) {
          fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroupName}`;
        }
      }
      return fieldName;
    }
    static fromWasm(wasm2, parent) {
      const pv = new PseudoValue(wasm2, parent);
      return pv;
    }
    static create(nodeOrAlias, tile, value, parent) {
      if (!parent || !parent.__) {
        throw new Error("Must have a parent or parent class for a pseudo-node");
      }
      const alias = typeof nodeOrAlias === "string" ? nodeOrAlias : nodeOrAlias.alias;
      const wasm2 = parent.__.createPseudoValue(alias, tile, parent);
      return PseudoValue.fromWasm(wasm2, parent);
    }
    getParentTileId() {
      const tile = this._wasm.tile;
      return tile ? tile.parenttile_id : null;
    }
    async getTile() {
      await this.updateValue();
      let relationships = [];
      if (this.inner) {
        [this.tile, relationships] = await this.inner.getTile();
      }
      let tileValue;
      if (this._cachedValue !== null) {
        const value = await this._cachedValue;
        if (value !== null) {
          const [newTileValue, ownRelationships] = await value.__asTileData();
          tileValue = newTileValue;
          relationships = [
            ...relationships,
            ...ownRelationships
          ];
        } else {
          tileValue = null;
        }
      } else {
        tileValue = null;
      }
      const tile = this._wasm.tile;
      if (!tile) {
        throw Error("No tile available");
      }
      tile.data = tile.data || /* @__PURE__ */ new Map();
      const nodeid = this._wasm.nodeId;
      if (tileValue === null) {
        if (tile.data.has(nodeid)) {
          tile.data.delete(nodeid);
        }
      } else {
        tile.data.set(nodeid, tileValue);
      }
      const returnTile = this.independent ? tile : null;
      return [
        returnTile,
        relationships
      ];
    }
    clear() {
      this._wasm.clear();
      this._cachedValue = null;
      const tile = this._wasm.tile;
      if (tile && tile.data && tile.data.has(this._wasm.nodeId)) {
        tile.data.delete(this._wasm.nodeId);
      }
    }
    updateValue(newTile) {
      if (newTile) {
        this._wasm.tile = newTile;
      }
      if (this._cachedValue !== null) {
        return this._cachedValue;
      }
      if (!this._wasm.tile && this.inner) {
        this._cachedValue = new AttrPromise(async (resolve) => {
          const [innerTile] = await this.inner.getTile();
          this._wasm.tile = innerTile;
          const result = await this._updateValueReal();
          resolve(result);
        });
        return this._cachedValue;
      }
      this._cachedValue = this._updateValueReal();
      return this._cachedValue;
    }
    _updateValueReal() {
      const snapshot = this.getSnapshot();
      if (!this._wasm.tile) {
        const nodegroupId = snapshot.nodegroupId || "";
        const sortorder = snapshot.sortorder;
        this._wasm.tile = new StaticTile({
          nodegroup_id: nodegroupId,
          tileid: null,
          data: /* @__PURE__ */ new Map(),
          sortorder,
          resourceinstance_id: "",
          parenttile_id: null,
          provisionaledits: null,
          ensureId: () => ""
        });
      }
      let data;
      const currentTile = this._wasm.tile;
      const nodeid = snapshot.nodeId;
      if (currentTile && currentTile.data !== null && currentTile.data.has(nodeid) && snapshot.datatype !== "semantic") {
        data = currentTile.data.get(nodeid);
      } else {
        data = snapshot.tileData ?? null;
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
      const vmPromise = getViewModel(this, currentTile, this._wasm.node, data, this.parent, this.isInner);
      const resolveAttr = (vm) => {
        if (vm !== null && vm instanceof Object) {
          vm.__parentPseudo = this;
          if (this.isOuter && this.inner) {
            vm._ = this.inner.getValue();
          }
          this.syncTileData(vm);
        }
        return vm;
      };
      return new AttrPromise((resolve) => {
        vmPromise.then((vm) => resolve(resolveAttr(vm)));
      });
    }
    syncTileData(vm) {
      if (vm === null) {
        this._wasm.setTileData(null);
        return;
      }
      if (typeof vm.__asTileData === "function") {
        const tileData = vm.__asTileData();
        if (tileData instanceof Promise) {
          tileData.then((data) => {
            this._wasm.setTileData(data);
          }).catch((e) => {
            console.warn("syncTileData: failed to get tile data", e);
          });
        } else {
          this._wasm.setTileData(tileData);
        }
      }
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
      const value = await this._cachedValue;
      if (value && value instanceof Object && "getChildTypes" in value && typeof value.getChildTypes === "function") {
        childTypes = value.getChildTypes();
      }
      if (this.inner) {
        Object.assign(childTypes, await this.inner.getChildTypes());
      }
      return childTypes;
    }
    getChildren(direct = null) {
      let children = [];
      if (this._cachedValue) ;
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
      __publicField(this, "parentValue", null);
      __publicField(this, "tile");
      __publicField(this, "parenttileId");
      __publicField(this, "ghostChildren", null);
      __publicField(this, "isOuter", false);
      __publicField(this, "isSingle", false);
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
        const nodegroupName = this.parent.$.model.getNodegroupName(this.node.nodegroup_id);
        if (nodegroupName && this.parent.__) {
          fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroupName}`;
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
      const resolved = await Promise.all(array);
      if (this.isSingle) {
        return resolved.length > 0 ? resolved[0] : null;
      }
      return resolved;
    }
    getValue() {
      if (this.isSingle) {
        if (this.length > 0) {
          return this[0];
        } else {
          return new AttrPromise((resolve) => resolve(null));
        }
      }
      return new AttrPromise((resolve) => resolve(this));
    }
    toString() {
      return `<PL: ${this.length}>`;
    }
  }
  function wrapRustPseudo(rustValue, wkri, model) {
    if (rustValue === null || rustValue === void 0) {
      return rustValue === null ? null : void 0;
    }
    if ("getAllValues" in rustValue && typeof rustValue.getAllValues === "function") {
      const wasmList = rustValue;
      const wasmValues = wasmList.getAllValues();
      const list = new PseudoList();
      if (wasmValues.length > 0) {
        const firstValue = wasmValues[0];
        const nodeId = firstValue.nodeId;
        const node = model.getNodeObjectFromId(nodeId);
        list.initialize(node, wkri);
      }
      list.isSingle = wasmList.isSingle;
      for (const wasmValue of wasmValues) {
        const pseudoValue = PseudoValue.fromWasm(wasmValue, wkri);
        list.push(pseudoValue.getValue());
      }
      return list;
    } else {
      return PseudoValue.fromWasm(rustValue, wkri);
    }
  }
  class ConfigurationOptions {
    constructor() {
      __publicField(this, "graphs", null);
      __publicField(this, "eagerLoadGraphs", false);
      __publicField(this, "defaultAllowAllNodegroups", false);
    }
  }
  class ResourceInstanceWrapper {
    constructor(wkri, model, resource, pruneTiles = true, lazy = false, assumeTilesComprehensiveForNodegroup = true) {
      __publicField(this, "wkri");
      __publicField(this, "model");
      __publicField(this, "wasmWrapper");
      __publicField(this, "resource");
      __publicField(this, "_pseudoCache", /* @__PURE__ */ new Map());
      __publicField(this, "cache");
      __publicField(this, "scopes");
      __publicField(this, "metadata");
      __publicField(this, "tilesLoaded", false);
      __publicField(this, "pruneTiles", true);
      const constructorStart = performance.now();
      this.wkri = wkri;
      this.model = model;
      this.pruneTiles = pruneTiles;
      let t0 = performance.now();
      if (resource) {
        this.wasmWrapper = newWASMResourceInstanceWrapperForResource(resource);
        this.resource = resource;
        recordWasmTiming("newWASMResourceInstanceWrapperForResource", performance.now() - t0);
      } else {
        this.wasmWrapper = newWASMResourceInstanceWrapperForModel(model.wkrm.graphId);
        recordWasmTiming("newWASMResourceInstanceWrapperForModel", performance.now() - t0);
      }
      this._pseudoCache = /* @__PURE__ */ new Map();
      this.cache = resource ? resource.__cache : void 0;
      this.scopes = resource ? resource.__scopes : void 0;
      this.metadata = resource ? resource.metadata : void 0;
      t0 = performance.now();
      this.wasmWrapper.setLazy(lazy);
      recordWasmTiming("setLazy", performance.now() - t0);
      if (resource) {
        const resourceId = resource.resourceinstance.resourceinstanceid;
        t0 = performance.now();
        this.wasmWrapper.setTileLoader((nodegroupId) => {
          const tiles = staticStore.loadTiles(resourceId, nodegroupId);
          return tiles;
        });
        recordWasmTiming("setTileLoader", performance.now() - t0);
      }
      t0 = performance.now();
      const tilesLoaded = this.wasmWrapper.tilesLoaded();
      recordWasmTiming("tilesLoaded (constructor)", performance.now() - t0);
      if (!tilesLoaded && resource && resource.tilesLoaded) {
        try {
          t0 = performance.now();
          this.wasmWrapper.loadTilesFromResource(resource, assumeTilesComprehensiveForNodegroup);
          recordWasmTiming("loadTilesFromResource", performance.now() - t0);
        } catch (e) {
          console.error("Failed to load tiles into WASM:", e);
        }
      }
      if (pruneTiles && resource) {
        t0 = performance.now();
        this.pruneResourceTiles();
        recordWasmTiming("pruneResourceTiles", performance.now() - t0);
      }
      recordWasmTiming("constructor total", performance.now() - constructorStart);
    }
    async ensureTilesLoaded() {
      if (!this.wasmWrapper.tilesLoaded()) {
        if (this.resource && this.resource.tiles && this.resource.tiles.length > 0) {
          try {
            this.wasmWrapper.loadTilesFromResource(this.resource, true);
            return;
          } catch (e) {
            console.error("Failed to load tiles from existing resource:", e);
          }
        }
        const resourceId = this.wasmWrapper.getResourceId();
        try {
          const fullResource = await staticStore.ensureFullResource(resourceId);
          if (fullResource && fullResource.tilesLoaded) {
            try {
              this.wasmWrapper.loadTilesFromResource(fullResource, true);
            } catch (e) {
              console.error("Failed to load tiles from resource:", e);
              const tiles = fullResource.tiles || [];
              this.wasmWrapper.loadTilesWasm(tiles, true);
            }
          }
        } catch (e) {
          console.warn(`Could not load tiles for resource ${resourceId} from registry:`, e);
        }
      }
    }
    pruneResourceTiles() {
      this.wasmWrapper.pruneResourceTiles();
    }
    async retrievePseudo(key, dflt = null, raiseError = false) {
      let result = this._pseudoCache.get(key);
      if (result instanceof Promise) {
        result = await result;
      }
      if (Array.isArray(result) && result.length > 0) {
        return result;
      }
      const rustValue = this.wasmWrapper.getCachedPseudo(key);
      if (rustValue) {
        const wrappedValue = wrapRustPseudo(rustValue, this.wkri, this.model);
        const wrappedArray = [
          wrappedValue
        ];
        this._pseudoCache.set(key, wrappedArray);
        return wrappedArray;
      }
      if (raiseError) {
        throw Error(`Unset key ${key}`);
      }
      return dflt;
    }
    async hasPseudo(key) {
      const value = await this.retrievePseudo(key, null, false);
      return value !== null && value !== void 0;
    }
    setPseudo(key, value) {
      this._pseudoCache.set(key, value);
    }
    async setDefaultPseudo(key, value) {
      const existingValue = await this.retrievePseudo(key, null, false);
      if (existingValue !== null) {
        return existingValue;
      }
      this._pseudoCache.set(key, value);
      return value;
    }
    async loadNodes(aliases) {
      for (const key of aliases) {
        await this.retrievePseudo(key);
      }
    }
    getName(update = false) {
      this.getDescriptors(update);
      return this.wasmWrapper.getName();
    }
    getDescriptors(update = false) {
      return this.wasmWrapper.getDescriptors(update);
    }
    addPseudo(childNode, tile) {
      const key = childNode.alias;
      if (!key) {
        throw Error(`Cannot add a pseudo node with no alias ${childNode.nodeid}`);
      }
      try {
        const isPermitted = this.model.isNodegroupPermitted(childNode.nodegroup_id || "", tile);
        const rustValue = this.wasmWrapper.makePseudoValue(key, (tile == null ? void 0 : tile.tileid) || null, isPermitted, false);
        if (rustValue === null || rustValue === void 0) {
          const child2 = new PseudoUnavailable(childNode);
          this.setDefaultPseudo(key, []).then((val) => val.push(child2));
          return child2;
        }
        const child = wrapRustPseudo(rustValue, this.wkri, this.model);
        this.setDefaultPseudo(key, []).then((val) => val.push(child));
        return child;
      } catch (e) {
        console.error("Rust makePseudoValue failed:", e);
        throw new Error(`Rust makePseudoValue failed: ${e}. This should not happen - check Rust implementation.`);
      }
    }
    allEntries() {
      return this._pseudoCache.entries();
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
      if (this.resource === null) ;
      else {
        promise = new Promise((resolve) => {
          resolve();
        });
      }
      return new AttrPromise((resolve) => {
        return promise.then(() => this.getRootViewModel()).then((root) => resolve(root[key]));
      });
    }
    async getRoot() {
      const node = this.model.getRootNode();
      if (!node) {
        return void 0;
      }
      const alias = node.alias;
      if (!(typeof alias == "string")) {
        throw Error(`Alias missing for node ${node.nodeid}`);
      }
      const rootPseudo = this.wasmWrapper.getRootPseudo();
      if (rootPseudo) {
        return PseudoValue.fromWasm(rootPseudo, this.wkri);
      }
      const rustValue = this.wasmWrapper.makePseudoValue(alias, null, true, false);
      const value = wrapRustPseudo(rustValue, this.wkri, this.model);
      return value;
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
    async populate(lazy) {
      var _a2, _b2;
      const populateStart = performance.now();
      const nodegroupObjs = this.model.getNodegroupObjects();
      const rootNode = this.model.getRootNode();
      if (rootNode.alias === null) {
        throw Error("Cannot populate a model with no proper root node");
      }
      try {
        if (!lazy) {
          let t02 = performance.now();
          const loaded = this.wasmWrapper.tilesLoaded();
          recordWasmTiming("tilesLoaded (populate)", performance.now() - t02);
          if (!loaded) {
            t02 = performance.now();
            await this.ensureTilesLoaded();
            recordWasmTiming("ensureTilesLoaded", performance.now() - t02);
          }
        }
        const nodegroupIds = [
          ...nodegroupObjs.keys()
        ];
        const t0 = performance.now();
        const result = this.wasmWrapper.populate(lazy, nodegroupIds, rootNode.alias);
        recordWasmTiming("populate (WASM)", performance.now() - t0);
        const allNodegroups = /* @__PURE__ */ new Map();
        const updatedNodegroups = result.allNodegroupsMap;
        if (updatedNodegroups instanceof Map) {
          for (const [key, value] of updatedNodegroups.entries()) {
            if (typeof value === "boolean") {
              allNodegroups.set(key, value);
            }
          }
        } else {
          for (const [key, value] of Object.entries(updatedNodegroups)) {
            if (typeof value === "boolean") {
              allNodegroups.set(key, value);
            }
          }
        }
        this._pseudoCache = /* @__PURE__ */ new Map();
        recordWasmTiming("populate total", performance.now() - populateStart);
      } catch (error) {
        const resourceId = ((_b2 = (_a2 = this.wasmWrapper).getResourceId) == null ? void 0 : _b2.call(_a2)) || "unknown";
        console.error(`[populate] Rust implementation failed for resource ${resourceId}:`, error);
        throw new Error(`populate failed for resource ${resourceId}: ${error}`);
      }
    }
    async getValueCache(build = true, getMeta = void 0) {
      if (build) {
        this.cache = await this.buildValueCache(getMeta);
      }
      return this.cache;
    }
    async buildValueCache(getMeta) {
      const cacheByTile = {};
      for (let pseudos of this._pseudoCache.values()) {
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
    release() {
      this._pseudoCache.clear();
      this.cache = void 0;
      this.scopes = void 0;
      this.metadata = void 0;
      this.wasmWrapper.release();
      this.resource = void 0;
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
        const cards = graph.cards || [];
        if (cards.filter((card2) => card2.nodegroup_id === nodegroup).length > 0) {
          throw Error(`This nodegroup, ${nodegroupId}, already has a card`);
        }
        const cardId = this._generateUuidv5(`card-ng-${nodegroupId}`);
        const card = new StaticCard({
          active: options.active === void 0 ? true : options.active,
          cardid: cardId,
          component_id: cardComponent.id,
          config: config || void 0,
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
        graph.pushCard(card);
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
        graph.pushNodegroup(nodegroup);
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
        description: description || void 0,
        exportable: options.exportable || false,
        fieldname: options.fieldname || void 0,
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
        sourcebranchpublication_id: void 0
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
        graph.pushNode(newNode);
        const edge = this._generateEdge(prnt.nodeid, nodeId, parentProperty);
        graph.pushEdge(edge);
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
          graph.pushCardXNodeXWidget(cardXNodeXWidget);
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
  ResourceModelWrapper = class extends WASMResourceModelWrapper {
    constructor(wkrm, graph, viewModelClass, defaultAllow) {
      super(wkrm, graph, defaultAllow);
      __publicField(this, "viewModelClass");
      __publicField(this, "permittedNodegroups");
      __publicField(this, "pruneTiles", true);
      this.pruneTiles = !defaultAllow;
      this.viewModelClass = viewModelClass;
    }
    getRoot() {
      const node = this.getRootNode();
      const pseudoNode = this.createPseudoNode(node.alias);
      return new NodeViewModel(pseudoNode, this);
    }
    buildNodes() {
      const graph = this.graph ?? graphManager.getGraph(this.wkrm.graphId);
      return this.buildNodesForGraph(graph);
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
    getNodegroupObjects() {
      if (!this.nodegroups) {
        this.buildNodes();
      }
      if (!this.nodegroups) {
        throw Error("Could not build nodegroups");
      }
      return this.nodegroups;
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
      super.pruneGraph(keepFunctions);
    }
    getPruneTiles(pruneTiles) {
      if (pruneTiles === void 0) {
        pruneTiles = this.pruneTiles;
      }
      return pruneTiles;
    }
    async all(params = void 0) {
      const paramObj = params || {
        limit: void 0,
        lazy: void 0,
        pruneTiles: this.getPruneTiles(params == null ? void 0 : params.pruneTiles)
      };
      const promises = [];
      for await (const resource of this.iterAll(paramObj)) {
        promises.push(resource);
      }
      return Promise.all(promises);
    }
    async *resourceGenerator(staticResources, lazy = false, pruneTiles) {
      for await (const staticResource of staticResources) {
        yield this.fromStaticResource(staticResource, lazy, pruneTiles);
      }
    }
    async *iterAll(params) {
      yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy, params.pruneTiles);
    }
    async *summaryGenerator(staticSummaries, lazy = true) {
      for await (const summary of staticSummaries) {
        const summaryResource = StaticResource.fromSummary(summary);
        yield this.fromStaticResource(summaryResource, lazy, false);
      }
    }
    async *iterAllSummaries(params) {
      yield* this.summaryGenerator(staticStore.loadAllSummaries(this.wkrm.graphId, params.limit), true);
    }
    async allSummaries(params = void 0) {
      const paramObj = params || {
        limit: void 0
      };
      const promises = [];
      for await (const resource of this.iterAllSummaries(paramObj)) {
        promises.push(resource);
      }
      return Promise.all(promises);
    }
    async loadFullResource(id) {
      const fullResource = await staticStore.ensureFullResource(id);
      return this.fromStaticResource(fullResource, false, true);
    }
    async findStatic(id) {
      return await staticStore.loadOne(id);
    }
    async find(id, lazy = true, pruneTiles) {
      const rivm = await this.findStatic(id);
      const x = this.fromStaticResource(rivm, lazy, pruneTiles);
      return x;
    }
    setPermittedNodegroups(permissions) {
      const nodegroups = this.getNodegroupObjects();
      const nodes = this.getNodeObjectsByAlias();
      this.permittedNodegroups = new Map([
        ...permissions
      ].map(([key, value]) => {
        const k = key ?? "";
        let resolvedKey;
        if (nodegroups.has(k) || k === "") {
          resolvedKey = k;
        } else {
          const node = nodes.get(k);
          if (node) {
            resolvedKey = node.nodeid ?? "";
          } else {
            throw Error(`Could not find ${key} in nodegroups for permissions`);
          }
        }
        return [
          resolvedKey,
          value
        ];
      }));
      super.setPermittedNodegroups(this.permittedNodegroups);
    }
    getPermittedNodegroups() {
      if (!this.permittedNodegroups) {
        const permissions = new Map([
          ...this.getNodegroupObjects()
        ].map(([k, _]) => [
          k ?? "",
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
    isNodegroupPermitted(nodegroupId, _tile) {
      const permitted = this.getPermittedNodegroups().get(nodegroupId ?? "");
      if (permitted === void 0 || permitted === false) {
        return false;
      }
      if (permitted === true) {
        return true;
      }
      if (typeof permitted === "object" && "path" in permitted && "allowed" in permitted) {
        return true;
      }
      throw Error(`Ambiguous permission state: ${JSON.stringify(permitted)} for nodegroup ${nodegroupId}`);
    }
    makeInstance(id, resource, pruneTiles, lazy = false) {
      pruneTiles = this.getPruneTiles(pruneTiles);
      if (!this.viewModelClass) {
        throw Error(`Cannot instantiate without a viewModelClass in ${this.wkrm.modelClassName}`);
      }
      const instance = new this.viewModelClass(id, this.viewModelClass.prototype.__, (rivm) => new ResourceInstanceWrapper(rivm, this, resource, pruneTiles, lazy), null);
      return instance;
    }
    fromStaticResource(resource, lazy = false, pruneTiles) {
      const start = performance.now();
      const wkri = this.makeInstance(resource.resourceinstance.resourceinstanceid, resource, pruneTiles, lazy);
      recordWasmTiming("makeInstance", performance.now() - start);
      if (!wkri.$) {
        throw Error("Could not load resource from static definition");
      }
      const pop = wkri.$.populate(lazy).then(() => {
        recordWasmTiming("fromStaticResource total", performance.now() - start);
        return wkri;
      });
      return pop;
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
  function makeResourceModelWrapper(viewModelClass, wkrm, graph, defaultAllow) {
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
    const wrapper = new ResourceModelWrapper(wkrm, graph, vmc, defaultAllow);
    vmc.prototype.__ = wrapper;
    return vmc;
  }
  GraphManager = class {
    constructor(archesClient2) {
      __publicField(this, "_initialized", false);
      __publicField(this, "archesClient");
      __publicField(this, "graphs");
      __publicField(this, "wkrms");
      __publicField(this, "defaultAllow", false);
      this.archesClient = archesClient2;
      this.graphs = /* @__PURE__ */ new Map();
      this.wkrms = /* @__PURE__ */ new Map();
    }
    getPruneTiles(pruneTiles) {
      if (pruneTiles === void 0) {
        pruneTiles = !this.defaultAllow;
      }
      return pruneTiles;
    }
    async initialize(configurationOptions = void 0) {
      if (this._initialized) {
        return;
      }
      if (configurationOptions === void 0) {
        configurationOptions = new ConfigurationOptions();
      }
      const graphJsons = await this.archesClient.getGraphs();
      this.defaultAllow = configurationOptions.defaultAllowAllNodegroups;
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
        if (!(meta instanceof StaticGraphMeta)) {
          meta = new StaticGraphMeta(meta);
        }
        meta.graphid = meta.graphid || graphId;
        const wkrm = new WKRM(meta);
        this.wkrms.set(wkrm.modelClassName, wkrm);
      });
      if (configurationOptions.eagerLoadGraphs) {
        await Promise.all(graphs.map(([g]) => this.loadGraph(g, configurationOptions.defaultAllowAllNodegroups)));
      }
      this._initialized = true;
    }
    async loadGraph(modelClass, defaultAllow = false) {
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
      const graph = await this.archesClient.getGraph(wkrm.meta);
      if (!graph) {
        throw Error(`Could not load graph ${wkrm.graphId}`);
      }
      nodeConfigManager.loadFromGraph(graph);
      let model;
      if (typeof modelClass == "string") {
        modelClassName = modelClass;
        model = makeResourceModelWrapper(void 0, wkrm, graph, defaultAllow);
      } else {
        modelClassName = modelClass.name;
        model = makeResourceModelWrapper(modelClass, wkrm, graph, defaultAllow);
      }
      this.graphs.set(graph.graphid, model.prototype.__);
      return model.prototype.__;
    }
    async get(modelClass, defaultAllow = false) {
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
        return this.loadGraph(modelClass, defaultAllow);
      }
      return wrapper;
    }
    async getResource(resourceId, lazy = true, pruneTiles) {
      pruneTiles = this.getPruneTiles(pruneTiles);
      const rivm = await staticStore.loadOne(resourceId);
      let graph = this.graphs.get(rivm.resourceinstance.graph_id);
      if (!graph) {
        graph = await this.loadGraph(rivm.resourceinstance.graph_id, !pruneTiles);
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
  CollectionMutator = class {
    constructor(collection) {
      __publicField(this, "collection");
      this.collection = collection;
    }
    getCollection() {
      return this.collection;
    }
    static createEmpty(name, options) {
      const opts = typeof options === "string" ? {
        collectionId: options
      } : options || {};
      const { collectionId, nodeType = "ConceptScheme", uri } = opts;
      const lang = getCurrentLanguage$1();
      let prefLabels;
      let nameValue;
      if (typeof name === "string") {
        nameValue = name;
        prefLabels = {
          [lang]: StaticValue.create("", "prefLabel", name)
        };
      } else {
        nameValue = name[lang] || Object.values(name)[0];
        prefLabels = {};
        for (const [l, v] of Object.entries(name)) {
          prefLabels[l] = StaticValue.create("", "prefLabel", v);
        }
      }
      const id = collectionId || generateUuidv5([
        "collection"
      ], nameValue);
      const collection = new StaticCollection({
        id,
        uri,
        prefLabels,
        nodeType,
        concepts: {},
        __allConcepts: {},
        __values: {}
      });
      return new CollectionMutator(collection);
    }
    static createArchesCollection(name, options) {
      return CollectionMutator.createEmpty(name, {
        ...options,
        nodeType: "Collection"
      });
    }
    addConcept(options) {
      if (options.parentId) {
        return this.addChildConcept(options.parentId, options);
      }
      const concept = this._createConcept(options);
      this.collection.concepts[concept.id] = concept;
      this._indexConcept(concept);
      return {
        concept,
        id: concept.id
      };
    }
    addMember(options) {
      if (this.collection.nodeType === "ConceptScheme") {
        console.warn("addMember called on ConceptScheme - use addConcept for clarity");
      }
      return this.addConcept(options);
    }
    addChildConcept(parentId, options) {
      const parent = this._findConcept(parentId);
      if (!parent) {
        throw new Error(`Parent concept with ID ${parentId} not found`);
      }
      const concept = this._createConcept(options);
      if (!parent.children) {
        parent.children = [];
      }
      parent.children.push(concept);
      parent.children.sort((a, b) => (a.sortOrder ?? 999) - (b.sortOrder ?? 999));
      this._indexConcept(concept);
      return {
        concept,
        id: concept.id
      };
    }
    addConcepts(concepts, parentId) {
      const results = [];
      for (const conceptData of concepts) {
        const { children, ...conceptOptions } = conceptData;
        const result = parentId ? this.addChildConcept(parentId, conceptOptions) : this.addConcept(conceptOptions);
        results.push(result);
        if (children && children.length > 0) {
          const childResults = this.addConcepts(children, result.id);
          results.push(...childResults);
        }
      }
      return results;
    }
    removeConcept(conceptId) {
      if (this.collection.concepts[conceptId]) {
        const concept = this.collection.concepts[conceptId];
        this._unindexConcept(concept);
        delete this.collection.concepts[conceptId];
        return true;
      }
      for (const topConcept of Object.values(this.collection.concepts)) {
        if (this._removeFromChildren(topConcept, conceptId)) {
          return true;
        }
      }
      return false;
    }
    moveConcept(conceptId, newParentId) {
      const concept = this._findConcept(conceptId);
      if (!concept) {
        return false;
      }
      this.removeConcept(conceptId);
      this._indexConcept(concept);
      if (newParentId === null) {
        this.collection.concepts[concept.id] = concept;
      } else {
        const newParent = this._findConcept(newParentId);
        if (!newParent) {
          this.collection.concepts[concept.id] = concept;
          return false;
        }
        if (!newParent.children) {
          newParent.children = [];
        }
        newParent.children.push(concept);
        newParent.children.sort((a, b) => (a.sortOrder ?? 999) - (b.sortOrder ?? 999));
      }
      return true;
    }
    updateConcept(conceptId, updates) {
      const concept = this._findConcept(conceptId);
      if (!concept) {
        return false;
      }
      if (updates.label !== void 0) {
        for (const value of Object.values(concept.prefLabels)) {
          delete this.collection.__values[value.id];
        }
        if (typeof updates.label === "string") {
          const lang = getCurrentLanguage$1();
          const valueId = generateUuidv5([
            "value"
          ], `${concept.id}/prefLabel/${updates.label}/${lang}`);
          const newValue = new StaticValue({
            id: valueId,
            value: updates.label
          }, concept);
          concept.prefLabels = {
            [lang]: newValue
          };
        } else {
          concept.prefLabels = {};
          for (const [lang, value] of Object.entries(updates.label)) {
            const valueId = generateUuidv5([
              "value"
            ], `${concept.id}/prefLabel/${value}/${lang}`);
            const newValue = new StaticValue({
              id: valueId,
              value
            }, concept);
            concept.prefLabels[lang] = newValue;
          }
        }
        for (const value of Object.values(concept.prefLabels)) {
          this.collection.__values[value.id] = value;
        }
      }
      if (updates.source !== void 0) {
        concept.source = updates.source;
      }
      if (updates.sortOrder !== void 0) {
        concept.sortOrder = updates.sortOrder;
      }
      if (updates.uri !== void 0) {
        concept.source = updates.uri;
      }
      return true;
    }
    getConcept(conceptId) {
      return this._findConcept(conceptId);
    }
    getAllConceptIds() {
      return Object.keys(this.collection.__allConcepts);
    }
    getConceptPath(conceptId) {
      const path = [];
      const findPath = (concepts, targetId) => {
        for (const concept of concepts) {
          if (concept.id === targetId) {
            path.push(concept.id);
            return true;
          }
          if (concept.children) {
            path.push(concept.id);
            if (findPath(concept.children, targetId)) {
              return true;
            }
            path.pop();
          }
        }
        return false;
      };
      if (findPath(Object.values(this.collection.concepts), conceptId)) {
        return path;
      }
      return null;
    }
    _createConcept(options) {
      const lang = getCurrentLanguage$1();
      let labelValue;
      if (typeof options.label === "string") {
        labelValue = options.label;
      } else {
        labelValue = options.label[lang] || Object.values(options.label)[0];
      }
      const conceptId = options.id || generateUuidv5([
        "concept"
      ], `${this.collection.id}/${labelValue}`);
      const prefLabels = {};
      if (typeof options.label === "string") {
        const valueId = generateUuidv5([
          "value"
        ], `${conceptId}/prefLabel/${options.label}/${lang}`);
        prefLabels[lang] = {
          id: valueId,
          value: options.label
        };
      } else {
        for (const [l, v] of Object.entries(options.label)) {
          const valueId = generateUuidv5([
            "value"
          ], `${conceptId}/prefLabel/${v}/${l}`);
          prefLabels[l] = {
            id: valueId,
            value: v
          };
        }
      }
      return new StaticConcept({
        id: conceptId,
        prefLabels,
        source: options.source || options.uri || null,
        sortOrder: options.sortOrder ?? null,
        children: null
      });
    }
    _findConcept(conceptId) {
      return this.collection.__allConcepts[conceptId] || null;
    }
    _indexConcept(concept) {
      this.collection.__allConcepts[concept.id] = concept;
      for (const value of Object.values(concept.prefLabels)) {
        this.collection.__values[value.id] = value;
      }
      if (concept.children) {
        for (const child of concept.children) {
          this._indexConcept(child);
        }
      }
    }
    _unindexConcept(concept) {
      delete this.collection.__allConcepts[concept.id];
      for (const value of Object.values(concept.prefLabels)) {
        delete this.collection.__values[value.id];
      }
      if (concept.children) {
        for (const child of concept.children) {
          this._unindexConcept(child);
        }
      }
    }
    _removeFromChildren(parent, conceptId) {
      if (!parent.children) {
        return false;
      }
      const index2 = parent.children.findIndex((c) => c.id === conceptId);
      if (index2 !== -1) {
        const removed = parent.children[index2];
        this._unindexConcept(removed);
        parent.children.splice(index2, 1);
        if (parent.children.length === 0) {
          parent.children = null;
        }
        return true;
      }
      for (const child of parent.children) {
        if (this._removeFromChildren(child, conceptId)) {
          return true;
        }
      }
      return false;
    }
  };
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
      if (value === null) {
        return null;
      }
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
      } else if (value instanceof String && typeof (value == null ? void 0 : value.forJson) === "function") {
        newValue = value.toString();
      } else if (typeof (value == null ? void 0 : value.getDisplay) === "function") {
        newValue = await value.getDisplay();
      } else if (typeof (value == null ? void 0 : value.forJson) === "function") {
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
      const childAliases = vm.__parentWkri.$.model.getChildNodeAliases(vm.__node.nodeid);
      const nodes = Object.fromEntries(childAliases.map((alias) => [
        alias,
        vm.__parentWkri.$.model.getNodeObjectFromAlias(alias)
      ]));
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
  renderers = Object.freeze(Object.defineProperty({
    __proto__: null,
    Cleanable,
    FlatMarkdownRenderer,
    MarkdownRenderer
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  version = "0.2.1-alpha.53";
  registerAlizarinTimingGetter(getTimingStats);
  registerWasmTimingGetter(getWasmTimings);
  let _wasmReadyResolve;
  wasmReady = new Promise((resolve) => {
    _wasmReadyResolve = resolve;
  });
  Promise.resolve().then(() => initWasm().then(_wasmReadyResolve));
  AlizarinModel = ResourceInstanceViewModel;
  setCurrentLanguage = setCurrentLanguage$1;
  getCurrentLanguage = getCurrentLanguage$1;
  slugify = slugify$1;
  getValueFromPath = getValueFromPath$1;
  getValueFromPathSync = getValueFromPathSync$1;
})();
export {
  AlizarinModel,
  CollectionMutator,
  GraphManager,
  GraphMutator,
  RDM,
  ResourceModelWrapper,
  WASMResourceModelWrapper,
  WKRM,
  __tla,
  client,
  collectionToSkosXml,
  collectionsToSkosXml,
  ensureWasmRdmCache,
  getCurrentLanguage,
  getTimingStats,
  getValueFromPath,
  getValueFromPathSync,
  graphManager,
  initWasm,
  interfaces,
  logTimingStats,
  newWASMResourceInstanceWrapperForResource,
  nodeConfig,
  parseSkosXml,
  parseSkosXmlToCollection,
  registerExtensionHandler,
  registerResolvableDatatype,
  renderers,
  resetTimingStats,
  setCurrentLanguage,
  setWasmURL,
  slugify,
  staticStore,
  staticTypes,
  index as tracing,
  unregisterResolvableDatatype,
  utils,
  version,
  viewModels,
  wasmReady
};
