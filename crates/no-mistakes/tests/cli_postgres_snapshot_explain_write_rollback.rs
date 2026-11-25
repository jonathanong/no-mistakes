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
    assert_eq!(transaction_lines, [1, 2, 7, 11, 23, 57, 59, 59]);
    assert_eq!(findings.len(), 10);
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
