import { test } from "vitest";
import GroupJSON from "./data/models/Group.json";
import PersonJSON from "./data/models/Person.json";
import ArchesPluginJSON from "./data/models/Arches Plugin.json";

async function graphsResponse({ task }, use) {
  // TODO: this sends more back than it should
  const models = {};
  const Group = GroupJSON["graph"][0];
  const Person = PersonJSON["graph"][0];
  const ArchesPlugin = ArchesPluginJSON["graph"][0];
  models[Group.graphid] = Group;
  models[Person.graphid] = Person;
  models[ArchesPlugin.graphid] = ArchesPlugin;
  await use({
    models: models,
  });
}

async function graphResponses({ task }, use) {
  const models = {};
  const Group = GroupJSON["graph"][0];
  const Person = PersonJSON["graph"][0];
  const ArchesPlugin = ArchesPluginJSON["graph"][0];
  models[Group.graphid] = Group;
  models[Person.graphid] = Person;
  models[ArchesPlugin.graphid] = ArchesPlugin;
  await use(models);
}

const apiTest = test.extend({
  graphsResponse: graphsResponse,
  graphResponses: graphResponses,
});

export { apiTest };
