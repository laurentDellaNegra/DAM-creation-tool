use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

use crate::geometry::{bearing_deg, destination_point};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamCreation {
    pub map: DamMap,
    pub date_range: DateRange,
    pub periods: Vec<Period>,
    pub display_levels: bool,
    pub altitude_correction: AltitudeCorrection,
    pub upper_buffer: BufferFilter,
    pub lower_buffer: BufferFilter,
    pub distribution: crate::DistributionSelection,
    #[serde(default)]
    pub a9: A9Level,
    pub text: TextInfo,
}

pub const MAX_PERIODS: usize = 16;
pub const MAX_POLYGON_POINTS: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DamMap {
    Predefined(SelectedStaticMap),
    Manual(ManualMap),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedStaticMap {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_geometry: Option<Vec<crate::Coordinate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_label_position: Option<crate::Coordinate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManualMap {
    pub name: String,
    pub geometry: ManualGeometry,
    pub attributes: ManualMapAttributes,
    pub label_position: Option<crate::Coordinate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ManualGeometry {
    Polygon {
        nodes: Vec<PolygonNode>,
    },
    ParaSymbol {
        point: Option<crate::Coordinate>,
    },
    TextNumber {
        point: Option<crate::Coordinate>,
        text: String,
        color: TextNumberColor,
        size: TextNumberSize,
    },
    PieCircle {
        center: Option<crate::Coordinate>,
        radius_nm: Option<f64>,
        first_angle_deg: f64,
        last_angle_deg: f64,
    },
    Strip {
        point1: Option<crate::Coordinate>,
        point2: Option<crate::Coordinate>,
        width_nm: Option<f64>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolygonNode {
    Point {
        coordinate: crate::Coordinate,
    },
    Arc {
        center: crate::Coordinate,
        radius_nm: f64,
    },
}

impl PolygonNode {
    pub fn point(coordinate: crate::Coordinate) -> Self {
        Self::Point { coordinate }
    }

    pub fn point_coordinate(&self) -> Option<crate::Coordinate> {
        match self {
            Self::Point { coordinate } => Some(*coordinate),
            Self::Arc { .. } => None,
        }
    }
}

/// Expand a list of polygon nodes into a flat list of coordinates by sampling
/// arcs between their adjacent point anchors.
pub fn expand_polygon_nodes(nodes: &[PolygonNode]) -> Vec<crate::Coordinate> {
    let n = nodes.len();
    if n == 0 {
        return Vec::new();
    }
    let mut result: Vec<crate::Coordinate> = Vec::new();
    for i in 0..n {
        match &nodes[i] {
            PolygonNode::Point { coordinate } => result.push(*coordinate),
            PolygonNode::Arc { center, radius_nm } => {
                let prev_anchor = adjacent_anchor(nodes, i, false);
                let next_anchor = adjacent_anchor(nodes, i, true);
                if let (Some(prev), Some(next)) = (prev_anchor, next_anchor) {
                    let start_angle = bearing_deg(*center, prev);
                    let end_angle = bearing_deg(*center, next);
                    let span = shorter_arc_span(start_angle, end_angle);
                    let segments: usize = 24;
                    for k in 1..segments {
                        let t = k as f64 / segments as f64;
                        let angle = start_angle + span * t;
                        result.push(destination_point(*center, angle, *radius_nm));
                    }
                }
            }
        }
    }
    result
}

fn adjacent_anchor(nodes: &[PolygonNode], from: usize, forward: bool) -> Option<crate::Coordinate> {
    let n = nodes.len();
    if n == 0 {
        return None;
    }
    let mut idx = from;
    for _ in 0..n {
        idx = if forward {
            (idx + 1) % n
        } else {
            (idx + n - 1) % n
        };
        if let PolygonNode::Point { coordinate } = nodes[idx] {
            return Some(coordinate);
        }
    }
    None
}

fn shorter_arc_span(start_deg: f64, end_deg: f64) -> f64 {
    let mut diff = end_deg - start_deg;
    while diff > 180.0 {
        diff -= 360.0;
    }
    while diff < -180.0 {
        diff += 360.0;
    }
    diff
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManualMapAttributes {
    pub category: ManualMapCategory,
    pub lateral_buffer_nm: f64,
}

impl Default for ManualMapAttributes {
    fn default() -> Self {
        Self {
            category: ManualMapCategory::Danger,
            lateral_buffer_nm: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualMapCategory {
    Prohibited,
    Danger,
    Restricted,
    Glider,
    Ctr,
    Cfz,
    Tma,
    Para,
    Other,
}

impl ManualMapCategory {
    pub const ALL: [ManualMapCategory; 9] = [
        ManualMapCategory::Prohibited,
        ManualMapCategory::Danger,
        ManualMapCategory::Restricted,
        ManualMapCategory::Glider,
        ManualMapCategory::Ctr,
        ManualMapCategory::Cfz,
        ManualMapCategory::Tma,
        ManualMapCategory::Para,
        ManualMapCategory::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ManualMapCategory::Prohibited => "Prohibited",
            ManualMapCategory::Danger => "Danger",
            ManualMapCategory::Restricted => "Restricted",
            ManualMapCategory::Glider => "Glider",
            ManualMapCategory::Ctr => "CTR",
            ManualMapCategory::Cfz => "CFZ",
            ManualMapCategory::Tma => "TMA",
            ManualMapCategory::Para => "Para",
            ManualMapCategory::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextNumberColor {
    Red,
    Green,
    Blue,
    Yellow,
    White,
}

impl TextNumberColor {
    pub const ALL: [TextNumberColor; 5] = [
        TextNumberColor::Red,
        TextNumberColor::Green,
        TextNumberColor::Blue,
        TextNumberColor::Yellow,
        TextNumberColor::White,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TextNumberColor::Red => "Red",
            TextNumberColor::Green => "Green",
            TextNumberColor::Blue => "Blue",
            TextNumberColor::Yellow => "Yellow",
            TextNumberColor::White => "White",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextNumberSize {
    Small,
    Medium,
    Large,
}

impl TextNumberSize {
    pub const ALL: [TextNumberSize; 3] = [
        TextNumberSize::Small,
        TextNumberSize::Medium,
        TextNumberSize::Large,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TextNumberSize::Small => "Small",
            TextNumberSize::Medium => "Medium",
            TextNumberSize::Large => "Large",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub active_weekdays: BTreeSet<Weekday>,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            start,
            end,
            active_weekdays: possible_weekdays(start, end),
        }
    }

    pub fn is_repetitive(&self) -> bool {
        self.start != self.end
    }

    pub fn effective_weekdays(&self) -> BTreeSet<Weekday> {
        if self.is_repetitive() {
            self.active_weekdays.clone()
        } else {
            BTreeSet::from([Weekday::from_chrono(self.start.weekday())])
        }
    }

    pub fn possible_weekdays(&self) -> BTreeSet<Weekday> {
        possible_weekdays(self.start, self.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    pub const ALL: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    pub fn from_chrono(value: chrono::Weekday) -> Self {
        match value {
            chrono::Weekday::Mon => Weekday::Mon,
            chrono::Weekday::Tue => Weekday::Tue,
            chrono::Weekday::Wed => Weekday::Wed,
            chrono::Weekday::Thu => Weekday::Thu,
            chrono::Weekday::Fri => Weekday::Fri,
            chrono::Weekday::Sat => Weekday::Sat,
            chrono::Weekday::Sun => Weekday::Sun,
        }
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Period {
    pub start_indication: bool,
    pub start_time: NaiveTime,
    pub end_indication: bool,
    pub end_time: NaiveTime,
    pub lower: Level,
    pub upper: Level,
}

impl Period {
    pub fn default_with_times(start_time: NaiveTime, end_time: NaiveTime) -> Self {
        Self {
            start_indication: true,
            start_time,
            end_indication: true,
            end_time,
            lower: Level::new(0, LevelUnit::FlightLevel),
            upper: Level::new(999, LevelUnit::FlightLevel),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level {
    pub value: u32,
    pub unit: LevelUnit,
}

impl Level {
    pub const fn new(value: u32, unit: LevelUnit) -> Self {
        Self { value, unit }
    }

    pub fn comparable_feet(self) -> u32 {
        match self.unit {
            LevelUnit::FlightLevel => self.value.saturating_mul(100),
            LevelUnit::Feet => self.value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LevelUnit {
    FlightLevel,
    Feet,
}

impl LevelUnit {
    pub fn label(self) -> &'static str {
        match self {
            LevelUnit::FlightLevel => "FL",
            LevelUnit::Feet => "ft",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum A9Level {
    #[default]
    L140_0,
    L150_1,
    L160_2,
}

impl A9Level {
    pub const ALL: [A9Level; 3] = [A9Level::L140_0, A9Level::L150_1, A9Level::L160_2];

    pub fn export_value(self) -> &'static str {
        match self {
            A9Level::L140_0 => "140:0",
            A9Level::L150_1 => "150:1",
            A9Level::L160_2 => "160:2",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            A9Level::L140_0 => "140",
            A9Level::L150_1 => "150",
            A9Level::L160_2 => "160",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AltitudeCorrection {
    #[default]
    None,
    QnhCorr,
    FlCorr,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BufferFilter {
    #[default]
    Default,
    Half,
    NoBuffer,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInfo {
    pub value: String,
}

pub fn possible_weekdays(start: NaiveDate, end: NaiveDate) -> BTreeSet<Weekday> {
    if end < start {
        return BTreeSet::new();
    }

    let mut result = BTreeSet::new();
    let mut date = start;
    while date <= end {
        result.insert(Weekday::from_chrono(date.weekday()));
        date += Duration::days(1);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a9_labels_are_short_but_export_values_keep_legacy_suffix() {
        assert_eq!(A9Level::L140_0.label(), "140");
        assert_eq!(A9Level::L150_1.label(), "150");
        assert_eq!(A9Level::L160_2.label(), "160");
        assert_eq!(A9Level::L140_0.export_value(), "140:0");
        assert_eq!(A9Level::L150_1.export_value(), "150:1");
        assert_eq!(A9Level::L160_2.export_value(), "160:2");
    }
}
