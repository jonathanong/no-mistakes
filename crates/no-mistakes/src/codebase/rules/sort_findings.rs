use super::RuleFinding;

pub(crate) fn sort_findings(findings: &mut Vec<RuleFinding>) {
    findings.sort();
    findings.dedup();
}
