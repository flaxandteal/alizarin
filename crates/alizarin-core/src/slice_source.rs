//! A slice of resources for Aggregate functions to visit.
//!
//! An Aggregate function (`graph::functions`) reads many resources and writes
//! tiles on one. It visits each member once and never needs random access, so a
//! slice is a visitor rather than a [`TileSource`](crate::tile_source::TileSource).
//! That lets one implementation cover a loaded registry and a business-data file
//! streamed element by element, so a large layer never has to be held in memory.

use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::{StaticGraph, StaticResource, StaticResourceRegistry, StaticTile};

/// One member of a slice, as the visitor sees it.
#[derive(Debug, Clone, Copy)]
pub struct SliceMember<'a> {
    pub graph_id: &'a str,
    pub resource_id: &'a str,
    pub tiles: &'a [StaticTile],
}

/// The resources an Aggregate function folds over, visited once each.
///
/// The visitor returns `Err` to stop the visit; `visit` passes that error on.
/// Visit order is unspecified, so an Aggregate must not depend on it (folds are
/// order-independent by contract).
pub trait SliceSource {
    fn visit(&self, f: &mut dyn FnMut(SliceMember<'_>) -> Result<(), String>)
        -> Result<(), String>;
}

fn member(resource: &StaticResource) -> SliceMember<'_> {
    SliceMember {
        graph_id: &resource.resourceinstance.graph_id,
        resource_id: &resource.resourceinstance.resourceinstanceid,
        tiles: resource.tiles.as_deref().unwrap_or(&[]),
    }
}

/// Every full resource in the registry. Summary-only entries have no tiles and
/// are not members.
impl SliceSource for StaticResourceRegistry {
    fn visit(
        &self,
        f: &mut dyn FnMut(SliceMember<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        for (_, resource) in self.iter_full() {
            f(member(resource))?;
        }
        Ok(())
    }
}

/// Resources already in memory.
impl SliceSource for Vec<StaticResource> {
    fn visit(
        &self,
        f: &mut dyn FnMut(SliceMember<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        for resource in self {
            f(member(resource))?;
        }
        Ok(())
    }
}

/// A `{ business_data: { resources: [...] } }` file, streamed one resource at a
/// time. Each `visit` re-reads the file; nothing is kept between visits.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct BusinessDataFileSlice {
    path: std::path::PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl BusinessDataFileSlice {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SliceSource for BusinessDataFileSlice {
    fn visit(
        &self,
        f: &mut dyn FnMut(SliceMember<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        use crate::loader::{
            parse_business_data_resource_bytes, stream_business_data_resources, LoaderError,
        };
        let file =
            std::fs::File::open(&self.path).map_err(|e| format!("{}: {e}", self.path.display()))?;
        stream_business_data_resources(file, |_offset, bytes| {
            let resource = parse_business_data_resource_bytes(bytes)?;
            f(member(&resource)).map_err(LoaderError::Other)
        })
        .map_err(|e| format!("{}: {e}", self.path.display()))
    }
}

/// Where an Aggregate looks up the graphs of a slice's members, to resolve paths.
pub trait GraphSource {
    fn graph(&self, graph_id: &str) -> Option<Arc<StaticGraph>>;
}

/// The graphs registered in this process (`register_graph`).
#[derive(Debug, Clone, Copy, Default)]
pub struct RegisteredGraphs;

impl GraphSource for RegisteredGraphs {
    fn graph(&self, graph_id: &str) -> Option<Arc<StaticGraph>> {
        crate::get_graph(graph_id)
    }
}

/// Graphs supplied directly, by graph id.
impl GraphSource for HashMap<String, Arc<StaticGraph>> {
    fn graph(&self, graph_id: &str) -> Option<Arc<StaticGraph>> {
        self.get(graph_id).cloned()
    }
}
