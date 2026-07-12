use super::*;
use std::time::Duration;

// A single test drives the whole enabled/disabled sequence: `ENABLED` is one
// process-wide static, so separate #[test] fns toggling it would race under
// cargo test's default parallel execution.
#[test]
fn enabled_reflects_set_enabled_and_trace_always_returns_its_result() {
    set_enabled(false);
    assert!(!enabled());
    assert_eq!(trace("disabled", || 1 + 1), 2);

    set_enabled(true);
    assert!(enabled());
    assert_eq!(trace("enabled", || "value"), "value");

    // record() must not panic in either state; its only effect is an
    // eprintln!, which isn't asserted on here (see the cli_extra.rs
    // integration test for stderr-content coverage).
    record("direct-call", Duration::from_millis(1));

    set_enabled(false);
    assert!(!enabled());
}
