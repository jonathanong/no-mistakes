use super::{clock, InvocationError, InvocationErrorKind};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(super) struct Deadline {
    pub(super) expires_at: Instant,
    pub(super) timeout: Duration,
    pub(super) owner: Option<std::thread::ThreadId>,
    pub(super) committed: bool,
}

pub(super) fn active_deadline() -> &'static RwLock<Option<Deadline>> {
    static ACTIVE: OnceLock<RwLock<Option<Deadline>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(None))
}

pub(super) struct DeadlineGuard {
    previous: Option<Deadline>,
    cancel_watch: Option<Arc<AtomicBool>>,
}

impl DeadlineGuard {
    pub(super) fn install_for_invocation(
        timeout: Option<Duration>,
        owner: Option<std::thread::ThreadId>,
    ) -> Result<Self> {
        let deadline = timeout
            .map(|timeout| {
                clock::now()
                    .checked_add(timeout)
                    .map(|expires_at| Deadline {
                        expires_at,
                        timeout,
                        owner,
                        committed: false,
                    })
                    .context("command timeout is too large")
            })
            .transpose()?;
        let mut active = active_deadline()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = std::mem::replace(&mut *active, deadline);
        Ok(Self {
            previous,
            cancel_watch: None,
        })
    }

    /// CLI invocations enforce the deadline even when native analysis is inside
    /// a CPU-bound rayon section that has not reached the next `check_timeout`.
    pub(super) fn install_for_cli(timeout: Option<Duration>) -> Result<Self> {
        let mut guard = Self::install_for_invocation(timeout, None)?;
        let Some(timeout) = timeout else {
            return Ok(guard);
        };
        let expires_at = active_deadline()
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .context("cli timeout installs a deadline")?
            .expires_at;
        let cancel = Arc::new(AtomicBool::new(false));
        let watcher = Arc::clone(&cancel);
        std::thread::Builder::new()
            .name("no-mistakes-timeout".to_string())
            .spawn(move || watch_deadline(watcher, expires_at, timeout))
            .context("starting command timeout watchdog")?;
        guard.cancel_watch = Some(cancel);
        Ok(guard)
    }
}

impl Drop for DeadlineGuard {
    fn drop(&mut self) {
        if let Some(cancel) = &self.cancel_watch {
            cancel.store(true, Ordering::Release);
        }
        *active_deadline()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = self.previous;
    }
}

fn phase_slot() -> &'static RwLock<&'static str> {
    static PHASE: OnceLock<RwLock<&'static str>> = OnceLock::new();
    PHASE.get_or_init(|| RwLock::new("command"))
}

/// Record the command phase included in a timeout diagnostic.
pub fn set_timeout_phase(phase: &'static str) {
    *phase_slot()
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = phase;
}

fn current_phase() -> &'static str {
    *phase_slot()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(super) fn timeout_diagnostic(seconds: u64, phase: &str) -> String {
    format!("command timed out after {seconds} seconds during {phase}")
}

fn watch_deadline(cancel: Arc<AtomicBool>, expires_at: Instant, timeout: Duration) {
    loop {
        if cancel.load(Ordering::Acquire) {
            return;
        }
        let now = clock::now();
        if now >= expires_at {
            if cancel.load(Ordering::Acquire) || !deadline_still_active(expires_at) {
                return;
            }
            let message = timeout_diagnostic(timeout.as_secs(), current_phase());
            eprintln!("error: {message}");
            super::child::process_tree::terminate_registered_groups();
            std::process::exit(124);
        }
        std::thread::sleep(
            expires_at
                .saturating_duration_since(now)
                .min(Duration::from_millis(200)),
        );
    }
}

fn deadline_still_active(expires_at: Instant) -> bool {
    active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_some_and(|deadline| !deadline.committed && deadline.expires_at == expires_at)
}

/// Return an error once the active invocation deadline has elapsed.
pub fn check_timeout() -> Result<()> {
    let deadline = *active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(deadline) = deadline else {
        return Ok(());
    };
    if deadline.committed {
        return Ok(());
    }
    if deadline
        .owner
        .is_some_and(|owner| owner != std::thread::current().id())
    {
        return Ok(());
    }
    if clock::now() < deadline.expires_at {
        return Ok(());
    }
    Err(InvocationError::new(
        InvocationErrorKind::CommandTimeout,
        timeout_diagnostic(deadline.timeout.as_secs(), current_phase()),
    )
    .into())
}

/// Validate the deadline and make output publication the invocation's commit point.
pub fn commit_timeout() -> Result<()> {
    let mut active = active_deadline()
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(deadline) = active.as_mut() else {
        return Ok(());
    };
    if deadline.committed
        || deadline
            .owner
            .is_some_and(|owner| owner != std::thread::current().id())
    {
        return Ok(());
    }
    if clock::now() >= deadline.expires_at {
        return Err(InvocationError::new(
            InvocationErrorKind::CommandTimeout,
            timeout_diagnostic(deadline.timeout.as_secs(), current_phase()),
        )
        .into());
    }
    deadline.committed = true;
    Ok(())
}

pub(super) fn remaining_timeout() -> std::io::Result<Option<Duration>> {
    let deadline = *active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(deadline) = deadline else {
        return Ok(None);
    };
    if deadline.committed {
        return Ok(None);
    }
    if deadline
        .owner
        .is_some_and(|owner| owner != std::thread::current().id())
    {
        return Ok(None);
    }
    deadline
        .expires_at
        .checked_duration_since(clock::now())
        .map(Some)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::TimedOut, "command timed out"))
}

#[cfg(test)]
mod tests;
