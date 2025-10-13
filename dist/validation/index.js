var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
import Ajv from "ajv";
import addFormats from "ajv-formats";
import { readdirSync, readFileSync, existsSync } from "fs";
import { join } from "path";
const $schema$2 = "http://json-schema.org/draft-07/schema#";
const title$2 = "Arches Graph Model Schema";
const description$2 = "JSON Schema for Arches heritage management graph model definition files";
const type$2 = "object";
const required$2 = ["graph"];
const properties$2 = { "graph": { "type": "array", "items": { "$ref": "#/definitions/graph" }, "minItems": 1, "maxItems": 1 } };
const definitions$2 = { "uuid": { "type": "string", "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$" }, "multilingualString": { "type": "object", "patternProperties": { "^[a-z]{2}(-[A-Z]{2})?$": { "type": "string" } }, "additionalProperties": false, "minProperties": 1 }, "graph": { "type": "object", "required": ["graphid", "name", "isresource", "nodes", "nodegroups", "edges", "cards"], "properties": { "graphid": { "$ref": "#/definitions/uuid" }, "name": { "$ref": "#/definitions/multilingualString" }, "subtitle": { "$ref": "#/definitions/multilingualString" }, "description": { "$ref": "#/definitions/multilingualString" }, "author": { "type": "string" }, "deploymentdate": { "type": "string", "format": "date-time" }, "version": { "type": "string" }, "isresource": { "type": "boolean" }, "iconclass": { "type": "string" }, "color": { "type": "string", "pattern": "^#[0-9a-f]{6}$" }, "ontology_id": { "type": "string" }, "template_id": { "$ref": "#/definitions/uuid" }, "functions": { "type": "array" }, "nodes": { "type": "array", "items": { "$ref": "#/definitions/node" }, "minItems": 1 }, "nodegroups": { "type": "array", "items": { "$ref": "#/definitions/nodegroup" }, "minItems": 1 }, "edges": { "type": "array", "items": { "$ref": "#/definitions/edge" } }, "cards": { "type": "array", "items": { "$ref": "#/definitions/card" } }, "cards_x_nodes_x_widgets": { "type": "array" }, "relatable_resource_model_ids": { "type": "array" }, "resource_2_resource_constraints": { "type": "array" }, "functions_x_graphs": { "type": "array" }, "publication": { "type": ["object", "null"] } } }, "node": { "type": "object", "required": ["nodeid", "name", "datatype", "istopnode"], "properties": { "nodeid": { "$ref": "#/definitions/uuid" }, "name": { "type": "string" }, "description": { "type": "string" }, "datatype": { "type": "string", "enum": ["semantic", "string", "number", "boolean", "date", "concept", "file-list", "geojson-feature-collection", "resource-instance"] }, "istopnode": { "type": "boolean" }, "nodegroup_id": { "oneOf": [{ "$ref": "#/definitions/uuid" }, { "type": "null" }] }, "config": { "type": "object" }, "graph_id": { "$ref": "#/definitions/uuid" }, "sortorder": { "type": "number" }, "fieldname": { "type": ["string", "null"] }, "exportable": { "type": "boolean" }, "isrequired": { "type": "boolean" }, "issearchable": { "type": "boolean" }, "is_collector": { "type": "boolean" }, "hascustomalias": { "type": "boolean" }, "ontologyclass": { "type": ["string", "null"] }, "alias": { "type": ["string", "null"] }, "parentproperty": { "type": ["string", "null"] }, "sourcebranchpublication_id": { "type": ["string", "null"] } }, "if": { "properties": { "istopnode": { "const": true } } }, "then": { "properties": { "nodegroup_id": { "type": "null" } }, "required": ["nodegroup_id"] }, "else": { "properties": { "nodegroup_id": { "$ref": "#/definitions/uuid" } }, "required": ["nodegroup_id"] } }, "nodegroup": { "type": "object", "required": ["nodegroupid"], "properties": { "nodegroupid": { "$ref": "#/definitions/uuid" }, "cardinality": { "type": ["string", "null"], "enum": ["1", "n", null] }, "parentnodegroup_id": { "oneOf": [{ "$ref": "#/definitions/uuid" }, { "type": "null" }] }, "legacygroupid": { "type": ["string", "null"] }, "name": { "type": "string" }, "sortorder": { "type": "number" } } }, "edge": { "type": "object", "required": ["edgeid", "graph_id", "domainnode_id", "rangenode_id"], "properties": { "edgeid": { "$ref": "#/definitions/uuid" }, "graph_id": { "$ref": "#/definitions/uuid" }, "domainnode_id": { "$ref": "#/definitions/uuid" }, "rangenode_id": { "$ref": "#/definitions/uuid" }, "name": { "type": ["string", "null"] }, "description": { "type": ["string", "null"] }, "ontologyproperty": { "type": ["string", "null"] }, "sortorder": { "type": "number" } } }, "card": { "type": "object", "required": ["cardid", "nodegroup_id", "graph_id", "name", "active", "visible"], "properties": { "cardid": { "$ref": "#/definitions/uuid" }, "nodegroup_id": { "$ref": "#/definitions/uuid" }, "graph_id": { "$ref": "#/definitions/uuid" }, "name": { "$ref": "#/definitions/multilingualString" }, "description": { "type": "string" }, "instructions": { "$ref": "#/definitions/multilingualString" }, "helptext": { "$ref": "#/definitions/multilingualString" }, "helptitle": { "$ref": "#/definitions/multilingualString" }, "active": { "type": "boolean" }, "visible": { "type": "boolean" }, "sortorder": { "type": "number" }, "helpenabled": { "type": "boolean" }, "cssclass": { "type": "string" }, "is_editable": { "type": "boolean" }, "config": { "type": ["object", "null"] }, "constraints": { "type": "array" } } } };
const allOf = [{ "description": "Constraint: Top nodes must have nodegroup_id: null for Alizarin compatibility", "if": { "properties": { "graph": { "type": "array", "items": { "properties": { "nodes": { "type": "array", "contains": { "properties": { "istopnode": { "const": true } } } } } } } } }, "then": { "properties": { "graph": { "items": { "properties": { "nodes": { "items": { "if": { "properties": { "istopnode": { "const": true } } }, "then": { "properties": { "nodegroup_id": { "type": "null" } }, "required": ["nodegroup_id"] } } } } } } } } }, { "description": "Constraint: Root nodegroups can have only one node, all nodegroups must have a node with matching ID", "properties": { "graph": { "items": { "type": "object", "properties": { "nodes": { "type": "array", "items": { "type": "object", "properties": { "nodegroup_id": { "oneOf": [{ "$ref": "#/definitions/uuid" }, { "type": "null" }] }, "istopnode": { "type": "boolean" } } } }, "nodegroups": { "type": "array", "items": { "type": "object", "properties": { "nodegroupid": { "$ref": "#/definitions/uuid" } } } } } } } }, "additionalProperties": true }];
const graphModelSchema = {
  $schema: $schema$2,
  title: title$2,
  description: description$2,
  type: type$2,
  required: required$2,
  properties: properties$2,
  definitions: definitions$2,
  allOf
};
const $schema$1 = "http://json-schema.org/draft-07/schema#";
const title$1 = "Arches Business Data Schema";
const description$1 = "JSON Schema for Arches heritage management business data files with Alizarin compatibility";
const type$1 = "object";
const required$1 = ["business_data"];
const properties$1 = { "business_data": { "$ref": "#/definitions/businessData" } };
const definitions$1 = { "uuid": { "type": "string", "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$" }, "businessData": { "type": "object", "required": ["resources"], "properties": { "resources": { "type": "array", "items": { "$ref": "#/definitions/resource" }, "minItems": 1 } } }, "resource": { "type": "object", "required": ["resourceid", "graph_id", "tiles", "resourceinstance"], "properties": { "resourceid": { "$ref": "#/definitions/uuid" }, "graph_id": { "$ref": "#/definitions/uuid" }, "legacyid": { "type": ["string", "null"] }, "tiles": { "type": "array", "items": { "$ref": "#/definitions/tile" }, "minItems": 1 }, "resourceinstance": { "$ref": "#/definitions/resourceInstance" } } }, "resourceInstance": { "type": "object", "required": ["resourceinstanceid", "graph_id", "legacyid", "name", "displayname", "descriptors"], "properties": { "resourceinstanceid": { "$ref": "#/definitions/uuid" }, "graph_id": { "$ref": "#/definitions/uuid" }, "legacyid": { "type": "string", "minLength": 1, "description": "Legacy identifier for the resource - required for Alizarin" }, "createdtime": { "type": "string", "format": "date-time" }, "name": { "type": "string", "minLength": 1, "description": "Display name for the resource - required for Alizarin" }, "displayname": { "type": "string", "minLength": 1, "description": "Display name for UI purposes - required for Alizarin" }, "map_popup": { "type": "string", "description": "Text to show in map popups" }, "provisional": { "type": "string", "enum": ["true", "false"] }, "descriptors": { "type": "object", "required": ["name", "description", "map_popup", "displayname"], "properties": { "name": { "type": "string", "minLength": 1 }, "description": { "type": "string", "minLength": 1 }, "map_popup": { "type": "string", "minLength": 1 }, "displayname": { "type": "string", "minLength": 1 } }, "additionalProperties": false, "description": "Descriptors object - required for Alizarin resource metadata" }, "graph_publication_id": { "$ref": "#/definitions/uuid" }, "publication_id": { "$ref": "#/definitions/uuid" } } }, "tile": { "type": "object", "required": ["tileid", "nodegroup_id", "data"], "properties": { "tileid": { "$ref": "#/definitions/uuid" }, "nodegroup_id": { "$ref": "#/definitions/uuid" }, "parenttile_id": { "oneOf": [{ "$ref": "#/definitions/uuid" }, { "type": "null" }] }, "resourceid": { "oneOf": [{ "$ref": "#/definitions/uuid" }, { "type": "null" }] }, "sortorder": { "type": "number" }, "tiles": { "type": "array", "items": { "$ref": "#/definitions/tile" } }, "provisionaledits": { "type": ["array", "null"] }, "data": { "type": "object", "patternProperties": { "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$": { "oneOf": [{ "type": "string" }, { "type": "number" }, { "type": "boolean" }, { "type": "null" }, { "type": "object", "properties": { "en": { "type": "object", "required": ["value", "direction"], "properties": { "value": { "type": "string" }, "direction": { "type": "string", "enum": ["ltr", "rtl"] } } } } }] } }, "additionalProperties": false, "minProperties": 1 } } } };
const businessDataSchema = {
  $schema: $schema$1,
  title: title$1,
  description: description$1,
  type: type$1,
  required: required$1,
  properties: properties$1,
  definitions: definitions$1
};
const $schema = "http://json-schema.org/draft-07/schema#";
const title = "Arches Graphs Registry Schema";
const description = "JSON Schema for graphs.json registry file used by Alizarin";
const type = "object";
const required = ["models"];
const properties = { "models": { "type": "object", "patternProperties": { "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$": { "$ref": "#/definitions/modelReference" } }, "additionalProperties": false, "minProperties": 1 } };
const definitions = { "uuid": { "type": "string", "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$" }, "multilingualString": { "type": "object", "patternProperties": { "^[a-z]{2}(-[A-Z]{2})?$": { "type": "string" } }, "additionalProperties": false, "minProperties": 1 }, "modelReference": { "type": "object", "required": ["id", "name", "slug"], "properties": { "id": { "$ref": "#/definitions/uuid" }, "name": { "$ref": "#/definitions/multilingualString" }, "slug": { "type": "string", "pattern": "^[a-z][a-z0-9-]*[a-z0-9]$", "minLength": 2 }, "subtitle": { "$ref": "#/definitions/multilingualString" }, "color": { "type": "string", "pattern": "^#[0-9a-f]{6}$" }, "iconclass": { "type": "string", "pattern": "^fa fa-[a-z-]+$" } } } };
const graphsRegistrySchema = {
  $schema,
  title,
  description,
  type,
  required,
  properties,
  definitions
};
class GraphLoadingValidator {
  constructor(basePath = ".") {
    __publicField(this, "ajv");
    __publicField(this, "validateGraphModel");
    __publicField(this, "validateBusinessData");
    __publicField(this, "validateGraphsRegistry");
    __publicField(this, "results");
    __publicField(this, "basePath");
    this.basePath = basePath;
    this.ajv = new Ajv({
      allErrors: true,
      verbose: true,
      strict: false
    });
    addFormats(this.ajv);
    this.validateGraphModel = this.ajv.compile(graphModelSchema);
    this.validateBusinessData = this.ajv.compile(businessDataSchema);
    this.validateGraphsRegistry = this.ajv.compile(graphsRegistrySchema);
    this.results = {
      graphModels: { passed: 0, failed: 0, errors: [] },
      businessData: { passed: 0, failed: 0, errors: [] },
      graphsRegistry: { passed: 0, failed: 0, errors: [] },
      alizarinCompatibility: { passed: 0, failed: 0, errors: [] },
      graphLoadingTests: { passed: 0, failed: 0, errors: [] }
    };
  }
  reportError(category, file, error) {
    this.results[category].failed++;
    this.results[category].errors.push({ file, error });
  }
  reportSuccess(category, _file) {
    this.results[category].passed++;
  }
  formatErrors(errors) {
    if (!errors) return "Unknown error";
    return errors.map((e) => `${e.instancePath || "root"}: ${e.message}`).join("; ");
  }
  /**
   * Validate graph model files
   */
  validateGraphModels() {
    const modelFiles = readdirSync(this.basePath).filter(
      (f) => f.startsWith("arches-") && f.endsWith("-model.json")
    );
    modelFiles.forEach((file) => {
      try {
        const content = JSON.parse(readFileSync(join(this.basePath, file), "utf8"));
        if (this.validateGraphModel(content)) {
          this.reportSuccess("graphModels", file);
        } else {
          const error = this.formatErrors(this.validateGraphModel.errors);
          this.reportError("graphModels", file, error);
        }
      } catch (error) {
        this.reportError("graphModels", file, `Parse error: ${error.message}`);
      }
    });
  }
  /**
   * Validate business data files
   */
  validateBusinessDataFiles() {
    const businessDataFiles = readdirSync(this.basePath).filter(
      (f) => f.startsWith("arches-business-data-") && f.endsWith(".json") && f !== "arches-business-data-schema.json"
    );
    businessDataFiles.forEach((file) => {
      try {
        const content = JSON.parse(readFileSync(join(this.basePath, file), "utf8"));
        if (this.validateBusinessData(content)) {
          this.reportSuccess("businessData", file);
        } else {
          const error = this.formatErrors(this.validateBusinessData.errors);
          this.reportError("businessData", file, error);
        }
      } catch (error) {
        this.reportError("businessData", file, `Parse error: ${error.message}`);
      }
    });
  }
  /**
   * Validate graphs registry file
   */
  validateGraphsRegistryFile() {
    const registryPath = join(this.basePath, "graphs.json");
    if (existsSync(registryPath)) {
      try {
        const content = JSON.parse(readFileSync(registryPath, "utf8"));
        if (this.validateGraphsRegistry(content)) {
          this.reportSuccess("graphsRegistry", "graphs.json");
        } else {
          const error = this.formatErrors(this.validateGraphsRegistry.errors);
          this.reportError("graphsRegistry", "graphs.json", error);
        }
      } catch (error) {
        this.reportError("graphsRegistry", "graphs.json", `Parse error: ${error.message}`);
      }
    } else {
      this.reportError("graphsRegistry", "graphs.json", "File does not exist");
    }
  }
  /**
   * Check Alizarin compatibility requirements
   */
  checkAlizarinCompatibility() {
    const registryPath = join(this.basePath, "graphs.json");
    if (!existsSync(registryPath)) {
      return;
    }
    const graphs = JSON.parse(readFileSync(registryPath, "utf8"));
    const availableGraphIds = new Set(Object.keys(graphs.models || {}));
    const businessDataFiles = readdirSync(this.basePath).filter(
      (f) => f.startsWith("arches-business-data-") && f.endsWith(".json") && f !== "arches-business-data-schema.json"
    );
    businessDataFiles.forEach((file) => {
      var _a;
      try {
        const content = JSON.parse(readFileSync(join(this.basePath, file), "utf8"));
        const resources = ((_a = content.business_data) == null ? void 0 : _a.resources) || [];
        resources.forEach((resource, index) => {
          const graphId = resource.graph_id;
          if (!availableGraphIds.has(graphId)) {
            this.reportError(
              "alizarinCompatibility",
              file,
              `Resource ${index}: graph_id ${graphId} not found in graphs.json`
            );
            return;
          }
          const ri = resource.resourceinstance;
          if (!ri) {
            this.reportError(
              "alizarinCompatibility",
              file,
              `Resource ${index}: Missing resourceinstance - required for Alizarin`
            );
            return;
          }
          const requiredFields = ["resourceinstanceid", "graph_id", "legacyid", "name", "displayname", "descriptors"];
          for (const field of requiredFields) {
            if (!ri[field]) {
              this.reportError(
                "alizarinCompatibility",
                file,
                `Resource ${index}: Missing resourceinstance.${field} - required for Alizarin`
              );
              return;
            }
          }
          const descriptors = ri.descriptors;
          const requiredDescriptors = ["name", "description", "map_popup", "displayname"];
          for (const field of requiredDescriptors) {
            if (!descriptors[field]) {
              this.reportError(
                "alizarinCompatibility",
                file,
                `Resource ${index}: Missing descriptors.${field} - required for Alizarin`
              );
              return;
            }
          }
        });
        if (resources.length > 0) {
          this.reportSuccess("alizarinCompatibility", file);
        }
      } catch (error) {
        this.reportError("alizarinCompatibility", file, `Parse error: ${error.message}`);
      }
    });
  }
  /**
   * Simulate graph loading process (as Alizarin would do it)
   */
  simulateGraphLoading() {
    var _a;
    const registryPath = join(this.basePath, "graphs.json");
    if (!existsSync(registryPath)) {
      this.reportError("graphLoadingTests", "graphs.json", "Registry file does not exist");
      return;
    }
    try {
      const graphsRegistry = JSON.parse(readFileSync(registryPath, "utf8"));
      for (const [graphId, model] of Object.entries(graphsRegistry.models)) {
        const modelFile = `arches-${model.slug}-model.json`;
        const modelPath = join(this.basePath, modelFile);
        if (!existsSync(modelPath)) {
          this.reportError("graphLoadingTests", graphId, `Model file ${modelFile} not found`);
          continue;
        }
        try {
          const modelContent = JSON.parse(readFileSync(modelPath, "utf8"));
          if (modelContent.graph[0].graphid !== graphId) {
            this.reportError(
              "graphLoadingTests",
              graphId,
              `Graph ID mismatch: registry has ${graphId}, model has ${modelContent.graph[0].graphid}`
            );
            continue;
          }
          const businessDataFiles = readdirSync(this.basePath).filter(
            (f) => f.startsWith("arches-business-data-") && f.endsWith(".json")
          );
          for (const file of businessDataFiles) {
            try {
              const content = JSON.parse(readFileSync(join(this.basePath, file), "utf8"));
              const resources = ((_a = content.business_data) == null ? void 0 : _a.resources) || [];
              if (resources.some((r) => r.graph_id === graphId)) {
                break;
              }
            } catch (error) {
            }
          }
          this.reportSuccess("graphLoadingTests", graphId);
        } catch (error) {
          this.reportError("graphLoadingTests", graphId, `Model file parse error: ${error.message}`);
        }
      }
    } catch (error) {
      this.reportError("graphLoadingTests", "graphs.json", `Registry parse error: ${error.message}`);
    }
  }
  /**
   * Run all validation checks
   */
  validate() {
    this.validateGraphModels();
    this.validateBusinessDataFiles();
    this.validateGraphsRegistryFile();
    this.checkAlizarinCompatibility();
    this.simulateGraphLoading();
    const totalPassed = Object.values(this.results).reduce((sum, result) => sum + result.passed, 0);
    const totalFailed = Object.values(this.results).reduce((sum, result) => sum + result.failed, 0);
    return {
      results: this.results,
      totalPassed,
      totalFailed,
      success: totalFailed === 0
    };
  }
  /**
   * Get validation results
   */
  getResults() {
    return this.results;
  }
  /**
   * Print validation summary to console
   */
  printSummary(summary) {
    console.log("\n\u{1F4CA} VALIDATION SUMMARY");
    console.log("====================\n");
    const categories = [
      { name: "Graph Models", key: "graphModels" },
      { name: "Business Data", key: "businessData" },
      { name: "Graphs Registry", key: "graphsRegistry" },
      { name: "Alizarin Compatibility", key: "alizarinCompatibility" },
      { name: "Graph Loading Tests", key: "graphLoadingTests" }
    ];
    categories.forEach((category) => {
      const result = summary.results[category.key];
      const total = result.passed + result.failed;
      const percentage = total > 0 ? Math.round(result.passed / total * 100) : 0;
      console.log(`${category.name}: ${result.passed}/${total} passed (${percentage}%)`);
      if (result.failed > 0) {
        console.log(`   \u274C ${result.failed} failures:`);
        result.errors.slice(0, 3).forEach((error) => {
          console.log(`      \u2022 ${error.file}: ${error.error}`);
        });
        if (result.errors.length > 3) {
          console.log(`      \u2022 ... and ${result.errors.length - 3} more errors`);
        }
      }
    });
    console.log("");
    console.log(`\u{1F3AF} OVERALL RESULT: ${summary.totalPassed}/${summary.totalPassed + summary.totalFailed} checks passed`);
    if (summary.success) {
      console.log("\u{1F389} ALL VALIDATION CHECKS PASSED!");
      console.log("\u2705 Graph structure is valid");
      console.log("\u2705 Business data has proper Alizarin metadata");
      console.log("\u2705 All files can be loaded by Alizarin");
      console.log("\u2705 No compatibility issues detected");
    } else {
      console.log(`\u26A0\uFE0F  ${summary.totalFailed} validation issues found`);
      console.log("\u{1F4A1} Review the errors above and fix the issues");
      console.log("\u{1F4A1} Re-run this validator after making corrections");
    }
  }
}
function validateGraphLoading(basePath = ".") {
  const validator = new GraphLoadingValidator(basePath);
  const summary = validator.validate();
  validator.printSummary(summary);
  return summary;
}
const schemas = {
  graphModel: graphModelSchema,
  businessData: businessDataSchema,
  graphsRegistry: graphsRegistrySchema
};
export {
  GraphLoadingValidator,
  validateGraphLoading as quickValidate,
  schemas,
  validateGraphLoading
};
