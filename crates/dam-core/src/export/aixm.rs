use crate::{
    AltitudeCorrection, BufferFilter, Coordinate, DamCreation, DamMap, Level, LevelUnit,
    ManualGeometry, ManualMap, ManualMapCategory, PolygonNode, TextNumberColor, TextNumberSize,
    ValidationError, ValidationIssue, Weekday, buffered_polygon_outlines, expand_polygon_nodes,
    geometry_center, polygon_arc_angles, strip_corners, validate_aixm_export_ready,
    validate_creation,
};
use chrono::NaiveDate;
use std::fmt;
use thiserror::Error;

const DESIGNATOR: &str = "UAV";
const STATIC_FALLBACK_LABEL: Coordinate = Coordinate {
    lon: 8.168292,
    lat: 46.92617,
};
const STATIC_FALLBACK_GEOMETRY: [Coordinate; 12] = [
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
];

#[derive(Debug, Error)]
pub enum AixmExportError {
    #[error("validation failed")]
    Validation(#[from] ValidationError),
    #[error("static map id must be numeric for AIXM export: {id}")]
    InvalidMapId { id: String },
    #[error("AIXM export requires complete geometry")]
    MissingGeometry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AixmXmlError {
    pub issues: Vec<ValidationIssue>,
}

impl fmt::Display for AixmXmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} AIXM XML issue(s)", self.issues.len())
    }
}

impl std::error::Error for AixmXmlError {}

#[derive(Debug, Clone)]
struct AixmMapData {
    map_id: String,
    name: String,
    geometry_components: Vec<GeometryComponent>,
    label_position: Coordinate,
    display_levels: bool,
    manual_info: Option<ManualMapInformation>,
}

#[derive(Debug, Clone)]
struct GeometryComponent {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
enum Segment {
    GeodesicString(Vec<Coordinate>),
    ArcByCenterPoint {
        center: Coordinate,
        radius_nm: f64,
        start_angle_deg: f64,
        end_angle_deg: f64,
    },
}

#[derive(Debug, Clone)]
struct ManualMapInformation {
    typ: &'static str,
    att: &'static str,
    dinfo: String,
    text: Option<TextNumberInformation>,
}

#[derive(Debug, Clone)]
struct TextNumberInformation {
    tx: String,
    tc: &'static str,
    tf: &'static str,
}

#[derive(Default)]
struct GmlIdAllocator {
    next: usize,
}

impl GmlIdAllocator {
    fn next(&mut self) -> String {
        self.next += 1;
        format!("id{}", self.next)
    }
}

pub fn to_aixm_xml(creation: &DamCreation) -> Result<String, AixmExportError> {
    validate_creation(creation)?;
    validate_aixm_export_ready(creation)?;

    let map = aixm_map_data(creation)?;
    let mut ids = GmlIdAllocator::default();
    let airspace_id = ids.next();
    let timeslice_id = ids.next();
    let validity_id = ids.next();
    let geometry_components = format_geometry_components(&map.geometry_components, &mut ids);
    let activation_id = ids.next();
    let timesheets = format_timesheets(creation, &mut ids);
    let extension_id = ids.next();
    let begin_position = format_midnight_utc(creation.date_range.start);
    let label_position = format_position(map.label_position);
    let map_information = build_map_information(creation, &map);

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<asn:airspaceStatusNotificationMessage
    xmlns:asn="http://www.skyguide.ch/cmm/atm/environment/aeronautical/airspace/statusanotification/v1"
    xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.skyguide.ch/cmm/atm/environment/aeronautical/airspace/statusanotification/v1 ./Schemas/AirspaceStatusNotificationMessage_v1_0.xsd">

    <asn:airspace gml:id="{airspace_id}" xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
                  xmlns:gml="http://www.opengis.net/gml/3.2">
        <aixm:timeSlice>
            <aixm:AirspaceTimeSlice gml:id="{timeslice_id}">
                <gml:validTime>
                    <gml:TimePeriod gml:id="{validity_id}">
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
{geometry_components}
                <aixm:activation>
                    <aixm:AirspaceActivation gml:id="{activation_id}">
{timesheets}
                    </aixm:AirspaceActivation>
                </aixm:activation>
                <aixm:extension>
                    <ext:DynamicAirspaceExtension gml:id="{extension_id}" xmlns:ext="http://www.skyguide.ch/aixm/v5.1.1/sg/5.0"
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
        airspace_id = escape_xml(&airspace_id),
        timeslice_id = escape_xml(&timeslice_id),
        validity_id = escape_xml(&validity_id),
        begin_position = escape_xml(&begin_position),
        designator = DESIGNATOR,
        name = escape_xml(&map.name),
        geometry_components = geometry_components,
        activation_id = escape_xml(&activation_id),
        timesheets = timesheets,
        extension_id = escape_xml(&extension_id),
        display_levels = yes_no(map.display_levels),
        label_position = label_position,
        map_id = escape_xml(&map.map_id),
        map_information = escape_xml(&map_information),
    ))
}

