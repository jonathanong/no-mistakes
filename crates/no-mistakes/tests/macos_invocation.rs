//! Run the existing invocation unit tests on macOS without compiling every
//! library test into one large `--lib` harness. Linux coverage still compiles
//! and runs the full suite.

// The included module uses the production diagnostics API through `crate::`.
// Re-exporting it here preserves those paths while keeping the exact unit-test
// sources, including their private-process and progress assertions.
pub use no_mistakes::diagnostics;

// Compile this API boundary on every host: the copied watchdog must be able to
// call its diagnostics bridge through the external-library re-export.
const _: fn(Option<std::sync::Arc<diagnostics::InvocationObserver>>, String) =
    diagnostics::render_timeout_stderr;
const _: fn(&diagnostics::InvocationObserver) =
    diagnostics::InvocationObserver::render_progress_signal_stderr;

#[cfg(target_os = "macos")]
#[allow(dead_code, unused_imports)]
#[path = "../src/invocation.rs"]
mod invocation;

// Keep CLI-level lock/timeout regressions in the native macOS CI harness too;
// the full suite_cli_queries harness otherwise runs only on Linux coverage CI.
// fsmonitor loads this helper through `crate::saved_fixture`, so the macOS
// harness must declare the same module the Linux queries suite declares.
#[cfg(target_os = "macos")]
#[path = "cli_invocation.rs"]
mod cli_invocation;
#[cfg(target_os = "macos")]
#[allow(dead_code)]
#[path = "common/saved_fixture.rs"]
mod saved_fixture;
