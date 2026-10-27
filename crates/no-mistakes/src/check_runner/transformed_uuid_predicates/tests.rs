use super::{run_all_with_options, AnalysisOptions};
use no_mistakes::config::v2::Effort;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-no-transformed-uuid-predicates/fixture"),
    )
}

fn run(config: &str, effort: Option<Effort>, verdicts: bool) -> super::CheckResults {
    let root = fixture();
    crate::test_support::without_hygiene_advisories(
        run_all_with_options(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            AnalysisOptions {
                effort,
                include_sql_verdicts: verdicts,
            },
        )
        .unwrap(),
    )
}

// Physical origins of the violating operands in the saved SQL and builder cases.
// The JOIN has two transformed UUID operands, so its line occurs twice.
const EXPECTED_LOCATIONS: &[(&str, usize)] = &[
    ("helper-query.ts", 4),
    ("query.ts", 5),
    ("queries.sql", 3),
    ("queries.sql", 4),
    ("queries.sql", 5),
    ("queries.sql", 6),
    ("queries.sql", 10),
    ("queries.sql", 10),
    ("queries.sql", 15),
    ("queries.sql", 25),
    ("queries.sql", 28),
];

#[test]
fn transformed_uuid_predicates_max_is_default_and_preserves_rewrite_messages() {
    let max = run(".no-mistakes.max.yml", None, false);
    let mut actual: Vec<_> = max
        .rules
        .iter()
        .map(|row| {
            assert_eq!(
                row.rule,
                no_mistakes::codebase::rules::postgres_no_transformed_uuid_predicates::RULE_ID
            );
            (row.file.as_str(), row.line)
        })
        .collect();
    let mut expected = EXPECTED_LOCATIONS.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected, "{:?}", max.rules);
    assert_eq!(max.rules, run(".no-mistakes.yml", None, false).rules);
    assert_eq!(
        max.rules,
        run(".no-mistakes.low-advise.yml", Some(Effort::Max), false).rules
    );
    assert!(max.advisories.is_empty());
    for finding in &max.rules {
        assert!(
            finding.message.contains("Cast the parameter"),
            "{finding:?}"
        );
        assert!(finding.message.contains("precomputed bound"), "{finding:?}");
    }
    for clause in ["WHERE", "JOIN ON", "HAVING"] {
        assert!(max.rules.iter().any(|row| row.message.contains(clause)));
    }
}

#[test]
fn transformed_uuid_predicates_low_routes_uncertainty_without_losing_definite_findings() {
    let max = run(".no-mistakes.max.yml", None, false);
    for (config, findings, advisories) in [
        (".no-mistakes.low-report.yml", EXPECTED_LOCATIONS.len(), 0),
        (".no-mistakes.low-advise.yml", EXPECTED_LOCATIONS.len(), 0),
        (".no-mistakes.low-ignore.yml", EXPECTED_LOCATIONS.len(), 0),
    ] {
        let report = run(config, None, false);
        assert_eq!(report.rules.len(), findings, "{config}: {:?}", report.rules);
        assert_eq!(
            report.advisories.len(),
            advisories,
            "{config}: {:?}",
            report.advisories
        );
        assert_eq!(report.analysis_warning_count, 0);
        assert!(report.analysis_fallbacks.is_empty());
        let definite: Vec<_> = report
            .rules
            .iter()
            .filter(|row| row.target.as_deref() != Some("unanalyzable"))
            .collect();
        assert_eq!(definite.len(), EXPECTED_LOCATIONS.len());
        assert!(
            definite.iter().any(|row| {
                row.file == "helper-query.ts"
                    && row.line == 4
                    && row.target.as_deref() == Some("users.id")
            }),
            "helper SQL finding should retain its physical origin: {definite:#?}"
        );
        for row in definite {
            assert!(max.rules.contains(row), "{row:?}");
        }
    }
    let advise = run(".no-mistakes.low-advise.yml", None, false);
    let detailed = run(".no-mistakes.low-advise.yml", None, true);
    assert_eq!(detailed.rules, advise.rules);
    assert_eq!(detailed.advisories, advise.advisories);
}

#[test]
fn transformed_uuid_predicates_sql_and_executor_suppressions_preserve_unsuppressed_control() {
    for effort in [Effort::Low, Effort::Max] {
        let report = run(".no-mistakes.suppression.yml", Some(effort), false);
        assert_eq!(report.rules.len(), 1, "{effort:?}: {:?}", report.rules);
        assert_eq!(report.rules[0].file, "suppressed.sql");
        assert_eq!(report.rules[0].line, 5);
        assert!(report.advisories.is_empty());
    }
}

#[test]
fn transformed_uuid_predicates_low_suppresses_unknown_owners_at_each_executor_call() {
    let report = run(".no-mistakes.unknown-suppression.yml", None, false);
    assert_eq!(report.rules.len(), 2, "{:?}", report.rules);
    assert!(report.rules.iter().all(|row| {
        row.file == "unknown-suppression.ts" && row.target.as_deref() == Some("unanalyzable")
    }));
    assert_eq!(
        report.rules.iter().map(|row| row.line).collect::<Vec<_>>(),
        [9, 10]
    );
    assert!(report.advisories.is_empty());
}

#[test]
fn transformed_uuid_predicates_raw_sql_uncertainty_remains_advisory_at_both_efforts() {
    for effort in [Effort::Low, Effort::Max] {
        let report = run(".no-mistakes.raw-advice.yml", Some(effort), false);
        assert!(report.rules.is_empty(), "{:?}", report.rules);
        assert_eq!(report.advisories.len(), 2, "{:?}", report.advisories);
        assert!(report
            .advisories
            .iter()
            .all(|row| row.rule == "postgres-no-transformed-uuid-predicates"));
        let detailed = run(".no-mistakes.raw-advice.yml", Some(effort), true);
        assert_eq!(detailed.rules, report.rules);
        assert_eq!(detailed.advisories, report.advisories);
    }
}
