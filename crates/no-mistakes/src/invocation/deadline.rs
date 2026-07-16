use super::{clock, InvocationError, InvocationErrorKind};
use anyhow::{Context, Result};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(super) struct Deadline {
    pub(super) expires_at: Instant,
    pub(super) timeout: Duration,
    pub(super) owner: Option<std::thread::ThreadId>,
}

pub(super) fn active_deadline() -> &'static RwLock<Option<Deadline>> {
    static ACTIVE: OnceLock<RwLock<Option<Deadline>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(None))
}

pub(super) struct DeadlineGuard {
    previous: Option<Deadline>,
}

impl DeadlineGuard {
    #[cfg(any(test, feature = "test-instrumentation"))]
    pub(super) fn install_with_owner(
        timeout: Option<Duration>,
        owner: Option<std::thread::ThreadId>,
    ) -> Result<Self> {
        Self::install_for_invocation(timeout, owner)
    }

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
                    })
                    .context("command timeout is too large")
            })
            .transpose()?;
        let mut active = active_deadline()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = std::mem::replace(&mut *active, deadline);
        Ok(Self { previous })
    }
}

impl Drop for DeadlineGuard {
    fn drop(&mut self) {
        *active_deadline()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = self.previous;
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
        format!(
            "command timed out after {} seconds",
            deadline.timeout.as_secs()
        ),
    )
    .into())
}

pub(super) fn remaining_timeout() -> std::io::Result<Option<Duration>> {
    let deadline = *active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(deadline) = deadline else {
        return Ok(None);
    };
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
