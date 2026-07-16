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

#[cfg(any(test, feature = "test-instrumentation"))]
#[doc(hidden)]
pub fn install_test_deadline(timeout: Duration) -> Result<impl Drop> {
    let serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let deadline =
        DeadlineGuard::install_with_owner(Some(timeout), Some(std::thread::current().id()))?;
    Ok(TestDeadlineGuard {
        _deadline: deadline,
        _serial: serial,
    })
}

#[cfg(any(test, feature = "test-instrumentation"))]
struct TestDeadlineGuard {
    _deadline: DeadlineGuard,
    _serial: std::sync::MutexGuard<'static, ()>,
}

#[cfg(any(test, feature = "test-instrumentation"))]
impl Drop for TestDeadlineGuard {
    fn drop(&mut self) {}
}

#[cfg(any(test, feature = "test-instrumentation"))]
fn deadline_test_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

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
    _parent_signals: child::ParentSignalForwardingGuard,
    _lock: File,
}

impl InvocationGuard {
    pub fn acquire(options: InvocationOptions) -> Result<Self> {
        Self::acquire_with_parent_signal_forwarding(options, false)
    }

    pub fn acquire_for_cli(options: InvocationOptions) -> Result<Self> {
        Self::acquire_with_parent_signal_forwarding(options, true)
    }

    fn acquire_with_parent_signal_forwarding(
        options: InvocationOptions,
        forward_parent_signals: bool,
    ) -> Result<Self> {
        let path = lock_path()?;
        Self::acquire_at_path(options, &path, forward_parent_signals)
    }

    fn acquire_at_path(
        options: InvocationOptions,
        path: &Path,
        forward_parent_signals: bool,
    ) -> Result<Self> {
        let lock = acquire_lock(path, options.lock_timeout, options.fail_on_lock)?;
        let deadline = DeadlineGuard::install_for_invocation(options.timeout, None)?;
        let parent_signals = child::ParentSignalForwardingGuard::install(
            forward_parent_signals && options.timeout.is_some(),
        )?;
        Ok(Self {
            _deadline: deadline,
            _parent_signals: parent_signals,
            _lock: lock,
        })
    }
}

pub fn timeout_exit_code(error: &anyhow::Error) -> Option<u8> {
    error.chain().find_map(|cause| {
        if cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|error| error.kind() == std::io::ErrorKind::TimedOut)
        {
            return Some(124);
        }
        cause
            .downcast_ref::<InvocationError>()
            .is_some_and(|error| {
                matches!(
                    error.kind(),
                    InvocationErrorKind::LockTimeout | InvocationErrorKind::CommandTimeout
                )
            })
            .then_some(124)
    })
}

pub(super) fn nonzero_seconds(seconds: u64) -> Option<Duration> {
    (seconds != 0).then(|| Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests;
