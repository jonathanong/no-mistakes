//! Run the shared invocation unit tests on Windows.

#[cfg(windows)]
pub use no_mistakes_invocation::hooks;

#[cfg(windows)]
#[expect(
    dead_code,
    unused_imports,
    reason = "The native harness compiles production invocation modules while exercising only platform-specific adapters."
)]
#[path = "../src/invocation.rs"]
mod invocation;

#[cfg(windows)]
#[path = "../src/test_support.rs"]
mod test_support;
