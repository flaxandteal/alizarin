// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Flax & Teal Limited

//! Pluggable tile source trait for loading tiles from external backends.
//!
//! This trait allows compiled-in tile sources (e.g. Rós Madair index files)
//! to provide tiles directly in Rust, avoiding FFI overhead when the tile
//! source and alizarin are compiled into the same binary.
//!
//! Platform wrappers (WASM, Python) check for a registered `TileSource`
//! before falling back to their platform-specific callback mechanisms.

use crate::graph::StaticTile;
use std::fmt;

/// Error type for tile source operations.
#[derive(Debug, Clone)]
pub enum TileSourceError {
    /// The requested resource was not found in this tile source.
    /// Callers may fall through to alternative tile loading mechanisms.
    ResourceNotFound { resource_id: String },
    /// A hard error occurred during tile loading.
    /// Callers should propagate this rather than falling through.
    LoadError(String),
}

impl fmt::Display for TileSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileSourceError::ResourceNotFound { resource_id } => {
                write!(f, "Resource not found: {}", resource_id)
            }
            TileSourceError::LoadError(msg) => {
                write!(f, "Tile load error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TileSourceError {}

/// A synchronous source of tile data for resource instances.
///
/// Implementations provide tiles from various backends (index files, databases,
/// in-memory caches) without the consumer needing to know the specifics.
///
/// This trait is intentionally synchronous. For async tile sources (e.g. JS
/// callbacks fetching over HTTP), platform wrappers use their own callback
/// mechanisms. `TileSource` is the fast path for compiled-together Rust code.
pub trait TileSource: Send + Sync {
    /// Load tiles for a resource.
    ///
    /// # Parameters
    /// - `resource_id`: Bare resource instance UUID (not a full URI).
    ///   The implementation handles URI construction internally.
    /// - `nodegroup_id`: Optional filter hint. If `Some`, implementations
    ///   may return only tiles for that nodegroup. If `None`, all tiles
    ///   for the resource should be returned.
    fn load_tiles(
        &self,
        resource_id: &str,
        nodegroup_id: Option<&str>,
    ) -> Result<Vec<StaticTile>, TileSourceError>;
}
