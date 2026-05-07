use crate::{DamCreation, LevelUnit, ValidationError, validate_creation};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("validation failed")]
    Validation(#[from] ValidationError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

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
            kind: "static_map_creation",
            map: ExportMap {
                id: creation.map.id.clone(),
                name: creation.map.name.clone(),
            },
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
pub struct ExportMap {
    pub id: String,
    pub name: String,
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
        AltitudeCorrection, BufferFilter, DateRange, DistributionSelection, Level, LevelUnit,
        Period, SelectedStaticMap, TextInfo,
    };
    use chrono::{NaiveDate, NaiveTime};
    use std::collections::BTreeSet;

    #[test]
    fn exports_stable_pretty_json_with_map_id_and_name_only() {
        let creation = DamCreation {
            map: SelectedStaticMap {
                id: "50714".to_owned(),
                name: "HAUT VALAIS".to_owned(),
            },
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
        };

        let json = to_pretty_json(&creation).unwrap();

        assert!(json.contains(r#""id": "50714""#));
        assert!(json.contains(r#""name": "HAUT VALAIS""#));
        assert!(!json.contains("geojson"));
        assert!(json.contains(r#""unit": "FL""#));
        assert!(json.contains(r#""unit": "ft""#));
    }
}
