use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/conflict-ordering/cases")
            .join(scenario),
    )
}

fn check(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn postgres_conflict_ordering_cli_reports_missing_canonical_order() {
    let output = check(&fixture("fail-missing-order"));
    let body = stdout(&output);
    assert!(!output.status.success(), "expected exit 1: {body}");
    assert!(body.contains("postgres-conflict-ordering"), "{body}");
    assert!(body.contains("missing-canonical-order"), "{body}");
}

#[test]
fn postgres_conflict_ordering_cli_accepts_a_catalog_ordered_writer() {
    let output = check(&fixture("pass-sql-include"));
    assert!(
        output.status.success(),
        "exit non-zero: {}",
        stdout(&output)
    );
}
