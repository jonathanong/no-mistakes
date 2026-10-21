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
        .args(["--config", ".no-mistakes.yml"])
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn mutable_registry_fails_for_cross_file_member_writes() {
    let root = fixture("postgres-no-mutable-sql-registry", "fail-member-write");
    let out = check(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(body.contains("postgres-no-mutable-sql-registry"), "{body}");
    assert!(body.contains("query.ts"), "{body}");
    assert!(body.contains("tables.ts"), "{body}");
}

#[test]
fn unbounded_identifier_fails_for_process_env() {
    let root = fixture("postgres-no-unbounded-sql-identifier", "fail-env");
    let out = check(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(
        body.contains("postgres-no-unbounded-sql-identifier"),
        "{body}"
    );
    assert!(body.contains("process.env"), "{body}");
}

#[test]
fn switch_validator_and_literal_registry_pass() {
    for (rule, scenario) in [
        ("postgres-no-unbounded-sql-identifier", "pass-switch"),
        ("postgres-no-mutable-sql-registry", "pass-literal-registry"),
    ] {
        let root = fixture(rule, scenario);
        let out = check(&root);
        assert!(out.status.success(), "{rule}/{scenario}: {}", stdout(&out));
    }
}
