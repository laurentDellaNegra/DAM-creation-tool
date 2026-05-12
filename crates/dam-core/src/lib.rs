mod catalog;
mod distribution;
mod export;
mod model;
mod validation;

pub use catalog::{
    CatalogDiagnostic, Coordinate, MapCatalog, MapDefaults, PreviewGeometry, PreviewPath,
    StaticMap, StaticMapSymbol, StaticMapSymbolKind, bundled_catalog, switzerland_default_preview,
};
pub use distribution::{
    DistributionSelection, Sector, UnitGroup, default_distribution, unit_groups,
};
pub use export::{
    AIXM_XML_CONTENT_TYPE, AixmExportError, AixmXmlError, ExportError, SubmissionPayload,
    aixm_xml_well_formed, build_aixm_payload, build_aixm_payload_from_xml, to_aixm_xml,
};
pub use model::{
    AltitudeCorrection, BufferFilter, DamCreation, DamMap, DateRange, Level, LevelUnit,
    MAX_PERIODS, MAX_POLYGON_POINTS, ManualGeometry, ManualMap, ManualMapAttributes,
    ManualMapCategory, ManualMapRendering, Period, PolygonNode, SelectedStaticMap, TextInfo,
    TextNumberColor, TextNumberSize, Weekday, expand_polygon_nodes,
};
pub use validation::{ValidationError, ValidationIssue, validate_creation};
