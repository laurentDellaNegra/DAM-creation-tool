use crate::{
    Coordinate, DamCreation, DamMap, DateRange, Level, LevelUnit, ManualGeometry, ManualMap,
    MapCatalog, Period, PolygonNode, SelectedStaticMap, ValidationError, ValidationIssue,
    expand_polygon_nodes, validate_creation,
};
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use std::collections::BTreeSet;
use std::fmt;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AixmImportError {
    pub issues: Vec<ValidationIssue>,
}

impl fmt::Display for AixmImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} AIXM import issue(s)", self.issues.len())
    }
}

impl std::error::Error for AixmImportError {}

#[derive(Debug, Clone, PartialEq)]
pub struct AixmXmlSummary {
    pub map_id: String,
    pub map_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub display_begin_time: bool,
    pub display_end_time: bool,
    pub lower_level: Level,
    pub upper_level: Level,
    pub display_levels: bool,
    pub geometry_points: usize,
    pub label_position: Coordinate,
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

pub fn aixm_xml_well_formed(xml: &str) -> Result<(), AixmImportError> {
    roxmltree::Document::parse(xml)
        .map(|_| ())
        .map_err(|error| import_error("aixm.xml", format!("XML is not well formed: {error}")))
}

pub fn summarize_aixm_xml(xml: &str) -> Result<AixmXmlSummary, AixmImportError> {
    parse_aixm_payload(xml).map(|payload| payload.summary())
}

pub fn apply_aixm_xml_update(
    base: &DamCreation,
    catalog: &MapCatalog,
    xml: &str,
) -> Result<DamCreation, AixmImportError> {
    let parsed = parse_aixm_payload(xml)?;
    let mut candidate = base.clone();

    candidate.date_range =
        date_range_with_preserved_weekdays(&base.date_range, parsed.start_date, parsed.end_date);
    candidate.periods = vec![Period {
        start_indication: parsed.display_begin_time,
        start_time: parsed.start_time,
        end_indication: parsed.display_end_time,
        end_time: parsed.end_time,
        lower: parsed.lower_level,
        upper: parsed.upper_level,
    }];
    candidate.display_levels = parsed.display_levels;

    candidate.map = match &base.map {
        DamMap::Predefined(_) => {
            let catalog_map = catalog.selected(parsed.map_id.trim()).ok_or_else(|| {
                import_error(
                    "map.id",
                    format!(
                        "AIXM mapId '{}' is not present in the bundled catalog.",
                        parsed.map_id
                    ),
                )
            })?;
            if parsed.map_name.trim() != catalog_map.name {
                return Err(import_error(
                    "map.name",
                    format!(
                        "AIXM name must match catalog map '{}', got '{}'.",
                        catalog_map.name, parsed.map_name
                    ),
                ));
            }

            let expected_map = DamMap::Predefined(SelectedStaticMap {
                id: catalog_map.id.clone(),
                name: catalog_map.name.clone(),
            });
            let expected = aixm_map_data(&expected_map).map_err(aixm_export_as_import_error)?;
            let mut expected_geometry = expected.geometry;
            trim_closing_coordinate(&mut expected_geometry);
            if !coordinates_match(&parsed.geometry, &expected_geometry) {
                return Err(import_error(
                    "map.geometry",
                    "Predefined map geometry cannot be edited from the AIXM preview.",
                ));
            }
            if !coordinate_matches(parsed.label_position, expected.label_position) {
                return Err(import_error(
                    "map.label_position",
                    "Predefined map label position cannot be edited from the AIXM preview.",
                ));
            }

            expected_map
        }
        DamMap::Manual(manual) => {
            if parsed.map_id.trim() != "0" {
                return Err(import_error(
                    "map.id",
                    "Manual AIXM preview must keep mapId equal to 0.",
                ));
            }
            if !matches!(manual.geometry, ManualGeometry::Polygon { .. }) {
                return Err(import_error(
                    "map.geometry",
                    "AIXM preview save supports manual polygon geometry only.",
                ));
            }
            if parsed.geometry.len() < 3 {
                return Err(import_error(
                    "map.geometry",
                    "AIXM polygon requires at least 3 coordinates.",
                ));
            }

            let mut updated = manual.clone();
            updated.name = parsed.map_name.clone();
            updated.geometry = ManualGeometry::Polygon {
                nodes: parsed
                    .geometry
                    .iter()
                    .copied()
                    .map(PolygonNode::point)
                    .collect(),
            };
            updated.label_position = Some(parsed.label_position);
            DamMap::Manual(updated)
        }
    };

    Ok(candidate)
}

#[derive(Debug, Clone)]
struct ParsedAixmPayload {
    map_id: String,
    map_name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    start_time: NaiveTime,
    end_time: NaiveTime,
    display_begin_time: bool,
    display_end_time: bool,
    lower_level: Level,
    upper_level: Level,
    display_levels: bool,
    geometry: Vec<Coordinate>,
    label_position: Coordinate,
}

impl ParsedAixmPayload {
    fn summary(&self) -> AixmXmlSummary {
        AixmXmlSummary {
            map_id: self.map_id.clone(),
            map_name: self.map_name.clone(),
            start_date: self.start_date,
            end_date: self.end_date,
            start_time: self.start_time,
            end_time: self.end_time,
            display_begin_time: self.display_begin_time,
            display_end_time: self.display_end_time,
            lower_level: self.lower_level,
            upper_level: self.upper_level,
            display_levels: self.display_levels,
            geometry_points: self.geometry.len(),
            label_position: self.label_position,
        }
    }
}

fn parse_aixm_payload(xml: &str) -> Result<ParsedAixmPayload, AixmImportError> {
    let doc = roxmltree::Document::parse(xml)
        .map_err(|error| import_error("aixm.xml", format!("XML is not well formed: {error}")))?;

    let timesheet_count = doc
        .descendants()
        .filter(|node| is_element(*node, "Timesheet"))
        .count();
    if timesheet_count != 1 {
        return Err(import_error(
            "periods",
            format!("AIXM preview supports exactly one Timesheet, got {timesheet_count}."),
        ));
    }

    let begin_position = required_text(&doc, "beginPosition", "gml:beginPosition")?;
    let (start_date, begin_time) = parse_begin_position(&begin_position)?;
    let start_date_dm = parse_day_month_text(
        &required_text(&doc, "startDate", "aixm:startDate")?,
        "aixm:startDate",
    )?;
    if start_date_dm != (start_date.day(), start_date.month()) {
        return Err(import_error(
            "aixm:startDate",
            "AIXM startDate must match the day/month from gml:beginPosition.",
        ));
    }

    let end_date = infer_end_date(
        start_date,
        parse_day_month_text(
            &required_text(&doc, "endDate", "aixm:endDate")?,
            "aixm:endDate",
        )?,
    )?;
    let start_time = parse_hm_time(
        &required_text(&doc, "startTime", "aixm:startTime")?,
        "aixm:startTime",
    )?;
    if start_time != begin_time {
        return Err(import_error(
            "aixm:startTime",
            "AIXM startTime must match gml:beginPosition time.",
        ));
    }
    let end_time = parse_hm_time(
        &required_text(&doc, "endTime", "aixm:endTime")?,
        "aixm:endTime",
    )?;

    let highest = required_element(&doc, "highestLevel", "ext:highestLevel")?;
    let lowest = required_element(&doc, "lowestLevel", "ext:lowestLevel")?;
    let upper_level = parse_level_element(highest, "ext:highestLevel")?;
    let lower_level = parse_level_element(lowest, "ext:lowestLevel")?;

    let mut geometry = parse_pos_list(
        &required_text(&doc, "posList", "gml:posList")?,
        "gml:posList",
    )?;
    trim_closing_coordinate(&mut geometry);

    let label_parent = required_element(
        &doc,
        "displayPositionLevelIndication",
        "ext:displayPositionLevelIndication",
    )?;
    let label_position = parse_position(
        &required_descendant_text(
            label_parent,
            "pos",
            "ext:displayPositionLevelIndication/gml:pos",
        )?,
        "ext:displayPositionLevelIndication/gml:pos",
    )?;

    Ok(ParsedAixmPayload {
        map_id: required_text(&doc, "mapId", "ext:mapId")?,
        map_name: required_text(&doc, "name", "aixm:name")?,
        start_date,
        end_date,
        start_time,
        end_time,
        display_begin_time: parse_yes_no(
            &required_text(&doc, "displayBeginTime", "ext:displayBeginTime")?,
            "ext:displayBeginTime",
        )?,
        display_end_time: parse_yes_no(
            &required_text(&doc, "displayEndTime", "ext:displayEndTime")?,
            "ext:displayEndTime",
        )?,
        lower_level,
        upper_level,
        display_levels: parse_yes_no(
            &required_text(&doc, "displayLevels", "ext:displayLevels")?,
            "ext:displayLevels",
        )?,
        geometry,
        label_position,
    })
}

fn import_error(field: impl Into<String>, message: impl Into<String>) -> AixmImportError {
    AixmImportError {
        issues: vec![ValidationIssue {
            field: field.into(),
            message: message.into(),
        }],
    }
}

fn aixm_export_as_import_error(error: AixmExportError) -> AixmImportError {
    import_error("aixm", error.to_string())
}

fn is_element(node: roxmltree::Node<'_, '_>, local_name: &str) -> bool {
    node.is_element() && node.tag_name().name() == local_name
}

fn required_element<'a, 'input>(
    doc: &'a roxmltree::Document<'input>,
    local_name: &str,
    field: &str,
) -> Result<roxmltree::Node<'a, 'input>, AixmImportError> {
    doc.descendants()
        .find(|node| is_element(*node, local_name))
        .ok_or_else(|| import_error(field, format!("Missing required AIXM element {field}.")))
}

fn required_text(
    doc: &roxmltree::Document<'_>,
    local_name: &str,
    field: &str,
) -> Result<String, AixmImportError> {
    node_text(required_element(doc, local_name, field)?, field)
}

fn required_descendant_text(
    parent: roxmltree::Node<'_, '_>,
    local_name: &str,
    field: &str,
) -> Result<String, AixmImportError> {
    let node = parent
        .descendants()
        .find(|node| is_element(*node, local_name))
        .ok_or_else(|| import_error(field, format!("Missing required AIXM element {field}.")))?;
    node_text(node, field)
}

fn node_text(node: roxmltree::Node<'_, '_>, field: &str) -> Result<String, AixmImportError> {
    let value = node.text().unwrap_or_default().trim();
    if value.is_empty() {
        Err(import_error(
            field,
            format!("AIXM element {field} is empty."),
        ))
    } else {
        Ok(value.to_owned())
    }
}

fn parse_begin_position(value: &str) -> Result<(NaiveDate, NaiveTime), AixmImportError> {
    let Some(utc_value) = value.trim().strip_suffix('Z') else {
        return Err(import_error(
            "gml:beginPosition",
            "gml:beginPosition must be UTC and end with Z.",
        ));
    };
    let Some((date, time)) = utc_value.split_once('T') else {
        return Err(import_error(
            "gml:beginPosition",
            "gml:beginPosition must use YYYY-MM-DDTHH:MM:SSZ.",
        ));
    };
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        import_error(
            "gml:beginPosition",
            "gml:beginPosition date must use YYYY-MM-DD.",
        )
    })?;
    let time = time.split('.').next().unwrap_or(time);
    let time = NaiveTime::parse_from_str(time, "%H:%M:%S").map_err(|_| {
        import_error(
            "gml:beginPosition",
            "gml:beginPosition time must use HH:MM:SS.",
        )
    })?;
    if time.second() != 0 {
        return Err(import_error(
            "gml:beginPosition",
            "gml:beginPosition seconds must be 00 for DAM preview editing.",
        ));
    }
    Ok((
        date,
        NaiveTime::from_hms_opt(time.hour(), time.minute(), 0).unwrap(),
    ))
}

