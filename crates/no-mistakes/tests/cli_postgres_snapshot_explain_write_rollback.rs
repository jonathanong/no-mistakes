use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_explain_write_rollback_check_json_snapshot() {
    let rule = "postgres-explain-write-rollback";
    let example = rules_path(rule, "fixture", "example");
    let cases = rules_path(rule, "fixture", "cases");
    let reports = assert_saved_fixture_report(
        "postgres_explain_write_rollback",
        &[
            ("autocommit", example.clone(), example.join(".no-mistakes.yml")),
            ("transactions", cases.clone(), cases.join(".no-mistakes.yml")),
            (
                "unanalyzable_report",
                cases.clone(),
                rules_path(rule, "configs", "unanalyzable-report.yml"),
            ),
            (
                "unanalyzable_ignore",
                cases.clone(),
                rules_path(rule, "configs", "unanalyzable-ignore.yml"),
            ),
            ("disabled", cases.clone(), rules_path(rule, "configs", "disabled.yml")),
            ("no_rule", cases, rules_path(rule, "configs", "no-rules.yml")),
        ],
        &[],
        &[],
    );
    assert_eq!(reports["autocommit"]["exitCode"], 1);
    assert_eq!(reports["transactions"]["exitCode"], 1);
    let findings = reports["transactions"]["report"]["rules"].as_array().unwrap();
    let mut transaction_lines: Vec<_> = findings
        .iter()
        .filter(|finding| finding["file"] == "benchmarks/transactions.sql")
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    transaction_lines.sort_unstable();
    assert_eq!(
        transaction_lines,
        [1, 2, 7, 11, 23, 36, 41, 74, 86, 88, 88]
    );
    let mut set_config_lines: Vec<_> = findings
        .iter()
        .filter(|finding| finding["file"] == "benchmarks/set-config.sql")
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    set_config_lines.sort_unstable();
    assert_eq!(set_config_lines, [2, 6, 13, 18, 28, 32]);
    let mut regression_lines: Vec<_> = findings
        .iter()
        .filter(|finding| finding["file"] == "benchmarks/read-only-and-cte.sql")
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    regression_lines.sort_unstable();
    assert_eq!(regression_lines, [2, 8, 14, 25, 33]);
    let advisories = reports["transactions"]["report"]["advisories"]
        .as_array()
        .unwrap();
    assert_eq!(advisories.len(), 4);
    assert!(advisories
        .iter()
        .all(|item| item["target"] == "unanalyzable"));
    let mut regression_advisories: Vec<_> = advisories
        .iter()
        .filter(|item| item["file"] == "benchmarks/read-only-and-cte.sql")
        .map(|item| item["line"].as_u64().unwrap())
        .collect();
    regression_advisories.sort_unstable();
    assert_eq!(regression_advisories, [35, 36, 37]);
    assert!(advisories
        .iter()
        .any(|item| item["file"] == "benchmarks/unparsed.sql"));
    assert_eq!(findings.len(), 25);
    assert!(reports["unanalyzable_report"]["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["file"] == "benchmarks/unparsed.sql"
            && finding["target"] == "unanalyzable"));
    assert!(reports["unanalyzable_ignore"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(reports["disabled"]["exitCode"], 0);
    assert_eq!(reports["no_rule"]["exitCode"], 0);
}

#[test]
fn postgres_explain_write_rollback_requires_explicit_scope() {
    let rule = "postgres-explain-write-rollback";
    let root = rules_path(rule, "fixture", "cases");
    let config = rules_path(rule, "configs", "missing-scope.yml");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .args(["--jobs", "1", "--lock-timeout", "0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("options.sqlInclude"));
}
