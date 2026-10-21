use super::Effort;
use serde::{Deserialize, Serialize};

/// A requested low analysis could not rely on an enforced partner check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisFallback {
    pub rule: String,
    pub partner: String,
    pub requested: Effort,
    pub effective: Effort,
}
