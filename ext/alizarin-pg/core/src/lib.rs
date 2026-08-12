//! PostgreSQL COPY format output for bulk-loading tiles and resource instances.
//!
//! Produces tab-delimited text suitable for `COPY ... FROM STDIN` — the fastest
//! bulk ingest path PostgreSQL offers. Avoids SQL string generation entirely (no
//! injection risk, no quoting headaches).
//!
//! The primary API writes to `impl io::Write` so callers can stream to a file,
//! pipe, or network socket without buffering the entire dataset in memory.
//! Convenience `*_to_copy` functions that return `String` are provided for
//! smaller datasets and tests.
//!
//! # Arches table schemas
//!
//! **`tiles`**: tileid, resourceinstanceid, parenttileid, tiledata (JSONB),
//! nodegroupid, sortorder, provisionaledits (JSONB).
//!
//! **`resource_instances`**: resourceinstanceid, graphid, graphpublicationid,
//! name (JSONB), descriptors (JSONB), legacyid, createdtime,
//! resource_instance_lifecycle_state_id, principaluser_id.

use alizarin_core::{StaticResource, StaticTile, get_current_language};
use std::io;

const TAB: &[u8] = b"\t";
const NL: &[u8] = b"\n";

/// Escape a string for PG COPY text format.
///
/// Backslash, tab, newline, and carriage return must be escaped.
fn copy_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

/// Options for resource instance COPY output.
pub struct ResourceCopyOptions<'a> {
    pub graph_publication_id: Option<&'a str>,
    pub lifecycle_state_id: Option<&'a str>,
    pub created_time: Option<&'a str>,
}

// ── Streaming (io::Write) API ───────────────────────────────────────────────

