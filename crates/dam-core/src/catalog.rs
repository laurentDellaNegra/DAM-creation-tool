use crate::{AltitudeCorrection, BufferFilter, LevelUnit};
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

static MAPS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../assets/maps");
static SWITZERLAND_BORDER_SOURCE: &str =
    include_str!("../../../assets/preview/switzerland-border.geojson");

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapCatalog {
    pub maps: Vec<StaticMap>,
    pub diagnostics: Vec<CatalogDiagnostic>,
}

impl MapCatalog {
    pub fn from_entries(entries: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut maps = Vec::new();
        let mut diagnostics = Vec::new();

        for (path, source) in entries {
            match parse_static_map(&path, &source) {
                Ok(map) => maps.push(map),
                Err(message) => diagnostics.push(CatalogDiagnostic { path, message }),
            }
        }

        maps.sort_by(|a, b| a.id.cmp(&b.id).then(a.name.cmp(&b.name)));

        Self { maps, diagnostics }
    }

    pub fn selected(&self, id: &str) -> Option<&StaticMap> {
        self.maps.iter().find(|map| map.id == id)
    }
}

pub fn bundled_catalog() -> MapCatalog {
    let entries = MAPS_DIR.files().filter_map(|file| {
        let path = file.path().to_string_lossy().to_string();
        if Path::new(&path).extension().and_then(|ext| ext.to_str()) != Some("geojson") {
            return None;
        }

        let source = file.contents_utf8()?.to_owned();
        Some((path, source))
    });

    MapCatalog::from_entries(entries)
}