pub fn aixm_xml_well_formed(xml: &str) -> Result<(), AixmXmlError> {
    roxmltree::Document::parse(xml)
        .map(|_| ())
        .map_err(|error| xml_error("aixm.xml", format!("XML is not well formed: {error}")))
}

fn xml_error(field: impl Into<String>, message: impl Into<String>) -> AixmXmlError {
    AixmXmlError {
        issues: vec![ValidationIssue {
            field: field.into(),
            message: message.into(),
        }],
    }
}

fn aixm_map_data(creation: &DamCreation) -> Result<AixmMapData, AixmExportError> {
    match &creation.map {
        DamMap::Predefined(selected) => {
            let map_id = selected.id.trim();
            if map_id.parse::<u32>().is_err() {
                return Err(AixmExportError::InvalidMapId {
                    id: selected.id.clone(),
                });
            }

            let geometry = normalize_open_ring(
                selected
                    .fallback_geometry
                    .clone()
                    .filter(|coordinates| coordinates.len() >= 3)
                    .unwrap_or_else(|| STATIC_FALLBACK_GEOMETRY.to_vec()),
            );
            let label_position = selected
                .fallback_label_position
                .or_else(|| geometry_center(&geometry))
                .unwrap_or(STATIC_FALLBACK_LABEL);

            Ok(AixmMapData {
                map_id: map_id.to_owned(),
                name: selected.name.trim().to_uppercase(),
                geometry_components: vec![GeometryComponent {
                    segments: vec![Segment::GeodesicString(geometry)],
                }],
                label_position,
                display_levels: creation.display_levels,
                manual_info: None,
            })
        }
        DamMap::Manual(manual) => manual_map_data(manual, creation.display_levels),
    }
}

fn manual_map_data(
    manual: &ManualMap,
    display_levels: bool,
) -> Result<AixmMapData, AixmExportError> {
    let name = manual.name.trim().to_uppercase();
    let category = manual_category_code(manual.attributes.category);
    let buffer_nm = manual.attributes.lateral_buffer_nm;

    let (geometry_components, label_position, forced_display_levels, manual_info) = match &manual
        .geometry
    {
        ManualGeometry::Polygon { nodes } => {
            let components = polygon_components(nodes, buffer_nm)?;
            let label_position = manual
                .label_position
                .ok_or(AixmExportError::MissingGeometry)?;
            let dinfo = polygon_dinfo(nodes, manual.attributes.category, buffer_nm);
            (
                components,
                label_position,
                display_levels,
                ManualMapInformation {
                    typ: "LIST",
                    att: category,
                    dinfo,
                    text: None,
                },
            )
        }
        ManualGeometry::ParaSymbol { point } => {
            let point = point.ok_or(AixmExportError::MissingGeometry)?;
            (
                vec![GeometryComponent {
                    segments: Vec::new(),
                }],
                point,
                false,
                ManualMapInformation {
                    typ: "LIST",
                    att: "PARA",
                    dinfo: format!("type=PARA|cat=PARA|point={}", format_position(point)),
                    text: None,
                },
            )
        }
        ManualGeometry::TextNumber {
            point,
            text,
            color,
            size,
        } => {
            let point = point.ok_or(AixmExportError::MissingGeometry)?;
            let color = text_number_color_code(*color);
            let size = text_number_size_code(*size);
            (
                vec![GeometryComponent {
                    segments: Vec::new(),
                }],
                point,
                false,
                ManualMapInformation {
                    typ: "TEXT",
                    att: category,
                    dinfo: format!(
                        "type=TEXT_NUMBER|cat={category}|point={point}|text={text}|color={color}|size={size}",
                        point = format_position(point),
                        text = text.trim(),
                    ),
                    text: Some(TextNumberInformation {
                        tx: text.trim().to_owned(),
                        tc: color,
                        tf: size,
                    }),
                },
            )
        }
        ManualGeometry::PieCircle {
            center,
            radius_nm,
            first_angle_deg,
            last_angle_deg,
        } => {
            let center = center.ok_or(AixmExportError::MissingGeometry)?;
            let radius_nm = radius_nm.ok_or(AixmExportError::MissingGeometry)?;
            let label_position = manual
                .label_position
                .ok_or(AixmExportError::MissingGeometry)?;
            let mut components = vec![GeometryComponent {
                segments: vec![Segment::ArcByCenterPoint {
                    center,
                    radius_nm,
                    start_angle_deg: *first_angle_deg,
                    end_angle_deg: *last_angle_deg,
                }],
            }];
            if buffer_nm > 0.0 {
                components.push(GeometryComponent {
                    segments: vec![Segment::ArcByCenterPoint {
                        center,
                        radius_nm: radius_nm + buffer_nm,
                        start_angle_deg: *first_angle_deg,
                        end_angle_deg: *last_angle_deg,
                    }],
                });
            }
            (
                components,
                label_position,
                display_levels,
                ManualMapInformation {
                    typ: "LIST",
                    att: category,
                    dinfo: format!(
                        "type=PIE_CIRCLE|cat={category}|buffer_nm={buffer}|center={center}|radius_nm={radius}|angles={start},{end}",
                        buffer = format_decimal(buffer_nm),
                        center = format_position(center),
                        radius = format_decimal(radius_nm),
                        start = format_decimal(*first_angle_deg),
                        end = format_decimal(*last_angle_deg),
                    ),
                    text: None,
                },
            )
        }
        ManualGeometry::Strip {
            point1,
            point2,
            width_nm,
        } => {
            let point1 = point1.ok_or(AixmExportError::MissingGeometry)?;
            let point2 = point2.ok_or(AixmExportError::MissingGeometry)?;
            let width_nm = width_nm.ok_or(AixmExportError::MissingGeometry)?;
            let label_position = manual
                .label_position
                .ok_or(AixmExportError::MissingGeometry)?;
            let corners = strip_corners(point1, point2, width_nm);
            let components = polygon_buffer_components(corners, buffer_nm);
            (
                components,
                label_position,
                display_levels,
                ManualMapInformation {
                    typ: "LIST",
                    att: category,
                    dinfo: format!(
                        "type=STRIP|cat={category}|buffer_nm={buffer}|point1={point1}|point2={point2}|width_nm={width}",
                        buffer = format_decimal(buffer_nm),
                        point1 = format_position(point1),
                        point2 = format_position(point2),
                        width = format_decimal(width_nm),
                    ),
                    text: None,
                },
            )
        }
    };

    Ok(AixmMapData {
        map_id: "0".to_owned(),
        name,
        geometry_components,
        label_position,
        display_levels: forced_display_levels,
        manual_info: Some(manual_info),
    })
}

