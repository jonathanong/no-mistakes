use std::path::PathBuf;
use std::process::Command;

#[test]
fn invalid_sql_encoding_is_file_local_and_obeys_all_schema_policies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-regress-sql/fixture/encoding");
    for policy in ["default", "report", "advise", "ignore"] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(format!("{policy}.yml")))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&output.stderr)));
        let diagnostics = |field: &str| {
            report[field]
                .as_array()
                .unwrap()
                .iter()
                .filter(|item| item["file"] == "bad.sql")
                .collect::<Vec<_>>()
        };
        assert_eq!(
            diagnostics("rules").len(),
            if policy == "default" || policy == "report" {
                11
            } else {
                0
            },
            "{policy}: {report}"
        );
        assert_eq!(
            diagnostics("advisories").len(),
            if policy == "advise" { 11 } else { 0 },
            "{policy}: {report}"
        );
        assert!(diagnostics("rules")
            .iter()
            .chain(diagnostics("advisories").iter())
            .all(|item| item["line"] == 1
                && item["message"].as_str().unwrap().contains("invalid UTF-8")));
        // A valid sibling still reaches the real ADD COLUMN policy finding.
        assert!(
            report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["file"] == "good.sql"
                    && item["rule"] == "postgres-no-add-column"
                    && item["line"] == 2),
            "{report}"
        );
        assert!(!output.status.success());
    }
}
