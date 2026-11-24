//! `batch_trees_to_pg_copy`: trees to COPY text, converted by alizarin itself.
//!
//! This module converts nothing and links no alizarin-core (only pg core's
//! field writers and the ABI types): a copy of core here would have its own,
//! empty, graph, extension and RDM registries. It passes alizarin's
//! `batch_trees_to_row_sink` a [`RowSinkInfo`], and alizarin hands back each
//! converted tile and resource as a row of slices borrowed from its own
//! memory, which are escaped straight into the COPY buffers here.

use std::ffi::c_void;

use alizarin_extension_api::{abi_fingerprint, ResourceRow, RowSinkInfo, TileRow};
use alizarin_pg_core::{
    write_resource_fields, write_tile_fields, ResourceCopyOptions, ResourceFields, TileFields,
};

/// The COPY text written so far, for the `tiles` and `resource_instances`
/// tables.
struct CopySink<'a> {
    tiles: Vec<u8>,
    resources: Vec<u8>,
    opts: ResourceCopyOptions<'a>,
}

/// # Safety
/// `ctx` must be the [`CopySink`] the [`RowSinkInfo`] was built over, and
/// `row` a live [`TileRow`] (both guaranteed by the host for the call).
unsafe extern "C" fn on_tile(ctx: *mut c_void, row: *const TileRow) {
    let sink = &mut *(ctx as *mut CopySink);
    let row = &*row;
    let fields = TileFields {
        tileid: row.tileid.as_str(),
        resourceinstance_id: row.resourceinstance_id.as_str().unwrap_or_default(),
        parenttile_id: row.parenttile_id.as_str(),
        tiledata: row.data_json.as_str().unwrap_or("{}"),
        nodegroup_id: row.nodegroup_id.as_str().unwrap_or_default(),
        sortorder: row.has_sortorder.then_some(row.sortorder),
        provisionaledits: row.provisionaledits_json.as_str(),
    };
    // Writing to a Vec cannot fail.
    let _ = write_tile_fields(&mut sink.tiles, &fields);
}

/// # Safety
/// As [`on_tile`], for a [`ResourceRow`].
unsafe extern "C" fn on_resource(ctx: *mut c_void, row: *const ResourceRow) {
    let sink = &mut *(ctx as *mut CopySink);
    let row = &*row;
    let fields = ResourceFields {
        resourceinstanceid: row.resourceinstanceid.as_str().unwrap_or_default(),
        graph_id: row.graph_id.as_str().unwrap_or_default(),
        graph_publication_id: row.graph_publication_id.as_str(),
        name: row.name.as_str(),
        description: row.description.as_str(),
        map_popup: row.map_popup.as_str(),
        descriptors_empty: row.descriptors_empty,
        legacyid: row.legacyid.as_str(),
        createdtime: row.createdtime.as_str(),
        principaluser_id: row.has_principaluser_id.then_some(row.principaluser_id),
        language: row.language.as_str().unwrap_or_default(),
    };
    // Writing to a Vec cannot fail.
    let _ = write_resource_fields(&mut sink.resources, &fields, &sink.opts);
}

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyModule};

    /// The capsule's name: the capsule keeps this pointer, so it is static.
    const SINK_CAPSULE_NAME: &[u8] = b"alizarin_pg.row_sink\0";

    #[pyfunction]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        trees_json,
        graph_id,
        graph_publication_id=None,
        lifecycle_state_id=None,
        created_time=None,
        from_camel=false,
        strict=true,
        id_keys=None,
        resolve_markers=true,
        random_ids=false,
    ))]
    fn batch_trees_to_pg_copy(
        py: Python,
        trees_json: String,
        graph_id: String,
        graph_publication_id: Option<String>,
        lifecycle_state_id: Option<String>,
        created_time: Option<String>,
        from_camel: bool,
        strict: bool,
        id_keys: Option<Vec<String>>,
        resolve_markers: bool,
        random_ids: bool,
    ) -> PyResult<PyObject> {
        let mut sink = CopySink {
            tiles: Vec::new(),
            resources: Vec::new(),
            opts: ResourceCopyOptions {
                graph_publication_id: graph_publication_id.as_deref(),
                lifecycle_state_id: lifecycle_state_id.as_deref(),
                created_time: created_time.as_deref(),
            },
        };
        let info = RowSinkInfo {
            abi: abi_fingerprint(),
            ctx: &mut sink as *mut CopySink as *mut c_void,
            tile_fn: on_tile,
            resource_fn: on_resource,
        };

        // SAFETY: the capsule points at `info`, which outlives the call below,
        // the only use of the capsule; alizarin calls the sink only during it.
        let capsule: Bound<'_, PyAny> = unsafe {
            let capsule = pyo3::ffi::PyCapsule_New(
                &info as *const RowSinkInfo as *mut c_void,
                SINK_CAPSULE_NAME.as_ptr() as *const std::os::raw::c_char,
                None,
            );
            if capsule.is_null() {
                return Err(PyErr::fetch(py));
            }
            Bound::from_owned_ptr(py, capsule)
        };

        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("from_camel", from_camel)?;
        kwargs.set_item("strict", strict)?;
        kwargs.set_item("id_keys", id_keys)?;
        kwargs.set_item("resolve_markers", resolve_markers)?;
        kwargs.set_item("random_ids", random_ids)?;
        let batch = py
            .import_bound("alizarin")?
            .getattr("batch_trees_to_row_sink")?
            .call((trees_json, graph_id, capsule), Some(&kwargs))?;

        let tiles_copy = String::from_utf8(sink.tiles).expect("COPY output is always valid UTF-8");
        let resources_copy =
            String::from_utf8(sink.resources).expect("COPY output is always valid UTF-8");
        let output = PyDict::new_bound(py);
        output.set_item("tiles_copy", tiles_copy)?;
        output.set_item("resources_copy", resources_copy)?;
        output.set_item("errors", batch.get_item("errors")?)?;
        output.set_item("error_count", batch.get_item("error_count")?)?;
        Ok(output.into_any().unbind())
    }

    #[pymodule]
    fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(batch_trees_to_pg_copy, m)?)?;
        Ok(())
    }
}
