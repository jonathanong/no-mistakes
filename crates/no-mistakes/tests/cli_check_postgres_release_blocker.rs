use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn supported_postgres_ddl_has_no_schema_diagnostics_even_with_report_policy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source/ddl-compat");
    for config in [".no-mistakes.yml", "guarded-do.yml", "guarded-function.yml"] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(&root)
            .args(["--config", config])
            .output()
            .unwrap();
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(output.status.success(), "{report}");
        assert_eq!(report["rules"], serde_json::json!([]));
        assert_eq!(report["advisories"], serde_json::json!([]));
    }
}

#[test]
fn schema_defaults_advise_for_valid_unsupported_sql_and_report_malformed_sql() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/unsupported-policy");
    for (config, findings, advisories) in [
        ("default.yml", 44, 33),
        ("report.yml", 77, 0),
        ("advise.yml", 0, 77),
        ("ignore.yml", 0, 0),
        ("override.yml", 77, 0),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(&root)
            .args(["--config", config])
            .output()
            .unwrap();
        let report: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
        assert_eq!(output.status.success(), findings == 0, "{config}: {report}");
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{config}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}"
        );
        if config == "default.yml" {
            // All eleven rules preserve malformed SQL as failures, independently
            // of the advisory-only valid statements in the same migration.
            assert!(report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .all(|finding| { (4..=7).contains(&finding["line"].as_u64().unwrap()) }));
            assert!(report["advisories"]
                .as_array()
                .unwrap()
                .iter()
                .all(|finding| { (1..=3).contains(&finding["line"].as_u64().unwrap()) }));
        }
    }
}
