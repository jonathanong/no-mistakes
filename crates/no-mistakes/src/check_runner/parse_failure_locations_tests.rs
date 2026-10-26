use super::{run_all_with_options, AnalysisOptions, CheckResults};
use no_mistakes::config::v2::Effort;
use std::path::PathBuf;
const RULES: &[&str] = &[
    "postgres-no-transformed-uuid-predicates",
    "postgres-generated-column-predicates",
    "postgres-bounded-statements",
    "postgres-explicit-columns",
    "postgres-required-predicates",
    "postgres-sql-shape-policy",
];
fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-parse-failure-locations/fixture"),
    )
}
fn run(name: &str, effort: Effort) -> CheckResults {
    let root = fixture();
    crate::test_support::without_hygiene_advisories(
        run_all_with_options(
            root.clone(),
            Some(root.join(name)),
            None,
            false,
            false,
            AnalysisOptions {
                effort: Some(effort),
                include_sql_verdicts: true,
            },
        )
        .unwrap(),
    )
}

fn findings_for_rule<'a>(
    report: &'a CheckResults,
    rule: &str,
) -> Vec<&'a no_mistakes::codebase::rules::RuleFinding> {
    report.rules.iter().filter(|row| row.rule == rule).collect()
}

fn finding_lines(report: &CheckResults, rule: &str) -> Vec<usize> {
    findings_for_rule(report, rule)
        .into_iter()
        .map(|row| row.line)
        .collect()
}

fn assert_idempotent_insert_and_schema_findings(
    report: &CheckResults,
    insert_lines: &[usize],
    schema_lines: &[usize],
) {
    let rows = findings_for_rule(report, "postgres-idempotent-insert");
    let insert: Vec<_> = rows
        .iter()
        .filter(|row| row.target.as_deref() == Some("insert"))
        .map(|row| row.line)
        .collect();
    let schema: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.message
                .contains("could not be analyzed for postgres-idempotent-insert")
        })
        .map(|row| row.line)
        .collect();
    assert_eq!(
        insert, insert_lines,
        "DML INSERT findings: {:?}",
        report.rules
    );
    assert_eq!(schema, schema_lines, "schema findings: {:?}", report.rules);
    assert_eq!(
        rows.len(),
        insert_lines.len() + schema_lines.len(),
        "unexpected idempotent finding: {:?}",
        report.rules
    );
}
#[test]
fn parse_failure_locations_report_each_statement_for_every_raw_sql_consumer() {
    for effort in [Effort::Low, Effort::Max] {
        let report = run(".no-mistakes.failures.yml", effort);
        assert_eq!(report.rules.len(), 16, "{:?}", report.rules);
        for rule in RULES.iter().filter(|rule| **rule != RULES[1]) {
            assert_eq!(
                finding_lines(&report, rule),
                [3, 6],
                "{rule}: {:?}",
                report.rules
            );
        }
        // Generated-column parse failures each produce both a raw SQL and a schema finding.
        assert_eq!(finding_lines(&report, RULES[1]), [3, 3, 6, 6]);
        assert_idempotent_insert_and_schema_findings(&report, &[], &[3, 6]);
    }
}
#[test]
fn parse_failure_locations_next_line_suppressions_apply_independently() {
    for effort in [Effort::Low, Effort::Max] {
        assert!(run(".no-mistakes.suppressed.yml", effort).rules.is_empty());
        let report = run(".no-mistakes.one-suppressed.yml", effort);
        assert_eq!(report.rules.len(), 8, "{:?}", report.rules);
        assert!(report.rules.iter().all(|row| row.line == 6));
        assert_eq!(finding_lines(&report, RULES[1]), [6, 6]);
        assert_idempotent_insert_and_schema_findings(&report, &[], &[6]);
    }
}
#[test]
fn parse_failure_locations_preserve_valid_sibling_rule_findings() {
    let report = run(".no-mistakes.valid-siblings.yml", Effort::Max);
    for (rule, expected) in [
        (RULES[0], vec![3, 4, 5, 6]),
        (RULES[2], vec![3, 5, 7]),
        (RULES[3], vec![3, 4, 5, 6]),
        (RULES[4], vec![3, 4, 5, 6, 7]),
        (RULES[5], vec![3, 4, 5, 6]),
    ] {
        assert_eq!(
            report
                .rules
                .iter()
                .filter(|row| row.rule == rule)
                .map(|row| row.line)
                .collect::<Vec<_>>(),
            expected,
            "{rule}: {:?}",
            report.rules
        );
    }
    assert_eq!(finding_lines(&report, RULES[1]), [3, 3, 4, 5, 5, 6]);
    assert_idempotent_insert_and_schema_findings(&report, &[], &[3, 5]);
}
#[test]
fn parse_failure_locations_raw_advice_uses_each_failure_start_and_directive() {
    for effort in [Effort::Low, Effort::Max] {
        let report = run(".no-mistakes.uuid-advice.yml", effort);
        assert!(report.rules.is_empty());
        assert_eq!(
            report
                .advisories
                .iter()
                .map(|row| row.line)
                .collect::<Vec<_>>(),
            [3, 6]
        );
        let report = run(".no-mistakes.uuid-one-suppressed-advice.yml", effort);
        assert!(report.rules.is_empty());
        assert_eq!(
            report
                .advisories
                .iter()
                .map(|row| row.line)
                .collect::<Vec<_>>(),
            [6]
        );
    }
}
#[test]
fn parse_failure_locations_idempotent_insert_keeps_dml_findings_separate_from_schema_warnings() {
    let report = run(".no-mistakes.insert-failures.yml", Effort::Max);
    assert_eq!(
        finding_lines(&report, "postgres-idempotent-insert"),
        [3, 3, 4],
        "{:?}",
        report.rules
    );
    assert_idempotent_insert_and_schema_findings(&report, &[3], &[3, 4]);

    let mixed = run(".no-mistakes.mixed-insert.yml", Effort::Max);
    assert_eq!(finding_lines(&mixed, "postgres-idempotent-insert"), [3]);
    assert_idempotent_insert_and_schema_findings(&mixed, &[], &[3]);
}

#[test]
fn parse_failure_locations_idempotency_retains_recovered_insert_proof() {
    let recovered = run(".no-mistakes.recovered-insert.yml", Effort::Max);
    // The INSERT proof survives; the separate invalid recursive view is a schema gap.
    assert_eq!(
        finding_lines(&recovered, "postgres-idempotent-insert"),
        [3],
        "{:?}",
        recovered.rules
    );
    assert_idempotent_insert_and_schema_findings(&recovered, &[], &[3]);

    let trigger = run(".no-mistakes.failed-trigger.yml", Effort::Max);
    assert_eq!(finding_lines(&trigger, "postgres-idempotent-insert"), [1]);
    assert_idempotent_insert_and_schema_findings(&trigger, &[], &[1]);

    let report = run(".no-mistakes.recovered-and-failed-insert.yml", Effort::Max);
    assert_eq!(
        finding_lines(&report, "postgres-idempotent-insert"),
        [3, 4, 4],
        "{:?}",
        report.rules
    );
    assert_idempotent_insert_and_schema_findings(&report, &[4], &[3, 4]);
}
