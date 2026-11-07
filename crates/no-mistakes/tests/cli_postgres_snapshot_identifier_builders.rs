use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::process::Command;

#[test]
fn postgres_identifier_exact_builders_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-builders",
    );
    for (shape, config) in [
        ("clean", ".no-mistakes.yml"),
        ("negative", ".no-mistakes-negative.yml"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let findings = report["rules"].as_array().unwrap();
        if shape == "clean" {
            assert!(output.status.success(), "{report}");
            assert!(findings.is_empty(), "{report}");
        } else {
            for name in [
                "rawStatement",
                "mutatedStatement",
                "escapedStatement",
                "conditionalStatement",
                "rawChainStatement",
            ] {
                assert!(
                    findings.iter().any(|finding| finding["target"] == name),
                    "missing {name}: {report}"
                );
            }
        }
        let name = format!("postgres_identifier_exact_builders_{shape}");
        let cases = [("max", root.clone(), root.join(config))];
        if shape == "clean" {
            assert_saved_fixture_clean(&name, &cases);
        } else {
            assert_saved_fixture(&name, &cases);
        }
    }
}
