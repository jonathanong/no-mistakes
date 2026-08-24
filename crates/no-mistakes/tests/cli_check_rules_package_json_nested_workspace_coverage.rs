use std::path::PathBuf;
use std::process::Command;

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/package-json-nested-workspace-coverage/missing"),
    )
}

#[test]
fn check_reports_nested_workspace_coverage_with_the_same_rule_id_as_the_node_api() {
    let root = fixture();
    let output = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes")))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "expected a finding: {body}");
    assert!(
        body.contains("package-json-nested-workspace-coverage"),
        "{body}"
    );
}
