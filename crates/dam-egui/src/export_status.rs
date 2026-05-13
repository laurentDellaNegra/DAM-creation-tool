use dam_core::ValidationIssue;

#[derive(Debug, Clone)]
pub enum ExportStatus {
    Idle,
    Invalid(Vec<ValidationIssue>),
    Building,
    Ready { message: String },
    Failed { message: String },
}

impl ExportStatus {
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }
}
