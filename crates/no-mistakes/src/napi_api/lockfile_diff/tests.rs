use super::*;
use crate::codebase::lockfile::PackageManager;
use crate::test_support::{git_commit_all, git_init};
use std::path::{Path, PathBuf};

fn saved(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lockfile/napi-lockfile-diff")
        .join(name)
}

fn copy_lock(dir: &Path, name: &str) {
    std::fs::copy(saved(name), dir.join("pnpm-lock.yaml")).unwrap();
}

fn materialize(name: &str) -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(&saved(name))
}

fn committed_lock(name: &str, message: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    copy_lock(dir.path(), name);
    git_init(dir.path());
    git_commit_all(dir.path(), message);
    dir
}

fn invoke(root: &Path, body: &str) -> napi::Result<String> {
    let root = root.to_str().unwrap().replace('\\', "/");
    let json = if body.is_empty() {
        format!(r#"{{"root": "{root}"}}"#)
    } else {
        format!(r#"{{"root": "{root}", {body}}}"#)
    };
    lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(json))
}

fn empty_commit(root: &Path) {
    let output = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=no-mistakes tests",
            "-c",
            "user.email=no-mistakes-tests@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "empty",
        ])
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

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

#[test]
fn git_show_file_invalid_ref_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    git_init(dir.path());
    let result = git_show_file(dir.path(), "nonexistent-ref-xyz", "any-file.yaml").unwrap();
    assert!(result.is_none());
}

#[test]
fn lockfile_diff_json_impl_invalid_json_returns_err() {
    let error =
        crate::napi_api::options::parse_options::<super::LockfileDiffOptions>("not valid json {{{")
            .unwrap_err();
    assert!(error.reason.contains("invalid options JSON"));
}

#[test]
fn lockfile_diff_json_impl_root_with_no_lockfiles_returns_empty_array() {
    let dir = tempfile::tempdir().unwrap();
    let result = invoke(dir.path(), r#""base": "HEAD""#).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 0);
}

#[test]
fn lockfile_diff_json_impl_pnpm_changed_package() {
    let dir = committed_lock("lodash-4.17.20-old.yaml", "initial");
    copy_lock(dir.path(), "lodash-4.17.21-new.yaml");
    let result = invoke(dir.path(), r#""base": "HEAD""#).unwrap();
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
    let dir = committed_lock("packages-empty.yaml", "initial");
    // git_show_file returns None for the missing ref.
    assert!(invoke(dir.path(), r#""base": "nonexistent-ref-xyz""#).is_err());
}

#[test]
fn lockfile_diff_json_impl_head_option_reads_from_git() {
    // `head` reads new content from git instead of the worktree.
    let dir = committed_lock("lodash-4.17.20-v1.yaml", "v1");
    copy_lock(dir.path(), "lodash-4.17.21-v2.yaml");
    git_commit_all(dir.path(), "v2");
    let result = invoke(dir.path(), r#""base": "HEAD~1", "head": "HEAD""#).unwrap();
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
    // Root is a git subdirectory, so git show must use a repo-relative path.
    let dir = materialize("subdirectory-root");
    let root = dir.path();
    let sub = root.join("packages/api");
    git_init(root);
    git_commit_all(root, "initial");
    copy_lock(&sub, "lodash-4.17.21-new.yaml");
    let result = invoke(&sub, r#""base": "HEAD""#).unwrap();
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
    let result = invoke(dir.path(), "");
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
    assert!(
        invoke(dir.path(), "\"base\": \"\"").is_err(),
        "empty base should be an error"
    );
}

#[test]
fn lockfile_diff_json_impl_invalid_head_returns_err() {
    let dir = committed_lock("lodash-4.17.20-old.yaml", "initial");
    // Explicit lockfile plus an invalid head fails in git_show_file.
    let result = invoke(
        dir.path(),
        r#""base": "HEAD", "head": "nonexistent-ref-xyz", "lockfile": "pnpm-lock.yaml""#,
    );
    assert!(
        result.is_err(),
        "invalid head ref should be an error when lockfile is explicit"
    );
}

#[test]
fn lockfile_diff_json_impl_head_autodetect_new_lockfile() {
    // Head introduces a lockfile absent at base, so every package is added.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_init(root);
    empty_commit(root);
    copy_lock(root, "lodash-4.17.20-old.yaml");
    git_commit_all(root, "add lockfile");
    copy_lock(root, "lodash-4.17.21-new.yaml");
    git_commit_all(root, "v2");
    let result = invoke(root, r#""base": "HEAD~2", "head": "HEAD""#).unwrap();
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
    let dir = committed_lock("lodash-4.17.20-old.yaml", "initial");
    let result = invoke(
        dir.path(),
        r#""base": "HEAD", "head": "nonexistent-ref-xyz""#,
    );
    assert!(result.is_err(), "invalid head ref should return an error");
    assert!(
        result.unwrap_err().reason.contains("does not exist"),
        "error should mention ref does not exist"
    );
}

#[test]
fn lockfile_diff_json_impl_invalid_base_with_head_returns_err() {
    let dir = committed_lock("lodash-4.17.20-old.yaml", "initial");
    let result = invoke(
        dir.path(),
        r#""base": "nonexistent-base-xyz", "head": "HEAD", "lockfile": "pnpm-lock.yaml""#,
    );
    assert!(
        result.is_err(),
        "invalid base with head should return an error"
    );
}

#[test]
fn lockfile_diff_json_impl_unknown_lockfile_name_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let result = invoke(
        dir.path(),
        r#""base": "HEAD", "lockfile": "custom-lock.txt""#,
    )
    .unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert!(
        entries.is_empty(),
        "unknown lockfile name should be skipped: {entries:?}"
    );
}

#[test]
fn lockfile_diff_json_impl_head_with_subdirectory_root() {
    // Head plus a git subdirectory builds a repo-relative git show path.
    let dir = materialize("subdirectory-root");
    let root = dir.path();
    let sub = root.join("packages/api");
    git_init(root);
    git_commit_all(root, "v1");
    copy_lock(&sub, "lodash-4.17.21-new.yaml");
    git_commit_all(root, "v2");
    let result = invoke(&sub, r#""base": "HEAD~1", "head": "HEAD""#).unwrap();
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
    let dir = committed_lock("lodash-4.17.20-old.yaml", "initial");
    std::fs::remove_file(dir.path().join("pnpm-lock.yaml")).unwrap();
    git_commit_all(dir.path(), "delete-lock");
    let result = invoke(
        dir.path(),
        r#""base": "HEAD~1", "head": "HEAD", "lockfile": "pnpm-lock.yaml""#,
    )
    .unwrap();
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
