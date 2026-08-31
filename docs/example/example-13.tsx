type AlizarinModule = typeof import('alizarin');

// Extension datatypes: `reference` (CLM — a controlled-list value with inline
// labels) and `file-list` (filelist — images/files). Both come from out-of-tree
// handlers registered over the extension ABI; the docs harness imports
// @alizarin/clm and @alizarin/filelist to register them.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up
  try {
// @alizarin-code-begin
    class Site extends AlizarinModel<Site> {};
    const Sites = await graphManager.get(Site);
    const sites = await Sites.all();

    return (
      <div>{
        sites.map(async (site: Site, i: number) => {
          const name = await site.name;
          const designation = await site.heritage_designation; // reference → label
          const photos = await site.site_photos;               // file-list (array)
          const items = photos ? await Promise.all([...photos]) : [];
          return (
            <figure key={ i } style={{ margin: '0 0 16px' }}>
              <figcaption>{ name } — { designation ? `${designation}` : 'unlisted' }</figcaption>
              <div style={{ display: 'flex', gap: '8px' }}>{
                items.map((f: any, j: number) =>
                  f?.url ? <img key={ j } src={ f.url } alt={ f.name } width={ 120 } /> : null
                )
              }</div>
            </figure>
          );
        })
      }</div>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
