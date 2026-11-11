//! Folding `geojson-feature-collection` values into one geometry summary.
//!
//! Summary: `{"count", "bbox": [min_lng, min_lat, max_lng, max_lat],
//! "geometry_types": [...]}`. `bbox` is absent until a position is seen, and
//! `geometry_types` is sorted, so the summary is the same whatever order values
//! are folded or merged in. `geometry_types` is inclusive: the types seen among
//! folded values, never shrunk by a later removal (see `notes/spans-scope.md`,
//! "Incremental updates").
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
        summary.absorb(Summary::of_value(value)?);
    }
    Ok(summary.write())
}

/// Whether any geometry folded into `summary` could intersect `predicate`'s
/// `{"bbox": [min_lng, min_lat, max_lng, max_lat]}` (inclusive envelopes).
/// Conservative: `false` only when the summary's bbox is disjoint from it, or
/// it folded no geometry at all; `true` for anything else it can't decide.
pub fn may_match(summary: &Value, predicate: &Value) -> bool {
    let Some(wanted) = predicate.get("bbox").and_then(|b| {
        let n: Vec<f64> = b.as_array()?.iter().map(Value::as_f64).collect::<Option<_>>()?;
        <[f64; 4]>::try_from(n).ok()
    }) else {
        return true;
    };
    let Ok(summary) = Summary::read(summary) else {
        return true;
    };
    match summary.bbox {
        None => false, // no geometry folded
        Some(have) => {
            !(have[2] < wanted[0] || have[0] > wanted[2] || have[3] < wanted[1] || have[1] > wanted[3])
        }
    }
}

/// Merge two geometry summaries.
pub fn merge(a: Value, b: Value) -> Result<Value, ExtensionError> {
    let mut summary = Summary::read(&a)?;
    summary.absorb(Summary::read(&b)?);
    Ok(summary.write())
}

/// Remove previously folded values from `acc`. Returns the summary and whether
/// it stayed exact: a removed geometry whose own bbox touches an edge of the
/// summary's bbox leaves that edge in place (a superset), so it is not exact.
/// `geometry_types` is inclusive and never shrinks. Removing the last value
/// gives the empty summary.
pub fn remove(acc: Value, values: &[&Value]) -> Result<(Value, bool), ExtensionError> {
    let mut summary = Summary::read(&acc)?;
    let mut exact = true;
    for value in values {
        let removed = Summary::of_value(value)?;
        if summary.count == 0 {
            exact = false; // removing from nothing: the caller's data is inconsistent
            continue;
        }
        summary.count -= 1;
        if let (Some(edges), Some(r)) = (summary.bbox, removed.bbox) {
            exact &=
                !(r[0] <= edges[0] || r[1] <= edges[1] || r[2] >= edges[2] || r[3] >= edges[3]);
        }
    }
    if summary.count == 0 {
        return Ok((identity(), true));
    }
    Ok((summary.write(), exact))
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

    /// The summary of one value: count 1, its bbox and geometry types. An empty
    /// value (`{}`) counts but has no geometry; anything else must be GeoJSON.
    fn of_value(value: &Value) -> Result<Summary, ExtensionError> {
        let mut summary = Summary {
            count: 1,
            ..Summary::default()
        };
        if value.as_object().is_some_and(|o| o.is_empty()) {
            return Ok(summary);
        }
        let gj = geojson::GeoJson::from_json_value(value.clone())
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
        Ok(summary)
    }

    /// Fold another summary into this one.
    fn absorb(&mut self, other: Summary) {
        self.count += other.count;
        if let Some([x0, y0, x1, y1]) = other.bbox {
            self.extend(x0, y0);
            self.extend(x1, y1);
        }
        self.types.extend(other.types);
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

    #[test]
    fn removal_marks_loose_only_when_an_edge_is_touched() {
        use alizarin_core::datatype_fold::{fold_identity, fold_remove, fold_values, is_loose};
        use alizarin_core::extension_type_registry::ExtensionTypeRegistry;

        let mut handlers = ExtensionTypeRegistry::new();
        crate::register(&mut handlers);
        let dt = crate::DATATYPE_NAME;
        let reg = Some(&handlers);
        let (west, middle, east) = (
            fc(&[point(-10.0, 54.0)]),
            fc(&[point(-7.0, 54.5)]),
            fc(&[point(-5.0, 55.0)]),
        );
        let acc = fold_values(
            dt,
            fold_identity(dt, None, reg).unwrap(),
            &[&west, &middle, &east],
            None,
            reg,
        )
        .unwrap()
        .summary;

        let inner = fold_remove(dt, acc.clone(), &[&middle], None, reg)
            .unwrap()
            .unwrap();
        assert!(!is_loose(&inner), "an interior geometry");
        assert_eq!(inner["count"], json!(2));
        assert_eq!(inner["bbox"], acc["bbox"]);

        let edge = fold_remove(dt, acc.clone(), &[&west], None, reg)
            .unwrap()
            .unwrap();
        assert!(is_loose(&edge), "the westmost geometry");
        assert_eq!(edge["bbox"], acc["bbox"], "the edge is kept: a superset");
        assert_eq!(
            edge["geometry_types"],
            json!(["Point"]),
            "types are inclusive"
        );

        let empty = fold_remove(dt, edge, &[&middle, &east], None, reg)
            .unwrap()
            .unwrap();
        assert_eq!(empty, identity(), "nothing left is exactly empty");
    }

    #[test]
    fn removing_a_value_without_geometry_stays_exact() {
        let acc = fold(identity(), &[&fc(&[point(1.0, 2.0)]), &json!({})]).unwrap();
        let (summary, exact) = remove(acc, &[&json!({})]).unwrap();
        assert!(exact);
        assert_eq!(summary["count"], json!(1));
    }
}
