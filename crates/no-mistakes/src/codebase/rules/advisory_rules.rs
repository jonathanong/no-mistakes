use super::{analysis_policy::AnalysisDiagnostics, RuleFinding};

pub const ADVISORY_RULE_IDS: &[&str] = &[super::POSTGRES_JSON_RELATION_IDS];

/// Route findings from always-advisory rules without treating uncertainty as advice.
#[doc(hidden)]
pub fn partition_findings(mut diagnostics: AnalysisDiagnostics) -> AnalysisDiagnostics {
    let (mut routed, findings): (Vec<_>, Vec<_>) = diagnostics
        .findings
        .into_iter()
        .partition(|finding: &RuleFinding| ADVISORY_RULE_IDS.contains(&finding.rule.as_str()));
    diagnostics.findings = findings;
    if !routed.is_empty() {
        diagnostics.advisories.append(&mut routed);
        diagnostics.advisories.sort();
    }
    diagnostics
}

#[cfg(test)]
mod tests;
