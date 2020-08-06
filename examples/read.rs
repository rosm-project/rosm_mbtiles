use rosm_geo::mercator::TileId;

use rosm_mbtiles::read::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mbtiles = std::env::args()
        .nth(1)
        .expect("Expected path to an MBTiles database as first argument");

    let conn = rusqlite::Connection::open(&mbtiles)?;

    let metadata = read_metadata(&conn)?;

    println!("{:?}", metadata);

    let tile_id = TileId::new(1, 2, 3)?;

    if let Ok(Some(tile_data)) = read_tile(&conn, tile_id.into()) {
        println!("Found tile {:?}, data length: {}", tile_id, tile_data.len());
    } else {
        println!("No tile found with id {:?}", tile_id)
    }

    Ok(())
}
