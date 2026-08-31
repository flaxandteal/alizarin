type AlizarinModule = typeof import('alizarin');

// The whole read API in a few lines: point at a model, list its resources, and
// await typed fields. Every binding exposes this same shape.
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
          const id = await org.legal_identifier;
          return (
            <li key={ i }>{ name }{ id ? ` — ${id}` : '' }</li>
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
