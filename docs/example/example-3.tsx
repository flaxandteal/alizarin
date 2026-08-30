type AlizarinModule = typeof import('alizarin');

// Following a relationship: each person's name plus their affiliated organisation.
// Property access is lazy — `person.affiliated_organisation` resolves the linked
// Organisation record only when awaited.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  class Organisation extends AlizarinModel<Organisation> {};
  await (await graphManager.get("Person")).all();       // warm-up
  await (await graphManager.get(Organisation)).all();   // load the linked graph first
  try {
// @alizarin-code-begin
    class Person extends AlizarinModel<Person> {};
    const People = await graphManager.get(Person);
    const people = await People.all();

    return (
      <ul>{
        people.map(async (person: Person, i: number) => {
          const name = await person['name'];
          const org = await person['affiliated_organisation'];
          const orgName = org ? await org['name'] : '(unaffiliated)';
          return (<li key={ i }>{ name } — { orgName }</li>);
        })
      }</ul>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
