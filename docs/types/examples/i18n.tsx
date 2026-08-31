type AlizarinModule = typeof import('alizarin');

// i18n: localized strings carry every translation. A StringViewModel resolves to
// the current language, but `.lang(code)` reads a specific one directly — so we
// can show English and Irish (ga) side by side from the same value.
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
        <thead><tr><th>English (en)</th><th>Irish (ga)</th></tr></thead>
        <tbody>{
          sites.map(async (site: Site, i: number) => {
            const name: any = await site.name;
            const en = name?.lang ? name.lang('en') : `${name}`;
            const ga = name?.lang ? name.lang('ga') : undefined;
            return (
              <tr key={ i }>
                <td>{ en ?? '—' }</td>
                <td>{ ga ?? '(same)' }</td>
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
