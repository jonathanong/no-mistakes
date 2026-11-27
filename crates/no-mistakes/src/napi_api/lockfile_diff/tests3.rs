use super::*;

#[test]
fn git_io_to_napi_preserves_the_io_error() {
    let error = git_io_to_napi(std::io::Error::other("git unavailable"));
    assert!(error.reason.contains("git unavailable"), "{}", error.reason);
}

#[test]
fn lockfile_diff_json_impl_reports_git_io_failures() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    drop(dir);
    let options = format!(
        r#"{{"root": "{}", "base": "HEAD"}}"#,
        root.to_str().unwrap().replace('\\', "/")
    );
    let error = lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(options))
        .expect_err("a deleted root cannot run git");
    assert!(
        !error.reason.is_empty(),
        "git IO mapping should surface a reason"
    );
}

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
fn lockfile_diff_json_impl_rejects_malformed_cargo_lock() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup_git_repo(root);
    let lock = root.join("Cargo.lock");
    let malformed = ecosystem_lock("cargo/malformed.lock");
    let valid = ecosystem_lock("cargo/base.lock");
    std::fs::copy(&malformed, &lock).unwrap();
    git_commit(root, "malformed");
    std::fs::copy(&valid, &lock).unwrap();
    let base_error = diff_lock(root).unwrap_err();
    assert!(
        base_error.reason.contains("malformed Cargo.lock content"),
        "{}",
        base_error.reason
    );

    git_commit(root, "valid");
    std::fs::copy(&malformed, &lock).unwrap();
    let head_error = diff_lock(root).unwrap_err();
    assert!(
        head_error.reason.contains("malformed Cargo.lock content"),
        "{}",
        head_error.reason
    );
}

fn ecosystem_lock(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/lockfile")
        .join(name)
}

fn git_commit(root: &std::path::Path, message: &str) {
    std::process::Command::new("git")
        .args(["add", "Cargo.lock"])
        .current_dir(root)
        .output()
        .unwrap();
    let output = std::process::Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn diff_lock(root: &std::path::Path) -> napi::Result<String> {
    lockfile_diff_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({
            "root": root,
            "base": "HEAD",
            "lockfile": "Cargo.lock"
        })
        .to_string(),
    ))
}
