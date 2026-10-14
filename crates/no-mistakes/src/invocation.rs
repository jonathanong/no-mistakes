//! Process-wide invocation coordination shared by the CLI and N-API entrypoints.

#[path = "invocation/child.rs"]
mod child;
#[path = "invocation/clock.rs"]
mod clock;
#[path = "invocation/deadline.rs"]
mod deadline;
#[path = "invocation/lock.rs"]
mod lock;
#[path = "invocation/napi_options.rs"]
mod napi_options;

pub use child::command_output;
pub(crate) use child::stream::stream_command_lines;
pub use deadline::{check_timeout, commit_timeout, set_timeout_phase};
pub use napi_options::{extract_napi_options, extract_napi_options_value, InvocationOptions};

use anyhow::{Context, Result};
use deadline::{Deadline, DeadlineGuard};
use lock::{acquire_lock, lock_path};
use std::fs::File;
#[cfg(test)]
use std::path::Path;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const MAX_INVOCATION_TIMEOUT_MS: u64 = (1_u64 << 53) - 1;

// Windows native CI compiles this helper into the ordinary rlib so the Job
// Object integration test can install a deadline without `--cfg test`.
#[cfg(any(test, feature = "test-instrumentation", windows))]
#[doc(hidden)]
pub fn install_test_deadline(timeout: Duration) -> Result<impl Drop> {
    let serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let deadline =
        DeadlineGuard::install_for_invocation(Some(timeout), Some(std::thread::current().id()))?;
    Ok(TestDeadlineGuard {
        _deadline: deadline,
        _serial: serial,
    })
}

#[cfg(any(test, feature = "test-instrumentation", windows))]
struct TestDeadlineGuard {
    _deadline: DeadlineGuard,
    _serial: std::sync::MutexGuard<'static, ()>,
}

#[cfg(any(test, feature = "test-instrumentation", windows))]
impl Drop for TestDeadlineGuard {
    fn drop(&mut self) {}
}

#[cfg(any(test, feature = "test-instrumentation", windows))]
fn deadline_test_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum InvocationProfile {
    /// Unbounded command and lock wait (`timeout: 0`, `lock-timeout: 0`).
    Ci,
}

#[derive(clap::Args, Debug, Clone, Copy)]
pub struct InvocationArgs {
    /// Maximum command execution time in seconds; 0 disables the deadline.
    #[arg(long, value_name = "SECONDS", default_value_t = DEFAULT_TIMEOUT_SECONDS, global = true)]
    timeout: u64,
    /// Opt-in shared acquisition and execution budget in positive milliseconds.
    #[arg(long, value_name = "MILLISECONDS", value_parser = clap::value_parser!(u64).range(1..=MAX_INVOCATION_TIMEOUT_MS), global = true)]
    invocation_timeout_ms: Option<u64>,
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
    /// Named timeout defaults. `ci` sets `--timeout 0 --lock-timeout 0`.
    #[arg(long, value_enum, global = true)]
    profile: Option<InvocationProfile>,
}

impl Default for InvocationArgs {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT_SECONDS,
            invocation_timeout_ms: None,
            lock_timeout: DEFAULT_TIMEOUT_SECONDS,
            fail_on_lock: false,
            profile: None,
        }
    }
}

impl InvocationArgs {
    pub fn options(self) -> InvocationOptions {
        let (timeout, lock_timeout) = match self.profile {
            Some(InvocationProfile::Ci) => (0, 0),
            None => (self.timeout, self.lock_timeout),
        };
        InvocationOptions {
            timeout: nonzero_seconds(timeout),
            invocation_timeout: self
                .profile
                .is_none()
                .then_some(self.invocation_timeout_ms)
                .flatten()
                .map(Duration::from_millis),
            lock_timeout: nonzero_seconds(lock_timeout),
            fail_on_lock: self.fail_on_lock,
            jobs: None,
        }
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
        Self::acquire_resolving_path(options, forward_parent_signals, lock_path)
    }

    #[cfg(test)]
    fn acquire_at_path(
        options: InvocationOptions,
        path: &Path,
        forward_parent_signals: bool,
    ) -> Result<Self> {
        Self::acquire_resolving_path(options, forward_parent_signals, || Ok(path.to_path_buf()))
    }

    fn acquire_resolving_path(
        options: InvocationOptions,
        forward_parent_signals: bool,
        resolve_path: impl FnOnce() -> Result<std::path::PathBuf>,
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
            Some(DeadlineGuard::install_for_cli_with_deadline(
                None,
                invocation_deadline,
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
        let lock_timeout = match (invocation_deadline, options.lock_timeout) {
            (Some(deadline), Some(lock)) => Some(
                deadline
                    .expires_at
                    .saturating_duration_since(clock::now())
                    .min(lock),
            ),
            (Some(deadline), None) => {
                Some(deadline.expires_at.saturating_duration_since(clock::now()))
            }
            (None, lock) => lock,
        };
        let lock = match acquire_lock(&path, lock_timeout, options.fail_on_lock) {
            Err(error)
                if invocation_deadline
                    .is_some_and(|deadline| deadline.expires_at <= clock::now())
                    && error
                        .downcast_ref::<InvocationError>()
                        .is_some_and(|error| error.kind() == InvocationErrorKind::LockTimeout) =>
            {
                let timeout = invocation_deadline
                    .expect("expired invocation deadline")
                    .timeout;
                return Err(InvocationError::new(
                    InvocationErrorKind::CommandTimeout,
                    deadline::timeout_diagnostic(timeout, "invocation.lock"),
                )
                .into());
            }
            result => result?,
        };
        // Keep the original absolute invocation deadline. Cancel the acquisition
        // watcher before arming the earlier of that deadline and command timeout.
        drop(early_deadline);
        if forward_parent_signals && invocation_deadline.is_some() {
            set_timeout_phase("command");
        }
        let deadline = match (forward_parent_signals, invocation_deadline) {
            (true, None) => DeadlineGuard::install_for_cli(options.timeout)?,
            (false, None) => DeadlineGuard::install_for_invocation(options.timeout, None)?,
            (true, limit) => DeadlineGuard::install_for_cli_with_deadline(options.timeout, limit)?,
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
#[path = "invocation/tests.rs"]
mod tests;
