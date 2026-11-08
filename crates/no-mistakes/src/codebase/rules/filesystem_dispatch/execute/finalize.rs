use super::*;

pub(super) fn findings(findings: &mut Vec<RuleFinding>) {
    findings.sort();
    findings.dedup_by(|left, right| {
        left == right
            && !crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS
                .contains(&left.rule.as_str())
    });
}
