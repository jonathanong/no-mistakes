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
fn github_actions_composite_step_schema_fails_for_timeout_minutes() {
    let root = fixture("github-actions-composite-step-schema", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");

    assert!(!findings.is_empty(), "expected findings");
    assert!(
        body.contains("github-actions-composite-step-schema"),
        "{body}"
    );
    assert!(body.contains("timeout-minutes"), "{body}");
    assert!(body.contains("action.yml"), "{body}");
}

#[test]
fn github_actions_composite_step_schema_passes_for_allowed_keys() {
    let root = fixture("github-actions-composite-step-schema", "pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn github_actions_composite_step_schema_cli_fails_for_timeout_minutes() {
    let root = fixture("github-actions-composite-step-schema", "fail");
    let output = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&output);

    assert!(!output.status.success(), "expected exit 1");
    assert!(
        body.contains("github-actions-composite-step-schema"),
        "{body}"
    );
    assert!(body.contains("timeout-minutes"), "{body}");
}

#[test]
fn github_actions_composite_step_schema_ignores_description_prose() {
    let root = fixture("github-actions-composite-step-schema", "description");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn github_actions_composite_step_schema_ignores_docker_actions() {
    let root = fixture("github-actions-composite-step-schema", "docker");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn github_actions_composite_step_schema_flags_invalid_yaml() {
    let root = fixture("github-actions-composite-step-schema", "invalid");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(body.contains("invalid YAML"), "{body}");
}
