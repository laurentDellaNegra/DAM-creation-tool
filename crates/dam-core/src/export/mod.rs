mod aixm;
mod json;
mod payload;

use crate::ValidationError;
use thiserror::Error;

pub use aixm::{AixmExportError, to_aixm_xml};
pub use json::DamExport;
pub use json::to_pretty_json;
pub use payload::{
    AIXM_XML_CONTENT_TYPE, JSON_CONTENT_TYPE, SubmissionPayload, build_aixm_payload,
    build_json_payload,
};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("validation failed")]
    Validation(#[from] ValidationError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("AIXM export failed: {0}")]
    Aixm(#[from] AixmExportError),
}
