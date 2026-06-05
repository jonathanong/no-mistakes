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
fn lockfile_diff_json_impl_subdirectory_root_uses_git_relative_path() {
    // When root is a subdirectory of the git repo, git show must use a repo-relative path.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let sub = root.join("packages").join("api");
    std::fs::create_dir_all(&sub).unwrap();
    let old_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let new_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";
    std::fs::write(sub.join("pnpm-lock.yaml"), old_lock).unwrap();
    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();
    std::fs::write(sub.join("pnpm-lock.yaml"), new_lock).unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD"}}"#,
        sub.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(
        entries.len(),
        1,
        "should find diff in subdirectory: {entries:?}"
    );
    let changed = entries[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "should detect lodash changed: {changed:?}"
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

    // When --lockfile is explicit and head is invalid, git_show_file fails → error.
    // Without --lockfile, detect_lockfiles_from_head silently returns empty (matches CLI behavior).
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD", "head": "nonexistent-ref-xyz", "lockfile": "pnpm-lock.yaml"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options);
    assert!(
        result.is_err(),
        "invalid head ref should be an error when lockfile is explicit"
    );
}

#[test]
fn lockfile_diff_json_impl_head_autodetect_new_lockfile() {
    // When head adds a new lockfile not present on disk, detect_lockfiles_from_head finds it
    // and base is treated as empty → all packages reported as added.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let lock_v1 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let lock_v2 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";
    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "empty"])
        .current_dir(root)
        .output()
        .unwrap();
    std::fs::write(root.join("pnpm-lock.yaml"), lock_v1).unwrap();
    std::process::Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "add lockfile"])
        .current_dir(root)
        .output()
        .unwrap();
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
    // base = HEAD~2 (no lockfile), head = HEAD (v2) → lodash should appear as added
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD~2", "head": "HEAD"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(
        entries.len(),
        1,
        "should detect new lockfile via head: {entries:?}"
    );
    let added = entries[0]["added"].as_array().unwrap();
    assert!(
        added.iter().any(|v| v == "lodash"),
        "lodash should be added when lockfile is new: {added:?}"
    );
}

#[test]
fn lockfile_diff_json_impl_invalid_head_no_lockfile_returns_err() {
    // Without explicit lockfile, an invalid head ref is rejected before autodetection
    // rather than silently returning an empty result.
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
    assert!(result.is_err(), "invalid head ref should return an error");
    assert!(
        result.unwrap_err().reason.contains("does not exist"),
        "error should mention ref does not exist"
    );
}

#[test]
fn lockfile_diff_json_impl_invalid_base_with_head_returns_err() {
    // When head is valid but base ref is invalid, git_show_file fails for old_content.
    // git_ref_exists returns false for the invalid base → error is returned.
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
        r#"{{"root": "{}", "base": "nonexistent-base-xyz", "head": "HEAD", "lockfile": "pnpm-lock.yaml"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options);
    assert!(
        result.is_err(),
        "invalid base with head should return an error"
    );
}

#[test]
fn lockfile_diff_json_impl_unknown_lockfile_name_skipped() {
    // When an explicit lockfile path has a name not recognized by detect_manager,
    // the entry is skipped → empty result (no error).
    let dir = tempfile::tempdir().unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD", "lockfile": "custom-lock.txt"}}"#,
        dir.path().to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert!(
        entries.is_empty(),
        "unknown lockfile name should be skipped: {entries:?}"
    );
}

#[test]
fn lockfile_diff_json_impl_head_with_subdirectory_root() {
    // When root is a git subdirectory and head is set, detect_lockfiles_from_head
    // builds a repo-relative path (e.g. packages/api/pnpm-lock.yaml) for git show.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let sub = root.join("packages").join("api");
    std::fs::create_dir_all(&sub).unwrap();
    let v1 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let v2 = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";
    std::fs::write(sub.join("pnpm-lock.yaml"), v1).unwrap();
    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "v1"])
        .current_dir(root)
        .output()
        .unwrap();
    std::fs::write(sub.join("pnpm-lock.yaml"), v2).unwrap();
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "v2"])
        .current_dir(root)
        .output()
        .unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD~1", "head": "HEAD"}}"#,
        sub.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(
        entries.len(),
        1,
        "should find diff in subdirectory with head: {entries:?}"
    );
    let changed = entries[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "should detect lodash changed: {changed:?}"
    );
}

#[test]
fn lockfile_diff_json_impl_deleted_lockfile_at_head_reports_removed() {
    // When head is valid but the lockfile was deleted, new content is empty
    // and all previously locked packages should be reported as removed.
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
    std::process::Command::new("git")
        .args(["rm", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "delete-lock"])
        .current_dir(root)
        .output()
        .unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD~1", "head": "HEAD", "lockfile": "pnpm-lock.yaml"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(
        entries.len(),
        1,
        "should detect deleted lockfile: {entries:?}"
    );
    let removed = entries[0]["removed"].as_array().unwrap();
    assert!(
        removed.iter().any(|v| v == "lodash"),
        "lodash should be reported as removed: {removed:?}"
    );
}
