//! Functions for writing MBTiles databases.

use rosm_geo::mercator::TmsTileId;

use rusqlite::{params, Transaction};

use crate::common::{FileFormat, Metadata};

/// Creates the `metadata` table.
pub fn create_metadata_table(tr: &Transaction) -> rusqlite::Result<()> {
    tr.execute(
        "CREATE TABLE metadata (
            name TEXT,
            value TEXT
        )",
        [],
    )?;
    Ok(())
}

/// Creates the `tiles` table.
pub fn create_tiles_table(tr: &Transaction) -> rusqlite::Result<()> {
    tr.execute(
        "CREATE TABLE tiles (
            zoom_level INTEGER,
            tile_column INTEGER,
            tile_row INTEGER,
            tile_data BLOB
        )",
        [],
    )?;
    Ok(())
}

/// Creates the optional `grids` and `grid_data` tables.
pub fn create_grid_tables(tr: &Transaction) -> rusqlite::Result<()> {
    tr.execute(
        "CREATE TABLE grids (
            zoom_level INTEGER,
            tile_column INTEGER,
            tile_row INTEGER,
            grid BLOB
        )",
        [],
    )?;
    tr.execute(
        "CREATE TABLE grid_data (
            zoom_level INTEGER,
            tile_column INTEGER,
            tile_row INTEGER,
            key_name TEXT,
            key_json TEXT
        )",
        [],
    )?;
    Ok(())
}

/// Creates the optional `tile_index` index for fast tile data lookup.
pub fn create_tile_index(tr: &Transaction) -> rusqlite::Result<()> {
    tr.execute(
        "CREATE UNIQUE INDEX tile_index ON tiles (
            zoom_level, 
            tile_column, 
            tile_row
        )",
        [],
    )?;
    Ok(())
}

/// Sets the officially assigned MBTiles magic number as application ID for the database.
pub fn set_application_id(tr: &Transaction) -> rusqlite::Result<()> {
    const MBTILES_ID: i32 = 0x4d504258;
    tr.execute(format!("PRAGMA application_id = {}", MBTILES_ID).as_str(), [])?;
    Ok(())
}

/// Writes the given metadata into the database.
pub fn write_metadata(tr: &Transaction, metadata: Metadata) -> Result<(), Box<dyn std::error::Error>> {
    let mut insert_metadata = tr.prepare_cached("INSERT INTO metadata (name, value) VALUES (?1, ?2)")?;

    insert_metadata.execute(params!["name", metadata.name])?;

    if let FileFormat::Pbf(mvt_metadata) = &metadata.format {
        insert_metadata.execute(params!["json", serde_json::to_string(&mvt_metadata)?])?;
    }

    let format_str: String = metadata.format.into();
    insert_metadata.execute(params!["format", format_str])?;

    if let Some(bounds) = &metadata.bounds {
        let tl = bounds.top_left();
        let br = bounds.bottom_right();
        insert_metadata.execute(params![
            "bounds",
            format!("{},{},{},{}", tl.lon(), br.lat(), br.lon(), tl.lat())
        ])?;
    }

    if let Some(center) = &metadata.center {
        let (coord, zoom) = center;
        insert_metadata.execute(params!["center", format!("{},{},{}", coord.lon(), coord.lat(), zoom)])?;
    }

    if let Some(zoom_range) = &metadata.zoom_range {
        insert_metadata.execute(params!["minzoom", zoom_range.start()])?;
        insert_metadata.execute(params!["maxzoom", zoom_range.end()])?;
    }

    if let Some(attribution) = &metadata.attribution {
        insert_metadata.execute(params!["attribution", attribution])?;
    }

    if let Some(description) = &metadata.description {
        insert_metadata.execute(params!["description", description])?;
    }

    if let Some(r#type) = metadata.r#type {
        let type_str: &'static str = r#type.into();
        insert_metadata.execute(params!["type", type_str])?;
    }

    if let Some(version) = &metadata.version {
        insert_metadata.execute(params!["version", version])?;
    }

    Ok(())
}

/// Writes the given tile data into the database.
///
/// **Note:** `tile_data` must be GZIP-compressed if Mapbox Vector Tile PBF is being stored.
pub fn write_tile(tr: &Transaction, tile_id: TmsTileId, tile_data: Vec<u8>) -> rusqlite::Result<()> {
    let mut insert_tile =
        tr.prepare_cached("INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)")?;
    insert_tile.execute(params![tile_id.z(), tile_id.x(), tile_id.y(), tile_data])?;
    Ok(())
}

/// Writes [UTFGrid](https://github.com/mapbox/utfgrid-spec) grid for the given tile.
///
/// **Note:** `grid` must be GZIP-compressed.
pub fn write_grid(tr: &Transaction, tile_id: TmsTileId, grid: Vec<u8>) -> rusqlite::Result<()> {
    let mut insert_grid =
        tr.prepare_cached("INSERT INTO grids (zoom_level, tile_column, tile_row, grid) VALUES (?1, ?2, ?3, ?4)")?;
    insert_grid.execute(params![tile_id.z(), tile_id.x(), tile_id.y(), grid])?;
    Ok(())
}

/// Writes [UTFGrid](https://github.com/mapbox/utfgrid-spec) data for the given tile and key.
pub fn write_grid_data(tr: &Transaction, tile_id: TmsTileId, key: &str, data: &str) -> rusqlite::Result<()> {
    let mut insert_grid_data = tr.prepare_cached(
        "INSERT INTO grid_data (zoom_level, tile_column, tile_row, key_name, key_json) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    insert_grid_data.execute(params![tile_id.z(), tile_id.x(), tile_id.y(), key, data])?;
    Ok(())
}

#[cfg(test)]
mod mbtiles_write_test {
    use std::collections::HashMap;

    use crate::common::{MvtMetadata, VectorLayer};

    #[test]
    fn write_vector_layer() {
        let layer = VectorLayer {
            id: "test".to_owned(),
            fields: HashMap::new(),
            description: String::new(),
            minzoom: None,
            maxzoom: None,
        };

        let json = serde_json::to_string(&layer).unwrap();

        assert_eq!(json, r#"{"id":"test","fields":{}}"#);
    }

    #[test]
    fn write_mvt_metadata() {
        let mvt_metadata = MvtMetadata {
            vector_layers: Vec::new(),
            tilestats: None,
        };

        let json = serde_json::to_string(&mvt_metadata).unwrap();

        assert_eq!(json, r#"{"vector_layers":[]}"#);
    }
}
