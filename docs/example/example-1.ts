import { AlizarinModel, graphManager, staticStore } from 'alizarin'; 

async function run({print}: {print: ((...inp: any) => void)}) {
  // Preload so we do not need individual JSON files.
  staticStore.cacheMetadataOnly = false;
  class Person extends AlizarinModel<Person> {};
  (await graphManager.get("Session")).all();
  (await graphManager.get("Talk")).all();
  (await graphManager.get(Person)).all();
  (await graphManager.get("Institution")).all();
  try {
// @alizcode-begin
    class Session extends AlizarinModel<Session> {};
    const Sessions = await graphManager.get(Session);

    for (const session of await Sessions.all()) {
      print('Session', await session.name);
      for (const slot of [1, 2, 3, 4, 5, 6]) {
        const talk = session[`slot_${slot}`];
        const title = await talk.title;
        if (title) {
          const presenterNames = await Promise.all(
            (await talk.presenter).map(
              async (presenter: Promise<Person>) => (await presenter)['name']
            )
          );
          print(slot, ":", title, ' -- ', presenterNames.join(', '))
        } else {
          print(slot, ":", '(empty)')
        }
      }
    }
// @alizcode-end
  } catch (e: any) {
    print(e)
  }
}
export default {run};
