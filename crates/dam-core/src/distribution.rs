use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyDistribution {
    pub flags: [bool; 12],
}

pub type DistributionSelection = LegacyDistribution;

impl LegacyDistribution {
    pub const ORDER: [LegacyDistributionTarget; 12] = [
        LegacyDistributionTarget::AccUpper,
        LegacyDistributionTarget::AccLower,
        LegacyDistributionTarget::App,
        LegacyDistributionTarget::FicDelta,
        LegacyDistributionTarget::Arfa,
        LegacyDistributionTarget::TwrZurich,
        LegacyDistributionTarget::TdiBern,
        LegacyDistributionTarget::TdiBuochs,
        LegacyDistributionTarget::TdiDubendorf,
        LegacyDistributionTarget::TdiEmmen,
        LegacyDistributionTarget::TdiLugano,
        LegacyDistributionTarget::TdiStGallen,
    ];

    pub fn all() -> Self {
        Self { flags: [true; 12] }
    }

    pub fn none() -> Self {
        Self { flags: [false; 12] }
    }

    pub fn is_empty(&self) -> bool {
        !self.flags.iter().any(|selected| *selected)
    }

    pub fn is_selected(&self, target: LegacyDistributionTarget) -> bool {
        self.flags[target.index()]
    }

    pub fn set(&mut self, target: LegacyDistributionTarget, selected: bool) {
        self.flags[target.index()] = selected;
    }

    pub fn selected_count(&self) -> usize {
        self.flags.iter().filter(|selected| **selected).count()
    }

    pub fn to_legacy_flags(&self) -> String {
        self.flags
            .iter()
            .map(|selected| if *selected { "1" } else { "0" })
            .collect::<Vec<_>>()
            .join("/")
    }
}

impl Default for LegacyDistribution {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LegacyDistributionTarget {
    AccUpper,
    AccLower,
    App,
    FicDelta,
    Arfa,
    TwrZurich,
    TdiBern,
    TdiBuochs,
    TdiDubendorf,
    TdiEmmen,
    TdiLugano,
    TdiStGallen,
}

impl LegacyDistributionTarget {
    pub const ORDER: [LegacyDistributionTarget; 12] = LegacyDistribution::ORDER;

    pub fn id(self) -> &'static str {
        match self {
            Self::AccUpper => "ACC_UPPER",
            Self::AccLower => "ACC_LOWER",
            Self::App => "APP",
            Self::FicDelta => "FIC_DELTA",
            Self::Arfa => "ARFA",
            Self::TwrZurich => "TWR_ZURICH",
            Self::TdiBern => "TDI_BERN",
            Self::TdiBuochs => "TDI_BUOCHS",
            Self::TdiDubendorf => "TDI_DUBENDORF",
            Self::TdiEmmen => "TDI_EMMEN",
            Self::TdiLugano => "TDI_LUGANO",
            Self::TdiStGallen => "TDI_ST_GALLEN",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AccUpper => "ACC Upper",
            Self::AccLower => "ACC Lower",
            Self::App => "APP",
            Self::FicDelta => "FIC Delta",
            Self::Arfa => "ARFA",
            Self::TwrZurich => "TWR Zurich",
            Self::TdiBern => "TDI Bern",
            Self::TdiBuochs => "TDI Buochs",
            Self::TdiDubendorf => "TDI Dubendorf",
            Self::TdiEmmen => "TDI Emmen",
            Self::TdiLugano => "TDI Lugano",
            Self::TdiStGallen => "TDI St. Gallen",
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Self::AccUpper => 0,
            Self::AccLower => 1,
            Self::App => 2,
            Self::FicDelta => 3,
            Self::Arfa => 4,
            Self::TwrZurich => 5,
            Self::TdiBern => 6,
            Self::TdiBuochs => 7,
            Self::TdiDubendorf => 8,
            Self::TdiEmmen => 9,
            Self::TdiLugano => 10,
            Self::TdiStGallen => 11,
        }
    }
}

pub fn default_distribution() -> LegacyDistribution {
    LegacyDistribution::all()
}

pub fn legacy_distribution_from_catalog_stations(
    stations: &[String],
) -> Option<LegacyDistribution> {
    if stations.is_empty() {
        return None;
    }

    let mut distribution = LegacyDistribution::none();
    let mut matched_any = false;

    for station in stations {
        let normalized = station.trim().to_ascii_uppercase().replace(['-', ' '], "_");
        let targets: &[LegacyDistributionTarget] = match normalized.as_str() {
            "APP" => &[LegacyDistributionTarget::App],
            "FIC" | "DLT" | "DELTA" => &[LegacyDistributionTarget::FicDelta],
            "ARF" | "ARFA" => &[LegacyDistributionTarget::Arfa],
            "TWR" => &[LegacyDistributionTarget::TwrZurich],
            "BRN" => &[LegacyDistributionTarget::TdiBern],
            "BUO" => &[LegacyDistributionTarget::TdiBuochs],
            "DUB" => &[LegacyDistributionTarget::TdiDubendorf],
            "EMM" => &[LegacyDistributionTarget::TdiEmmen],
            "LUG" => &[LegacyDistributionTarget::TdiLugano],
            "STG" => &[LegacyDistributionTarget::TdiStGallen],
            "UAC" => &[LegacyDistributionTarget::AccUpper],
            _ => &[],
        };

        for target in targets {
            distribution.set(*target, true);
            matched_any = true;
        }
    }

    if matched_any && !distribution.is_empty() {
        Some(distribution)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_distribution_exports_12_selected_flags() {
        assert_eq!(
            LegacyDistribution::all().to_legacy_flags(),
            "1/1/1/1/1/1/1/1/1/1/1/1"
        );
    }

    #[test]
    fn station_defaults_map_to_legacy_order_without_emptying_unknowns() {
        let distribution =
            legacy_distribution_from_catalog_stations(&["APP".into(), "FIC".into(), "BRN".into()])
                .unwrap();

        assert_eq!(distribution.to_legacy_flags(), "0/0/1/1/0/0/1/0/0/0/0/0");
        assert!(legacy_distribution_from_catalog_stations(&["UNKNOWN".into()]).is_none());
    }
}
