use super::{command_name, sidecar_path, LockHolder};
use crate::invocation::lock::acquire_lock;
use std::ffi::OsString;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/invocation")
        .join(name)
}

#[test]
fn command_records_omit_option_and_positional_values() {
    let cases: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture("holder-command/argv.json")).unwrap(),
    )
    .unwrap();
    for case in cases.as_array().unwrap() {
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| OsString::from(arg.as_str().unwrap()));
        assert_eq!(command_name(args), case["command"].as_str().unwrap());
    }
    assert_eq!(command_name([OsString::from("x".repeat(600))]).len(), 512);
}

#[cfg(unix)]
#[test]
fn replacing_a_saved_holder_record_keeps_it_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let lock = directory.path().join("invocation.lock");
    let sidecar = sidecar_path(&lock);
    std::fs::copy(fixture("lock-holder/invocation.lock.holder"), &sidecar).unwrap();
    let _held = acquire_lock(&lock, None, false).unwrap();
    assert_eq!(
        std::fs::metadata(sidecar).unwrap().permissions().mode() & 0o077,
        0
    );
    assert_eq!(LockHolder::read(&lock).unwrap().pid, std::process::id());
    // Unique staging files are cleaned up after the atomic replacement.
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[test]
fn holder_record_failures_do_not_publish_partial_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let missing_parent = directory.path().join("missing/invocation.lock");
    assert!(LockHolder::current().record(&missing_parent).is_err());
    assert!(LockHolder::read(&missing_parent).is_none());
}

#[test]
fn failed_atomic_replacement_cleans_up_staging_and_keeps_acquisition_available() {
    let directory =
        crate::test_support::materialize_saved_fixture(&fixture("holder-record-target-directory"));
    let lock = directory.path().join("invocation.lock");
    assert!(LockHolder::current().record(&lock).is_err());
    assert!(sidecar_path(&lock).is_dir());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    // Diagnostic publication failure must never prevent a real lock acquisition.
    let _held = acquire_lock(&lock, None, false).unwrap();
    assert!(LockHolder::read(&lock).is_none());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
}
