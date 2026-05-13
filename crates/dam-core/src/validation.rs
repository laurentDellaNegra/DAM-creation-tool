use crate::{
    DamCreation, DamMap, MAX_PERIODS, MAX_POLYGON_POINTS, ManualGeometry, ManualMap, PolygonNode,
    Weekday,
};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
}

impl ValidationIssue {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub issues: Vec<ValidationIssue>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} validation issue(s)", self.issues.len())
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_creation(creation: &DamCreation) -> Result<(), ValidationError> {
    let mut issues = Vec::new();

    validate_map(&creation.map, &mut issues);

    if creation.date_range.end < creation.date_range.start {
        issues.push(ValidationIssue::new(
            "date_range.end",
            "End date must be on or after start date.",
        ));
    }

    let possible_weekdays = creation.date_range.possible_weekdays();
    let effective_weekdays = creation.date_range.effective_weekdays();
    if creation.date_range.is_repetitive() {
        if creation.date_range.active_weekdays.is_empty() {
            issues.push(ValidationIssue::new(
                "date_range.active_weekdays",
                "At least one active weekday is required.",
            ));
        }
        if !creation
            .date_range
            .active_weekdays
            .is_subset(&possible_weekdays)
        {
            issues.push(ValidationIssue::new(
                "date_range.active_weekdays",
                "Active weekdays must be included in the selected date range.",
            ));
        }
    }

    if creation.periods.is_empty() {
        issues.push(ValidationIssue::new(
            "periods",
            "At least one activation period is required.",
        ));
    }
    if creation.periods.len() > MAX_PERIODS {
        issues.push(ValidationIssue::new(
            "periods",
            format!("At most {MAX_PERIODS} activation periods are allowed."),
        ));
    }

    for (index, period) in creation.periods.iter().enumerate() {
        let prefix = format!("periods[{index}]");
        if period.start_time >= period.end_time {
            issues.push(ValidationIssue::new(
                format!("{prefix}.end_time"),
                "End time must be after start time.",
            ));
        }

        if period.lower.value > 99_999 {
            issues.push(ValidationIssue::new(
                format!("{prefix}.lower.value"),
                "Lower level must be at most 99999.",
            ));
        }
        if period.upper.value > 99_999 {
            issues.push(ValidationIssue::new(
                format!("{prefix}.upper.value"),
                "Upper level must be at most 99999.",
            ));
        }
        if period.lower.comparable_feet() > period.upper.comparable_feet() {
            issues.push(ValidationIssue::new(
                format!("{prefix}.upper"),
                "Upper level must be greater than or equal to lower level.",
            ));
        }
    }

    validate_period_overlaps(&creation.periods, &effective_weekdays, &mut issues);

    if creation.distribution.is_empty() {
        issues.push(ValidationIssue::new(
            "distribution",
            "At least one distribution target must be selected.",
        ));
    }

    if creation.text.value.chars().count() > 250 {
        issues.push(ValidationIssue::new(
            "text.value",
            "Text must be 250 characters or fewer.",
        ));
    }
    validate_exported_string(&creation.text.value, "text.value", &mut issues);

    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationError { issues })
    }
}

pub fn validate_aixm_export_ready(creation: &DamCreation) -> Result<(), ValidationError> {
    let mut issues = Vec::new();

    validate_map_export_ready(&creation.map, &mut issues);
    if creation.periods.is_empty() {
        issues.push(ValidationIssue::new(
            "periods",
            "At least one activation period is required for AIXM export.",
        ));
    }
    if creation.periods.len() > MAX_PERIODS {
        issues.push(ValidationIssue::new(
            "periods",
            format!("At most {MAX_PERIODS} activation periods can be exported."),
        ));
    }
    if creation.distribution.is_empty() {
        issues.push(ValidationIssue::new(
            "distribution",
            "At least one distribution target must be selected for AIXM export.",
        ));
    }
    validate_exported_string(&creation.text.value, "text.value", &mut issues);

    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationError { issues })
    }
}

fn validate_map(map: &DamMap, issues: &mut Vec<ValidationIssue>) {
    match map {
        DamMap::Predefined(selected) => {
            if selected.id.trim().is_empty() {
                issues.push(ValidationIssue::new("map.id", "Static map id is required."));
            }
            if selected.name.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    "map.name",
                    "Static map name is required.",
                ));
            }
        }
        DamMap::Manual(manual) => validate_manual_map(manual, issues),
    }
}

