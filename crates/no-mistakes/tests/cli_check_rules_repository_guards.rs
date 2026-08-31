use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(rule: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(rule)
            .join("fixture")
            .join(scenario),
    )
}

fn check(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

#[test]
fn no_test_git_sha_is_cli_visible() {
    let output = check(&fixture("no-test-git-sha", "fail"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "expected a finding: {stdout}");
    assert!(stdout.contains("no-test-git-sha"), "{stdout}");
}

#[test]
fn no_sparse_checkout_is_cli_visible() {
    let output = check(&fixture("no-sparse-checkout", "fail"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "expected a finding: {stdout}");
    assert!(stdout.contains("no-sparse-checkout"), "{stdout}");
}

#[test]
fn no_sparse_checkout_is_accounted_by_aggregate_suppression_audit() {
    let root = fixture("no-sparse-checkout", "fail");
    let output = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--include-suppressed", "--format", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "fixture also has unsuppressed findings: {stdout}"
    );
    assert!(stdout.contains("no-sparse-checkout"), "{stdout}");
    assert!(stdout.contains("suppressed"), "{stdout}");
}
