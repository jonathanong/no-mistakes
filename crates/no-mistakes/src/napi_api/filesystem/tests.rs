use super::{
    acquire_planning_artifact_lock_impl, flock_impl, rename_no_replace_impl,
    unlock_planning_artifact_lock_impl, validate_planning_artifact_lock_identity,
};

#[test]
fn does_not_replace_an_existing_destination() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source");
    let destination = directory.path().join("destination");
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&destination).unwrap();

    assert!(!rename_no_replace_impl(&source, &destination).unwrap());
    assert!(source.is_dir());
    assert!(destination.is_dir());

    std::fs::remove_dir(&destination).unwrap();
    assert!(rename_no_replace_impl(&source, &destination).unwrap());
    assert!(!source.exists());
    assert!(destination.is_dir());
}

#[test]
fn reports_an_underlying_rename_error() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("missing-source");
    let destination = directory.path().join("destination");

    let error = rename_no_replace_impl(&source, &destination).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(!destination.exists());
}

#[cfg(unix)]
#[test]
fn serializes_lock_holders_and_releases_on_drop() {
    use std::sync::mpsc;
    use std::time::Duration;

    let directory = tempfile::tempdir().unwrap();
    let lock_path = directory.path().join("artifact.lock");
    let first = acquire_planning_artifact_lock_impl(&lock_path).unwrap();
    let (sender, receiver) = mpsc::channel();
    let contender_path = lock_path.clone();
    let contender = std::thread::spawn(move || {
        let lock = acquire_planning_artifact_lock_impl(&contender_path).unwrap();
        sender.send(lock).unwrap();
    });

    assert!(receiver.recv_timeout(Duration::from_millis(50)).is_err());
    drop(first);
    let second = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
    unlock_planning_artifact_lock_impl(&second).unwrap();
    drop(second);
    contender.join().unwrap();

    let metadata = std::fs::metadata(lock_path).unwrap();
    use std::os::unix::fs::PermissionsExt;
    assert!(metadata.is_file());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

#[cfg(unix)]
#[test]
fn reports_lock_syscall_and_identity_errors() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    std::fs::write(&first, "first").unwrap();
    std::fs::write(&second, "second").unwrap();

    assert!(flock_impl(-1, libc::LOCK_EX).is_err());
    assert_eq!(
        validate_planning_artifact_lock_identity(
            &std::fs::metadata(first).unwrap(),
            &std::fs::metadata(second).unwrap(),
        )
        .unwrap_err()
        .kind(),
        std::io::ErrorKind::InvalidData
    );
}

#[cfg(unix)]
#[test]
fn rejects_linked_lock_paths() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let victim = directory.path().join("victim");
    let symlink_path = directory.path().join("symlink.lock");
    let hardlink_path = directory.path().join("hardlink.lock");
    std::fs::write(&victim, "protected").unwrap();
    symlink(&victim, &symlink_path).unwrap();
    std::fs::hard_link(&victim, &hardlink_path).unwrap();

    assert!(acquire_planning_artifact_lock_impl(&symlink_path).is_err());
    assert_eq!(
        acquire_planning_artifact_lock_impl(&hardlink_path)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData
    );
    assert_eq!(std::fs::read_to_string(victim).unwrap(), "protected");
}
