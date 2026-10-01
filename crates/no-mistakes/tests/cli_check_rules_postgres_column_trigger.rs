use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-column-requires-trigger/fixture")
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

const RULE: &str = "postgres-column-requires-trigger";

#[test]
fn postgres_column_requires_trigger_passes_when_the_trigger_maintains_the_column() {
    let out = check("pass");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_column_requires_trigger_fails_when_the_column_has_no_trigger() {
    let out = check("fail");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(
        body.contains("table has column updated_at but no BEFORE UPDATE FOR EACH ROW trigger executing fn_touch_updated_at()"),
        "{body}"
    );
}

#[test]
fn postgres_column_requires_trigger_reports_stale_allow_entries() {
    let out = check("fail-stale-allow");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(
        body.contains("stale postgres-column-requires-trigger allow entry: table:missing"),
        "{body}"
    );
}

#[test]
fn postgres_column_requires_trigger_json_has_rule_id() {
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
