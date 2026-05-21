use super::{find_in_path, is_executable_file};
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

struct PermissionsGuard {
    path: PathBuf,
    permissions: Permissions,
}

impl PermissionsGuard {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            permissions: fs::metadata(path)
                .expect("fixture copy should exist")
                .permissions(),
        }
    }
}

impl Drop for PermissionsGuard {
    fn drop(&mut self) {
        fs::set_permissions(&self.path, self.permissions.clone())
            .expect("fixture copy mode should restore");
    }
}

fn proxy_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/no-mistakes-proxy")
        .join(name)
}

#[test]
fn is_executable_file_accepts_regular_file_with_execute_bit() {
    assert!(is_executable_file(&proxy_fixture(
        "bin/no-mistakes-fixture-proxy"
    )));
}

#[test]
fn is_executable_file_rejects_regular_file_without_execute_bit() {
    assert!(!is_executable_file(&proxy_fixture(
        "non-executable-bin/no-mistakes-fixture-proxy"
    )));
}

#[test]
fn is_executable_file_rejects_directory_with_execute_bit() {
    assert!(!is_executable_file(&proxy_fixture(
        "directory-bin/no-mistakes-fixture-proxy"
    )));
}

#[test]
fn is_executable_file_rejects_file_current_user_cannot_execute() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let blocked = temp_dir.path().join("no-mistakes-fixture-proxy");
    fs::copy(
        proxy_fixture("owner-blocked-bin/no-mistakes-fixture-proxy"),
        &blocked,
    )
    .expect("fixture should copy");
    let _guard = PermissionsGuard::new(&blocked);

    let mut blocked_permissions = fs::metadata(&blocked)
        .expect("fixture copy should exist")
        .permissions();
    blocked_permissions.set_mode(0o001);
    fs::set_permissions(&blocked, blocked_permissions).expect("fixture mode should change");

    assert!(!is_executable_file(&blocked));
}

#[test]
fn find_in_path_accepts_absolute_executable_path() {
    let executable = proxy_fixture("bin/no-mistakes-fixture-proxy");

    assert_eq!(
        find_in_path(executable.to_str().expect("fixture path should be utf8")),
        Some(executable)
    );
}
