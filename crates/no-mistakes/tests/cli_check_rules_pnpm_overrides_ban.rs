use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-overrides-ban/fixture")
            .join(scenario),
    )
}

fn config(rule: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("config")
        .join(name)
}

fn check(root: &Path, config: &Path) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const RULE: &str = "pnpm-overrides-ban";

#[test]
fn pnpm_overrides_ban_fails_workspace_overrides() {
    let root = fixture("fail-workspace");
    let out = check(
        &root,
        &config(RULE, "pnpm_overrides_ban_fails_workspace_overrides.yml"),
    );
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("pnpm-workspace.yaml"), "{body}");
    assert!(body.contains("overrides"), "{body}");
}

#[test]
fn pnpm_overrides_ban_passes_package_extensions() {
    let root = fixture("pass");
    let out = check(
        &root,
        &config(RULE, "pnpm_overrides_ban_fails_workspace_overrides.yml"),
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn pnpm_overrides_ban_json_has_rule_id() {
    let root = fixture("fail-pnpm-overrides");
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
