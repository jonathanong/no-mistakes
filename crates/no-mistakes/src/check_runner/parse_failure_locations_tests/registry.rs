use super::{CheckResults, RULES};

pub(super) fn assert_failure_locations(report: &CheckResults, lines: &[usize]) {
    // Each raw SQL consumer reports every failed statement. Generated-column
    // predicates also consume schema facts; idempotency consumes schema only.
    let mut expected: Vec<_> = RULES
        .iter()
        .chain([RULES[1], "postgres-idempotent-insert"].iter())
        .flat_map(|rule| lines.iter().map(move |line| (*rule, *line)))
        .collect();
    let mut actual: Vec<_> = report
        .rules
        .iter()
        .map(|row| (row.rule.as_str(), row.line))
        .collect();
    expected.sort_unstable();
    actual.sort_unstable();
    assert_eq!(actual, expected, "{:?}", report.rules);
}
