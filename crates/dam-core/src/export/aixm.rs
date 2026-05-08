use crate::{
    Coordinate, DamCreation, DamMap, Level, LevelUnit, ManualGeometry, ManualMap, ValidationError,
    expand_polygon_nodes, validate_creation,
};
use chrono::{NaiveDate, NaiveTime};
use thiserror::Error;

const DESIGNATOR: &str = "UAV";
const STATIC_FALLBACK_LABEL: Coordinate = Coordinate {
    lon: 8.168292,
    lat: 46.92617,
};
const STATIC_FALLBACK_GEOMETRY: [Coordinate; 13] = [
    Coordinate {
        lon: 8.340801,
        lat: 46.929693,
    },
    Coordinate {
        lon: 8.327222,
        lat: 46.964592,
    },
    Coordinate {
        lon: 8.160663,
        lat: 47.051113,
    },
    Coordinate {
        lon: 8.02433,
        lat: 47.048413,
    },
    Coordinate {
        lon: 8.003026,
        lat: 47.041616,
    },
    Coordinate {
        lon: 7.596441,
        lat: 46.821635,
    },
    Coordinate {
        lon: 7.667749,
        lat: 46.739194,
    },
    Coordinate {
        lon: 7.958477,
        lat: 46.75625,
    },
    Coordinate {
        lon: 8.014441,
        lat: 46.787578,
    },
    Coordinate {
        lon: 8.040969,
        lat: 46.787612,
    },
    Coordinate {
        lon: 8.431757,
        lat: 46.721279,
    },
    Coordinate {
        lon: 8.399366,
        lat: 46.941046,
    },
    Coordinate {
        lon: 8.340801,
        lat: 46.929693,
    },
];

