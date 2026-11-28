//! Run CLI integration checks through the already-built executable.
#![cfg(any(windows, target_os = "macos"))]

#[cfg_attr(windows, allow(unused_imports, reason = "Unix uses the pipe helpers."))]
mod cli_invocation;

#[cfg(unix)]
#[path = "../../no-mistakes/tests/common/saved_fixture.rs"]
mod saved_fixture;
