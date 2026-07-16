use super::{clock, InvocationError, InvocationErrorKind};
use anyhow::{Context, Result};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(super) struct Deadline {
    pub(super) expires_at: Instant,
    pub(super) timeout: Duration,
}

pub(super) fn active_deadline() -> &'static RwLock<Option<Deadline>> {
    static ACTIVE: OnceLock<RwLock<Option<Deadline>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(None))
}

pub(super) struct DeadlineGuard {
    previous: Option<Deadline>,
}

impl DeadlineGuard {
    pub(super) fn install(timeout: Option<Duration>) -> Result<Self> {
        let deadline = timeout
            .map(|timeout| {
                clock::now()
                    .checked_add(timeout)
                    .map(|expires_at| Deadline {
                        expires_at,
                        timeout,
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
    deadline
        .expires_at
        .checked_duration_since(clock::now())
        .map(Some)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::TimedOut, "command timed out"))
}
