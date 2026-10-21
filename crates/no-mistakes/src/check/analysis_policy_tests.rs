use super::*;
use no_mistakes::config::v2::Effort;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rule-analysis-policy/fixture")
}
fn report(config: &str, effort: Option<Effort>) -> check_runner::CheckResults {
    check_runner::run_all_with_options(
        root(),
        Some(config.into()),
        None,
        false,
        false,
        check_runner::AnalysisOptions {
            effort,
            include_sql_verdicts: false,
        },
    )
    .unwrap()
}

#[test]
fn shared_unanalyzable_policies_route_actual_sql_failures_advisories_and_skips() {
    for (mode, failures, advisories) in [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)] {
        let shared = report(&format!("sql-{mode}.yml"), None);
        let legacy = report(&format!("legacy-sql-{mode}.yml"), None);
        assert_eq!(
            check_runner::json_value(&shared),
            check_runner::json_value(&legacy)
        );
        assert_eq!(shared.rules.len(), failures, "{mode}");
        assert_eq!(shared.advisories.len(), advisories, "{mode}");
        assert_eq!(has_failures(&shared), failures != 0, "{mode}");
        assert!(shared.warnings.is_empty());
    }
}

#[test]
fn explicit_max_and_non_adopting_low_are_byte_identical_to_default() {
    for mode in ["report", "advise", "ignore"] {
        let config = format!("sql-{mode}.yml");
        let default = check_runner::json_value(&report(&config, None));
        for effort in [Effort::Max, Effort::Low] {
            assert_eq!(
                check_runner::json_value(&report(&config, Some(effort))),
                default
            );
        }
    }
}

#[test]
fn invalid_legacy_sql_alias_keeps_its_original_parser_error_at_max() {
    let execute = |effort| {
        check_runner::run_all_with_options(
            root(),
            Some("invalid-alias.yml".into()),
            None,
            false,
            false,
            check_runner::AnalysisOptions {
                effort,
                include_sql_verdicts: false,
            },
        )
        .err()
        .unwrap()
    };
    let default = format!("{:#}", execute(None));
    assert_eq!(format!("{:#}", execute(Some(Effort::Max))), default);
    assert!(default.contains("options.unanalyzableSql"));
    assert!(default.contains(" REPORT "));
}

#[test]
fn shared_defaults_preserve_malformed_options_errors_instead_of_panicking() {
    for name in ["malformed-options", "malformed-sequence"] {
        let execute = |config: String| {
            check_runner::run_all_with_options(
                root(),
                Some(config.into()),
                None,
                false,
                false,
                check_runner::AnalysisOptions::default(),
            )
            .err()
            .unwrap()
        };
        let legacy = format!("{:#}", execute(format!("{name}-legacy.yml")));
        let shared = format!("{:#}", execute(format!("{name}.yml")));
        assert_eq!(shared, legacy);
        assert!(shared.contains("invalid options for rule `postgres-require-query-annotation`"));
        assert!(shared.contains("options"));
    }
}
