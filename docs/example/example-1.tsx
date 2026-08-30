type AlizarinModule = typeof import('alizarin');

// A richer query following two relationships at once: each hazard footprint's
// name, the hazard model that produced it, and the scenario event it belongs to.
// Both links resolve lazily — only when the properties are awaited.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  class HazardModel extends AlizarinModel<HazardModel> {};
  class ScenarioEvent extends AlizarinModel<ScenarioEvent> {};
  await (await graphManager.get("HazardFootprint")).all(); // warm-up
  await (await graphManager.get(HazardModel)).all();        // load the linked graphs first
  await (await graphManager.get(ScenarioEvent)).all();
  try {
// @alizarin-code-begin
    class HazardFootprint extends AlizarinModel<HazardFootprint> {};
    const Footprints = await graphManager.get(HazardFootprint);
    const footprints = await Footprints.all();

    return (
      <table>
        <thead>
          <tr><th>Footprint</th><th>Model</th><th>Scenario</th></tr>
        </thead>
        <tbody>{
          footprints.map(async (footprint: HazardFootprint, i: number) => {
            const model = await footprint.produced_by_model;
            const scenario = await footprint.scenario;
            return (
              <tr key={ i }>
                <td>{ await footprint.name }</td>
                <td>{ model ? await model.name : '—' }</td>
                <td>{ scenario ? await scenario.name : '—' }</td>
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
