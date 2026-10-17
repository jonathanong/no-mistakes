use super::Deadline;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr;
#[cfg(test)]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{fence, AtomicU64, Ordering};
use std::sync::{LockResult, OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

#[cfg(test)]
pub(super) static PAUSE_WHILE_ODD: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
pub(super) static PAUSE_ODD_REACHED: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
pub(super) static ODD_SPINS: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
pub(super) static PAUSE_BEFORE_SECOND: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
pub(super) static PAUSE_SECOND_REACHED: AtomicBool = AtomicBool::new(false);

pub(in crate::invocation) struct ActiveDeadline {
    lock: RwLock<Option<Deadline>>,
}

struct Published {
    generation: AtomicU64,
    snapshot: UnsafeCell<Option<Deadline>>,
}

// SAFETY: publishers hold the active-deadline write lock, so only one thread
// writes the snapshot. Readers copy into `MaybeUninit` and keep the bits only
// when both generation loads observe the same even value.
unsafe impl Sync for Published {}

static PUBLISHED: Published = Published {
    generation: AtomicU64::new(0),
    snapshot: UnsafeCell::new(None),
};

impl Published {
    /// Publish `snapshot` for lock-free readers.
    ///
    /// While the caller holds the write lock: Release-store an odd generation,
    /// write the `Copy` snapshot, then Release-store the next even generation.
    /// Release fences keep that write after the odd store and before the even
    /// store. Readers Acquire-load the generation, retry while it is odd, copy
    /// the snapshot, Acquire-load again, and retry when the generation changed.
    fn publish(&self, snapshot: Option<Deadline>) {
        let generation = self.generation.load(Ordering::Relaxed).wrapping_add(1);
        self.generation.store(generation, Ordering::Release);
        fence(Ordering::Release);
        unsafe {
            // SAFETY: the write lock admits one publisher, and `snapshot` is a
            // valid `Copy` value. Readers interpret the cell only after a stable
            // even generation.
            ptr::write_volatile(self.snapshot.get(), snapshot);
        }
        #[cfg(test)]
        if PAUSE_WHILE_ODD.load(Ordering::Acquire) {
            PAUSE_ODD_REACHED.store(true, Ordering::Release);
            while PAUSE_WHILE_ODD.load(Ordering::Acquire) {
                std::hint::spin_loop();
            }
        }
        fence(Ordering::Release);
        self.generation
            .store(generation.wrapping_add(1), Ordering::Release);
    }

    fn load(&self) -> Option<Deadline> {
        loop {
            let first = self.generation.load(Ordering::Acquire);
            if first & 1 != 0 {
                #[cfg(test)]
                ODD_SPINS.fetch_add(1, Ordering::Relaxed);
                std::hint::spin_loop();
                continue;
            }
            let snapshot = unsafe {
                // SAFETY: `MaybeUninit` accepts a torn bit pattern. Those bits
                // are read as `Deadline` only if the generation is unchanged.
                ptr::read_volatile(self.snapshot.get().cast::<MaybeUninit<Option<Deadline>>>())
            };
            fence(Ordering::Acquire);
            #[cfg(test)]
            if PAUSE_BEFORE_SECOND.load(Ordering::Acquire) {
                PAUSE_SECOND_REACHED.store(true, Ordering::Release);
                while PAUSE_BEFORE_SECOND.load(Ordering::Acquire) {
                    std::hint::spin_loop();
                }
            }
            let second = self.generation.load(Ordering::Acquire);
            if first == second {
                return unsafe {
                    // SAFETY: both Acquire loads saw the same even generation, so
                    // this copy is one complete published snapshot.
                    snapshot.assume_init()
                };
            }
            std::hint::spin_loop();
        }
    }
}

pub(in crate::invocation) fn active_deadline() -> &'static ActiveDeadline {
    static ACTIVE: OnceLock<ActiveDeadline> = OnceLock::new();
    ACTIVE.get_or_init(|| ActiveDeadline {
        lock: RwLock::new(None),
    })
}

pub(super) fn published_deadline() -> Option<Deadline> {
    PUBLISHED.load()
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

#[cfg(test)]
impl ActiveDeadline {
    pub(super) fn clear_poison_for_test(&self) {
        self.lock.clear_poison();
    }
}

pub(in crate::invocation) struct ActiveWriteGuard<'a> {
    guard: RwLockWriteGuard<'a, Option<Deadline>>,
}

impl Drop for ActiveWriteGuard<'_> {
    fn drop(&mut self) {
        // The write guard still holds the lock; publish before it unlocks.
        PUBLISHED.publish(*self.guard);
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
