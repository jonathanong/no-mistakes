#[path = "deadline/phase.rs"]
mod phase;
use phase::current_phase;
pub use phase::set_timeout_phase;
pub(super) use phase::timeout_diagnostic;
#[path = "deadline/publish.rs"]
mod publish;
pub(super) use publish::active_deadline;
#[path = "deadline/watch.rs"]
mod watch;
use super::{clock, InvocationError, InvocationErrorKind};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
#[path = "deadline/cli.rs"]
mod cli;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(super) struct Deadline {
    pub(super) expires_at: Instant,
    pub(super) timeout: Duration,
    pub(super) owner: Option<std::thread::ThreadId>,
    pub(super) committed: bool,
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
        let started = clock::now();
        let deadline = timeout
            .map(|timeout| {
                started
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
        // Acquisition expiry belongs to the original invocation limit at entry,
        // not a shorter command deadline that expires while this guard is built.
        if limit.is_some_and(|limit| limit.expires_at <= started) {
            return Err(InvocationError::new(
                InvocationErrorKind::CommandTimeout,
                "invocation deadline elapsed during lock acquisition".to_string(),
            )
            .with_phase("invocation.lock")
            .into());
        }
        let deadline = match (deadline, limit) {
            (Some(command), Some(limit)) => Some(if command.expires_at <= limit.expires_at {
                command
            } else {
                limit
            }),
            (None, limit) => limit,
            (command, None) => command,
        };
        let mut active = active_deadline()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = std::mem::replace(&mut *active, deadline);
        Ok(Self {
            previous,
            cancel_watch: None,
        })
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

/// Snapshot that applies to this thread. Empty, committed, and foreign-owner
/// deadlines return before any clock read.
fn applicable_deadline() -> Option<Deadline> {
    let deadline = publish::published_deadline()?;
    if !deadline.applies_here() {
        return None;
    }
    Some(Deadline {
        expires_at: deadline.expires_at,
        timeout: deadline.timeout,
        owner: None,
        committed: false,
    })
}

/// Return an error once the active invocation deadline has elapsed.
pub fn check_timeout() -> Result<()> {
    let Some(timeout) = publish::expired_timeout() else {
        return Ok(());
    };
    Err(InvocationError::new(
        InvocationErrorKind::CommandTimeout,
        timeout_diagnostic(timeout, current_phase()),
    )
    .with_phase(current_phase())
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
        .with_phase(current_phase())
        .into());
    }
    deadline.committed = true;
    Ok(())
}

pub(super) fn remaining_timeout() -> std::io::Result<Option<Duration>> {
    publish::remaining_from_snapshots(applicable_deadline)
}

#[cfg(test)]
#[path = "deadline/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "deadline/large_timeout_tests.rs"]
mod large_timeout;
