type AlizarinModule = typeof import('alizarin');

// Three more scalar types: edtf (fuzzy/interval dates), boolean (with true/false
// labels from node config), and url ({url, url_label}). None is a plain string,
// so coerce each before rendering.
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
          const period = await site.survey_period;   // EDTFViewModel
          const isProtected = await site.is_protected; // BooleanViewModel → label
          const url = await site.record_url;          // UrlViewModel
          const link = url ? url.forJson() : null;    // { url, url_label }
          return (
            <li key={ i }>
              { name } — { period ? `${period}` : '?' } · { String(isProtected) }
              { link ? <> · <a href={ link.url }>{ link.url_label || link.url }</a></> : '' }
            </li>
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
