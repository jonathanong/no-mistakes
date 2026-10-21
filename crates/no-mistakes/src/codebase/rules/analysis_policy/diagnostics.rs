use crate::codebase::rules::RuleFinding;
use crate::config::v2::Unanalyzable;

/// Low-tier adopters route unsupported shapes through the same resolved policy.
#[derive(Default)]
pub struct AnalysisDiagnostics {
    pub findings: Vec<RuleFinding>,
    pub advisories: Vec<RuleFinding>,
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
}
