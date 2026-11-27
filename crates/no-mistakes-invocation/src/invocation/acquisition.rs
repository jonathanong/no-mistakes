use super::deadline::{Deadline, DeadlineGuard};
use super::lock_location::open_current_lock;
use super::{child, clock, set_timeout_phase, CliErrorOutput, InvocationGuard, InvocationOptions};
use anyhow::{Context, Result};
use std::path::PathBuf;
#[path = "acquisition/budget.rs"]
mod budget;
use budget::acquire_lock_with_budget;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
pub(super) use budget::classify_lock_result;
#[path = "acquisition/cli.rs"]
mod cli;

impl InvocationGuard {
    pub fn acquire(options: InvocationOptions) -> Result<Self> {
        Self::acquire_resolving_lock(
            options,
            false,
            || Ok(LockTarget::Current),
            CliErrorOutput::new(false),
        )
    }

    pub(super) fn acquire_resolving_path(
        options: InvocationOptions,
        forward_parent_signals: bool,
        resolve_path: impl FnOnce() -> Result<std::path::PathBuf>,
    ) -> Result<Self> {
        Self::acquire_resolving_path_with_json(options, forward_parent_signals, resolve_path, false)
    }

    pub(super) fn acquire_resolving_path_with_json(
        options: InvocationOptions,
        forward_parent_signals: bool,
        resolve_path: impl FnOnce() -> Result<std::path::PathBuf>,
        json: bool,
    ) -> Result<Self> {
        Self::acquire_resolving_lock(
            options,
            forward_parent_signals,
            || resolve_path().map(LockTarget::Exact),
            CliErrorOutput::new(json),
        )
    }

    fn acquire_resolving_lock(
        options: InvocationOptions,
        forward_parent_signals: bool,
        resolve_lock: impl FnOnce() -> Result<LockTarget>,
        output: CliErrorOutput,
    ) -> Result<Self> {
        let invocation_deadline = options
            .invocation_timeout
            .map(|timeout| {
                clock::now()
                    .checked_add(timeout)
                    .map(|expires_at| Deadline {
                        expires_at,
                        timeout,
                        owner: None,
                        committed: false,
                    })
                    .context("invocation timeout is too large")
            })
            .transpose()?;
        // CLI owns its process. N-API waiters must never replace the active
        // request's process-global deadline, cancellation, or child registry.
        let early_deadline = if forward_parent_signals && invocation_deadline.is_some() {
            set_timeout_phase("invocation.lock");
            Some(DeadlineGuard::install_for_cli_with_output(
                None,
                invocation_deadline,
                output.clone(),
            )?)
        } else {
            None
        };
        // Progress must capture termination during lock waiting, before the
        // first heartbeat. Retaining this guard also cleans up acquisition errors.
        let early_signals = if forward_parent_signals
            && crate::hooks::current_observer().is_some_and(|observer| observer.progress_enabled())
        {
            Some(child::ParentSignalForwardingGuard::install(true)?)
        } else {
            None
        };
        let target = resolve_lock()?;
        let lock = acquire_lock_with_budget(target, options, invocation_deadline)?;
        // Keep the original absolute invocation deadline. Cancel the acquisition
        // watcher before arming the earlier of that deadline and command timeout.
        drop(early_deadline);
        if forward_parent_signals && invocation_deadline.is_some() {
            set_timeout_phase("command");
        }
        let deadline = match (forward_parent_signals, invocation_deadline) {
            (true, None) => {
                DeadlineGuard::install_for_cli_with_output(options.timeout, None, output.clone())?
            }
            (false, None) => DeadlineGuard::install_for_invocation(options.timeout, None)?,
            (true, limit) => {
                DeadlineGuard::install_for_cli_with_output(options.timeout, limit, output)?
            }
            (false, limit) => {
                DeadlineGuard::install_for_invocation_with_deadline(options.timeout, None, limit)?
            }
        };
        let parent_signals = match early_signals {
            Some(guard) => guard,
            None => child::ParentSignalForwardingGuard::install(
                forward_parent_signals
                    && (options.timeout.is_some() || options.invocation_timeout.is_some()),
            )?,
        };
        Ok(Self {
            _deadline: deadline,
            _parent_signals: parent_signals,
            _lock: lock,
        })
    }
}

enum LockTarget {
    Current,
    Exact(PathBuf),
}
