//! Read-only report of the machine-wide invocation lock and its holder.
use super::holder::LockHolder;
use super::lock::lock_path;
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs::{File, TryLockError};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LockState {
    Held,
    Free,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockStatus {
    pub state: LockState,
    pub lock_file: String,
    /// Present only while the lock is held; stale records are never reported.
    pub holder: Option<LockHolder>,
}

/// Report whether another invocation currently holds the lock, and who.
///
/// The probe takes a shared advisory lock for an instant and releases it, so it
/// never waits and never changes the holder record.
pub fn lock_status() -> Result<LockStatus> {
    status_at(&lock_path()?)
}

pub(super) fn status_at(path: &Path) -> Result<LockStatus> {
    let report = |state, holder| LockStatus {
        state,
        lock_file: path.display().to_string(),
        holder,
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(report(LockState::Free, None));
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("opening invocation lock {}", path.display()))
        }
    };
    match file.try_lock_shared() {
        Ok(()) => {
            file.unlock()?;
            Ok(report(LockState::Free, None))
        }
        Err(TryLockError::WouldBlock) => Ok(report(LockState::Held, LockHolder::read(path))),
        Err(TryLockError::Error(error)) => Err(super::lock::lock_error(path, error)),
    }
}
