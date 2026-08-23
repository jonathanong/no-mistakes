use super::*;

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
fn lockfile_diff_json_impl_binary_lockfile_explicit_returns_err() {
    // Covers lines 62-65: when the explicit lockfile path is a binary format
    // (bun.lockb), detect_manager returns None and is_binary_lockfile returns true
    // → error instead of a silent skip.
    let dir = tempfile::tempdir().unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD", "lockfile": "bun.lockb"}}"#,
        dir.path().to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(options));
    assert!(result.is_err(), "binary lockfile should return an error");
    let err = result.unwrap_err();
    assert!(
        err.reason.contains("binary lockfile"),
        "error should mention binary lockfile: {}",
        err.reason
    );
}

#[test]
fn lockfile_diff_json_impl_invalid_head_without_explicit_lockfile_returns_err() {
    // Covers the git_ref_exists guard added before detect_lockfiles_from_head:
    // when `head` is present but not a valid ref and no explicit lockfile is
    // supplied, the function must return an error rather than silently returning [].
    let dir = tempfile::tempdir().unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD", "head": "nonexistent-ref-xyz"}}"#,
        dir.path().to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(options));
    assert!(result.is_err(), "invalid head ref should return an error");
    let err = result.unwrap_err();
    assert!(
        err.reason.contains("does not exist"),
        "error should mention 'does not exist': {}",
        err.reason
    );
}

#[test]
fn lockfile_diff_json_impl_newly_added_no_head_reports_all_added() {
    // Covers lines 114-116 (no-head branch): valid base ref but file absent at
    // base → old_content treated as empty → all packages reported as added.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-x}\n";
    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "empty"])
        .current_dir(root)
        .output()
        .unwrap();
    // Write lockfile to disk but do NOT commit it — so HEAD doesn't have it
    std::fs::write(root.join("pnpm-lock.yaml"), lock).unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let result = lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(entries.len(), 1, "should detect newly added lockfile");
    let added = entries[0]["added"].as_array().unwrap();
    assert!(
        added.iter().any(|v| v == "lodash"),
        "all packages should be reported as added for a new lockfile: {added:?}"
    );
}

#[test]
fn lockfile_diff_napi_ignores_worktree_lockfile_but_honors_explicit_path() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass3-visibility");
    let root = fixture.path();
    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    let automatic = lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root, "base": "HEAD" }).to_string(),
    ))
    .unwrap();
    let automatic: Vec<serde_json::Value> = serde_json::from_str(&automatic).unwrap();
    assert!(automatic.is_empty());

    let explicit = lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({
            "root": root,
            "base": "HEAD",
            "lockfile": "pnpm-lock.yaml"
        })
        .to_string(),
    ))
    .unwrap();
    let explicit: Vec<serde_json::Value> = serde_json::from_str(&explicit).unwrap();
    assert_eq!(explicit.len(), 1);
    assert!(explicit[0]["added"]
        .as_array()
        .unwrap()
        .iter()
        .any(|package| package == "lodash"));
}

#[test]
fn find_git_root_returns_none_outside_a_repository() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(find_git_root(dir.path()).unwrap(), None);
}

#[test]
fn git_ref_exists_is_false_for_missing_refs() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());
    assert!(!git_ref_exists(dir.path(), "missing-ref").unwrap());
    assert!(git_ref_exists(dir.path(), "HEAD").is_ok());
}

#[test]
fn detect_lockfiles_from_head_uses_subdirectory_paths() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup_git_repo(root);
    std::fs::create_dir_all(root.join("app")).unwrap();
    std::fs::write(root.join("app/pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "app/pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "lock"])
        .current_dir(root)
        .output()
        .unwrap();
    let found = detect_lockfiles_from_head(root, "HEAD", &root.join("app")).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].ends_with("pnpm-lock.yaml"));
}
