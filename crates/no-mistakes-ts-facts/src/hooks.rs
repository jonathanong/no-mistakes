//! Inverted diagnostics hooks for `no-mistakes-ts-facts`.

use std::path::Path;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilePass {
    Parse,
    ValueFlowExtract,
    ValueFlowFinalize,
}

pub trait DiagnosticsHooks: Send + Sync {
    fn record_work(&self, metric: &'static str, count: u64);
    fn trace_file(&self, pass: FilePass, path: &Path, units: u64, operation: &mut dyn FnMut());
    fn trace_current_file(&self, pass: FilePass, units: u64, operation: &mut dyn FnMut());
}

static REGISTERED_HOOKS: OnceLock<&'static dyn DiagnosticsHooks> = OnceLock::new();

pub fn register_diagnostics_hooks(hooks: &'static dyn DiagnosticsHooks) {
    let _ = REGISTERED_HOOKS.set(hooks);
}

pub fn record_work(metric: &'static str, count: u64) {
    record_work_with(REGISTERED_HOOKS.get().copied(), metric, count);
}

pub fn trace_file<T>(pass: FilePass, path: &Path, units: u64, operation: impl FnOnce() -> T) -> T {
    trace_file_with(
        REGISTERED_HOOKS.get().copied(),
        pass,
        path,
        units,
        operation,
    )
}

pub fn trace_current_file<T>(pass: FilePass, units: u64, operation: impl FnOnce() -> T) -> T {
    trace_current_file_with(REGISTERED_HOOKS.get().copied(), pass, units, operation)
}

fn record_work_with(hooks: Option<&dyn DiagnosticsHooks>, metric: &'static str, count: u64) {
    if let Some(hooks) = hooks {
        hooks.record_work(metric, count);
    }
}

fn run_hook<T>(operation: impl FnOnce() -> T, invoke: impl FnOnce(&mut dyn FnMut())) -> T {
    let mut slot = Some(operation);
    let mut result = None;
    invoke(&mut || {
        let op = slot
            .take()
            .expect("diagnostics hook must invoke operation once");
        result = Some(op());
    });
    result.expect("diagnostics hook must invoke operation")
}

fn trace_file_with<T>(
    hooks: Option<&dyn DiagnosticsHooks>,
    pass: FilePass,
    path: &Path,
    units: u64,
    operation: impl FnOnce() -> T,
) -> T {
    match hooks {
        Some(hooks) => run_hook(operation, |cb| hooks.trace_file(pass, path, units, cb)),
        None => operation(),
    }
}

fn trace_current_file_with<T>(
    hooks: Option<&dyn DiagnosticsHooks>,
    pass: FilePass,
    units: u64,
    operation: impl FnOnce() -> T,
) -> T {
    match hooks {
        Some(hooks) => run_hook(operation, |cb| hooks.trace_current_file(pass, units, cb)),
        None => operation(),
    }
}

#[cfg(test)]
mod tests;
