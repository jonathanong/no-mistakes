//! Run the existing invocation unit tests on macOS without compiling every
//! library test into one large `--lib` harness. The ordinary library build and
//! Linux coverage job still compile the full crate test suite.

#![cfg(target_os = "macos")]

// The included module uses the production diagnostics API through `crate::`.
// Re-exporting it here preserves those paths while keeping the exact unit-test
// sources, including their private-process and progress assertions.
pub use no_mistakes::diagnostics;

#[allow(dead_code, unused_imports)]
#[path = "../src/invocation.rs"]
mod invocation;
