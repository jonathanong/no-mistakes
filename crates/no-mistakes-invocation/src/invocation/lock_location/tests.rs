use super::{open_first_location, LockLocation};
use std::path::Path;

fn fixture_file() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/invocation/read-only-runtime/repo/message.ts"
    ))
}

#[test]
fn exhausted_locations_do_not_select_a_lock() {
    let blocked = fixture_file();
    assert!(open_first_location([
        LockLocation::user(blocked),
        LockLocation::user(blocked),
        LockLocation::user("relative-lock-dir"),
    ])
    .is_none());
}

#[cfg(unix)]
#[test]
fn temporary_locations_reuse_private_directories_and_reject_shared_entries() {
    use std::os::unix::fs::PermissionsExt;

    let root = crate::test_support::materialize_saved_fixture(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/invocation/private-lock-temp"
    )));
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let private = root.path().join("private");
    let shared = root.path().join("shared");
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(open_first_location([LockLocation::private_temp(private.clone())]).is_some());
    assert!(open_first_location([LockLocation::private_temp(shared)]).is_none());

    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(open_first_location([LockLocation::private_temp(private.clone())]).is_none());

    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o770)).unwrap();
    assert!(open_first_location([LockLocation::private_temp(private)]).is_none());
}