fn parse_day_month_text(value: &str, field: &str) -> Result<(u32, u32), AixmImportError> {
    let Some((day, month)) = value.trim().split_once('-') else {
        return Err(import_error(field, format!("{field} must use DD-MM.")));
    };
    let day = day
        .parse::<u32>()
        .map_err(|_| import_error(field, format!("{field} day must be numeric.")))?;
    let month = month
        .parse::<u32>()
        .map_err(|_| import_error(field, format!("{field} month must be numeric.")))?;
    Ok((day, month))
}

fn infer_end_date(start: NaiveDate, day_month: (u32, u32)) -> Result<NaiveDate, AixmImportError> {
    let (day, month) = day_month;
    let mut end = NaiveDate::from_ymd_opt(start.year(), month, day)
        .or_else(|| NaiveDate::from_ymd_opt(start.year() + 1, month, day))
        .ok_or_else(|| import_error("aixm:endDate", "AIXM endDate is not a valid date."))?;
    if end < start {
        end = NaiveDate::from_ymd_opt(start.year() + 1, month, day)
            .ok_or_else(|| import_error("aixm:endDate", "AIXM endDate is not a valid date."))?;
    }
    Ok(end)
}

fn parse_hm_time(value: &str, field: &str) -> Result<NaiveTime, AixmImportError> {
    NaiveTime::parse_from_str(value.trim(), "%H:%M")
        .map_err(|_| import_error(field, format!("{field} must use HH:MM.")))
}

