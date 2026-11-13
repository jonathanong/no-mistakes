#![expect(clippy::disallowed_methods, reason = "This module owns its observer control clock.")]

#[expect(dead_code, reason = "A reviewed expectation remains visible and self-checking.")]
fn deliberately_unused() {}
