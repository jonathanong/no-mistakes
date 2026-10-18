use std::path::PathBuf;
use std::process::Command;

#[test]
fn imported_configured_sql_helper_is_checked_without_owning_http_query() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-executor-provenance/helper-tracing-configured-import",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{report}");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    let finding = &findings[0];
    assert_eq!(
        finding["rule"], "postgres-require-query-annotation",
        "{report}"
    );
    assert_eq!(finding["file"], "src/helper.mts", "{report}");
    assert_eq!(finding["line"], 4, "{report}");
    assert_eq!(finding["target"], "annotation", "{report}");
}
