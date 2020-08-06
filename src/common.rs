//! Common types for reading/writing MBTiles databases.

use rosm_geo::coord::GeoCoord;
use rosm_geo::rect::GeoRect;

use rosm_geostats::Tilestats;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::convert::{Into, TryFrom};
use std::ops::RangeInclusive;

/// File format of the tile data.
#[derive(Debug)]
pub enum FileFormat {
    /// GZIP-compressed [Mapbox Vector Tiles](https://github.com/mapbox/vector-tile-spec).
    Pbf(MvtMetadata),
    Jpg,
    Png,
    Webp,
    /// An [IETF media type](https://www.iana.org/assignments/media-types/media-types.xhtml) format.
    Other(String)
}

impl Into<String> for FileFormat {
    fn into(self) -> String {
        match self {
            FileFormat::Pbf(_) => "pbf".to_owned(),
            FileFormat::Jpg => "jpg".to_owned(),
            FileFormat::Png => "png".to_owned(),
            FileFormat::Webp => "webp".to_owned(),
            FileFormat::Other(ietf_type) => ietf_type
        }
    }
}

impl Default for FileFormat {
    fn default() -> Self {
        FileFormat::Other(String::new())
    }
}

#[derive(Debug)]
pub enum Type {
    Overlay,
    BaseLayer,
}

impl Into<&'static str> for Type {
    fn into(self) -> &'static str {
        match self {
            Type::Overlay => "overlay",
            Type::BaseLayer => "baselayer",
        }
    }
}

impl TryFrom<&str> for Type {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "overlay" => Ok(Type::Overlay),
            "baselayer" => Ok(Type::BaseLayer),
            _ => Err(()),
        }
    }
}

/// A key/value store for settings. 
#[derive(Debug, Default)]
pub struct Metadata {
    /// The human-readable name of the tileset.
    pub name: String,
    /// The file format of the tile data.
    pub format: FileFormat,
    /// The maximum extent of the rendered map area.
    pub bounds: Option<GeoRect>,
    /// The longitude, latitude, and zoom level of the default view of the map.
    pub center: Option<(GeoCoord, u32)>,
    /// The lowest and highest zoom levels for which the tileset provides data.
    pub zoom_range: Option<RangeInclusive<u32>>,
    /// An attribution string, which explains the sources of data and/or style for the map.
    pub attribution: Option<String>,
    /// A description of the tileset's content.
    pub description: Option<String>,
    pub r#type: Option<Type>,
    /// The version of the tileset. This refers to a revision of the tileset itself, not of the MBTiles specification.
    pub version: Option<u32>,
    /// Additional rows stored for other purposes.
    pub custom: HashMap<String, String>,
}

/// Additional metadata for [Mapbox Vector Tile](https://github.com/mapbox/vector-tile-spec) datasets.
#[derive(Debug, Serialize, Deserialize)]
pub struct MvtMetadata {
    /// Description of vector tile data layers.
    pub vector_layers: Vec<VectorLayer>,

    /// An object in the [mapbox-geostats](https://github.com/mapbox/mapbox-geostats) format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilestats: Option<Tilestats>,
}

/// Description for a specific layer of vector tile data.
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorLayer {
    /// The layer ID, which is referred to as the name of the layer in the [Mapbox Vector Tile spec](https://github.com/mapbox/vector-tile-spec).
    pub id: String,

    /// The names and types of attributes available in this layer.
    pub fields: HashMap<String, FieldType>,

    /// A human-readable description of the layer's contents.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    /// The lowest zoom level whose tiles this layer appears in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<u32>,

    /// The highest zoom level whose tiles this layer appears in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u32>,
}

/// Layer attribute type.
///
/// **Note:** attributes with mixed types should be serialized as string.
#[derive(Debug, Serialize, Deserialize)]
pub enum FieldType {
    Number, Boolean, String
}
