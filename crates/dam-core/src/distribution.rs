use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributionSelection {
    pub sectors: BTreeSet<String>,
}

impl DistributionSelection {
    pub fn all() -> Self {
        Self {
            sectors: unit_groups()
                .iter()
                .flat_map(|group| group.sectors.iter().map(|sector| sector.id.to_owned()))
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sectors.is_empty()
    }
}

pub fn default_distribution() -> DistributionSelection {
    DistributionSelection::all()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitGroup {
    pub region: &'static str,
    pub unit: &'static str,
    pub label: &'static str,
    pub sectors: &'static [Sector],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sector {
    pub id: &'static str,
    pub label: &'static str,
}

pub fn unit_groups() -> &'static [UnitGroup] {
    &UNIT_GROUPS
}

const GVA_ACC_UPPER: &[Sector] = &[
    Sector {
        id: "GVA:UL1",
        label: "L1",
    },
    Sector {
        id: "GVA:UL2",
        label: "L2",
    },
    Sector {
        id: "GVA:UL3",
        label: "L3",
    },
    Sector {
        id: "GVA:UL4",
        label: "L4",
    },
    Sector {
        id: "GVA:UL5",
        label: "L5",
    },
    Sector {
        id: "GVA:UL6",
        label: "L6",
    },
];

const GVA_ACC_LOWER: &[Sector] = &[
    Sector {
        id: "GVA:INN",
        label: "INN",
    },
    Sector {
        id: "GVA:INS",
        label: "INS",
    },
    Sector {
        id: "GVA:INE",
        label: "INE",
    },
    Sector {
        id: "GVA:INL",
        label: "INL",
    },
];

const GVA_APP: &[Sector] = &[
    Sector {
        id: "GVA:ARR",
        label: "ARR",
    },
    Sector {
        id: "GVA:FIN",
        label: "FIN",
    },
    Sector {
        id: "GVA:DEP",
        label: "DEP",
    },
    Sector {
        id: "GVA:PRN",
        label: "PRN",
    },
];

const GVA_MIL_DLT_FIC: &[Sector] = &[
    Sector {
        id: "GVA:MIL",
        label: "MIL",
    },
    Sector {
        id: "GVA:DLT",
        label: "DLT",
    },
    Sector {
        id: "GVA:FIC",
        label: "FIC",
    },
];

const GVA_SPVR_FMP: &[Sector] = &[
    Sector {
        id: "GVA:SPVR",
        label: "SPVR",
    },
    Sector {
        id: "GVA:FMP",
        label: "FMP",
    },
];

const ZRH_ACC_UPPER: &[Sector] = &[
    Sector {
        id: "ZRH:UM1",
        label: "M1",
    },
    Sector {
        id: "ZRH:UM2",
        label: "M2",
    },
    Sector {
        id: "ZRH:UM3",
        label: "M3",
    },
    Sector {
        id: "ZRH:UM4",
        label: "M4",
    },
    Sector {
        id: "ZRH:UM5",
        label: "M5",
    },
    Sector {
        id: "ZRH:UM6",
        label: "M6",
    },
];

const ZRH_ACC_LOWER: &[Sector] = &[
    Sector {
        id: "ZRH:LOW",
        label: "W",
    },
    Sector {
        id: "ZRH:LOS",
        label: "S",
    },
    Sector {
        id: "ZRH:LOE",
        label: "E",
    },
    Sector {
        id: "ZRH:LON",
        label: "N",
    },
];

const ZRH_APP: &[Sector] = &[
    Sector {
        id: "ZRH:APW",
        label: "APW",
    },
    Sector {
        id: "ZRH:FIN",
        label: "FIN",
    },
    Sector {
        id: "ZRH:APE",
        label: "APE",
    },
    Sector {
        id: "ZRH:CAP",
        label: "CAP",
    },
    Sector {
        id: "ZRH:DEP",
        label: "DEP",
    },
    Sector {
        id: "ZRH:RSV",
        label: "RSV",
    },
    Sector {
        id: "ZRH:PRN",
        label: "PRN",
    },
];

const ZRH_ARFA_DLT_FIC: &[Sector] = &[
    Sector {
        id: "ZRH:ARFA",
        label: "ARFA",
    },
    Sector {
        id: "ZRH:FIC",
        label: "FIC",
    },
    Sector {
        id: "ZRH:DLT",
        label: "DLT",
    },
];

const ZRH_SPVR_FMP: &[Sector] = &[
    Sector {
        id: "ZRH:SPVR",
        label: "SPVR",
    },
    Sector {
        id: "ZRH:FMP",
        label: "FMP",
    },
];

const UNIT_GROUPS: [UnitGroup; 10] = [
    UnitGroup {
        region: "Geneva",
        unit: "ACC_UPPER",
        label: "ACC - upper",
        sectors: GVA_ACC_UPPER,
    },
    UnitGroup {
        region: "Geneva",
        unit: "ACC_LOWER",
        label: "ACC - lower",
        sectors: GVA_ACC_LOWER,
    },
    UnitGroup {
        region: "Geneva",
        unit: "APP",
        label: "APP",
        sectors: GVA_APP,
    },
    UnitGroup {
        region: "Geneva",
        unit: "MIL_DLT_FIC",
        label: "MIL/DLT/FIC",
        sectors: GVA_MIL_DLT_FIC,
    },
    UnitGroup {
        region: "Geneva",
        unit: "SPVR_FMP",
        label: "SPVR/FMP",
        sectors: GVA_SPVR_FMP,
    },
    UnitGroup {
        region: "Zurich",
        unit: "ACC_UPPER",
        label: "ACC - upper",
        sectors: ZRH_ACC_UPPER,
    },
    UnitGroup {
        region: "Zurich",
        unit: "ACC_LOWER",
        label: "ACC - lower",
        sectors: ZRH_ACC_LOWER,
    },
    UnitGroup {
        region: "Zurich",
        unit: "APP",
        label: "APP",
        sectors: ZRH_APP,
    },
    UnitGroup {
        region: "Zurich",
        unit: "ARFA_DLT_FIC",
        label: "ARFA/DLT/FIC",
        sectors: ZRH_ARFA_DLT_FIC,
    },
    UnitGroup {
        region: "Zurich",
        unit: "SPVR_FMP",
        label: "SPVR/FMP",
        sectors: ZRH_SPVR_FMP,
    },
];
