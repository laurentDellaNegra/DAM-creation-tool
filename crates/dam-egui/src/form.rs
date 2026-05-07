use crate::{current_date_text, parse_date, parse_level, parse_time};
use chrono::NaiveDate;
use dam_core::{
    AltitudeCorrection, BufferFilter, DamCreation, DateRange, DistributionSelection, LevelUnit,
    MapCatalog, Period, SelectedStaticMap, StaticMap, TextInfo, ValidationIssue, Weekday,
    default_distribution,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct DamFormState {
    pub selected_map_id: Option<String>,
    pub map_search: String,
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
}

impl DamFormState {
    pub fn new(_catalog: &MapCatalog) -> Self {
        let today = current_date_text();
        let mut state = Self {
            selected_map_id: None,
            map_search: String::new(),
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
            map: selected_map,
            date_range,
            periods,
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
