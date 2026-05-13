mod catalog;
mod distribution;
mod export;
mod geometry;
mod model;
mod validation;

pub use catalog::{
    CatalogDiagnostic, Coordinate, MapCatalog, MapDefaults, PreviewGeometry, PreviewPath,
    StaticMap, StaticMapSymbol, StaticMapSymbolKind, bundled_catalog, switzerland_default_preview,
};
pub use distribution::{
    DistributionSelection, LegacyDistribution, LegacyDistributionTarget, default_distribution,
    legacy_distribution_from_catalog_stations,
};
pub use export::{
    AIXM_XML_CONTENT_TYPE, AixmExportError, AixmPayload, AixmXmlError, ExportError,
    aixm_xml_well_formed, build_aixm_payload, build_aixm_payload_from_xml, to_aixm_xml,
};
pub use geometry::{
    angle_span, bearing_deg, buffered_polygon_outlines, destination_point, distance_nm,
    geometry_center, is_full_circle, pie_circle_points, polygon_arc_angles, strip_corners,
};
pub use model::{
    A9Level, AltitudeCorrection, BufferFilter, DamCreation, DamMap, DateRange, Level, LevelUnit,
    MAX_PERIODS, MAX_POLYGON_POINTS, ManualGeometry, ManualMap, ManualMapAttributes,
    ManualMapCategory, Period, PolygonNode, SelectedStaticMap, TextInfo, TextNumberColor,
    TextNumberSize, Weekday, expand_polygon_nodes,
};
pub use validation::{
    ValidationError, ValidationIssue, validate_aixm_export_ready, validate_creation,
};
