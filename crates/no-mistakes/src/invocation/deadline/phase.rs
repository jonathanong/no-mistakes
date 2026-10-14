use std::sync::{OnceLock, RwLock};
use std::time::Duration;

fn phase_slot() -> &'static RwLock<&'static str> {
    static PHASE: OnceLock<RwLock<&'static str>> = OnceLock::new();
    PHASE.get_or_init(|| RwLock::new("command"))
}

/// Record the command phase included in a timeout diagnostic.
pub fn set_timeout_phase(phase: &'static str) {
    *phase_slot()
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = phase;
}

pub(super) fn current_phase() -> &'static str {
    *phase_slot()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(in crate::invocation) fn timeout_diagnostic(timeout: Duration, phase: &str) -> String {
    if timeout.subsec_nanos() == 0 {
        format!(
            "command timed out after {} seconds during {phase}",
            timeout.as_secs()
        )
    } else {
        format!(
            "command timed out after {} milliseconds during {phase}",
            timeout.as_millis()
        )
    }
}