fn validate_map_export_ready(map: &DamMap, issues: &mut Vec<ValidationIssue>) {
    match map {
        DamMap::Predefined(selected) => {
            if selected.id.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    "map.id",
                    "A predefined map must be selected for export.",
                ));
            }
            validate_exported_string(&selected.name, "map.name", issues);
        }
        DamMap::Manual(manual) => {
            validate_exported_string(&manual.name, "map.name", issues);
            if matches!(manual.geometry, ManualGeometry::ParaSymbol { .. })
                && !manual.name.to_ascii_uppercase().contains("PARA")
            {
                issues.push(ValidationIssue::new(
                    "map.name",
                    "Para symbol map name must contain PARA.",
                ));
            }
            validate_manual_geometry_export_ready(&manual.geometry, issues);
        }
    }
}

fn validate_manual_map(manual: &ManualMap, issues: &mut Vec<ValidationIssue>) {
    if manual.name.trim().is_empty() {
        issues.push(ValidationIssue::new(
            "map.name",
            "Manual map name is required.",
        ));
    }
    validate_exported_string(&manual.name, "map.name", issues);

    if manual_geometry_uses_level_label(&manual.geometry) {
        if let Some(label) = manual.label_position {
            validate_coordinate(label, "map.label_position", issues);
        } else {
            issues.push(ValidationIssue::new(
                "map.label_position",
                "Level label position is required.",
            ));
        }
    } else if let Some(label) = manual.label_position {
        validate_coordinate(label, "map.label_position", issues);
    }

    validate_manual_attributes(manual, issues);
    validate_manual_geometry(&manual.geometry, issues);
}

fn manual_geometry_uses_level_label(geometry: &ManualGeometry) -> bool {
    matches!(
        geometry,
        ManualGeometry::Polygon { .. }
            | ManualGeometry::PieCircle { .. }
            | ManualGeometry::Strip { .. }
    )
}

fn validate_manual_attributes(manual: &ManualMap, issues: &mut Vec<ValidationIssue>) {
    if !manual.attributes.lateral_buffer_nm.is_finite() || manual.attributes.lateral_buffer_nm < 0.0
    {
        issues.push(ValidationIssue::new(
            "map.attributes.lateral_buffer_nm",
            "Lateral buffer must be zero or greater.",
        ));
    }
}

fn validate_manual_geometry(geometry: &ManualGeometry, issues: &mut Vec<ValidationIssue>) {
    match geometry {
        ManualGeometry::Polygon { nodes } => {
            let point_count = nodes
                .iter()
                .filter(|node| matches!(node, PolygonNode::Point { .. }))
                .count();
            if point_count < 3 {
                issues.push(ValidationIssue::new(
                    "map.geometry.nodes",
                    "Polygon requires at least 3 points.",
                ));
            }
            if nodes.len() > MAX_POLYGON_POINTS {
                issues.push(ValidationIssue::new(
                    "map.geometry.nodes",
                    format!("Polygon can contain at most {MAX_POLYGON_POINTS} rows."),
                ));
            }
            for (index, node) in nodes.iter().enumerate() {
                match node {
                    PolygonNode::Point { coordinate } => {
                        validate_coordinate(
                            *coordinate,
                            &format!("map.geometry.nodes[{index}]"),
                            issues,
                        );
                    }
                    PolygonNode::Arc { center, radius_nm } => {
                        validate_coordinate(
                            *center,
                            &format!("map.geometry.nodes[{index}].center"),
                            issues,
                        );
                        if !radius_nm.is_finite() || *radius_nm <= 0.0 {
                            issues.push(ValidationIssue::new(
                                format!("map.geometry.nodes[{index}].radius_nm"),
                                "Arc radius must be greater than zero NM.",
                            ));
                        }
                        if !has_point_anchor(nodes, index, false)
                            || !has_point_anchor(nodes, index, true)
                        {
                            issues.push(ValidationIssue::new(
                                format!("map.geometry.nodes[{index}]"),
                                "Polygon arc requires valid adjacent point anchors.",
                            ));
                        }
                    }
                }
            }
        }
        ManualGeometry::ParaSymbol { point } => {
            validate_required_coordinate(*point, "map.geometry.point", issues);
        }
        ManualGeometry::TextNumber {
            point,
            text,
            color: _,
            size: _,
        } => {
            validate_required_coordinate(*point, "map.geometry.point", issues);
            let length = text.chars().count();
            if text.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    "map.geometry.text",
                    "Text and number value is required.",
                ));
            }
            if length > 25 {
                issues.push(ValidationIssue::new(
                    "map.geometry.text",
                    "Text and number value must be 25 characters or fewer.",
                ));
            }
            validate_exported_string(text, "map.geometry.text", issues);
        }
        ManualGeometry::PieCircle {
            center,
            radius_nm,
            first_angle_deg,
            last_angle_deg,
        } => {
            validate_required_coordinate(*center, "map.geometry.center", issues);
            match radius_nm {
                Some(radius) if radius.is_finite() && *radius > 0.0 => {}
                _ => issues.push(ValidationIssue::new(
                    "map.geometry.radius_nm",
                    "Radius must be greater than zero NM.",
                )),
            }
            validate_angle(*first_angle_deg, "map.geometry.first_angle_deg", issues);
            validate_angle(*last_angle_deg, "map.geometry.last_angle_deg", issues);
            if first_angle_deg == last_angle_deg {
                issues.push(ValidationIssue::new(
                    "map.geometry.last_angle_deg",
                    "First and last angles must differ; use 0 and 360 for a full circle.",
                ));
            }
        }
        ManualGeometry::Strip {
            point1,
            point2,
            width_nm,
        } => {
            validate_required_coordinate(*point1, "map.geometry.point1", issues);
            validate_required_coordinate(*point2, "map.geometry.point2", issues);
            if let (Some(point1), Some(point2)) = (*point1, *point2)
                && point1 == point2
            {
                issues.push(ValidationIssue::new(
                    "map.geometry.point2",
                    "Strip endpoints must be different.",
                ));
            }
            match width_nm {
                Some(width) if width.is_finite() && *width > 0.0 => {}
                _ => issues.push(ValidationIssue::new(
                    "map.geometry.width_nm",
                    "Strip width must be greater than zero NM.",
                )),
            }
        }
    }
}

