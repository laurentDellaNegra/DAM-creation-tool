use crate::{current_date_text, parse_date, parse_level, parse_time};
use chrono::NaiveDate;
use dam_core::{
    AltitudeCorrection, BufferFilter, Coordinate, DamCreation, DamMap, DateRange,
    DistributionSelection, LevelUnit, MAX_POLYGON_POINTS, ManualGeometry, ManualMap,
    ManualMapAttributes, ManualMapCategory, ManualMapRendering, MapCatalog, Period,
    SelectedStaticMap, StaticMap, TextInfo, TextNumberColor, TextNumberSize, ValidationIssue,
    Weekday, default_distribution,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMode {
    Predefined,
    Manual,
}

impl MapMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Predefined => "Predefined map",
            Self::Manual => "Manual map",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualGeometryType {
    Polygon,
    ParaSymbol,
    TextNumber,
    PieCircle,
    Strip,
}

impl ManualGeometryType {
    pub const ALL: [ManualGeometryType; 5] = [
        ManualGeometryType::Polygon,
        ManualGeometryType::ParaSymbol,
        ManualGeometryType::TextNumber,
        ManualGeometryType::PieCircle,
        ManualGeometryType::Strip,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Polygon => "Polygon",
            Self::ParaSymbol => "Para symbol",
            Self::TextNumber => "Text and number",
            Self::PieCircle => "Pie / circle",
            Self::Strip => "Strip",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieClickTarget {
    Center,
    Radius,
}

impl PieClickTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Center => "Center",
            Self::Radius => "Radius",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripClickTarget {
    Point1,
    Point2,
}

impl StripClickTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Point1 => "Point 1",
            Self::Point2 => "Point 2",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DamFormState {
    pub map_mode: MapMode,
    pub selected_map_id: Option<String>,
    pub map_search: String,
    pub manual: ManualMapState,
    pub start_date: String,
    pub end_date: String,
    pub active_weekdays: BTreeSet<Weekday>,
    pub possible_weekdays: BTreeSet<Weekday>,
    pub periods: Vec<PeriodRowState>,
    pub altitude_correction: AltitudeCorrection,
    pub upper_buffer: BufferFilter,
    pub lower_buffer: BufferFilter,
    pub distribution: DistributionSelection,
    pub text: String,
    pub display_text: bool,
    pub display_levels: bool,
}

#[derive(Debug, Clone)]
pub struct ManualMapState {
    pub name: String,
    pub geometry_type: ManualGeometryType,
    pub polygon: PolygonDraftState,
    pub para_symbol: PointDraftState,
    pub text_number: TextNumberDraftState,
    pub pie_circle: PieCircleDraftState,
    pub strip: StripDraftState,
    pub attributes: ManualAttributesState,
}

impl Default for ManualMapState {
    fn default() -> Self {
        Self {
            name: String::new(),
            geometry_type: ManualGeometryType::Polygon,
            polygon: PolygonDraftState::default(),
            para_symbol: PointDraftState::default(),
            text_number: TextNumberDraftState::default(),
            pie_circle: PieCircleDraftState::default(),
            strip: StripDraftState::default(),
            attributes: ManualAttributesState::default(),
        }
    }
}

impl ManualMapState {
    pub fn to_manual_map(&self, issues: &mut Vec<ValidationIssue>) -> ManualMap {
        let attributes = self.attributes.to_attributes(issues);
        let geometry = match self.geometry_type {
            ManualGeometryType::Polygon => ManualGeometry::Polygon {
                points: self.polygon.points(issues),
            },
            ManualGeometryType::ParaSymbol => ManualGeometry::ParaSymbol {
                point: self.para_symbol.coordinate("map.geometry.point", issues),
            },
            ManualGeometryType::TextNumber => ManualGeometry::TextNumber {
                point: self
                    .text_number
                    .point
                    .coordinate("map.geometry.point", issues),
                text: self.text_number.text.clone(),
                color: self.text_number.color,
                size: self.text_number.size,
            },
            ManualGeometryType::PieCircle => ManualGeometry::PieCircle {
                center: self
                    .pie_circle
                    .center
                    .coordinate("map.geometry.center", issues),
                radius_nm: parse_optional_positive_f64(
                    &self.pie_circle.radius_nm,
                    "map.geometry.radius_nm",
                    issues,
                ),
                first_angle_deg: parse_f64_or_default(
                    &self.pie_circle.first_angle_deg,
                    0.0,
                    "map.geometry.first_angle_deg",
                    issues,
                ),
                last_angle_deg: parse_f64_or_default(
                    &self.pie_circle.last_angle_deg,
                    360.0,
                    "map.geometry.last_angle_deg",
                    issues,
                ),
            },
            ManualGeometryType::Strip => ManualGeometry::Strip {
                point1: self.strip.point1.coordinate("map.geometry.point1", issues),
                point2: self.strip.point2.coordinate("map.geometry.point2", issues),
                width_nm: parse_optional_positive_f64(
                    &self.strip.width_nm,
                    "map.geometry.width_nm",
                    issues,
                ),
            },
        };

        ManualMap {
            name: self.name.clone(),
            geometry,
            attributes,
            label_position: self.label_position(),
        }
    }

    pub fn preview_manual_map(&self) -> ManualMap {
        let mut issues = Vec::new();
        self.to_manual_map(&mut issues)
    }

    pub fn label_position(&self) -> Option<Coordinate> {
        match self.geometry_type {
            ManualGeometryType::Polygon => self.polygon.label_position(),
            ManualGeometryType::ParaSymbol => self.para_symbol.silent_coordinate(),
            ManualGeometryType::TextNumber => self.text_number.point.silent_coordinate(),
            ManualGeometryType::PieCircle => self.pie_circle.label_position(),
            ManualGeometryType::Strip => self.strip.label_position(),
        }
    }

    pub fn apply_click(&mut self, coordinate: Coordinate) -> bool {
        match self.geometry_type {
            ManualGeometryType::Polygon => self.polygon.apply_click(coordinate),
            ManualGeometryType::ParaSymbol => {
                self.para_symbol.set_coordinate(coordinate);
                true
            }
            ManualGeometryType::TextNumber => {
                self.text_number.point = CoordinateFieldState::from_coordinate(coordinate);
                true
            }
            ManualGeometryType::PieCircle => {
                self.pie_circle.apply_click(coordinate);
                true
            }
            ManualGeometryType::Strip => {
                self.strip.apply_click(coordinate);
                true
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManualAttributesState {
    pub category: ManualMapCategory,
    pub rendering: ManualMapRendering,
    pub lateral_buffer_nm: String,
}

impl Default for ManualAttributesState {
    fn default() -> Self {
        let defaults = ManualMapAttributes::default();
        Self {
            category: defaults.category,
            rendering: defaults.rendering,
            lateral_buffer_nm: format_nm(defaults.lateral_buffer_nm),
        }
    }
}

impl ManualAttributesState {
    fn to_attributes(&self, issues: &mut Vec<ValidationIssue>) -> ManualMapAttributes {
        ManualMapAttributes {
            category: self.category,
            rendering: self.rendering,
            lateral_buffer_nm: parse_f64_or_default(
                &self.lateral_buffer_nm,
                0.0,
                "map.attributes.lateral_buffer_nm",
                issues,
            ),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PolygonDraftState {
    pub points: Vec<CoordinateFieldState>,
}

impl PolygonDraftState {
    fn points(&self, issues: &mut Vec<ValidationIssue>) -> Vec<Coordinate> {
        self.points
            .iter()
            .enumerate()
            .filter_map(|(index, point)| {
                point.coordinate(&format!("map.geometry.points[{index}]"), issues)
            })
            .collect()
    }

    fn label_position(&self) -> Option<Coordinate> {
        self.points
            .iter()
            .rev()
            .find_map(CoordinateFieldState::silent_coordinate)
    }

    fn apply_click(&mut self, coordinate: Coordinate) -> bool {
        if self.points.len() >= MAX_POLYGON_POINTS {
            return false;
        }
        self.points
            .push(CoordinateFieldState::from_coordinate(coordinate));
        true
    }
}

#[derive(Debug, Clone, Default)]
pub struct PointDraftState {
    pub point: CoordinateFieldState,
}

impl PointDraftState {
    fn coordinate(&self, field: &str, issues: &mut Vec<ValidationIssue>) -> Option<Coordinate> {
        self.point.coordinate(field, issues)
    }

    fn silent_coordinate(&self) -> Option<Coordinate> {
        self.point.silent_coordinate()
    }

    fn set_coordinate(&mut self, coordinate: Coordinate) {
        self.point = CoordinateFieldState::from_coordinate(coordinate);
    }
}

#[derive(Debug, Clone)]
pub struct TextNumberDraftState {
    pub point: CoordinateFieldState,
    pub text: String,
    pub color: TextNumberColor,
    pub size: TextNumberSize,
}

impl Default for TextNumberDraftState {
    fn default() -> Self {
        Self {
            point: CoordinateFieldState::default(),
            text: String::new(),
            color: TextNumberColor::Red,
            size: TextNumberSize::Medium,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PieCircleDraftState {
    pub center: CoordinateFieldState,
    pub radius_nm: String,
    pub first_angle_deg: String,
    pub last_angle_deg: String,
    pub click_target: PieClickTarget,
    radius_label_position: Option<Coordinate>,
}

impl Default for PieCircleDraftState {
    fn default() -> Self {
        Self {
            center: CoordinateFieldState::default(),
            radius_nm: String::new(),
            first_angle_deg: "0".to_owned(),
            last_angle_deg: "360".to_owned(),
            click_target: PieClickTarget::Center,
            radius_label_position: None,
        }
    }
}

impl PieCircleDraftState {
    fn label_position(&self) -> Option<Coordinate> {
        let center = self.center.silent_coordinate()?;
        if let Some(radius) = parse_silent_positive_f64(&self.radius_nm) {
            if let Some(label) = self.radius_label_position {
                return Some(label);
            }
            return Some(destination_point(center, 90.0, radius));
        }
        Some(center)
    }

    fn apply_click(&mut self, coordinate: Coordinate) {
        if self.center.silent_coordinate().is_none() {
            self.center = CoordinateFieldState::from_coordinate(coordinate);
            self.click_target = PieClickTarget::Radius;
            return;
        }

        if parse_silent_positive_f64(&self.radius_nm).is_none() {
            if let Some(center) = self.center.silent_coordinate() {
                self.radius_nm = format_nm(distance_nm(center, coordinate));
                self.radius_label_position = Some(coordinate);
            }
            return;
        }

        match self.click_target {
            PieClickTarget::Center => {
                self.center = CoordinateFieldState::from_coordinate(coordinate);
                self.radius_label_position = None;
            }
            PieClickTarget::Radius => {
                if let Some(center) = self.center.silent_coordinate() {
                    self.radius_nm = format_nm(distance_nm(center, coordinate));
                    self.radius_label_position = Some(coordinate);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StripDraftState {
    pub point1: CoordinateFieldState,
    pub point2: CoordinateFieldState,
    pub width_nm: String,
    pub click_target: StripClickTarget,
}

impl Default for StripDraftState {
    fn default() -> Self {
        Self {
            point1: CoordinateFieldState::default(),
            point2: CoordinateFieldState::default(),
            width_nm: String::new(),
            click_target: StripClickTarget::Point1,
        }
    }
}

impl StripDraftState {
    fn label_position(&self) -> Option<Coordinate> {
        self.point2
            .silent_coordinate()
            .or_else(|| self.point1.silent_coordinate())
    }

    fn apply_click(&mut self, coordinate: Coordinate) {
        if self.point1.silent_coordinate().is_none() {
            self.point1 = CoordinateFieldState::from_coordinate(coordinate);
            self.click_target = StripClickTarget::Point2;
            return;
        }
        if self.point2.silent_coordinate().is_none() {
            self.point2 = CoordinateFieldState::from_coordinate(coordinate);
            self.click_target = StripClickTarget::Point2;
            return;
        }

        match self.click_target {
            StripClickTarget::Point1 => {
                self.point1 = CoordinateFieldState::from_coordinate(coordinate);
            }
            StripClickTarget::Point2 => {
                self.point2 = CoordinateFieldState::from_coordinate(coordinate);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CoordinateFieldState {
    pub lat: String,
    pub lon: String,
}

impl CoordinateFieldState {
    fn from_coordinate(coordinate: Coordinate) -> Self {
        Self {
            lat: format_coordinate(coordinate.lat),
            lon: format_coordinate(coordinate.lon),
        }
    }

    fn coordinate(&self, field: &str, issues: &mut Vec<ValidationIssue>) -> Option<Coordinate> {
        let lat = self.lat.trim();
        let lon = self.lon.trim();
        if lat.is_empty() && lon.is_empty() {
            return None;
        }
        if lat.is_empty() || lon.is_empty() {
            issues.push(ValidationIssue {
                field: field.to_owned(),
                message: "Latitude and longitude are required together.".to_owned(),
            });
            return None;
        }

        let lat = parse_coordinate_number(lat, &format!("{field}.lat"), -90.0, 90.0, issues)?;
        let lon = parse_coordinate_number(lon, &format!("{field}.lon"), -180.0, 180.0, issues)?;
        Some(Coordinate { lon, lat })
    }

    fn silent_coordinate(&self) -> Option<Coordinate> {
        let lat = self.lat.trim().parse::<f64>().ok()?;
        let lon = self.lon.trim().parse::<f64>().ok()?;
        if !lat.is_finite()
            || !lon.is_finite()
            || !(-90.0..=90.0).contains(&lat)
            || !(-180.0..=180.0).contains(&lon)
        {
            return None;
        }
        Some(Coordinate { lon, lat })
    }
}

impl DamFormState {
    pub fn new(_catalog: &MapCatalog) -> Self {
        let today = current_date_text();
        let mut state = Self {
            map_mode: MapMode::Predefined,
            selected_map_id: None,
            map_search: String::new(),
            manual: ManualMapState::default(),
            start_date: today.clone(),
            end_date: today,
            active_weekdays: BTreeSet::new(),
            possible_weekdays: BTreeSet::new(),
            periods: vec![PeriodRowState::default()],
            altitude_correction: AltitudeCorrection::None,
            upper_buffer: BufferFilter::Default,
            lower_buffer: BufferFilter::Default,
            distribution: default_distribution(),
            text: String::new(),
            display_text: false,
            display_levels: true,
        };
        state.sync_weekdays_from_dates();
        state
    }

    pub fn selected_map<'a>(&self, catalog: &'a MapCatalog) -> Option<&'a StaticMap> {
        catalog.selected(self.selected_map_id.as_deref()?)
    }

    pub fn is_repetitive_range(&self) -> bool {
        match (
            parse_form_date(&self.start_date),
            parse_form_date(&self.end_date),
        ) {
            (Some(start), Some(end)) => start != end,
            _ => false,
        }
    }

    pub fn sync_weekdays_from_dates(&mut self) {
        let (Some(start), Some(end)) = (
            parse_form_date(&self.start_date),
            parse_form_date(&self.end_date),
        ) else {
            return;
        };

        let possible = dam_core::DateRange::new(start, end).possible_weekdays();
        if possible != self.possible_weekdays {
            let active = if self.active_weekdays.is_empty() {
                possible.clone()
            } else {
                self.active_weekdays
                    .intersection(&possible)
                    .copied()
                    .collect::<BTreeSet<_>>()
            };
            self.active_weekdays = if active.is_empty() {
                possible.clone()
            } else {
                active
            };
            self.possible_weekdays = possible;
        }
    }

    pub fn to_creation(&self, catalog: &MapCatalog) -> Result<DamCreation, Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        let map = match self.map_mode {
            MapMode::Predefined => {
                let selected_map = match self.selected_map(catalog) {
                    Some(map) => SelectedStaticMap {
                        id: map.id.clone(),
                        name: map.name.clone(),
                    },
                    None => {
                        issues.push(ValidationIssue {
                            field: "map".to_owned(),
                            message: "Select a valid static map.".to_owned(),
                        });
                        SelectedStaticMap {
                            id: String::new(),
                            name: String::new(),
                        }
                    }
                };
                DamMap::Predefined(selected_map)
            }
            MapMode::Manual => DamMap::Manual(self.manual.to_manual_map(&mut issues)),
        };

        let start = parse_date(&self.start_date, "date_range.start", &mut issues);
        let end = parse_date(&self.end_date, "date_range.end", &mut issues);

        let date_range = match (start, end) {
            (Some(start), Some(end)) => DateRange {
                start,
                end,
                active_weekdays: if start == end {
                    BTreeSet::new()
                } else {
                    self.active_weekdays.clone()
                },
            },
            _ => DateRange::new(
                NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            ),
        };

        let periods = self
            .periods
            .iter()
            .enumerate()
            .filter_map(|(index, row)| row.to_period(index, &mut issues))
            .collect();

        if !issues.is_empty() {
            return Err(issues);
        }

        Ok(DamCreation {
            map,
            date_range,
            periods,
            display_levels: self.display_levels,
            altitude_correction: self.altitude_correction,
            upper_buffer: self.upper_buffer,
            lower_buffer: self.lower_buffer,
            distribution: self.distribution.clone(),
            text: TextInfo {
                value: self.text.clone(),
                display: self.display_text,
            },
        })
    }
}

fn parse_form_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

fn parse_coordinate_number(
    value: &str,
    field: &str,
    min: f64,
    max: f64,
    issues: &mut Vec<ValidationIssue>,
) -> Option<f64> {
    let parsed = match value.parse::<f64>() {
        Ok(parsed) => parsed,
        Err(_) => {
            issues.push(ValidationIssue {
                field: field.to_owned(),
                message: "Coordinate must be a number.".to_owned(),
            });
            return None;
        }
    };
    if !parsed.is_finite() || parsed < min || parsed > max {
        issues.push(ValidationIssue {
            field: field.to_owned(),
            message: format!("Coordinate must be between {min} and {max}."),
        });
        return None;
    }
    Some(parsed)
}

fn parse_f64_or_default(
    value: &str,
    fallback: f64,
    field: &str,
    issues: &mut Vec<ValidationIssue>,
) -> f64 {
    let value = value.trim();
    if value.is_empty() {
        return fallback;
    }
    match value.parse::<f64>() {
        Ok(parsed) if parsed.is_finite() => parsed,
        _ => {
            issues.push(ValidationIssue {
                field: field.to_owned(),
                message: "Value must be a number.".to_owned(),
            });
            fallback
        }
    }
}

fn parse_optional_positive_f64(
    value: &str,
    field: &str,
    issues: &mut Vec<ValidationIssue>,
) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    match value.parse::<f64>() {
        Ok(parsed) if parsed.is_finite() => Some(parsed),
        _ => {
            issues.push(ValidationIssue {
                field: field.to_owned(),
                message: "Value must be a number.".to_owned(),
            });
            None
        }
    }
}

fn parse_silent_positive_f64(value: &str) -> Option<f64> {
    let parsed = value.trim().parse::<f64>().ok()?;
    if parsed.is_finite() && parsed > 0.0 {
        Some(parsed)
    } else {
        None
    }
}

fn format_coordinate(value: f64) -> String {
    format!("{value:.6}")
}

fn format_nm(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

fn distance_nm(left: Coordinate, right: Coordinate) -> f64 {
    const EARTH_RADIUS_NM: f64 = 3440.065;
    let lat1 = left.lat.to_radians();
    let lat2 = right.lat.to_radians();
    let dlat = (right.lat - left.lat).to_radians();
    let dlon = (right.lon - left.lon).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    EARTH_RADIUS_NM * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

fn destination_point(origin: Coordinate, bearing_deg: f64, distance_nm: f64) -> Coordinate {
    const EARTH_RADIUS_NM: f64 = 3440.065;
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

#[derive(Debug, Clone)]
pub struct PeriodRowState {
    pub start_indication: bool,
    pub start_time: String,
    pub end_indication: bool,
    pub end_time: String,
    pub lower: LevelFieldState,
    pub upper: LevelFieldState,
}

impl Default for PeriodRowState {
    fn default() -> Self {
        Self {
            start_indication: true,
            start_time: "09:00".to_owned(),
            end_indication: true,
            end_time: "10:00".to_owned(),
            lower: LevelFieldState {
                value: "000".to_owned(),
                explicit_unit: LevelUnit::FlightLevel,
            },
            upper: LevelFieldState {
                value: "999".to_owned(),
                explicit_unit: LevelUnit::FlightLevel,
            },
        }
    }
}

impl PeriodRowState {
    fn to_period(&self, index: usize, issues: &mut Vec<ValidationIssue>) -> Option<Period> {
        let start_time = parse_time(
            &self.start_time,
            &format!("periods[{index}].start_time"),
            issues,
        );
        let end_time = parse_time(
            &self.end_time,
            &format!("periods[{index}].end_time"),
            issues,
        );
        let lower = parse_level(
            &self.lower.value,
            self.lower.effective_unit(),
            &format!("periods[{index}].lower"),
            issues,
        );
        let upper = parse_level(
            &self.upper.value,
            self.upper.effective_unit(),
            &format!("periods[{index}].upper"),
            issues,
        );

        Some(Period {
            start_indication: self.start_indication,
            start_time: start_time?,
            end_indication: self.end_indication,
            end_time: end_time?,
            lower: lower?,
            upper: upper?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LevelFieldState {
    pub value: String,
    pub explicit_unit: LevelUnit,
}

impl LevelFieldState {
    pub fn is_forced_feet(&self) -> bool {
        self.value
            .trim()
            .chars()
            .filter(|c| c.is_ascii_digit())
            .count()
            >= 4
    }

    pub fn effective_unit(&self) -> LevelUnit {
        if self.is_forced_feet() {
            LevelUnit::Feet
        } else {
            self.explicit_unit
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_digit_levels_are_forced_to_feet_without_losing_explicit_unit() {
        let mut field = LevelFieldState {
            value: "085".to_owned(),
            explicit_unit: LevelUnit::FlightLevel,
        };
        assert_eq!(field.effective_unit(), LevelUnit::FlightLevel);

        field.value = "4500".to_owned();
        assert_eq!(field.effective_unit(), LevelUnit::Feet);
        assert_eq!(field.explicit_unit, LevelUnit::FlightLevel);

        field.value = "450".to_owned();
        assert_eq!(field.effective_unit(), LevelUnit::FlightLevel);
    }
}
