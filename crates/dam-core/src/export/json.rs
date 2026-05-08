use crate::{
    DamCreation, DamMap, LevelUnit, ManualGeometry, ManualMap, ManualMapAttributes, PolygonNode,
    validate_creation,
};
use serde::Serialize;

use super::ExportError;

pub fn to_pretty_json(creation: &DamCreation) -> Result<String, ExportError> {
    validate_creation(creation)?;
    let export = DamExport::from_creation(creation);
    Ok(serde_json::to_string_pretty(&export)?)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DamExport {
    pub version: u32,
    pub kind: &'static str,
    pub map: ExportMap,
    pub date_range: ExportDateRange,
    pub periods: Vec<ExportPeriod>,
    pub display_levels: bool,
    pub altitude_correction: &'static str,
    pub upper_buffer: &'static str,
    pub lower_buffer: &'static str,
    pub distribution: ExportDistribution,
    pub text: ExportText,
}

impl DamExport {
    pub fn from_creation(creation: &DamCreation) -> Self {
        Self {
            version: 1,
            kind: "dam_creation",
            map: ExportMap::from(&creation.map),
            date_range: ExportDateRange {
                start_date: creation.date_range.start.format("%Y-%m-%d").to_string(),
                end_date: creation.date_range.end.format("%Y-%m-%d").to_string(),
                active_weekdays: creation
                    .date_range
                    .effective_weekdays()
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            },
            periods: creation
                .periods
                .iter()
                .map(|period| ExportPeriod {
                    start_indication: period.start_indication,
                    start_time: period.start_time.format("%H:%M").to_string(),
                    end_indication: period.end_indication,
                    end_time: period.end_time.format("%H:%M").to_string(),
                    lower: ExportLevel::from(period.lower),
                    upper: ExportLevel::from(period.upper),
                })
                .collect(),
            display_levels: creation.display_levels,
            altitude_correction: match creation.altitude_correction {
                crate::AltitudeCorrection::None => "none",
                crate::AltitudeCorrection::QnhCorr => "qnh_corr",
                crate::AltitudeCorrection::FlCorr => "fl_corr",
            },
            upper_buffer: match creation.upper_buffer {
                crate::BufferFilter::Default => "default",
                crate::BufferFilter::Half => "half",
                crate::BufferFilter::NoBuffer => "no_buffer",
            },
            lower_buffer: match creation.lower_buffer {
                crate::BufferFilter::Default => "default",
                crate::BufferFilter::Half => "half",
                crate::BufferFilter::NoBuffer => "no_buffer",
            },
            distribution: ExportDistribution {
                sectors: creation.distribution.sectors.iter().cloned().collect(),
            },
            text: ExportText {
                value: creation.text.value.clone(),
                display: creation.text.display,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportMap {
    Predefined {
        id: String,
        name: String,
    },
    Manual {
        name: String,
        geometry: ExportManualGeometry,
        attributes: ManualMapAttributes,
        #[serde(skip_serializing_if = "Option::is_none")]
        label_position: Option<crate::Coordinate>,
    },
}

impl From<&DamMap> for ExportMap {
    fn from(map: &DamMap) -> Self {
        match map {
            DamMap::Predefined(selected) => Self::Predefined {
                id: selected.id.clone(),
                name: selected.name.clone(),
            },
            DamMap::Manual(manual) => Self::from(manual),
        }
    }
}

impl From<&ManualMap> for ExportMap {
    fn from(manual: &ManualMap) -> Self {
        Self::Manual {
            name: manual.name.clone(),
            geometry: ExportManualGeometry::from(&manual.geometry),
            attributes: manual.attributes.clone(),
            label_position: manual.label_position,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportManualGeometry {
    Polygon {
        nodes: Vec<PolygonNode>,
    },
    ParaSymbol {
        point: crate::Coordinate,
    },
    TextNumber {
        point: crate::Coordinate,
        text: String,
        color: crate::TextNumberColor,
        size: crate::TextNumberSize,
    },
    PieCircle {
        center: crate::Coordinate,
        radius_nm: f64,
        first_angle_deg: f64,
        last_angle_deg: f64,
    },
    Strip {
        point1: crate::Coordinate,
        point2: crate::Coordinate,
        width_nm: f64,
    },
}

impl From<&ManualGeometry> for ExportManualGeometry {
    fn from(geometry: &ManualGeometry) -> Self {
        match geometry {
            ManualGeometry::Polygon { nodes } => Self::Polygon {
                nodes: nodes.clone(),
            },
            ManualGeometry::ParaSymbol { point } => Self::ParaSymbol {
                point: point.expect("manual maps are validated before export"),
            },
            ManualGeometry::TextNumber {
                point,
                text,
                color,
                size,
            } => Self::TextNumber {
                point: point.expect("manual maps are validated before export"),
                text: text.clone(),
                color: *color,
                size: *size,
            },
            ManualGeometry::PieCircle {
                center,
                radius_nm,
                first_angle_deg,
                last_angle_deg,
            } => Self::PieCircle {
                center: center.expect("manual maps are validated before export"),
                radius_nm: radius_nm.expect("manual maps are validated before export"),
                first_angle_deg: *first_angle_deg,
                last_angle_deg: *last_angle_deg,
            },
            ManualGeometry::Strip {
                point1,
                point2,
                width_nm,
            } => Self::Strip {
                point1: point1.expect("manual maps are validated before export"),
                point2: point2.expect("manual maps are validated before export"),
                width_nm: width_nm.expect("manual maps are validated before export"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportDateRange {
    pub start_date: String,
    pub end_date: String,
    pub active_weekdays: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportPeriod {
    pub start_indication: bool,
    pub start_time: String,
    pub end_indication: bool,
    pub end_time: String,
    pub lower: ExportLevel,
    pub upper: ExportLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ExportLevel {
    pub value: u32,
    pub unit: &'static str,
}

impl From<crate::Level> for ExportLevel {
    fn from(level: crate::Level) -> Self {
        Self {
            value: level.value,
            unit: match level.unit {
                LevelUnit::FlightLevel => "FL",
                LevelUnit::Feet => "ft",
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportDistribution {
    pub sectors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportText {
    pub value: String,
    pub display: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AltitudeCorrection, BufferFilter, Coordinate, DateRange, DistributionSelection, Level,
        LevelUnit, ManualMapCategory, ManualMapRendering, Period, SelectedStaticMap, TextInfo,
        TextNumberColor, TextNumberSize,
    };
    use chrono::{NaiveDate, NaiveTime};
    use std::collections::BTreeSet;

    fn valid_creation(map: DamMap) -> DamCreation {
        DamCreation {
            map,
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 5, 7).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 5, 8).unwrap(),
                active_weekdays: BTreeSet::from([crate::Weekday::Thu, crate::Weekday::Fri]),
            },
            periods: vec![Period {
                start_indication: true,
                start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end_indication: true,
                end_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                lower: Level::new(85, LevelUnit::FlightLevel),
                upper: Level::new(9_500, LevelUnit::Feet),
            }],
            display_levels: true,
            altitude_correction: AltitudeCorrection::QnhCorr,
            upper_buffer: BufferFilter::Half,
            lower_buffer: BufferFilter::NoBuffer,
            distribution: DistributionSelection {
                sectors: BTreeSet::from(["GVA:INN".to_owned()]),
            },
            text: TextInfo {
                value: "comment".to_owned(),
                display: true,
            },
        }
    }

    #[test]
    fn exports_stable_pretty_json_with_tagged_predefined_map() {
        let creation = valid_creation(DamMap::Predefined(SelectedStaticMap {
            id: "50714".to_owned(),
            name: "HAUT VALAIS".to_owned(),
        }));

        let json = to_pretty_json(&creation).unwrap();

        assert!(json.contains(r#""kind": "predefined""#));
        assert!(json.contains(r#""id": "50714""#));
        assert!(json.contains(r#""name": "HAUT VALAIS""#));
        assert!(!json.contains("geojson"));
        assert!(json.contains(r#""unit": "FL""#));
        assert!(json.contains(r#""unit": "ft""#));
    }

    #[test]
    fn invalid_creation_does_not_export_json() {
        let mut creation = valid_creation(DamMap::Predefined(SelectedStaticMap {
            id: "50714".to_owned(),
            name: "HAUT VALAIS".to_owned(),
        }));
        creation.distribution.sectors.clear();

        let error = to_pretty_json(&creation).unwrap_err();

        assert!(matches!(error, ExportError::Validation(_)));
    }

    #[test]
    fn exports_manual_geometry_and_attributes_deterministically() {
        let creation = valid_creation(DamMap::Manual(ManualMap {
            name: "Manual polygon".to_owned(),
            geometry: ManualGeometry::Polygon {
                nodes: vec![
                    PolygonNode::point(Coordinate {
                        lon: 7.0,
                        lat: 46.0,
                    }),
                    PolygonNode::point(Coordinate {
                        lon: 7.2,
                        lat: 46.0,
                    }),
                    PolygonNode::point(Coordinate {
                        lon: 7.2,
                        lat: 46.2,
                    }),
                ],
            },
            attributes: ManualMapAttributes {
                category: ManualMapCategory::Restricted,
                rendering: ManualMapRendering::Line,
                lateral_buffer_nm: 2.5,
            },
            label_position: Some(Coordinate {
                lon: 7.2,
                lat: 46.2,
            }),
        }));

        let json = to_pretty_json(&creation).unwrap();

        assert!(json.contains(r#""kind": "manual""#));
        assert!(json.contains(r#""name": "Manual polygon""#));
        assert!(json.contains(r#""kind": "polygon""#));
        assert!(json.contains(r#""category": "restricted""#));
        assert!(json.contains(r#""rendering": "line""#));
        assert!(json.contains(r#""lateral_buffer_nm": 2.5"#));
        assert!(json.contains(r#""display_levels": true"#));
    }

    #[test]
    fn exports_manual_text_number_without_level_label_position() {
        let creation = valid_creation(DamMap::Manual(ManualMap {
            name: "Manual text".to_owned(),
            geometry: ManualGeometry::TextNumber {
                point: Some(Coordinate {
                    lon: 7.0,
                    lat: 46.0,
                }),
                text: "TXT".to_owned(),
                color: TextNumberColor::Blue,
                size: TextNumberSize::Medium,
            },
            attributes: ManualMapAttributes {
                category: ManualMapCategory::Other,
                rendering: ManualMapRendering::Line,
                lateral_buffer_nm: 0.0,
            },
            label_position: None,
        }));

        let json = to_pretty_json(&creation).unwrap();

        assert!(json.contains(r#""kind": "text_number""#));
        assert!(!json.contains("label_position"));
    }
}
