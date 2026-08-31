type AlizarinModule = typeof import('alizarin');

// GeoJSONViewModel wraps a `geojson-feature-collection`. It proxies the
// underlying FeatureCollection, so you read `.type` / `.features` directly — and
// can hand the whole thing to a map library (see the live map on the page).
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
          const name = await site.name;
          const boundary: any = await site.boundary; // GeoJSONViewModel
          const features = boundary?.features ?? [];
          const geom = features[0]?.geometry?.type ?? '—';
          return (
            <li key={ i }>{ name } — { features.length } feature(s), { geom }</li>
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
