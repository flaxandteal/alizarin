type AlizarinModule = typeof import('alizarin');

// Programmatic schema change with GraphMutator: load the Site model's graph, add
// a new string field under the root node, and apply — getting back a new graph.
// (For static model definitions the declarative CSV/JSON format is simpler; this
// is the in-code route for dynamic changes.)
async function run() {
  const { graphManager, GraphMutator }: AlizarinModule = await import('alizarin');
  const Sites = await graphManager.get("Site"); // load the model + its graph
  await Sites.all();                            // warm-up
  try {
// @alizarin-code-begin
    const baseGraph = Sites.graph;              // the model's StaticGraph
    const before = [...(Sites.nodes ?? new Map()).values()].length;

    const mutator = new GraphMutator(baseGraph);
    mutator.addStringNode(
      'site',                                   // parent (root) node alias
      'local_reference', 'Local Reference', '1',
      'http://www.cidoc-crm.org/cidoc-crm/E41_Appellation',
      'http://www.cidoc-crm.org/cidoc-crm/P1_is_identified_by',
    );
    const newGraph: any = mutator.apply();      // a new StaticGraph, base untouched

    const raw = newGraph.nodes;
    const nodes = raw instanceof Map ? [...raw.values()]
      : Array.isArray(raw) ? raw : Object.values(raw ?? {});
    const aliases = nodes.map((n: any) => n?.alias).filter(Boolean);

    return (
      <div>
        <p>{ before } nodes → { aliases.length } after adding <code>local_reference</code></p>
        <ul>{ aliases.map((a: string, i: number) => (
          <li key={ i }>{ a === 'local_reference' ? <strong>{ a } (new)</strong> : a }</li>
        )) }</ul>
      </div>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
