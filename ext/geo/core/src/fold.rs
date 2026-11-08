//! Folding `geojson-feature-collection` values into one geometry summary.
//!
//! Summary: `{"count", "bbox": [min_lng, min_lat, max_lng, max_lat],
//! "geometry_types": [...]}`. `bbox` is absent until a position is seen, and
//! `geometry_types` is sorted, so the summary is the same whatever order values
//! are folded or merged in.
//!
//! The bbox is a plain lon/lat envelope: it does not handle geometries that
//! cross the antimeridian (it will span the globe), and for sparse layers it
//! over-claims the region between clusters. A hull or cell-set is the answer to
//! the second, and belongs here when needed.

use std::collections::BTreeSet;

use alizarin_core::extension_type_registry::ExtensionError;
use serde_json::{json, Value};

/// The empty geometry summary.
pub fn identity() -> Value {
    json!({ "count": 0 })
}

/// Fold feature-collection values into `acc`. An empty value (`{}`) counts but
/// adds no geometry; anything that is not GeoJSON fails the fold.
pub fn fold(acc: Value, values: &[&Value]) -> Result<Value, ExtensionError> {
    let mut summary = Summary::read(&acc)?;
    for value in values {
        summary.count += 1;
        if value.as_object().is_some_and(|o| o.is_empty()) {
            continue;
        }
        let gj = geojson::GeoJson::from_json_value((*value).clone())
            .map_err(|e| ExtensionError::new(format!("invalid GeoJSON: {e}")))?;
        match gj {
            geojson::GeoJson::FeatureCollection(fc) => {
                for feature in fc.features {
                    if let Some(geometry) = feature.geometry {
                        summary.add(&geometry.value);
                    }
                }
            }
            geojson::GeoJson::Feature(f) => {
                if let Some(geometry) = f.geometry {
                    summary.add(&geometry.value);
                }
            }
            geojson::GeoJson::Geometry(g) => summary.add(&g.value),
        }
    }
    Ok(summary.write())
}

/// Merge two geometry summaries.
pub fn merge(a: Value, b: Value) -> Result<Value, ExtensionError> {
    let mut summary = Summary::read(&a)?;
    let other = Summary::read(&b)?;
    summary.count += other.count;
    if let Some([x0, y0, x1, y1]) = other.bbox {
        summary.extend(x0, y0);
        summary.extend(x1, y1);
    }
    summary.types.extend(other.types);
    Ok(summary.write())
}

#[derive(Default)]
struct Summary {
    count: u64,
    bbox: Option<[f64; 4]>,
    types: BTreeSet<String>,
}

impl Summary {
    fn read(value: &Value) -> Result<Summary, ExtensionError> {
        let bad = || ExtensionError::new(format!("not a geometry summary: {value}"));
        let count = value.get("count").and_then(Value::as_u64).ok_or_else(bad)?;
        let bbox = match value.get("bbox") {
            None => None,
            Some(b) => {
                let n: Vec<f64> = b
                    .as_array()
                    .ok_or_else(bad)?
                    .iter()
                    .map(|v| v.as_f64().ok_or_else(bad))
                    .collect::<Result<_, _>>()?;
                Some(<[f64; 4]>::try_from(n).map_err(|_| bad())?)
            }
        };
        let types = match value.get("geometry_types") {
            None => BTreeSet::new(),
            Some(t) => t
                .as_array()
                .ok_or_else(bad)?
                .iter()
                .map(|v| v.as_str().map(str::to_string).ok_or_else(bad))
                .collect::<Result<_, _>>()?,
        };
        Ok(Summary { count, bbox, types })
    }

    fn write(&self) -> Value {
        let mut out = json!({ "count": self.count });
        if let Some(bbox) = self.bbox {
            out["bbox"] = json!(bbox);
        }
        if !self.types.is_empty() {
            out["geometry_types"] = json!(self.types);
        }
        out
    }

    fn add(&mut self, geometry: &geojson::Value) {
        use geojson::Value::*;
        self.types.insert(geometry.type_name().to_string());
        match geometry {
            Point(p) => self.position(p),
            MultiPoint(ps) | LineString(ps) => ps.iter().for_each(|p| self.position(p)),
            MultiLineString(ls) | Polygon(ls) => ls.iter().flatten().for_each(|p| self.position(p)),
            MultiPolygon(polys) => polys
                .iter()
                .flatten()
                .flatten()
                .for_each(|p| self.position(p)),
            GeometryCollection(gs) => gs.iter().for_each(|g| self.add(&g.value)),
        }
    }

    fn position(&mut self, p: &[f64]) {
        if let [x, y, ..] = p {
            self.extend(*x, *y);
        }
    }

