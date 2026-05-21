use super::{find_in_path, is_executable_file};
use std::path::PathBuf;

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
fn find_in_path_accepts_absolute_executable_path() {
    let executable = proxy_fixture("bin/no-mistakes-fixture-proxy");

    assert_eq!(
        find_in_path(executable.to_str().expect("fixture path should be utf8")),
        Some(executable)
    );
}
