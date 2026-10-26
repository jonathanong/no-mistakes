use crate::gitignore_fixture;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
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

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn scenario_file(name: &str, file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join(name)
        .join(file)
}

fn committed_repo(name: &str) -> tempfile::TempDir {
    let dir = crate::saved_fixture::materialize_path(&fixture(name));
    git_init(dir.path());
    git_commit(dir.path(), &["-A"], "initial");
    dir
}

fn copy_after(name: &str, dest: &Path) {
    std::fs::copy(scenario_file(name, "after-pnpm-lock.yaml"), dest).unwrap();
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn lockfile_changed_file_without_base_emits_warning() {
    let root = fixture("lockfile-impact-no-baseline");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "pnpm-lock.yaml",
        "--global-config-fallback=true",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        plan["fallback_triggered"].as_bool().unwrap(),
        "should trigger fallback when no baseline: {plan:?}"
    );
    let reason = plan["fallback_reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("baseline") || reason.contains("lockfile"),
        "fallback_reason should mention baseline/lockfile: {reason}"
    );
}

#[test]
fn lockfile_changed_file_without_base_no_fallback_when_disabled() {
    let root = fixture("lockfile-impact-no-baseline");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "pnpm-lock.yaml",
        "--global-config-fallback=false",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "fallback should not trigger when disabled: {plan:?}"
    );
}

#[test]
fn lockfile_diff_subcommand_invalid_base_exits_ok_with_warning() {
    let root = fixture("lockfile-impact-pnpm");
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "nonexistent-branch-xyzzy",
        "--lockfile",
        "pnpm-lock.yaml",
    ]);
    assert!(
        output.status.success(),
        "should exit 0 even on missing base: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = stdout(&output);
    let arr: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        arr.as_array().unwrap().len(),
        0,
        "no diff when base unreachable"
    );
}

#[test]
fn lockfile_diff_subcommand_with_git_repo() {
    let dir = committed_repo("lockfile-diff-pnpm-long");
    let root = dir.path();
    copy_after("lockfile-diff-pnpm-long", &root.join("pnpm-lock.yaml"));

    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--lockfile",
        "pnpm-lock.yaml",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = stdout(&output);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["manager"], "pnpm");
    let changed = arr[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "should detect lodash as changed: {changed:?}"
    );
}

#[test]
fn lockfile_diff_paths_format() {
    let dir = committed_repo("lockfile-diff-pnpm-short");
    let root = dir.path();
    copy_after("lockfile-diff-pnpm-short", &root.join("pnpm-lock.yaml"));

    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--lockfile",
        "pnpm-lock.yaml",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(
        out.contains("lodash"),
        "paths format should list lodash: {out}"
    );
}

#[test]
fn tests_plan_lockfile_no_global_config_trigger_with_lockfile() {
    let root = fixture("lockfile-impact-pnpm");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "pnpm-lock.yaml",
        "--global-config-fallback=true",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        plan["fallback_triggered"].as_bool().unwrap(),
        "should trigger fallback when no baseline: {plan:?}"
    );
    let reason = plan["fallback_reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("baseline") || reason.contains("lockfile"),
        "fallback_reason should mention baseline: {reason}"
    );
}

// Covers detect_lockfiles_in_root (auto-detection without --lockfile) and yarn manager_name.
#[test]
fn lockfile_diff_auto_detect_yarn_lock() {
    let dir = committed_repo("lockfile-diff-yarn");
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
    ]);
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 1, "should auto-detect yarn.lock");
    assert_eq!(arr[0]["manager"], "yarn");
}

#[test]
fn lockfile_diff_ignores_worktree_lockfile_but_honors_explicit_path() {
    let fixture = gitignore_fixture::materialize("pass3-visibility");
    let root = fixture.path();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    for (key, value) in [("user.email", "test@test.com"), ("user.name", "Test")] {
        Command::new("git")
            .args(["config", key, value])
            .current_dir(root)
            .output()
            .unwrap();
    }
    Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    let automatic = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
    ]);
    assert!(automatic.status.success());
    let automatic: Vec<serde_json::Value> = serde_json::from_str(&stdout(&automatic)).unwrap();
    assert!(automatic.is_empty());

    let explicit = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--lockfile",
        "pnpm-lock.yaml",
    ]);
    assert!(explicit.status.success());
    let explicit: Vec<serde_json::Value> = serde_json::from_str(&stdout(&explicit)).unwrap();
    assert_eq!(explicit.len(), 1);
    assert!(explicit[0]["added"]
        .as_array()
        .unwrap()
        .iter()
        .any(|package| package == "lodash"));
}

// Covers the --head branch where new content is read via git show <head>:.
#[test]
fn lockfile_diff_head_reads_from_git() {
    let dir = committed_repo("lockfile-diff-pnpm-react");
    let root = dir.path();
    copy_after("lockfile-diff-pnpm-react", &root.join("pnpm-lock.yaml"));
    git_commit(root, &["pnpm-lock.yaml"], "v2");
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
        "--lockfile",
        "pnpm-lock.yaml",
    ]);
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    let changed = arr[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "react"),
        "react should be changed: {changed:?}"
    );
}

// Covers the continue branch when detect_manager returns None for an unknown filename.
#[test]
fn lockfile_diff_unrecognized_lockfile_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--lockfile",
        "custom-lock.txt",
    ]);
    assert!(output.status.success());
    let arr: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        arr.as_array().unwrap().len(),
        0,
        "unrecognized lockfile skipped"
    );
}

// Covers npm manager_name.
#[test]
fn lockfile_diff_npm_manager() {
    let dir = committed_repo("lockfile-diff-npm");
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--lockfile",
        "package-lock.json",
    ]);
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr[0]["manager"], "npm");
}

// Covers bun manager_name.
#[test]
fn lockfile_diff_bun_manager() {
    let dir = committed_repo("lockfile-diff-bun");
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--lockfile",
        "bun.lock",
    ]);
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr[0]["manager"], "bun");
}
