use std::path::PathBuf;
use std::process::Command;

#[test]
fn invalid_sql_encoding_is_decoded_lossily_and_keeps_the_valid_sibling() {
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
        assert!(diagnostics("rules").is_empty(), "{policy} rules: {report}");
        assert!(
            diagnostics("advisories").is_empty(),
            "{policy} advisories: {report}"
        );
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
