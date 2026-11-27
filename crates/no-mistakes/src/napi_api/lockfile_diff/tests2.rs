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

fn saved(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lockfile/napi-lockfile-diff")
        .join(name)
}

#[test]
fn binary_lockfile_warning_is_additive_and_matches_the_saved_cli_fixture() {
    use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
    use serde_json::{json, Value};

    let saved = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/lockfile-diff-bun-binary");
    let fixture = materialize_saved_fixture(&saved.join("fixture"));
    let root = fixture.path();
    git_init(root);
    git_commit_all(root, "Add binary lockfile");
    let mut options = json!({"root": root, "base": "HEAD", "lockfile": "bun.lockb"});
    let error = lockfile_diff_json_impl(options.clone()).unwrap_err();
    assert!(error.reason.contains("binary lockfile"));

    options["includeWarnings"] = true.into();
    let output = lockfile_diff_json_impl(options).unwrap();
    let report: Value = serde_json::from_str(&output).unwrap();
    let expected: Value =
        serde_json::from_str(&std::fs::read_to_string(saved.join("expected-node.json")).unwrap())
            .unwrap();
    assert_eq!(report, expected);
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
    setup_git_repo(root);
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "empty"])
        .current_dir(root)
        .output()
        .unwrap();
    // Copy the lockfile onto disk but do NOT commit it — so HEAD doesn't have it.
    std::fs::copy(saved("lodash-4.17.21-x.yaml"), root.join("pnpm-lock.yaml")).unwrap();
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let mut input: serde_json::Value = serde_json::from_str(&options).unwrap();
    let result = lockfile_diff_json_impl(input.clone()).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    input["includeWarnings"] = true.into();
    let report: serde_json::Value =
        serde_json::from_str(&lockfile_diff_json_impl(input).unwrap()).unwrap();
    assert_eq!(
        report,
        serde_json::json!({"entries": entries, "warnings": []})
    );
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
    // `app/` already exists in the saved tree, so the test does not create it.
    let dir = crate::test_support::materialize_saved_fixture(&saved("app-lock"));
    let root = dir.path();
    setup_git_repo(root);
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
