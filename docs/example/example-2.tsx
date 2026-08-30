type AlizarinModule = typeof import('alizarin');

// A minimal query: load a model and list every instance's name.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up: ensure the graph is loaded
  try {
// @alizarin-code-begin
    class Site extends AlizarinModel<Site> {};
    const Sites = await graphManager.get(Site);
    const sites = await Sites.all();

    return (
      <ul>{
        sites.map(async (site: Site, i: number) => (
          <li key={ i }>{ await site.name }</li>
        ))
      }</ul>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
