type AlizarinModule = typeof import('alizarin');

// Deterministic IDs: the same input always produces the same UUID. IDs are
// UUIDv5 (namespace + key), so a rebuild from unchanged input yields byte-
// identical output — which makes exports diffable and cacheable.
async function run() {
  const { AlizarinModel, graphManager, GraphMutator, utils }: AlizarinModule = await import('alizarin');
  await (await graphManager.get("Site")).all(); // warm-up
  try {
// @alizarin-code-begin
    // 1. The primitive: same (namespace, key) → same UUID, every time.
    const a = utils.generateUuidv5(['graph', 'demo'], 'survey-area');
    const b = utils.generateUuidv5(['graph', 'demo'], 'survey-area');
    const different = utils.generateUuidv5(['graph', 'demo'], 'other-key');

    // 2. In practice: mutating the same base graph the same way twice produces
    //    the same new node id — so a regenerated graph diffs cleanly.
    class Site extends AlizarinModel<Site> {};
    const base = (await graphManager.get(Site)).graph;
    const idOf = () => {
      const m = new GraphMutator(base);
      m.addStringNode('site', 'note', 'Note', '1',
        'http://www.cidoc-crm.org/cidoc-crm/E62_String',
        'http://www.cidoc-crm.org/cidoc-crm/P3_has_note');
      const g: any = m.apply();
      const nodes = g.nodes instanceof Map ? [...g.nodes.values()] : Object.values(g.nodes ?? {});
      return nodes.find((n: any) => n?.alias === 'note')?.nodeid;
    };
    const first = idOf();
    const second = idOf();

    return (
      <ul>
        <li>generateUuidv5(same key): <code>{ a }</code> === <code>{ b }</code> → <strong>{ String(a === b) }</strong></li>
        <li>generateUuidv5(different key): differs → <strong>{ String(a !== different) }</strong></li>
        <li>mutate twice, new node id identical: <strong>{ String(first === second) }</strong><br/><code>{ first }</code></li>
      </ul>
    );
// @alizarin-code-end
  } catch (e: any) {
    return (<div>Error: { e }</div>);
  }
}
export default {run};
