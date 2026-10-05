use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-lock-ordering/fixture")
            .join(scenario),
    )
}

fn check(scenario: &str) -> Output {
    let root = fixture(scenario);
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn reports_only_the_misspelled_scoped_name_when_opted_in() {
    let out = check("fail-unmatched-executor-names");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert_eq!(body.matches("was not imported from").count(), 1, "{body}");
    assert!(
        body.contains("executorFactoryNames entry `openTransacton`"),
        "{body}"
    );
    assert!(!body.contains("`openTransaction`"), "{body}");
    assert!(!body.contains("`TransactionQuery`"), "{body}");
}

#[test]
fn stays_silent_when_reporting_is_off() {
    let out = check("pass-unmatched-names-off");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn stays_silent_when_every_configured_name_matches() {
    let out = check("pass-unmatched-all-matched");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}
