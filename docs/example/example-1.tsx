type AlizarinModule = typeof import('alizarin');

async function run() {
  const { AlizarinModel, graphManager, staticStore }: AlizarinModule = await import('alizarin');
  // Preload so we do not need individual JSON files.
  staticStore.cacheMetadataOnly = false;
  class Person extends AlizarinModel<Person> {};
  (await graphManager.get("Session")).all();
  (await graphManager.get("Talk")).all();
  (await graphManager.get(Person)).all();
  (await graphManager.get("Institution")).all();
  try {
// @alizarin-code-begin
    class Session extends AlizarinModel<Session> {};
    const Sessions = await graphManager.get(Session);
    const sessions = await Sessions.all();

    return Promise.all(
      sessions.map(async (session: Session) => {
        return (
          <div>
            <h1>Session { await session['name'] }</h1>

            <ul>{
              [1, 2, 3, 4, 5, 6].map(async (slot: number) => {
                const talk = session[`slot_${slot}`];
                const title = await talk.title;
                if (title) {
                  const presenters = await talk.presenter;
                  const presenterNames = await Promise.all(
                    presenters.map(
                      async (presenter: Promise<Person>) => (await presenter)['name']
                    )
                  );
                  return (
                    <li key={ slot }>{ slot } : { title } -- { presenterNames.join(', ') }</li>
                  );
                } else {
                  return (
                    <li key={ slot }>{ slot } : (empty)</li>
                  );
                }
              })
            }
            </ul>
          </div>
        )
      })
    );
// @alizarin-code-end
  } catch (e: any) {
    return (
      <div>Error: { e }</div>
    );
  }
}
export default {run};
