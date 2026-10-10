import { test } from "vitest";
import { assert } from "chai";
import { normalizeModelName } from "../js/graphManager";

// C7: graphManager.get() must resolve a model by its display name, slug or alias,
// not only its exact PascalCased class name. normalizeModelName is the matcher.
test("normalizeModelName collapses name variants to one key (C7)", () => {
  // "NPC" / "npc" / "Npc" (class name) all resolve to the same model.
  assert.equal(normalizeModelName("NPC"), normalizeModelName("Npc"));
  assert.equal(normalizeModelName("npc"), normalizeModelName("Npc"));

  // Spaces, hyphens and underscores are ignored, so a display name / slug matches
  // the generated class name.
  assert.equal(normalizeModelName("Monster Type"), normalizeModelName("MonsterType"));
  assert.equal(normalizeModelName("monster-type"), normalizeModelName("MonsterType"));
  assert.equal(normalizeModelName("Monster_Type"), "monstertype");

  // Distinct models must NOT collide.
  assert.notEqual(normalizeModelName("NPC"), normalizeModelName("Monster Type"));
});