fn polygon_components(
    nodes: &[PolygonNode],
    buffer_nm: f64,
) -> Result<Vec<GeometryComponent>, AixmExportError> {
    let segments = polygon_segments(nodes)?;
    let outline = normalize_open_ring(expand_polygon_nodes(nodes));
    let mut components = vec![GeometryComponent { segments }];
    if buffer_nm > 0.0 {
        for buffered in buffered_polygon_outlines(&outline, buffer_nm)
            .into_iter()
            .take(1)
        {
            components.push(GeometryComponent {
                segments: vec![Segment::GeodesicString(normalize_open_ring(buffered))],
            });
        }
    }
    Ok(components)
}

fn polygon_segments(nodes: &[PolygonNode]) -> Result<Vec<Segment>, AixmExportError> {
    let mut segments = Vec::new();
    let mut point_run = Vec::new();

    for (index, node) in nodes.iter().enumerate() {
        match node {
            PolygonNode::Point { coordinate } => point_run.push(*coordinate),
            PolygonNode::Arc { center, radius_nm } => {
                if point_run.len() >= 2 {
                    segments.push(Segment::GeodesicString(std::mem::take(&mut point_run)));
                } else {
                    point_run.clear();
                }
                let (start_angle_deg, end_angle_deg) = polygon_arc_angles(nodes, index)
                    .map_err(|_| AixmExportError::MissingGeometry)?;
                segments.push(Segment::ArcByCenterPoint {
                    center: *center,
                    radius_nm: *radius_nm,
                    start_angle_deg,
                    end_angle_deg,
                });
            }
        }
    }

    if point_run.len() >= 2 {
        segments.push(Segment::GeodesicString(point_run));
    }

    if segments.is_empty() {
        return Err(AixmExportError::MissingGeometry);
    }
    Ok(segments)
}

fn polygon_buffer_components(points: Vec<Coordinate>, buffer_nm: f64) -> Vec<GeometryComponent> {
    let points = normalize_open_ring(points);
    let mut components = vec![GeometryComponent {
        segments: vec![Segment::GeodesicString(points.clone())],
    }];
    if buffer_nm > 0.0 {
        for buffered in buffered_polygon_outlines(&points, buffer_nm)
            .into_iter()
            .take(1)
        {
            components.push(GeometryComponent {
                segments: vec![Segment::GeodesicString(normalize_open_ring(buffered))],
            });
        }
    }
    components
}

