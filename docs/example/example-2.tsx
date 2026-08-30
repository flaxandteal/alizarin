type AlizarinModule = typeof import('alizarin');

// A minimal query: load a model and list every instance's name.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  (await graphManager.get("Person")).all(); // warm-up: ensure the graph is loaded
  try {
// @alizarin-code-begin
    class Person extends AlizarinModel<Person> {};
    const People = await graphManager.get(Person);
    const people = await People.all();

    return (
      <ul>{
        people.map(async (person: Person, i: number) => (
          <li key={ i }>{ await person['name'] }</li>
        ))
      }</ul>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
