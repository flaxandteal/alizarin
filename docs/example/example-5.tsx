type AlizarinModule = typeof import('alizarin');

// Values come back typed and parsed, resolved lazily only when awaited. Here the
// start and end dates live in a nested semantic nodegroup (`event_span`), so we
// await the group first, then its typed date fields.
async function run() {
  const { AlizarinModel, graphManager }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("ScenarioEvent")).all(); // warm-up
  try {
// @alizarin-code-begin
    class ScenarioEvent extends AlizarinModel<ScenarioEvent> {};
    const Events = await graphManager.get(ScenarioEvent);
    const events = await Events.all();

    return (
      <ul>{
        events.map(async (event: ScenarioEvent, i: number) => {
          const name = await event.name;
          const span = await event.event_span;
          const start = span ? await span.start_date : null;
          const end = span ? await span.end_date : null;
          return (
            <li key={ i }>{ name } — { start ?? '?' } → { end ?? '?' }</li>
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
