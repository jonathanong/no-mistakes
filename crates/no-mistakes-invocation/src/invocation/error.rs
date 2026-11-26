//! Invocation failures and the exit-code and JSON contract that describes them.
use super::LockHolder;
use serde_json::{json, Value};

/// Analysis deadline (`--timeout`, `--invocation-timeout-ms`), matching `timeout(1)`.
pub const EXIT_TIMEOUT: u8 = 124;
/// Lock contention: EX_TEMPFAIL from sysexits.h. Retrying later may succeed.
pub const EXIT_TEMPFAIL: u8 = 75;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationErrorKind {
    LockBusy,
    LockTimeout,
    CommandTimeout,
}

impl InvocationErrorKind {
    /// Stable machine-readable name used in JSON errors.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LockBusy => "lock_busy",
            Self::LockTimeout => "lock_timeout",
            Self::CommandTimeout => "deadline",
        }
    }
}

#[derive(Debug)]
pub struct InvocationError {
    kind: InvocationErrorKind,
    message: String,
    phase: Option<String>,
    holder: Option<LockHolder>,
}

impl InvocationError {
    pub(super) fn new(kind: InvocationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            phase: None,
            holder: None,
        }
    }

    pub(super) fn with_phase(mut self, phase: &str) -> Self {
        self.phase = Some(phase.to_string());
        self
    }

    pub(super) fn with_holder(mut self, holder: Option<LockHolder>) -> Self {
        self.holder = holder;
        self
    }

    pub fn kind(&self) -> InvocationErrorKind {
        self.kind
    }

    pub fn phase(&self) -> Option<&str> {
        self.phase.as_deref()
    }

    pub fn holder(&self) -> Option<&LockHolder> {
        self.holder.as_ref()
    }
}

impl std::fmt::Display for InvocationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for InvocationError {}

fn invocation_error(error: &anyhow::Error) -> Option<&InvocationError> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<InvocationError>())
}

/// Exit 124 when the analysis deadline elapsed; lock failures are not deadlines.
pub fn timeout_exit_code(error: &anyhow::Error) -> Option<u8> {
    error.chain().find_map(|cause| {
        let io_timeout = cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|error| error.kind() == std::io::ErrorKind::TimedOut);
        let deadline = cause
            .downcast_ref::<InvocationError>()
            .is_some_and(|error| error.kind() == InvocationErrorKind::CommandTimeout);
        (io_timeout || deadline).then_some(EXIT_TIMEOUT)
    })
}

/// The CLI exit status for a failed invocation: 75 lock contention, 124 deadline, else 2.
pub fn exit_code(error: &anyhow::Error) -> u8 {
    let lock = invocation_error(error).is_some_and(|error| {
        matches!(
            error.kind(),
            InvocationErrorKind::LockBusy | InvocationErrorKind::LockTimeout
        )
    });
    match (lock, timeout_exit_code(error)) {
        (true, _) => EXIT_TEMPFAIL,
        (_, Some(code)) => code,
        _ => 2,
    }
}

/// Structured `{"error": {...}}` document for invocation failures, or `None`
/// for any other error.
pub fn error_report(error: &anyhow::Error) -> Option<Value> {
    let found = invocation_error(error);
    let kind = match found {
        Some(found) => found.kind(),
        None => timeout_exit_code(error).map(|_| InvocationErrorKind::CommandTimeout)?,
    };
    let mut body = json!({
        "kind": kind.as_str(),
        "phase": found.and_then(InvocationError::phase),
        "message": format!("{error:#}"),
        "exitCode": exit_code(error),
    });
    if let Some(holder) = found.and_then(InvocationError::holder) {
        body["holder"] = serde_json::to_value(holder).ok()?;
    }
    Some(json!({ "error": body }))
}
