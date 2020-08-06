# rosm_mbtiles

A Rust library for reading and writing [MBTiles](https://github.com/mapbox/mbtiles-spec) 1.3 databases.

> MBTiles is a specification for storing tiled map data in SQLite databases for immediate usage and for transfer.

## Dependencies

- [rosm_geo](https://github.com/yzsolt/rosm_geo) for basic geographic types
- [rosm_geostats](https://github.com/yzsolt/rosm_geostats) for reading/writing embedded Mapbox geostats
- [rusqlite](https://github.com/rusqlite/rusqlite) for reading/writing MBTiles databases
- [serde_json](https://github.com/serde-rs/json) for reading/writing vector tileset metadata
