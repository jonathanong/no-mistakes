use super::{active_deadline, current_phase, timeout_diagnostic};
#[path = "output.rs"]
mod output;
use crate::invocation::clock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) fn watch_deadline(
    cancel: Arc<AtomicBool>,
    expires_at: Instant,
    timeout: Duration,
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    output: crate::invocation::CliErrorOutput,
) {
    loop {
        if cancel.load(Ordering::Acquire) {
            return;
        }
        let now = clock::now();
        if now >= expires_at {
            if cancel.load(Ordering::Acquire) || !deadline_still_active(expires_at) {
                return;
            }
            let phase = current_phase();
            let message = timeout_diagnostic(timeout, phase);
            {
                let error = crate::invocation::InvocationError::new(
                    crate::invocation::InvocationErrorKind::CommandTimeout,
                    message.clone(),
                )
                .with_phase(phase)
                .into();
                if let Some(report) = output.report_or_wait(&error) {
                    output::emit(format!("{report}\n"), output.clone());
                }
            }
            crate::diagnostics::render_timeout_stderr(observer, message);
            crate::invocation::child::process_tree::terminate_registered_groups();
            std::process::exit(124);
        }
        std::thread::sleep(
            expires_at
                .saturating_duration_since(now)
                .min(Duration::from_millis(200)),
        );
    }
}

fn deadline_still_active(expires_at: Instant) -> bool {
    active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_some_and(|deadline| !deadline.committed && deadline.expires_at == expires_at)
}
