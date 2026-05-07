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
}

impl StaticMap {
    pub fn label(&self) -> String {
        format!("{} - {}", self.id, self.name)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PreviewGeometry {
    pub paths: Vec<Vec<Coordinate>>,
    pub bbox: Option<BoundingBox>,
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

    Ok(StaticMap {
        id,
        name,
        description,
        preview,
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
                extract_geometry_paths(geometry, &mut preview.paths);
            }
        }
    }
    preview.bbox = bounding_box(&preview.paths);
    preview
}

fn extract_geometry_paths(geometry: &Value, paths: &mut Vec<Vec<Coordinate>>) {
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

fn push_nested_line_strings(value: &Value, paths: &mut Vec<Vec<Coordinate>>) {
    if let Some(lines) = value.as_array() {
        for line in lines {
            push_line_string(line, paths);
        }
    }
}

fn push_line_string(value: &Value, paths: &mut Vec<Vec<Coordinate>>) {
    let Some(points) = value.as_array() else {
        return;
    };

    let path: Vec<Coordinate> = points
        .iter()
        .filter_map(|point| {
            let tuple = point.as_array()?;
            let lon = tuple.first()?.as_f64()?;
            let lat = tuple.get(1)?.as_f64()?;
            Some(Coordinate { lon, lat })
        })
        .collect();

    if path.len() >= 2 {
        paths.push(path);
    }
}

fn bounding_box(paths: &[Vec<Coordinate>]) -> Option<BoundingBox> {
    let first = paths.iter().flatten().copied().next()?;
    let mut bbox = BoundingBox {
        min_lon: first.lon,
        min_lat: first.lat,
        max_lon: first.lon,
        max_lat: first.lat,
    };

    for coordinate in paths.iter().flatten().copied() {
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
            "properties": {"color": "#2d5f5f"}
          }]
        }"##;

        let catalog = MapCatalog::from_entries([("50714.geojson".to_owned(), source.to_owned())]);

        assert!(catalog.diagnostics.is_empty());
        assert_eq!(catalog.maps[0].id, "50714");
        assert_eq!(catalog.maps[0].name, "HAUT VALAIS");
        assert_eq!(catalog.maps[0].preview.paths.len(), 1);
        assert!(catalog.maps[0].preview.bbox.is_some());
    }

    #[test]
    fn loads_switzerland_border_preview() {
        let preview = switzerland_border_preview();

        assert_eq!(preview.paths.len(), 1);
        assert!(preview.bbox.is_some());
    }
}
