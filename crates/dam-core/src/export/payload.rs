use super::ExportError;
use super::aixm::{AixmExportError, to_aixm_xml};
use crate::{DamCreation, DamMap};

pub const AIXM_XML_CONTENT_TYPE: &str = "application/xml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AixmPayload {
    pub filename: String,
    pub content_type: &'static str,
    pub body: String,
}

pub fn build_aixm_payload(creation: &DamCreation) -> Result<AixmPayload, ExportError> {
    let body = to_aixm_xml(creation).map_err(|error| match error {
        AixmExportError::Validation(error) => ExportError::Validation(error),
        error => ExportError::Aixm(error),
    })?;

    Ok(AixmPayload {
        filename: aixm_filename(creation),
        content_type: AIXM_XML_CONTENT_TYPE,
        body,
    })
}

pub fn build_aixm_payload_from_xml(creation: &DamCreation, body: String) -> AixmPayload {
    AixmPayload {
        filename: aixm_filename(creation),
        content_type: AIXM_XML_CONTENT_TYPE,
        body,
    }
}

fn aixm_filename(creation: &DamCreation) -> String {
    let date = creation.date_range.start.format("%Y%m%d");
    match &creation.map {
        DamMap::Predefined(selected) => format!(
            "DAM-{}-{}-{date}.xml",
            sanitize_filename_part(&selected.id),
            sanitize_filename_part(&selected.name.to_uppercase())
        ),
        DamMap::Manual(manual) => format!(
            "DAM-{}-{date}.xml",
            sanitize_filename_part(&manual.name.to_uppercase())
        ),
    }
}

fn sanitize_filename_part(value: &str) -> String {
    let mut sanitized = String::new();
    let mut last_was_separator = false;
    for ch in value.trim().chars() {
        let out = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_uppercase())
        } else if matches!(ch, '-' | '_' | ' ' | '/' | '\\' | ':' | '.' | '(' | ')') {
            Some('-')
        } else {
            None
        };
        if let Some(out) = out {
            if out == '-' {
                if !last_was_separator && !sanitized.is_empty() {
                    sanitized.push(out);
                    last_was_separator = true;
                }
            } else {
                sanitized.push(out);
                last_was_separator = false;
            }
        }
    }
    while sanitized.ends_with('-') {
        sanitized.pop();
    }
    if sanitized.is_empty() {
        "UNKNOWN".to_owned()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        A9Level, AltitudeCorrection, BufferFilter, Coordinate, DamMap, DateRange,
        DistributionSelection, ExportError, Level, LevelUnit, ManualGeometry, ManualMap,
        ManualMapAttributes, ManualMapCategory, Period, SelectedStaticMap, TextInfo,
    };
    use chrono::{NaiveDate, NaiveTime};
    use std::collections::BTreeSet;

    fn valid_creation() -> DamCreation {
        DamCreation {
            map: DamMap::Predefined(SelectedStaticMap {
                id: "50714".to_owned(),
                name: "HAUT VALAIS".to_owned(),
                fallback_geometry: None,
                fallback_label_position: None,
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
            distribution: DistributionSelection::all(),
            a9: A9Level::default(),
            text: TextInfo {
                value: "comment".to_owned(),
            },
        }
    }

    #[test]
    fn invalid_creation_does_not_build_aixm_payload() {
        let mut creation = valid_creation();
        creation.distribution = DistributionSelection::none();

        let error = build_aixm_payload(&creation).unwrap_err();

        assert!(matches!(error, ExportError::Validation(_)));
    }

    #[test]
    fn aixm_payload_uses_expected_filename_content_type_and_body() {
        let creation = valid_creation();
        let payload = build_aixm_payload(&creation).unwrap();

        assert_eq!(payload.filename, "DAM-50714-HAUT-VALAIS-20260507.xml");
        assert_eq!(payload.content_type, AIXM_XML_CONTENT_TYPE);
        assert_eq!(payload.body, to_aixm_xml(&creation).unwrap());
    }

    #[test]
    fn edited_aixm_payload_uses_aixm_filename_and_body() {
        let creation = valid_creation();
        let payload = build_aixm_payload_from_xml(&creation, "<xml/>".to_owned());

        assert_eq!(payload.filename, "DAM-50714-HAUT-VALAIS-20260507.xml");
        assert_eq!(payload.content_type, AIXM_XML_CONTENT_TYPE);
        assert_eq!(payload.body, "<xml/>");
    }

    #[test]
    fn manual_filename_is_uppercase_and_sanitized() {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(ManualMap {
            name: "my/manual para!".to_owned(),
            geometry: ManualGeometry::ParaSymbol {
                point: Some(Coordinate {
                    lon: 7.0,
                    lat: 46.0,
                }),
            },
            attributes: ManualMapAttributes {
                category: ManualMapCategory::Para,
                lateral_buffer_nm: 0.0,
            },
            label_position: None,
        });

        let payload = build_aixm_payload(&creation).unwrap();

        assert_eq!(payload.filename, "DAM-MY-MANUAL-PARA-20260507.xml");
    }

    #[test]
    fn multiple_activation_periods_build_payload() {
        let mut creation = valid_creation();
        creation.periods.push(Period {
            start_indication: true,
            start_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            end_indication: true,
            end_time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            lower: Level::new(85, LevelUnit::FlightLevel),
            upper: Level::new(9_500, LevelUnit::Feet),
        });

        let payload = build_aixm_payload(&creation).unwrap();

        assert_eq!(payload.body.matches("<aixm:Timesheet").count(), 4);
    }
}
