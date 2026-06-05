use super::*;
use crate::codebase::lockfile::PackageManager;

// ---------------------------------------------------------------------------
// manager_name
// ---------------------------------------------------------------------------

#[test]
fn manager_name_npm() {
    assert_eq!(manager_name(PackageManager::Npm), "npm");
}

#[test]
fn manager_name_pnpm() {
    assert_eq!(manager_name(PackageManager::Pnpm), "pnpm");
}

#[test]
fn manager_name_yarn() {
    assert_eq!(manager_name(PackageManager::Yarn), "yarn");
}

#[test]
fn manager_name_bun() {
    assert_eq!(manager_name(PackageManager::Bun), "bun");
}

// ---------------------------------------------------------------------------
// git_show_file
// ---------------------------------------------------------------------------

#[test]
fn git_show_file_invalid_ref_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Initialise a bare git repo so git commands work
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();

    // Non-existent ref → None
    let result = git_show_file(root, "nonexistent-ref-xyz", "any-file.yaml");
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// lockfile_diff_json_impl — error paths
// ---------------------------------------------------------------------------

#[test]
fn lockfile_diff_json_impl_invalid_json_returns_err() {
    let result = lockfile_diff_json_impl("not valid json {{{".to_string());
    assert!(result.is_err());
}

#[test]
fn lockfile_diff_json_impl_root_with_no_lockfiles_returns_empty_array() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Provide a valid root that has no lockfiles at all
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// lockfile_diff_json_impl — happy path with a real git repo
// ---------------------------------------------------------------------------

fn setup_git_repo(root: &std::path::Path) {
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();
}

#[test]
fn lockfile_diff_json_impl_pnpm_changed_package() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let old_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let new_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";

    std::fs::write(root.join("pnpm-lock.yaml"), old_lock).unwrap();

    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Update lockfile on disk (new content)
    std::fs::write(root.join("pnpm-lock.yaml"), new_lock).unwrap();

    let options = format!(
        r#"{{"root": "{}", "base": "HEAD"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["manager"], "pnpm");
    let changed = entries[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "expected lodash in changed: {changed:?}"
    );
}

#[test]
fn lockfile_diff_json_impl_invalid_base_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("pnpm-lock.yaml"),
        "lockfileVersion: '9.0'\n\npackages:\n",
    )
    .unwrap();

    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    let options = format!(
        r#"{{"root": "{}", "base": "nonexistent-ref-xyz"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options);
    // git_show_file returns None → should return Err
    assert!(result.is_err());
}

#[test]
fn lockfile_diff_json_impl_head_option_reads_from_git() {
    // When `head` is specified, new_content comes from git_show_file(head) instead of disk.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let v1_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-v1}\n";
    let v2_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-v2}\n";

    std::fs::write(root.join("pnpm-lock.yaml"), v1_lock).unwrap();

    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "v1"])
        .current_dir(root)
        .output()
        .unwrap();

    std::fs::write(root.join("pnpm-lock.yaml"), v2_lock).unwrap();
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

    // base = HEAD~1 (v1), head = HEAD (v2)
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD~1", "head": "HEAD"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(entries.len(), 1);
    let changed = entries[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "expected lodash changed between v1 and v2: {changed:?}"
    );
}

#[test]
fn lockfile_diff_json_impl_missing_base_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let options = format!(
        r#"{{"root": "{}"}}"#,
        dir.path().to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options);
    assert!(result.is_err(), "missing base should be an error");
    let err = result.unwrap_err();
    assert!(
        err.reason.contains("base"),
        "error should mention base: {}",
        err.reason
    );
}

#[test]
fn lockfile_diff_json_impl_empty_base_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": ""}}"#,
        dir.path().to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options);
    assert!(result.is_err(), "empty base should be an error");
}

#[test]
fn lockfile_diff_json_impl_invalid_head_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    std::fs::write(root.join("pnpm-lock.yaml"), lock).unwrap();
    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    let options = format!(
        r#"{{"root": "{}", "base": "HEAD", "head": "nonexistent-ref-xyz"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options);
    assert!(result.is_err(), "invalid head ref should be an error");
}
