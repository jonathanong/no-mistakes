use crate::saved_fixture;
use std::path::PathBuf;
use std::process::Command;
fn fixture() -> tempfile::TempDir {
    saved_fixture::materialize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/coverage-producer-assertion-owner/fixture"),
    )
}
#[test]
fn cli_projects_explicit_assertion_ownership() {
    let fixture = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "dependencies",
            "src/producers.mts#crossProcess",
            "--relationship",
            "assertion-owner",
            "--symbols",
            "--format",
            "json",
            "--root",
        ])
        .arg(fixture.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["roots"],
        serde_json::json!(["src/producers.mts#crossProcess"])
    );
    assert!(
        value["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["file"] == "src/assertions.mts"
                && node["symbol"] == "assertResult"
                && node["depth"] == 1
                && node["via"] == serde_json::json!(["assertion-owner"])),
        "{value}"
    );
}
#[test]
fn cli_check_exposes_coverage_producer_findings() {
    let fixture = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(fixture.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = value["rules"].as_array().unwrap();
    assert!(findings.iter().any(
        |finding| finding["rule"] == "coverage-producer-assertion-owner"
            && finding["target"] == "productionOnly"
    ));
    for accepted in [
        "direct",
        "imported",
        "callback",
        "inlineCallback",
        "crossProcess",
        "assertionCtor",
    ] {
        assert!(
            !findings.iter().any(|finding| finding["target"] == accepted),
            "{value}"
        );
    }
}

#[test]
fn cli_reports_generator_uncertainty_without_assertion_proof() {
    let fixture = fixture();
    for (policy, findings, advisories) in [("report", 7, 0), ("advise", 0, 7), ("ignore", 0, 0)] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(fixture.path())
            .arg("--config")
            .arg(fixture.path().join(format!("generators-{policy}.yml")))
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{report}"
        );
        assert_eq!(
            super::policy_advisories::without_hygiene(&report).len(),
            advisories,
            "{report}"
        );
    }
}

#[test]
fn cli_names_coverage_owner_uncertainty_in_human_and_json_output() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/coverage-producer-assertion-owner-uncertainty/fixture");
    for (config, expected) in [
        (
            "dispatch",
            vec!["unresolved dispatch", "src/producer.mts:2", "`dispatch`"],
        ),
        (
            "depth",
            vec!["traversal limit maxDepth=1", "last module src/helper.mts"],
        ),
        (
            "nodes",
            vec!["traversal limit maxNodes=1", "last module src/producer.mts"],
        ),
        (
            "mixed",
            vec![
                "unresolved dispatch",
                "`dispatch`",
                "traversal limit maxDepth=1",
            ],
        ),
        (
            "both",
            vec![
                "traversal limit maxDepth=1",
                "traversal limit maxNodes=2",
                "last module src/helper.mts",
            ],
        ),
        (
            "custom",
            vec![
                "Assert the dispatch result",
                "unresolved dispatch",
                "`dispatch`",
            ],
        ),
        (
            "source",
            vec!["source failure", "src/broken.mts", "Unexpected token"],
        ),
    ] {
        for format in ["human", "json"] {
            let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!("{config}.yml")))
                .args(["--format", format])
                .output()
                .unwrap();
            assert!(!output.status.success());
            let text = if format == "json" {
                let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                let field = if config == "source" {
                    "incomplete"
                } else {
                    "rules"
                };
                report[field][0]["message"].as_str().unwrap().to_string()
            } else {
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )
            };
            for part in &expected {
                assert!(
                    text.contains(part),
                    "{config}/{format}: missing {part}: {text}"
                );
            }
            assert!(!text.contains("unresolved dispatch, source failure, or traversal limit"));
        }
    }
}
