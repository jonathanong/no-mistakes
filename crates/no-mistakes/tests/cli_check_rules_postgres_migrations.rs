use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join("fixture")
            .join(scenario),
    )
}

fn check_fixture_config(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn postgres_fk_index_fails_without_a_leading_index() {
    let root = fixture("postgres-fk-index", "fail");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("postgres-fk-index"), "{body}");
    assert!(body.contains("comments.post_id"), "{body}");
}

#[test]
fn postgres_fk_index_passes_with_a_btree_index() {
    let root = fixture("postgres-fk-index", "pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_constraint_validate_fails_without_validate() {
    let root = fixture("postgres-constraint-validate", "fail-missing");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("postgres-constraint-validate"), "{body}");
}

#[test]
fn postgres_constraint_validate_passes_when_paired() {
    let root = fixture("postgres-constraint-validate", "pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_constraint_validate_passes_when_not_valid_is_inside_do() {
    let root = fixture("postgres-constraint-validate", "pass-do-block");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_constraint_validate_fails_when_do_block_add_is_unvalidated() {
    let root = fixture("postgres-constraint-validate", "fail-do-missing");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("postgres-constraint-validate"), "{body}");
}
