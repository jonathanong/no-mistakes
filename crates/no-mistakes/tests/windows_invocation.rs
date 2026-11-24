//! Run the CLI invocation tests that are not Unix-only on Windows without
//! compiling the full `suite_cli_queries` harness. Linux coverage CI still
//! runs them in that suite.

#![cfg(windows)]

#[allow(
    dead_code,
    unused_imports,
    reason = "The shared invocation tests keep Unix-only helpers and imports beside the portable cases."
)]
#[path = "cli_invocation.rs"]
mod cli_invocation;
