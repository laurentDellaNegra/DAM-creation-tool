mod aixm;
mod payload;

use crate::ValidationError;
use thiserror::Error;

pub use aixm::{AixmExportError, AixmXmlError, aixm_xml_well_formed, to_aixm_xml};
pub use payload::{
    AIXM_XML_CONTENT_TYPE, AixmPayload, build_aixm_payload, build_aixm_payload_from_xml,
};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("validation failed")]
    Validation(#[from] ValidationError),
    #[error("AIXM export failed: {0}")]
    Aixm(#[from] AixmExportError),
}
