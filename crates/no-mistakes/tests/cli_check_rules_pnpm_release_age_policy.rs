use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-release-age-policy/fixture")
            .join(scenario),
    )
}

fn check_fixture_config(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const RULE: &str = "pnpm-release-age-policy";

#[test]
fn pnpm_release_age_policy_fails_exclude() {
    let root = fixture("fail-exclude");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("unknown-package"), "{body}");
}

#[test]
fn pnpm_release_age_policy_passes() {
    let root = fixture("pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn pnpm_release_age_policy_json_has_rule_id() {
    let root = fixture("fail-exclude");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(body.contains(RULE), "{body}");
    assert!(!out.status.success());
}

#[test]
fn pnpm_release_age_policy_filesystem_runner_discovers_files() {
    let root = fixture("fail-exclude");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == no_mistakes::codebase::rules::PNPM_RELEASE_AGE_POLICY),
        "{findings:?}"
    );
}
