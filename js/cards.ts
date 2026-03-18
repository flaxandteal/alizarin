// The Widgets here are AGPLv3 from Arches DB setup.
//
import {
  StaticNode
} from "./static-types";

class CardComponent {
  id: string;
  name: string;

  constructor(id: string, name: string) {
    this.id = id;
    this.name = name;
  }
};

class Widget {
  id: string;
  name: string;
  datatype: string;
  defaultConfig: string; // as JSON - always need a fresh copy

  constructor(id: string, name: string, datatype: string, defaultConfig: string) {
    this.id = id;
    this.name = name;
    this.datatype = datatype;
    this.defaultConfig = defaultConfig;
  }

  getDefaultConfig(): {[key: string]: any} {
    return JSON.parse(this.defaultConfig);
  }
};
const DEFAULT_CARD_COMPONENT = new CardComponent(
  'f05e4d3a-53c1-11e8-b0ea-784f435179ea',
  'Default Card'
);
const _WIDGET_VALUES: [string, string, string, string][] = [
    ['10000000-0000-0000-0000-000000000001', 'text-widget', 'string', '{ "placeholder": "Enter text", "width": "100%", "maxLength": null}'],
    ['10000000-0000-0000-0000-000000000002', 'concept-select-widget', 'concept', '{ "placeholder": "Select an option", "options": [] }'],
    ['10000000-0000-0000-0000-000000000012', 'concept-multiselect-widget', 'concept-list', '{ "placeholder": "Select an option", "options": [] }'],
    ['10000000-0000-0000-0000-000000000015', 'domain-select-widget', 'domain-value', '{ "placeholder": "Select an option" }'],
    ['10000000-0000-0000-0000-000000000016', 'domain-multiselect-widget', 'domain-value-list', '{ "placeholder": "Select an option" }'],
    ['10000000-0000-0000-0000-000000000003', 'switch-widget', 'boolean', '{ "subtitle": "Click to switch"}'],
    ['10000000-0000-0000-0000-000000000004', 'datepicker-widget', 'date', `{
      "placeholder": "Enter date",
      "viewMode": "days",
      "dateFormat": "YYYY-MM-DD",
      "minDate": false,
      "maxDate": false
    }`],
    ['10000000-0000-0000-0000-000000000005', 'rich-text-widget', 'string', '{}'],
    ['10000000-0000-0000-0000-000000000006', 'radio-boolean-widget', 'boolean', '{"trueLabel": "Yes", "falseLabel": "No"}'],
    ['10000000-0000-0000-0000-000000000007', 'map-widget', 'geojson-feature-collection', `{
      "basemap": "streets",
      "geometryTypes": [{"text":"Point", "id":"Point"}, {"text":"Line", "id":"Line"}, {"text":"Polygon", "id":"Polygon"}],
      "overlayConfigs": [],
      "overlayOpacity": 0.0,
      "geocodeProvider": "MapzenGeocoder",
      "zoom": 0,
      "maxZoom": 20,
      "minZoom": 0,
      "centerX": 0,
      "centerY": 0,
      "pitch": 0.0,
      "bearing": 0.0,
      "geocodePlaceholder": "Search",
      "geocoderVisible": true,
      "featureColor": null,
      "featureLineWidth": null,
      "featurePointSize": null
    }`],
    ['10000000-0000-0000-0000-000000000008', 'number-widget', 'number', '{ "placeholder": "Enter number", "width": "100%", "min":"", "max":""}'],
    ['10000000-0000-0000-0000-000000000009', 'concept-radio-widget', 'concept', '{ "options": [] }'],
    ['10000000-0000-0000-0000-000000000013', 'concept-checkbox-widget', 'concept-list', '{ "options": [] }'],
    ['10000000-0000-0000-0000-000000000017', 'domain-radio-widget', 'domain-value', '{}'],
    ['10000000-0000-0000-0000-000000000018', 'domain-checkbox-widget', 'domain-value-list', '{}'],
    ['10000000-0000-0000-0000-000000000019', 'file-widget', 'file-list', '{"acceptedFiles": "", "maxFilesize": "200"}'],
];
const WIDGETS: {[key: string]: Widget} = Object.fromEntries(_WIDGET_VALUES.map((constructor: [string, string, string, string]): [string, Widget] => [constructor[1], new Widget(...constructor)]));

function getDefaultWidgetForNode(node: StaticNode, preferences: {[key: string]: string} = {}) {
  const datatype = node.datatype;

  // For example, you can use this to prefer a rich-text field.
  if (datatype in preferences) {
    return WIDGETS[preferences[datatype]];
  }

  if (datatype === 'semantic') {
    throw Error("Not default widget for a semantic node");
  } else if (datatype === 'number') {
    return WIDGETS['number-widget'];
  } else if (datatype === 'string') {
    return WIDGETS['text-widget'];
  } else if (datatype === 'concept') {
    return WIDGETS['concept-select-widget'];
  } else if (datatype === 'concept-list') {
    return WIDGETS['concept-multiselect-widget'];
  } else if (datatype === 'domain-value') {
    return WIDGETS['domain-select-widget'];
  } else if (datatype === 'domain-value-list') {
    return WIDGETS['domain-multiselect-widget'];
  } else if (datatype === 'geojson-feature-collection') {
    return WIDGETS['geojson-feature-collection'];
  } else if (datatype === 'boolean') {
    return WIDGETS['switch-widget'];
  } else if (datatype === 'date') {
    return WIDGETS['datepicker-widget'];
  } else {
    throw Error(`No default widget for ${datatype} datatype - perhaps you could supply a manual preference`);
  }
}

export { DEFAULT_CARD_COMPONENT, CardComponent, getDefaultWidgetForNode, Widget };
