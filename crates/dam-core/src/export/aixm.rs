use crate::{DamCreation, ValidationError, validate_creation};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AixmExportError {
    #[error("validation failed")]
    Validation(#[from] ValidationError),
    #[error("AIXM export template is not configured")]
    TemplateMissing,
}

pub fn to_aixm_xml(creation: &DamCreation) -> Result<String, AixmExportError> {
    validate_creation(creation)?;
    Err(AixmExportError::TemplateMissing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AltitudeCorrection, BufferFilter, DamMap, DateRange, DistributionSelection, Level,
        LevelUnit, Period, SelectedStaticMap, TextInfo,
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
    fn aixm_export_validates_before_template_lookup() {
        let mut creation = valid_creation();
        creation.distribution.sectors.clear();

        let error = to_aixm_xml(&creation).unwrap_err();

        assert!(matches!(error, AixmExportError::Validation(_)));
    }

    #[test]
    fn aixm_export_reports_missing_template_for_valid_creation() {
        let error = to_aixm_xml(&valid_creation()).unwrap_err();

        assert!(matches!(error, AixmExportError::TemplateMissing));
    }
}
