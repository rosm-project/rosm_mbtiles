use rosm_geo::mercator::TileId;

use rosm_mbtiles::common::*;
use rosm_mbtiles::write::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = rusqlite::Connection::open(&"example.mbtiles")?;

    let tr = conn.transaction()?;

    set_application_id(&tr)?;

    create_metadata_table(&tr)?;
    create_tiles_table(&tr)?;
    // Optionally: create_grid_tables(&tr)?;

    create_tile_index(&tr)?;

    let mvt_metadata = MvtMetadata {
        vector_layers: Vec::new(),
        tilestats: None,
    };

    let metadata = Metadata {
        name: "example_tileset".to_owned(),
        format: FileFormat::Pbf(mvt_metadata),
        ..Default::default()
    };

    write_metadata(&tr, metadata)?;

    let tile_id = TileId::new(1, 2, 3)?;
    let tile_data = Vec::new(); // Gzip-compressed MVT PBF
    write_tile(&tr, tile_id.into(), tile_data)?;

    tr.commit()?;

    Ok(())
}
