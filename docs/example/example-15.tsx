type AlizarinModule = typeof import('alizarin');

// Functions run at load time. The primary-descriptor function takes a string
// template (e.g. "Heritage site — <Name>") and node values, and computes the
// resource's name / description / map-popup — no field on the tile stores them.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up
  try {
// @alizarin-code-begin
    class Site extends AlizarinModel<Site> {};
    const Sites = await graphManager.get(Site);
    const sites = await Sites.all();

    return (
      <table>
        <thead><tr><th>Computed name</th><th>Computed description</th></tr></thead>
        <tbody>{
          sites.map(async (site: Site, i: number) => {
            // Descriptors are computed by the load-time function, not stored:
            const name = await site.getName(true);
            const description = await site.getDescription(true);
            return (
              <tr key={ i }>
                <td>{ name }</td>
                <td>{ description }</td>
              </tr>
            );
          })
        }</tbody>
      </table>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
