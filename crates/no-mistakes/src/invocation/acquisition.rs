use super::deadline::{self, Deadline, DeadlineGuard};
use super::lock::{acquire_lock, acquire_lock_from, lock_path};
use super::{
    child, clock, set_timeout_phase, CliErrorOutput, InvocationError, InvocationErrorKind,
    InvocationGuard, InvocationOptions,
};
use anyhow::{Context, Result};
#[path = "acquisition/cli.rs"]
mod cli;

impl InvocationGuard {
    pub fn acquire(options: InvocationOptions) -> Result<Self> {
        Self::acquire_resolving_path(options, false, lock_path)
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
        Self::acquire_resolving_path_with_output(
            options,
            forward_parent_signals,
            resolve_path,
            CliErrorOutput::new(json),
        )
    }

    fn acquire_resolving_path_with_output(
        options: InvocationOptions,
        forward_parent_signals: bool,
        resolve_path: impl FnOnce() -> Result<std::path::PathBuf>,
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
            && crate::diagnostics::current().is_some_and(|observer| observer.progress_enabled())
        {
            Some(child::ParentSignalForwardingGuard::install(true)?)
        } else {
            None
        };
        let path = resolve_path()?;
        let lock = acquire_lock_with_budget(&path, options, invocation_deadline)?;
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

fn acquire_lock_with_budget(
    path: &std::path::Path,
    options: InvocationOptions,
    invocation_deadline: Option<Deadline>,
) -> Result<std::fs::File> {
    // Path resolution may consume the budget before either a fail-fast or free open.
    if invocation_deadline.is_some_and(|deadline| deadline.expires_at <= clock::now()) {
        return Err(InvocationError::new(
            InvocationErrorKind::CommandTimeout,
            "invocation deadline elapsed during lock acquisition",
        )
        .with_phase("invocation.lock")
        .into());
    }
    let acquisition_started = clock::now();
    let remaining = invocation_deadline.map(|deadline| {
        deadline
            .expires_at
            .saturating_duration_since(acquisition_started)
    });
    // Fail-fast contention overrides the explicit wait limit, not the shared budget.
    let lock_limit = options.lock_timeout.filter(|_| !options.fail_on_lock);
    let invocation_limited =
        remaining.is_some_and(|remaining| lock_limit.is_none_or(|lock| remaining <= lock));
    let lock_timeout = match (remaining, lock_limit) {
        (Some(remaining), Some(lock)) => Some(remaining.min(lock)),
        (Some(remaining), None) => Some(remaining),
        (None, lock) => lock,
    };
    let result = match invocation_deadline {
        Some(_) => acquire_lock_from(
            path,
            lock_timeout,
            options.fail_on_lock,
            Some(acquisition_started),
        ),
        None => acquire_lock(path, lock_timeout, options.fail_on_lock),
    };
    classify_lock_result(result, invocation_deadline, invocation_limited)
}

pub(super) fn classify_lock_result(
    result: Result<std::fs::File>,
    invocation_deadline: Option<Deadline>,
    invocation_limited: bool,
) -> Result<std::fs::File> {
    match result {
        Err(error)
            if invocation_deadline.is_some()
                && error
                    .downcast_ref::<InvocationError>()
                    .is_some_and(|error| match error.kind() {
                        InvocationErrorKind::LockTimeout => invocation_limited,
                        InvocationErrorKind::LockBusy => invocation_deadline
                            .is_some_and(|deadline| deadline.expires_at <= clock::now()),
                        _ => false,
                    }) =>
        {
            let timeout = invocation_deadline
                .expect("expired invocation deadline")
                .timeout;
            Err(InvocationError::new(
                InvocationErrorKind::CommandTimeout,
                deadline::timeout_diagnostic(timeout, "invocation.lock"),
            )
            .with_phase("invocation.lock")
            .with_holder(
                error
                    .downcast_ref::<InvocationError>()
                    .and_then(|error| error.holder().cloned()),
            )
            .into())
        }
        result => result,
    }
}
