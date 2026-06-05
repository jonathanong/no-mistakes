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
