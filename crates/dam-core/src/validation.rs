use crate::{DamCreation, Weekday};
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

    if creation.map.id.trim().is_empty() {
        issues.push(ValidationIssue::new("map.id", "Static map id is required."));
    }
    if creation.map.name.trim().is_empty() {
        issues.push(ValidationIssue::new(
            "map.name",
            "Static map name is required.",
        ));
    }

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
            "distribution.sectors",
            "At least one Unit/Sector must be selected.",
        ));
    }

    if creation.text.value.chars().count() > 250 {
        issues.push(ValidationIssue::new(
            "text.value",
            "Text must be 250 characters or fewer.",
        ));
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationError { issues })
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
        AltitudeCorrection, BufferFilter, DateRange, DistributionSelection, Level, LevelUnit,
        Period, SelectedStaticMap, TextInfo,
    };
    use chrono::NaiveDate;

    fn valid_creation() -> DamCreation {
        DamCreation {
            map: SelectedStaticMap {
                id: "50714".to_owned(),
                name: "HAUT VALAIS".to_owned(),
            },
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
            altitude_correction: AltitudeCorrection::None,
            upper_buffer: BufferFilter::Default,
            lower_buffer: BufferFilter::Default,
            distribution: DistributionSelection::all(),
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
}
