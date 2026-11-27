use super::{current, print_stderr, timeout, InvocationObserver};
use std::fmt::Arguments;
use std::sync::Arc;

struct NoMistakesDiagnosticsHooks;

impl no_mistakes_invocation::DiagnosticsObserver for InvocationObserver {
    fn progress_enabled(&self) -> bool {
        InvocationObserver::progress_enabled(self)
    }

    fn render_timeout_stderr(self: Arc<Self>, message: String) {
        timeout::render_timeout_stderr(Some(self), message);
    }

    fn render_progress_signal_stderr(&self) {
        InvocationObserver::render_progress_signal_stderr(self);
    }
}

impl no_mistakes_invocation::DiagnosticsHooks for NoMistakesDiagnosticsHooks {
    fn current_observer(&self) -> Option<Arc<dyn no_mistakes_invocation::DiagnosticsObserver>> {
        current().map(|observer| observer as Arc<dyn no_mistakes_invocation::DiagnosticsObserver>)
    }

    fn print_stderr(&self, message: Arguments<'_>) {
        print_stderr(message);
    }

    fn render_timeout_stderr(
        &self,
        observer: Option<Arc<dyn no_mistakes_invocation::DiagnosticsObserver>>,
        message: String,
    ) {
        match observer {
            Some(observer) => observer.render_timeout_stderr(message),
            None => timeout::render_timeout_stderr(None, message),
        }
    }
}

static DIAGNOSTICS_HOOKS: NoMistakesDiagnosticsHooks = NoMistakesDiagnosticsHooks;

pub(crate) fn ensure_invocation_hooks_registered() {
    no_mistakes_invocation::register_diagnostics_hooks(&DIAGNOSTICS_HOOKS);
}
