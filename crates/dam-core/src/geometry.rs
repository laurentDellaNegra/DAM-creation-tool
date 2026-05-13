use crate::{Coordinate, PolygonNode};
use geo::Buffer;

const EARTH_RADIUS_NM: f64 = 3440.065;
const EARTH_RADIUS_M: f64 = 6_371_008.8;
const METERS_PER_NM: f64 = 1_852.0;

#[derive(Debug, Clone, PartialEq)]
pub enum GeometryError {
    MissingArcAnchor { index: usize },
}

pub fn bearing_deg(from: Coordinate, to: Coordinate) -> f64 {
    let lat1 = from.lat.to_radians();
    let lat2 = to.lat.to_radians();
    let dlon = (to.lon - from.lon).to_radians();
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    y.atan2(x).to_degrees()
}

pub fn destination_point(origin: Coordinate, bearing_deg: f64, distance_nm: f64) -> Coordinate {
    let angular = distance_nm / EARTH_RADIUS_NM;
    let bearing = bearing_deg.to_radians();
    let lat1 = origin.lat.to_radians();
    let lon1 = origin.lon.to_radians();
    let lat2 = (lat1.sin() * angular.cos() + lat1.cos() * angular.sin() * bearing.cos()).asin();
    let lon2 = lon1
        + (bearing.sin() * angular.sin() * lat1.cos())
            .atan2(angular.cos() - lat1.sin() * lat2.sin());
    Coordinate {
        lon: lon2.to_degrees(),
        lat: lat2.to_degrees(),
    }
}

pub fn distance_nm(left: Coordinate, right: Coordinate) -> f64 {
    let lat1 = left.lat.to_radians();
    let lat2 = right.lat.to_radians();
    let dlat = (right.lat - left.lat).to_radians();
    let dlon = (right.lon - left.lon).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    EARTH_RADIUS_NM * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

pub fn strip_corners(point1: Coordinate, point2: Coordinate, width_nm: f64) -> Vec<Coordinate> {
    let bearing = bearing_deg(point1, point2);
    let half_width = width_nm / 2.0;
    vec![
        destination_point(point1, bearing - 90.0, half_width),
        destination_point(point2, bearing - 90.0, half_width),
        destination_point(point2, bearing + 90.0, half_width),
        destination_point(point1, bearing + 90.0, half_width),
    ]
}

pub fn polygon_arc_angles(
    nodes: &[PolygonNode],
    arc_index: usize,
) -> Result<(f64, f64), GeometryError> {
    let Some(PolygonNode::Arc { center, .. }) = nodes.get(arc_index) else {
        return Err(GeometryError::MissingArcAnchor { index: arc_index });
    };
    let n = nodes.len();
    if n < 3 {
        return Err(GeometryError::MissingArcAnchor { index: arc_index });
    }
    let prev = if arc_index == 0 { n - 1 } else { arc_index - 1 };
    let next = (arc_index + 1) % n;
    let (
        PolygonNode::Point {
            coordinate: prev_anchor,
        },
        PolygonNode::Point {
            coordinate: next_anchor,
        },
    ) = (&nodes[prev], &nodes[next])
    else {
        return Err(GeometryError::MissingArcAnchor { index: arc_index });
    };

    Ok((
        bearing_deg(*center, *prev_anchor),
        bearing_deg(*center, *next_anchor),
    ))
}

pub fn is_full_circle(first_angle_deg: f64, last_angle_deg: f64) -> bool {
    (first_angle_deg - 0.0).abs() < f64::EPSILON && (last_angle_deg - 360.0).abs() < f64::EPSILON
}

pub fn angle_span(first_angle_deg: f64, last_angle_deg: f64) -> f64 {
    if is_full_circle(first_angle_deg, last_angle_deg) {
        360.0
    } else if last_angle_deg >= first_angle_deg {
        last_angle_deg - first_angle_deg
    } else {
        -(first_angle_deg - last_angle_deg)
    }
}

pub fn pie_circle_points(
    center: Coordinate,
    radius_nm: f64,
    first_angle_deg: f64,
    last_angle_deg: f64,
) -> Vec<Coordinate> {
    let full_circle = is_full_circle(first_angle_deg, last_angle_deg);
    let span = angle_span(first_angle_deg, last_angle_deg);
    let segments = if full_circle { 96 } else { 48 };
    let mut points = Vec::with_capacity(segments + 2);

    if !full_circle {
        points.push(center);
    }

    for index in 0..=segments {
        let fraction = index as f64 / segments as f64;
        let angle = first_angle_deg + span * fraction;
        points.push(destination_point(center, angle, radius_nm));
    }

    points
}

pub fn geometry_center(points: &[Coordinate]) -> Option<Coordinate> {
    if points.is_empty() {
        return None;
    }
    let (lon_sum, lat_sum) = points.iter().fold((0.0, 0.0), |(lon, lat), point| {
        (lon + point.lon, lat + point.lat)
    });
    Some(Coordinate {
        lon: lon_sum / points.len() as f64,
        lat: lat_sum / points.len() as f64,
    })
}

pub fn buffered_polygon_outlines(points: &[Coordinate], buffer_nm: f64) -> Vec<Vec<Coordinate>> {
    if points.len() < 3 || buffer_nm <= 0.0 {
        return Vec::new();
    }

    let Some(projection) = LocalProjection::from_points(points) else {
        return Vec::new();
    };
    let mut exterior: Vec<geo::Coord> = points
        .iter()
        .map(|point| {
            let (x, y) = projection.project(*point);
            geo::Coord { x, y }
        })
        .collect();
    if exterior.first() != exterior.last()
        && let Some(first) = exterior.first().copied()
    {
        exterior.push(first);
    }

    let polygon = geo::Polygon::new(geo::LineString::from(exterior), Vec::new());
    polygon
        .buffer(buffer_nm * METERS_PER_NM)
        .0
        .iter()
        .filter_map(|polygon| {
            let mut coordinates: Vec<Coordinate> = polygon
                .exterior()
                .0
                .iter()
                .map(|coord| projection.unproject(coord.x, coord.y))
                .collect();
            if coordinates.first() == coordinates.last() {
                coordinates.pop();
            }
            (coordinates.len() >= 3).then_some(coordinates)
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct LocalProjection {
    origin: Coordinate,
    origin_lat_rad: f64,
    cos_origin_lat: f64,
}

impl LocalProjection {
    fn from_points(points: &[Coordinate]) -> Option<Self> {
        let origin = geometry_center(points)?;
        if !origin.lon.is_finite() || !origin.lat.is_finite() {
            return None;
        }
        let origin_lat_rad = origin.lat.to_radians();
        let cos_origin_lat = origin_lat_rad.cos();
        if cos_origin_lat.abs() < 1e-6 {
            return None;
        }
        Some(Self {
            origin,
            origin_lat_rad,
            cos_origin_lat,
        })
    }

    fn project(self, coordinate: Coordinate) -> (f64, f64) {
        let x =
            (coordinate.lon - self.origin.lon).to_radians() * EARTH_RADIUS_M * self.cos_origin_lat;
        let y = (coordinate.lat.to_radians() - self.origin_lat_rad) * EARTH_RADIUS_M;
        (x, y)
    }

    fn unproject(self, x: f64, y: f64) -> Coordinate {
        Coordinate {
            lon: self.origin.lon + (x / (EARTH_RADIUS_M * self.cos_origin_lat)).to_degrees(),
            lat: (self.origin_lat_rad + y / EARTH_RADIUS_M).to_degrees(),
        }
    }
}
