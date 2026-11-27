use super::LockLocation;
use anyhow::Result;
use std::path::Path;

#[cfg(unix)]
pub(super) fn temporary_locations() -> impl Iterator<Item = LockLocation> {
    use std::path::PathBuf;

    let temp = std::env::temp_dir();
    let system = PathBuf::from("/tmp");
    [temp, system].into_iter().map(|directory| {
        LockLocation::private_temp(directory.join(format!("no-mistakes-{}", effective_uid())))
    })
}

#[cfg(unix)]
fn effective_uid() -> u32 {
    // SAFETY: geteuid has no arguments and does not modify process state.
    unsafe { libc::geteuid() }
}

#[cfg(unix)]
pub(super) fn prepare_private_temp(directory: &Path) -> Result<()> {
    use std::fs::{self, DirBuilder};
    use std::io::ErrorKind;
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};

    let root = directory
        .parent()
        .expect("temporary lock has a parent directory");
    anyhow::ensure!(root.is_absolute(), "temporary directory must be absolute");
    let metadata = fs::metadata(root)?;
    let uid = effective_uid();
    anyhow::ensure!(
        metadata.is_dir()
            && (metadata.uid() == 0 || metadata.uid() == uid)
            && (metadata.mode() & 0o022 == 0 || metadata.mode() & 0o1000 != 0),
        "untrusted temporary directory {}",
        root.display()
    );

    match DirBuilder::new().mode(0o700).create(directory) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(directory)?;
            anyhow::ensure!(
                metadata.is_dir() && metadata.uid() == uid && metadata.mode() & 0o077 == 0,
                "temporary lock directory {} must be private and owned by the current user",
                directory.display()
            );
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(windows)]
pub(super) fn temporary_locations() -> impl Iterator<Item = LockLocation> {
    directories::BaseDirs::new()
        .map(|base| {
            LockLocation::private_temp(base.data_local_dir().join("Temp").join("no-mistakes"))
        })
        .into_iter()
}

#[cfg(not(any(unix, windows)))]
pub(super) fn temporary_locations() -> impl Iterator<Item = LockLocation> {
    std::iter::empty()
}

#[cfg(not(unix))]
pub(super) fn prepare_private_temp(directory: &Path) -> Result<()> {
    super::create_lock_directory(directory)
}
