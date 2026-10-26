use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-plan-lockfile")
        .join(name)
}

fn materialize(name: &str) -> tempfile::TempDir {
    crate::saved_fixture::materialize_path(&fixture_dir(name))
}

fn git_init(root: &Path) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    for (k, v) in [("user.email", "test@test.com"), ("user.name", "Test")] {
        Command::new("git")
            .args(["config", k, v])
            .current_dir(root)
            .output()
            .unwrap();
    }
}

fn git_commit(root: &Path, pathspec: &[&str], message: &str) {
    Command::new("git")
        .arg("add")
        .args(pathspec)
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .output()
        .unwrap();
}

fn copy_saved(src: &Path, dest: &Path) {
    std::fs::copy(src, dest).unwrap();
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn tests_plan_with_lockfile_base_finds_impacted_tests() {
    let dir = materialize("direct-lodash-impact/initial");
    let root = dir.path();
    git_init(root);
    git_commit(root, &["-A"], "initial");
    copy_saved(
        &fixture_dir("direct-lodash-impact/after-pnpm-lock.yaml"),
        &root.join("pnpm-lock.yaml"),
    );

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "should not trigger fallback: {plan:?}"
    );
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected
            .iter()
            .any(|t| t["test_file"].as_str().unwrap().contains("utils.test")),
        "should find utils.test.mts via lodash change: {selected:?}"
    );
}

// Covers the --head branch in analyze_lockfile_changes: new content from git show <head>:.
#[test]
fn tests_plan_lockfile_head_option() {
    let dir = materialize("lockfile-head-option/initial");
    let root = dir.path();
    git_init(root);
    git_commit(root, &["pnpm-lock.yaml"], "initial");
    copy_saved(
        &fixture_dir("lockfile-head-option/after-pnpm-lock.yaml"),
        &root.join("pnpm-lock.yaml"),
    );
    git_commit(root, &["pnpm-lock.yaml"], "v2");
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD~1",
            "--head",
            "HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "should not fallback: {plan:?}"
    );
}

// Covers find_git_root returning None (tempdir without git init): no-baseline warning emitted
// because args.base is None, so analyze_lockfile_changes falls through to the None branch.
// find_git_root is always called but returns None here; git_root falls back to root.
#[test]
fn tests_plan_lockfile_no_git_repo_emits_warning() {
    let dir = materialize("no-git-baseline");
    let root = dir.path();
    // No --base: avoids the CLI's own git-diff phase (which would fail in a non-git dir).
    // analyze_lockfile_changes still calls find_git_root (returns None → git_root = root)
    // and then hits the args.base = None branch, emitting the lockfile-no-baseline warning.
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-no-baseline"),
        "expected lockfile-no-baseline warning: {plan:?}"
    );
}

// Covers analyze_lockfile_changes when base ref is invalid: git_ref_exists returns false
// and a lockfile-no-baseline warning is emitted with fallback_triggered=true.
#[test]
fn tests_plan_invalid_base_ref_falls_back_with_warning() {
    let dir = materialize("invalid-head-ref");
    let root = dir.path();
    git_init(root);
    git_commit(root, &["pnpm-lock.yaml"], "initial");
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "nonexistent-base-xyz",
            "--global-config-fallback=true",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        plan["fallback_triggered"].as_bool().unwrap(),
        "invalid base should trigger fallback: {plan:?}"
    );
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-no-baseline"),
        "expected lockfile-no-baseline warning: {plan:?}"
    );
}

// Covers the transitive-dep fallback: a changed package that is not directly imported
// by any codebase file (no reverse edge in the dep graph) triggers a full-suite fallback.
#[test]
fn tests_plan_transitive_dep_triggers_fallback() {
    let dir = materialize("transitive-debug/initial");
    let root = dir.path();
    git_init(root);
    git_commit(root, &["-A"], "initial");
    // Only debug changed (lodash unchanged) → debug is transitive → fallback
    copy_saved(
        &fixture_dir("transitive-debug/after-pnpm-lock.yaml"),
        &root.join("pnpm-lock.yaml"),
    );

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD",
            "--global-config-fallback=true",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        plan["fallback_triggered"].as_bool().unwrap(),
        "transitive dep change should trigger fallback: {plan:?}"
    );
}

// Covers is_diff_only_mode: when --diff is used without --head in a git repo with
// --base provided, the working tree may still be at the base. Should emit warning.
#[test]
fn tests_plan_diff_only_mode_without_head_emits_warning() {
    let dir = materialize("invalid-head-ref");
    let root = dir.path();
    git_init(root);
    git_commit(root, &["pnpm-lock.yaml"], "initial");
    let diff_path = fixture_dir("diff-only-fallback/lockfile.diff");
    // Run with --diff (diff-only mode) and --base but no --head.
    // is_diff_only_mode detects this and falls back instead of reading stale disk content.
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--diff",
            diff_path.to_str().unwrap(),
            "--base",
            "HEAD",
            "--global-config-fallback=true",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true, "{plan:?}");
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-no-baseline"),
        "expected lockfile-no-baseline warning in diff-only mode: {plan:?}"
    );
}

// Diff-only lockfile analysis cannot identify changed packages, but it must not
// select the full suite unless global fallback is explicitly enabled.
#[test]
fn tests_plan_diff_only_mode_without_global_fallback_flag_does_not_fall_back() {
    let dir = materialize("invalid-head-ref");
    let root = dir.path();
    git_init(root);
    git_commit(root, &["pnpm-lock.yaml"], "initial");
    let diff_path = fixture_dir("diff-only-fallback/lockfile.diff");
    // Explicitly disable global fallback so the policy boundary is regression-tested.
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--diff",
            diff_path.to_str().unwrap(),
            "--base",
            "HEAD",
            "--global-config-fallback=false",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false, "{plan:?}");
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-no-baseline"),
        "expected lockfile-no-baseline warning: {plan:?}"
    );
}

// Covers the newly-added-lockfile path in analyze_lockfile_changes: when a lockfile is
// first introduced on a branch (base doesn't have it), treat the base as empty so all
// packages in the new content are considered added and can be traced to tests.
#[test]
fn tests_plan_newly_added_lockfile_traces_packages() {
    let dir = materialize("newly-added-direct/initial");
    let root = dir.path();
    git_init(root);
    git_commit(root, &["package.json", "src/"], "initial-no-lock");
    copy_saved(
        &fixture_dir("newly-added-direct/pnpm-lock.yaml"),
        &root.join("pnpm-lock.yaml"),
    );

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "should not trigger fallback for newly added lockfile: {plan:?}"
    );
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected
            .iter()
            .any(|t| t["test_file"].as_str().unwrap().contains("utils.test")),
        "should find utils.test.mts via lodash (newly added): {selected:?}"
    );
}

// Test for invalid-head-ref behavior moved to cli_tests_plan_lockfile2.rs to
// keep this file under the 500-line test limit.
