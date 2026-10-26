use crate::codebase::rules::{RuleFinding, SuppressedFinding};
use crate::config::v2::Unanalyzable;

/// Low-tier adopters route unsupported shapes through the same resolved policy.
#[derive(Default)]
pub struct AnalysisDiagnostics {
    pub findings: Vec<RuleFinding>,
    pub advisories: Vec<RuleFinding>,
    /// Findings consumed by a contextual directive before aggregate filtering.
    pub suppressed: Vec<SuppressedFinding>,
}
impl AnalysisDiagnostics {
    pub fn unanalyzable(&mut self, policy: Unanalyzable, finding: RuleFinding) {
        match policy {
            Unanalyzable::Report => self.findings.push(finding),
            Unanalyzable::Advise => self.advisories.push(finding),
            Unanalyzable::Ignore => {}
        }
    }
    pub fn has_failures(&self) -> bool {
        !self.findings.is_empty()
    }
    pub fn append(&mut self, other: Self) {
        self.findings.extend(other.findings);
        self.advisories.extend(other.advisories);
        self.suppressed.extend(other.suppressed);
    }
}
