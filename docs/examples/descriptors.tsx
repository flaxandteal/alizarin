type AlizarinModule = typeof import('alizarin');

// Names, descriptions and slugs: a resource's human-facing identity. name and
// description are computed by the descriptor function (see Functions); the slug
// is a URL-safe identifier derived deterministically from the name via slugify().
async function run() {
  const { AlizarinModel, graphManager, slugify }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up
  try {
// @alizarin-code-begin
    class Site extends AlizarinModel<Site> {};
    const Sites = await graphManager.get(Site);
    const sites = await Sites.all();

    return (
      <table>
        <thead><tr><th>Name</th><th>Slug</th><th>Description</th></tr></thead>
        <tbody>{
          sites.map(async (site: Site, i: number) => {
            const name = await site.getName(true);          // computed descriptor
            const description = await site.getDescription(true);
            const slug = slugify(name);                      // URL-safe, deterministic
            return (
              <tr key={ i }>
                <td>{ name }</td>
                <td><code>{ slug }</code></td>
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
