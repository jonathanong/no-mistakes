#[path = "common/saved_fixture.rs"]
mod saved_fixture;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_tracked_knip(name: &str) -> tempfile::TempDir {
    let fixture = saved_fixture::materialize("rules", name);
    git(fixture.path(), &["init", "-q", "--initial-branch=main"]);
    git(fixture.path(), &["add", "."]);
    fixture
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/config-path-references")
            .join(scenario),
    )
}

fn check_fixture_config(root: &Path) -> Output {
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

const RULE: &str = "config-path-references";

#[test]
fn config_path_references_presets_fail() {
    let root = fixture("presets-fail");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("missing-app"), "{body}");
}

#[test]
fn config_path_references_presets_pass() {
    let root = fixture("presets-pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn config_path_references_presets_json_has_rule_id() {
    let root = fixture("presets-fail");
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
fn config_path_references_knip_workspace_fixtures_pass() {
    let fixture = git_tracked_knip("config-path-references/knip-workspace-fixtures-pass");
    let out = check_fixture_config(fixture.path());
    let body = stdout(&out);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "exit non-zero: {body} {err}");
}

#[test]
fn config_path_references_knip_workspace_fixtures_fail() {
    let fixture = git_tracked_knip("config-path-references/knip-workspace-fixtures-fail");
    let out = check_fixture_config(fixture.path());
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(
        body.contains(
            "backend/dependency-cruiser-rules/__tests__/fixtures/build-insert-query/missing.mts"
        ),
        "{body}"
    );
}
