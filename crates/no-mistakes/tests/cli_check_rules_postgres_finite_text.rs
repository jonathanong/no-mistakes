use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-finite-text-columns/fixture")
            .join(scenario),
    )
}

fn check(root: &PathBuf, json: bool) -> Output {
    let mut command = Command::new(bin());
    command
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"));
    if json {
        command.args(["--format", "json"]);
    }
    command.output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const RULE: &str = "postgres-finite-text-columns";

#[test]
fn postgres_finite_text_columns_passes_an_open_text_column() {
    let root = fixture("pass");
    let out = check(&root, false);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_finite_text_columns_fails_a_pinned_status() {
    let root = fixture("fail");
    let out = check(&root, false);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(
        body.contains("text column holds a fixed set of values ('draft', 'sent', 'paid')"),
        "{body}"
    );
}

#[test]
fn postgres_finite_text_columns_fails_a_stale_allow_entry() {
    let root = fixture("fail-stale-allow");
    let out = check(&root, false);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(
        body.contains("stale postgres-finite-text-columns allow entry"),
        "{body}"
    );
}

#[test]
fn postgres_finite_text_columns_json_has_rule_id() {
    let root = fixture("fail");
    let out = check(&root, true);
    let body = stdout(&out);
    assert!(body.contains(RULE), "{body}");
    assert!(!out.status.success());
    let again = check(&root, true);
    assert_eq!(stdout(&out), stdout(&again));
}

#[test]
fn postgres_finite_text_columns_cli_honors_jsonc_suppression_directives() {
    for scenario in ["line-disable", "next-line-disable", "file-disable"] {
        let root = fixture(scenario);
        let out = check(&root, false);
        assert!(
            out.status.success(),
            "fixture {scenario} should suppress its finding: {}",
            stdout(&out)
        );
    }
}
