//! Process-wide invocation locking, deadlines, watchdogs, and child process-tree control.

pub mod hooks;
mod invocation;

pub use hooks::{register_diagnostics_hooks, DiagnosticsHooks, DiagnosticsObserver};
pub use invocation::*;

#[cfg(test)]
mod test_support;