/// Write a single tile as one COPY row to `w`.
pub fn write_tile_row<W: io::Write>(w: &mut W, tile: &StaticTile) -> io::Result<()> {
    let tileid = tile.tileid.as_deref().unwrap_or("\\N");
    let parenttileid = tile
        .parenttile_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("\\N");

    let tiledata = serde_json::to_string(&tile.data).unwrap_or_default();

    let sortorder = tile
        .sortorder
        .map(|s| s.to_string())
        .unwrap_or_else(|| "0".to_string());

    let provisionaledits = tile
        .provisionaledits
        .as_ref()
        .map(|p| {
            let v = serde_json::Value::Array(p.clone());
            copy_escape(&v.to_string())
        })
        .unwrap_or_else(|| "\\N".to_string());

    w.write_all(tileid.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(tile.resourceinstance_id.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(parenttileid.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(copy_escape(&tiledata).as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(tile.nodegroup_id.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(sortorder.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(provisionaledits.as_bytes())?;
    w.write_all(NL)?;
    Ok(())
}

/// Write all tiles for a single resource to `w`.
pub fn write_resource_tiles<W: io::Write>(
    w: &mut W,
    resource: &StaticResource,
) -> io::Result<()> {
    if let Some(ref tiles) = resource.tiles {
        for tile in tiles {
            write_tile_row(w, tile)?;
        }
    }
    Ok(())
}

/// Write all tiles for a slice of resources to `w`.
pub fn write_all_tiles<W: io::Write>(
    w: &mut W,
    resources: &[StaticResource],
) -> io::Result<()> {
    for resource in resources {
        write_resource_tiles(w, resource)?;
    }
    Ok(())
}

/// Write a single resource instance as one COPY row to `w`.
pub fn write_resource_instance_row<W: io::Write>(
    w: &mut W,
    resource: &StaticResource,
    opts: &ResourceCopyOptions<'_>,
) -> io::Result<()> {
    let ri = &resource.resourceinstance;

    let lang = get_current_language();

    let name_json = match &ri.descriptors.name {
        Some(n) => copy_escape(
            &serde_json::json!({&lang: {"direction": "ltr", "value": n}}).to_string(),
        ),
        None => "\\N".to_string(),
    };

    let descriptors_json = if ri.descriptors.is_empty() {
        "\\N".to_string()
    } else {
        let mut desc = serde_json::Map::new();
        if let Some(ref n) = ri.descriptors.name {
            desc.insert(
                "name".to_string(),
                serde_json::json!({&lang: {"direction": "ltr", "value": n}}),
            );
        }
        if let Some(ref d) = ri.descriptors.description {
            desc.insert(
                "description".to_string(),
                serde_json::json!({&lang: {"direction": "ltr", "value": d}}),
            );
        }
        if let Some(ref mp) = ri.descriptors.map_popup {
            desc.insert(
                "map_popup".to_string(),
                serde_json::json!({&lang: {"direction": "ltr", "value": mp}}),
            );
        }
        copy_escape(&serde_json::Value::Object(desc).to_string())
    };

    let legacyid = ri
        .legacyid
        .as_deref()
        .map(|s| copy_escape(s))
        .unwrap_or_else(|| "\\N".to_string());

    let createdtime = opts
        .created_time
        .or(ri.createdtime.as_deref())
        .unwrap_or("\\N");

    let gpub_id = opts
        .graph_publication_id
        .or(ri.graph_publication_id.as_deref())
        .unwrap_or("\\N");

    let lc_state_id = opts.lifecycle_state_id.unwrap_or("\\N");

    let principal_user = ri
        .principaluser_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "\\N".to_string());

    w.write_all(ri.resourceinstanceid.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(ri.graph_id.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(gpub_id.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(name_json.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(descriptors_json.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(legacyid.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(createdtime.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(lc_state_id.as_bytes())?;
    w.write_all(TAB)?;
    w.write_all(principal_user.as_bytes())?;
    w.write_all(NL)?;
    Ok(())
}

/// Write all resource instance rows for a slice of resources to `w`.
pub fn write_all_resource_instances<W: io::Write>(
    w: &mut W,
    resources: &[StaticResource],
    opts: &ResourceCopyOptions<'_>,
) -> io::Result<()> {
    for resource in resources {
        write_resource_instance_row(w, resource, opts)?;
    }
    Ok(())
}

// ── Convenience (String) API ────────────────────────────────────────────────

/// Produce PG COPY text for the `tiles` table.
pub fn tiles_to_copy(resources: &[StaticResource]) -> String {
    let mut buf = Vec::new();
    write_all_tiles(&mut buf, resources).expect("writing to Vec never fails");
    String::from_utf8(buf).expect("COPY output is always valid UTF-8")
}

/// Produce PG COPY text for the `resource_instances` table.
pub fn resources_to_copy(
    resources: &[StaticResource],
    graph_publication_id: Option<&str>,
    lifecycle_state_id: Option<&str>,
    created_time: Option<&str>,
) -> String {
    let opts = ResourceCopyOptions {
        graph_publication_id,
        lifecycle_state_id,
        created_time,
    };
    let mut buf = Vec::new();
    write_all_resource_instances(&mut buf, resources, &opts)
        .expect("writing to Vec never fails");
    String::from_utf8(buf).expect("COPY output is always valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use alizarin_core::{StaticResourceDescriptors, StaticResourceMetadata};
    use std::collections::HashMap;

    fn make_tile(tileid: &str, resource_id: &str, nodegroup: &str) -> StaticTile {
        let mut data = HashMap::new();
        data.insert("node-1".to_string(), serde_json::json!("test value"));
        StaticTile {
            tileid: Some(tileid.to_string()),
            resourceinstance_id: resource_id.to_string(),
            parenttile_id: None,
            nodegroup_id: nodegroup.to_string(),
            data,
            sortorder: Some(0),
            provisionaledits: None,
        }
    }

    fn make_resource(id: &str, graph_id: &str, tiles: Vec<StaticTile>) -> StaticResource {
        StaticResource {
            resourceinstance: StaticResourceMetadata {
                resourceinstanceid: id.to_string(),
                graph_id: graph_id.to_string(),
                name: "Test Resource".to_string(),
                descriptors: StaticResourceDescriptors {
                    name: Some("Test Resource".to_string()),
                    description: None,
                    map_popup: None,
                    slug: None,
                },
                publication_id: None,
                principaluser_id: None,
                legacyid: None,
                graph_publication_id: None,
                createdtime: None,
                lastmodified: None,
            },
            tiles: Some(tiles),
            metadata: HashMap::new(),
            cache: None,
            scopes: None,
            tiles_loaded: Some(true),
        }
    }

    #[test]
    fn test_copy_escape() {
        assert_eq!(copy_escape("hello"), "hello");
        assert_eq!(copy_escape("a\tb"), "a\\tb");
        assert_eq!(copy_escape("a\\b"), "a\\\\b");
        assert_eq!(copy_escape("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_tiles_to_copy_basic() {
        let tile = make_tile("tile-1", "res-1", "ng-1");
        let resource = make_resource("res-1", "graph-1", vec![tile]);
        let copy = tiles_to_copy(&[resource]);

        let lines: Vec<&str> = copy.trim().lines().collect();
        assert_eq!(lines.len(), 1);

        let cols: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(cols.len(), 7);
        assert_eq!(cols[0], "tile-1");
        assert_eq!(cols[1], "res-1");
        assert_eq!(cols[2], "\\N"); // parenttileid
        assert_eq!(cols[4], "ng-1");
        assert_eq!(cols[5], "0"); // sortorder
        assert_eq!(cols[6], "\\N"); // provisionaledits
    }

    #[test]
    fn test_tiles_to_copy_jsonb_escaped() {
        let mut tile = make_tile("tile-1", "res-1", "ng-1");
        tile.data.insert(
            "node-2".to_string(),
            serde_json::json!({"key": "value\twith\ttabs"}),
        );
        let resource = make_resource("res-1", "graph-1", vec![tile]);
        let copy = tiles_to_copy(&[resource]);

        let lines: Vec<&str> = copy.trim().lines().collect();
        assert_eq!(lines.len(), 1);
        let cols: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(cols.len(), 7);
        assert!(cols[3].contains("\\t"));
    }

    #[test]
    fn test_resources_to_copy_basic() {
        let resource = make_resource("res-1", "graph-1", vec![]);
        let copy = resources_to_copy(&[resource], None, None, Some("2026-01-01T00:00:00Z"));

        let lines: Vec<&str> = copy.trim().lines().collect();
        assert_eq!(lines.len(), 1);

        let cols: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(cols[0], "res-1");
        assert_eq!(cols[1], "graph-1");
        assert_eq!(cols[2], "\\N"); // graphpublicationid
        assert!(cols[3].contains("Test Resource")); // name
        assert!(cols[4].contains("Test Resource")); // descriptors
        assert_eq!(cols[5], "\\N"); // legacyid
        assert_eq!(cols[6], "2026-01-01T00:00:00Z"); // createdtime
    }

    #[test]
    fn test_resources_to_copy_with_params() {
        let resource = make_resource("res-1", "graph-1", vec![]);
        let copy = resources_to_copy(
            &[resource],
            Some("pub-123"),
            Some("lc-state-1"),
            Some("2026-01-01T00:00:00Z"),
        );

        let cols: Vec<&str> = copy.trim().split('\t').collect();
        assert_eq!(cols[2], "pub-123");
        assert_eq!(cols[7], "lc-state-1");
    }

    #[test]
    fn test_streaming_matches_convenience() {
        let tile = make_tile("tile-1", "res-1", "ng-1");
        let resource = make_resource("res-1", "graph-1", vec![tile]);

        let convenience = tiles_to_copy(&[resource.clone()]);

        let mut streamed = Vec::new();
        write_all_tiles(&mut streamed, &[resource]).unwrap();
        let streamed = String::from_utf8(streamed).unwrap();

        assert_eq!(convenience, streamed);
    }

    #[test]
    fn test_write_single_tile_row() {
        let tile = make_tile("t-1", "r-1", "n-1");
        let mut buf = Vec::new();
        write_tile_row(&mut buf, &tile).unwrap();
        let line = String::from_utf8(buf).unwrap();
        assert!(line.starts_with("t-1\t"));
        assert!(line.ends_with('\n'));
    }
}
