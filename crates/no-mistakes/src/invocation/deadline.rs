#[path = "deadline/watch.rs"]
mod watch;
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
        Self::install_for_invocation_with_deadline(timeout, owner, None)
    }

    pub(super) fn install_for_invocation_with_deadline(
        timeout: Option<Duration>,
        owner: Option<std::thread::ThreadId>,
        limit: Option<Deadline>,
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
        let deadline = match (deadline, limit) {
            (Some(command), Some(limit)) => Some(if command.expires_at <= limit.expires_at {
                command
            } else {
                limit
            }),
            (None, limit) => limit,
            (command, None) => command,
        };
        if limit.is_some() && deadline.is_some_and(|deadline| deadline.expires_at <= clock::now()) {
            return Err(InvocationError::new(
                InvocationErrorKind::CommandTimeout,
                "invocation deadline elapsed during lock acquisition".to_string(),
            )
            .into());
        }
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
        Self::install_for_cli_with_deadline(timeout, None)
    }

    pub(super) fn install_for_cli_with_deadline(
        timeout: Option<Duration>,
        limit: Option<Deadline>,
    ) -> Result<Self> {
        let mut guard = Self::install_for_invocation_with_deadline(timeout, None, limit)?;
        let active = active_deadline()
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .copied();
        let Some(deadline) = active else {
            return Ok(guard);
        };
        let expires_at = deadline.expires_at;
        let timeout = deadline.timeout;
        let cancel = Arc::new(AtomicBool::new(false));
        let watcher = Arc::clone(&cancel);
        let observer = crate::diagnostics::current();
        std::thread::Builder::new()
            .name("no-mistakes-timeout".to_string())
            .spawn(move || watch::watch_deadline(watcher, expires_at, timeout, observer))
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

pub(super) fn timeout_diagnostic(timeout: Duration, phase: &str) -> String {
    if timeout.subsec_nanos() == 0 {
        format!(
            "command timed out after {} seconds during {phase}",
            timeout.as_secs()
        )
    } else {
        format!(
            "command timed out after {} milliseconds during {phase}",
            timeout.as_millis()
        )
    }
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
        timeout_diagnostic(deadline.timeout, current_phase()),
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
            timeout_diagnostic(deadline.timeout, current_phase()),
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
#[path = "deadline/tests.rs"]
mod tests;
