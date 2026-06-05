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
    let result = lockfile_diff_json_impl(options);
    assert!(result.is_err(), "binary lockfile should return an error");
    let err = result.unwrap_err();
    assert!(
        err.reason.contains("binary lockfile"),
        "error should mention binary lockfile: {}",
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
    let result = lockfile_diff_json_impl(options).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(entries.len(), 1, "should detect newly added lockfile");
    let added = entries[0]["added"].as_array().unwrap();
    assert!(
        added.iter().any(|v| v == "lodash"),
        "all packages should be reported as added for a new lockfile: {added:?}"
    );
}
