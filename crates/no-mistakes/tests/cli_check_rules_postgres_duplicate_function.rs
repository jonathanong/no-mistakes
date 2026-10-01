use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-duplicate-function-body/fixture")
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

const RULE: &str = "postgres-duplicate-function-body";

#[test]
fn postgres_duplicate_function_body_passes_for_a_single_function() {
    let out = check("pass");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_duplicate_function_body_fails_when_bodies_match() {
    let out = check("fail");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(
        body.contains(
            "function body duplicates 2 other function(s) after normalising names and literals"
        ),
        "{body}"
    );
}

#[test]
fn postgres_duplicate_function_body_reports_stale_allow_entries() {
    let out = check("fail-stale-allow");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(
        body.contains(
            "stale postgres-duplicate-function-body allow entry: function:fn_orders_audit"
        ),
        "{body}"
    );
}

#[test]
fn postgres_duplicate_function_body_json_has_rule_id() {
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