#[derive(Debug, Error)]
pub enum AixmExportError {
    #[error("validation failed")]
    Validation(#[from] ValidationError),
    #[error("AIXM export supports exactly one activation period for now, got {count}")]
    UnsupportedPeriodCount { count: usize },
    #[error("AIXM export supports manual polygon geometry only for now, got {geometry}")]
    UnsupportedManualGeometry { geometry: &'static str },
    #[error("AIXM export does not support manual lateral buffers yet")]
    UnsupportedLateralBuffer,
    #[error("AIXM export requires at least 3 geometry coordinates")]
    MissingGeometry,
    #[error("static map id must be numeric for AIXM export: {id}")]
    InvalidMapId { id: String },
}

struct AixmMapData {
    map_id: String,
    name: String,
    geometry: Vec<Coordinate>,
    label_position: Coordinate,
}

pub fn to_aixm_xml(creation: &DamCreation) -> Result<String, AixmExportError> {
    validate_creation(creation)?;

    if creation.periods.len() != 1 {
        return Err(AixmExportError::UnsupportedPeriodCount {
            count: creation.periods.len(),
        });
    }

    let period = &creation.periods[0];
    let map = aixm_map_data(&creation.map)?;
    let begin_position = format_datetime_utc(creation.date_range.start, period.start_time);
    let start_date = format_date(creation.date_range.start);
    let end_date = format_date(creation.date_range.end);
    let start_time = format_time(period.start_time);
    let end_time = format_time(period.end_time);
    let map_information = build_map_information();
    let geometry = format_pos_list(&map.geometry);
    let label_position = format_position(map.label_position);

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<asn:airspaceStatusNotificationMessage
    xmlns:asn="http://www.skyguide.ch/cmm/atm/environment/aeronautical/airspace/statusanotification/v1"
    xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.skyguide.ch/cmm/atm/environment/aeronautical/airspace/statusanotification/v1 ./Schemas/AirspaceStatusNotificationMessage_v1_0.xsd">

    <asn:airspace gml:id="id1" xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
                  xmlns:gml="http://www.opengis.net/gml/3.2">
        <aixm:timeSlice>
            <aixm:AirspaceTimeSlice gml:id="id2">
                <gml:validTime>
                    <gml:TimePeriod gml:id="id3">
                        <gml:beginPosition>{begin_position}</gml:beginPosition>
                        <gml:endPosition/>
                    </gml:TimePeriod>
                </gml:validTime>
                <aixm:interpretation>BASELINE</aixm:interpretation>
                <aixm:sequenceNumber>0</aixm:sequenceNumber>
                <aixm:correctionNumber>0</aixm:correctionNumber>
                <aixm:type>OTHER</aixm:type>
                <aixm:designator>{designator}</aixm:designator>
                <aixm:name>{name}</aixm:name>
                <aixm:geometryComponent>
                    <aixm:AirspaceGeometryComponent gml:id="id4">
                        <aixm:theAirspaceVolume>
                            <aixm:AirspaceVolume gml:id="id5">
                                <aixm:horizontalProjection>
                                    <aixm:Surface gml:id="id6">
                                        <gml:patches>
                                            <gml:PolygonPatch>
                                                <gml:exterior>
                                                    <gml:Ring>
                                                        <gml:curveMember>
                                                            <gml:Curve gml:id="id7">
                                                                <gml:segments>
                                                                    <gml:GeodesicString>
                                                                        <gml:posList>{geometry}</gml:posList>
                                                                    </gml:GeodesicString>
                                                                </gml:segments>
                                                            </gml:Curve>
                                                        </gml:curveMember>
                                                    </gml:Ring>
                                                </gml:exterior>
                                            </gml:PolygonPatch>
                                        </gml:patches>
                                    </aixm:Surface>
                                </aixm:horizontalProjection>
                            </aixm:AirspaceVolume>
                        </aixm:theAirspaceVolume>
                    </aixm:AirspaceGeometryComponent>
                </aixm:geometryComponent>
                <aixm:activation>
                    <aixm:AirspaceActivation gml:id="id8">
                        <aixm:timeInterval>
                            <aixm:Timesheet gml:id="id9">
                                <aixm:timeReference>UTC</aixm:timeReference>
                                <aixm:startDate>{start_date}</aixm:startDate>
                                <aixm:endDate>{end_date}</aixm:endDate>
                                <aixm:startTime>{start_time}</aixm:startTime>
                                <aixm:endTime>{end_time}</aixm:endTime>
                                <aixm:daylightSavingAdjust>NO</aixm:daylightSavingAdjust>
                                <aixm:extension>
                                    <ext:DamTimesheetExtension gml:id="id10"
                                                               xmlns:ext="http://www.skyguide.ch/aixm/v5.1.1/sg/5.0"
                                                               xsi:schemaLocation="http://www.skyguide.ch/aixm/v5.1.1/sg/5.0 ./Schemas/sgaixm_dynasp_features_v51_0.xsd">
                                        <ext:displayBeginTime>{display_begin}</ext:displayBeginTime>
                                        <ext:displayEndTime>{display_end}</ext:displayEndTime>
                                        <ext:highestLevel uom="{highest_uom}">{highest_level}</ext:highestLevel>
                                        <ext:highestLevelReference>STD</ext:highestLevelReference>
                                        <ext:lowestLevel uom="{lowest_uom}">{lowest_level}</ext:lowestLevel>
                                        <ext:lowestLevelReference>STD</ext:lowestLevelReference>
                                    </ext:DamTimesheetExtension>
                                </aixm:extension>
                            </aixm:Timesheet>
                        </aixm:timeInterval>
                    </aixm:AirspaceActivation>
                </aixm:activation>
                <aixm:extension>
                    <ext:DynamicAirspaceExtension gml:id="id11" xmlns:ext="http://www.skyguide.ch/aixm/v5.1.1/sg/5.0"
                                                  xsi:schemaLocation="http://www.skyguide.ch/aixm/v5.1.1/sg/5.0 ./Schemas/sgaixm_dynasp_features_v51_0.xsd">
                        <ext:displayLevels>{display_levels}</ext:displayLevels>
                        <ext:displayPositionLevelIndication>
                            <gml:pos>{label_position}</gml:pos>
                        </ext:displayPositionLevelIndication>
                        <ext:flightLevelCorrection>0</ext:flightLevelCorrection>
                        <ext:mapHasBeenDeleted>NO</ext:mapHasBeenDeleted>
                        <ext:mapId>{map_id}</ext:mapId>
                        <ext:mapInformation>{map_information}</ext:mapInformation>
                        <ext:sliceId>0</ext:sliceId>
                    </ext:DynamicAirspaceExtension>
                </aixm:extension>
            </aixm:AirspaceTimeSlice>
        </aixm:timeSlice>
    </asn:airspace>

</asn:airspaceStatusNotificationMessage>
"#,
        begin_position = escape_xml(&begin_position),
        designator = DESIGNATOR,
        name = escape_xml(&map.name),
        geometry = geometry,
        start_date = escape_xml(&start_date),
        end_date = escape_xml(&end_date),
        start_time = escape_xml(&start_time),
        end_time = escape_xml(&end_time),
        display_begin = yes_no(period.start_indication),
        display_end = yes_no(period.end_indication),
        highest_uom = level_uom(period.upper),
        highest_level = format_level(period.upper),
        lowest_uom = level_uom(period.lower),
        lowest_level = format_level(period.lower),
        display_levels = yes_no(creation.display_levels),
        label_position = label_position,
        map_id = escape_xml(&map.map_id),
        map_information = escape_xml(&map_information),
    ))
}

