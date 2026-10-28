//! One request owns the right to publish its invocation failure as JSON.
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const OUTPUT_WAIT: Duration = Duration::from_millis(25);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum State {
    #[default]
    Fresh,
    InFlight,
    Complete,
}

#[derive(Default)]
struct Claim {
    state: Mutex<State>,
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
        let mut state = claim
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if wait && *state == State::InFlight {
            state = claim
                .ready
                .wait_timeout_while(state, OUTPUT_WAIT, |state| *state == State::InFlight)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
        if *state == State::Fresh {
            *state = State::InFlight;
            return Some(report);
        }
        None
    }

    #[doc(hidden)]
    pub fn complete(&self) {
        if let Some(claim) = &self.0 {
            *claim
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = State::Complete;
            claim.ready.notify_all();
        }
    }

    /// A failed spawn wrote nothing. Wake a boundary already waiting on this
    /// claim, then give its retry a bounded chance to publish before hard exit.
    pub(super) fn release_and_wait(&self) {
        let Some(claim) = &self.0 else { return };
        let mut state = claim
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *state != State::InFlight {
            return;
        }
        *state = State::Fresh;
        claim.ready.notify_all();
        // No claim lock is held during I/O, including an ordinary boundary retry.
        drop(
            claim
                .ready
                .wait_timeout_while(state, OUTPUT_WAIT, |state| *state != State::Complete),
        );
    }
}

#[cfg(test)]
#[path = "cli_output/tests.rs"]
mod tests;
