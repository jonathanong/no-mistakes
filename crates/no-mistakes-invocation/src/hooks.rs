//! Inverted diagnostics hooks for `no-mistakes-invocation`.

use std::fmt::Arguments;
use std::sync::{Arc, OnceLock};

pub trait DiagnosticsObserver: Send + Sync {
    fn progress_enabled(&self) -> bool;
    fn render_timeout_stderr(self: Arc<Self>, message: String);
    fn render_progress_signal_stderr(&self);
}

pub trait DiagnosticsHooks: Send + Sync {
    fn current_observer(&self) -> Option<Arc<dyn DiagnosticsObserver>>;
    fn print_stderr(&self, message: Arguments<'_>);
    fn render_timeout_stderr(
        &self,
        observer: Option<Arc<dyn DiagnosticsObserver>>,
        message: String,
    );
}

static REGISTERED_HOOKS: OnceLock<&'static dyn DiagnosticsHooks> = OnceLock::new();

pub fn register_diagnostics_hooks(hooks: &'static dyn DiagnosticsHooks) {
    let _ = REGISTERED_HOOKS.set(hooks);
}

pub fn current_observer() -> Option<Arc<dyn DiagnosticsObserver>> {
    current_observer_from(REGISTERED_HOOKS.get().copied())
}

pub fn print_stderr(message: Arguments<'_>) {
    print_stderr_with(REGISTERED_HOOKS.get().copied(), message);
}

pub fn render_timeout_stderr(observer: Option<Arc<dyn DiagnosticsObserver>>, message: String) {
    render_timeout_stderr_with(REGISTERED_HOOKS.get().copied(), observer, message);
}

fn current_observer_from(
    hooks: Option<&dyn DiagnosticsHooks>,
) -> Option<Arc<dyn DiagnosticsObserver>> {
    hooks.and_then(DiagnosticsHooks::current_observer)
}

fn print_stderr_with(hooks: Option<&dyn DiagnosticsHooks>, message: Arguments<'_>) {
    match hooks {
        Some(hooks) => hooks.print_stderr(message),
        None => eprintln!("{message}"),
    }
}

fn render_timeout_stderr_with(
    hooks: Option<&dyn DiagnosticsHooks>,
    observer: Option<Arc<dyn DiagnosticsObserver>>,
    message: String,
) {
    match hooks {
        Some(hooks) => hooks.render_timeout_stderr(observer, message),
        None => match observer {
            Some(observer) => observer.render_timeout_stderr(message),
            None => eprintln!("error: {message}"),
        },
    }
}

#[cfg(test)]
mod tests;
