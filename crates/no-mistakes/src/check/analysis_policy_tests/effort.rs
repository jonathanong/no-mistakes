use super::*;

#[test]
fn low_default_tags_keep_literal_heads_and_apply_line_suppressions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads");
    let results = check_runner::run_all_with_options(
        root,
        None,
        None,
        false,
        false,
        check_runner::AnalysisOptions::default(),
    )
    .unwrap();
    assert_eq!(
        results
            .rules
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [("default-tags.mts", 7), ("default-tags.mts", 9)]
    );
    assert!(results.warnings.is_empty());
}

#[test]
fn low_ignored_tag_warning_is_structured_and_non_failing() {
    let low = report("sql-tags.yml", None);
    assert!(low.rules.is_empty());
    assert!(low.advisories.is_empty());
    assert_eq!(low.warnings.len(), 1);
    assert!(low.warnings[0].contains("trustedSqlTags is deprecated and ignored at effort low"));
    assert!(!has_failures(&low));
    assert!(report("sql-tags.yml", Some(Effort::Max))
        .warnings
        .is_empty());
}

#[test]
fn sql_explicit_max_is_byte_identical_to_default() {
    for mode in ["report", "advise", "ignore"] {
        let config = format!("sql-{mode}.yml");
        let default = check_runner::json_value(&report(&config, None));
        assert_eq!(
            check_runner::json_value(&report(&config, Some(Effort::Max))),
            default
        );
        let low = report(&config, Some(Effort::Low));
        assert_eq!(low.rules.len(), usize::from(mode == "report"));
        assert_eq!(low.advisories.len(), usize::from(mode == "advise"));
        assert_eq!(has_failures(&low), mode == "report");
        assert!(low.warnings.is_empty());
        for finding in low.rules.iter().chain(&low.advisories) {
            assert!(finding.message.contains("query(\"/* name */ \" + sql)"));
            assert!(finding.message.contains("one local const"));
        }
    }
}

#[test]
fn low_keeps_nonadopter_fallback_and_enables_registered_adopter() {
    let config = "project-no-policy.yml";
    let default = report(config, None);
    assert_eq!(
        default
            .rules
            .iter()
            .map(|finding| (finding.rule.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [
            ("postgres-no-mutable-sql-registry", 4),
            ("postgres-no-unbounded-sql-identifier", 5),
        ]
    );
    let default_json = check_runner::json_value(&default);
    assert_eq!(
        check_runner::json_value(&report(config, Some(Effort::Max))),
        default_json
    );

    let low = report(config, Some(Effort::Low));
    assert_eq!(
        low.warnings,
        [
            "postgres-no-mutable-sql-registry: requested effort low; effective effort max because this rule has no low implementation",
        ]
    );
    assert!(low.analysis_fallbacks.is_empty());
    let low_json = check_runner::json_value(&low);
    assert!(low_json.get("analysisFallbacks").is_none());
    let mutable = |rows: &[no_mistakes::codebase::rules::RuleFinding]| {
        rows.iter()
            .filter(|row| row.rule == "postgres-no-mutable-sql-registry")
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(mutable(&low.rules), mutable(&default.rules));
    assert!(low
        .rules
        .iter()
        .any(|row| row.rule == "postgres-no-unbounded-sql-identifier"
            && row.message.contains("effort low")));
}
