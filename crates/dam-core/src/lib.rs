mod catalog;
mod distribution;
mod export;
mod model;
mod validation;

pub use catalog::{
    CatalogDiagnostic, Coordinate, MapCatalog, MapDefaults, PreviewGeometry, PreviewPath,
    StaticMap, bundled_catalog, switzerland_default_preview,
};
pub use distribution::{
    DistributionSelection, Sector, UnitGroup, default_distribution, unit_groups,
};
pub use export::{
    AIXM_XML_CONTENT_TYPE, AixmExportError, AixmImportError, AixmXmlSummary, DamExport,
    ExportError, JSON_CONTENT_TYPE, SubmissionPayload, aixm_xml_well_formed, apply_aixm_xml_update,
    build_aixm_payload, build_json_payload, summarize_aixm_xml, to_aixm_xml, to_pretty_json,
};
pub use model::{
    AltitudeCorrection, BufferFilter, DamCreation, DamMap, DateRange, Level, LevelUnit,
    MAX_PERIODS, MAX_POLYGON_POINTS, ManualGeometry, ManualMap, ManualMapAttributes,
    ManualMapCategory, ManualMapRendering, Period, PolygonNode, SelectedStaticMap, TextInfo,
    TextNumberColor, TextNumberSize, Weekday, expand_polygon_nodes,
};
pub use validation::{ValidationError, ValidationIssue, validate_creation};
