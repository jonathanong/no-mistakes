use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join("fixture")
            .join(scenario),
    )
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(name))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn version_pin_consistency_fails_on_mismatch() {
    let root = fixture("version-pin-consistency", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("version-pin-consistency"), "{body}");
    assert!(body.contains("version mismatch"), "{body}");
}

#[test]
fn version_pin_consistency_passes_when_pins_match() {
    let root = fixture("version-pin-consistency", "pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn version_pin_consistency_skips_when_files_absent() {
    let root = fixture("version-pin-consistency", "skip-absent");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn version_pin_consistency_cli_fails_on_mismatch() {
    let root = fixture("version-pin-consistency", "fail");
    let output = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&output);
    assert!(!output.status.success(), "expected exit 1");
    assert!(body.contains("version-pin-consistency"), "{body}");
    assert!(body.contains("version mismatch"), "{body}");
}

#[test]
fn version_pin_consistency_cli_passes_when_pins_match() {
    let root = fixture("version-pin-consistency", "pass");
    let output = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(
        output.status.success(),
        "expected exit 0: {}",
        stdout(&output)
    );
}
