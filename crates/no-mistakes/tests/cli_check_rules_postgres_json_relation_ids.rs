use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-json-relation-ids/fixture")
}

fn check(effort: &str) -> std::process::Output {
    configured_check("no-mistakes.low.yml", effort)
}

fn configured_check(config: &str, effort: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root())
        .args(["--config", config, "--effort", effort, "--format", "json"])
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

#[test]
fn missing_catalog_ordinals_remain_advisory_at_both_efforts() {
    for effort in ["low", "max"] {
        let output = configured_check("no-mistakes.missing-ordinals.yml", effort);
        assert!(output.status.success(), "{output:?}");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["rules"], serde_json::json!([]));
        let advisories = report["advisories"].as_array().unwrap();
        assert_eq!(advisories.len(), 1, "{report}");
        assert_eq!(advisories[0]["line"], 2);
        assert_eq!(advisories[0]["target"], "table:orders");
    }
}
