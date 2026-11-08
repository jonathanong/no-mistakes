use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/unused-postgres-disable-directives/fixture")
}

#[test]
fn check_json_snapshot_postgres_unused_disables() {
    let root = root();
    let cases = [
        ("report", ".no-mistakes.yml"),
        ("advise", "advise.yml"),
        ("ignore", "ignore.yml"),
        ("clean", "clean.yml"),
        ("low", "low.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), root.join(config)));
    crate::check_json_snapshot::assert_saved_fixture("postgres_unused_disables", &cases);
}

#[test]
fn postgres_unused_disables_cover_each_statement_rule_and_skip_unexecuted_text() {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root())
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = report["rules"].as_array().unwrap();
    for rule in [
        "postgres-required-predicates",
        "postgres-bounded-statements",
        "postgres-no-transformed-uuid-predicates",
        "postgres-lock-ordering",
        "postgres-lock-written-rows-only",
    ] {
        let unused = rows
            .iter()
            .filter(|row| row["rule"] == "unused-disable-directive" && row["target"] == rule)
            .count();
        assert_eq!(unused, 3, "{rule}: {report}");
    }
    assert_eq!(rows.len(), 15, "{report}");
}

#[test]
fn check_json_snapshot_postgres_used_disables() {
    let root = root().parent().unwrap().join("used");
    crate::check_json_snapshot::assert_saved_fixture(
        "postgres_used_disables",
        &[("used", root.clone(), root.join(".no-mistakes.yml"))],
    );
}

#[test]
fn postgres_used_disables_are_consumed_by_each_statement_rule() {
    let root = root().parent().unwrap().join("used");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root)
        .args(["--format", "json", "--include-suppressed"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
    assert_eq!(
        report["suppressed"].as_array().unwrap().len(),
        5,
        "{report}"
    );
}

#[test]
fn check_json_snapshot_postgres_disables_in_helpers_and_variants() {
    let root = root().parent().unwrap().join("edges");
    let cases = [("max", ".no-mistakes.yml"), ("low", "low.yml")]
        .map(|(label, config)| (label, root.clone(), root.join(config)));
    crate::check_json_snapshot::assert_saved_fixture("postgres_disables_in_helpers", &cases);
}
