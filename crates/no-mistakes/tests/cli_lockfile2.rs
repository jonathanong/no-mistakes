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

fn empty_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git_init(dir.path());
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "empty"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    dir
}

fn copy_lock(name: &str, dest: &Path) {
    std::fs::copy(fixture(name).join("pnpm-lock.yaml"), dest).unwrap();
}

fn copy_after(name: &str, dest: &Path) {
    std::fs::copy(scenario_file(name, "after-pnpm-lock.yaml"), dest).unwrap();
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

// Covers --head failure path in run_diff: when git show <head>:<file> fails,
// emits a warning to stderr and skips the entry (empty JSON array output).
#[test]
fn lockfile_diff_head_failure_warns_and_skips() {
    let dir = committed_repo("lockfile-diff-pnpm-x");
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--head",
        "nonexistent-ref-xyz",
        "--lockfile",
        "pnpm-lock.yaml",
    ]);
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 0, "entry skipped when head ref is missing");
}

// Covers find_git_root in run_diff: when --root is a monorepo subdirectory,
// git show uses a repo-root-relative path (e.g. packages/api/pnpm-lock.yaml).
#[test]
fn lockfile_diff_subdirectory_root_uses_git_relative_path() {
    let dir = committed_repo("lockfile-diff-pnpm-subdir");
    let root = dir.path();
    let sub = root.join("packages").join("api");
    copy_after("lockfile-diff-pnpm-subdir", &sub.join("pnpm-lock.yaml"));
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        sub.to_str().unwrap(),
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
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 1, "should find diff in subdirectory: {arr:?}");
    let changed = arr[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "should detect lodash changed: {changed:?}"
    );
}

// Covers the "newly added lockfile" path in run_diff: when --head introduces a lockfile
// that did not exist at base, treat base as empty so all packages are reported as added.
#[test]
fn lockfile_diff_newly_added_lockfile_reports_added_packages() {
    let dir = empty_repo();
    let root = dir.path();
    copy_lock("lockfile-diff-pnpm-x", &root.join("pnpm-lock.yaml"));
    git_commit(root, &["pnpm-lock.yaml"], "add-lock");

    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 1, "should detect newly added lockfile: {arr:?}");
    let added = arr[0]["added"].as_array().unwrap();
    assert!(
        added.iter().any(|v| v == "lodash"),
        "lodash should be reported as added: {added:?}"
    );
}

// Covers the base-ref validation in run_diff: when --base is an invalid ref and --head is
// provided, the command warns and skips rather than treating the lockfile as newly added.
#[test]
fn lockfile_diff_invalid_base_with_head_warns_and_skips() {
    let dir = committed_repo("lockfile-diff-pnpm-x");
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "nonexistent-base-xyz",
        "--head",
        "HEAD",
        "--lockfile",
        "pnpm-lock.yaml",
    ]);
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 0, "invalid base ref should be skipped: {arr:?}");
}

// Covers the "deleted lockfile at head" path: when --head is a valid ref but the
// lockfile was removed in that commit, treat new content as empty so all packages
// previously in the lockfile are reported as removed.
#[test]
fn lockfile_diff_deleted_lockfile_at_head_reports_removed_packages() {
    let dir = committed_repo("lockfile-diff-pnpm-x");
    let root = dir.path();
    Command::new("git")
        .args(["rm", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "remove-lock"])
        .current_dir(root)
        .output()
        .unwrap();
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
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 1, "should detect deleted lockfile: {arr:?}");
    let removed = arr[0]["removed"].as_array().unwrap();
    assert!(
        removed.iter().any(|v| v == "lodash"),
        "lodash should be reported as removed: {removed:?}"
    );
}

// Covers detect_lockfiles_from_head with a non-empty prefix: when --root is a git
// subdirectory and --head is provided, the prefix is prepended to each candidate
// name for the git show probe (e.g. packages/api/pnpm-lock.yaml).
#[test]
fn lockfile_diff_subdirectory_root_with_head_uses_prefixed_path() {
    let dir = committed_repo("lockfile-diff-pnpm-subdir");
    let root = dir.path();
    let sub = root.join("packages").join("api");
    copy_after("lockfile-diff-pnpm-subdir", &sub.join("pnpm-lock.yaml"));
    git_commit(root, &["-A"], "v2");
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        sub.to_str().unwrap(),
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 1, "should find diff in subdirectory: {arr:?}");
    let changed = arr[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "should detect lodash changed: {changed:?}"
    );
}

// Covers detect_lockfiles_from_head: when --head is provided without --lockfile,
// auto-detection reads candidate names from the head commit, not from disk.
// This matters when the checkout is still at base and the head adds a new lockfile.
#[test]
fn lockfile_diff_head_autodetect_from_commit() {
    let dir = committed_repo("lockfile-diff-pnpm-short");
    let root = dir.path();
    copy_after("lockfile-diff-pnpm-short", &root.join("pnpm-lock.yaml"));
    git_commit(root, &["pnpm-lock.yaml"], "v2");
    // Revert disk to old content so disk-based autodetect would find nothing new.
    // But --head HEAD means we detect from HEAD commit → finds pnpm-lock.yaml there.
    copy_lock("lockfile-diff-pnpm-short", &root.join("pnpm-lock.yaml"));

    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 1, "should find pnpm-lock.yaml via head: {arr:?}");
    let changed = arr[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "should detect lodash changed: {changed:?}"
    );
}

// Covers the no-head newly-added-lockfile path: when --base is a valid ref but the
// lockfile did not exist at that ref, treat the old content as empty so all packages
// in the current working-tree lockfile are reported as added.
#[test]
fn lockfile_diff_no_head_newly_added_lockfile_reports_added() {
    let dir = empty_repo();
    let root = dir.path();
    copy_lock("lockfile-diff-pnpm-x", &root.join("pnpm-lock.yaml"));
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
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        arr.len(),
        1,
        "should detect newly added lockfile in working tree: {arr:?}"
    );
    let added = arr[0]["added"].as_array().unwrap();
    assert!(
        added.iter().any(|v| v == "lodash"),
        "lodash should be reported as added: {added:?}"
    );
}

// Covers the explicit binary lockfile path in run_diff: when --lockfile explicitly
// names a binary lockfile (bun.lockb), detect_manager returns None and the command
// should emit a warning to stderr rather than silently producing an empty result.
#[test]
fn lockfile_diff_explicit_binary_lockfile_warns() {
    let dir = committed_repo("lockfile-diff-bun-binary");
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--lockfile",
        "bun.lockb",
    ]);
    assert!(output.status.success(), "should exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("binary lockfile"),
        "should warn about binary lockfile: {stderr}"
    );
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 0, "binary lockfile produces empty diff: {arr:?}");
}

// Covers lockfile.rs lines 70-74: when --head is invalid and no explicit --lockfile
// is given, the outer git_ref_exists check fires before detect_lockfiles_from_head,
// emits a warning, and returns success with empty output.
#[test]
fn lockfile_diff_invalid_head_without_explicit_lockfile_warns_and_succeeds() {
    let dir = committed_repo("lockfile-diff-pnpm-x");
    let root = dir.path();
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "HEAD",
        "--head",
        "nonexistent-ref-xyz",
    ]);
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist"),
        "should warn about invalid head ref: {stderr}"
    );
    // Early return before JSON output — stdout is empty, not `[]`
    assert!(
        stdout(&output).trim().is_empty(),
        "should produce no output when returning early"
    );
}
