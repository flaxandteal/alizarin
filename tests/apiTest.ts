import { test } from "vitest";
import * as GroupJSON from "./data/models/Group.json";
import * as PersonJSON from "./data/models/Person.json";
import * as ArchesPluginJSON from "./data/models/Arches Plugin.json";

async function graphsResponse({}, use: any) {
  // TODO: this sends more back than it should
  const models: Record<string, any> = {};
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

async function graphResponses({}, use: any) {
  const models: Record<string, any> = {};
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