    fn extend(&mut self, x: f64, y: f64) {
        let b = self.bbox.get_or_insert([x, y, x, y]);
        b[0] = b[0].min(x);
        b[1] = b[1].min(y);
        b[2] = b[2].max(x);
        b[3] = b[3].max(y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fc(geometries: &[Value]) -> Value {
        json!({
            "type": "FeatureCollection",
            "features": geometries.iter().map(|g| json!({
                "type": "Feature", "properties": {}, "geometry": g
            })).collect::<Vec<_>>()
        })
    }

    fn point(x: f64, y: f64) -> Value {
        json!({ "type": "Point", "coordinates": [x, y] })
    }

    #[test]
    fn folds_to_a_bbox_and_sorted_geometry_types() {
        let polygon = json!({
            "type": "Polygon",
            "coordinates": [[[-6.0, 54.0], [-5.0, 54.0], [-5.0, 55.0], [-6.0, 54.0]]]
        });
        let a = fc(&[point(-7.5, 54.5), polygon]);
        let b = fc(&[point(-5.5, 53.0)]);
        let summary = fold(identity(), &[&a, &b, &json!({})]).unwrap();
        assert_eq!(
            summary,
            json!({
                "count": 3,
                "bbox": [-7.5, 53.0, -5.0, 55.0],
                "geometry_types": ["Point", "Polygon"]
            })
        );
    }

    #[test]
    fn merge_matches_one_fold_in_either_order() {
        let a = fc(&[point(1.0, 2.0)]);
        let b = fc(&[
            point(-3.0, 8.0),
            json!({ "type": "LineString", "coordinates": [[0.0, 0.0], [5.0, 1.0]] }),
        ]);
        let whole = fold(identity(), &[&a, &b]).unwrap();
        let fa = fold(identity(), &[&a]).unwrap();
        let fb = fold(identity(), &[&b]).unwrap();
        assert_eq!(merge(fa.clone(), fb.clone()).unwrap(), whole);
        assert_eq!(merge(fb, fa).unwrap(), whole);
    }

    #[test]
    fn empty_values_count_but_add_no_bbox() {
        assert_eq!(
            fold(identity(), &[&json!({})]).unwrap(),
            json!({ "count": 1 })
        );
    }

    #[test]
    fn invalid_geojson_fails_the_fold() {
        assert!(fold(identity(), &[&json!({ "type": "Nonsense" })]).is_err());
    }

    /// The seam end to end: core's runner, this handler registered, folding a
    /// geometry path over a registry of resources.
    #[test]
    fn core_runner_folds_geometry_through_the_registered_handler() {
        use alizarin_core::extension_type_registry::ExtensionTypeRegistry;
        use alizarin_core::graph::StaticResourceRegistry;
        use alizarin_core::graph_model_access::GraphModelAccess;
        use alizarin_core::{StaticResource, StaticTile};

        const C: &str = "http://www.cidoc-crm.org/cidoc-crm/";
        let (graph, _) = alizarin_core::build_graph_from_model_csvs(
            &format!("name,ontology_class,author,description,is_resource\nSite,{C}E27,,,true"),
            &format!(
                "parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property,description,collection_name,required,searchable,exportable,sortorder\n\
                 ,location,location,geojson-feature-collection,n,{C}E53,{C}P53,,,,,,"
            ),
            None,
            "https://example.org/test",
            Default::default(),
        )
        .unwrap();
        let node = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("location"))
            .unwrap();

        let resource = |id: &str, geometry: Value| -> StaticResource {
            let mut tile = StaticTile::new_empty(node.nodegroup_id.clone().unwrap());
            tile.tileid = Some(format!("{id}-tile"));
            tile.resourceinstance_id = id.to_string();
            tile.data.insert(node.nodeid.clone(), fc(&[geometry]));
            serde_json::from_value(json!({
                "resourceinstance": {
                    "resourceinstanceid": id, "graph_id": graph.graphid, "name": id,
                    "descriptors": {}
                },
                "tiles": [tile],
            }))
            .unwrap()
        };
        let mut resources = StaticResourceRegistry::new();
        resources.merge_from_resources(
            &[
                resource("r1", point(-6.0, 54.0)),
                resource("r2", point(-8.0, 55.5)),
            ],
            true,
            false,
        );
        let mut handlers = ExtensionTypeRegistry::new();
        crate::register(&mut handlers);

        let results = alizarin_core::fold::fold_resources(
            &resources,
            &["r1".to_string(), "r2".to_string()],
            &GraphModelAccess::from_graph(&graph),
            &["location".to_string()],
            Some(&handlers),
        )
        .unwrap();
        assert_eq!(results[0].values, 2);
        assert_eq!(
            results[0].summary,
            Some(json!({
                "count": 2,
                "bbox": [-8.0, 54.0, -6.0, 55.5],
                "geometry_types": ["Point"]
            }))
        );
    }
}
