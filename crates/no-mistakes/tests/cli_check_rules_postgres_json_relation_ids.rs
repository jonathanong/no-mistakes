use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-json-relation-ids/fixture")
}

fn check(effort: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root())
        .args([
            "--config",
            "no-mistakes.low.yml",
            "--effort",
            effort,
            "--format",
            "json",
        ])
        .output()
        .unwrap()
}

#[test]
fn literal_json_relation_advisories_never_fail_the_cli() {
    for effort in ["low", "max"] {
        let first = check(effort);
        let second = check(effort);
        assert!(first.status.success(), "{first:?}");
        assert!(second.status.success(), "{second:?}");
        assert_eq!(first.stdout, second.stdout);
        let report: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
        assert_eq!(report["rules"], serde_json::json!([]));
        let advisories = report["advisories"].as_array().unwrap();
        assert_eq!(advisories.len(), 2, "{report}");
        assert!(advisories.iter().all(|finding| {
            finding["rule"] == "postgres-json-relation-ids"
                && finding["file"] == "db/fail.sql"
                && finding["target"] == "column:orders.details"
        }));
    }
}

#[test]
fn invalid_json_relation_options_remain_fatal_configuration_errors() {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root())
        .args(["--config", "no-mistakes.duplicate.yml", "--format", "json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("postgres-json-relation-ids option keyColumnMap: duplicate key accountId"));
}