fn aixm_map_data(map: &DamMap) -> Result<AixmMapData, AixmExportError> {
    match map {
        DamMap::Predefined(selected) => {
            let map_id = selected.id.trim();
            if map_id.parse::<u32>().is_err() {
                return Err(AixmExportError::InvalidMapId {
                    id: selected.id.clone(),
                });
            }

            Ok(AixmMapData {
                map_id: map_id.to_owned(),
                name: selected.name.clone(),
                geometry: STATIC_FALLBACK_GEOMETRY.to_vec(),
                label_position: STATIC_FALLBACK_LABEL,
            })
        }
        DamMap::Manual(manual) => Ok(AixmMapData {
            map_id: "0".to_owned(),
            name: manual.name.clone(),
            geometry: manual_geometry(manual)?,
            label_position: manual
                .label_position
                .ok_or(AixmExportError::MissingGeometry)?,
        }),
    }
}

fn manual_geometry(manual: &ManualMap) -> Result<Vec<Coordinate>, AixmExportError> {
    if manual.attributes.lateral_buffer_nm > 0.0 {
        return Err(AixmExportError::UnsupportedLateralBuffer);
    }

    let ManualGeometry::Polygon { nodes } = &manual.geometry else {
        return Err(AixmExportError::UnsupportedManualGeometry {
            geometry: manual_geometry_name(&manual.geometry),
        });
    };

    let mut coordinates = expand_polygon_nodes(nodes);
    if coordinates.len() < 3 {
        return Err(AixmExportError::MissingGeometry);
    }

    close_ring(&mut coordinates);
    Ok(coordinates)
}

fn manual_geometry_name(geometry: &ManualGeometry) -> &'static str {
    match geometry {
        ManualGeometry::Polygon { .. } => "polygon",
        ManualGeometry::ParaSymbol { .. } => "para_symbol",
        ManualGeometry::TextNumber { .. } => "text_number",
        ManualGeometry::PieCircle { .. } => "pie_circle",
        ManualGeometry::Strip { .. } => "strip",
    }
}

fn close_ring(coordinates: &mut Vec<Coordinate>) {
    let Some(first) = coordinates.first().copied() else {
        return;
    };
    let Some(last) = coordinates.last().copied() else {
        return;
    };
    if (first.lon - last.lon).abs() > f64::EPSILON || (first.lat - last.lat).abs() > f64::EPSILON {
        coordinates.push(first);
    }
}

fn format_datetime_utc(date: NaiveDate, time: NaiveTime) -> String {
    format!(
        "{}T{}.000Z",
        date.format("%Y-%m-%d"),
        time.format("%H:%M:%S")
    )
}

fn format_date(date: NaiveDate) -> String {
    date.format("%d-%m").to_string()
}

