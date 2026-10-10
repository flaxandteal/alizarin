import { test } from "vitest";
import { assert } from "chai";
import { PseudoList } from "../js/pseudos";

// C6: a cardinality-N nodegroup whose node is itself list-valued yields a
// PseudoList of per-tile lists. flattened() resolves and flattens that one level
// so readers get a single list of values instead of a list of lists.
test("PseudoList.flattened() flattens per-tile lists one level, awaiting nested promises (C6)", async () => {
  const pl = new PseudoList();
  // Two tiles, each holding a (possibly async) list of values — the shape a
  // cardinality-N resource-instance-list resolves to.
  (pl as unknown as any[]).push(
    Promise.resolve([Promise.resolve("a"), Promise.resolve("b")]),
  );
  (pl as unknown as any[]).push(Promise.resolve(["c"]));

  const flat = await pl.flattened();
  assert.deepEqual(flat, ["a", "b", "c"]);
});

test("PseudoList.flattened() is equivalent to awaiting for scalar cardinality-N (C6)", async () => {
  const pl = new PseudoList();
  (pl as unknown as any[]).push(Promise.resolve("first"));
  (pl as unknown as any[]).push(Promise.resolve("second"));

  const flat = await pl.flattened();
  assert.deepEqual(flat, ["first", "second"]);
});

test("PseudoList.flattened() drops null/undefined entries (C6)", async () => {
  const pl = new PseudoList();
  (pl as unknown as any[]).push(Promise.resolve(["a", null]));
  (pl as unknown as any[]).push(Promise.resolve(null));
  (pl as unknown as any[]).push(Promise.resolve("b"));

  const flat = await pl.flattened();
  assert.deepEqual(flat, ["a", "b"]);
});
