type AlizarinModule = typeof import('alizarin');

// The minimal query every backend answers identically: load a model, get all of
// its resources, and read a top-level field. Here, the sites by name.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up
  try {
// @alizarin-code-begin
    class Site extends AlizarinModel<Site> {};
    const Sites = await graphManager.get(Site);
    const sites = await Sites.all();

    return (
      <div>
        <p>{ sites.length } sites</p>
        <ul>{
          sites.map(async (site: Site, i: number) => {
            const name = await site.name;
            return (<li key={ i }>{ name }</li>);
          })
        }</ul>
      </div>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
