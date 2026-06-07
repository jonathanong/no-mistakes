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
    let yaml = std::fs::read_to_string(root.join(name)).unwrap();
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), &yaml).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

// ── github-actions-pinned-hash ───────────────────────────────────────────────

#[test]
fn github_actions_pinned_hash_fails_for_tag_ref() {
    let root = fixture("github-actions-pinned-hash", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");

    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("github-actions-pinned-hash"), "{body}");
    assert!(body.contains("ci.yml"), "{body}");
}

#[test]
fn github_actions_pinned_hash_passes_for_pinned_workflows() {
    let root = fixture("github-actions-pinned-hash", "pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn github_actions_pinned_hash_cli_fails_for_tag_ref() {
    let root = fixture("github-actions-pinned-hash", "fail");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("github-actions-pinned-hash"), "{body}");
    assert!(body.contains("ci.yml"), "{body}");
}

#[test]
fn github_actions_pinned_hash_passes_for_local_actions() {
    let root = fixture("github-actions-pinned-hash", "local-action-pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}