fn format_geometry_components(
    components: &[GeometryComponent],
    ids: &mut GmlIdAllocator,
) -> String {
    components
        .iter()
        .map(|component| {
            let component_id = ids.next();
            let volume_id = ids.next();
            let surface_id = ids.next();
            let curve_id = ids.next();
            let segments = format_segments(&component.segments);
            format!(
                r#"                <aixm:geometryComponent>
                    <aixm:AirspaceGeometryComponent gml:id="{component_id}">
                        <aixm:theAirspaceVolume>
                            <aixm:AirspaceVolume gml:id="{volume_id}">
                                <aixm:horizontalProjection>
                                    <aixm:Surface gml:id="{surface_id}">
                                        <gml:patches>
                                            <gml:PolygonPatch>
                                                <gml:exterior>
                                                    <gml:Ring>
                                                        <gml:curveMember>
                                                            <gml:Curve gml:id="{curve_id}">
{segments}
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
                </aixm:geometryComponent>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_segments(segments: &[Segment]) -> String {
    if segments.is_empty() {
        return "                                                                <gml:segments/>"
            .to_owned();
    }

    let body = segments
        .iter()
        .map(|segment| match segment {
            Segment::GeodesicString(coordinates) => {
                let positions = coordinates
                    .iter()
                    .map(|coordinate| {
                        format!(
                            "                                                                        <gml:pos>{}</gml:pos>",
                            format_position(*coordinate)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    r#"                                                                    <gml:GeodesicString>
{positions}
                                                                    </gml:GeodesicString>"#
                )
            }
            Segment::ArcByCenterPoint {
                center,
                radius_nm,
                start_angle_deg,
                end_angle_deg,
            } => format!(
                r#"                                                                    <gml:ArcByCenterPoint numArc="1">
                                                                        <gml:pos>{center}</gml:pos>
                                                                        <gml:radius uom="NM">{radius}</gml:radius>
                                                                        <gml:startAngle uom="deg">{start}</gml:startAngle>
                                                                        <gml:endAngle uom="deg">{end}</gml:endAngle>
                                                                    </gml:ArcByCenterPoint>"#,
                center = format_position(*center),
                radius = format_decimal(*radius_nm),
                start = format_decimal(*start_angle_deg),
                end = format_decimal(*end_angle_deg),
            ),
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"                                                                <gml:segments>
{body}
                                                                </gml:segments>"#
    )
}

fn format_timesheets(creation: &DamCreation, ids: &mut GmlIdAllocator) -> String {
    let mut sheets = Vec::new();
    let weekdays = creation.date_range.effective_weekdays();
    for period in &creation.periods {
        if creation.date_range.is_repetitive() {
            for weekday in Weekday::ALL {
                if weekdays.contains(&weekday) {
                    sheets.push(format_timesheet(
                        ids,
                        creation.date_range.start,
                        creation.date_range.end,
                        Some(weekday),
                        period,
                    ));
                }
            }
        } else {
            sheets.push(format_timesheet(
                ids,
                creation.date_range.start,
                creation.date_range.end,
                None,
                period,
            ));
        }
    }
    sheets.join("\n")
}

fn format_timesheet(
    ids: &mut GmlIdAllocator,
    start_date: NaiveDate,
    end_date: NaiveDate,
    weekday: Option<Weekday>,
    period: &crate::Period,
) -> String {
    let timesheet_id = ids.next();
    let extension_id = ids.next();
    let day = weekday
        .map(|weekday| {
            format!(
                "                                <aixm:day>{}</aixm:day>\n",
                weekday_code(weekday)
            )
        })
        .unwrap_or_default();
    format!(
        r#"                        <aixm:timeInterval>
                            <aixm:Timesheet gml:id="{timesheet_id}">
                                <aixm:timeReference>UTC</aixm:timeReference>
                                <aixm:startDate>{start_date}</aixm:startDate>
                                <aixm:endDate>{end_date}</aixm:endDate>
{day}                                <aixm:startTime>{start_time}</aixm:startTime>
                                <aixm:endTime>{end_time}</aixm:endTime>
                                <aixm:daylightSavingAdjust>NO</aixm:daylightSavingAdjust>
                                <aixm:extension>
                                    <ext:DamTimesheetExtension gml:id="{extension_id}"
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
                        </aixm:timeInterval>"#,
        start_date = format_date(start_date),
        end_date = format_date(end_date),
        start_time = period.start_time.format("%H:%M"),
        end_time = period.end_time.format("%H:%M"),
        display_begin = yes_no(period.start_indication),
        display_end = yes_no(period.end_indication),
        highest_uom = level_uom(period.upper),
        highest_level = format_level(period.upper),
        lowest_uom = level_uom(period.lower),
        lowest_level = format_level(period.lower),
    )
}

fn build_map_information(creation: &DamCreation, map: &AixmMapData) -> String {
    let mut pairs = vec![
        ("src", "ZRH".to_owned()),
        ("lvl", creation.a9.export_value().to_owned()),
        ("dist", creation.distribution.to_legacy_flags()),
        (
            "qnh",
            match creation.altitude_correction {
                AltitudeCorrection::QnhCorr => "1",
                AltitudeCorrection::None | AltitudeCorrection::FlCorr => "0",
            }
            .to_owned(),
        ),
        (
            "flc",
            match creation.altitude_correction {
                AltitudeCorrection::FlCorr => "1",
                AltitudeCorrection::None | AltitudeCorrection::QnhCorr => "0",
            }
            .to_owned(),
        ),
        (
            "ulh",
            matches!(creation.upper_buffer, BufferFilter::Half)
                .then_some("1")
                .unwrap_or("0")
                .to_owned(),
        ),
        (
            "llh",
            matches!(creation.lower_buffer, BufferFilter::Half)
                .then_some("1")
                .unwrap_or("0")
                .to_owned(),
        ),
        (
            "uln",
            matches!(creation.upper_buffer, BufferFilter::NoBuffer)
                .then_some("1")
                .unwrap_or("0")
                .to_owned(),
        ),
        (
            "lln",
            matches!(creation.lower_buffer, BufferFilter::NoBuffer)
                .then_some("1")
                .unwrap_or("0")
                .to_owned(),
        ),
        ("txt", creation.text.value.trim().to_owned()),
    ];

    if let Some(manual) = &map.manual_info {
        pairs.push(("typ", manual.typ.to_owned()));
        pairs.push(("att", manual.att.to_owned()));
        pairs.push(("dinfo", manual.dinfo.clone()));
        if let Some(text) = &manual.text {
            pairs.push(("tx", text.tx.clone()));
            pairs.push(("tc", text.tc.to_owned()));
            pairs.push(("tf", text.tf.to_owned()));
        }
    }

    pairs
        .into_iter()
        .map(|(key, value)| format!("{key}={}", quote_map_information_value(&value)))
        .collect::<Vec<_>>()
        .join(";")
}

fn quote_map_information_value(value: &str) -> String {
    let value = value.trim();
    let needs_quotes = value.chars().any(|ch| matches!(ch, ';' | '\\' | '"' | '='));
    if !needs_quotes {
        return value.to_owned();
    }
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn polygon_dinfo(nodes: &[PolygonNode], category: ManualMapCategory, buffer_nm: f64) -> String {
    let nodes = nodes
        .iter()
        .map(|node| match node {
            PolygonNode::Point { coordinate } => format!("P:{}", format_position(*coordinate)),
            PolygonNode::Arc { center, radius_nm } => format!(
                "A:{} {}",
                format_position(*center),
                format_decimal(*radius_nm)
            ),
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "type=POLYGON|cat={}|buffer_nm={}|nodes={nodes}",
        manual_category_code(category),
        format_decimal(buffer_nm)
    )
}

fn manual_category_code(category: ManualMapCategory) -> &'static str {
    match category {
        ManualMapCategory::Prohibited => "PROHIBITED",
        ManualMapCategory::Danger => "DANGER",
        ManualMapCategory::Restricted => "RESTRICTED",
        ManualMapCategory::Glider => "GLIDER",
        ManualMapCategory::Ctr => "CTR",
        ManualMapCategory::Cfz => "CFZ",
        ManualMapCategory::Tma => "TMA",
        ManualMapCategory::Para => "PARA",
        ManualMapCategory::Other => "OTHER",
    }
}

fn text_number_color_code(color: TextNumberColor) -> &'static str {
    match color {
        TextNumberColor::Red => "RED",
        TextNumberColor::Green => "GREEN",
        TextNumberColor::Blue => "BLUE",
        TextNumberColor::Yellow => "YELLOW",
        TextNumberColor::White => "WHITE",
    }
}

fn text_number_size_code(size: TextNumberSize) -> &'static str {
    match size {
        TextNumberSize::Small => "SMALL",
        TextNumberSize::Medium => "MEDIUM",
        TextNumberSize::Large => "LARGE",
    }
}

fn normalize_open_ring(mut coordinates: Vec<Coordinate>) -> Vec<Coordinate> {
    while coordinates.len() >= 2 && coordinates.first() == coordinates.last() {
        coordinates.pop();
    }
    coordinates
}

fn format_midnight_utc(date: NaiveDate) -> String {
    format!("{}T00:00:00.000Z", date.format("%Y-%m-%d"))
}

fn format_date(date: NaiveDate) -> String {
    date.format("%d-%m").to_string()
}

fn weekday_code(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "MON",
        Weekday::Tue => "TUE",
        Weekday::Wed => "WED",
        Weekday::Thu => "THU",
        Weekday::Fri => "FRI",
        Weekday::Sat => "SAT",
        Weekday::Sun => "SUN",
    }
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
    level.value.to_string()
}

fn yes_no(value: bool) -> &'static str {
    if value { "YES" } else { "NO" }
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
        A9Level, DateRange, DistributionSelection, LegacyDistributionTarget, ManualMapAttributes,
        SelectedStaticMap, TextInfo,
    };
    use chrono::{NaiveDate, NaiveTime};
    use std::collections::BTreeSet;

    fn valid_creation() -> DamCreation {
        DamCreation {
            map: DamMap::Predefined(SelectedStaticMap {
                id: "50714".to_owned(),
                name: "haut valais".to_owned(),
                fallback_geometry: Some(vec![
                    coord(7.1, 46.1),
                    coord(7.2, 46.2),
                    coord(7.3, 46.1),
                    coord(7.1, 46.1),
                ]),
                fallback_label_position: Some(coord(7.25, 46.15)),
            }),
            date_range: DateRange::new(date(2026, 5, 7), date(2026, 5, 7)),
            periods: vec![period(
                "09:00",
                "10:30",
                Level::new(0, LevelUnit::FlightLevel),
                Level::new(140, LevelUnit::FlightLevel),
            )],
            display_levels: true,
            altitude_correction: AltitudeCorrection::None,
            upper_buffer: BufferFilter::Default,
            lower_buffer: BufferFilter::Default,
            distribution: DistributionSelection::all(),
            a9: A9Level::default(),
            text: TextInfo::default(),
        }
    }

    fn manual_creation(geometry: ManualGeometry) -> DamCreation {
        let mut creation = valid_creation();
        creation.map = DamMap::Manual(ManualMap {
            name: "MY MANUAL".to_owned(),
            geometry,
            attributes: ManualMapAttributes {
                category: ManualMapCategory::Danger,
                lateral_buffer_nm: 0.0,
            },
            label_position: Some(coord(7.15, 46.15)),
        });
        creation
    }

    fn period(start: &str, end: &str, lower: Level, upper: Level) -> crate::Period {
        crate::Period {
            start_indication: true,
            start_time: NaiveTime::parse_from_str(start, "%H:%M").unwrap(),
            end_indication: false,
            end_time: NaiveTime::parse_from_str(end, "%H:%M").unwrap(),
            lower,
            upper,
        }
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn coord(lon: f64, lat: f64) -> Coordinate {
        Coordinate { lon, lat }
    }

    #[test]
    fn predefined_export_has_fixed_metadata_midnight_and_repeated_pos() {
        let xml = to_aixm_xml(&valid_creation()).unwrap();

        assert!(xml.contains("<aixm:designator>UAV</aixm:designator>"));
        assert!(xml.contains("<aixm:type>OTHER</aixm:type>"));
        assert!(xml.contains("<aixm:sequenceNumber>0</aixm:sequenceNumber>"));
        assert!(xml.contains("<aixm:correctionNumber>0</aixm:correctionNumber>"));
        assert!(xml.contains("<ext:sliceId>0</ext:sliceId>"));
        assert!(xml.contains("<ext:mapHasBeenDeleted>NO</ext:mapHasBeenDeleted>"));
        assert!(xml.contains("<aixm:name>HAUT VALAIS</aixm:name>"));
        assert!(xml.contains("<gml:beginPosition>2026-05-07T00:00:00.000Z</gml:beginPosition>"));
        assert!(xml.contains("<gml:endPosition/>"));
        assert!(xml.contains("<gml:GeodesicString>"));
        assert!(xml.contains("<gml:pos>7.1 46.1</gml:pos>"));
        assert!(!xml.contains("gml:posList"));
        assert!(xml.contains("<gml:pos>7.25 46.15</gml:pos>"));
    }

    #[test]
    fn single_day_omits_day_and_repetitive_range_emits_selected_weekdays_for_each_period() {
        let single = to_aixm_xml(&valid_creation()).unwrap();
        assert!(!single.contains("<aixm:day>"));

        let mut creation = valid_creation();
        creation.date_range = DateRange {
            start: date(2026, 5, 7),
            end: date(2026, 5, 10),
            active_weekdays: BTreeSet::from([Weekday::Thu, Weekday::Sat]),
        };
        creation.periods.push(period(
            "11:00",
            "12:00",
            Level::new(85, LevelUnit::FlightLevel),
            Level::new(9500, LevelUnit::Feet),
        ));

        let xml = to_aixm_xml(&creation).unwrap();

        assert_eq!(xml.matches("<aixm:Timesheet").count(), 4);
        assert_eq!(xml.matches("<aixm:day>THU</aixm:day>").count(), 2);
        assert_eq!(xml.matches("<aixm:day>SAT</aixm:day>").count(), 2);
        assert!(xml.contains(r#"<ext:highestLevel uom="FT">9500</ext:highestLevel>"#));
        assert!(xml.contains(r#"<ext:lowestLevel uom="FL">85</ext:lowestLevel>"#));
        assert!(!xml.contains(">085</ext:lowestLevel>"));
    }

    #[test]
    fn map_information_has_v1_key_order_a9_distribution_and_correction_flags() {
        let mut creation = valid_creation();
        creation.a9 = A9Level::L160_2;
        creation.altitude_correction = AltitudeCorrection::FlCorr;
        creation.upper_buffer = BufferFilter::Half;
        creation.lower_buffer = BufferFilter::NoBuffer;
        creation.text.value = r#"needs;quote=\"yes\""#.to_owned();
        creation.distribution = DistributionSelection::none();
        creation
            .distribution
            .set(LegacyDistributionTarget::AccUpper, true);
        creation
            .distribution
            .set(LegacyDistributionTarget::TdiStGallen, true);

        let xml = to_aixm_xml(&creation).unwrap();

        assert!(xml.contains(
            "src=ZRH;lvl=160:2;dist=1/0/0/0/0/0/0/0/0/0/0/1;qnh=0;flc=1;ulh=1;llh=0;uln=0;lln=1;txt=&quot;needs;quote="
        ));
        assert!(xml.contains("yes"));
    }

    #[test]
    fn predefined_map_information_omits_manual_fields() {
        let xml = to_aixm_xml(&valid_creation()).unwrap();

        assert!(!xml.contains(";typ="));
        assert!(!xml.contains(";att="));
        assert!(!xml.contains(";dinfo="));
        assert!(!xml.contains(";tx="));
    }

    #[test]
    fn manual_polygon_exports_points_arcs_label_category_and_no_pos_list() {
        let mut creation = manual_creation(ManualGeometry::Polygon {
            nodes: vec![
                PolygonNode::point(coord(7.0, 46.0)),
                PolygonNode::Arc {
                    center: coord(7.1, 46.05),
                    radius_nm: 5.0,
                },
                PolygonNode::point(coord(7.2, 46.0)),
                PolygonNode::point(coord(7.2, 46.2)),
            ],
        });
        if let DamMap::Manual(manual) = &mut creation.map {
            manual.attributes.category = ManualMapCategory::Restricted;
        }

        let xml = to_aixm_xml(&creation).unwrap();

        assert!(xml.contains("<ext:mapId>0</ext:mapId>"));
        assert!(xml.contains("<aixm:name>MY MANUAL</aixm:name>"));
        assert!(xml.contains("<gml:ArcByCenterPoint numArc=\"1\">"));
        assert!(xml.contains("<gml:radius uom=\"NM\">5</gml:radius>"));
        assert!(xml.contains("<gml:pos>7.2 46.2</gml:pos>"));
        assert!(xml.contains("<gml:pos>7.15 46.15</gml:pos>"));
        assert!(xml.contains(";typ=LIST;att=RESTRICTED;dinfo="));
        assert!(!xml.contains("gml:posList"));
    }

    #[test]
    fn polygon_arc_without_adjacent_point_anchor_blocks_validation() {
        let creation = manual_creation(ManualGeometry::Polygon {
            nodes: vec![
                PolygonNode::point(coord(7.0, 46.0)),
                PolygonNode::Arc {
                    center: coord(7.1, 46.05),
                    radius_nm: 5.0,
                },
                PolygonNode::Arc {
                    center: coord(7.2, 46.05),
                    radius_nm: 5.0,
                },
                PolygonNode::point(coord(7.2, 46.2)),
            ],
        });

        let err = to_aixm_xml(&creation).unwrap_err();

        let AixmExportError::Validation(err) = err else {
            panic!("expected validation error");
        };
        assert!(
            err.issues
                .iter()
                .any(|issue| issue.field == "map.geometry.nodes[1]")
        );
    }

    #[test]
    fn para_symbol_exports_empty_segments_forced_no_display_levels_and_requires_para_name() {
        let mut creation = manual_creation(ManualGeometry::ParaSymbol {
            point: Some(coord(7.0, 46.0)),
        });
        if let DamMap::Manual(manual) = &mut creation.map {
            manual.name = "DROP PARA AREA".to_owned();
            manual.attributes.category = ManualMapCategory::Para;
            manual.label_position = None;
        }

        let xml = to_aixm_xml(&creation).unwrap();

        assert!(xml.contains("<gml:segments/>"));
        assert!(xml.contains("<ext:displayLevels>NO</ext:displayLevels>"));
        assert!(xml.contains("<gml:pos>7 46</gml:pos>"));
        assert!(xml.contains(";typ=LIST;att=PARA;dinfo=&quot;type=PARA|cat=PARA|point=7 46&quot;"));

        if let DamMap::Manual(manual) = &mut creation.map {
            manual.name = "DROP AREA".to_owned();
        }
        assert!(to_aixm_xml(&creation).is_err());
    }

    #[test]
    fn text_number_exports_empty_segments_text_metadata_and_forced_no_display_levels() {
        let mut creation = manual_creation(ManualGeometry::TextNumber {
            point: Some(coord(7.0, 46.0)),
            text: "A12".to_owned(),
            color: TextNumberColor::Yellow,
            size: TextNumberSize::Large,
        });
        if let DamMap::Manual(manual) = &mut creation.map {
            manual.label_position = None;
        }

        let xml = to_aixm_xml(&creation).unwrap();

        assert!(xml.contains("<gml:segments/>"));
        assert!(xml.contains("<ext:displayLevels>NO</ext:displayLevels>"));
        assert!(xml.contains(";typ=TEXT;att=DANGER;dinfo=&quot;type=TEXT_NUMBER"));
        assert!(xml.contains(";tx=A12;tc=YELLOW;tf=LARGE"));
    }

    #[test]
    fn pie_circle_exports_arc_signed_angles_and_buffer_component() {
        let mut creation = manual_creation(ManualGeometry::PieCircle {
            center: Some(coord(7.0, 46.0)),
            radius_nm: Some(5.0),
            first_angle_deg: -45.0,
            last_angle_deg: 180.0,
        });
        if let DamMap::Manual(manual) = &mut creation.map {
            manual.attributes.lateral_buffer_nm = 2.0;
        }

        let xml = to_aixm_xml(&creation).unwrap();

        assert_eq!(xml.matches("<aixm:geometryComponent>").count(), 2);
        assert!(xml.contains("<gml:startAngle uom=\"deg\">-45</gml:startAngle>"));
        assert!(xml.contains("<gml:endAngle uom=\"deg\">180</gml:endAngle>"));
        assert!(xml.contains("<gml:radius uom=\"NM\">7</gml:radius>"));

        if let DamMap::Manual(manual) = &mut creation.map
            && let ManualGeometry::PieCircle {
                first_angle_deg, ..
            } = &mut manual.geometry
        {
            *first_angle_deg = -361.0;
        }
        assert!(to_aixm_xml(&creation).is_err());
    }

    #[test]
    fn strip_exports_computed_corners_and_buffer_component() {
        let mut creation = manual_creation(ManualGeometry::Strip {
            point1: Some(coord(7.0, 46.0)),
            point2: Some(coord(7.2, 46.2)),
            width_nm: Some(3.0),
        });
        if let DamMap::Manual(manual) = &mut creation.map {
            manual.attributes.lateral_buffer_nm = 1.0;
        }

        let xml = to_aixm_xml(&creation).unwrap();

        assert_eq!(xml.matches("<aixm:geometryComponent>").count(), 2);
        assert!(xml.contains("<gml:GeodesicString>"));
        assert!(xml.contains(";typ=LIST;att=DANGER;dinfo=&quot;type=STRIP"));
        assert!(!xml.contains("gml:posList"));
    }

    #[test]
    fn string_sanitation_blocks_newlines_before_export() {
        let mut creation = valid_creation();
        creation.text.value = "bad\ntext".to_owned();

        let err = to_aixm_xml(&creation).unwrap_err();

        let AixmExportError::Validation(err) = err else {
            panic!("expected validation error");
        };
        assert!(err.issues.iter().any(|issue| issue.field == "text.value"));
    }

    #[test]
    fn aixm_xml_well_formed_reports_parse_errors() {
        let error = aixm_xml_well_formed("<root>").unwrap_err();

        assert_eq!(error.issues[0].field, "aixm.xml");
    }
}
