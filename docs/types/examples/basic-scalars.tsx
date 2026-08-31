type AlizarinModule = typeof import('alizarin');

// Basic scalar types: a StringViewModel (name) and a NumberViewModel (area, in a
// nested nodegroup). String view models render directly; wrap other scalars in a
// template literal so a Number/Date object never reaches the DOM as a raw object.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up
  try {
// @alizarin-code-begin
    class Site extends AlizarinModel<Site> {};
    const Sites = await graphManager.get(Site);
    const sites = await Sites.all();

    return (
      <ul>{
        sites.map(async (site: Site, i: number) => {
          const name = await site.name;                 // StringViewModel
          const dim = await site.area_dimension;        // nested nodegroup
          const area = dim ? await dim.area_m2 : null;  // NumberViewModel
          return (
            <li key={ i }>{ name }{ area != null ? ` — ${area} m²` : '' }</li>
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