fn validate_required_coordinate(
    coordinate: Option<crate::Coordinate>,
    field: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if let Some(coordinate) = coordinate {
        validate_coordinate(coordinate, field, issues);
    } else {
        issues.push(ValidationIssue::new(field, "Coordinate is required."));
    }
}

fn validate_coordinate(
    coordinate: crate::Coordinate,
    field: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if !coordinate.lat.is_finite()
        || !coordinate.lon.is_finite()
        || coordinate.lat < -90.0
        || coordinate.lat > 90.0
        || coordinate.lon < -180.0
        || coordinate.lon > 180.0
    {
        issues.push(ValidationIssue::new(
            field,
            "Coordinate must contain valid latitude and longitude.",
        ));
    }
}

fn validate_angle(angle: f64, field: &str, issues: &mut Vec<ValidationIssue>) {
    if !angle.is_finite() || !(-360.0..=360.0).contains(&angle) {
        issues.push(ValidationIssue::new(
            field,
            "Angle must be between -360 and 360 degrees.",
        ));
    }
}

fn validate_manual_geometry_export_ready(
    geometry: &ManualGeometry,
    issues: &mut Vec<ValidationIssue>,
) {
    match geometry {
        ManualGeometry::Polygon { nodes } => {
            for (index, node) in nodes.iter().enumerate() {
                if matches!(node, PolygonNode::Arc { .. })
                    && (!has_point_anchor(nodes, index, false)
                        || !has_point_anchor(nodes, index, true))
                {
                    issues.push(ValidationIssue::new(
                        format!("map.geometry.nodes[{index}]"),
                        "Polygon arc requires valid adjacent point anchors.",
                    ));
                }
            }
        }
        ManualGeometry::TextNumber { text, .. } => {
            validate_exported_string(text, "map.geometry.text", issues);
        }
        ManualGeometry::ParaSymbol { .. }
        | ManualGeometry::PieCircle { .. }
        | ManualGeometry::Strip { .. } => {}
    }
}

fn has_point_anchor(nodes: &[PolygonNode], index: usize, previous: bool) -> bool {
    if nodes.len() < 2 {
        return false;
    }
    let anchor_index = if previous {
        if index == 0 {
            nodes.len() - 1
        } else {
            index - 1
        }
    } else {
        (index + 1) % nodes.len()
    };
    matches!(nodes.get(anchor_index), Some(PolygonNode::Point { .. }))
}

fn validate_exported_string(value: &str, field: &str, issues: &mut Vec<ValidationIssue>) {
    if value.chars().any(char::is_control) {
        issues.push(ValidationIssue::new(
            field,
            "Exported text must not contain newlines or control characters.",
        ));
    }
}

