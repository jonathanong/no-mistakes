use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn setup_git_repo_with_file(root: &std::path::Path, filename: &str, content: &str) {
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

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

// Covers --head failure path in run_diff: when git show <head>:<file> fails,
// emits a warning to stderr and skips the entry (empty JSON array output).
#[test]
fn lockfile_diff_head_failure_warns_and_skips() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let content = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-x}\n";
    setup_git_repo_with_file(root, "pnpm-lock.yaml", content);
    let output = Command::new(bin())
        .args([
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
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 0, "entry skipped when head ref is missing");
}

// Covers find_git_root in run_diff: when --root is a monorepo subdirectory,
// git show uses a repo-root-relative path (e.g. packages/api/pnpm-lock.yaml).
#[test]
fn lockfile_diff_subdirectory_root_uses_git_relative_path() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let sub = root.join("packages").join("api");
    std::fs::create_dir_all(&sub).unwrap();
    let old_content = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    std::fs::write(sub.join("pnpm-lock.yaml"), old_content).unwrap();
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
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();
    let new_content = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";
    std::fs::write(sub.join("pnpm-lock.yaml"), new_content).unwrap();
    let output = Command::new(bin())
        .args([
            "lockfile",
            "diff",
            "--root",
            sub.to_str().unwrap(),
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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let lock_v1 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-x}\n";

    // Initial commit with NO lockfile
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
        .args(["commit", "--allow-empty", "-m", "empty"])
        .current_dir(root)
        .output()
        .unwrap();

    // Second commit adds the lockfile
    std::fs::write(root.join("pnpm-lock.yaml"), lock_v1).unwrap();
    Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add-lock"])
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
        ])
        .output()
        .expect("no-mistakes should run");

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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let lock_content = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-x}\n";
    setup_git_repo_with_file(root, "pnpm-lock.yaml", lock_content);
    let output = Command::new(bin())
        .args([
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
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 0, "invalid base ref should be skipped: {arr:?}");
}

// Covers the "deleted lockfile at head" path: when --head is a valid ref but the
// lockfile was removed in that commit, treat new content as empty so all packages
// previously in the lockfile are reported as removed.
#[test]
fn lockfile_diff_deleted_lockfile_at_head_reports_removed_packages() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let lock_v1 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-x}\n";
    setup_git_repo_with_file(root, "pnpm-lock.yaml", lock_v1);
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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let sub = root.join("packages").join("api");
    std::fs::create_dir_all(&sub).unwrap();
    let old = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let new = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";
    std::fs::write(sub.join("pnpm-lock.yaml"), old).unwrap();
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
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();
    std::fs::write(sub.join("pnpm-lock.yaml"), new).unwrap();
    Command::new("git")
        .args(["add", "-A"])
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
            sub.to_str().unwrap(),
            "--base",
            "HEAD~1",
            "--head",
            "HEAD",
        ])
        .output()
        .expect("no-mistakes should run");
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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let lock_v1 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let lock_v2 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";

    setup_git_repo_with_file(root, "pnpm-lock.yaml", lock_v1);
    std::fs::write(root.join("pnpm-lock.yaml"), lock_v2).unwrap();
    std::process::Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "v2"])
        .current_dir(root)
        .output()
        .unwrap();

    // Revert disk to old content so disk-based autodetect would find nothing new.
    // But --head HEAD means we detect from HEAD commit → finds pnpm-lock.yaml there.
    std::fs::write(root.join("pnpm-lock.yaml"), lock_v1).unwrap();

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
            // No --lockfile: auto-detect from head
        ])
        .output()
        .expect("no-mistakes should run");
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

// Covers the explicit binary lockfile path in run_diff: when --lockfile explicitly
// names a binary lockfile (bun.lockb), detect_manager returns None and the command
// should emit a warning to stderr rather than silently producing an empty result.
#[test]
fn lockfile_diff_explicit_binary_lockfile_warns() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // Write a minimal bun.lockb stand-in so the path exists
    std::fs::write(root.join("bun.lockb"), b"binarydata").unwrap();
    setup_git_repo_with_file(root, "bun.lockb", "binarydata");
    let output = Command::new(bin())
        .args([
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--lockfile",
            "bun.lockb",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success(), "should exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("binary lockfile"),
        "should warn about binary lockfile: {stderr}"
    );
    let arr: Vec<serde_json::Value> = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(arr.len(), 0, "binary lockfile produces empty diff: {arr:?}");
}
