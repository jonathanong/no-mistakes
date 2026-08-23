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
