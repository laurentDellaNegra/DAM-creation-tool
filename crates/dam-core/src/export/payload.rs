use super::ExportError;
use super::aixm::{AixmExportError, to_aixm_xml};
use super::json::to_pretty_json;
use crate::{DamCreation, DamMap};

pub const JSON_CONTENT_TYPE: &str = "application/json";
pub const AIXM_XML_CONTENT_TYPE: &str = "application/xml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionPayload {
    pub filename: String,
    pub content_type: &'static str,
    pub body: String,
}

pub fn build_json_payload(creation: &DamCreation) -> Result<SubmissionPayload, ExportError> {
    Ok(SubmissionPayload {
        filename: "dam-export.json".to_owned(),
        content_type: JSON_CONTENT_TYPE,
        body: to_pretty_json(creation)?,
    })
}

pub fn build_aixm_payload(creation: &DamCreation) -> Result<SubmissionPayload, ExportError> {
    let body = to_aixm_xml(creation).map_err(|error| match error {
        AixmExportError::Validation(error) => ExportError::Validation(error),
        error => ExportError::Aixm(error),
    })?;

    Ok(SubmissionPayload {
        filename: aixm_filename(creation),
        content_type: AIXM_XML_CONTENT_TYPE,
        body,
    })
}

fn aixm_filename(creation: &DamCreation) -> String {
    let map_part = match &creation.map {
        DamMap::Predefined(selected) => sanitize_filename_part(&selected.id),
        DamMap::Manual(_) => "manual".to_owned(),
    };
    format!(
        "dam-{map_part}-{}.xml",
        creation.date_range.start.format("%Y%m%d")
    )
}

fn sanitize_filename_part(value: &str) -> String {
    let sanitized = value
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>();
    if sanitized.is_empty() {
        "unknown".to_owned()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AixmExportError, AltitudeCorrection, BufferFilter, DamMap, DateRange,
        DistributionSelection, ExportError, Level, LevelUnit, Period, SelectedStaticMap, TextInfo,
    };
    use chrono::{NaiveDate, NaiveTime};
    use std::collections::BTreeSet;

    fn valid_creation() -> DamCreation {
        DamCreation {
            map: DamMap::Predefined(SelectedStaticMap {
                id: "50714".to_owned(),
                name: "HAUT VALAIS".to_owned(),
            }),
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
    fn json_payload_uses_expected_filename_content_type_and_body() {
        let creation = valid_creation();
        let payload = build_json_payload(&creation).unwrap();

        assert_eq!(payload.filename, "dam-export.json");
        assert_eq!(payload.content_type, JSON_CONTENT_TYPE);
        assert_eq!(payload.body, to_pretty_json(&creation).unwrap());
    }

    #[test]
    fn invalid_creation_does_not_build_json_payload() {
        let mut creation = valid_creation();
        creation.distribution.sectors.clear();

        let error = build_json_payload(&creation).unwrap_err();

        assert!(matches!(error, ExportError::Validation(_)));
    }

    #[test]
    fn invalid_creation_does_not_build_aixm_payload() {
        let mut creation = valid_creation();
        creation.distribution.sectors.clear();

        let error = build_aixm_payload(&creation).unwrap_err();

        assert!(matches!(error, ExportError::Validation(_)));
    }

    #[test]
    fn aixm_payload_uses_expected_filename_content_type_and_body() {
        let creation = valid_creation();
        let payload = build_aixm_payload(&creation).unwrap();

        assert_eq!(payload.filename, "dam-50714-20260507.xml");
        assert_eq!(payload.content_type, AIXM_XML_CONTENT_TYPE);
        assert_eq!(payload.body, to_aixm_xml(&creation).unwrap());
    }

    #[test]
    fn unsupported_aixm_creation_does_not_build_payload() {
        let mut creation = valid_creation();
        creation.periods.push(Period {
            start_indication: true,
            start_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            end_indication: true,
            end_time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            lower: Level::new(85, LevelUnit::FlightLevel),
            upper: Level::new(9_500, LevelUnit::Feet),
        });

        let error = build_aixm_payload(&creation).unwrap_err();

        assert!(matches!(
            error,
            ExportError::Aixm(AixmExportError::UnsupportedPeriodCount { count: 2 })
        ));
    }
}
