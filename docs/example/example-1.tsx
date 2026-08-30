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
      <ul>{
        footprints.map(async (footprint: HazardFootprint, i: number) => {
          const name = await footprint.name;
          const model = await footprint.produced_by_model;
          const scenario = await footprint.scenario;
          const modelName = model ? await model.name : '(unknown model)';
          const scenarioName = scenario ? await scenario.name : '(no scenario)';
          return (
            <li key={ i }>{ name } — modelled by { modelName }, under { scenarioName }</li>
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
