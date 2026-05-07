use crate::{current_date_text, parse_date, parse_level, parse_time};
use chrono::NaiveDate;
use dam_core::{
    AltitudeCorrection, BufferFilter, Coordinate, DamCreation, DamMap, DateRange,
    DistributionSelection, LevelUnit, MAX_POLYGON_POINTS, ManualGeometry, ManualMap,
    ManualMapAttributes, ManualMapCategory, ManualMapRendering, MapCatalog, MapDefaults, Period,
    PolygonNode, SelectedStaticMap, StaticMap, TextInfo, TextNumberColor, TextNumberSize,
    ValidationIssue, Weekday, default_distribution,
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
pub enum ClickTarget {
    PolygonPoint(usize),
    PolygonArcCenter(usize),
    PolygonArcRadius(usize),
    PolygonLabel,
    ParaSymbolPoint,
    TextNumberPoint,
    PieCenter,
    PieRadius,
    PieLabel,
    StripPoint1,
    StripPoint2,
    StripWidth,
    StripLabel,
}

#[derive(Debug, Clone)]
pub struct NextClickInfo {
    pub anchor: Option<Coordinate>,
    pub label: String,
    pub show_distance: bool,
    pub draw_anchor_line: bool,
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
        let attributes = self.attributes.to_attributes(self.geometry_type, issues);
        let geometry = match self.geometry_type {
            ManualGeometryType::Polygon => ManualGeometry::Polygon {
                nodes: self.polygon.nodes(issues),
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

    pub fn preview_with_cursor(&self, target: ClickTarget, cursor: Coordinate) -> ManualMap {
        let mut probe = self.clone();
        probe.apply_click_target(target, cursor);
        probe.preview_manual_map()
    }

    /// After filling `target`, return the next field that should auto-receive focus,
    /// or `None` if the flow is done. May append a new empty polygon row.
    pub fn next_click_target_after(&mut self, target: ClickTarget) -> Option<ClickTarget> {
        match target {
            ClickTarget::PolygonPoint(i) | ClickTarget::PolygonArcRadius(i) => {
                for j in (i + 1)..self.polygon.nodes.len() {
                    return Some(match self.polygon.nodes[j] {
                        PolygonNodeDraft::Point(_) => ClickTarget::PolygonPoint(j),
                        PolygonNodeDraft::Arc(_) => ClickTarget::PolygonArcCenter(j),
                    });
                }
                if self.polygon.nodes.len() < MAX_POLYGON_POINTS {
                    self.polygon
                        .nodes
                        .push(PolygonNodeDraft::Point(CoordinateFieldState::default()));
                    Some(ClickTarget::PolygonPoint(self.polygon.nodes.len() - 1))
                } else {
                    None
                }
            }
            ClickTarget::PolygonArcCenter(i) => Some(ClickTarget::PolygonArcRadius(i)),
            ClickTarget::PolygonLabel => None,
            ClickTarget::ParaSymbolPoint | ClickTarget::TextNumberPoint => None,
            ClickTarget::PieCenter => Some(ClickTarget::PieRadius),
            ClickTarget::PieRadius => None,
            ClickTarget::PieLabel => None,
            ClickTarget::StripPoint1 => Some(ClickTarget::StripPoint2),
            ClickTarget::StripPoint2 => Some(ClickTarget::StripWidth),
            ClickTarget::StripWidth => None,
            ClickTarget::StripLabel => None,
        }
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

    pub fn apply_click_target(&mut self, target: ClickTarget, coord: Coordinate) {
        match target {
            ClickTarget::PolygonPoint(i) => {
                if let Some(PolygonNodeDraft::Point(field)) = self.polygon.nodes.get_mut(i) {
                    *field = CoordinateFieldState::from_coordinate(coord);
                }
            }
            ClickTarget::PolygonArcCenter(i) => {
                if let Some(PolygonNodeDraft::Arc(arc)) = self.polygon.nodes.get_mut(i) {
                    arc.center = CoordinateFieldState::from_coordinate(coord);
                }
            }
            ClickTarget::PolygonArcRadius(i) => {
                if let Some(PolygonNodeDraft::Arc(arc)) = self.polygon.nodes.get_mut(i) {
                    if let Some(center) = arc.center.silent_coordinate() {
                        arc.radius_nm = format_nm(distance_nm(center, coord));
                    }
                }
            }
            ClickTarget::PolygonLabel => {
                self.polygon.label = CoordinateFieldState::from_coordinate(coord);
            }
            ClickTarget::ParaSymbolPoint => {
                self.para_symbol.point = CoordinateFieldState::from_coordinate(coord);
            }
            ClickTarget::TextNumberPoint => {
                self.text_number.point = CoordinateFieldState::from_coordinate(coord);
            }
            ClickTarget::PieCenter => {
                self.pie_circle.center = CoordinateFieldState::from_coordinate(coord);
            }
            ClickTarget::PieRadius => {
                if let Some(center) = self.pie_circle.center.silent_coordinate() {
                    self.pie_circle.radius_nm = format_nm(distance_nm(center, coord));
                }
            }
            ClickTarget::PieLabel => {
                self.pie_circle.label = CoordinateFieldState::from_coordinate(coord);
            }
            ClickTarget::StripPoint1 => {
                self.strip.point1 = CoordinateFieldState::from_coordinate(coord);
            }
            ClickTarget::StripPoint2 => {
                self.strip.point2 = CoordinateFieldState::from_coordinate(coord);
            }
            ClickTarget::StripWidth => {
                if let (Some(p1), Some(p2)) = (
                    self.strip.point1.silent_coordinate(),
                    self.strip.point2.silent_coordinate(),
                ) {
                    let perp = perpendicular_distance_nm(p1, p2, coord);
                    self.strip.width_nm = format_nm(perp * 2.0);
                } else if let Some(p1) = self.strip.point1.silent_coordinate() {
                    self.strip.width_nm = format_nm(distance_nm(p1, coord));
                }
            }
            ClickTarget::StripLabel => {
                self.strip.label = CoordinateFieldState::from_coordinate(coord);
            }
        }
    }

    pub fn next_click_info(&self, target: ClickTarget, level_label: &str) -> NextClickInfo {
        match target {
            ClickTarget::PolygonPoint(i) => {
                let anchor = self.polygon.previous_anchor(i);
                NextClickInfo {
                    anchor,
                    label: format!("Point {}", point_label_index(&self.polygon, i)),
                    show_distance: false,
                    draw_anchor_line: anchor.is_some(),
                }
            }
            ClickTarget::PolygonArcCenter(i) => NextClickInfo {
                anchor: None,
                label: format!("Arc {} center", arc_label_index(&self.polygon, i)),
                show_distance: false,
                draw_anchor_line: false,
            },
            ClickTarget::PolygonArcRadius(i) => {
                let center = self.polygon.nodes.get(i).and_then(|node| match node {
                    PolygonNodeDraft::Arc(arc) => arc.center.silent_coordinate(),
                    _ => None,
                });
                NextClickInfo {
                    anchor: center,
                    label: format!("Arc {} radius", arc_label_index(&self.polygon, i)),
                    show_distance: true,
                    draw_anchor_line: center.is_some(),
                }
            }
            ClickTarget::PolygonLabel
            | ClickTarget::PieLabel
            | ClickTarget::StripLabel => NextClickInfo {
                anchor: None,
                label: level_label.to_owned(),
                show_distance: false,
                draw_anchor_line: false,
            },
            ClickTarget::ParaSymbolPoint | ClickTarget::TextNumberPoint => NextClickInfo {
                anchor: None,
                label: "Position".to_owned(),
                show_distance: false,
                draw_anchor_line: false,
            },
            ClickTarget::PieCenter => NextClickInfo {
                anchor: None,
                label: "Center".to_owned(),
                show_distance: false,
                draw_anchor_line: false,
            },
            ClickTarget::PieRadius => {
                let center = self.pie_circle.center.silent_coordinate();
                NextClickInfo {
                    anchor: center,
                    label: "Radius".to_owned(),
                    show_distance: true,
                    draw_anchor_line: center.is_some(),
                }
            }
            ClickTarget::StripPoint1 => NextClickInfo {
                anchor: None,
                label: "Point 1".to_owned(),
                show_distance: false,
                draw_anchor_line: false,
            },
            ClickTarget::StripPoint2 => {
                let anchor = self.strip.point1.silent_coordinate();
                NextClickInfo {
                    anchor,
                    label: "Point 2".to_owned(),
                    show_distance: false,
                    draw_anchor_line: anchor.is_some(),
                }
            }
            ClickTarget::StripWidth => {
                let anchor = self.strip.point1.silent_coordinate();
                NextClickInfo {
                    anchor,
                    label: "Width".to_owned(),
                    show_distance: true,
                    draw_anchor_line: anchor.is_some(),
                }
            }
        }
    }
}

fn point_label_index(polygon: &PolygonDraftState, index: usize) -> usize {
    polygon
        .nodes
        .iter()
        .take(index + 1)
        .filter(|node| matches!(node, PolygonNodeDraft::Point(_)))
        .count()
}

fn arc_label_index(polygon: &PolygonDraftState, index: usize) -> usize {
    polygon
        .nodes
        .iter()
        .take(index + 1)
        .filter(|node| matches!(node, PolygonNodeDraft::Arc(_)))
        .count()
}

fn perpendicular_distance_nm(line_a: Coordinate, line_b: Coordinate, point: Coordinate) -> f64 {
    let bearing = bearing_deg(line_a, line_b);
    let along_bearing = bearing_deg(line_a, point);
    let dist = distance_nm(line_a, point);
    let angle = (along_bearing - bearing).to_radians();
    (dist * angle.sin()).abs()
}

fn bearing_deg(from: Coordinate, to: Coordinate) -> f64 {
    let lat1 = from.lat.to_radians();
    let lat2 = to.lat.to_radians();
    let dlon = (to.lon - from.lon).to_radians();
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    y.atan2(x).to_degrees()
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
    fn to_attributes(
        &self,
        geometry_type: ManualGeometryType,
        issues: &mut Vec<ValidationIssue>,
    ) -> ManualMapAttributes {
        let lateral_buffer_nm = if geometry_supports_buffer(geometry_type) {
            parse_f64_or_default(
                &self.lateral_buffer_nm,
                0.0,
                "map.attributes.lateral_buffer_nm",
                issues,
            )
        } else {
            0.0
        };
        ManualMapAttributes {
            category: self.category,
            rendering: self.rendering,
            lateral_buffer_nm,
        }
    }
}

pub fn geometry_supports_buffer(geometry_type: ManualGeometryType) -> bool {
    matches!(
        geometry_type,
        ManualGeometryType::Polygon
            | ManualGeometryType::PieCircle
            | ManualGeometryType::Strip
    )
}

#[derive(Debug, Clone, Default)]
pub struct PolygonDraftState {
    pub nodes: Vec<PolygonNodeDraft>,
    pub label: CoordinateFieldState,
}

#[derive(Debug, Clone)]
pub enum PolygonNodeDraft {
    Point(CoordinateFieldState),
    Arc(ArcDraftState),
}

#[derive(Debug, Clone, Default)]
pub struct ArcDraftState {
    pub center: CoordinateFieldState,
    pub radius_nm: String,
}

impl PolygonDraftState {
    fn nodes(&self, issues: &mut Vec<ValidationIssue>) -> Vec<PolygonNode> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| match node {
                PolygonNodeDraft::Point(field) => field
                    .coordinate(&format!("map.geometry.nodes[{index}]"), issues)
                    .map(|coordinate| PolygonNode::Point { coordinate }),
                PolygonNodeDraft::Arc(arc) => {
                    let center = arc
                        .center
                        .coordinate(&format!("map.geometry.nodes[{index}].center"), issues);
                    let radius_nm = parse_optional_positive_f64(
                        &arc.radius_nm,
                        &format!("map.geometry.nodes[{index}].radius_nm"),
                        issues,
                    );
                    match (center, radius_nm) {
                        (Some(center), Some(radius_nm)) => {
                            Some(PolygonNode::Arc { center, radius_nm })
                        }
                        _ => None,
                    }
                }
            })
            .collect()
    }

    fn label_position(&self) -> Option<Coordinate> {
        self.label.silent_coordinate()
    }

    pub fn previous_anchor(&self, before_index: usize) -> Option<Coordinate> {
        self.nodes
            .iter()
            .take(before_index)
            .rev()
            .find_map(|node| match node {
                PolygonNodeDraft::Point(field) => field.silent_coordinate(),
                PolygonNodeDraft::Arc(arc) => arc.center.silent_coordinate(),
            })
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
    pub label: CoordinateFieldState,
}

impl Default for PieCircleDraftState {
    fn default() -> Self {
        Self {
            center: CoordinateFieldState::default(),
            radius_nm: String::new(),
            first_angle_deg: "0".to_owned(),
            last_angle_deg: "360".to_owned(),
            label: CoordinateFieldState::default(),
        }
    }
}

impl PieCircleDraftState {
    fn label_position(&self) -> Option<Coordinate> {
        self.label.silent_coordinate()
    }
}

#[derive(Debug, Clone)]
pub struct StripDraftState {
    pub point1: CoordinateFieldState,
    pub point2: CoordinateFieldState,
    pub width_nm: String,
    pub label: CoordinateFieldState,
}

impl Default for StripDraftState {
    fn default() -> Self {
        Self {
            point1: CoordinateFieldState::default(),
            point2: CoordinateFieldState::default(),
            width_nm: String::new(),
            label: CoordinateFieldState::default(),
        }
    }
}

impl StripDraftState {
    fn label_position(&self) -> Option<Coordinate> {
        self.label.silent_coordinate()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CoordinateFieldState {
    pub lat: String,
    pub lon: String,
}

impl CoordinateFieldState {
    pub fn from_coordinate(coordinate: Coordinate) -> Self {
        Self {
            lat: format_coordinate(coordinate.lat),
            lon: format_coordinate(coordinate.lon),
        }
    }

    pub fn coordinate(&self, field: &str, issues: &mut Vec<ValidationIssue>) -> Option<Coordinate> {
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

    pub fn silent_coordinate(&self) -> Option<Coordinate> {
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

    pub fn apply_defaults(&mut self, defaults: &MapDefaults) {
        if let Some(ref val) = defaults.lower_level {
            for period in &mut self.periods {
                period.lower.value = val.clone();
            }
        }
        if let Some(unit) = defaults.lower_unit {
            for period in &mut self.periods {
                period.lower.explicit_unit = unit;
            }
        }
        if let Some(ref val) = defaults.upper_level {
            for period in &mut self.periods {
                period.upper.value = val.clone();
            }
        }
        if let Some(unit) = defaults.upper_unit {
            for period in &mut self.periods {
                period.upper.explicit_unit = unit;
            }
        }
        if let Some(bt) = defaults.start_indication {
            for period in &mut self.periods {
                period.start_indication = bt;
            }
        }
        if let Some(et) = defaults.end_indication {
            for period in &mut self.periods {
                period.end_indication = et;
            }
        }
        if let Some(dl) = defaults.display_levels {
            self.display_levels = dl;
        }
        if let Some(ac) = defaults.altitude_correction {
            self.altitude_correction = ac;
        }
        if let Some(ub) = defaults.upper_buffer {
            self.upper_buffer = ub;
        }
        if let Some(lb) = defaults.lower_buffer {
            self.lower_buffer = lb;
        }
        if let Some(ref text) = defaults.text {
            self.text = text.clone();
            self.display_text = true;
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
