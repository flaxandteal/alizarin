type AlizarinModule = typeof import('alizarin');

// Following a relationship within a graph: each site and its parent site,
// resolved lazily by awaiting the linked resource, then its name.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up (parent_site links Site → Site)
  try {
// @alizarin-code-begin
    class Site extends AlizarinModel<Site> {};
    const Sites = await graphManager.get(Site);
    const sites = await Sites.all();

    return (
      <ul>{
        sites.map(async (site: Site, i: number) => {
          const name = await site.name;
          const parent = await site.parent_site;
          const parentName = parent ? await parent.name : null;
          return (
            <li key={ i }>{ name }{ parentName ? ` ⊂ ${parentName}` : ' (top-level)' }</li>
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
