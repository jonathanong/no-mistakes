use super::scan::finding;
use super::{BannedShapes, RuleFinding};
use crate::codebase::postgres::statements::SqlFactSite;
pub(super) fn located_findings(
    file: &str,
    select: &crate::codebase::postgres::SqlSelectFact,
    shapes: &BannedShapes,
    select_index: usize,
    line_at: impl Fn(usize) -> usize,
) -> Vec<(SqlFactSite, RuleFinding)> {
    let mut findings = Vec::new();
    if shapes.correlated_exists_set_operation {
        for (index, exists) in select.exists_set_operations.iter().enumerate() {
            if exists.correlated {
                findings.push((
                    SqlFactSite::Exists(select_index, index),
                    finding(
                        file,
                        line_at(exists.line),
                        "do not wrap a set operation in a correlated EXISTS",
                        "correlated-exists-set-operation",
                    ),
                ));
            }
        }
    }
    if shapes.not_in_subquery {
        for (index, line) in select.not_in_subqueries.iter().enumerate() {
            findings.push((SqlFactSite::NotIn(select_index, index), finding(
                file,
                line_at(*line),
                "NOT IN (SELECT …) returns no rows when the subquery yields a NULL and cannot become an anti-join; use NOT EXISTS (SELECT 1 FROM … WHERE …)",
                "not-in-subquery",
            )));
        }
    }
    if shapes.count_for_existence {
        for (index, count) in select.count_existence_checks.iter().enumerate() {
            findings.push((SqlFactSite::Count(select_index, index), finding(
                file,
                line_at(count.line),
                if count.negated { "COUNT(*) compared with 0/1 counts every matching row to test absence; use NOT EXISTS (SELECT 1 FROM … WHERE …)" } else { "COUNT(*) compared with 0/1 counts every matching row to test existence; use EXISTS (SELECT 1 FROM … WHERE …)" },
                "count-for-existence",
            )));
        }
    }
    findings
}
