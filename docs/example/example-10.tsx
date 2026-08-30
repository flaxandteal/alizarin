type AlizarinModule = typeof import('alizarin');

// Concept values resolve against a controlled list (RDM collection): the tile
// stores a value-id, and ConceptValueViewModel — a String subclass — resolves
// and stringifies to the concept's label in the current language.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Organisation")).all(); // warm-up
  try {
// @alizarin-code-begin
    class Organisation extends AlizarinModel<Organisation> {};
    const Organisations = await graphManager.get(Organisation);
    const organisations = await Organisations.all();

    return (
      <ul>{
        organisations.map(async (org: Organisation, i: number) => {
          const name = await org.name;
          const sector = await org.sector; // ConceptValueViewModel → label
          return (
            <li key={ i }>{ name }{ sector ? ` — ${sector}` : '' }</li>
          );
        })
      }</ul>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
