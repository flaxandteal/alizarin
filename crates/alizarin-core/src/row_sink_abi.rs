//! Row sinks across a binding boundary: the host side of
//! [`RowSinkInfo`], as [`derive_abi`](crate::derive_abi) is for derive
//! providers.
//!
//! An extension module that writes a bulk output (alizarin-pg's COPY text)
//! can't convert trees itself: a copy of core in it would have its own,
//! empty, registries. So it links none, and passes the host a
//! [`RowSinkInfo`]; the host converts the batch
//! with its own graph, extension handlers and RDM cache, then hands the sink
//! each tile and resource as a row of slices borrowed from the converted
//! resources. Nothing is copied or parsed at the boundary: the only encoding
//! is each tile's data as JSON text, which the row carries.

use std::sync::Arc;

use alizarin_extension_api::{abi_fingerprint, ResourceRow, RowSinkInfo, StrRef, TileRow};
use serde::Serialize;

use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::graph::{StaticGraph, StaticResource};
use crate::json_conversion::{batch_convert_trees, label_resolution, BatchOptions};

/// What a binding's `batch_trees_to_row_sink` returns: the counts written
/// to the sink, and the per-tree errors (in non-strict mode) and warnings.
#[derive(Debug, Serialize)]
pub struct RowSinkBatch {
    pub resource_count: usize,
    pub tile_count: usize,
    pub errors: Vec<String>,
    pub error_count: usize,
    pub warnings: Vec<String>,
    pub warning_count: usize,
}

/// Hand `resource`'s tiles, then the resource, to `sink`. Returns the number
/// of tiles.
///
/// # Safety
/// `sink`'s fn pointers must be valid for `sink.ctx`.
unsafe fn write_resource(
    sink: &RowSinkInfo,
    resource: &StaticResource,
    language: &str,
) -> Result<usize, String> {
    let tiles = resource.tiles.as_deref().unwrap_or_default();
    for tile in tiles {
        let data = serde_json::to_string(&tile.data)
            .map_err(|e| format!("Failed to serialize tile data: {e}"))?;
        let edits = tile
            .provisionaledits
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| format!("Failed to serialize provisional edits: {e}"))?;
        let row = TileRow {
            tileid: StrRef::opt(tile.tileid.as_deref()),
            resourceinstance_id: StrRef::new(&tile.resourceinstance_id),
            parenttile_id: StrRef::opt(tile.parenttile_id.as_deref()),
            nodegroup_id: StrRef::new(&tile.nodegroup_id),
            data_json: StrRef::new(&data),
            provisionaledits_json: StrRef::opt(edits.as_deref()),
            sortorder: tile.sortorder.unwrap_or(0),
            has_sortorder: tile.sortorder.is_some(),
        };
        (sink.tile_fn)(sink.ctx, &row);
    }
    let ri = &resource.resourceinstance;
    let row = ResourceRow {
        resourceinstanceid: StrRef::new(&ri.resourceinstanceid),
        graph_id: StrRef::new(&ri.graph_id),
        graph_publication_id: StrRef::opt(ri.graph_publication_id.as_deref()),
        name: StrRef::opt(ri.descriptors.name.as_deref()),
        description: StrRef::opt(ri.descriptors.description.as_deref()),
        map_popup: StrRef::opt(ri.descriptors.map_popup.as_deref()),
        descriptors_empty: ri.descriptors.is_empty(),
        legacyid: StrRef::opt(ri.legacyid.as_deref()),
        createdtime: StrRef::opt(ri.createdtime.as_deref()),
        principaluser_id: ri.principaluser_id.unwrap_or(0),
        has_principaluser_id: ri.principaluser_id.is_some(),
        language: StrRef::new(language),
    };
    (sink.resource_fn)(sink.ctx, &row);
    Ok(tiles.len())
}

