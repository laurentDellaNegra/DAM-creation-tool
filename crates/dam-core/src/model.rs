use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamCreation {
    pub map: SelectedStaticMap,
    pub date_range: DateRange,
    pub periods: Vec<Period>,
    pub altitude_correction: AltitudeCorrection,
    pub upper_buffer: BufferFilter,
    pub lower_buffer: BufferFilter,
    pub distribution: crate::DistributionSelection,
    pub text: TextInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedStaticMap {
    pub id: String,
    pub name: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AltitudeCorrection {
    None,
    QnhCorr,
    FlCorr,
}

impl Default for AltitudeCorrection {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BufferFilter {
    Default,
    Half,
    NoBuffer,
}

impl Default for BufferFilter {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInfo {
    pub value: String,
    pub display: bool,
}

impl Default for TextInfo {
    fn default() -> Self {
        Self {
            value: String::new(),
            display: false,
        }
    }
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
