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

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn check_json(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn postgres_required_predicates_json_has_rule_id() {
    let root = fixture("postgres-required-predicates", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("postgres-required-predicates"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_sql_shape_policy_json_has_rule_id() {
    let root = fixture("postgres-sql-shape-policy", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("postgres-sql-shape-policy"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_idempotent_insert_json_has_rule_id() {
    let root = fixture("postgres-idempotent-insert", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("postgres-idempotent-insert"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_idempotent_insert_passes_do_nothing() {
    let root = fixture("postgres-idempotent-insert", "pass-do-nothing");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}
