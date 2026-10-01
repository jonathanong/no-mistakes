use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-status-with-lifecycle-timestamps/fixture")
            .join(scenario),
    )
}

fn check(scenario: &str) -> std::process::Output {
    let root = fixture(scenario);
    Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const RULE: &str = "postgres-status-with-lifecycle-timestamps";

#[test]
fn postgres_status_lifecycle_passes_for_a_generated_status() {
    let out = check("pass");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_status_lifecycle_fails_for_a_stored_status() {
    let out = check("fail");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(
        body.contains(
            "table stores status column state next to lifecycle timestamps sent_at, failed_at"
        ),
        "{body}"
    );
}

#[test]
fn postgres_status_lifecycle_reports_stale_allow_entries() {
    let out = check("fail-stale-allow");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(
        body.contains("stale postgres-status-with-lifecycle-timestamps allow entry: table:missing"),
        "{body}"
    );
}

#[test]
fn postgres_status_lifecycle_json_has_rule_id() {
    let root = fixture("fail");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(body.contains(RULE), "{body}");
    assert!(!out.status.success());
}
