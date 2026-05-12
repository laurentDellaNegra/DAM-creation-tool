use dam_core::{SubmissionPayload, ValidationIssue};

#[derive(Debug, Clone)]
pub enum SubmissionStatus {
    Idle,
    Invalid(Vec<ValidationIssue>),
    Building,
    Ready { message: String },
    Submitting,
    Sent { message: String },
    Failed { message: String },
}

impl SubmissionStatus {
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

#[derive(Debug, Clone)]
pub struct SubmissionEndpoint {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionResult {
    #[allow(dead_code)]
    Sent(String),
    Failed(String),
}

pub fn submit_payload(
    endpoint: Option<&SubmissionEndpoint>,
    _payload: &SubmissionPayload,
) -> SubmissionResult {
    let Some(endpoint) = endpoint else {
        return SubmissionResult::Failed(
            "Submission endpoint not configured. Use Download AIXM to export locally.".to_owned(),
        );
    };

    if endpoint.url.trim().is_empty() {
        return SubmissionResult::Failed(
            "Submission endpoint not configured. Use Download AIXM to export locally.".to_owned(),
        );
    }

    SubmissionResult::Failed(format!(
        "Submission endpoint POST is not implemented yet: {}",
        endpoint.url
    ))
}
