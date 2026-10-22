use super::*;

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
fn non_adopting_low_preserves_findings_and_reports_fallback() {
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
    let mut default_json = check_runner::json_value(&default);
    assert_eq!(
        check_runner::json_value(&report(config, Some(Effort::Max))),
        default_json
    );

    let low = report(config, Some(Effort::Low));
    assert_eq!(
        low.warnings,
        [
            "postgres-no-mutable-sql-registry: requested effort low; effective effort max because this rule has no low implementation",
            "postgres-no-unbounded-sql-identifier: requested effort low; effective effort max because this rule has no low implementation",
        ]
    );
    assert!(low.analysis_fallbacks.is_empty());
    let mut low_json = check_runner::json_value(&low);
    assert!(low_json.get("analysisFallbacks").is_none());
    // After removing fallback warnings, the low-tier findings match exactly.
    default_json.as_object_mut().unwrap().remove("warnings");
    low_json.as_object_mut().unwrap().remove("warnings");
    assert_eq!(low_json, default_json);
}
