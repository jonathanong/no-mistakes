use std::path::PathBuf;
use std::process::Command;

#[test]
fn declared_http_queue_compatibility_has_cli_exit_and_json_parity() {
    for (name, count) in [("pass", 0), ("fail", 1), ("suppression", 1)] {
        let root = no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/rules/declared-payload-compatibility")
                .join(name),
        );
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&output.stderr)));
        assert_eq!(output.status.success(), count == 0, "{report}");
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), count, "{report}");
        assert!(findings
            .iter()
            .all(|f| f["rule"] == "declared-payload-compatibility"));
    }
}
