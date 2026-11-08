use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::process::Command;

const RULE: &str = "postgres-column-check-constraints";

#[test]
fn postgres_column_check_constraints_check_json_snapshot() {
    let cases = [
        ("fail", "fail", "catalog.yml"),
        ("pass", "pass", "catalog.yml"),
        ("report", "fail", "report.yml"),
        ("severity_advisory", "fail", "severity-advisory.yml"),
        ("mixed_severities", "fail", "mixed-severities.yml"),
        ("allow", "fail", "allow.yml"),
        ("stale_allow", "fail", "stale-allow.yml"),
        ("off", "fail", "off.yml"),
        ("custom", "fail", "custom.yml"),
        ("excluded", "fail", "excluded.yml"),
        ("different_pattern", "fail", "different-pattern.yml"),
        ("aliases", "fail", "aliases.yml"),
        ("project", "fail", "project.yml"),
        ("no_catalog", "no-catalog", "no-catalog.yml"),
        ("empty_options", "no-catalog", "empty.yml"),
        ("missing_catalog", "missing-catalog", "catalog.yml"),
        ("file_disable", "file-disable", "catalog.yml"),
        ("line_disable", "line-disable", "catalog.yml"),
        ("next_line_disable", "next-line-disable", "catalog.yml"),
        ("bad_reason", "fail", "bad-reason.yml"),
        ("invalid_allow", "fail", "invalid-allow.yml"),
        ("duplicate_allow", "fail", "duplicate-allow.yml"),
        ("bad_mode", "fail", "bad-mode.yml"),
        ("bad_type", "fail", "bad-type.yml"),
        ("bad_predicate", "fail", "bad-predicate.yml"),
        ("bad_glob", "fail", "bad-glob.yml"),
        ("empty_pattern", "fail", "empty-pattern.yml"),
        ("empty_types", "fail", "empty-types.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", config),
        )
    });
    assert_saved_fixture("postgres_column_check_constraints", &cases);
}

#[test]
fn postgres_column_check_constraints_keeps_safe_and_unsafe_controls() {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(rules_path(RULE, "fixture", "fail"))
        .arg("--config")
        .arg(rules_path(RULE, "configs", "catalog.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"], serde_json::json!([]));
    let targets: std::collections::BTreeSet<_> = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    for column in [
        "ge_count",
        "reverse_count",
        "gt_count",
        "and_count",
        "nested_count",
        "quoted_count",
        "upper_count",
        "no_inherit_count",
        "catalog_gt_count",
        "catalog_reverse_count",
        "CASE_count",
        "text_count",
        "eq_count",
        "positive_count",
        "strict_count",
        "reverse_strict_count",
        "constant_count",
    ] {
        assert!(
            !targets.contains(format!("column:metrics.{column}").as_str()),
            "{column}"
        );
    }
    for column in [
        "missing_count",
        "or_count",
        "not_count",
        "wrong_count",
        "negative_count",
        "unvalidated_count",
        "cast_count",
        "case_count",
        "broken_count",
        "null_count",
        "small_count",
        "big_count",
        "alias_count",
        "int_count",
        "wrong_case_count",
        "malformed_count",
        "trailing_count",
        "not_enforced_count",
        "invalid_token_count",
        "not_check_count",
        "string_count",
        "function_count",
        "overflow_count",
        "text_cast_count",
        "Needs_count",
        "negative_equal_count",
        "weak_strict_count",
        "weak_reverse_count",
    ] {
        assert!(
            targets.contains(format!("column:metrics.{column}").as_str()),
            "{column}"
        );
    }
    assert_eq!(targets.len(), 28);
    let quoted = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["target"] == "column:metrics.Needs_count")
        .unwrap();
    assert!(quoted["message"]
        .as_str()
        .unwrap()
        .contains(r#"CHECK ("Needs_count" >= 0)"#));
}
