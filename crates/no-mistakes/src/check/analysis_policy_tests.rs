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
fn explicit_max_stays_identical_and_non_adopting_low_is_a_nonfailing_note() {
    for mode in ["report", "advise", "ignore"] {
        let config = format!("sql-{mode}.yml");
        let default = check_runner::json_value(&report(&config, None));
        assert_eq!(
            check_runner::json_value(&report(&config, Some(Effort::Max))),
            default
        );
        assert!(default.get("analysisFallbacks").is_none());
        let low = report(&config, Some(Effort::Low));
        assert_eq!(has_failures(&low), mode == "report");
        let mut low = check_runner::json_value(&low);
        assert_eq!(low["warnings"], serde_json::json!([
            "postgres-require-query-annotation: requested effort low; effective effort max because this rule has no low implementation"
        ]));
        low["warnings"] = serde_json::json!([]);
        assert_eq!(low, default);
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
        assert_eq!(
            shared,
            concat!(
                "postgres-require-query-annotation option importSpecifier: ",
                "set importSpecifier (or executorNames) to select executor calls; ",
                "set executorNames: [] to scan only SQL files and native SQL ",
                "(see docs/migrations/explicit-postgres-executors.md)"
            )
        );
    }
}

#[test]
fn cli_effort_override_preserves_policy_exit_status_on_saved_sql() {
    for effort in [Effort::Low, Effort::Max] {
        for (policy, expected) in [
            ("report", ExitCode::FAILURE),
            ("advise", ExitCode::SUCCESS),
            ("ignore", ExitCode::SUCCESS),
        ] {
            let status = run(CheckArgs {
                root: root(),
                config: Some(format!("sql-{policy}.yml").into()),
                tsconfig: None,
                effort: Some(effort),
                format: Format::Json,
                json: false,
                include_suppressed: false,
                include_runner_config_deadlines: false,
                include_sql_verdicts: false,
                timings: false,
                verbose_timings: false,
            })
            .unwrap();
            assert_eq!(status, expected, "{effort:?}/{policy}");
        }
    }
}

#[test]
fn inherited_ignored_sql_aliases_preserve_both_cross_file_rule_reports() {
    let baseline = report("project-no-policy.yml", None);
    let findings: Vec<_> = baseline
        .rules
        .iter()
        .map(|finding| (finding.rule.as_str(), finding.line))
        .collect();
    assert_eq!(
        findings,
        [
            ("postgres-no-mutable-sql-registry", 4),
            ("postgres-no-unbounded-sql-identifier", 5),
        ]
    );
    let baseline = check_runner::json_value(&baseline);
    for config in [
        "project-fail.yml",
        "project-previously-ignored.yml",
        "opaque-legacy.yml",
    ] {
        assert_eq!(
            check_runner::json_value(&report(config, None)),
            baseline,
            "{config}"
        );
    }
}

#[test]
fn structured_analysis_fallbacks_are_typed_and_omitted_when_empty() {
    let mut results = check_runner::empty_results([None]);
    assert!(check_runner::json_value(&results)
        .get("analysisFallbacks")
        .is_none());
    results
        .analysis_fallbacks
        .push(no_mistakes::config::v2::AnalysisFallback {
            rule: "test-effort-adopter".into(),
            partner: "test-partner".into(),
            requested: Effort::Low,
            effective: Effort::Max,
        });
    assert_eq!(
        check_runner::json_value(&results)["analysisFallbacks"],
        serde_json::json!([
            {"rule": "test-effort-adopter", "partner": "test-partner", "requested": "low", "effective": "max"}
        ])
    );
}

#[test]
fn canonical_policy_wins_valid_alias_conflicts_on_actual_sql() {
    assert_eq!(
        check_runner::json_value(&report("shared-valid-annotation.yml", None)),
        check_runner::json_value(&report("sql-advise.yml", None))
    );
}
