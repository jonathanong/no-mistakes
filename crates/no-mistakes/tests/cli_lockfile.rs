#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn setup_git_repo_with_file(root: &Path, filename: &str, content: &str) {
    std::fs::write(root.join(filename), content).unwrap();
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
    Command::new("git")
        .args(["add", filename])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let old_lock = r#"lockfileVersion: '9.0'

packages:
  lodash@4.17.20:
    resolution: {integrity: sha512-PlhdFcillOINfeV7Ni6oF1TAEayyZBoZ8bcshTHqOYJYlrqzRK5hagpagky5o4HfCzzd1TRkXPMFq6cKk9rGg==}
"#;
    let new_lock = r#"lockfileVersion: '9.0'

packages:
  lodash@4.17.21:
    resolution: {integrity: sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZXGO6ysFxfwjE3d56g==}
"#;

    std::fs::write(root.join("pnpm-lock.yaml"), old_lock).unwrap();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    std::fs::write(root.join("pnpm-lock.yaml"), new_lock).unwrap();

    let output = Command::new(bin())
        .args([
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--lockfile",
            "pnpm-lock.yaml",
        ])
        .output()
        .expect("no-mistakes should run");

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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let old_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let new_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";

    std::fs::write(root.join("pnpm-lock.yaml"), old_lock).unwrap();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();
    std::fs::write(root.join("pnpm-lock.yaml"), new_lock).unwrap();

    let output = Command::new(bin())
        .args([
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
        ])
        .output()
        .expect("no-mistakes should run");

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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let content = "# yarn lockfile v1\n\nlodash@^4.0.0:\n  version \"4.17.21\"\n  resolved \"https://r.yarn/lodash.tgz\"\n  integrity sha512-x\n";
    setup_git_repo_with_file(root, "yarn.lock", content);
    let output = Command::new(bin())
        .args([
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
        ])
        .output()
        .expect("no-mistakes should run");
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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let v1 = "lockfileVersion: '9.0'\n\npackages:\n  react@18.2.0:\n    resolution: {integrity: sha512-v1}\n";
    let v2 = "lockfileVersion: '9.0'\n\npackages:\n  react@18.3.0:\n    resolution: {integrity: sha512-v2}\n";
    setup_git_repo_with_file(root, "pnpm-lock.yaml", v1);
    std::fs::write(root.join("pnpm-lock.yaml"), v2).unwrap();
    Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "v2"])
        .current_dir(root)
        .output()
        .unwrap();
    let output = Command::new(bin())
        .args([
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
        ])
        .output()
        .expect("no-mistakes should run");
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
    let output = Command::new(bin())
        .args([
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--lockfile",
            "custom-lock.txt",
        ])
        .output()
        .expect("no-mistakes should run");
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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let content = r#"{"lockfileVersion":2,"packages":{"node_modules/axios":{"version":"1.6.0","resolved":"https://r.npm/axios","integrity":"sha512-x"}}}"#;
    setup_git_repo_with_file(root, "package-lock.json", content);
    let output = Command::new(bin())
        .args([
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--lockfile",
            "package-lock.json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr[0]["manager"], "npm");
}

// Covers bun manager_name.
#[test]
fn lockfile_diff_bun_manager() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let content =
        r#"{"lockfileVersion":0,"packages":{"axios":["axios@1.6.0",{},{"integrity":"sha512-x"}]}}"#;
    setup_git_repo_with_file(root, "bun.lock", content);
    let output = Command::new(bin())
        .args([
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--lockfile",
            "bun.lock",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr[0]["manager"], "bun");
}
