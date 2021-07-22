//! Functions for reading MBTiles databases.

use rosm_geo::coord::GeoCoord;
use rosm_geo::mercator::TmsTileId;
use rosm_geo::rect::GeoRect;

use rusqlite::params;

use std::convert::TryFrom;

use crate::common::{FileFormat, Metadata, MvtMetadata, Type};

/// Reads metadata from the given database.
pub fn read_metadata(conn: &rusqlite::Connection) -> Result<Metadata, Box<dyn std::error::Error>> {
    let mut select_metadata = conn.prepare_cached("SELECT name, value FROM metadata")?;
    let mut rows = select_metadata.query([])?;

    let mut metadata = Metadata::default();

    let mut zoom_range = (None, None);
    let mut format_str = String::new();
    let mut mvt_metadata_json = String::new();

    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let value: String = row.get(1)?;

        match name.as_str() {
            "name" => metadata.name = value,
            "format" => format_str = value,
            "bounds" => {
                let split: Vec<&str> = value.split(",").collect();
                if split.len() == 4 {
                    let bounds = (
                        split[0].parse::<f64>(),
                        split[1].parse::<f64>(),
                        split[2].parse::<f64>(),
                        split[3].parse::<f64>(),
                    );
                    if let (Ok(left), Ok(bottom), Ok(right), Ok(top)) = bounds {
                        let tl_br = (GeoCoord::from_degrees(left, top), GeoCoord::from_degrees(right, bottom));
                        if let (Ok(tl), Ok(br)) = tl_br {
                            if let Ok(bbox) = GeoRect::new(tl, br) {
                                metadata.bounds = Some(bbox);
                            }
                        }
                    }
                }
            }
            "center" => {
                let split: Vec<&str> = value.split(",").collect();
                if split.len() == 3 {
                    let center = (
                        split[0].parse::<f64>(),
                        split[1].parse::<f64>(),
                        split[2].parse::<u32>(),
                    );
                    if let (Ok(lon), Ok(lat), Ok(zoom_level)) = center {
                        if let Ok(coord) = GeoCoord::from_degrees(lon, lat) {
                            metadata.center = Some((coord, zoom_level));
                        }
                    }
                }
            }
            "minzoom" => {
                if let Ok(minzoom) = value.parse::<u32>() {
                    zoom_range.0 = Some(minzoom);
                }
            }
            "maxzoom" => {
                if let Ok(maxzoom) = value.parse::<u32>() {
                    zoom_range.1 = Some(maxzoom);
                }
            }
            "attribution" => metadata.attribution = Some(value),
            "description" => metadata.description = Some(value),
            "type" => {
                if let Ok(r#type) = Type::try_from(value.as_str()) {
                    metadata.r#type = Some(r#type);
                }
            }
            "version" => {
                if let Ok(version) = value.parse::<u32>() {
                    metadata.version = Some(version);
                }
            }
            "json" => mvt_metadata_json = value,
            unknown_key => {
                metadata.custom.insert(unknown_key.to_owned(), value);
            }
        }
    }

    // TODO: error on empty format_str

    metadata.format = match format_str.as_str() {
        "pbf" => {
            let mvt_metadata = serde_json::from_str::<MvtMetadata>(&mvt_metadata_json)?;
            FileFormat::Pbf(mvt_metadata)
        }
        "jpg" => FileFormat::Jpg,
        "png" => FileFormat::Png,
        "webp" => FileFormat::Webp,
        ietf_type => FileFormat::Other(ietf_type.to_owned()),
    };

    if let (Some(minzoom), Some(maxzoom)) = zoom_range {
        metadata.zoom_range = Some(minzoom..=maxzoom);
    }

    Ok(metadata)
}

/// Reads the given tile from the database.
///
/// If the tile is not found, `None` is returned.
pub fn read_tile(conn: &rusqlite::Connection, tile_id: TmsTileId) -> rusqlite::Result<Option<Vec<u8>>> {
    let mut select_tile = conn
        .prepare_cached("SELECT tile_data FROM tiles WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3")?;
    let mut rows = select_tile.query(params![tile_id.z(), tile_id.x(), tile_id.y()])?;

    if let Some(row) = rows.next()? {
        let tile_data: Vec<u8> = row.get(0)?;
        Ok(Some(tile_data))
    } else {
        Ok(None)
    }
}

/// Reads the given grid from the database.
///
/// If the grid is not found, `None` is returned.
pub fn read_grid(conn: &rusqlite::Connection, tile_id: TmsTileId) -> rusqlite::Result<Option<Vec<u8>>> {
    let mut select_grid =
        conn.prepare_cached("SELECT grid FROM grids WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3")?;
    let mut rows = select_grid.query(params![tile_id.z(), tile_id.x(), tile_id.y()])?;

    if let Some(row) = rows.next()? {
        let grid: Vec<u8> = row.get(0)?;
        Ok(Some(grid))
    } else {
        Ok(None)
    }
}

/// Reads the grid data for the given key from the database.
///
/// If the grid data is not found, `None` is returned.
pub fn read_grid_data(conn: &rusqlite::Connection, tile_id: TmsTileId, key: &str) -> rusqlite::Result<Option<String>> {
    let mut select_grid = conn.prepare_cached(
        "SELECT key_json FROM grid_data WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3 AND key = ?4",
    )?;
    let mut rows = select_grid.query(params![tile_id.z(), tile_id.x(), tile_id.y(), key])?;

    if let Some(row) = rows.next()? {
        let grid_data: String = row.get(0)?;
        Ok(Some(grid_data))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod mbtiles_read_test {
    use super::*;

    #[test]
    fn read_mvt_metadata() {
        let json = r#"{
            "vector_layers": [
                {
                    "id": "tl_2016_us_county",
                    "description": "Census counties",
                    "minzoom": 0,
                    "maxzoom": 5,
                    "fields": {
                        "ALAND": "Number",
                        "AWATER": "Number",
                        "GEOID": "String",
                        "MTFCC": "String",
                        "NAME": "String"
                    }
                }
            ]
        }"#;

        let mvt_json = serde_json::from_str::<MvtMetadata>(json);

        assert!(mvt_json.is_ok());
    }
}
