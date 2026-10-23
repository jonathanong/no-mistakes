use std::{path::PathBuf, process::Command};

#[test]
fn git_identity_literal_prefixes_fail_check_with_structured_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-git-identity-mutation/fixture/literal-prefixes");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(report["rules"], serde_json::to_value(expected).unwrap());
    assert_eq!(report["rules"].as_array().unwrap().len(), 33);
    assert_eq!(report["warnings"], serde_json::json!([]));
}
