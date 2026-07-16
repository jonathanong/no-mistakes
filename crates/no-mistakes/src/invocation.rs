//! Process-wide invocation coordination shared by the CLI and N-API entrypoints.

mod child;
mod clock;
mod deadline;
mod lock;
mod napi_options;

pub use child::command_output;
pub use deadline::check_timeout;
pub use napi_options::extract_napi_options;

use anyhow::Result;
use deadline::DeadlineGuard;
use lock::{acquire_lock, lock_path};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;

#[derive(clap::Args, Debug, Clone, Copy)]
pub struct InvocationArgs {
    /// Maximum command execution time in seconds; 0 disables the deadline.
    #[arg(long, value_name = "SECONDS", default_value_t = DEFAULT_TIMEOUT_SECONDS, global = true)]
    timeout: u64,
    /// Maximum time to wait for another invocation in seconds; 0 waits indefinitely.
    #[arg(
        long = "lock-timeout",
        value_name = "SECONDS",
        default_value_t = DEFAULT_TIMEOUT_SECONDS,
        global = true
    )]
    lock_timeout: u64,
    /// Fail immediately when another no-mistakes invocation holds the lock.
    #[arg(long, global = true)]
    fail_on_lock: bool,
}

impl Default for InvocationArgs {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT_SECONDS,
            lock_timeout: DEFAULT_TIMEOUT_SECONDS,
            fail_on_lock: false,
        }
    }
}

impl InvocationArgs {
    pub fn options(self) -> InvocationOptions {
        InvocationOptions {
            timeout: nonzero_seconds(self.timeout),
            lock_timeout: nonzero_seconds(self.lock_timeout),
            fail_on_lock: self.fail_on_lock,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvocationOptions {
    pub timeout: Option<Duration>,
    pub lock_timeout: Option<Duration>,
    pub fail_on_lock: bool,
}

impl Default for InvocationOptions {
    fn default() -> Self {
        InvocationArgs::default().options()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationErrorKind {
    LockBusy,
    LockTimeout,
    CommandTimeout,
}

#[derive(Debug)]
pub struct InvocationError {
    kind: InvocationErrorKind,
    message: String,
}

impl InvocationError {
    pub(super) fn new(kind: InvocationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> InvocationErrorKind {
        self.kind
    }
}

impl std::fmt::Display for InvocationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for InvocationError {}

pub struct InvocationGuard {
    _deadline: DeadlineGuard,
    _lock: File,
}

impl InvocationGuard {
    pub fn acquire(options: InvocationOptions) -> Result<Self> {
        let path = lock_path()?;
        Self::acquire_at_path(options, &path)
    }

    fn acquire_at_path(options: InvocationOptions, path: &Path) -> Result<Self> {
        let lock = acquire_lock(path, options.lock_timeout, options.fail_on_lock)?;
        let deadline = DeadlineGuard::install(options.timeout)?;
        Ok(Self {
            _deadline: deadline,
            _lock: lock,
        })
    }
}

pub fn timeout_exit_code(error: &anyhow::Error) -> Option<u8> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<InvocationError>())
        .and_then(|error| {
            matches!(
                error.kind(),
                InvocationErrorKind::LockTimeout | InvocationErrorKind::CommandTimeout
            )
            .then_some(124)
        })
}

pub(super) fn nonzero_seconds(seconds: u64) -> Option<Duration> {
    (seconds != 0).then(|| Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests;
