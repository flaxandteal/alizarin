# ORM scenarios

Each `*.json` here is a sequence of ORM steps, played and checked step by step
by every backend:

- TypeScript on WASM and NAPI: `tests/dual-backend/scenarios.test.ts`
- Python: `crates/alizarin-python/tests/test_scenarios.py`

So the ORMs cannot drift on anything a scenario covers. A new scenario is a new
file; both runners pick it up.

## Format

```json
{
  "_doc": "What it shows.",
  "fixture": "view-models",
  "steps": [ ... ]
}
```

`fixture` is a directory under `tests/data/` holding `graph.json` (one graph),
`resource.json` (the resource the steps act on) and, optionally,
`collection.json` (its concepts' collection).

## Steps

A step's first key is its kind; each runner has one handler per kind, and an
unknown kind fails.

| Step | Meaning |
|---|---|
| `{"read": alias, "forJson": v, "str": s}` | Read the attribute, a view model: its JSON form is `v` and its string form `s` (each checked only when given). |
| `{"pending": alias, "value": v}` | Read an attribute assigned since the last commit: the value as given, `v` (coercion happens at commit). |
| `{"assign": alias, "value": v}` | Assign `v` (TypeScript `resource[alias] = v`, Python `setattr`). |
| `{"assign_fails": alias, "value": v, "error": text}` | The assignment must raise where it is made, its message containing `text` (an edit the graph rules out). |
| `{"commit": {"upserted": {alias: v}, "removed": [ids]}}` | Commit (`$commit()` / `commit()`): exactly one upserted tile per alias listed, holding `v` for that alias's node, and these tile ids removed. |
| `{"commit_fails": text}` | The commit must raise, its message containing `text`; nothing is applied. |
