use super::holder::LockHolder;
use super::{clock, InvocationError, InvocationErrorKind};
use anyhow::{Context, Result};
use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;
use std::time::{Duration, Instant};

const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub(super) fn acquire_lock(
    path: &Path,
    timeout: Option<Duration>,
    fail_on_lock: bool,
) -> Result<File> {
    acquire_lock_from(path, timeout, fail_on_lock, None)
}

// Shared-budget callers anchor acquisition before opening the file. Ordinary
// callers retain their existing wait-only clock and first-attempt semantics.
pub(super) fn acquire_lock_from(
    path: &Path,
    timeout: Option<Duration>,
    fail_on_lock: bool,
    acquisition_started: Option<Instant>,
) -> Result<File> {
    acquire_open_lock(
        path,
        open_lock_file(path)?,
        timeout,
        fail_on_lock,
        acquisition_started,
    )
}

pub(super) fn open_lock_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("opening invocation lock {}", path.display()))
}

pub(super) fn acquire_open_lock(
    path: &Path,
    file: File,
    timeout: Option<Duration>,
    fail_on_lock: bool,
    acquisition_started: Option<Instant>,
) -> Result<File> {
    let started = acquisition_started.unwrap_or_else(clock::now);
    let mut first_attempt = true;
    let mut last_reported_secs = None;
    loop {
        if !first_attempt || acquisition_started.is_some() {
            if let Some(timeout) = timeout.filter(|timeout| started.elapsed() >= *timeout) {
                return Err(lock_timeout_error(timeout, LockHolder::read(path)));
            }
        }
        match classify_try_lock(file.try_lock(), path, fail_on_lock) {
            TryLockOutcome::Acquired => {
                // Diagnostic only: a failed record must not fail the invocation.
                let _ = LockHolder::current().record(path);
                return Ok(file);
            }
            TryLockOutcome::Failed(error) => return Err(error),
            TryLockOutcome::Wait => {
                report_lock_wait(path, started.elapsed(), &mut last_reported_secs);
            }
        }
        first_attempt = false;

        let sleep_for = match timeout {
            Some(timeout) => timeout
                .saturating_sub(started.elapsed())
                .min(LOCK_POLL_INTERVAL),
            None => LOCK_POLL_INTERVAL,
        };
        std::thread::sleep(sleep_for);
    }
}

enum TryLockOutcome {
    Acquired,
    Wait,
    Failed(anyhow::Error),
}

fn classify_try_lock(
    result: Result<(), TryLockError>,
    path: &Path,
    fail_on_lock: bool,
) -> TryLockOutcome {
    match result {
        Ok(()) => TryLockOutcome::Acquired,
        Err(TryLockError::Error(error)) => TryLockOutcome::Failed(lock_error(path, error)),
        Err(TryLockError::WouldBlock) if fail_on_lock => {
            TryLockOutcome::Failed(lock_busy_error(LockHolder::read(path)))
        }
        Err(TryLockError::WouldBlock) => TryLockOutcome::Wait,
    }
}

fn report_lock_wait(path: &Path, elapsed: Duration, last_reported_secs: &mut Option<u64>) {
    let secs = elapsed.as_secs();
    if last_reported_secs.is_some_and(|previous| previous == secs) {
        return;
    }
    *last_reported_secs = Some(secs);
    let holder = match LockHolder::read(path) {
        Some(holder) => holder.describe(),
        None => "another no-mistakes invocation".to_string(),
    };
    crate::hooks::print_stderr(format_args!(
        "waiting for lock held by {holder} for {secs}s"
    ));
}

fn held_by(holder: &Option<LockHolder>) -> String {
    match holder {
        Some(holder) => format!(" (held by {})", holder.describe()),
        None => String::new(),
    }
}

pub(super) fn lock_busy_error(holder: Option<LockHolder>) -> anyhow::Error {
    InvocationError::new(
        InvocationErrorKind::LockBusy,
        format!(
            "another no-mistakes invocation is already running{}",
            held_by(&holder)
        ),
    )
    .with_phase("invocation.lock")
    .with_holder(holder)
    .into()
}

pub(super) fn lock_timeout_error(timeout: Duration, holder: Option<LockHolder>) -> anyhow::Error {
    InvocationError::new(
        InvocationErrorKind::LockTimeout,
        format!(
            "timed out after {} seconds waiting for another no-mistakes invocation{}",
            timeout.as_secs(),
            held_by(&holder)
        ),
    )
    .with_phase("invocation.lock")
    .with_holder(holder)
    .into()
}

pub(super) fn lock_error(path: &Path, error: std::io::Error) -> anyhow::Error {
    anyhow::Error::new(error).context(format!("locking invocation file {}", path.display()))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "lock/tests.rs"]
mod tests;