fn validate_period_overlaps(
    periods: &[crate::Period],
    active_weekdays: &BTreeSet<Weekday>,
    issues: &mut Vec<ValidationIssue>,
) {
    if active_weekdays.is_empty() {
        return;
    }

    let mut ranges: Vec<(usize, NaiveTime, NaiveTime)> = periods
        .iter()
        .enumerate()
        .map(|(index, period)| (index, period.start_time, period.end_time))
        .collect();
    ranges.sort_by_key(|(_, start, end)| (*start, *end));

    for window in ranges.windows(2) {
        let (left_index, _, left_end) = window[0];
        let (right_index, right_start, _) = window[1];
        if right_start < left_end {
            issues.push(ValidationIssue::new(
                format!("periods[{right_index}]"),
                format!("Period overlaps with period {left_index}."),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        A9Level, AltitudeCorrection, BufferFilter, Coordinate, DateRange, DistributionSelection,
        Level, LevelUnit, ManualMapAttributes, ManualMapCategory, Period, PolygonNode,
        SelectedStaticMap, TextInfo, TextNumberColor, TextNumberSize,
    };
    use chrono::NaiveDate;

    fn valid_creation() -> DamCreation {
        DamCreation {
            map: DamMap::Predefined(SelectedStaticMap {
                id: "50714".to_owned(),
                name: "HAUT VALAIS".to_owned(),
                fallback_geometry: None,
                fallback_label_position: None,
            }),
            date_range: DateRange::new(
                NaiveDate::from_ymd_opt(2026, 5, 7).unwrap(),
                NaiveDate::from_ymd_opt(2026, 5, 7).unwrap(),
            ),
            periods: vec![Period {
                start_indication: true,
                start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end_indication: true,
                end_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                lower: Level::new(85, LevelUnit::FlightLevel),
                upper: Level::new(12_000, LevelUnit::Feet),
            }],
            display_levels: true,
            altitude_correction: AltitudeCorrection::None,
            upper_buffer: BufferFilter::Default,
            lower_buffer: BufferFilter::Default,
            distribution: DistributionSelection::all(),
            a9: A9Level::default(),
            text: TextInfo::default(),
        }
    }

    #[test]
    fn accepts_mixed_altitudes_when_converted_order_is_valid() {
        assert!(validate_creation(&valid_creation()).is_ok());
    }

    #[test]
    fn rejects_mixed_altitudes_when_converted_order_is_invalid() {
        let mut creation = valid_creation();
        creation.periods[0].lower = Level::new(150, LevelUnit::FlightLevel);
        creation.periods[0].upper = Level::new(12_000, LevelUnit::Feet);

        let err = validate_creation(&creation).unwrap_err();

        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "periods[0].upper")
        );
    }

    #[test]
    fn rejects_overlapping_periods() {
        let mut creation = valid_creation();
        creation.periods.push(Period {
            start_indication: true,
            start_time: NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            end_indication: true,
            end_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            lower: Level::new(0, LevelUnit::FlightLevel),
            upper: Level::new(999, LevelUnit::FlightLevel),
        });

        let err = validate_creation(&creation).unwrap_err();

        assert!(err.issues.iter().any(|issue| issue.field == "periods[1]"));
    }

    #[test]
    fn accepts_adjacent_periods() {
        let mut creation = valid_creation();
        creation.periods.push(Period {
            start_indication: true,
            start_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            end_indication: true,
            end_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            lower: Level::new(0, LevelUnit::FlightLevel),
            upper: Level::new(999, LevelUnit::FlightLevel),
        });

        assert!(validate_creation(&creation).is_ok());
    }

    #[test]
    fn rejects_more_than_16_periods() {
        let mut creation = valid_creation();
        creation.periods = (0..17)
            .map(|index| Period {
                start_indication: true,
                start_time: NaiveTime::from_hms_opt(index, 0, 0).unwrap(),
                end_indication: true,
                end_time: NaiveTime::from_hms_opt(index + 1, 0, 0).unwrap(),
                lower: Level::new(0, LevelUnit::FlightLevel),
                upper: Level::new(999, LevelUnit::FlightLevel),
            })
            .collect();

        let err = validate_creation(&creation).unwrap_err();

        assert!(err.issues.iter().any(|issue| issue.field == "periods"));
    }

    #[test]
    fn accepts_complete_manual_polygon() {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(manual_map(ManualGeometry::Polygon {
            nodes: vec![
                PolygonNode::point(coordinate(7.0, 46.0)),
                PolygonNode::point(coordinate(7.2, 46.0)),
                PolygonNode::point(coordinate(7.2, 46.2)),
            ],
        }));

        assert!(validate_creation(&creation).is_ok());
    }

    #[test]
    fn rejects_manual_polygon_without_label_position() {
        let mut creation = valid_creation();
        let mut map = manual_map(ManualGeometry::Polygon {
            nodes: vec![
                PolygonNode::point(coordinate(7.0, 46.0)),
                PolygonNode::point(coordinate(7.2, 46.0)),
                PolygonNode::point(coordinate(7.2, 46.2)),
            ],
        });
        map.label_position = None;
        creation.map = DamMap::Manual(map);

        let err = validate_creation(&creation).unwrap_err();

        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.label_position")
        );
    }

    #[test]
    fn rejects_incomplete_manual_polygon() {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(manual_map(ManualGeometry::Polygon {
            nodes: vec![
                PolygonNode::point(coordinate(7.0, 46.0)),
                PolygonNode::point(coordinate(7.2, 46.0)),
            ],
        }));

        let err = validate_creation(&creation).unwrap_err();

        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.geometry.nodes")
        );
    }

    #[test]
    fn rejects_manual_polygon_with_more_than_10_rows() {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(manual_map(ManualGeometry::Polygon {
            nodes: (0..11)
                .map(|index| PolygonNode::point(coordinate(7.0 + f64::from(index) * 0.01, 46.0)))
                .collect(),
        }));

        let err = validate_creation(&creation).unwrap_err();

        assert!(
            err.issues
                .iter()
                .any(|issue| issue.message.contains("at most 10"))
        );
    }

    #[test]
    fn validates_required_manual_point_geometries() {
        for geometry in [
            ManualGeometry::ParaSymbol { point: None },
            ManualGeometry::TextNumber {
                point: None,
                text: "TXT".to_owned(),
                color: TextNumberColor::Red,
                size: TextNumberSize::Medium,
            },
        ] {
            let mut creation = valid_creation();
            creation.map = DamMap::Manual(manual_map(geometry));

            let err = validate_creation(&creation).unwrap_err();

            assert!(
                err.issues
                    .iter()
                    .any(|issue| issue.field == "map.geometry.point")
            );
        }
    }

    #[test]
    fn accepts_manual_point_geometries_without_level_label_position() {
        for geometry in [
            ManualGeometry::ParaSymbol {
                point: Some(coordinate(7.0, 46.0)),
            },
            ManualGeometry::TextNumber {
                point: Some(coordinate(7.0, 46.0)),
                text: "TXT".to_owned(),
                color: TextNumberColor::Red,
                size: TextNumberSize::Medium,
            },
        ] {
            let mut creation = valid_creation();
            let mut map = manual_map(geometry);
            map.label_position = None;
            creation.map = DamMap::Manual(map);

            assert!(validate_creation(&creation).is_ok());
        }
    }

    #[test]
    fn rejects_long_text_number_value() {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(manual_map(ManualGeometry::TextNumber {
            point: Some(coordinate(7.0, 46.0)),
            text: "12345678901234567890123456".to_owned(),
            color: TextNumberColor::Red,
            size: TextNumberSize::Medium,
        }));

        let err = validate_creation(&creation).unwrap_err();

        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.geometry.text")
        );
    }

    #[test]
    fn rejects_invalid_pie_circle_radius_and_angles() {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(manual_map(ManualGeometry::PieCircle {
            center: Some(coordinate(7.0, 46.0)),
            radius_nm: Some(0.0),
            first_angle_deg: 45.0,
            last_angle_deg: 45.0,
        }));

        let err = validate_creation(&creation).unwrap_err();

        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.geometry.radius_nm")
        );
        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.geometry.last_angle_deg")
        );
    }

    #[test]
    fn rejects_incomplete_strip() {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(manual_map(ManualGeometry::Strip {
            point1: Some(coordinate(7.0, 46.0)),
            point2: None,
            width_nm: None,
        }));

        let err = validate_creation(&creation).unwrap_err();

        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.geometry.point2")
        );
        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.geometry.width_nm")
        );
    }

    fn manual_map(geometry: ManualGeometry) -> ManualMap {
        ManualMap {
            name: "MANUAL MAP".to_owned(),
            geometry,
            attributes: ManualMapAttributes {
                category: ManualMapCategory::Danger,
                lateral_buffer_nm: 0.0,
            },
            label_position: Some(coordinate(7.0, 46.0)),
        }
    }

    fn coordinate(lon: f64, lat: f64) -> Coordinate {
        Coordinate { lon, lat }
    }
}
