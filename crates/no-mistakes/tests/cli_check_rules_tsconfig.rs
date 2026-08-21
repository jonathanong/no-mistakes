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
fn tsconfig_file_coverage_fails_for_an_orphan_file() {
    let root = fixture("tsconfig-file-coverage", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("tsconfig-file-coverage"), "{body}");
    assert!(body.contains("orphan.ts"), "{body}");
}

#[test]
fn tsconfig_file_coverage_passes_when_sources_are_included() {
    let root = fixture("tsconfig-file-coverage", "pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn tsconfig_file_coverage_cli_fails_for_an_orphan_file() {
    let root = fixture("tsconfig-file-coverage", "fail");
    let output = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&output);
    assert!(!output.status.success(), "expected exit 1");
    assert!(body.contains("tsconfig-file-coverage"), "{body}");
    assert!(body.contains("orphan.ts"), "{body}");
}

#[test]
fn tsconfig_file_coverage_allow_and_auxiliary_pass() {
    for scenario in ["allow", "auxiliary"] {
        let root = fixture("tsconfig-file-coverage", scenario);
        let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
        assert!(findings.is_empty(), "{scenario}: {findings:?}");
    }
}
