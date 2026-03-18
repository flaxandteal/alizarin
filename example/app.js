import Session from './session.js';
import { initializeAlizarin } from './utils.js';
console.log('init');
import { graphManager, staticStore } from '../dist/alizarin';

const { ref } = Vue;
const sessionRows = ref([]);

const App = {
  name: 'App',
  data() {
    return {
      sessionRows,
      Session
    };
  },
  mounted: async () => {
    sessionRows.value = await run();
    console.log(sessionRows.value);
  },
  components: {
    Session
  },
  template: `
    <Session :session="session" v-for="session in sessionRows"/>
  `
}

async function run() {
    await initializeAlizarin();
    // Preload so we do not need individual JSON files.
    staticStore.cacheMetadataOnly = false;
    (await graphManager.get("Session")).all();
    (await graphManager.get("Talk")).all();
    (await graphManager.get("Person")).all();
    (await graphManager.get("Institution")).all();

    const Sessions = await graphManager.get("Session");
    const sessions = await Sessions.all();

    return await Promise.all(sessions);
  }
Vue.createApp(App).mount('#app');
