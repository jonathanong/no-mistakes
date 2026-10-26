use super::{SuppressedFinding, SuppressionDirective};

/// Minimal request-local provenance for every directive that hid a contribution.
/// Public finding deduplication must not discard this suppression usage.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConsumedDisableDirective {
    pub source_file: String,
    pub rule: String,
    pub directive: SuppressionDirective,
}

impl From<&SuppressedFinding> for ConsumedDisableDirective {
    fn from(finding: &SuppressedFinding) -> Self {
        Self {
            source_file: finding.source_file.clone(),
            rule: finding.rule.clone(),
            directive: finding.directive.clone(),
        }
    }
}
