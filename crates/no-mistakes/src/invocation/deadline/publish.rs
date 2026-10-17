use super::Deadline;
use std::ops::{Deref, DerefMut};
use std::sync::{LockResult, OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

pub(super) mod pauses;
pub(super) mod snapshot;

pub(in crate::invocation) struct ActiveDeadline {
    lock: RwLock<Option<Deadline>>,
}

pub(in crate::invocation) fn active_deadline() -> &'static ActiveDeadline {
    static ACTIVE: OnceLock<ActiveDeadline> = OnceLock::new();
    ACTIVE.get_or_init(|| ActiveDeadline {
        lock: RwLock::new(None),
    })
}

pub(super) fn published_deadline() -> Option<snapshot::Observed> {
    snapshot::load()
}

/// Two snapshots that are both already expired. A commit or restore that
/// publishes between the loads returns no deadline.
pub(super) fn consecutive_expired(mut load: impl FnMut() -> Option<Deadline>) -> Option<Deadline> {
    let deadline = load()?;
    if super::super::clock::now() < deadline.expires_at {
        return None;
    }
    let deadline = load()?;
    if super::super::clock::now() < deadline.expires_at {
        return None;
    }
    Some(deadline)
}

/// Remaining time from two snapshots. The second load drops a deadline that
/// was cleared after the first copy saw it already expired.
pub(super) fn remaining_from_snapshots(
    mut load: impl FnMut() -> Option<Deadline>,
) -> std::io::Result<Option<Duration>> {
    let Some(deadline) = load() else {
        return Ok(None);
    };
    if let Some(remaining) = deadline
        .expires_at
        .checked_duration_since(super::super::clock::now())
    {
        return Ok(Some(remaining));
    }
    let Some(deadline) = load() else {
        return Ok(None);
    };
    deadline
        .expires_at
        .checked_duration_since(super::super::clock::now())
        .map(Some)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::TimedOut, "command timed out"))
}

pub(in crate::invocation) struct ActiveWriteGuard<'a> {
    guard: RwLockWriteGuard<'a, Option<Deadline>>,
}

impl Drop for ActiveWriteGuard<'_> {
    fn drop(&mut self) {
        // The write guard still holds the lock; publish before it unlocks.
        snapshot::publish(*self.guard);
    }
}

impl Deref for ActiveWriteGuard<'_> {
    type Target = Option<Deadline>;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl DerefMut for ActiveWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl ActiveDeadline {
    pub(in crate::invocation) fn read(&self) -> LockResult<RwLockReadGuard<'_, Option<Deadline>>> {
        self.lock.read()
    }

    pub(in crate::invocation) fn write(&self) -> LockResult<ActiveWriteGuard<'_>> {
        match self.lock.write() {
            Ok(guard) => Ok(ActiveWriteGuard { guard }),
            Err(poisoned) => Err(PoisonError::new(ActiveWriteGuard {
                guard: poisoned.into_inner(),
            })),
        }
    }
}
