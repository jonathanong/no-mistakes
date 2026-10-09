use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture")
            .join(name),
    )
}

#[test]
fn configured_banned_function_shape_reports_sql_and_embedded_calls() {
    let root = fixture("banned-functions");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config"])
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding["file"] == "positions.sql"
            && finding["target"] == "banned-function-call"));
    assert!(findings
        .iter()
        .any(|finding| finding["file"] == "embedded.test.ts"));
    assert!(
        findings
            .iter()
            .all(|finding| finding["file"] != "disabled.sql"
                && finding["file"] != "disabled-file.sql")
    );
}

#[test]
fn empty_function_list_is_a_cli_configuration_error() {
    let root = fixture("banned-functions-empty");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config"])
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        diagnostic.contains("shapeOptions.bannedFunctionCall.functions"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("nonempty function list"),
        "{diagnostic}"
    );
}

#[test]
fn clause_scoped_function_bans_preserve_cli_diagnostics_and_suppression() {
    let root = fixture("clause-api");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config"])
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 5, "{report}");
    assert!(findings
        .iter()
        .all(|finding| finding["rule"] == "postgres-sql-shape-policy"
            && finding["target"] == "banned-function-call"));
    let scoped = findings
        .iter()
        .filter(|finding| {
            finding["message"]
                .as_str()
                .unwrap()
                .contains("uuidv7() is banned")
        })
        .collect::<Vec<_>>();
    assert_eq!(scoped.len(), 4);
    assert!(scoped.iter().all(|finding| finding["message"]
        .as_str()
        .unwrap()
        .ends_with("; compute a stable bound once per statement")));
    assert!(findings
        .iter()
        .all(|finding| finding["line"].as_u64().unwrap() < 12
            && finding["file"] == "sql/queries.sql"));
}