/// Convert a JSON array of trees as `batch_trees_to_tiles_from_json` does,
/// then hand every converted tile and resource to `sink`, in order.
///
/// # Safety
/// `sink` must point to a valid [`RowSinkInfo`] whose fn pointers and `ctx`
/// stay valid for this call. Its ABI fingerprint is checked before anything
/// else is read.
pub unsafe fn batch_trees_to_row_sink_from_json(
    trees_json: &str,
    graph: &StaticGraph,
    opts: &BatchOptions,
    ext_registry: Option<&ExtensionTypeRegistry>,
    cache: Option<Arc<crate::RdmCache>>,
    sink: *const RowSinkInfo,
) -> Result<RowSinkBatch, String> {
    if sink.is_null() {
        return Err("null RowSinkInfo pointer".to_string());
    }
    let sink = &*sink;
    if sink.abi != abi_fingerprint() {
        return Err(format!(
            "row sink ABI mismatch: built against {:?}, host is {:?} — \
             rebuild the extension against this alizarin version.",
            sink.abi,
            abi_fingerprint()
        ));
    }
    let trees: Vec<serde_json::Value> =
        serde_json::from_str(trees_json).map_err(|e| format!("Failed to parse trees: {e}"))?;
    let labels = label_resolution(graph, ext_registry, opts.from_camel, cache);
    let lookup = labels.as_ref().map(|(a, c)| (a, c.as_ref()));
    let outcome = batch_convert_trees(&trees, graph, ext_registry, lookup, opts)?;

    let language = crate::get_current_language();
    let mut tile_count = 0;
    for resource in &outcome.resources {
        tile_count += write_resource(sink, resource, &language)?;
    }
    Ok(RowSinkBatch {
        resource_count: outcome.resources.len(),
        tile_count,
        error_count: outcome.errors.len(),
        warning_count: outcome.warnings.len(),
        errors: outcome.errors,
        warnings: outcome.warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    #[derive(Default)]
    struct Collected {
        tiles: Vec<(Option<String>, String, String, Option<i32>)>,
        resources: Vec<(String, Option<String>, bool, String)>,
    }

    unsafe extern "C" fn on_tile(ctx: *mut c_void, row: *const TileRow) {
        let out = &mut *(ctx as *mut Collected);
        let row = &*row;
        out.tiles.push((
            row.tileid.as_str().map(str::to_string),
            row.nodegroup_id.as_str().unwrap().to_string(),
            row.data_json.as_str().unwrap().to_string(),
            row.has_sortorder.then_some(row.sortorder),
        ));
    }

    unsafe extern "C" fn on_resource(ctx: *mut c_void, row: *const ResourceRow) {
        let out = &mut *(ctx as *mut Collected);
        let row = &*row;
        out.resources.push((
            row.resourceinstanceid.as_str().unwrap().to_string(),
            row.name.as_str().map(str::to_string),
            row.descriptors_empty,
            row.language.as_str().unwrap().to_string(),
        ));
    }

    fn group_graph() -> StaticGraph {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/data/models/Group.json"
        );
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let mut graph: StaticGraph = serde_json::from_value(json["graph"][0].clone()).unwrap();
        graph.build_indices();
        graph
    }

    fn opts(graph: &StaticGraph) -> BatchOptions<'_> {
        BatchOptions {
            graph_id: &graph.graphid,
            from_camel: false,
            strict: true,
            random_ids: false,
            resolve_markers: false,
            id_keys: None,
            scopes: None,
            tile_namespace: None,
        }
    }

    fn sink(out: &mut Collected) -> RowSinkInfo {
        RowSinkInfo {
            abi: abi_fingerprint(),
            ctx: out as *mut Collected as *mut c_void,
            tile_fn: on_tile,
            resource_fn: on_resource,
        }
    }

    #[test]
    fn hands_the_sink_each_converted_tile_then_its_resource() {
        let graph = group_graph();
        let trees = r#"[{"resourceinstanceid": "11111111-1111-1111-1111-111111111111",
            "basic_info": [{"name": {"en": "Test Group"}}]}]"#;
        let mut out = Collected::default();
        let info = sink(&mut out);
        let batch = unsafe {
            batch_trees_to_row_sink_from_json(trees, &graph, &opts(&graph), None, None, &info)
        }
        .unwrap();

        assert_eq!(
            (batch.resource_count, batch.tile_count),
            (1, out.tiles.len())
        );
        assert_eq!(batch.error_count, 0);
        assert!(out.tiles.iter().any(|t| t.2.contains("Test Group")));
        assert_eq!(out.resources.len(), 1);
        assert_eq!(out.resources[0].0, "11111111-1111-1111-1111-111111111111");
        assert_eq!(out.resources[0].3, crate::get_current_language());
    }

    #[test]
    fn refuses_a_sink_built_against_another_abi() {
        let graph = group_graph();
        let mut out = Collected::default();
        let mut info = sink(&mut out);
        info.abi.version += 1;
        let err = unsafe {
            batch_trees_to_row_sink_from_json("[]", &graph, &opts(&graph), None, None, &info)
        }
        .unwrap_err();
        assert!(err.contains("ABI mismatch"), "{err}");
    }
}
