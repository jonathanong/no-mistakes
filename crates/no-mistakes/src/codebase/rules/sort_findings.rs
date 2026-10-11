use super::RuleFinding;

pub(crate) fn sort_findings(findings: &mut Vec<RuleFinding>) {
    findings.sort();
    findings.dedup();
}

/// PostgreSQL alternatives can prove the same target through different statements.
pub(crate) fn sort_postgres_findings(findings: &mut Vec<super::RuleFinding>) {
    sort_findings(findings);
}
