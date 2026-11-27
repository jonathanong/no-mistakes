//! Nonblocking report of the selected per-user invocation lock and its holder.
use super::holder::LockHolder;
use super::lock_location::{open_current_lock, OpenedLock};
use anyhow::Result;
use serde::Serialize;
use std::fs::{File, TryLockError};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LockState {
    Held,
    Free,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockStatus {
    pub state: LockState,
    pub lock_file: Option<String>,
    /// Best-effort metadata while held; a handoff can briefly expose the prior holder.
    pub holder: Option<LockHolder>,
}

/// Report whether another invocation currently holds the lock, and who.
///
/// The probe takes a shared advisory lock for an instant and releases it, so it
/// never waits or changes the holder record. A simultaneous fail-fast exclusive
/// acquisition can observe this brief probe as contention.
pub fn lock_status() -> Result<LockStatus> {
    match open_current_lock() {
        Some(OpenedLock { path, file }) => status_for_file(&path, file),
        None => Ok(LockStatus {
            state: LockState::Unavailable,
            lock_file: None,
            holder: None,
        }),
    }
}

pub(super) fn status_for_file(path: &Path, file: File) -> Result<LockStatus> {
    let report = |state, holder| LockStatus {
        state,
        lock_file: Some(path.display().to_string()),
        holder,
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