pub fn switzerland_border_preview() -> PreviewGeometry {
    parse_preview_geometry(SWITZERLAND_BORDER_SOURCE).unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticMap {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub preview: PreviewGeometry,
    pub defaults: MapDefaults,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDefaults {
    pub label_coordinate: Option<Coordinate>,
    pub lower_level: Option<String>,
    pub lower_unit: Option<LevelUnit>,
    pub upper_level: Option<String>,
    pub upper_unit: Option<LevelUnit>,
    pub start_indication: Option<bool>,
    pub end_indication: Option<bool>,
    pub display_levels: Option<bool>,
    pub altitude_correction: Option<AltitudeCorrection>,
    pub upper_buffer: Option<BufferFilter>,
    pub lower_buffer: Option<BufferFilter>,
    pub text: Option<String>,
    pub stations: Vec<String>,
}

impl StaticMap {
    pub fn label(&self) -> String {
        format!("{} - {}", self.id, self.name)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PreviewGeometry {
    pub paths: Vec<PreviewPath>,
    pub bbox: Option<BoundingBox>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewPath {
    pub coordinates: Vec<Coordinate>,
    #[serde(rename = "borderColor")]
    pub border_color: Option<[u8; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub lon: f64,
    pub lat: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

impl BoundingBox {
    fn include(&mut self, coordinate: Coordinate) {
        self.min_lon = self.min_lon.min(coordinate.lon);
        self.min_lat = self.min_lat.min(coordinate.lat);
        self.max_lon = self.max_lon.max(coordinate.lon);
        self.max_lat = self.max_lat.max(coordinate.lat);
    }

    pub fn center(&self) -> Coordinate {
        Coordinate {
            lon: (self.min_lon + self.max_lon) / 2.0,
            lat: (self.min_lat + self.max_lat) / 2.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogDiagnostic {
    pub path: String,
    pub message: String,
}

fn parse_static_map(path: &str, source: &str) -> Result<StaticMap, String> {
    let value: Value =
        serde_json::from_str(source).map_err(|error| format!("invalid JSON: {error}"))?;

    if value.get("type").and_then(Value::as_str) != Some("FeatureCollection") {
        return Err("expected GeoJSON FeatureCollection".to_owned());
    }

    let id = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| "map id could not be derived from filename".to_owned())?
        .to_owned();

    let name = value
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| "top-level name is required".to_owned())?
        .to_owned();

    let description = value
        .get("description")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let preview = preview_geometry_from_value(&value);
    let defaults = extract_defaults(&value);

    Ok(StaticMap {
        id,
        name,
        description,
        preview,
        defaults,
    })
}

fn parse_preview_geometry(source: &str) -> Result<PreviewGeometry, String> {
    let value: Value =
        serde_json::from_str(source).map_err(|error| format!("invalid JSON: {error}"))?;

    if value.get("type").and_then(Value::as_str) != Some("FeatureCollection") {
        return Err("expected GeoJSON FeatureCollection".to_owned());
    }

    Ok(preview_geometry_from_value(&value))
}

fn preview_geometry_from_value(value: &Value) -> PreviewGeometry {
    let mut preview = PreviewGeometry::default();
    if let Some(features) = value.get("features").and_then(Value::as_array) {
        for feature in features {
            if let Some(geometry) = feature.get("geometry") {
                let border_color = feature
                    .get("properties")
                    .and_then(|p| p.get("borderColor"))
                    .and_then(Value::as_str)
                    .and_then(parse_hex_color);
                let before = preview.paths.len();
                extract_geometry_paths(geometry, &mut preview.paths);
                if let Some(rgb) = border_color {
                    for path in &mut preview.paths[before..] {
                        path.border_color = Some(rgb);
                    }
                }
            }
        }
    }
    preview.bbox = bounding_box(&preview.paths);
    preview
}

fn extract_geometry_paths(geometry: &Value, paths: &mut Vec<PreviewPath>) {
    let Some(kind) = geometry.get("type").and_then(Value::as_str) else {
        return;
    };
    let Some(coordinates) = geometry.get("coordinates") else {
        return;
    };

    match kind {
        "LineString" => push_line_string(coordinates, paths),
        "MultiLineString" | "Polygon" => push_nested_line_strings(coordinates, paths),
        "MultiPolygon" => {
            if let Some(polygons) = coordinates.as_array() {
                for polygon in polygons {
                    push_nested_line_strings(polygon, paths);
                }
            }
        }
        _ => {}
    }
}

fn push_nested_line_strings(value: &Value, paths: &mut Vec<PreviewPath>) {
    if let Some(lines) = value.as_array() {
        for line in lines {
            push_line_string(line, paths);
        }
    }
}

fn push_line_string(value: &Value, paths: &mut Vec<PreviewPath>) {
    let Some(points) = value.as_array() else {
        return;
    };

    let coordinates: Vec<Coordinate> = points
        .iter()
        .filter_map(|point| {
            let tuple = point.as_array()?;
            let lon = tuple.first()?.as_f64()?;
            let lat = tuple.get(1)?.as_f64()?;
            Some(Coordinate { lon, lat })
        })
        .collect();

    if coordinates.len() >= 2 {
        paths.push(PreviewPath {
            coordinates,
            border_color: None,
        });
    }
}

fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

fn extract_defaults(value: &Value) -> MapDefaults {
    let features = match value.get("features").and_then(Value::as_array) {
        Some(f) => f,
        None => return MapDefaults::default(),
    };

    let first_point = features.iter().find(|f| {
        f.get("geometry")
            .and_then(|g| g.get("type"))
            .and_then(Value::as_str)
            == Some("Point")
    });

    let Some(point_feature) = first_point else {
        return MapDefaults::default();
    };

    let label_coordinate = point_feature
        .get("geometry")
        .and_then(|g| g.get("coordinates"))
        .and_then(Value::as_array)
        .and_then(|coords| {
            let lon = coords.first()?.as_f64()?;
            let lat = coords.get(1)?.as_f64()?;
            Some(Coordinate { lon, lat })
        });

    let remarks = point_feature
        .get("properties")
        .and_then(|p| p.get("remarks"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let mut defaults = parse_remarks(remarks);
    defaults.label_coordinate = label_coordinate;
    defaults
}

const KNOWN_STATIONS: &[&str] = &[
    "APP", "ARF", "BRN", "BUO", "DUB", "EMM", "FIC", "LAC", "LUG", "MEZ", "SIO", "STG", "TWR",
    "UAC",
];

fn parse_remarks(remarks: &str) -> MapDefaults {
    let mut defaults = MapDefaults::default();

    let Some(body) = remarks.strip_prefix("LEVEL/") else {
        return defaults;
    };

    let tokens: Vec<&str> = body.split('/').collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];

        if let Some(val) = token.strip_prefix("ll=") {
            defaults.lower_level = Some(val.to_owned());
            defaults.lower_unit = Some(LevelUnit::FlightLevel);
            if tokens.get(i + 1) == Some(&"ft") {
                defaults.lower_unit = Some(LevelUnit::Feet);
                i += 1;
            }
        } else if let Some(val) = token.strip_prefix("ul=") {
            defaults.upper_level = Some(val.to_owned());
            defaults.upper_unit = Some(LevelUnit::FlightLevel);
            if tokens.get(i + 1) == Some(&"ft") {
                defaults.upper_unit = Some(LevelUnit::Feet);
                i += 1;
            }
        } else if let Some(val) = token.strip_prefix("TEXT=") {
            defaults.text = Some(val.to_owned());
        } else {
            match token {
                "ft" => {}
                "lft" => defaults.lower_unit = Some(LevelUnit::Feet),
                "uft" => defaults.upper_unit = Some(LevelUnit::Feet),
                "bt" => defaults.start_indication = Some(true),
                "et" => defaults.end_indication = Some(true),
                "dl" => defaults.display_levels = Some(true),
                "flc" => defaults.altitude_correction = Some(AltitudeCorrection::FlCorr),
                "qnh" => defaults.altitude_correction = Some(AltitudeCorrection::QnhCorr),
                "lnbu" => defaults.lower_buffer = Some(BufferFilter::NoBuffer),
                "unbu" => defaults.upper_buffer = Some(BufferFilter::NoBuffer),
                "uhbu" => defaults.upper_buffer = Some(BufferFilter::Half),
                "lhbu" => defaults.lower_buffer = Some(BufferFilter::Half),
                "restricted" | "danger" | "glider" | "tra" | "parachute" | "other" => {}
                station if KNOWN_STATIONS.contains(&station) => {
                    defaults.stations.push(station.to_owned());
                }
                _ => {}
            }
        }
        i += 1;
    }

    defaults
}

fn bounding_box(paths: &[PreviewPath]) -> Option<BoundingBox> {
    let first = paths
        .iter()
        .flat_map(|p| p.coordinates.iter())
        .copied()
        .next()?;
    let mut bbox = BoundingBox {
        min_lon: first.lon,
        min_lat: first.lat,
        max_lon: first.lon,
        max_lat: first.lat,
    };

    for coordinate in paths.iter().flat_map(|p| p.coordinates.iter()).copied() {
        bbox.include(coordinate);
    }

    Some(bbox)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_static_map_metadata_from_filename_and_geojson_name() {
        let source = r##"{
          "type": "FeatureCollection",
          "name": "HAUT VALAIS",
          "description": "MAP, WARNINGS, TRA, 50714 - HAUT VALAIS",
          "features": [{
            "type": "Feature",
            "geometry": {
              "type": "LineString",
              "coordinates": [[7.8, 46.4], [8.0, 46.2]]
            },
            "properties": {"color": "#ffffff", "borderColor": "#2d5f5f"}
          }]
        }"##;

        let catalog = MapCatalog::from_entries([("50714.geojson".to_owned(), source.to_owned())]);

        assert!(catalog.diagnostics.is_empty());
        assert_eq!(catalog.maps[0].id, "50714");
        assert_eq!(catalog.maps[0].name, "HAUT VALAIS");
        assert_eq!(catalog.maps[0].preview.paths.len(), 1);
        assert_eq!(
            catalog.maps[0].preview.paths[0].border_color,
            Some([0x2d, 0x5f, 0x5f])
        );
        assert!(catalog.maps[0].preview.bbox.is_some());
    }

    #[test]
    fn loads_switzerland_border_preview() {
        let preview = switzerland_border_preview();

        assert_eq!(preview.paths.len(), 1);
        assert!(preview.bbox.is_some());
    }

    #[test]
    fn parses_remarks_with_feet_levels_and_all_flags() {
        let defaults = parse_remarks(
            "LEVEL/ll=13000/ft/ul=17000/ft/other/dl/lft/flc/lnbu/APP/ARF/BUO/DUB/EMM/FIC/LAC/LUG/STG/TWR/UAC/TEXT=SION BUFFER ZONE",
        );

        assert_eq!(defaults.lower_level.as_deref(), Some("13000"));
        assert_eq!(defaults.lower_unit, Some(LevelUnit::Feet));
        assert_eq!(defaults.upper_level.as_deref(), Some("17000"));
        assert_eq!(defaults.upper_unit, Some(LevelUnit::Feet));
        assert_eq!(defaults.display_levels, Some(true));
        assert_eq!(
            defaults.altitude_correction,
            Some(AltitudeCorrection::FlCorr)
        );
        assert_eq!(defaults.lower_buffer, Some(BufferFilter::NoBuffer));
        assert_eq!(defaults.text.as_deref(), Some("SION BUFFER ZONE"));
        assert_eq!(defaults.stations.len(), 11);
        assert!(defaults.stations.contains(&"APP".to_owned()));
        assert!(defaults.stations.contains(&"UAC".to_owned()));
    }

    #[test]
    fn parses_remarks_with_fl_levels() {
        let defaults = parse_remarks(
            "LEVEL/ll=3000/ft/ul=3500/ft/bt/et/glider/uhbu/APP/FIC/TWR/TEXT=LSR72 Bohlhof",
        );

        assert_eq!(defaults.lower_level.as_deref(), Some("3000"));
        assert_eq!(defaults.lower_unit, Some(LevelUnit::Feet));
        assert_eq!(defaults.upper_level.as_deref(), Some("3500"));
        assert_eq!(defaults.upper_unit, Some(LevelUnit::Feet));
        assert_eq!(defaults.start_indication, Some(true));
        assert_eq!(defaults.end_indication, Some(true));
        assert_eq!(defaults.upper_buffer, Some(BufferFilter::Half));
        assert_eq!(defaults.text.as_deref(), Some("LSR72 Bohlhof"));
        assert_eq!(defaults.stations, vec!["APP", "FIC", "TWR"]);
    }

    #[test]
    fn parses_remarks_with_fl_without_ft_suffix() {
        let defaults = parse_remarks("LEVEL/ll=100/ul=999/bt/et/tra/BRN/TEXT=TRA EUC25SP");

        assert_eq!(defaults.lower_level.as_deref(), Some("100"));
        assert_eq!(defaults.lower_unit, Some(LevelUnit::FlightLevel));
        assert_eq!(defaults.upper_level.as_deref(), Some("999"));
        assert_eq!(defaults.upper_unit, Some(LevelUnit::FlightLevel));
        assert_eq!(defaults.start_indication, Some(true));
        assert_eq!(defaults.end_indication, Some(true));
        assert_eq!(defaults.text.as_deref(), Some("TRA EUC25SP"));
        assert_eq!(defaults.stations, vec!["BRN"]);
    }

    #[test]
    fn returns_empty_defaults_for_non_level_remarks() {
        let defaults = parse_remarks("PPR");
        assert_eq!(defaults, MapDefaults::default());
    }

    #[test]
    fn extracts_label_coordinate_and_defaults_from_geojson() {
        let source = r##"{
          "type": "FeatureCollection",
          "name": "Test Map",
          "features": [{
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [8.384, 47.654]},
            "properties": {"remarks": "LEVEL/ll=000/ul=065/bt/et/dl/uhbu/ARF/FIC/TEXT=Test Box"}
          }, {
            "type": "Feature",
            "geometry": {"type": "Polygon", "coordinates": [[[8.0, 47.0], [8.5, 47.0], [8.5, 47.5], [8.0, 47.0]]]},
            "properties": {}
          }]
        }"##;

        let catalog = MapCatalog::from_entries([("99999.geojson".to_owned(), source.to_owned())]);
        let map = &catalog.maps[0];

        assert!(map.defaults.label_coordinate.is_some());
        let coord = map.defaults.label_coordinate.unwrap();
        assert!((coord.lon - 8.384).abs() < 1e-6);
        assert!((coord.lat - 47.654).abs() < 1e-6);
        assert_eq!(map.defaults.text.as_deref(), Some("Test Box"));
        assert_eq!(map.defaults.upper_buffer, Some(BufferFilter::Half));
        assert_eq!(map.defaults.display_levels, Some(true));
    }
}