fn parse_yes_no(value: &str, field: &str) -> Result<bool, AixmImportError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "YES" => Ok(true),
        "NO" => Ok(false),
        _ => Err(import_error(field, format!("{field} must be YES or NO."))),
    }
}

fn parse_level_element(
    node: roxmltree::Node<'_, '_>,
    field: &str,
) -> Result<Level, AixmImportError> {
    let value = node_text(node, field)?;
    let value = value
        .parse::<u32>()
        .map_err(|_| import_error(field, format!("{field} must contain a numeric level.")))?;
    let unit = match node
        .attribute("uom")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "fl" => LevelUnit::FlightLevel,
        "ft" => LevelUnit::Feet,
        _ => {
            return Err(import_error(
                field,
                format!("{field} must declare uom=\"FL\" or uom=\"ft\"."),
            ));
        }
    };
    Ok(Level::new(value, unit))
}

fn parse_pos_list(value: &str, field: &str) -> Result<Vec<Coordinate>, AixmImportError> {
    let values = value
        .split_whitespace()
        .map(|part| {
            part.parse::<f64>()
                .map_err(|_| import_error(field, format!("{field} contains a non-numeric value.")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() % 2 != 0 {
        return Err(import_error(
            field,
            format!("{field} must contain lon/lat pairs."),
        ));
    }
    let mut coordinates = Vec::with_capacity(values.len() / 2);
    for pair in values.chunks_exact(2) {
        coordinates.push(parse_coordinate(pair[0], pair[1], field)?);
    }
    Ok(coordinates)
}

fn parse_position(value: &str, field: &str) -> Result<Coordinate, AixmImportError> {
    let values = value
        .split_whitespace()
        .map(|part| {
            part.parse::<f64>()
                .map_err(|_| import_error(field, format!("{field} contains a non-numeric value.")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != 2 {
        return Err(import_error(
            field,
            format!("{field} must contain one lon/lat pair."),
        ));
    }
    parse_coordinate(values[0], values[1], field)
}

fn parse_coordinate(lon: f64, lat: f64, field: &str) -> Result<Coordinate, AixmImportError> {
    if !lon.is_finite()
        || !lat.is_finite()
        || !(-180.0..=180.0).contains(&lon)
        || !(-90.0..=90.0).contains(&lat)
    {
        return Err(import_error(
            field,
            "AIXM coordinate must be finite lon/lat degrees within valid bounds.",
        ));
    }
    Ok(Coordinate { lon, lat })
}

fn trim_closing_coordinate(coordinates: &mut Vec<Coordinate>) {
    if coordinates.len() > 1
        && let (Some(first), Some(last)) = (coordinates.first(), coordinates.last())
        && coordinate_matches(*first, *last)
    {
        coordinates.pop();
    }
}

fn coordinates_match(left: &[Coordinate], right: &[Coordinate]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| coordinate_matches(*left, *right))
}

fn coordinate_matches(left: Coordinate, right: Coordinate) -> bool {
    const EPSILON: f64 = 0.000001;
    (left.lon - right.lon).abs() <= EPSILON && (left.lat - right.lat).abs() <= EPSILON
}

fn date_range_with_preserved_weekdays(
    base: &DateRange,
    start: NaiveDate,
    end: NaiveDate,
) -> DateRange {
    if start == end {
        return DateRange {
            start,
            end,
            active_weekdays: BTreeSet::new(),
        };
    }

    let possible = DateRange::new(start, end).possible_weekdays();
    let active_weekdays = base
        .active_weekdays
        .intersection(&possible)
        .copied()
        .collect::<BTreeSet<_>>();
    DateRange {
        start,
        end,
        active_weekdays: if active_weekdays.is_empty() {
            possible
        } else {
            active_weekdays
        },
    }
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
        ManualMapAttributes, ManualMapCategory, ManualMapRendering, MapCatalog, MapDefaults,
        Period, PolygonNode, PreviewGeometry, SelectedStaticMap, StaticMap, TextInfo,
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

    fn static_catalog() -> MapCatalog {
        MapCatalog {
            maps: vec![
                static_map("50714", "HAUT VALAIS"),
                static_map("99999", "OTHER MAP"),
            ],
            diagnostics: Vec::new(),
        }
    }

    fn static_map(id: &str, name: &str) -> StaticMap {
        StaticMap {
            id: id.to_owned(),
            name: name.to_owned(),
            description: None,
            preview: PreviewGeometry::default(),
            defaults: MapDefaults::default(),
        }
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

    #[test]
    fn aixm_import_updates_manual_polygon_and_preserves_form_only_fields() {
        let base = manual_polygon_creation();
        let xml = to_aixm_xml(&base)
            .unwrap()
            .replace(
                "<aixm:name>Manual &lt;DAM&gt;</aixm:name>",
                "<aixm:name>Edited Manual</aixm:name>",
            )
            .replace(
                "<gml:pos>7.154023 46.512928</gml:pos>",
                "<gml:pos>7.2 46.6</gml:pos>",
            );

        let candidate = apply_aixm_xml_update(&base, &MapCatalog::default(), &xml).unwrap();

        let DamMap::Manual(manual) = candidate.map else {
            panic!("expected manual map");
        };
        assert_eq!(manual.name, "Edited Manual");
        assert_eq!(
            manual.label_position,
            Some(Coordinate {
                lon: 7.2,
                lat: 46.6
            })
        );
        assert_eq!(candidate.distribution, base.distribution);
        assert_eq!(candidate.text, base.text);
        assert_eq!(candidate.altitude_correction, base.altitude_correction);
        assert_eq!(candidate.upper_buffer, base.upper_buffer);
        assert_eq!(candidate.lower_buffer, base.lower_buffer);
    }

    #[test]
    fn aixm_import_updates_predefined_map_from_catalog() {
        let base = valid_creation();
        let xml = to_aixm_xml(&base)
            .unwrap()
            .replace(
                "<aixm:name>HAUT VALAIS</aixm:name>",
                "<aixm:name>OTHER MAP</aixm:name>",
            )
            .replace(
                "<ext:mapId>50714</ext:mapId>",
                "<ext:mapId>99999</ext:mapId>",
            );

        let candidate = apply_aixm_xml_update(&base, &static_catalog(), &xml).unwrap();

        assert!(matches!(
            candidate.map,
            DamMap::Predefined(SelectedStaticMap { ref id, ref name })
                if id == "99999" && name == "OTHER MAP"
        ));
    }

    #[test]
    fn aixm_import_rejects_predefined_geometry_edits() {
        let base = valid_creation();
        let xml = to_aixm_xml(&base)
            .unwrap()
            .replace("8.340801 46.929693", "8.350801 46.929693");

        let error = apply_aixm_xml_update(&base, &static_catalog(), &xml).unwrap_err();

        assert_eq!(error.issues[0].field, "map.geometry");
    }

    #[test]
    fn aixm_import_rejects_conflicting_begin_and_start_times() {
        let base = valid_creation();
        let xml = to_aixm_xml(&base).unwrap().replace(
            "<aixm:startTime>09:00</aixm:startTime>",
            "<aixm:startTime>09:01</aixm:startTime>",
        );

        let error = apply_aixm_xml_update(&base, &static_catalog(), &xml).unwrap_err();

        assert_eq!(error.issues[0].field, "aixm:startTime");
    }
}
