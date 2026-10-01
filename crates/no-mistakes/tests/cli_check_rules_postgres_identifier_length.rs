use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-identifier-length/fixture")
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

const RULE: &str = "postgres-identifier-length";

#[test]
fn postgres_identifier_length_passes_a_63_byte_name() {
    let root = fixture("pass");
    let out = check(&root, false);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_identifier_length_fails_a_long_index_name() {
    let root = fixture("fail");
    let out = check(&root, false);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(
        body.contains(
            "identifier \"idx_invoice_line_items_account_id_created_at_status_currency_code\" is 65 bytes"
        ),
        "{body}"
    );
}

#[test]
fn postgres_identifier_length_json_has_rule_id() {
    let root = fixture("fail");
    let out = check(&root, true);
    let body = stdout(&out);
    assert!(body.contains(RULE), "{body}");
    assert!(!out.status.success());
    let again = check(&root, true);
    assert_eq!(stdout(&out), stdout(&again));
}

#[test]
fn postgres_identifier_length_next_line_suppression_passes() {
    let root = fixture("pass-suppressed");
    let out = check(&root, false);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_identifier_length_line_suppression_passes() {
    let root = fixture("suppress-line");
    let out = check(&root, false);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_identifier_length_file_suppression_passes() {
    let root = fixture("suppress-file");
    let out = check(&root, false);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}