fn format_time(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

fn format_pos_list(coordinates: &[Coordinate]) -> String {
    coordinates
        .iter()
        .map(|coordinate| format_position(*coordinate))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_position(coordinate: Coordinate) -> String {
    format!(
        "{} {}",
        format_decimal(coordinate.lon),
        format_decimal(coordinate.lat)
    )
}

fn format_decimal(value: f64) -> String {
    let value = if value == 0.0 { 0.0 } else { value };
    let formatted = format!("{value:.6}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn level_uom(level: Level) -> &'static str {
    match level.unit {
        LevelUnit::FlightLevel => "FL",
        LevelUnit::Feet => "FT",
    }
}

fn format_level(level: Level) -> String {
    match level.unit {
        LevelUnit::FlightLevel => format!("{:03}", level.value),
        LevelUnit::Feet => level.value.to_string(),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "YES" } else { "NO" }
}

fn build_map_information() -> String {
    "src=ZRH;lvl=140:0;dist=1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1;qnh=0;flc=0;ulh=0;llh=0;uln=0;lln=0;txt=;".to_owned()
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AltitudeCorrection, BufferFilter, DateRange, DistributionSelection, Level, LevelUnit,
        ManualMapAttributes, ManualMapCategory, ManualMapRendering, Period, PolygonNode,
        SelectedStaticMap, TextInfo,
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
                end_indication: false,
                end_time: NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
                lower: Level::new(0, LevelUnit::FlightLevel),
                upper: Level::new(140, LevelUnit::FlightLevel),
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

    fn manual_polygon_creation() -> DamCreation {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(ManualMap {
            name: "Manual <DAM>".to_owned(),
            geometry: ManualGeometry::Polygon {
                nodes: vec![
                    PolygonNode::point(Coordinate {
                        lon: 7.069034,
                        lat: 46.538691,
                    }),
                    PolygonNode::point(Coordinate {
                        lon: 7.278337,
                        lat: 46.611854,
                    }),
                    PolygonNode::point(Coordinate {
                        lon: 7.375914,
                        lat: 46.552703,
                    }),
                ],
            },
            attributes: ManualMapAttributes {
                category: ManualMapCategory::Danger,
                rendering: ManualMapRendering::Surface,
                lateral_buffer_nm: 0.0,
            },
            label_position: Some(Coordinate {
                lon: 7.154023,
                lat: 46.512928,
            }),
        });
        creation
    }

    #[test]
    fn aixm_export_validates_before_generation() {
        let mut creation = valid_creation();
        creation.distribution.sectors.clear();

        let error = to_aixm_xml(&creation).unwrap_err();

        assert!(matches!(error, AixmExportError::Validation(_)));
    }

    #[test]
    fn aixm_export_generates_static_map_xml_with_fallback_geometry() {
        let xml = to_aixm_xml(&valid_creation()).unwrap();

        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\""));
        assert!(xml.contains("<aixm:designator>UAV</aixm:designator>"));
        assert!(xml.contains("<aixm:name>HAUT VALAIS</aixm:name>"));
        assert!(xml.contains("<gml:beginPosition>2026-05-07T09:00:00.000Z</gml:beginPosition>"));
        assert!(xml.contains("<gml:endPosition/>"));
        assert!(xml.contains("<aixm:startDate>07-05</aixm:startDate>"));
        assert!(xml.contains("<aixm:endDate>08-05</aixm:endDate>"));
        assert!(xml.contains("<aixm:startTime>09:00</aixm:startTime>"));
        assert!(xml.contains("<aixm:endTime>10:30</aixm:endTime>"));
        assert!(xml.contains("<ext:displayBeginTime>YES</ext:displayBeginTime>"));
        assert!(xml.contains("<ext:displayEndTime>NO</ext:displayEndTime>"));
        assert!(xml.contains(r#"<ext:highestLevel uom="FL">140</ext:highestLevel>"#));
        assert!(xml.contains(r#"<ext:lowestLevel uom="FL">000</ext:lowestLevel>"#));
        assert!(xml.contains("<ext:mapId>50714</ext:mapId>"));
        assert!(xml.contains("<ext:sliceId>0</ext:sliceId>"));
        assert!(xml.contains("<gml:posList>8.340801 46.929693 8.327222 46.964592"));
        assert!(xml.contains("<gml:pos>8.168292 46.92617</gml:pos>"));
        assert!(xml.contains(
            "<ext:mapInformation>src=ZRH;lvl=140:0;dist=1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1;qnh=0;flc=0;ulh=0;llh=0;uln=0;lln=0;txt=;</ext:mapInformation>"
        ));
    }

    #[test]
    fn aixm_export_generates_manual_polygon_xml_with_lon_lat_pos_list() {
        let xml = to_aixm_xml(&manual_polygon_creation()).unwrap();

        assert!(xml.contains("<aixm:name>Manual &lt;DAM&gt;</aixm:name>"));
        assert!(xml.contains("<ext:mapId>0</ext:mapId>"));
        assert!(xml.contains("<gml:posList>7.069034 46.538691 7.278337 46.611854 7.375914 46.552703 7.069034 46.538691</gml:posList>"));
        assert!(xml.contains("<gml:pos>7.154023 46.512928</gml:pos>"));
    }

    #[test]
    fn aixm_export_rejects_multiple_periods_for_now() {
        let mut creation = valid_creation();
        creation.periods.push(Period {
            start_indication: true,
            start_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            end_indication: true,
            end_time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            lower: Level::new(0, LevelUnit::FlightLevel),
            upper: Level::new(140, LevelUnit::FlightLevel),
        });

        let error = to_aixm_xml(&creation).unwrap_err();

        assert!(matches!(
            error,
            AixmExportError::UnsupportedPeriodCount { count: 2 }
        ));
    }

    #[test]
    fn aixm_export_rejects_unsupported_manual_geometry() {
        let mut creation = manual_polygon_creation();
        if let DamMap::Manual(manual) = &mut creation.map {
            manual.geometry = ManualGeometry::ParaSymbol {
                point: Some(Coordinate {
                    lon: 7.0,
                    lat: 46.0,
                }),
            };
        }

        let error = to_aixm_xml(&creation).unwrap_err();

        assert!(matches!(
            error,
            AixmExportError::UnsupportedManualGeometry {
                geometry: "para_symbol"
            }
        ));
    }

    #[test]
    fn aixm_export_rejects_manual_lateral_buffer_for_now() {
        let mut creation = manual_polygon_creation();
        if let DamMap::Manual(manual) = &mut creation.map {
            manual.attributes.lateral_buffer_nm = 1.0;
        }

        let error = to_aixm_xml(&creation).unwrap_err();

        assert!(matches!(error, AixmExportError::UnsupportedLateralBuffer));
    }
}
