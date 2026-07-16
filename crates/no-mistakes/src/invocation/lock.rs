use super::{clock, InvocationError, InvocationErrorKind};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::time::Duration;

const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub(super) fn lock_path() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", "no-mistakes")
        .context("could not determine the current user's invocation lock directory")?;
    let directory = project_dirs
        .runtime_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project_dirs.cache_dir().to_path_buf());
    std::fs::create_dir_all(&directory).with_context(|| {
        format!(
            "creating no-mistakes invocation lock directory {}",
            directory.display()
        )
    })?;
    Ok(directory.join("invocation.lock"))
}

pub(super) fn acquire_lock(
    path: &Path,
    timeout: Option<Duration>,
    fail_on_lock: bool,
) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("opening invocation lock {}", path.display()))?;
    let started = clock::now();
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::Error(error)) => {
                return Err(error)
                    .with_context(|| format!("locking invocation file {}", path.display()));
            }
            Err(TryLockError::WouldBlock) if fail_on_lock => {
                return Err(InvocationError::new(
                    InvocationErrorKind::LockBusy,
                    "another no-mistakes invocation is already running",
                )
                .into());
            }
            Err(TryLockError::WouldBlock) => {}
        }

        let sleep_for = match timeout {
            Some(timeout) => {
                let Some(remaining) = timeout.checked_sub(started.elapsed()) else {
                    return Err(InvocationError::new(
                        InvocationErrorKind::LockTimeout,
                        format!(
                            "timed out after {} seconds waiting for another no-mistakes invocation",
                            timeout.as_secs()
                        ),
                    )
                    .into());
                };
                remaining.min(LOCK_POLL_INTERVAL)
            }
            None => LOCK_POLL_INTERVAL,
        };
        std::thread::sleep(sleep_for);
    }
}
