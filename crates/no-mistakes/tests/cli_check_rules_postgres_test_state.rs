//! The generic check command owns severity, overrides, and suppressions.
use std::path::PathBuf;
use std::process::Command;

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-test-state-mutation/fixture"),
    )
}

#[test]
fn test_state_mutations_use_the_check_pipeline() {
    let root = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config"])
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{body}");
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert!(!findings.is_empty(), "{body}");
    for finding in findings {
        assert_eq!(finding["rule"], "postgres-no-test-state-mutation");
        let file = finding["file"].as_str().unwrap();
        assert!(!file.contains("suppressed"), "{finding}");
        assert!(
            file.starts_with("tests/") || file.starts_with("helpers/"),
            "{finding}"
        );
    }
    for target in ["pg_sleep", "session_replication_role", "CREATE TABLE"] {
        assert!(
            findings.iter().any(|finding| finding["target"] == target),
            "{body}"
        );
    }
}
