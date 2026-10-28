//! Process-wide invocation coordination shared by the CLI and N-API entrypoints.

#[path = "invocation/acquisition.rs"]
mod acquisition;
#[path = "invocation/child.rs"]
mod child;
#[path = "invocation/clock.rs"]
mod clock;
#[path = "invocation/deadline.rs"]
mod deadline;
#[path = "invocation/error.rs"]
mod error;
#[path = "invocation/holder.rs"]
mod holder;
#[path = "invocation/lock.rs"]
mod lock;
#[path = "invocation/lock_status.rs"]
mod lock_status;
#[path = "invocation/napi_options.rs"]
mod napi_options;

pub use child::command_output;
pub(crate) use child::stream::stream_command_lines;
pub use deadline::{check_timeout, commit_timeout, set_timeout_phase};
pub use error::{
    error_report, exit_code, timeout_exit_code, InvocationError, InvocationErrorKind,
    EXIT_TEMPFAIL, EXIT_TIMEOUT,
};
pub use holder::LockHolder;
pub use lock_status::{lock_status, LockState, LockStatus};
pub use napi_options::{extract_napi_options, extract_napi_options_value, InvocationOptions};

use deadline::DeadlineGuard;
use std::fs::File;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const MAX_INVOCATION_TIMEOUT_MS: u64 = (1_u64 << 53) - 1;

// Windows native CI compiles this helper into the ordinary rlib so the Job
// Object integration test can install a deadline without `--cfg test`.
#[cfg(any(test, feature = "test-instrumentation", windows))]
#[doc(hidden)]
pub fn install_test_deadline(timeout: Duration) -> anyhow::Result<impl Drop> {
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

pub struct InvocationGuard {
    _deadline: DeadlineGuard,
    _parent_signals: child::ParentSignalForwardingGuard,
    _lock: File,
}

pub(super) fn nonzero_seconds(seconds: u64) -> Option<Duration> {
    (seconds != 0).then(|| Duration::from_secs(seconds))
}

#[cfg(test)]
#[path = "invocation/tests.rs"]
mod tests;
