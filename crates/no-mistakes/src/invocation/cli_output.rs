//! One request owns the right to publish its invocation failure as JSON.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const OUTPUT_WAIT: Duration = Duration::from_millis(25);

#[derive(Default)]
struct Claim {
    claimed: AtomicBool,
    completed: Mutex<bool>,
    ready: Condvar,
}

#[doc(hidden)]
#[derive(Clone, Default)]
pub struct CliErrorOutput(Option<Arc<Claim>>);

impl CliErrorOutput {
    pub fn new(json: bool) -> Self {
        Self(json.then(|| Arc::new(Claim::default())))
    }

    /// Claim a report without waiting; completion-aware callers use report_or_wait.
    pub fn report(&self, error: &anyhow::Error) -> Option<serde_json::Value> {
        self.report_with_wait(error, false)
    }

    /// Give either winning writer a bounded chance to finish before process exit.
    pub fn report_or_wait(&self, error: &anyhow::Error) -> Option<serde_json::Value> {
        self.report_with_wait(error, true)
    }

    fn report_with_wait(&self, error: &anyhow::Error, wait: bool) -> Option<serde_json::Value> {
        let claim = self.0.as_ref()?;
        let report = super::error_report(error)?;
        if claim
            .claimed
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return Some(report);
        }
        if wait {
            let completed = claim
                .completed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // No completion lock is held during output. A stalled or unstarted
            // writer cannot extend the boundary's grace period.
            drop(
                claim
                    .ready
                    .wait_timeout_while(completed, OUTPUT_WAIT, |completed| !*completed),
            );
        }
        None
    }

    #[doc(hidden)]
    pub fn complete(&self) {
        if let Some(claim) = &self.0 {
            *claim
                .completed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
            claim.ready.notify_all();
        }
    }
}

#[cfg(test)]
#[path = "cli_output/tests.rs"]
mod tests;
