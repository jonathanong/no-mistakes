//! Run the invocation unit and CLI tests on macOS from `no-mistakes-invocation`
//! without compiling `no-mistakes`.

#[cfg(unix)]
pub use no_mistakes_invocation::hooks;

#[cfg(unix)]
#[expect(
    dead_code,
    unused_imports,
    reason = "The native harness compiles production invocation modules while exercising only platform-specific adapters."
)]
#[path = "../src/invocation.rs"]
mod invocation;

#[cfg(unix)]
#[path = "../../no-mistakes/tests/cli_invocation.rs"]
mod cli_invocation;

#[cfg(unix)]
#[path = "../../no-mistakes/tests/common/saved_fixture.rs"]
mod saved_fixture;

#[cfg(unix)]
mod test_support {
    pub(crate) use super::saved_fixture::materialize_path as materialize_saved_fixture;
}
