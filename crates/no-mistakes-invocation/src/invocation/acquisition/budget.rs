use super::{clock, open_current_lock, LockTarget};
use crate::invocation::deadline::{self, Deadline};
use crate::invocation::lock::{acquire_open_lock, open_lock_file};
use crate::invocation::lock_location::OpenedLock;
use crate::invocation::{InvocationError, InvocationErrorKind, InvocationOptions};
use anyhow::Result;
use std::fs::File;

pub(super) fn acquire_lock_with_budget(
    target: LockTarget,
    options: InvocationOptions,
    invocation_deadline: Option<Deadline>,
) -> Result<Option<File>> {
    check_acquisition_deadline(invocation_deadline)?;
    let acquisition_started = clock::now();
    let opened = match target {
        LockTarget::Current => open_current_lock(),
        LockTarget::Exact(path) => {
            let file = open_lock_file(&path)?;
            Some(OpenedLock { path, file })
        }
    };
    let Some(OpenedLock { path, file }) = opened else {
        check_acquisition_deadline(invocation_deadline)?;
        return Ok(None);
    };
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
    let result = acquire_open_lock(
        &path,
        file,
        lock_timeout,
        options.fail_on_lock,
        invocation_deadline.map(|_| acquisition_started),
    );
    classify_lock_result(result, invocation_deadline, invocation_limited).map(Some)
}

fn check_acquisition_deadline(invocation_deadline: Option<Deadline>) -> Result<()> {
    if invocation_deadline.is_some_and(|deadline| deadline.expires_at <= clock::now()) {
        return Err(InvocationError::new(
            InvocationErrorKind::CommandTimeout,
            "invocation deadline elapsed during lock acquisition",
        )
        .with_phase("invocation.lock")
        .into());
    }
    Ok(())
}

pub(in crate::invocation) fn classify_lock_result(
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
