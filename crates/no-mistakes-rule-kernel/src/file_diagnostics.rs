use crate::finding::{sort_findings, RuleFinding};
use crate::rule_ids::UNUSED_ALLOW_ENTRY;
use crate::suppression::SuppressedFinding;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Unanalyzable {
    #[default]
    Report,
    Advise,
    Ignore,
}

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

pub enum InputUncertainty {
    Read,
    ReadOrParse,
}

pub fn route_application(
    policy: Unanalyzable,
    message: Option<&str>,
    findings: Vec<RuleFinding>,
    uncertain: impl Fn(&RuleFinding) -> bool,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    for mut finding in findings {
        if finding.rule == UNUSED_ALLOW_ENTRY {
            diagnostics.findings.push(finding);
            continue;
        }
        if let Some(message) = message {
            finding.message = message.to_owned();
        }
        if finding.target.as_deref() == Some("unanalyzable") && uncertain(&finding) {
            diagnostics.unanalyzable(policy, finding);
        } else {
            diagnostics.findings.push(finding);
        }
    }
    diagnostics
}

pub fn sorted(mut diagnostics: AnalysisDiagnostics) -> AnalysisDiagnostics {
    sort_findings(&mut diagnostics.findings);
    sort_findings(&mut diagnostics.advisories);
    diagnostics
}

#[derive(Default, Debug)]
pub struct ClassifiedInputFindings {
    pub findings: Vec<RuleFinding>,
    pub uncertainty: Vec<RuleFinding>,
}
