//! Inverted diagnostics hooks for `no-mistakes-invocation`.

use std::fmt::Arguments;
use std::io::{self, Write};
use std::sync::{mpsc, Arc, OnceLock};
use std::time::Duration;

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
            None => write_bounded_stderr(format!("error: {message}\n")),
        },
    }
}

fn write_bounded_stderr(text: String) {
    let (completed, receiver) = mpsc::channel();
    let _ = std::thread::Builder::new()
        .name("no-mistakes-timeout-stderr".to_string())
        .spawn(move || {
            if let Ok(mut writer) = duplicate_stderr() {
                let _ = writer.write_all(text.as_bytes());
                let _ = writer.flush();
            }
            let _ = completed.send(());
        });
    let _ = receiver.recv_timeout(Duration::from_millis(25));
}

#[cfg(unix)]
fn duplicate_stderr() -> io::Result<std::fs::File> {
    use std::os::fd::AsFd;
    std::io::stderr()
        .as_fd()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}

#[cfg(windows)]
fn duplicate_stderr() -> io::Result<std::fs::File> {
    use std::os::windows::io::AsHandle;
    std::io::stderr()
        .as_handle()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}

#[cfg(not(any(unix, windows)))]
fn duplicate_stderr() -> io::Result<impl Write> {
    Ok(std::io::stderr())
}

#[cfg(test)]
mod tests;
