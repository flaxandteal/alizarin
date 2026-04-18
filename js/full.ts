// Full build: alizarin core + all extension datatypes in a single bundle.
// Extensions import from 'alizarin' which is aliased (via vite.config.js)
// to ./main.ts, ensuring a single shared module instance.

export * from './main';

// Side-effect imports: these register custom datatypes and display serializers
import '../ext/js/@alizarin/filelist/src/main';
import '../ext/js/@alizarin/clm/src/main';
