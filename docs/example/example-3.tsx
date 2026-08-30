type AlizarinModule = typeof import('alizarin');

// Following a relationship: each talk's title plus its presenters' names.
// Property access is lazy — `talk.presenter` resolves the linked Person records
// only when awaited.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  class Person extends AlizarinModel<Person> {};
  (await graphManager.get("Talk")).all();  // warm-up
  (await graphManager.get(Person)).all();
  try {
// @alizarin-code-begin
    class Talk extends AlizarinModel<Talk> {};
    const Talks = await graphManager.get(Talk);
    const talks = await Talks.all();

    return (
      <ul>{
        talks.map(async (talk: Talk, i: number) => {
          const title = await talk.title;
          if (!title) {
            return null;
          }
          const presenters = await talk.presenter;
          const names = await Promise.all(
            presenters.map(async (p: Promise<Person>) => (await p)['name'])
          );
          return (<li key={ i }>{ title } — { names.join(', ') }</li>);
        })
      }</ul>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
