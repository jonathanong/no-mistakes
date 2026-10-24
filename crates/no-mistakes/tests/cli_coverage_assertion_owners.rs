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
    ] {
        assert!(
            !findings.iter().any(|finding| finding["target"] == accepted),
            "{value}"
        );
    }
}
