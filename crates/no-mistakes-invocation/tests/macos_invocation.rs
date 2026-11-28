//! Run the shared invocation unit tests on the native Unix platform.

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
#[path = "../src/test_support.rs"]
mod test_support;
